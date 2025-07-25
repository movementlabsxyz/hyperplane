use hyperplane::{
    hyper_scheduler::node::HyperSchedulerNode,
    confirmation_layer::node::ConfirmationLayerNode,
    confirmation_layer::ConfirmationLayer,
    hyper_ig::node::HyperIGNode,
    types::ChainId,
    utils::logging,
};
use tokio::time::Duration;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Helper function to create test nodes with basic setup
/// Returns a tuple of the nodes and the current block number at the end of the setup
pub async fn setup_test_nodes(block_interval: Duration) 
-> (Arc<Mutex<HyperSchedulerNode>>, Arc<Mutex<ConfirmationLayerNode>>, Arc<Mutex<HyperIGNode>>, Arc<Mutex<HyperIGNode>>, u64) {
    setup_test_nodes_with_preloaded_accounts(block_interval, 0, 0).await
}

/// Helper function to create test nodes with preloaded accounts
/// Returns a tuple of the nodes and the current block number at the end of the setup
pub async fn setup_test_nodes_with_preloaded_accounts(block_interval: Duration, num_accounts: u32, preload_value: u32) 
-> (Arc<Mutex<HyperSchedulerNode>>, Arc<Mutex<ConfirmationLayerNode>>, Arc<Mutex<HyperIGNode>>, Arc<Mutex<HyperIGNode>>, u64) {
    // Initialize logging
    logging::init_logging();
    
    // Create channels for communication
    let (sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel(100);
    let (sender_hig1_to_hs, receiver_hig1_to_hs) = mpsc::channel(100);
    let (sender_hig2_to_hs, receiver_hig2_to_hs) = mpsc::channel(100);
    let (sender_cl_to_hig1, receiver_cl_to_hig1) = mpsc::channel(100);
    let (sender_cl_to_hig2, receiver_cl_to_hig2) = mpsc::channel(100);
    
    // Create nodes with their channels
    let hs_node = Arc::new(Mutex::new(HyperSchedulerNode::new(sender_hs_to_cl)));
    let cl_node = Arc::new(Mutex::new(ConfirmationLayerNode::new_with_block_interval(
        receiver_hs_to_cl,
        block_interval,
    ).expect("Failed to create confirmation node")));
    let hig_node_1 = Arc::new(Mutex::new(HyperIGNode::new_with_preloaded_accounts(receiver_cl_to_hig1, sender_hig1_to_hs, ChainId("chain-1".to_string()), 4, true, num_accounts, preload_value)));
    let hig_node_2 = Arc::new(Mutex::new(HyperIGNode::new_with_preloaded_accounts(receiver_cl_to_hig2, sender_hig2_to_hs, ChainId("chain-2".to_string()), 4, true, num_accounts, preload_value)));

    // Start the nodes
    HyperSchedulerNode::start(hs_node.clone()).await;
    HyperIGNode::start(hig_node_1.clone()).await;
    HyperIGNode::start(hig_node_2.clone()).await;
    ConfirmationLayerNode::start(cl_node.clone()).await;

    // Register chains in CL
    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());
    {
        let mut cl_node_guard = cl_node.lock().await;
        cl_node_guard.register_chain(chain_id_1.clone(), sender_cl_to_hig1).await.expect("Failed to register chain");
        cl_node_guard.register_chain(chain_id_2.clone(), sender_cl_to_hig2).await.expect("Failed to register chain");
    }

    // Register chains in HS
    {
        let mut hs_node_guard = hs_node.lock().await;
        hs_node_guard.register_chain(chain_id_1.clone(), receiver_hig1_to_hs).await.expect("Failed to register chain");
        hs_node_guard.register_chain(chain_id_2.clone(), receiver_hig2_to_hs).await.expect("Failed to register chain");
    }

    // Wait for block production to be ready
    let mut attempts = 0;
    while attempts < 10 {
        if let Ok(interval) = cl_node.lock().await.get_block_interval().await {
            if interval == block_interval {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        logging::log("TEST", &format!("Waiting for block production to be ready.. attempt: {}", attempts));
        attempts += 1;
    }

    // Wait a couple of blocks to ensure the block production is ready
    tokio::time::sleep(block_interval * 2).await;
    let current_block = cl_node.lock().await.get_current_block().await.unwrap();
    logging::log("NODES SETUP", &format!("Nodes setup complete, current block: {}", current_block));

    (hs_node, cl_node, hig_node_1, hig_node_2, current_block)
}
