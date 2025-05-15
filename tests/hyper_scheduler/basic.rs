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


/// Test receiving a success proposal for a CAT
/// - Verify CAT is stored with success status
#[tokio::test]
async fn test_receive_success_proposal_first_message() {
    println!("\n=== Starting test_receive_success_proposal_first_message ===");
    
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
    
    println!("=== Test completed successfully ===\n");
}

/// Test receiving a failure proposal for a two-chain CAT
/// - Verify CAT is stored with failure status
#[tokio::test]
async fn test_receive_failure_proposal_first_message() {
    println!("\n=== Starting test_receive_failure_proposal_first_message ===");
    
    let mut hs_node = create_hs_node();
    println!("[Test] HyperSchedulerNode created");

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Failure;
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

/// Test rejecting duplicate proposals
/// - Verify that proposals create records
/// - Verify that duplicate proposals are rejected
#[tokio::test]
async fn test_duplicate_rejection() {
    println!("\n=== Starting test_duplicate_rejection ===");
    
    let mut hs_node = create_hs_node();
    println!("[Test] HyperSchedulerNode created");

    // Test proposal behavior
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Success;
    
    // First proposal should create a record
    hs_node.receive_cat_status_proposal(cat_id.clone(), status.clone())
        .await
        .expect("Failed to receive first CAT status proposal");
    println!("[Test] First proposal received successfully");

    // Second proposal should be rejected
    let result = hs_node.receive_cat_status_proposal(cat_id.clone(), status.clone())
        .await;
    assert!(result.is_err(), "Expected duplicate proposal to be rejected");
    println!("[Test] Verified duplicate proposal was rejected");
    
    println!("=== Test completed successfully ===\n");
}
