use std::collections::{HashMap, HashSet};
use crate::types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited, CAT, CATId, SubBlock};
use super::{HyperIG, HyperIGError};
use crate::types::cat::CATStatusUpdate;
use crate::types::communication::{Sender, Receiver};
use crate::types::communication::hig_to_hs::CATStatusUpdateMessage;
use crate::types::communication::cl_to_hig::SubBlockMessage;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;

/// Node implementation of the Hyper Information Gateway
pub struct HyperIGNode {
    /// Map of transaction IDs to their current status
    transaction_statuses: Arc<RwLock<HashMap<TransactionId, TransactionStatus>>>,
    /// Set of pending transaction IDs
    pending_transactions: HashSet<TransactionId>,
    /// Proposed status for CAT transactions
    cat_proposed_statuses: HashMap<TransactionId, CATStatusLimited>,
    /// Receiver for messages from Confirmation Layer
    receiver_from_cl: Option<Receiver<SubBlockMessage>>,
    /// Sender for messages to Hyper Scheduler
    sender_to_hs: Option<Sender<CATStatusUpdateMessage>>,
}

impl HyperIGNode {
    /// Create a new HyperIGNode
    pub fn new(receiver_from_cl: Receiver<SubBlockMessage>, sender_to_hs: Sender<CATStatusUpdateMessage>) -> Self {
        Self {
            transaction_statuses: Arc::new(RwLock::new(HashMap::new())),
            pending_transactions: HashSet::new(),
            cat_proposed_statuses: HashMap::new(),
            receiver_from_cl: Some(receiver_from_cl),
            sender_to_hs: Some(sender_to_hs),
        }
    }

    /// Process a subblock
    pub async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        tracing::info!("Processing subblock: block_id={}, chain_id={}, tx_count={}", subblock.block_id.0, subblock.chain_id.0, subblock.transactions.len());
        for tx in &subblock.transactions {
            tracing::info!("Executing transaction: id={}, data={}", tx.id.0, tx.data);
        }
        for tx in subblock.transactions {
            self.execute_transaction(tx).await.map_err(|e| HyperIGError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Handle a CAT transaction
    async fn handle_cat_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // CAT transactions always stay pending
        self.transaction_statuses.write().await.insert(transaction.id.clone(), TransactionStatus::Pending);
        self.pending_transactions.insert(transaction.id.clone());
        
        // TODO: This is a dummy implementation for testing. for now it stays forever pending until we handle dependency
        if transaction.data == "DEPENDENT" {
            self.transaction_statuses.write().await.insert(transaction.id.clone(), TransactionStatus::Pending);
            self.pending_transactions.insert(transaction.id.clone());
            Ok(TransactionStatus::Pending)
        } else {
            self.transaction_statuses.write().await.insert(transaction.id.clone(), TransactionStatus::Success);
            Ok(TransactionStatus::Success)
        }
    }

    /// Handle a regular transaction
    async fn _handle_regular_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // check if data is dependent
        // TODO: This is a dummy implementation for testing. for now it stays forever pending until we handle dependency
        if transaction.data == "DEPENDENT" {
            self.transaction_statuses.write().await.insert(transaction.id.clone(), TransactionStatus::Pending);
            self.pending_transactions.insert(transaction.id.clone());
            Ok(TransactionStatus::Pending)
        } else {
            self.transaction_statuses.write().await.insert(transaction.id.clone(), TransactionStatus::Success);
            Ok(TransactionStatus::Success)
        }
    }

    /// Handle a status update transaction
    async fn handle_status_update(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // Status update transactions are always successful
        self.transaction_statuses.write().await.insert(transaction.id.clone(), TransactionStatus::Success);
        Ok(TransactionStatus::Success)
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
        } else {
            // Regular transaction
            tracing::info!("Executing regular transaction: {}", transaction.id);
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
        // Store the proposed status locally
        self.cat_proposed_statuses.insert(TransactionId(cat_id.0.clone()), status.clone());


        // create a cat status update
        let cat_status_update = CATStatusUpdate {
            cat_id: cat_id.clone(),
            status: status.clone(),
        };

        // Create and send the proposal
        let proposal = CATStatusUpdateMessage {
            cat_status_update: cat_status_update,
        };
        self.sender_to_hs.as_ref()
            .ok_or_else(|| HyperIGError::Communication("No sender available".to_string()))?
            .send(proposal)
            .await
            .map_err(|e| HyperIGError::Communication(e.to_string()))?;

        Ok(())
    }

    async fn resolve_transaction(&mut self, tx: CAT) -> Result<TransactionStatus, HyperIGError> {
        // Convert CATId to TransactionId by using the inner String value
        let transaction_id = TransactionId(tx.id.0);
        self.get_resolution_status(transaction_id).await
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