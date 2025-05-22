use crate::{
    types::{CATId, StatusLimited, ChainId, CATStatus},
    hyper_scheduler::{node::HyperSchedulerNode, HyperScheduler},
};
use tokio::sync::mpsc;

// create a HyperSchedulerNode with empty channels
fn create_hs_node() -> HyperSchedulerNode {
    let (_, receiver_from_hig_1) = mpsc::channel(1);
    let (_, receiver_from_hig_2) = mpsc::channel(1);
    let (sender_to_cl, _) = mpsc::channel(1);
    HyperSchedulerNode::new(receiver_from_hig_1, receiver_from_hig_2, sender_to_cl)
}


/// Test receiving a success proposal for a CAT
/// - Verify CAT is stored with pending status
#[tokio::test]
async fn test_receive_success_proposal_first_message() {
    println!("\n=== Starting test_receive_success_proposal_first_message ===");
    
    let mut hs_node = create_hs_node();
    println!("[TEST]   HyperSchedulerNode created");

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status_proposal = StatusLimited::Success;
    let constituent_chains = vec![ChainId("chain-1".to_string())];
    println!("[TEST]   Created CAT ID: {} with status: {:?}", cat_id.0, status_proposal);

    // Receive the status proposal directly
    println!("[TEST]   Receiving CAT status proposal...");
    hs_node.process_cat_status_proposal(cat_id.clone(), ChainId("chain-1".to_string()), constituent_chains.clone(), status_proposal.clone())
        .await
        .expect("Failed to receive CAT status proposal");
    println!("[TEST]   CAT status proposal received successfully");

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
    
    let mut hs_node = create_hs_node();
    println!("[TEST]   HyperSchedulerNode created");

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status_proposed = StatusLimited::Failure;
    let constituent_chains = vec![ChainId("chain-1".to_string())];
    println!("[TEST]   Created CAT ID: {} with status: {:?}", cat_id.0, status_proposed);

    // Receive the status proposal directly
    println!("[TEST]   Receiving CAT status proposal...");
    hs_node.process_cat_status_proposal(cat_id.clone(), ChainId("chain-1".to_string()), constituent_chains.clone(), status_proposed.clone())
        .await
        .expect("Failed to receive CAT status proposal");
    println!("[TEST]   CAT status proposal received successfully");

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
    
    let mut hs_node = create_hs_node();
    println!("[TEST]   HyperSchedulerNode created");

    // Test proposal behavior
    let cat_id = CATId("test-cat".to_string());
    let status = StatusLimited::Success;
    let constituent_chains = vec![ChainId("chain-1".to_string())];
    
    // First proposal should create a record
    hs_node.process_cat_status_proposal(cat_id.clone(), ChainId("chain-1".to_string()), constituent_chains.clone(), status.clone())
        .await
        .expect("Failed to receive first CAT status proposal");
    println!("[TEST]   First proposal received successfully");

    // Second proposal should be rejected
    let result = hs_node.process_cat_status_proposal(cat_id.clone(), ChainId("chain-1".to_string()), constituent_chains.clone(), status.clone())
        .await;
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

    let mut hs_node = create_hs_node();
    println!("[TEST]   HyperSchedulerNode created");

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = StatusLimited::Success;
    let constituent_chains = vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())];
    println!("[TEST]   Created CAT ID: {} with status: {:?}", cat_id.0, status);

    // Process the status proposal from first chain
    println!("[TEST]   Processing CAT status proposal from first chain...");
    hs_node.process_cat_status_proposal(cat_id.clone(), ChainId("chain-1".to_string()), constituent_chains.clone(), status.clone())
        .await
        .expect("Failed to process first CAT status proposal");
    println!("[TEST]   First proposal processed successfully");

    // Process the status proposal from second chain
    println!("[TEST]   Processing CAT status proposal from second chain...");
    hs_node.process_cat_status_proposal(cat_id.clone(), ChainId("chain-2".to_string()), constituent_chains.clone(), status.clone())
        .await
        .expect("Failed to process second CAT status proposal");
    println!("[TEST]   Second proposal processed successfully");

    // Verify the CAT status is updated to success
    let stored_status = hs_node.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    println!("[TEST]   Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, CATStatus::Success);
    println!("[TEST]   Status verification successful");

    println!("=== Test completed successfully ===\n");
}