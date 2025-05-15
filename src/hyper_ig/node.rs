use std::collections::{HashMap, HashSet};
use crate::types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited, CAT, CATId, SubBlock, CATStatusUpdate};
use super::{HyperIG, HyperIGError};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The internal state of the HyperIGNode
struct HyperIGState {
    /// Map of transaction IDs to their current status
    transaction_statuses: HashMap<TransactionId, TransactionStatus>,
    /// Set of pending transaction IDs
    pending_transactions: HashSet<TransactionId>,
    /// Proposed status for CAT transactions
    cat_proposed_statuses: HashMap<TransactionId, CATStatusLimited>,
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

    /// Process messages without holding the node lock
    pub async fn process_messages(hig_node: Arc<Mutex<HyperIGNode>>) {
        println!("  [HIG]   [Message loop task] Attempting to acquire hig_node lock...");
        let mut node = hig_node.lock().await;
        println!("  [HIG]   [Message loop task] Acquired hig_node lock");
        let mut receiver = node.receiver_cl_to_hig.take().expect("Receiver already taken");
        let _state = node.state.clone();
        drop(node); // Release the lock before starting the loop
        println!("  [HIG]   [Message loop task] Released hig_node lock");
        
        // Process messages
        while let Some(subblock) = receiver.recv().await {
            println!("  [HIG]   [Message loop task] Received subblock: block_id={}, chain_id={}, tx_count={}", 
                subblock.block_id, subblock.chain_id.0, subblock.transactions.len());
            
            // Process each transaction in the subblock
            for tx in subblock.transactions {
                println!("  [HIG]   [Message loop task] Processing transaction: {}", tx.id.0);
                println!("  [HIG]   [Message loop task] Attempting to acquire node lock for transaction...");
                {
                    let mut node = hig_node.lock().await;
                    println!("  [HIG]   [Message loop task] Acquired node lock for transaction");
                    if let Err(e) = HyperIG::execute_transaction(&mut *node, tx).await {
                        println!("  [HIG]   [Message loop task] Error executing transaction: {}", e);
                    }
                    println!("  [HIG]   [Message loop task] Updated state, releasing lock");
                }
                println!("  [HIG]   [Message loop task] Released node lock after transaction");
            }
            println!("  [HIG]   [Message loop task] Successfully processed subblock");
        }
        println!("  [HIG]   [Message loop task] Message processing loop exiting");
    }

    /// Process a subblock
    pub async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        println!("  [HIG]   Processing subblock: block_id={}, chain_id={}, tx_count={}", 
            subblock.block_id, subblock.chain_id.0, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("  [HIG]   Executing transaction: id={}, data={}", tx.id.0, tx.data);
        }
        for tx in subblock.transactions {
            HyperIG::execute_transaction(self, tx).await.map_err(|e| HyperIGError::Internal(e.to_string()))?;
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
        _transaction_id: TransactionId,
        status: CATStatusLimited
    ) -> Result<(), HyperIGError> {
        if let Some(sender) = &mut self.sender_hig_to_hs {
            let status_update = CATStatusUpdate {
                cat_id: cat_id.clone(),
                status: status.clone(),
            };
            sender.send(status_update).await.map_err(|e| HyperIGError::Communication(e.to_string()))?;
        }
        Ok(())
    }

    /// Get the proposed status for a CAT transaction
    pub async fn get_proposed_status(&self, transaction_id: TransactionId) -> Result<CATStatusLimited, anyhow::Error> {
        Ok(self.state.lock().await.cat_proposed_statuses.get(&transaction_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No proposed status found for transaction"))?)
    }

    /// Handle a CAT transaction
    async fn handle_cat_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // CAT transactions are always pending
        self.state.lock().await.transaction_statuses.insert(transaction.id.clone(), TransactionStatus::Pending);
        // Add to pending transactions set
        self.state.lock().await.pending_transactions.insert(transaction.id.clone());
        
        // Store proposed status based on transaction data
        let proposed_status = if transaction.data.contains("SUCCESS") {
            CATStatusLimited::Success
        } else {
            // Default to Failure for all other cases
            CATStatusLimited::Failure
        };
        self.state.lock().await.cat_proposed_statuses.insert(transaction.id.clone(), proposed_status);
        
        Ok(TransactionStatus::Pending)
    }

    /// Handle a status update transaction
    async fn handle_status_update(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // Status update transactions are always successful
        self.state.lock().await.transaction_statuses.insert(transaction.id.clone(), TransactionStatus::Success);
        Ok(TransactionStatus::Success)
    }
}

#[async_trait::async_trait]
impl HyperIG for HyperIGNode {
    async fn execute_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        println!("  [HIG]   Executing transaction: {}", transaction.id.0);
        // Store initial status
        self.state.lock().await.transaction_statuses.insert(transaction.id.clone(), TransactionStatus::Pending);
        println!("  [HIG]   Set initial status to Pending for transaction: {}", transaction.id.0);

        // Execute the transaction
        let status = if transaction.data.starts_with("CAT") {
            self.handle_cat_transaction(transaction.clone()).await?
        } else if transaction.data.starts_with("STATUS_UPDATE") {
            self.handle_status_update(transaction.clone()).await?
        } else if transaction.data.starts_with("DEPENDENT_ON_CAT") {
            // Add to pending transactions set
            self.state.lock().await.pending_transactions.insert(transaction.id.clone());
            // Transactions depending on CATs stay pending until the CAT is resolved
            TransactionStatus::Pending
        } else {
            // Regular transaction
            println!("  [HIG]   Executing regular transaction: {}", transaction.id);
            // Store Success status for regular transactions
            self.state.lock().await.transaction_statuses.insert(transaction.id.clone(), TransactionStatus::Success);
            println!("  [HIG]   Set final status to Success for transaction: {}", transaction.id.0);
            TransactionStatus::Success
        };

        // Update status
        self.state.lock().await.transaction_statuses.insert(transaction.id.clone(), status.clone());
        println!("  [HIG]   Updated status to {:?} for transaction: {}", status, transaction.id.0);

        // Send status proposal to Hyper Scheduler if it's a CAT transaction
        if transaction.data.starts_with("CAT") {
            let cat_id = CATId(transaction.id.0.clone());
            self.send_cat_status_proposal(cat_id, CATStatusLimited::Success).await?;
        }

        Ok(status)
    }

    async fn get_transaction_status(&self, tx_id: TransactionId) -> Result<TransactionStatus, anyhow::Error> {
        println!("  [HIG]   Getting status for transaction: {}", tx_id.0);
        let statuses = self.state.lock().await.transaction_statuses.get(&tx_id)
            .cloned()
            .ok_or_else(|| {
                println!("  [HIG]   Transaction not found: {}", tx_id.0);
                anyhow::anyhow!("Transaction not found: {}", tx_id)
            })?;
        println!("  [HIG]   Found status for transaction {}: {:?}", tx_id.0, statuses);
        Ok(statuses)
    }

    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error> {
        Ok(self.state.lock().await.pending_transactions.iter().cloned().collect())
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
} 