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
pub async fn setup_test_nodes_with_block_production_choice(block_interval: Duration, start_block_production: bool) -> (Arc<Mutex<HyperSchedulerNode>>, Arc<Mutex<ConfirmationLayerNode>>, Arc<Mutex<HyperIGNode>>) {
    // Create channels for communication
    let (sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel(100);
    let (sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel(100);
    let (sender_hig_to_hs, receiver_hig_to_hs) = mpsc::channel(100);
    
    // Create nodes with their channels
    let hs_node = Arc::new(Mutex::new(HyperSchedulerNode::new(receiver_hig_to_hs, sender_hs_to_cl)));
    let cl_node = Arc::new(Mutex::new(ConfirmationLayerNode::new_with_block_interval(
        receiver_hs_to_cl,
        sender_cl_to_hig,
        block_interval
    ).expect("Failed to create confirmation node")));
    let hig_node = Arc::new(Mutex::new(HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs)));

    // Start the HS incoming message processing loop
    let hs_node_for_message_loop = hs_node.clone();
    let _hs_message_loop_handle = tokio::spawn(async move {
        HyperSchedulerNode::process_messages(hs_node_for_message_loop).await;
    });

    // Start the HIG incoming block processing loop
    let hig_node_for_message_loop = hig_node.clone();
    let _hig_message_loop_handle = tokio::spawn(async move {
        let mut node = hig_node_for_message_loop.lock().await;
        node.start().await;
    });

    // Start block production if requested (default to true)
    if start_block_production {
        // Clone the state for block production
        let cl_node_for_block_production = cl_node.clone();
        let _block_production_handle = tokio::spawn(async move {
            ConfirmationLayerNode::start_block_production(cl_node_for_block_production).await;
        });

        // Wait for block production to be ready
        let mut attempts = 0;
        while attempts < 10 {
            if let Ok(interval) = cl_node.lock().await.get_block_interval().await {
                if interval == block_interval {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }
    }

    (hs_node, cl_node, hig_node)
}

/// Helper function to create test nodes with basic setup and default block production
pub async fn setup_test_nodes(block_interval: Duration) -> (Arc<Mutex<HyperSchedulerNode>>, Arc<Mutex<ConfirmationLayerNode>>, Arc<Mutex<HyperIGNode>>) {
    setup_test_nodes_with_block_production_choice(block_interval, true).await
}

pub async fn setup_test_nodes_no_block_production() -> (Arc<Mutex<HyperSchedulerNode>>, Arc<Mutex<ConfirmationLayerNode>>, Arc<Mutex<HyperIGNode>>) {
    setup_test_nodes_with_block_production_choice(Duration::from_millis(100), false).await
}

