use crate::types::{CATId, TransactionId, StatusLimited, CLTransaction, ChainId, CATStatusUpdate, CATStatus};
use super::{HyperScheduler, HyperSchedulerError};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio;

/// The internal state of the HyperSchedulerNode
pub struct HyperSchedulerState {
    /// The chain IDs of valid chains
    pub registered_chains: HashSet<ChainId>,
    /// Map of CAT IDs to their constituent chains
    pub constituent_chains: HashMap<CATId, Vec<ChainId>>,
    /// Map of CAT IDs to their status (this is the result of the cat_chainwise_statuses)
    pub cat_statuses: HashMap<CATId, CATStatus>,
    /// Map of CAT IDs to their status per constituent chain
    pub cat_chainwise_statuses: HashMap<CATId, HashMap<ChainId, StatusLimited>>,
}

/// A node that implements the HyperScheduler trait
pub struct HyperSchedulerNode {
    /// The internal state of the node
    pub state: Arc<Mutex<HyperSchedulerState>>,
    /// Receiver for messages from Hyper IG
    pub receiver_from_hig_1: Option<mpsc::Receiver<CATStatusUpdate>>,
    pub receiver_from_hig_2: Option<mpsc::Receiver<CATStatusUpdate>>,
    /// Sender for messages to CL
    pub sender_to_cl: Option<mpsc::Sender<CLTransaction>>,
}

impl Clone for HyperSchedulerNode {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            receiver_from_hig_1: None, // Can't clone receiver
            receiver_from_hig_2: None, // Can't clone receiver
            sender_to_cl: self.sender_to_cl.clone(),
        }
    }
}

impl HyperSchedulerNode {
    /// Create a new HyperSchedulerNode
    pub fn new(receiver_from_hig_1: mpsc::Receiver<CATStatusUpdate>, receiver_from_hig_2: mpsc::Receiver<CATStatusUpdate>, sender_to_cl: mpsc::Sender<CLTransaction>) -> Self {
        Self {
            state: Arc::new(Mutex::new(HyperSchedulerState {
                cat_statuses: HashMap::new(),
                registered_chains: HashSet::new(),
                constituent_chains: HashMap::new(),
                cat_chainwise_statuses: HashMap::new(),
            })),
            receiver_from_hig_1: Some(receiver_from_hig_1),
            receiver_from_hig_2: Some(receiver_from_hig_2),
            sender_to_cl: Some(sender_to_cl),
        }
    }

    /// Get a clone of the sender to the confirmation layer
    pub async fn get_sender_to_cl(&self) -> mpsc::Sender<CLTransaction> {
        self.sender_to_cl.as_ref().expect("Sender to CL not set").clone()
    }

    /// Process messages without holding the node lock
    pub async fn process_messages(hs_node: Arc<Mutex<HyperSchedulerNode>>, which_receiver: u8) {
        // println!("  [HS]   [Message loop task] Attempting to acquire hs_node lock...");
        let mut node = hs_node.lock().await;
        // println!("  [HS]   [Message loop task] Acquired hs_node lock");
        let mut receiver = match which_receiver {
            1 => node.receiver_from_hig_1.take().expect("Receiver already taken"),
            2 => node.receiver_from_hig_2.take().expect("Receiver already taken"),
            _ => panic!("Invalid receiver index"),
        };
        drop(node); // Release the lock before starting the loop
        // println!("  [HS]   [Message loop task] Released hs_node lock");
        
        // Process messages
        while let Some(status_update) = receiver.recv().await {
            println!("  [HS]   [Message loop task] Received status proposal for cat-id='{}': data='{:?}' : chains='{:?}'", status_update.cat_id, status_update.status, status_update.constituent_chains);

            let mut node = hs_node.lock().await;
            if let Err(e) = node.process_cat_status_proposal(
                status_update.cat_id.clone(),
                status_update.chain_id.clone(),
                status_update.constituent_chains.clone(),
                status_update.status.clone()
            ).await {
                println!("  [HS]   Failed to process status proposal: {:?}", e);
                continue;
            }

            // Send status update to CL
            if let Err(e) = node.send_cat_status_update(status_update.cat_id.clone(), status_update.constituent_chains.clone(), status_update.status.clone()).await {
                println!("  [HS]   Failed to send status update: {:?}", e);
            }
        }
        println!("  [HS]   [Message loop task] Message processing loop exiting");
    }

    /// Start the message processing loop 
    pub async fn start(node: Arc<Mutex<HyperSchedulerNode>>) {
        let node1 = node.clone();
        let node2 = node.clone();
        tokio::spawn(async move { HyperSchedulerNode::process_messages(node1, 1).await });
        tokio::spawn(async move { HyperSchedulerNode::process_messages(node2, 2).await });
    }

    /// Register a chain with the HyperSchedulerNode
    pub async fn register_chain(&mut self, chain_id: ChainId) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if state.registered_chains.contains(&chain_id) {
            return Err(format!("   [HS]   Chain {} already registered", chain_id.0));
        }
        state.registered_chains.insert(chain_id.clone());
        println!("   [HS]   Chain {} registered", chain_id.0);
        Ok(())
    }



    /// Submit a transaction to the confirmation layer
    pub async fn submit_transaction_to_cl(&mut self, tx: CLTransaction) -> Result<(), String> {
        println!("  [HS]   submit_transaction called for transaction: id={}, data={}, chain_ids={:?}", 
            tx.id.0, tx.data, tx.constituent_chains.iter().map(|c| c.0.clone()).collect::<Vec<_>>());
        if let Some(sender) = &self.sender_to_cl {
            sender.send(tx).await.map_err(|e| e.to_string())
        } else {
            Err("No sender to CL set".to_string())
        }
    }
}

#[async_trait]
impl HyperScheduler for HyperSchedulerNode {
    async fn get_cat_status(&self, id: CATId) -> Result<CATStatus, HyperSchedulerError> {
        println!("  [HS]   get_cat_status called for tx-id='{}'", id.0);
        // println!("  [HS]   Attempting to acquire state lock for get_cat_status...");
        let result = {
            let state = self.state.lock().await;
            // println!("  [HS]   Acquired state lock for get_cat_status");
            let result = state.cat_statuses.get(&id).cloned();
            // println!("  [HS]   Retrieved status, releasing lock");
            result
        };
        // println!("  [HS]   Released state lock after get_cat_status");
        if let Some(ref status) = result {
            println!("  [HS]   get_cat_status found status for tx-id='{}': {:?}", id.0, status);
        } else {
            println!("  [HS]   get_cat_status did not find status for tx-id='{}'", id.0);
        }
        result.ok_or_else(|| HyperSchedulerError::CATNotFound(id))
    }

    async fn get_pending_cats(&self) -> Result<Vec<CATId>, HyperSchedulerError> {
        Ok(self.state.lock().await.cat_statuses.keys().cloned().collect())
    }
    /// Process a status proposal for a CAT
    /// 
    /// cat_id: the ID of the CAT
    /// this_chain_id: the ID of the chain that is proposing the status
    /// constituent_chains: the IDs of the chains that are part of the CAT
    /// status: the status that the proposing chain is proposing
    async fn process_cat_status_proposal(&mut self, cat_id: CATId, this_chain_id: ChainId, constituent_chains: Vec<ChainId>, status: StatusLimited) -> Result<(), HyperSchedulerError> {
        println!("  [HS]   process_cat_status_proposal called for cat-id='{}' by chain-id='{}' with status {:?}", cat_id.0, this_chain_id.0, status);
        let mut state = self.state.lock().await;
        
        // Check if CAT proposal already exists
        if let Some(chain_statuses) = state.cat_chainwise_statuses.get(&cat_id) {
            if chain_statuses.contains_key(&this_chain_id) {
                println!("  [HS]   CAT {} already exists, rejecting duplicate proposal", cat_id.0);
                return Err(HyperSchedulerError::DuplicateProposal(cat_id));
            }
        }
        
        // Store the status proposal - this should never fail as the map is initialized in new()
        state.cat_chainwise_statuses.entry(cat_id.clone()).or_insert_with(HashMap::new).insert(this_chain_id.clone(), status.clone());
        println!("  [HS]   Proposal for {} from {} set to {:?}", cat_id.0, this_chain_id.0, status);

        // when reaching this point the cat should not be set to success. this is a severe bug so we should panic
        if matches!(state.cat_statuses.get(&cat_id), Some(CATStatus::Success)) {
            panic!("  [HS]   Cat status is already set to success. This is a severe bug, please fix immediately.");
        }

        // if the cat is already set to failure, we don't need to do anything
        if matches!(state.cat_statuses.get(&cat_id), Some(CATStatus::Failure)) {
            println!("  [HS]   CAT {} is already set to failure, skipping", cat_id.0);
            return Ok(());
        // if the proposal is failure, we set the status of the cat itself to failure
        } else if status == StatusLimited::Failure {
            state.cat_statuses.insert(cat_id.clone(), CATStatus::Failure);
            println!("  [HS]   Status for {} set to {:?}", cat_id.0, CATStatus::Failure);
            state.constituent_chains.insert(cat_id.clone(), constituent_chains.clone());
            println!("  [HS]   Constituent chains for {} set to {:?}", cat_id.0, constituent_chains);
        // if the cat does not exist in cat_statuses, we need to add it
        } else if !state.cat_statuses.contains_key(&cat_id) {
            // since this cat is new, and we need two chains to be successful, we set the status to Pending
            state.cat_statuses.insert(cat_id.clone(), CATStatus::Pending);
            println!("  [HS]   Status for {} set to {:?}", cat_id.0, CATStatus::Pending);
            state.constituent_chains.insert(cat_id.clone(), constituent_chains.clone());
            println!("  [HS]   Constituent chains for {} set to {:?}", cat_id.0, constituent_chains);
        // if the cat proposal already exists, we need to check if all chains have submitted their status
        } else {
            // the cat status should be pending at this point
            if !matches!(state.cat_statuses.get(&cat_id), Some(CATStatus::Pending)) {
                return Err(HyperSchedulerError::Internal("Cat status is not pending".to_string()));
            }
            // the constituent chains recorded for the cat should be the same as the ones in the proposal
            if state.constituent_chains.get(&cat_id) != Some(&constituent_chains) {
                return Err(HyperSchedulerError::Internal("Constituent chains do not match".to_string()));
            }
            // we need to check if the proposed statuses in cat_chainwise_statuses are all present and set to success for all constituent chains
            for chain_id in constituent_chains {
                if !matches!(state.cat_chainwise_statuses.get(&cat_id).unwrap().get(&chain_id), Some(&StatusLimited::Success)) {
                    return Err(HyperSchedulerError::Internal("Not all chains have submitted a success status".to_string()));
                }
                // all is well and complete. Set the status of the cat to success
                state.cat_statuses.insert(cat_id.clone(), CATStatus::Success);
                println!("  [HS]   Status for {} set to {:?}", cat_id.0, CATStatus::Success);
            }
        }

        Ok(())
    }

    async fn send_cat_status_update(&mut self, cat_id: CATId, constituent_chains: Vec<ChainId>, status: StatusLimited) -> Result<(), HyperSchedulerError> {
        println!("  [HS]   send_cat_status_update called for CAT {}", cat_id.0);

        let status_str = match status {
            StatusLimited::Success => "STATUS_UPDATE:Success.CAT_ID:".to_string() + &cat_id.0,
            StatusLimited::Failure => "STATUS_UPDATE:Failure.CAT_ID:".to_string() + &cat_id.0,
        };

        // Send the status update to the confirmation layer
        if let Some(sender) = &self.sender_to_cl {
            let tx = CLTransaction {
                id: TransactionId(cat_id.0.clone()+".UPDATE"),
                data: status_str.clone(),
                constituent_chains: constituent_chains.clone(),
            };
            println!("  [HS]   Submitting status update transaction to CL: id={}, data={}, chain_ids={:?}", tx.id.0, tx.data, tx.constituent_chains.iter().map(|c| c.0.clone()).collect::<Vec<_>>());
            sender.send(tx)
                .await
                .map_err(|e| HyperSchedulerError::Internal(e.to_string()))?;
        } else {
            println!("  [HS]   No sender to CL set, cannot send status update");
            return Err(HyperSchedulerError::Internal("No sender to CL set".to_string()));
        }

        Ok(())
    }
} 