use std::collections::HashMap;
use tokio::time::{Duration, interval};
use crate::types::{
    BlockId, ChainId, SubBlock, CLTransaction, Transaction,
};
use super::{ConfirmationLayer, ConfirmationLayerError};
use crate::types::communication::{Sender, Receiver, Message};
use crate::types::communication::cl_to_hig::SubBlockMessage;
use crate::types::communication::hs_to_cl::CLTransactionMessage;

/// A simple node implementation of the ConfirmationLayer
pub struct ConfirmationLayerNode {
    /// Currently registered chains
    chains: Vec<ChainId>,
    /// Current block ID
    current_block: BlockId,
    /// Block interval
    block_interval: Duration,
    /// Pending transactions for each chain
    pending_txs: HashMap<ChainId, Vec<CLTransaction>>,
    /// Stored subblocks by chain and block ID
    subblocks: HashMap<(ChainId, BlockId), SubBlock>,
    /// Receiver for messages from Hyper Scheduler
    receiver_from_hs: Option<Receiver<CLTransactionMessage>>,
    /// Sender for messages to Hyper IG
    sender_to_hig: Option<Sender<SubBlockMessage>>,
}

impl ConfirmationLayerNode {
    /// Create a new ConfirmationLayerNode with default settings
    pub fn new(hs_receiver: Receiver<CLTransactionMessage>, hig_sender: Sender<SubBlockMessage>) -> Self {
        Self {
            chains: Vec::new(),
            current_block: BlockId("0".to_string()),
            block_interval: Duration::from_millis(100),
            pending_txs: HashMap::new(),
            subblocks: HashMap::new(),
            receiver_from_hs: Some(hs_receiver),
            sender_to_hig: Some(hig_sender),
        }
    }

    /// Create a new ConfirmationLayerNode with a custom block interval
    pub fn new_with_block_interval(
        receiver_from_hs: Receiver<CLTransactionMessage>,
        sender_to_hig: Sender<SubBlockMessage>,
        interval: Duration
    ) -> Result<Self, ConfirmationLayerError> {
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        Ok(Self {
            chains: Vec::new(),
            current_block: BlockId("0".to_string()),
            block_interval: interval,
            pending_txs: HashMap::new(),
            subblocks: HashMap::new(),
            receiver_from_hs: Some(receiver_from_hs),
            sender_to_hig: Some(sender_to_hig),
        })
    }

    /// Start the message processing loop
    pub async fn start(&mut self) {
        let mut receiver = self.receiver_from_hs.take().expect("Receiver already taken");
        while let Some(message) = receiver.receive().await {
            match message {
                Message::Message(proposal) => {
                    tracing::info!("Received transaction from HS: {:?}", proposal.cl_transaction);
                    if let Err(e) = self.submit_transaction(proposal.cl_transaction).await {
                        tracing::error!("Failed to process transaction: {}", e);
                    }
                }
            }
        }
    }

    /// Start the block production loop
    pub async fn start_block_production(&mut self) {
        let mut interval = interval(self.block_interval.clone());
        loop {
            interval.tick().await;
            let current_block = self.current_block.clone();

            // Process each chain
            for (chain_id, chain_txs) in self.pending_txs.iter_mut() {
                if !chain_txs.is_empty() {
                    // Create a new subblock
                    let subblock = SubBlock {
                        block_id: current_block.clone(),
                        chain_id: chain_id.clone(),
                        transactions: chain_txs.iter().map(|tx| Transaction {
                            id: tx.id.clone(),
                            data: tx.data.clone(),
                        }).collect(),
                    };
                    self.subblocks.insert((chain_id.clone(), current_block.clone()), subblock.clone());
                    
                    // Send subblock to HIG
                    if let Some(sender) = &self.sender_to_hig {
                        if let Err(e) = sender.send(SubBlockMessage { subblock }).await {
                            tracing::error!("Failed to send subblock to HIG: {}", e);
                        }
                    }
                }
            }

            // Increment block ID
            let next_block = (current_block.0.parse::<u64>().unwrap() + 1).to_string();
            self.current_block = BlockId(next_block);
        }
    }
}

#[async_trait::async_trait]
impl ConfirmationLayer for ConfirmationLayerNode {
    async fn register_chain(&mut self, chain_id: ChainId) -> Result<BlockId, ConfirmationLayerError> {
        self.chains.push(chain_id);
        Ok(self.current_block.clone())
    }

    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        let chain_txs = self.pending_txs.entry(transaction.chain_id.clone())
            .or_insert_with(Vec::new);
        chain_txs.push(transaction);
        Ok(())
    }

    async fn get_subblock(&self, chain_id: ChainId, block_id: BlockId) -> Result<SubBlock, ConfirmationLayerError> {
        self.subblocks.get(&(chain_id.clone(), block_id.clone()))
            .cloned()
            .ok_or_else(|| ConfirmationLayerError::SubBlockNotFound(chain_id, block_id))
    }

    async fn get_current_block(&self) -> Result<BlockId, ConfirmationLayerError> {
        Ok(self.current_block.clone())
    }

    async fn get_registered_chains(&self) -> Result<Vec<ChainId>, ConfirmationLayerError> {
        Ok(self.chains.clone())
    }

    async fn set_block_interval(&mut self, interval: Duration) -> Result<(), ConfirmationLayerError> {
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        self.block_interval = interval;
        Ok(())
    }

    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError> {
        Ok(self.block_interval.clone())
    }
} 