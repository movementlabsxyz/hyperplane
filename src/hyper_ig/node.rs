use std::collections::{HashMap, HashSet};
use crate::types::{Transaction, TransactionId, TransactionStatus, StatusLimited, CAT, CATId, SubBlock, CATStatusUpdate};
use super::{HyperIG, HyperIGError};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use std::time::Duration;
use crate::types::ChainId;
/// The internal state of the HyperIGNode
struct HyperIGState {
    /// Map of transaction IDs to their current status
    transaction_statuses: HashMap<TransactionId, TransactionStatus>,
    /// Set of pending transaction IDs
    pending_transactions: HashSet<TransactionId>,
    /// Proposed status for CAT transactions
    cat_proposed_statuses: HashMap<TransactionId, StatusLimited>,
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
    pub fn new(receiver_cl_to_hig: mpsc::Receiver<SubBlock>, sender_hig_to_hs: mpsc::Sender<CATStatusUpdate>) -> Self {
        Self {
            state: Arc::new(Mutex::new(HyperIGState {
                transaction_statuses: HashMap::new(),
                pending_transactions: HashSet::new(),
                cat_proposed_statuses: HashMap::new(),
            })),
            receiver_cl_to_hig: Some(receiver_cl_to_hig),
            sender_hig_to_hs: Some(sender_hig_to_hs),
        }
    }

    /// Process messages
    pub async fn process_messages(hig_node: Arc<Mutex<HyperIGNode>>) {
        println!("  [HIG]   [Message loop task] Starting message processing loop");
        loop {
            // println!("  [HIG]   [Message loop task] Attempting to acquire hig_node lock...");
            let mut node = hig_node.lock().await;
            // println!("  [HIG]   [Message loop task] Acquired hig_node lock");

            // Get the receiver from the node
            let receiver = if let Some(receiver) = &mut node.receiver_cl_to_hig {
                receiver
            } else {
                println!("  [HIG]   [Message loop task] No receiver available, exiting loop");
                break;
            };

            // Try to receive a message
            match receiver.try_recv() {
                Ok(subblock) => {
                    println!("  [HIG]   [Message loop task] Received subblock: block_id={}, chain_id={}, tx_count={}", 
                        subblock.block_id, subblock.chain_id.0, subblock.transactions.len());
                    
                    // Process the subblock
                    if let Err(e) = node.process_subblock(subblock).await {
                        println!("  [HIG]   [Message loop task] Error processing subblock: {}", e);
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // No message available, release the lock and wait a bit
                    drop(node);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    println!("  [HIG]   [Message loop task] Channel disconnected, exiting loop");
                    break;
                }
            }
        }
        println!("  [HIG]   [Message loop task] Message processing loop exiting");
    }

    /// Process a subblock
    pub async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        println!("  [HIG]   Processing subblock: block_id={}, chain_id={}, tx_count={}", 
            subblock.block_id, subblock.chain_id.0, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("  [HIG]   ...tx-id={} : data={}", tx.id.0, tx.data);
        }
        for tx in subblock.transactions {
            HyperIG::process_transaction(self, tx).await.map_err(|e| HyperIGError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Start the node's block processing loop
    pub async fn start(&mut self) {
        println!("  [HIG]   Starting HyperIG node");
        if let Err(e) = self.process_incoming_subblocks().await {
            println!("  [HIG]   Error in message processing loop: {}", e);
        }
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

    /// Send a CAT status proposal to the Hyper Scheduler with a specific transaction ID
    pub async fn send_cat_status_proposal_with_transaction_id(
        &mut self,
        cat_id: CATId,
        _tx_id: TransactionId,
        status: StatusLimited
    ) -> Result<(), HyperIGError> {
        if let Some(sender) = &mut self.sender_hig_to_hs {
            let status_update = CATStatusUpdate {
                cat_id: cat_id.clone(),
                // TODO dummy chain id for now
                chain_id: ChainId("dummy-chain".to_string()),
                status: status.clone(),
            };
            sender.send(status_update).await.map_err(|e| HyperIGError::Communication(e.to_string()))?;
        }
        Ok(())
    }

    /// Get the proposed status for a CAT transaction
    pub async fn get_proposed_status(&self, tx_id: TransactionId) -> Result<StatusLimited, anyhow::Error> {
        Ok(self.state.lock().await.cat_proposed_statuses.get(&tx_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No proposed status found for transaction"))?)
    }

    /// Handle a CAT transaction
    async fn handle_cat_transaction(&mut self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        println!("  [HIG]   Handling CAT transaction: {}", tx.id.0);
        // CAT transactions are always pending
        self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Pending);
        // Add to pending transactions set
        self.state.lock().await.pending_transactions.insert(tx.id.clone());
        
        // Store proposed status based on transaction data
        let proposed_status = if tx.data.contains("Success") {
            StatusLimited::Success
        } else {
            // Default to Failure for all other cases
            StatusLimited::Failure
        };
        self.state.lock().await.cat_proposed_statuses.insert(tx.id.clone(), proposed_status);
        
        Ok(TransactionStatus::Pending)
    }

    /// Handle a status update transaction
    async fn handle_status_update(&mut self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        println!("  [HIG]   Handling status update tx-id='{}' : data='{}'", tx.id.0, tx.data);
        
        // Extract the CAT ID and status from the transaction data
        // Format: STATUS_UPDATE.<Status>.CAT_ID:<cat_id>
        let parts: Vec<&str> = tx.data.split(".").collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid status update format: {}", tx.data));
        }
        
        let status_part = parts[1];
        let cat_id_part = parts[2];
        if !cat_id_part.starts_with("CAT_ID:") {
            return Err(anyhow::anyhow!("Invalid CAT ID format in status update: {}", cat_id_part));
        }
        
        let cat_id = cat_id_part.replace("CAT_ID:", "");
        let original_tx_id = TransactionId(cat_id.clone());
        
        // Update the status of the original CAT transaction
        let status = if status_part == "Success" {
            TransactionStatus::Success
        } else if status_part == "Failure" {
            TransactionStatus::Failure
        } else {
            return Err(anyhow::anyhow!("Invalid status in update: {}", status_part));
        };
        println!("  [HIG]   ... (Before) status of original tx-id='{}': {:?}", original_tx_id.0, self.state.lock().await.transaction_statuses.get(&original_tx_id));
        self.state.lock().await.transaction_statuses.insert(original_tx_id.clone(), status.clone());
        println!("  [HIG]   Updated status to {:?} for original CAT tx-id='{}'", status, original_tx_id.0);
        println!("  [HIG]   ... (After)  status of original tx-id='{}': {:?}", original_tx_id.0, self.state.lock().await.transaction_statuses.get(&original_tx_id));
        
        // Remove from pending transactions if present
        self.state.lock().await.pending_transactions.remove(&original_tx_id);
        
        Ok(TransactionStatus::Success)
    }

    /// Handle a regular transaction
    async fn handle_regular_transaction(&self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        println!("  [HIG]   Executing regular transaction: {}", tx.id);
        // check the data if its "REGULAR.SIMULATION.<Status>"
        if tx.data.starts_with("REGULAR.SIMULATION.Success") {
            // Store Success status for regular transactions
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Success);
            println!("  [HIG]   Set final status to Success for transaction: {}", tx.id.0);
            Ok(TransactionStatus::Success)
        } else if tx.data.starts_with("REGULAR.SIMULATION.Failure") {
            // Store Failure status for regular transactions
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Failure);
            println!("  [HIG]   Set final status to Failure for transaction: {}", tx.id.0);
            Ok(TransactionStatus::Failure)
        } else {
            // TODO we only handle correct data txs for now to have strict control over the transactions. We may get rid of this later.
            return Err(anyhow::anyhow!("Invalid regular transaction data: '{}'", tx.data));
        }
    }

    /// Handle a dependent regular transaction
    async fn handle_dependent_regular_transaction(&self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        println!("  [HIG]   Processing dependent transaction: {}", transaction.id);
        // Add to pending transactions set
        self.state.lock().await.pending_transactions.insert(transaction.id.clone());
        // Transactions depending on CATs stay pending until the CAT is resolved
        Ok(TransactionStatus::Pending)
    }
}

#[async_trait]
impl HyperIG for HyperIGNode {
    async fn process_transaction(&mut self, tx: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        println!("  [HIG]   Processing tx-id='{}' : data='{}'", tx.id, tx.data);

        // handle the case where it is a status update separately
        // because it doesn't need to be inserted into the transaction statuses
        let status = if tx.data.starts_with("STATUS_UPDATE") {
            self.handle_status_update(tx.clone()).await?
        } else {
            // now handle the case where it is any of the other transaction types
            // Store initial status
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), TransactionStatus::Pending);
            println!("  [HIG]   Set initial status to Pending for tx-id: '{}' : data: '{}'", tx.id, tx.data);
            
            let status = if tx.data.starts_with("CAT") {
                self.handle_cat_transaction(tx.clone()).await?
            } else if tx.data.starts_with("DEPENDENT") {
                self.handle_dependent_regular_transaction(tx.clone()).await?
            } else {
                self.handle_regular_transaction(tx.clone()).await?
            };
            
            // Update status
            self.state.lock().await.transaction_statuses.insert(tx.id.clone(), status.clone());
            println!("  [HIG]   Updated status to {:?} for tx-id='{}'", status, tx.id.0);
            
            status
        };

        // Send status proposal to Hyper Scheduler if it's a CAT transaction
        if tx.data.starts_with("CAT") {
            // extract the cat id and status from the data
            let parts: Vec<&str> = tx.data.split(":").collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!("Invalid CAT tx data format: expected 'CAT.SIMULATION.<Status>:<ID>', got '{}'", tx.data));
            }
            let cat_id = CATId(parts[1].to_string());
            let status = if parts[0].contains("Success") {
                StatusLimited::Success
            } else if parts[0].contains("Failure") {
                StatusLimited::Failure
            } else {
                return Err(anyhow::anyhow!("Invalid CAT status in data: {}", parts[0]));
            };
            println!("  [HIG]   Extracted cat-id='{}' with status: {:?}", cat_id.0, status);
            println!("  [HIG]   Sending status proposal for cat-id='{}'", cat_id.0);
            self.send_cat_status_proposal(cat_id, status).await?;
            println!("  [HIG]   Status proposal sent for CAT transaction.");
        }

        Ok(status)
    }

    async fn get_transaction_status(&self, tx_id: TransactionId) -> Result<TransactionStatus, anyhow::Error> {
        println!("  [HIG]   Getting status for tx-id='{}'", tx_id);
        let statuses = self.state.lock().await.transaction_statuses.get(&tx_id)
            .cloned()
            .ok_or_else(|| {
                println!("  [HIG]   Transaction not found tx-id='{}'", tx_id);
                anyhow::anyhow!("Transaction not found: {}", tx_id)
            })?;
        println!("  [HIG]   Found status for tx-id='{}': {:?}", tx_id, statuses);
        Ok(statuses)
    }

    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error> {
        Ok(self.state.lock().await.pending_transactions.iter().cloned().collect())
    }

    async fn send_cat_status_proposal(&mut self, cat_id: CATId, status: StatusLimited) -> Result<(), HyperIGError> {
        if let Some(sender) = &mut self.sender_hig_to_hs {
            let status_update = CATStatusUpdate {
                cat_id: cat_id.clone(),
                // TODO dummy chain id for now
                chain_id: ChainId("dummy-chain".to_string()),
                status: status.clone(),
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
        println!("  [HIG]   Processing subblock: block_id={}, chain_id={}, tx_count={}", 
            subblock.block_id, subblock.chain_id.0, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("  [HIG]   .......tx-id={} : data={}", tx.id.0, tx.data);
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

    async fn send_cat_status_proposal(&mut self, cat_id: CATId, status: StatusLimited) -> Result<(), HyperIGError> {
        let mut node = self.lock().await;
        node.send_cat_status_proposal(cat_id, status).await
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