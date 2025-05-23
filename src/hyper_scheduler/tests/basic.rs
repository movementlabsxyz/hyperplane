use crate::{
    types::{CATId, CATStatusLimited, ChainId, CATStatus, CATStatusUpdate},
    hyper_scheduler::{node::HyperSchedulerNode, HyperScheduler, HyperSchedulerError},
};
use tokio::sync::mpsc;

// create a HyperSchedulerNode with empty channels
fn setup_hs_node() -> HyperSchedulerNode {
    let (sender_to_cl, _) = mpsc::channel(100);
    HyperSchedulerNode::new(sender_to_cl)
}

// create a HyperSchedulerNode with two registered chains
async fn setup_hs_node_with_chains() -> (HyperSchedulerNode, mpsc::Sender<CATStatusUpdate>, mpsc::Sender<CATStatusUpdate>) {
    let mut hs_node = setup_hs_node();
    println!("[TEST]   HyperSchedulerNode created");

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Register both chains
    let (sender_1, receiver_1) = mpsc::channel(1);
    let (sender_2, receiver_2) = mpsc::channel(1);
    hs_node.register_chain(chain_id_1.clone(), receiver_1).await.expect("Failed to register chain-1");
    hs_node.register_chain(chain_id_2.clone(), receiver_2).await.expect("Failed to register chain-2");

    // Verify both chains are registered
    let registered_chains = hs_node.get_registered_chains().await.expect("Failed to get registered chains");
    assert!(registered_chains.contains(&chain_id_1));
    assert!(registered_chains.contains(&chain_id_2));

    (hs_node, sender_1, sender_2)
}

/// Test that receiving a CAT for an unregistered chain returns an error
#[tokio::test]
async fn test_receive_cat_for_unregistered_chain() {
    println!("\n=== Starting test_receive_cat_for_unregistered_chain ===");
    
    let mut hs_node = setup_hs_node();
    println!("[TEST]   HyperSchedulerNode created");

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Register chain-1
    let (_sender_1, receiver_1) = mpsc::channel(100);
    hs_node.register_chain(chain_id_1.clone(), receiver_1).await.expect("Failed to register chain-1");

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status_proposed = CATStatusLimited::Success;
    let constituent_chains = vec![chain_id_1.clone(), chain_id_2.clone()];
    println!("[TEST]   Created cat-id='{}' with status: {:?}", cat_id.0, status_proposed);

    // Try to process the status proposal directly
    println!("[TEST]   Processing CAT status proposal...");
    let result = hs_node.process_cat_status_proposal(
        cat_id.clone(),
        chain_id_1.clone(),
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
    println!("[TEST]   Verified error since chain-2 is not registered");

    println!("=== Test completed successfully ===\n");
}

/// Test receiving a success proposal for a CAT
/// - Verify CAT is stored with pending status
#[tokio::test]
async fn test_receive_success_proposal_first_message() {
    println!("\n=== Starting test_receive_success_proposal_first_message ===");
    
    let (mut hs_node, _sender_1, _sender_2) = setup_hs_node_with_chains().await;

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status_proposal = CATStatusLimited::Success;
    let constituent_chains = vec![chain_id_1.clone(), chain_id_2.clone()];
    println!("[TEST]   Created cat-id='{}' with status: {:?}", cat_id.0, status_proposal);

    // Process the status proposal directly
    println!("[TEST]   Processing CAT status proposal...");
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        chain_id_1.clone(),
        constituent_chains.clone(),
        status_proposal.clone()
    ).await.expect("Failed to process status proposal");
    
    // Verify HS stored the status
    let status_stored = hs_node.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    println!("[TEST]   Retrieved stored status: {:?}", status_stored);
    assert_eq!(status_stored, CATStatus::Pending);
    println!("[TEST]   Status verification successful");
    
    println!("=== Test completed successfully ===\n");
}

/// Test receiving a failure proposal for a two-chain CAT
/// - Verify CAT is stored with failure status
#[tokio::test]
async fn test_receive_failure_proposal_first_message() {
    println!("\n=== Starting test_receive_failure_proposal_first_message ===");
    
    let (mut hs_node, _sender_1, _sender_2) = setup_hs_node_with_chains().await;

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status_proposed = CATStatusLimited::Failure;
    let constituent_chains = vec![chain_id_1.clone(), chain_id_2.clone()];
    println!("[TEST]   Created cat-id='{}' with status: {:?}", cat_id.0, status_proposed);

    // Process the status proposal directly
    println!("[TEST]   Processing CAT status proposal...");
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        chain_id_1.clone(),
        constituent_chains.clone(),
        status_proposed.clone()
    ).await.expect("Failed to process status proposal");

    // Verify HS stored the status
    let stored_status = hs_node.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    println!("[TEST]   Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, CATStatus::Failure);
    println!("[TEST]   Status verification successful");

    // Verify CAT is in pending list
    let pending_cats = hs_node.get_pending_cats()
        .await
        .expect("Failed to get pending CATs");
    println!("[TEST]   Retrieved pending CATs: {:?}", pending_cats);
    assert!(pending_cats.contains(&cat_id));
    println!("[TEST]   Pending CAT verification successful");
    
    println!("=== Test completed successfully ===\n");
}

/// Test rejecting duplicate proposals
/// - Verify that proposals create records
/// - Verify that duplicate proposals are rejected
#[tokio::test]
async fn test_duplicate_rejection() {
    println!("\n=== Starting test_duplicate_rejection ===");
    
    let (mut hs_node, _sender_1, _sender_2) = setup_hs_node_with_chains().await;

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Test proposal behavior
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Success;
    let constituent_chains = vec![chain_id_1.clone(), chain_id_2.clone()];
    
    // First proposal should create a record
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        chain_id_1.clone(),
        constituent_chains.clone(),
        status.clone()
    ).await.expect("Failed to process first proposal");

    // Second proposal should be rejected
    let result = hs_node.process_cat_status_proposal(
        cat_id.clone(),
        chain_id_1.clone(),
        constituent_chains.clone(),
        status.clone()
    ).await;
    assert!(result.is_err(), "Expected duplicate proposal to be rejected");
    println!("[TEST]   Verified duplicate proposal was rejected");
    
    println!("=== Test completed successfully ===\n");
}

/// Test processing proposals for a two-chain CAT
/// - Receive a success proposal from first chain
/// - Receive a success proposal from second chain
/// - Verify that the CAT status is updated to success
#[tokio::test]
async fn test_process_proposals_for_two_chain_cat() {
    println!("\n=== Starting test_process_proposals_for_two_chain_cat ===");

    let (mut hs_node, _sender_1, _sender_2) = setup_hs_node_with_chains().await;

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Success;
    let constituent_chains = vec![chain_id_1.clone(), chain_id_2.clone()];
    println!("[TEST]   Created cat-id='{}' with status: {:?}", cat_id.0, status);

    // Process status proposal from first chain
    println!("[TEST]   Processing CAT status proposal from first chain...");
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        chain_id_1.clone(),
        constituent_chains.clone(),
        status.clone()
    ).await.expect("Failed to process first proposal");

    // Process status proposal from second chain
    println!("[TEST]   Processing CAT status proposal from second chain...");
    hs_node.process_cat_status_proposal(
        cat_id.clone(),
        chain_id_2.clone(),
        constituent_chains.clone(),
        status.clone()
    ).await.expect("Failed to process second proposal");

    // Verify the CAT status is updated to success
    let stored_status = hs_node.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    println!("[TEST]   Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, CATStatus::Success);
    println!("[TEST]   Status verification successful");

    println!("=== Test completed successfully ===\n");
}