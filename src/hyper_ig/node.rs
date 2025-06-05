use std::collections::{HashMap, HashSet};
use crate::types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited, CAT, CATId, SubBlock, CATStatusUpdate};
use super::{HyperIG, HyperIGError};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use std::time::Duration;
use crate::types::ChainId;
use crate::types::communication::cl_to_hig::{STATUS_UPDATE_PATTERN, parse_cat_transaction};
use crate::utils::logging::log;
use crate::mock_vm::MockVM;
use x_chain_vm::transaction::Transaction as VMTransaction;
/// The internal state of the HyperIGNode
struct HyperIGState {
    /// Map of transaction IDs to their current status
    transaction_statuses: HashMap<TransactionId, TransactionStatus>,
    /// Set of pending transaction IDs
    pending_transactions: HashSet<TransactionId>,
    /// Proposed status for CAT transactions
    cat_proposed_statuses: HashMap<TransactionId, CATStatusLimited>,
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

impl HyperIGNode {
    /// Create a new HyperIGNode
    pub fn new(receiver_cl_to_hig: mpsc::Receiver<SubBlock>, sender_hig_to_hs: mpsc::Sender<CATStatusUpdate>, my_chain_id: ChainId) -> Self {
        Self {
            state: Arc::new(Mutex::new(HyperIGState {
                transaction_statuses: HashMap::new(),
                pending_transactions: HashSet::new(),
                cat_proposed_statuses: HashMap::new(),
                my_chain_id: my_chain_id,
                vm: MockVM::new(),
            })),
            receiver_cl_to_hig: Some(receiver_cl_to_hig),
            sender_hig_to_hs: Some(sender_hig_to_hs),
        }
    }

    /// Process messages
    pub async fn process_messages(hig_node: Arc<Mutex<HyperIGNode>>) -> Result<(), HyperIGError> {
        log("HIG", "[Message loop task] Starting message processing loop");
        loop {
            let mut node = hig_node.lock().await;

            // Get the receiver from the node
            let receiver = if let Some(receiver) = &mut node.receiver_cl_to_hig {
                receiver
            } else {
                log("HIG", "[Message loop task] No receiver available, exiting loop");
                break;
            };

            // Try to receive a message
            match receiver.try_recv() {
                Ok(subblock) => {
                    // Process the subblock
                    if let Err(e) = node.process_subblock(subblock).await {
                        log("HIG", &format!("[Message loop task] Error processing subblock: {}", e));
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // No message available, release the lock and wait a bit
                    drop(node);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    log("HIG", "[Message loop task] Channel disconnected, exiting loop");
                    break;
                }
            }
        }
        log("HIG", "[Message loop task] Message processing loop exiting");
        Ok(())
    }

    /// Process a subblock
    pub async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        // check if the received subblock matches our chain id. If not we have a bug.
        log("HIG", &format!("[process_subblock] Processing subblock: block_id={}, chain_id={}, tx_count={}", 
        subblock.block_height, subblock.chain_id.0, subblock.transactions.len()));
        
        if subblock.chain_id.0 != self.state.lock().await.my_chain_id.0 {
            log("HIG", &format!("[Message loop task] [ERROR] Received subblock with chain_id='{}', but should be '{}', ignoring", 
                subblock.chain_id.0, self.state.lock().await.my_chain_id.0));
            return Err(HyperIGError::WrongChainId { 
                expected: self.state.lock().await.my_chain_id.clone(),
                received: subblock.chain_id.clone(),
            });
        }

        for tx in &subblock.transactions {
            log("HIG", &format!("[process_subblock] tx-id={} : data={}", tx.id.0, tx.data));
        }
        for tx in subblock.transactions {
            HyperIG::process_transaction(self, tx).await.map_err(|e| HyperIGError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Start the node's block processing loop
    pub async fn start(node: Arc<Mutex<HyperIGNode>>) {
        log("HIG", "Starting HyperIG node");
        tokio::spawn(async move { HyperIGNode::process_messages(node).await.unwrap() });
    }

    /// Process incoming subblocks from the Confirmation Layer
    pub async fn process_incoming_subblocks(&mut self) -> Result<(), HyperIGError> {
        if let Some(receiver) = self.receiver_cl_to_hig.take() {
            let mut receiver = receiver;
            while let Some(subblock) = receiver.recv().await {
                self.process_subblock(subblock).await?;
            }
        }
        Ok(())
    }

    /// Get the proposed status for a CAT transaction
    pub async fn get_proposed_status(&self, tx_id: TransactionId) -> Result<CATStatusLimited, anyhow::Error> {
        Ok(self.state.lock().await.cat_proposed_statuses.get(&tx_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No proposed status found for transaction"))?)
    }

    /// Handle a CAT transaction
    async fn handle_cat_transaction(&mut self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        log("HIG", &format!("Handling CAT transaction: {}", tx.id.0));
        // CAT transactions are always pending
        self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Pending);
        // Add to pending transactions set
        self.state.lock().await.pending_transactions.insert(tx.id.clone());
        
        // Store proposed status based on transaction data
        let (_, proposed_status) = parse_cat_transaction(&tx.data)?;
        self.state.lock().await.cat_proposed_statuses.insert(tx.id.clone(), proposed_status);
        
        Ok(TransactionStatus::Pending)
    }

    /// Handle a status update transaction
    async fn handle_status_update(&mut self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        log("HIG", &format!("Handling status update tx-id='{}' : data='{}'", tx.id.0, tx.data));
        
        // Extract the CAT ID and status from the transaction data using regex
        if !STATUS_UPDATE_PATTERN.is_match(&tx.data) {
            return Err(anyhow::anyhow!("Invalid status update format: {}", tx.data));
        }
        let cat_id = STATUS_UPDATE_PATTERN.captures(&tx.data)
            .and_then(|caps| caps.name("cat_id"))
            .ok_or_else(|| anyhow::anyhow!("Failed to extract CAT ID from status update"))?;
        let cat_id = CATId(cat_id.as_str().to_string());
        let tx_id = TransactionId(cat_id.0.clone());
        
        // Has format STATUS_UPDATE:<Status>.CAT_ID:<cat_id>
        let status_part = tx.data.split(".").collect::<Vec<&str>>()[0];
        log("HIG", &format!("... (Before) status_part='{}'", status_part));
        // now we tacke the status part after the first :
        let status_part = status_part.split(":").collect::<Vec<&str>>()[1];
        log("HIG", &format!("... (After) status_part='{}'", status_part));
        let status = if status_part == "Success" {
            TransactionStatus::Success
        } else if status_part == "Failure" {
            TransactionStatus::Failure
        } else {
            return Err(anyhow::anyhow!("Invalid status in update: {}", status_part));
        };
        log("HIG", &format!("... (Before) status of original tx-id='{}': {:?}", tx_id.0, self.state.lock().await.transaction_statuses.get(&tx_id)));
        self.state.lock().await.transaction_statuses.insert(tx_id.clone(), status.clone());
        log("HIG", &format!("Updated status to {:?} for original CAT tx-id='{}'", status, tx_id.0));
        log("HIG", &format!("... (After)  status of original tx-id='{}': {:?}", tx_id.0, self.state.lock().await.transaction_statuses.get(&tx_id)));
        
        // Remove from pending transactions if present
        self.state.lock().await.pending_transactions.remove(&tx_id);
        
        Ok(status)
    }

    /// Handle a regular transaction
    async fn handle_regular_transaction(&self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        log("HIG", &format!("Executing regular transaction: {}", tx.id));
        
        // Extract the command part between the dots
        let command = tx.data.split('.').nth(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid transaction format"))?;

        // Parse and execute the transaction using x-chain-vm's parse_input
        let vm_tx = x_chain_vm::parse_input(command)
            .map_err(|e| anyhow::anyhow!("Failed to parse transaction: {}", e))?;
        
        // Execute the transaction
        let execution = vm_tx.execute(&self.state.lock().await.vm.get_state());

        // Update transaction status based on execution result
        let status = if execution.is_success() {
            TransactionStatus::Success
        } else {
            TransactionStatus::Failure
        };
        self.state.lock().await.transaction_statuses.insert(tx.id.clone(), status.clone());
        log("HIG", &format!("Set final status to {:?} for transaction: {}", status, tx.id.0));
        Ok(status)
    }

    /// Handle a dependent regular transaction
    async fn handle_dependent_regular_transaction(&self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        log("HIG", &format!("Processing dependent transaction: {}", transaction.id));
        // Add to pending transactions set
        self.state.lock().await.pending_transactions.insert(transaction.id.clone());
        // Transactions depending on CATs stay pending until the CAT is resolved
        Ok(TransactionStatus::Pending)
    }
}

#[async_trait]
impl HyperIG for HyperIGNode {
    async fn process_transaction(&mut self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        log("HIG", &format!("[process_transaction] Processing tx-id='{}' : data='{}'", tx.id, tx.data));

        // handle the case where it is a status update separately
        // because it doesn't need to be inserted into the transaction statuses
        let status = if tx.data.starts_with("STATUS_UPDATE") {
            self.handle_status_update(tx.clone()).await?
        } else {
            // now handle the case where it is any of the other transaction types
            // Store initial status
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Pending);
            log("HIG", &format!("[process_transaction] Set initial status to Pending for tx-id: '{}' : data: '{}'", tx.id, tx.data));
            
            let status = if tx.data.starts_with("CAT") {
                self.handle_cat_transaction(tx.clone()).await?
            } else if tx.data.starts_with("DEPENDENT") {
                self.handle_dependent_regular_transaction(tx.clone()).await?
            } else {
                self.handle_regular_transaction(tx.clone()).await?
            };
            
            // Update status
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), status.clone());
            log("HIG", &format!("[process_transaction] Updated status to {:?} for tx-id='{}'", status, tx.id.0));
            
            status
        };

        // Send status proposal to Hyper Scheduler if it's a CAT transaction
        if tx.data.starts_with("CAT") {
            let (cat_id, status) = parse_cat_transaction(&tx.data)?;
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

            log("HIG", &format!("[process_transaction] Extracted cat-id='{}', status='{:?}', chains='{:?}'", cat_id.0, status, constituent_chains));
            log("HIG", &format!("[process_transaction] Sending status proposal for cat-id='{}'", cat_id.0));
            self.send_cat_status_proposal(cat_id, status, constituent_chains).await?;
            log("HIG", "[process_transaction] Status proposal sent for CAT transaction.");
        }

        Ok(status)
    }

    async fn get_transaction_status(&self, tx_id: TransactionId) -> Result<TransactionStatus, anyhow::Error> {
        log("HIG", &format!("[get_transaction_status] Getting status for tx-id='{}'", tx_id));
        let statuses = self.state.lock().await.transaction_statuses.get(&tx_id)
            .cloned()
            .ok_or_else(|| {
                log("HIG", &format!("[get_transaction_status] Transaction not found tx-id='{}'", tx_id));
                anyhow::anyhow!("Transaction not found: {}", tx_id)
            })?;
        log("HIG", &format!("[get_transaction_status] Found status for tx-id='{}': {:?}", tx_id, statuses));
        Ok(statuses)
    }

    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error> {
        Ok(self.state.lock().await.pending_transactions.iter().cloned().collect())
    }

    /// Send a CAT status proposal to the Hyper Scheduler
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

    async fn resolve_transaction(&mut self, tx: CAT) -> Result<TransactionStatus, HyperIGError> {
        let statuses = self.state.lock().await.transaction_statuses.get(&TransactionId(tx.id.0.clone()))
            .cloned()
            .ok_or_else(|| HyperIGError::TransactionNotFound(TransactionId(tx.id.0.clone())))?;
        Ok(statuses)
    }

    async fn get_resolution_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError> {
        let statuses = self.state.lock().await.transaction_statuses.get(&id)
            .cloned()
            .ok_or_else(|| HyperIGError::TransactionNotFound(id))?;
        Ok(statuses)
    }

    // this is a duplicate of the other process subblock function
    // TODO remove one of them
    async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        // println!("  [HIG]   Processing subblock: block_id={}, chain_id={}, tx_count={}", 
        //     subblock.block_height, subblock.chain_id.0, subblock.transactions.len());
        for _tx in &subblock.transactions {
            // println!("  [HIG]   .......tx-id={} : data={}", tx.id.0, tx.data);
        }
        for tx in subblock.transactions {
            HyperIG::process_transaction(self, tx).await.map_err(|e| HyperIGError::Internal(e.to_string()))?;
        }
        Ok(())
    }
}

#[async_trait]
impl HyperIG for Arc<Mutex<HyperIGNode>> {
    async fn process_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        let mut node = self.lock().await;
        node.process_transaction(transaction).await
    }

    async fn get_transaction_status(&self, transaction_id: TransactionId) -> Result<TransactionStatus, anyhow::Error> {
        let node = self.lock().await;
        node.get_transaction_status(transaction_id).await
    }

    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error> {
        let node = self.lock().await;
        node.get_pending_transactions().await
    }

    async fn send_cat_status_proposal(&mut self, cat_id: CATId, status: CATStatusLimited, constituent_chains: Vec<ChainId>) -> Result<(), HyperIGError> {
        let mut node = self.lock().await;
        node.send_cat_status_proposal(cat_id, status, constituent_chains).await
    }

    async fn resolve_transaction(&mut self, tx: CAT) -> Result<TransactionStatus, HyperIGError> {
        let mut node = self.lock().await;
        node.resolve_transaction(tx).await
    }

    async fn get_resolution_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError> {
        let node = self.lock().await;
        node.get_resolution_status(id).await
    }

    async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        let mut node = self.lock().await;
        node.process_subblock(subblock).await
    }
} 