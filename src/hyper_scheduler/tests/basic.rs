use crate::{
    types::{CATId, CATStatusLimited, CATStatus, CATStatusUpdate, constants},
    hyper_scheduler::{node::HyperSchedulerNode, HyperScheduler, HyperSchedulerError},
};
use tokio::sync::mpsc;
use hyperplane::utils::logging;

// create a HyperSchedulerNode with empty channels
fn setup_hs_node() -> HyperSchedulerNode {
    let (sender_to_cl, _) = mpsc::channel(100);
    HyperSchedulerNode::new(sender_to_cl)
}

// create a HyperSchedulerNode with two registered chains
async fn setup_hs_node_with_chains() -> (HyperSchedulerNode, mpsc::Sender<CATStatusUpdate>, mpsc::Sender<CATStatusUpdate>) {
    let mut hs_node = setup_hs_node();
    logging::log("TEST", "HyperSchedulerNode created");

    // Register both chains
    let (sender_1, receiver_1) = mpsc::channel(1);
    let (sender_2, receiver_2) = mpsc::channel(1);
    hs_node.register_chain(constants::chain_1(), receiver_1).await.expect("Failed to register chain-1");
    hs_node.register_chain(constants::chain_2(), receiver_2).await.expect("Failed to register chain-2");

    // Verify both chains are registered
    let registered_chains = hs_node.get_registered_chains().await.expect("Failed to get registered chains");
    assert!(registered_chains.contains(&constants::chain_1()));
    assert!(registered_chains.contains(&constants::chain_2()));

    (hs_node, sender_1, sender_2)
}

/// Test that receiving a CAT for an unregistered chain returns an error
#[tokio::test]
async fn test_receive_cat_for_unregistered_chain() {
    logging::log("TEST", "=== Starting test_receive_cat_for_unregistered_chain ===");
    
    let mut hs_node = setup_hs_node();
    logging::log("TEST", "HyperSchedulerNode created");

    // Register chain-1
    let (_sender_1, receiver_1) = mpsc::channel(100);
    hs_node.register_chain(constants::chain_1(), receiver_1).await.expect("Failed to register chain-1");

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status_proposed = CATStatusLimited::Success;
    let constituent_chains = vec![constants::chain_1(), constants::chain_2()];
    logging::log("TEST", &format!("Created cat-id='{}' with status: {:?}", cat_id.0, status_proposed));

    // Try to process the status proposal directly
    logging::log("TEST", "Processing CAT status proposal...");
    let result = hs_node.process_cat_status_proposal(
        cat_id.clone(),
        constants::chain_1(),
        constituent_chains.clone(),
        status_proposed.clone()
    ).await;
    
    // Verify we got an error about unregistered chain
    assert!(result.is_err(), "Expected error since chain-2 is not registered");
    if let Err(HyperSchedulerError::InvalidCATProposal(msg)) = result {
        assert!(msg.contains("not registered"), "Expected error about unregistered chain");
    } else {
        panic!("Expected InvalidCATProposal error");
    }
    logging::log("TEST", "Verified error since chain-2 is not registered");

    logging::log("TEST", "=== Test completed successfully ===");
}

/// Test receiving a success proposal for a CAT
/// - Verify CAT is stored with pending status
#[tokio::test]
async fn test_receive_success_proposal_first_message() {
    logging::log("TEST", "\n=== Starting test_receive_success_proposal_first_message ===");
    
    let (mut hs_node, _sender_1, _sender_2) = setup_hs_node_with_chains().await;

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status_proposal = CATStatusLimited::Success;
    let constituent_chains = vec![constants::chain_1(), constants::chain_2()];
    logging::log("TEST", &format!("Created cat-id='{}' with status: {:?}", cat_id.0, status_proposal));

    // Process the status proposal directly
    logging::log("TEST", "Processing CAT status proposal...");
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        constants::chain_1(),
        constituent_chains.clone(),
        status_proposal.clone()
    ).await.expect("Failed to process status proposal");
    
    // Verify HS stored the status
    let status_stored = hs_node.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    logging::log("TEST", &format!("Retrieved stored status: {:?}", status_stored));
    assert_eq!(status_stored, CATStatus::Pending);
    logging::log("TEST", "Status verification successful");
    
    logging::log("TEST", "=== Test completed successfully ===");
}

/// Test receiving a failure proposal for a two-chain CAT
/// - Verify CAT is stored with failure status
#[tokio::test]
async fn test_receive_failure_proposal_first_message() {
    logging::log("TEST", "\n=== Starting test_receive_failure_proposal_first_message ===");
    
    let (mut hs_node, _sender_1, _sender_2) = setup_hs_node_with_chains().await;

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status_proposed = CATStatusLimited::Failure;
    let constituent_chains = vec![constants::chain_1(), constants::chain_2()];
    logging::log("TEST", &format!("Created cat-id='{}' with status: {:?}", cat_id.0, status_proposed));

    // Process the status proposal directly
    logging::log("TEST", "Processing CAT status proposal...");
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        constants::chain_1(),
        constituent_chains.clone(),
        status_proposed.clone()
    ).await.expect("Failed to process status proposal");

    // Verify HS stored the status
    let stored_status = hs_node.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    logging::log("TEST", &format!("Retrieved stored status: {:?}", stored_status));
    assert_eq!(stored_status, CATStatus::Failure);
    logging::log("TEST", "Status verification successful");

    // Verify CAT is in pending list
    let pending_cats = hs_node.get_pending_cats()
        .await
        .expect("Failed to get pending CATs");
    logging::log("TEST", &format!("Retrieved pending CATs: {:?}", pending_cats));
    assert!(pending_cats.contains(&cat_id));
    logging::log("TEST", "Pending CAT verification successful");
    
    logging::log("TEST", "=== Test completed successfully ===");
}

/// Test rejecting duplicate proposals
/// - Verify that proposals create records
/// - Verify that duplicate proposals are rejected
#[tokio::test]
async fn test_duplicate_rejection() {
    logging::log("TEST", "\n=== Starting test_duplicate_rejection ===");
    
    let (mut hs_node, _sender_1, _sender_2) = setup_hs_node_with_chains().await;

    // Test proposal behavior
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Success;
    let constituent_chains = vec![constants::chain_1(), constants::chain_2()];
    
    // First proposal should create a record
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        constants::chain_1(),
        constituent_chains.clone(),
        status.clone()
    ).await.expect("Failed to process first proposal");

    // Second proposal should be rejected
    let result = hs_node.process_cat_status_proposal(
        cat_id.clone(),
        constants::chain_1(),
        constituent_chains.clone(),
        status.clone()
    ).await;
    assert!(result.is_err(), "Expected duplicate proposal to be rejected");
    logging::log("TEST", "Verified duplicate proposal was rejected");
    
    logging::log("TEST", "=== Test completed successfully ===");
}

/// Test processing proposals for a two-chain CAT
/// - Receive a success proposal from first chain
/// - Receive a success proposal from second chain
/// - Verify that the CAT status is updated to success
#[tokio::test]
async fn test_process_proposals_for_two_chain_cat() {
    logging::log("TEST", "\n=== Starting test_process_proposals_for_two_chain_cat ===");

    let (mut hs_node, _sender_1, _sender_2) = setup_hs_node_with_chains().await;

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Success;
    let constituent_chains = vec![constants::chain_1(), constants::chain_2()];
    logging::log("TEST", &format!("Created cat-id='{}' with status: {:?}", cat_id.0, status));

    // Process status proposal from first chain
    logging::log("TEST", "Processing CAT status proposal from first chain...");
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        constants::chain_1(),
        constituent_chains.clone(),
        status.clone()
    ).await.expect("Failed to process first proposal");

    // Process status proposal from second chain
    logging::log("TEST", "Processing CAT status proposal from second chain...");
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        constants::chain_2(),
        constituent_chains.clone(),
        status.clone()
    ).await.expect("Failed to process second proposal");

    // Verify the CAT status is updated to success
    let stored_status = hs_node.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    logging::log("TEST", &format!("Retrieved stored status: {:?}", stored_status));
    assert_eq!(stored_status, CATStatus::Success);
    logging::log("TEST", "Status verification successful");

    logging::log("TEST", "=== Test completed successfully ===");
}

/// Test that a CAT cannot be set to Success if constituent chains don't match
/// - Set the CAT to Success with chains 1 and 2
/// - Attempt to set the CAT to Success with chains 1 only
/// - Verify that the CAT status is not changed
#[tokio::test]
async fn test_cannot_set_success_if_constituent_chains_dont_match() {
    logging::log("TEST", "\n=== Starting test_cannot_set_success_if_constituent_chains_dont_match ===");
    
    let (mut hs_node, _sender_1, _sender_2) = setup_hs_node_with_chains().await;
    let chain_id_3 = constants::chain_3();
    let cat_id = CATId("test-cat".to_string());

    // register also chain-3
    let (_sender_3, receiver_3) = mpsc::channel(1);
    hs_node.register_chain(chain_id_3.clone(), receiver_3).await.expect("Failed to register chain-3");
    
    // First proposal with chains 1 and 2
    let constituent_chains_1 = vec![constants::chain_1(), constants::chain_2()];
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        constants::chain_1(),
        constituent_chains_1.clone(),
        CATStatusLimited::Success
    ).await.expect("Failed to process first proposal");

    // Try to set Success with different constituent chains
    let constituent_chains_2 = vec![constants::chain_2(), chain_id_3.clone()];
    let result = hs_node.process_cat_status_proposal(
        cat_id.clone(),
        constants::chain_2(),
        constituent_chains_2,
        CATStatusLimited::Success
    ).await;

    assert!(result.is_err(), "Should not be able to set Success with different constituent chains");
    if let Err(HyperSchedulerError::ConstituentChainsMismatch { expected, received }) = result {
        assert_eq!(expected, constituent_chains_1, "Expected first set of constituent chains");
        assert_eq!(received, vec![constants::chain_2(), chain_id_3.clone()], "Expected second set of constituent chains");
    } else {
        panic!("Expected ConstituentChainsMismatch error");
    }
}