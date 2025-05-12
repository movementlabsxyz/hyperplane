use hyperplane::{
    types::{CATId, CATStatusUpdate, TransactionId, Transaction},
    hyper_scheduler::HyperSchedulerNode,
    hyper_ig::{HyperIG, HyperIGNode},
};

/// Tests the storage of a single CAT status update in HS:
/// 1. HIG proposes a Success status for a CAT
/// 2. HS receives and stores the status
/// 3. HS can retrieve the stored Success status
#[tokio::test]
async fn test_single_cat_status_storage() {
    // Create a Hyper IG node and Hyper Scheduler node
    let mut hig_node = HyperIGNode::new();
    let hs_node = HyperSchedulerNode::new();

    // Set the hyper scheduler in the Hyper IG node
    hig_node.set_hyper_scheduler(Box::new(hs_node));

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusUpdate::Success;

    // Propose the status update
    hig_node.send_cat_status_proposal(cat_id.clone(), status.clone())
        .await
        .expect("Failed to propose CAT status update");

    // Get the proposed status from the Hyper IG node
    let proposed_status = hig_node.get_proposed_status(TransactionId(cat_id.0.clone()))
        .await
        .expect("Failed to get proposed status");

    // Verify the proposed status matches
    assert_eq!(proposed_status, status);
}

/// Tests the storage of multiple CAT status updates in HS:
/// 1. HIG proposes Success and Failure statuses for different CATs
/// 2. HS stores each status update
/// 3. HS can retrieve all stored statuses in the correct order
#[tokio::test]
async fn test_multiple_cat_status_storage() {
    // Create a Hyper IG node and Hyper Scheduler node
    let mut hig_node = HyperIGNode::new();
    let hs_node = HyperSchedulerNode::new();

    // Set the hyper scheduler in the Hyper IG node
    hig_node.set_hyper_scheduler(Box::new(hs_node));

    // Create multiple CAT IDs and status updates
    let updates = vec![
        (CATId("cat1".to_string()), CATStatusUpdate::Success),
        (CATId("cat2".to_string()), CATStatusUpdate::Failure),
        (CATId("cat3".to_string()), CATStatusUpdate::Success),
    ];

    // Propose each status update
    for (cat_id, status) in updates.clone() {
        hig_node.send_cat_status_proposal(cat_id.clone(), status.clone())
            .await
            .expect("Failed to propose CAT status update");

        // Get the proposed status from the Hyper IG node
        let proposed_status = hig_node.get_proposed_status(TransactionId(cat_id.0.clone()))
            .await
            .expect("Failed to get proposed status");

        // Verify the proposed status matches
        assert_eq!(proposed_status, status);
    }
}

/// Integration: HIG proposes a Success status update, HS stores it
#[tokio::test]
async fn test_status_update_success() {
    let mut hig = HyperIGNode::new();
    let hs = HyperSchedulerNode::new();
    hig.set_hyper_scheduler(Box::new(hs));

    // Submit a CAT transaction (simulate)
    let cat_id = CATId("cat-tx".to_string());
    let tx = Transaction {
        id: TransactionId(cat_id.0.clone()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    hig.execute_transaction(tx).await.expect("Failed to execute CAT transaction");

    // Propose a success status update
    hig.send_cat_status_proposal(cat_id.clone(), CATStatusUpdate::Success)
        .await
        .expect("Failed to propose status update");

    // Check that HS has the correct status
    let hs_ref = hig.hyper_scheduler().unwrap();
    let status = hs_ref.get_cat_status(cat_id.clone()).await.unwrap();
    assert_eq!(status, CATStatusUpdate::Success);
}

/// Integration: HIG proposes a Failure status update, HS stores it
#[tokio::test]
async fn test_status_update_failure() {
    let mut hig = HyperIGNode::new();
    let hs = HyperSchedulerNode::new();
    hig.set_hyper_scheduler(Box::new(hs));

    // Submit a CAT transaction (simulate)
    let cat_id = CATId("cat-tx".to_string());
    let tx = Transaction {
        id: TransactionId(cat_id.0.clone()),
        data: "CAT.SIMULATION.FAILURE".to_string(),
    };
    hig.execute_transaction(tx).await.expect("Failed to execute CAT transaction");

    // Propose a failure status update
    hig.send_cat_status_proposal(cat_id.clone(), CATStatusUpdate::Failure)
        .await
        .expect("Failed to propose status update");

    // Check that HS has the correct status
    let hs_ref = hig.hyper_scheduler().unwrap();
    let status = hs_ref.get_cat_status(cat_id.clone()).await.unwrap();
    assert_eq!(status, CATStatusUpdate::Failure);
} 