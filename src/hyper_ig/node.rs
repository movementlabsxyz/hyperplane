use std::collections::{HashMap, HashSet};
use crate::types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited, CATId, SubBlock, CATStatusUpdate, CLTransactionId};
use super::{HyperIG, HyperIGError};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use std::time::Duration;
use crate::types::ChainId;
use crate::types::communication::cl_to_hig::{STATUS_UPDATE_PATTERN};
use crate::utils::logging::log;
use crate::mock_vm::MockVM;
use x_chain_vm::transaction::Transaction as VMTransaction;
use x_chain_vm::transaction::TxSet1;

//==============================================================================
// State Management
//==============================================================================

/// The internal state of the HyperIGNode
struct HyperIGState {
    /// Map of transaction IDs to their original transactions
    received_txs: HashMap<TransactionId, Transaction>,
    /// Map of transaction IDs to their current status
    transaction_statuses: HashMap<TransactionId, TransactionStatus>,
    /// Set of pending transaction IDs
    pending_transactions: HashSet<TransactionId>,
    /// Proposed status for CAT transactions
    cat_proposed_statuses: HashMap<TransactionId, CATStatusLimited>,
    /// Mapping from CAT IDs to transaction IDs
    cat_to_tx_id: HashMap<CATId, TransactionId>,
    /// Map of locked keys to the CAT transaction ID that locked them
    key_locked_by_tx: HashMap<String, TransactionId>,
    /// Map of keys to transactions waiting on them
    key_causes_dependencies_for_txs: HashMap<String, Vec<TransactionId>>,
    /// Map of transaction IDs to the transaction IDs they depend on
    tx_depends_on_txs: HashMap<TransactionId, HashSet<TransactionId>>,
    /// my chain id
    my_chain_id: ChainId,
    /// Mock VM for transaction execution
    vm: MockVM,
}

/// Node implementation of the Hyper Information Gateway
pub struct HyperIGNode {
    /// The internal state of the node
    state: Arc<Mutex<HyperIGState>>,
    /// Receiver for messages from Confirmation Layer
    receiver_cl_to_hig: Option<mpsc::Receiver<SubBlock>>,
    /// Sender for messages to Hyper Scheduler
    sender_hig_to_hs: Option<mpsc::Sender<CATStatusUpdate>>,
}

//==============================================================================
// Node Initialization and Lifecycle
//==============================================================================

impl HyperIGNode {
    /// Creates a new HyperIGNode instance.
    /// 
    /// # Arguments
    /// * `receiver_cl_to_hig` - Channel receiver for messages from Confirmation Layer
    /// * `sender_hig_to_hs` - Channel sender for messages to Hyper Scheduler
    /// * `my_chain_id` - The chain ID this node is responsible for
    /// 
    /// # Returns
    /// A new HyperIGNode instance
    pub fn new(receiver_cl_to_hig: mpsc::Receiver<SubBlock>, sender_hig_to_hs: mpsc::Sender<CATStatusUpdate>, my_chain_id: ChainId) -> Self {
        Self {
            state: Arc::new(Mutex::new(HyperIGState {
                transaction_statuses: HashMap::new(),
                pending_transactions: HashSet::new(),
                cat_proposed_statuses: HashMap::new(),
                cat_to_tx_id: HashMap::new(),
                key_locked_by_tx: HashMap::new(),
                key_causes_dependencies_for_txs: HashMap::new(),
                tx_depends_on_txs: HashMap::new(),
                received_txs: HashMap::new(),
                my_chain_id: my_chain_id.clone(),
                vm: MockVM::new(),
            })),
            receiver_cl_to_hig: Some(receiver_cl_to_hig),
            sender_hig_to_hs: Some(sender_hig_to_hs),
        }
    }

    /// Starts the node's block processing loop.
    /// 
    /// This function spawns a new task that continuously processes incoming messages
    /// from the Confirmation Layer.
    /// 
    /// # Arguments
    /// * `node` - An Arc<Mutex<HyperIGNode>> containing the node instance
    pub async fn start(node: Arc<Mutex<HyperIGNode>>) {
        let chain_id = node.lock().await.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), "Starting HyperIG node");
        tokio::spawn(async move { HyperIGNode::process_messages(node).await.unwrap() });
    }

    /// Processes incoming messages from the Confirmation Layer.
    /// 
    /// This function runs in a loop, continuously checking for new messages
    /// and processing them as they arrive.
    /// 
    /// # Arguments
    /// * `hig_node` - An Arc<Mutex<HyperIGNode>> containing the node instance
    /// 
    /// # Returns
    /// Result indicating success or failure of the message processing loop
    pub async fn process_messages(hig_node: Arc<Mutex<HyperIGNode>>) -> Result<(), HyperIGError> {
        let chain_id = hig_node.lock().await.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), "Starting message processing loop");
        loop {
            let mut node = hig_node.lock().await;

            // Get the receiver from the node
            let receiver = if let Some(receiver) = &mut node.receiver_cl_to_hig {
                receiver
            } else {
                log(&format!("HIG-{}", chain_id), "No receiver available, exiting loop");
                break;
            };

            // Try to receive a message
            match receiver.try_recv() {
                Ok(subblock) => {
                    // Process the subblock
                    if let Err(e) = node.process_subblock(subblock).await {
                        log(&format!("HIG-{}", chain_id), &format!("Error processing subblock: {}", e));
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // No message available, release the lock and wait a bit
                    drop(node);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    log(&format!("HIG-{}", chain_id), "Channel disconnected, exiting loop");
                    break;
                }
            }
        }
        log(&format!("HIG-{}", chain_id), "Message processing loop exiting");
        Ok(())
    }

    /// Gets the keys accessed by a transaction.
    /// 
    /// # Arguments
    /// * `command` - The transaction command to analyze
    /// 
    /// # Returns
    /// Result containing a vector of keys accessed by the transaction
    async fn get_transaction_keys(&self, command: &str) -> Result<Vec<String>, anyhow::Error> {
        // Parse the transaction using x-chain-vm's parse_input
        let vm_tx = x_chain_vm::parse_input(command)
            .map_err(|e| anyhow::anyhow!("Failed to parse transaction: {}", e))?;
        
        // Get the keys accessed by the transaction
        let keys = match vm_tx {
            TxSet1::Credit { receiver, amount: _ } => vec![receiver.to_string()],
            TxSet1::Send { sender, receiver, amount: _ } => vec![sender.to_string(), receiver.to_string()],
            TxSet1::Skip | TxSet1::Help | TxSet1::Status => vec![],
        };
        
        Ok(keys)
    }

    /// Checks if any keys accessed by a transaction are locked.
    /// 
    /// # Arguments
    /// * `keys` - The keys to check
    /// 
    /// # Returns
    /// Result containing the transaction ID that locked any of the keys, if any
    async fn check_locked_keys(&self, keys: &[String]) -> Result<Option<TransactionId>, anyhow::Error> {
        let state = self.state.lock().await;
        for key in keys {
            if let Some(tx_id) = state.key_locked_by_tx.get(key) {
                return Ok(Some(tx_id.clone()));
            }
        }
        Ok(None)
    }

    /// Adds a transaction to the dependency list for each key it accesses.
    /// 
    /// # Arguments
    /// * `tx_id` - The transaction ID to add
    /// * `keys` - The keys the transaction accesses
    async fn add_transaction_dependencies(&self, tx_id: TransactionId, keys: &[String]) {
        let mut state = self.state.lock().await;
        let chain_id = state.my_chain_id.0.clone();
        let tx_id_clone = tx_id.clone();
        log(&format!("HIG-{}", chain_id), &format!("Adding dependencies for tx-id='{}' with keys: {:?}", tx_id_clone.0, keys));
        
        // First, collect all locking transaction IDs
        let locking_tx_ids: Vec<(String, TransactionId)> = keys.iter()
            .filter_map(|key| {
                state.key_locked_by_tx.get(key)
                    .map(|locking_tx_id| (key.clone(), locking_tx_id.clone()))
            })
            .collect();
        
        // Add transaction to the dependency list for each key
        for key in keys {
            state.key_causes_dependencies_for_txs
                .entry(key.clone())
                .or_insert_with(Vec::new)
                .push(tx_id.clone());
            log(&format!("HIG-{}", chain_id), &format!("Added tx-id='{}' to dependencies of key '{}'. (in key_causes_dependencies_for_txs)", tx_id_clone.0, key));
        }
        
        // Add the locking transactions as dependencies
        for (key, locking_tx_id) in locking_tx_ids {
            state.tx_depends_on_txs
                .entry(tx_id.clone())
                .or_insert_with(HashSet::new)
                .insert(locking_tx_id.clone());
            log(&format!("HIG-{}", chain_id), &format!("Added tx-id='{}' as dependency of tx-id='{}' (locked key '{}'). (in tx_depends_on_txs)", 
                locking_tx_id.0, tx_id_clone.0, key));
        }
    }

    /// Checks if a transaction would succeed if executed.
    /// 
    /// Parses and executes the transaction using the mock VM to determine
    /// if it would succeed, without actually applying any changes.
    /// 
    /// # Arguments
    /// * `command` - The transaction command to check
    /// 
    /// # Returns
    /// Result containing whether the transaction would succeed
    async fn check_transaction_execution(&self, command: &str) -> Result<bool, anyhow::Error> {
        // Parse and execute the transaction using x-chain-vm's parse_input
        let vm_tx = x_chain_vm::parse_input(command)
            .map_err(|e| anyhow::anyhow!("Failed to parse transaction: {}", e))?;
        
        // Execute the transaction to check if it would succeed
        let execution = vm_tx.execute(&self.state.lock().await.vm.get_state());
        
        Ok(execution.is_success())
    }
}

//==============================================================================
// CAT Transaction Handling
//==============================================================================

impl HyperIGNode {
    /// Handles a CAT (Cross-Chain Atomic Transaction).
    /// 
    /// Marks the transaction as pending, checks if it would succeed if executed,
    /// and stores its proposed status.
    /// 
    /// # Arguments
    /// * `tx` - The CAT transaction to handle
    /// 
    /// # Returns
    /// Result containing the transaction status (always Pending)
    async fn handle_cat_transaction(&mut self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        let chain_id = self.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), &format!("Handling CAT transaction: {}", tx.id.0));
        
        // Store the transaction
        self.state.lock().await.received_txs.insert(tx.id.clone(), tx.clone());
        
        // CAT transactions are always pending
        self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Pending);
        // Add to pending transactions set
        self.state.lock().await.pending_transactions.insert(tx.id.clone());

        // Extract the command part between the dots
        let command = tx.data.split('.').nth(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid transaction format"))?;

        // Get keys accessed by this transaction
        let keys = self.get_transaction_keys(command).await?;

        // Lock all keys accessed by this CAT
        {
            let mut state = self.state.lock().await;
            for key in &keys {
                state.key_locked_by_tx.insert(key.clone(), tx.id.clone());
            }
        }

        // Check if transaction would succeed (but don't execute it)
        let would_succeed = self.check_transaction_execution(command).await?;
        log(&format!("HIG-{}", chain_id), &format!("CAT transaction would {} if executed", 
            if would_succeed { "succeed" } else { "fail" }));
        
        // Store proposed status based on transaction data
        let proposed_status = if would_succeed {
            CATStatusLimited::Success
        } else {
            CATStatusLimited::Failure
        };
        self.state.lock().await.cat_proposed_statuses.insert(tx.id.clone(), proposed_status);

        // Store the mapping from CAT ID to transaction ID
        let cat_id = CATId(tx.cl_id.clone());
        self.state.lock().await.cat_to_tx_id.insert(cat_id, tx.id.clone());
        
        Ok(TransactionStatus::Pending)
    }

    /// Handles a status update transaction.
    /// 
    /// Updates the status of a CAT transaction based on the status update message.
    /// 
    /// # Arguments
    /// * `tx` - The status update transaction to handle
    /// 
    /// # Returns
    /// Result containing the updated transaction status
    async fn handle_status_update(&mut self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        let chain_id = self.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), &format!("Handling status update tx-id='{}' : data='{}'", tx.id.0, tx.data));
        
        // Extract the CAT ID and status from the transaction data using regex
        if !STATUS_UPDATE_PATTERN.is_match(&tx.data) {
            return Err(anyhow::anyhow!("Invalid status update format: {}", tx.data));
        }
        let cat_id = STATUS_UPDATE_PATTERN.captures(&tx.data)
            .and_then(|caps| caps.name("cat_id"))
            .ok_or_else(|| anyhow::anyhow!("Failed to extract CAT ID from status update"))?;
        let cat_id = CATId(CLTransactionId(cat_id.as_str().to_string()));
        
        // Get the transaction ID from the CAT ID mapping
        let tx_id = self.state.lock().await.cat_to_tx_id.get(&cat_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No transaction ID found for CAT ID: {}", cat_id.0))?;
        
        // Has format STATUS_UPDATE:<Status>.CAT_ID:<cat_id>
        let status_part = tx.data.split(".").collect::<Vec<&str>>()[0];
        let status_part = status_part.split(":").collect::<Vec<&str>>()[1];
        log(&format!("HIG-{}", chain_id), &format!("... Extracted status update='{}'", status_part));
        let status = if status_part == "Success" {
            TransactionStatus::Success
        } else if status_part == "Failure" {
            TransactionStatus::Failure
        } else {
            return Err(anyhow::anyhow!("Invalid status in update: {}", status_part));
        };
        log(&format!("HIG-{}", chain_id), &format!("... (Before) status of tx-id='{}': {:?}", tx_id.0, self.state.lock().await.transaction_statuses.get(&tx_id)));
        self.state.lock().await.transaction_statuses.insert(tx_id.clone(), status.clone());
        log(&format!("HIG-{}", chain_id), &format!("Updated status to '{:?}' for tx-id='{}', which is part of CAT-id='{}'", status, tx_id.0, cat_id.0));
        log(&format!("HIG-{}", chain_id), &format!("... (After)  status of tx-id='{}': {:?}", tx_id.0, self.state.lock().await.transaction_statuses.get(&tx_id)));
        
        // If the CAT was successful, execute its transaction
        if status == TransactionStatus::Success {
            let tx_data = self.state.lock().await.received_txs.get(&tx_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Transaction data not found: {}", tx_id))?;
            
            // Extract the command part between the dots
            let command = tx_data.data.split('.').nth(1)
                .ok_or_else(|| anyhow::anyhow!("Invalid transaction format"))?;
            
            // Execute the transaction
            let mut state = self.state.lock().await;
            state.vm.execute_transaction(command)?;
            log(&format!("HIG-{}", chain_id), &format!("Executed CAT transaction tx-id='{}'", tx_id.0));
        }
        
        // Remove from pending transactions if present
        self.state.lock().await.pending_transactions.remove(&tx_id);

        // Process any transactions that were waiting on this CAT
        self.process_pending_transactions(tx_id, status.clone()).await?;
        
        Ok(status)
    }

    /// Processes pending transactions that were waiting on a resolved CAT.
    /// 
    /// # Arguments
    /// * `cat_tx_id` - The transaction ID of the resolved CAT
    /// * `status` - The final status of the CAT
    async fn process_pending_transactions(&mut self, cat_tx_id: TransactionId, _status: TransactionStatus) -> Result<(), anyhow::Error> {
        let chain_id = self.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), &format!("Processing transactions pending on CAT tx-id='{}'", cat_tx_id.0));

        // for now only one transaction can lock a key. TODO : this will change as we add deeper dependencies.
        // Get all keys that were locked by this CAT
        let locked_keys: Vec<String> = {
            let state = self.state.lock().await;
            state.key_locked_by_tx.iter()
                .filter(|(_, tx_id)| **tx_id == cat_tx_id)
                .map(|(key, _)| key.clone())
                .collect()
        };
        log(&format!("HIG-{}", chain_id), &format!("Found locked keys: {:?}", locked_keys));

        // Remove the locks
        {
            let mut state = self.state.lock().await;
            for key in &locked_keys {
                state.key_locked_by_tx.remove(key);
            }
        }
        log(&format!("HIG-{}", chain_id), &format!("Removed locked keys '{:?}' by tx-id='{}' in key_locked_by_tx", locked_keys, cat_tx_id.0));

        // Process all transactions that were waiting on these keys
        for key in locked_keys {
            log(&format!("HIG-{}", chain_id), &format!("Processing transactions waiting on key '{}'", key));
            let pending_txs = {
                let mut state = self.state.lock().await;
                state.key_causes_dependencies_for_txs.remove(&key)
                    .unwrap_or_default()
            };
            log(&format!("HIG-{}", chain_id), &format!("Found pending transactions: {:?}", pending_txs));

            for tx_id in pending_txs {
                log(&format!("HIG-{}", chain_id), &format!("Checking dependencies for tx-id='{}'", tx_id.0));
                
                // First check if we should process this transaction
                let (should_process, remaining_deps) = {
                    let mut state = self.state.lock().await;
                    if let Some(dependencies) = state.tx_depends_on_txs.get_mut(&tx_id) {
                        dependencies.remove(&cat_tx_id);
                        let is_empty = dependencies.is_empty();
                        let deps = dependencies.clone();
                        if is_empty {
                            state.tx_depends_on_txs.remove(&tx_id);
                        }
                        (is_empty, deps)
                    } else {
                        (false, HashSet::new())
                    }
                };

                log(&format!("HIG-{}", chain_id), &format!("Dependencies after removal: {:?}", remaining_deps));

                if should_process {
                    log(&format!("HIG-{}", chain_id), &format!("Will process tx-id='{}'", tx_id.0));
                    // Get transaction from state
                    let tx = {
                        let state = self.state.lock().await;
                        state.received_txs.get(&tx_id)
                            .cloned()
                            .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_id))?
                    };

                    log(&format!("HIG-{}", chain_id), &format!("Processing pending transaction tx-id='{}' (all dependencies resolved)", tx_id.0));
                    self.process_transaction(tx).await?;
                    log(&format!("HIG-{}", chain_id), &format!("Finished processing tx-id='{}'", tx_id.0));
                } else {
                    log(&format!("HIG-{}", chain_id), &format!("Transaction tx-id='{}' still has remaining dependencies", tx_id.0));
                }
            }
        }

        log(&format!("HIG-{}", chain_id), "Finished processing all pending transactions");
        Ok(())
    }

    /// Gets the proposed status for a CAT transaction.
    /// 
    /// # Arguments
    /// * `tx_id` - The ID of the transaction to get the status for
    /// 
    /// # Returns
    /// Result containing the proposed status or an error if not found
    pub async fn get_proposed_status(&self, tx_id: TransactionId) -> Result<CATStatusLimited, anyhow::Error> {
        Ok(self.state.lock().await.cat_proposed_statuses.get(&tx_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No proposed status found for transaction"))?)
    }
}

//==============================================================================
// Regular Transaction Handling
//==============================================================================

impl HyperIGNode {
    /// Handles a regular transaction.
    /// 
    /// Extracts the command from the transaction data, executes it using the mock VM,
    /// and updates the transaction status based on the execution result.
    /// 
    /// # Arguments
    /// * `tx` - The transaction to handle
    /// 
    /// # Returns
    /// Result containing the final transaction status
    async fn handle_regular_transaction(&self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        let chain_id = self.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), &format!("Executing regular transaction: {}", tx.id));
        
        // Store the transaction
        self.state.lock().await.received_txs.insert(tx.id.clone(), tx.clone());
        
        // Extract the command part between the dots
        let command = tx.data.split('.').nth(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid transaction format"))?;
        log(&format!("HIG-{}", chain_id), &format!("Extracted command: {}", command));

        // Get keys accessed by this transaction
        let keys = self.get_transaction_keys(command).await?;
        log(&format!("HIG-{}", chain_id), &format!("Transaction accesses keys: {:?}", keys));

        // Check if any keys are locked
        if let Some(locking_tx_id) = self.check_locked_keys(&keys).await? {
            log(&format!("HIG-{}", chain_id), &format!("Transaction tx-id='{}' is blocked by CAT tx-id='{}'", tx.id.0, locking_tx_id.0));
            
            // Add this transaction to the dependency list for each key
            self.add_transaction_dependencies(tx.id.clone(), &keys).await;
            
            // Mark as pending
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Pending);
            self.state.lock().await.pending_transactions.insert(tx.id.clone());
            
            return Ok(TransactionStatus::Pending);
        }

        // Check if transaction would succeed
        let would_succeed = self.check_transaction_execution(command).await?;
        log(&format!("HIG-{}", chain_id), &format!("Transaction would {} if executed", 
            if would_succeed { "succeed" } else { "fail" }));

        // If it would succeed, execute it
        if would_succeed {
            let mut state = self.state.lock().await;
            log(&format!("HIG-{}", chain_id), "Executing transaction...");
            state.vm.execute_transaction(command)?;
            log(&format!("HIG-{}", chain_id), "Transaction executed successfully");

            // Get the balance for account 1 from the VM state, returns 0 if account doesn't exist
            let balance = TxSet1::Skip.get_value(state.vm.get_state(), 1);
            log(&format!("HIG-{}", chain_id), &format!("Balance of key 1: {}", balance));

            // Update transaction status
            state.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Success);
            log(&format!("HIG-{}", chain_id), &format!("Set final status to 'Success' for transaction: {}", tx.id.0));
            Ok(TransactionStatus::Success)
        } else {
            // Update transaction status
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Failure);
            log(&format!("HIG-{}", chain_id), &format!("Set final status to 'Failure' for transaction: {}", tx.id.0));
            Ok(TransactionStatus::Failure)
        }
    }

}

//==============================================================================
// HyperIG Trait Implementation
//==============================================================================

#[async_trait]
impl HyperIG for HyperIGNode {
    /// Processes a transaction.
    /// 
    /// Handles different types of transactions (regular, CAT, dependent, status update)
    /// and updates their status accordingly.
    /// 
    /// # Arguments
    /// * `tx` - The transaction to process
    /// 
    /// # Returns
    /// Result containing the final transaction status
    async fn process_transaction(&mut self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        let chain_id = self.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), &format!("Processing tx-id='{}' : data='{}'", tx.id, tx.data));

        // handle the case where it is a status update separately
        // because it doesn't need to be inserted into the transaction statuses
        let status = if tx.data.starts_with("STATUS_UPDATE") {
            self.handle_status_update(tx.clone()).await?
        } else {
            // now handle the case where it is any of the other transaction types
            // Store initial status
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Pending);
            log(&format!("HIG-{}", chain_id), &format!("Set initial status to Pending for tx-id: '{}' : data: '{}'", tx.id, tx.data));
            
            let status = if tx.data.starts_with("CAT") {
                self.handle_cat_transaction(tx.clone()).await?
            } else {
                self.handle_regular_transaction(tx.clone()).await?
            };
            
            // Update status
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), status.clone());
            log(&format!("HIG-{}", chain_id), &format!("Updated status to '{:?}' for tx-id='{}'", status, tx.id.0));
            
            status
        };

        // Send status proposal to Hyper Scheduler if it's a CAT transaction
        if tx.data.starts_with("CAT") {
            let cat_id = CATId(tx.cl_id.clone());
            let constituent_chains = tx.constituent_chains.clone();
            
            // Validate constituent chains
            if constituent_chains.len() <= 1 {
                return Err(HyperIGError::InvalidCATConstituentChains(
                    "CAT must have more than one constituent chain".to_string()
                ).into());
            }
            
            // Check if own chain is part of constituent chains
            if !constituent_chains.contains(&self.state.lock().await.my_chain_id) {
                return Err(HyperIGError::InvalidCATConstituentChains(
                    format!("Own chain '{}' is not part of constituent chains", self.state.lock().await.my_chain_id.0)
                ).into());
            }

            // Get the proposed status from cat_proposed_statuses
            let proposed_status = self.state.lock().await.cat_proposed_statuses.get(&tx.id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No proposed status found for CAT transaction"))?;

            log(&format!("HIG-{}", chain_id), &format!("Extracted cat-id='{}', status='{:?}', chains='{:?}'", cat_id.0, proposed_status, constituent_chains));
            log(&format!("HIG-{}", chain_id), &format!("Sending status proposal for cat-id='{}'", cat_id.0));
            self.send_cat_status_proposal(cat_id, proposed_status, constituent_chains).await?;
            log(&format!("HIG-{}", chain_id), "Status proposal sent for CAT transaction.");
        }

        Ok(status)
    }

    /// Gets the current status of a transaction.
    /// 
    /// # Arguments
    /// * `tx_id` - The ID of the transaction to get the status for
    /// 
    /// # Returns
    /// Result containing the transaction status or an error if not found
    async fn get_transaction_status(&self, tx_id: TransactionId) -> Result<TransactionStatus, anyhow::Error> {
        let chain_id = self.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), &format!("Getting status for tx-id='{}'", tx_id));
        let statuses = self.state.lock().await.transaction_statuses.get(&tx_id)
            .cloned()
            .ok_or_else(|| {
                log(&format!("HIG-{}", chain_id), &format!("Transaction not found tx-id='{}'", tx_id));
                anyhow::anyhow!("Transaction not found: {}", tx_id)
            })?;
        log(&format!("HIG-{}", chain_id), &format!("Found status for tx-id='{}': {:?}", tx_id, statuses));
        Ok(statuses)
    }

    /// Gets the list of pending transactions.
    /// 
    /// # Returns
    /// Result containing a vector of pending transaction IDs
    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error> {
        Ok(self.state.lock().await.pending_transactions.iter().cloned().collect())
    }

    /// Sends a CAT status proposal to the Hyper Scheduler.
    /// 
    /// # Arguments
    /// * `cat_id` - The ID of the CAT transaction
    /// * `status` - The proposed status
    /// * `constituent_chains` - The chains involved in the CAT
    /// 
    /// # Returns
    /// Result indicating success or failure of sending the proposal
    async fn send_cat_status_proposal(&mut self, cat_id: CATId, status: CATStatusLimited, constituent_chains: Vec<ChainId>) -> Result<(), HyperIGError> {
        if let Some(sender) = &mut self.sender_hig_to_hs {
            let status_update = CATStatusUpdate {
                cat_id: cat_id.clone(),
                chain_id: self.state.lock().await.my_chain_id.clone(),
                status: status.clone(),
                constituent_chains: constituent_chains.clone(),
            };
            sender.send(status_update).await.map_err(|e| HyperIGError::Communication(e.to_string()))?;
        }
        Ok(())
    }

    /// Gets the resolution status of a transaction.
    /// 
    /// # Arguments
    /// * `id` - The ID of the transaction to get the resolution status for
    /// 
    /// # Returns
    /// Result containing the resolution status or an error if not found
    async fn get_resolution_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError> {
        let statuses = self.state.lock().await.transaction_statuses.get(&id)
            .cloned()
            .ok_or_else(|| HyperIGError::TransactionNotFound(id))?;
        Ok(statuses)
    }

    /// Processes a subblock of transactions.
    /// 
    /// Validates the subblock's chain ID and processes each transaction within it.
    /// 
    /// # Arguments
    /// * `subblock` - The SubBlock containing transactions to process
    /// 
    /// # Returns
    /// Result indicating success or failure of subblock processing
    async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        // check if the received subblock matches our chain id. If not we have a bug.
        let chain_id = self.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), &format!("Processing subblock: block_id={}, chain_id={}, tx_count={}", 
        subblock.block_height, subblock.chain_id.0, subblock.transactions.len()));
        
        if subblock.chain_id.0 != self.state.lock().await.my_chain_id.0 {
            log(&format!("HIG-{}", chain_id), &format!("[ERROR] Received subblock with chain_id='{}', but should be '{}', ignoring", 
                subblock.chain_id.0, self.state.lock().await.my_chain_id.0));
            return Err(HyperIGError::WrongChainId { 
                expected: self.state.lock().await.my_chain_id.clone(),
                received: subblock.chain_id.clone(),
            });
        }

        // Track seen transaction IDs to skip duplicates
        let mut seen_tx_ids = HashSet::new();
        
        for tx in &subblock.transactions {
            log(&format!("HIG-{}", chain_id), &format!("tx-id={} : data={}", tx.id.0, tx.data));
            
            // Skip if we've seen this transaction ID before in this subblock
            if !seen_tx_ids.insert(tx.id.clone()) {
                log(&format!("HIG-{}", chain_id), &format!("Skipping duplicate transaction ID: {}", tx.id.0));
                continue;
            }
            
            // Skip if transaction ID already exists in our state
            if self.state.lock().await.received_txs.contains_key(&tx.id) {
                log(&format!("HIG-{}", chain_id), &format!("Skipping transaction ID that already exists in state: {}", tx.id.0));
                continue;
            }
            
            // Process the transaction
            HyperIG::process_transaction(self, tx.clone()).await.map_err(|e| HyperIGError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Gets the dependencies of a transaction.
    /// 
    /// # Arguments
    /// * `transaction_id` - The ID of the transaction to get the dependencies for
    /// 
    /// # Returns
    /// Result containing a vector of transaction IDs that are dependencies of the given transaction
    async fn get_transaction_dependencies(&self, transaction_id: TransactionId) -> Result<Vec<TransactionId>, HyperIGError> {
        let state = self.state.lock().await;
        let chain_id = state.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), &format!("Getting dependencies for tx-id='{}'", transaction_id.0));
        
        // Get the transaction IDs this transaction depends on
        let dependencies = state.tx_depends_on_txs.get(&transaction_id)
            .cloned()
            .unwrap_or_default();
        
        log(&format!("HIG-{}", chain_id), &format!("Found dependencies for tx-id='{}': {:?}", transaction_id.0, dependencies));
        Ok(dependencies.into_iter().collect())
    }

    /// Gets the data of a transaction.
    /// 
    /// # Arguments
    /// * `tx_id` - The ID of the transaction to get the data for
    /// 
    /// # Returns
    /// Result containing the transaction data or an error if not found
    async fn get_transaction_data(&self, tx_id: TransactionId) -> Result<String, anyhow::Error> {
        let chain_id = self.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), &format!("Getting data for tx-id='{}'", tx_id));
        let tx = self.state.lock().await.received_txs.get(&tx_id)
            .cloned()
            .ok_or_else(|| {
                log(&format!("HIG-{}", chain_id), &format!("Transaction not found tx-id='{}'", tx_id));
                anyhow::anyhow!("Transaction not found: {}", tx_id)
            })?;
        log(&format!("HIG-{}", chain_id), &format!("Found data for tx-id='{}': {}", tx_id, tx.data));
        Ok(tx.data)
    }

    /// Gets the current state of the chain.
    /// Returns a HashMap containing the current state of all accounts and their balances.
    async fn get_chain_state(&self) -> Result<std::collections::HashMap<String, i64>, anyhow::Error> {
        let chain_id = self.state.lock().await.my_chain_id.0.clone();
        log(&format!("HIG-{}", chain_id), "Getting chain state");
        
        // Get the state from the MockVM and convert it to the expected format
        let vm_state = {
            let state = self.state.lock().await;
            state.vm.get_state().clone()
        };
        
        let mut state = std::collections::HashMap::new();
        
        // Convert u32 keys and values to String and i64
        for (key, value) in vm_state {
            state.insert(key.to_string(), value as i64);
        }
        
        log(&format!("HIG-{}", chain_id), &format!("Chain state: {:?}", state));
        Ok(state)
    }
}

//==============================================================================
// Arc<Mutex<HyperIGNode>> Implementation
//==============================================================================

#[async_trait]
impl HyperIG for Arc<Mutex<HyperIGNode>> {
    /// Processes a transaction.
    /// 
    /// Handles different types of transactions (regular, CAT, dependent, status update)
    /// and updates their status accordingly.
    /// 
    /// # Arguments
    /// * `tx` - The transaction to process
    /// 
    /// # Returns
    /// Result containing the final transaction status
    async fn process_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        let mut node = self.lock().await;
        node.process_transaction(transaction).await
    }

    /// Gets the current status of a transaction.
    /// 
    /// # Arguments
    /// * `tx_id` - The ID of the transaction to get the status for
    /// 
    /// # Returns
    /// Result containing the transaction status or an error if not found
    async fn get_transaction_status(&self, transaction_id: TransactionId) -> Result<TransactionStatus, anyhow::Error> {
        let node = self.lock().await;
        node.get_transaction_status(transaction_id).await
    }

    /// Gets the list of pending transactions.
    /// 
    /// # Returns
    /// Result containing a vector of pending transaction IDs
    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error> {
        let node = self.lock().await;
        node.get_pending_transactions().await
    }

    /// Sends a CAT status proposal to the Hyper Scheduler.
    /// 
    /// # Arguments
    /// * `cat_id` - The ID of the CAT transaction
    /// * `status` - The proposed status
    /// * `constituent_chains` - The chains involved in the CAT
    /// 
    /// # Returns
    /// Result indicating success or failure of sending the proposal
    async fn send_cat_status_proposal(&mut self, cat_id: CATId, status: CATStatusLimited, constituent_chains: Vec<ChainId>) -> Result<(), HyperIGError> {
        let mut node = self.lock().await;
        node.send_cat_status_proposal(cat_id, status, constituent_chains).await
    }

    /// Gets the resolution status of a transaction.
    /// 
    /// # Arguments
    /// * `id` - The ID of the transaction to get the resolution status for
    /// 
    /// # Returns
    /// Result containing the resolution status or an error if not found
    async fn get_resolution_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError> {
        let node = self.lock().await;
        node.get_resolution_status(id).await
    }

    /// Processes a subblock of transactions.
    /// 
    /// Validates the subblock's chain ID and processes each transaction within it.
    /// 
    /// # Arguments
    /// * `subblock` - The SubBlock containing transactions to process
    /// 
    /// # Returns
    /// Result indicating success or failure of subblock processing
    async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        let mut node = self.lock().await;
        node.process_subblock(subblock).await
    }

    async fn get_transaction_dependencies(&self, transaction_id: TransactionId) -> Result<Vec<TransactionId>, HyperIGError> {
        let node = self.lock().await;
        node.get_transaction_dependencies(transaction_id).await
    }

    async fn get_transaction_data(&self, tx_id: TransactionId) -> Result<String, anyhow::Error> {
        let node = self.lock().await;
        node.get_transaction_data(tx_id).await
    }

    /// Gets the current state of the chain.
    /// Returns a HashMap containing the current state of all accounts and their balances.
    async fn get_chain_state(&self) -> Result<std::collections::HashMap<String, i64>, anyhow::Error> {
        let node = self.lock().await;
        node.get_chain_state().await
    }
} 