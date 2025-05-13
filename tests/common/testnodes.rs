use hyperplane::{
    types::{Transaction, TransactionId, CATStatusLimited, ChainId, BlockId, CLTransaction, SubBlock, CATStatusUpdate},
    hyper_scheduler::node::HyperSchedulerNode,
    confirmation_layer::node::{ConfirmationLayerNode, ConfirmationLayerNodeWrapper},
    confirmation_layer::ConfirmationLayer,
    hyper_ig::node::HyperIGNode,
};
use tokio::time::Duration;
use tokio::sync::mpsc;
use std::sync::Arc;

pub async fn setup_test_nodes(block_interval: Duration) -> (HyperSchedulerNode, ConfirmationLayerNodeWrapper, HyperIGNode) {
    // Create channels for communication
    let (sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel::<CLTransaction>(100);
    let (sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel::<SubBlock>(100);
    let (sender_hig_to_hs, receiver_hig_to_hs) = mpsc::channel::<CATStatusUpdate>(100);
    
    // Create nodes with their channels
    let hs_node = HyperSchedulerNode::new(receiver_hig_to_hs, sender_hs_to_cl.clone());
    let cl_node = ConfirmationLayerNode::new_with_block_interval(
        receiver_hs_to_cl,
        sender_cl_to_hig.clone(),
        block_interval
    ).expect("Failed to create confirmation node");
    let hig_node = HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs);

    // Create the wrapper and store the sender
    let wrapper = ConfirmationLayerNodeWrapper::new(cl_node);
    wrapper.set_sender_hs_to_cl(sender_hs_to_cl).await;

    // Start block production in a separate task
    let wrapper_for_block_production = wrapper.clone();
    let _block_production_handle = tokio::spawn(async move {
        wrapper_for_block_production.start_block_production().await;
    });

    // Wait for block production to be ready
    let mut attempts = 0;
    while attempts < 10 {
        if let Ok(interval) = wrapper.get_block_interval().await {
            if interval == block_interval {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    (hs_node, wrapper, hig_node)
}

