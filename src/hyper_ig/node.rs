use std::collections::{HashMap, HashSet};
use crate::types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited, CAT, CATId, SubBlock, CATStatusUpdate};
use super::{HyperIG, HyperIGError};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Node implementation of the Hyper Information Gateway
pub struct HyperIGNode {
    /// Map of transaction IDs to their current status
    transaction_statuses: Arc<RwLock<HashMap<TransactionId, TransactionStatus>>>,
    /// Set of pending transaction IDs
    pending_transactions: HashSet<TransactionId>,
    /// Proposed status for CAT transactions
    cat_proposed_statuses: HashMap<TransactionId, CATStatusLimited>,
    /// Receiver for messages from Confirmation Layer
    receiver_cl_to_hig: Option<mpsc::Receiver<SubBlock>>,
    /// Sender for messages to Hyper Scheduler
    sender_hig_to_hs: Option<mpsc::Sender<CATStatusUpdate>>,
}

impl HyperIGNode {
    /// Create a new HyperIGNode
    pub fn new(receiver_cl_to_hig: mpsc::Receiver<SubBlock>, sender_hig_to_hs: mpsc::Sender<CATStatusUpdate>) -> Self {
        Self {
            transaction_statuses: Arc::new(RwLock::new(HashMap::new())),
            pending_transactions: HashSet::new(),
            cat_proposed_statuses: HashMap::new(),
            receiver_cl_to_hig: Some(receiver_cl_to_hig),
            sender_hig_to_hs: Some(sender_hig_to_hs),
        }
    }

    /// Process a subblock
    pub async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        println!("[HIG] Processing subblock: block_id={}, chain_id={}, tx_count={}", 
            subblock.block_id, subblock.chain_id.0, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("[HIG] Executing transaction: id={}, data={}", tx.id.0, tx.data);
        }
        for tx in subblock.transactions {
            self.execute_transaction(tx).await.map_err(|e| HyperIGError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Handle a CAT transaction
    async fn handle_cat_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // CAT transactions are always pending
        self.transaction_statuses.write().await.insert(transaction.id.clone(), TransactionStatus::Pending);
        // Add to pending transactions set
        self.pending_transactions.insert(transaction.id.clone());
        
        // Store proposed status based on transaction data
        let proposed_status = if transaction.data.contains("SUCCESS") {
            CATStatusLimited::Success
        } else {
            // Default to Failure for all other cases
            CATStatusLimited::Failure
        };
        self.cat_proposed_statuses.insert(transaction.id.clone(), proposed_status);
        
        Ok(TransactionStatus::Pending)
    }

    /// Handle a status update transaction
    async fn handle_status_update(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // Status update transactions are always successful
        self.transaction_statuses.write().await.insert(transaction.id.clone(), TransactionStatus::Success);
        Ok(TransactionStatus::Success)
    }

    /// Process incoming subblocks from the Confirmation Layer
    pub async fn process_incoming_subblocks(&mut self) -> Result<(), HyperIGError> {
        if let Some(receiver) = self.receiver_cl_to_hig.take() {
            let mut receiver = receiver;
            while let Some(subblock) = receiver.recv().await {
                println!("[HIG] Processing subblock: block_id={}, chain_id={}, tx_count={}", 
                    subblock.block_id, subblock.chain_id.0, subblock.transactions.len());
                self.process_subblock(subblock).await?;
            }
        }
        Ok(())
    }

    /// Start the node's message processing loop
    pub async fn start(&mut self) {
        println!("[HIG] Starting HyperIG node");
        if let Err(e) = self.process_incoming_subblocks().await {
            println!("[HIG] Error in message processing loop: {}", e);
        }
    }
}

#[async_trait::async_trait]
impl HyperIG for HyperIGNode {
    async fn execute_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // Store initial status
        self.transaction_statuses.write().await.insert(transaction.id.clone(), TransactionStatus::Pending);

        // Execute the transaction
        let status = if transaction.data.starts_with("CAT") {
            self.handle_cat_transaction(transaction.clone()).await?
        } else if transaction.data.starts_with("STATUS_UPDATE") {
            self.handle_status_update(transaction.clone()).await?
        } else if transaction.data.starts_with("DEPENDENT_ON_CAT") {
            // Add to pending transactions set
            self.pending_transactions.insert(transaction.id.clone());
            // Transactions depending on CATs stay pending until the CAT is resolved
            TransactionStatus::Pending
        } else {
            // Regular transaction
            println!("[HIG] Executing regular transaction: {}", transaction.id);
            TransactionStatus::Success
        };

        // Update status
        self.transaction_statuses.write().await.insert(transaction.id.clone(), status.clone());

        // Send status proposal to Hyper Scheduler if it's a CAT transaction
        if transaction.data.starts_with("CAT") {
            let cat_id = CATId(transaction.id.0.clone());
            self.send_cat_status_proposal(cat_id, CATStatusLimited::Success).await?;
        }

        Ok(status)
    }

    async fn get_transaction_status(&self, tx_id: TransactionId) -> Result<TransactionStatus, anyhow::Error> {
        let statuses = self.transaction_statuses.read().await;
        statuses.get(&tx_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_id))
    }

    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error> {
        Ok(self.pending_transactions.iter().cloned().collect())
    }

    async fn send_cat_status_proposal(&mut self, cat_id: CATId, status: CATStatusLimited) -> Result<(), HyperIGError> {
        if let Some(sender) = &mut self.sender_hig_to_hs {
            let status_update = CATStatusUpdate {
                cat_id: cat_id.clone(),
                status: status.clone(),
            };
            sender.send(status_update).await.map_err(|e| HyperIGError::Communication(e.to_string()))?;
        }
        Ok(())
    }

    async fn resolve_transaction(&mut self, tx: CAT) -> Result<TransactionStatus, HyperIGError> {
        // For now, just return the current status
        let statuses = self.transaction_statuses.read().await;
        statuses.get(&TransactionId(tx.id.0.clone()))
            .cloned()
            .ok_or_else(|| HyperIGError::TransactionNotFound(TransactionId(tx.id.0.clone())))
    }

    async fn get_resolution_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError> {
        let statuses = self.transaction_statuses.read().await;
        statuses.get(&id)
            .cloned()
            .ok_or_else(|| HyperIGError::TransactionNotFound(id))
    }
}

impl HyperIGNode {
    /// Get the proposed status for a CAT transaction
    pub async fn get_proposed_status(&self, transaction_id: TransactionId) -> Result<CATStatusLimited, anyhow::Error> {
        Ok(self.cat_proposed_statuses.get(&transaction_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No proposed status found for transaction"))?)
    }
} 