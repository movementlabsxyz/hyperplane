//! Network setup and account initialization for the Hyperplane simulator.
//! 
//! Handles node creation, chain registration, and account funding verification.

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use hyperplane::{
    types::{ChainId, CLTransaction, SubBlock},
    confirmation_layer::ConfirmationLayerNode,
};

// ------------------------------------------------------------------------------------------------
// Network Setup
// ------------------------------------------------------------------------------------------------

/// Creates a network of nodes with the specified number of nodes and chains
pub async fn create_network(num_nodes: usize, num_chains: usize) -> Vec<Arc<Mutex<ConfirmationLayerNode>>> {
    let mut nodes = Vec::new();
    let mut senders = Vec::new();
    
    // Create nodes
    for _i in 0..num_nodes {
        let (tx, rx) = mpsc::channel::<CLTransaction>(100);
        let node = ConfirmationLayerNode::new(rx);
        nodes.push(Arc::new(Mutex::new(node)));
        senders.push(tx);
    }
    
    // Register chains on each node
    for i in 0..num_chains {
        let chain_id = ChainId(format!("chain-{}", i));
        for (node, _sender) in nodes.iter().zip(senders.iter()) {
            let mut node = node.lock().await;
            let (subblock_tx, _) = mpsc::channel::<SubBlock>(100);
            node.register_chain(chain_id.clone(), subblock_tx).await.expect("Failed to register chain");
        }
    }
    
    nodes
}

 