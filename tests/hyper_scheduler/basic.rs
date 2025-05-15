use hyperplane::{
    types::{CATId, CATStatusLimited},
    hyper_scheduler::node::HyperSchedulerNode,
    HyperScheduler,
};
use tokio::sync::mpsc;

// create a HyperSchedulerNode with empty channels
fn create_hs_node() -> HyperSchedulerNode {
    let (_, receiver_from_hig) = mpsc::channel(1);
    let (sender_to_cl, _) = mpsc::channel(1);
    HyperSchedulerNode::new(receiver_from_hig, sender_to_cl)
}

/// Test receiving a success proposal for a single-chain CAT
/// NOTE: in principle single chain CATs dont make sense, but we can use them for better testing
/// - Verify proposal is stored
/// - Verify CAT is added to pending list
/// - Verify status is stored correctly
#[tokio::test]
async fn test_receive_success_proposal() {
    println!("\n=== Starting test_receive_success_proposal ===");
    
    let mut hs_node = create_hs_node();
    println!("[Test] HyperSchedulerNode created");

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Success;
    println!("[Test] Created CAT ID: {} with status: {:?}", cat_id.0, status);

    // Receive the status proposal directly
    println!("[Test] Receiving CAT status proposal...");
    hs_node.receive_cat_status_proposal(cat_id.clone(), status.clone())
        .await
        .expect("Failed to receive CAT status proposal");
    println!("[Test] CAT status proposal received successfully");

    // Verify HS stored the status
    let stored_status = hs_node.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    println!("[Test] Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, status);
    println!("[Test] Status verification successful");

    // Verify CAT is in pending list
    let pending_cats = hs_node.get_pending_cats()
        .await
        .expect("Failed to get pending CATs");
    println!("[Test] Retrieved pending CATs: {:?}", pending_cats);
    assert!(pending_cats.contains(&cat_id));
    println!("[Test] Pending CAT verification successful");
    
    println!("=== Test completed successfully ===\n");
}

/// TODO: Test receiving a failure proposal for a single-chain CAT
/// NOTE: in principle single chain CATs dont make sense, but we can use them for better testing
/// - Verify proposal is stored
/// - Verify CAT is added to pending list
/// - Verify status is stored correctly
#[tokio::test]
async fn test_receive_failure_proposal() {
    // TODO: Implement test
}

/// TODO: Test error cases for receiving proposals
/// - Receive proposal for non-existent CAT
/// - Receive duplicate proposal for same CAT
#[tokio::test]
async fn test_receive_proposal_errors() {
    // TODO: Implement test
}

/// TODO: Test sending success update for single-chain CAT
/// - Verify update is sent to CL
/// - Verify correct transaction format
/// - Verify correct chain ID
#[tokio::test]
async fn test_send_success_update() {
    // TODO: Implement test
}

/// TODO: Test sending failure update for single-chain CAT
/// - Verify update is sent to CL
/// - Verify correct transaction format
/// - Verify correct chain ID
#[tokio::test]
async fn test_send_failure_update() {
    // TODO: Implement test
}

/// TODO: Test error cases for sending updates
/// - Send update for non-existent CAT
/// - Send update when CL is not set
#[tokio::test]
async fn test_send_update_errors() {
    // TODO: Implement test
}

/// TODO: Test processing single-chain CAT (simplified case)
/// - Receive status proposal from HIG
/// - Verify status is stored
/// - Verify update is sent to CL immediately
#[tokio::test]
async fn test_process_single_chain_cat() {
    // TODO: Implement test
}

/// TODO: Test processing two-chain CAT (real case)
/// - Receive status proposal from HIG
/// - Verify status is stored
/// - Verify no update is sent to CL yet
/// - Receive status from first chain
/// - Verify no update is sent to CL yet
/// - Receive status from second chain
/// - Verify update is sent to CL
/// - Verify correct final status
#[tokio::test]
async fn test_process_two_chain_cat() {
    // TODO: Implement test
    // we cannot test this because we dont handle the case of multiple chains yet
}

/// TODO: Test processing two-chain CAT with conflicting statuses
/// - Receive success proposal from HIG
/// - Receive success from first chain
/// - Receive failure from second chain
/// - Verify failure update is sent to CL (failure takes precedence)
#[tokio::test]
async fn test_process_conflicting_statuses() {
    // TODO: Implement test
}

/// TODO: Test processing two-chain CAT with timeout
/// - Receive status proposal from HIG
/// - Receive status from first chain
/// - Wait for timeout
/// - Verify failure update is sent to CL
#[tokio::test]
async fn test_process_cat_timeout() {
    // TODO: Implement test
}
