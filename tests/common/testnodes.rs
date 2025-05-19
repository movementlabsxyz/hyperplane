use hyperplane::{
    hyper_scheduler::node::HyperSchedulerNode,
    confirmation_layer::node::ConfirmationLayerNode,
    confirmation_layer::ConfirmationLayer,
    hyper_ig::node::HyperIGNode,
};
use tokio::time::Duration;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Helper function to create test nodes with basic setup
/// Returns a tuple of the nodes and the current block number at the end of the setup
pub async fn setup_test_nodes_with_block_production_choice(block_interval: Duration, start_block_production: bool) 
-> (Arc<Mutex<HyperSchedulerNode>>, Arc<Mutex<ConfirmationLayerNode>>, Arc<Mutex<HyperIGNode>>, u64) {
    // Create channels for communication
    let (sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel(100);
    let (sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel(100);
    let (sender_hig_to_hs, receiver_hig_to_hs) = mpsc::channel(100);
    
    // Create nodes with their channels
    let hs_node = Arc::new(Mutex::new(HyperSchedulerNode::new(receiver_hig_to_hs, sender_hs_to_cl)));
    let cl_node = Arc::new(Mutex::new(ConfirmationLayerNode::new_with_block_interval(receiver_hs_to_cl,sender_cl_to_hig,block_interval).expect("Failed to create confirmation node")));
    let hig_node = Arc::new(Mutex::new(HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs)));

    // Start the HyperScheduler and HyperIG nodes
    HyperSchedulerNode::start(hs_node.clone()).await;
    HyperIGNode::start(hig_node.clone()).await;

    // Start the Confirmation Layer node (block production, default to true)
    if start_block_production {
        ConfirmationLayerNode::start(cl_node.clone()).await;

        // Wait for block production to be ready
        let mut attempts = 0;
        while attempts < 10 {
            if let Ok(interval) = cl_node.lock().await.get_block_interval().await {
                if interval == block_interval {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("[TEST]   Waiting for block production to be ready.. attempt: {}", attempts);
            attempts += 1;
        }
    }

    // Wait a couple of blocks to ensure the block production is ready
    tokio::time::sleep(block_interval * 2).await;
    println!("  [NODES SETUP]   Nodes setup complete, current block: {}", cl_node.lock().await.get_current_block().await.unwrap());
    let current_block = cl_node.lock().await.get_current_block().await.unwrap();

    (hs_node, cl_node, hig_node, current_block)
}

/// Helper function to create test nodes with block production
/// Returns a tuple of the nodes and the current block number at the end of the setup
pub async fn setup_test_nodes(block_interval: Duration) -> (Arc<Mutex<HyperSchedulerNode>>, Arc<Mutex<ConfirmationLayerNode>>, Arc<Mutex<HyperIGNode>>, u64) {
    setup_test_nodes_with_block_production_choice(block_interval, true).await
}

/// Helper function to create test nodes with no block production
/// Returns a tuple of the nodes and the current block number at the end of the setup
pub async fn setup_test_nodes_no_block_production() -> (Arc<Mutex<HyperSchedulerNode>>, Arc<Mutex<ConfirmationLayerNode>>, Arc<Mutex<HyperIGNode>>, u64) {
    setup_test_nodes_with_block_production_choice(Duration::from_millis(100), false).await
}

