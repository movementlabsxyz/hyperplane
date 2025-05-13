use hyperplane::{
    types::communication::Channel,
    types::communication::hig_to_hs::CATStatusUpdateMessage,
    types::communication::hs_to_cl::CLTransactionMessage,
    types::communication::cl_to_hig::SubBlockMessage,
    hyper_scheduler::node::HyperSchedulerNode,
    confirmation_layer::node::ConfirmationLayerNode,
    hyper_ig::node::HyperIGNode,
};
use tokio::time::Duration;

pub fn setup_test_nodes() -> (HyperSchedulerNode, ConfirmationLayerNode, HyperIGNode) {
    // Create channels for communication
    let channel_hs_to_cl = Channel::<CLTransactionMessage>::new(100);
    let (sender_to_cl, receiver_from_hs) = channel_hs_to_cl.split();

    let channel_cl_to_hig = Channel::<SubBlockMessage>::new(100);
    let (sender_to_hig, receiver_from_cl) = channel_cl_to_hig.split();

    let channel_hig_to_hs = Channel::<CATStatusUpdateMessage>::new(100);
    let (sender_to_hs, receiver_from_hig) = channel_hig_to_hs.split();
    
    // Create nodes with their channels
    let hs_node = HyperSchedulerNode::new(receiver_from_hig, sender_to_cl);
    let cl_node = ConfirmationLayerNode::new_with_block_interval(
        receiver_from_hs, 
        sender_to_hig, 
        Duration::from_millis(100)
    ).expect("Failed to create confirmation node");
    let hig_node = HyperIGNode::new(receiver_from_cl, sender_to_hs);

    (hs_node, cl_node, hig_node)
}