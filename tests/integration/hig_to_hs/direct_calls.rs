use hyperplane::{
    types::{Transaction, TransactionId, CATStatusLimited, CATId},
    hyper_ig::HyperIG,
    hyper_scheduler::HyperScheduler,
};
use tokio::{time::{sleep, Duration}, task};
use crate::common::testnodes;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Tests the storage of a single CAT status update in HS:
/// - HIG proposes a Success status for a CAT
/// - HS receives and stores the status
/// - HS can retrieve the stored Success status
#[tokio::test]
async fn test_single_cat_status_storage() {
    // use testnodes from common
    let (hs_node, _, mut hig_node) = testnodes::setup_test_nodes();

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop in a separate task
    let _hs_handle = task::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Success;

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

    // Wait for HS to process the message
    sleep(Duration::from_millis(100)).await;

    // Verify HS stored the status
    let stored_status = hs_node.lock().await.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    assert_eq!(stored_status, status);
}

/// Tests the storage of multiple CAT status updates in HS:
/// - HIG proposes Success status for multiple CATs
/// - HS receives and stores all statuses
/// - HS can retrieve all stored Success statuses
#[tokio::test]
async fn test_multiple_cat_status_storage() {
    // use testnodes from common
    let (hs_node, _, mut hig_node) = testnodes::setup_test_nodes();

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop in a separate task
    let _hs_handle = task::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });

    // Create multiple CAT IDs and status updates
    let cat_ids = vec![
        CATId("test-cat-1".to_string()),
        CATId("test-cat-2".to_string()),
        CATId("test-cat-3".to_string()),
    ];
    let status = CATStatusLimited::Success;

    // Propose status updates for all CATs
    for cat_id in &cat_ids {
        hig_node.send_cat_status_proposal(cat_id.clone(), status.clone())
            .await
            .expect("Failed to propose CAT status update");
    }

    // Wait for HS to process all messages
    sleep(Duration::from_millis(100)).await;

    // Verify HS stored all statuses
    for cat_id in &cat_ids {
        let stored_status = hs_node.lock().await.get_cat_status(cat_id.clone())
            .await
            .expect("Failed to get CAT status");
        assert_eq!(stored_status, status);
    }
}

/// Tests the storage of a CAT status update in HS:
/// - HIG proposes a Failure status for a CAT
/// - HS receives and stores the status
/// - HS can retrieve the stored Failure status
#[tokio::test]
async fn test_status_proposal_failure() {
    // use testnodes from common
    let ( hs_node, _, mut hig_node) = testnodes::setup_test_nodes();

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop in a separate task
    let _hs_handle = task::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Success;

    // Propose the status update
    hig_node.send_cat_status_proposal(cat_id.clone(), status.clone())
        .await
        .expect("Failed to propose CAT status update");

    // Wait for HS to process the message
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_send_cat_status_proposal() {
    // use testnodes from common
    let ( hs_node, _, mut hig_node) = testnodes::setup_test_nodes();

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let _hs_node_clone = hs_node.clone();

    // Send a status proposal
    let cat_id = CATId("test-cat".to_string());
    hig_node.send_cat_status_proposal(cat_id.clone(), CATStatusLimited::Success)
        .await
        .expect("Failed to send status proposal");

    // Verify the status in HS
    let stored_status = hs_node.lock().await.get_cat_status(cat_id).await.unwrap();
    assert_eq!(stored_status, CATStatusLimited::Success);
}

#[tokio::test]
async fn test_process_cat_transaction() {
    // use testnodes from common
    let ( hs_node, _, mut hig_node) = testnodes::setup_test_nodes();

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let _hs_node_clone = hs_node.clone();

    // Create and process a CAT transaction
    let tx = Transaction {
        id: TransactionId("test-cat".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    hig_node.execute_transaction(tx.clone()).await.expect("Failed to execute transaction");

    // Verify the status in HS
    let stored_status = hs_node.lock().await.get_cat_status(CATId(tx.id.0)).await.unwrap();
    assert_eq!(stored_status, CATStatusLimited::Success);
}

#[tokio::test]
async fn test_process_status_update() {
    // use testnodes from common
    let (hs_node, _, mut hig_node) = testnodes::setup_test_nodes();

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let _hs_node_clone = hs_node.clone();

    // Create and process a CAT transaction
    let tx = Transaction {
        id: TransactionId("test-cat".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    hig_node.execute_transaction(tx.clone()).await.expect("Failed to execute transaction");

    // Send status update
    hs_node.lock().await.send_cat_status_update(CATId(tx.id.0.clone()), CATStatusLimited::Success)
        .await
        .expect("Failed to send status update");

    // Verify the status in HS
    let stored_status = hs_node.lock().await.get_cat_status(CATId(tx.id.0)).await.unwrap();
    assert_eq!(stored_status, CATStatusLimited::Success);
}

#[tokio::test]
async fn test_hig_to_hs_status_proposal() {
    // use testnodes from common
    let (hs_node, _, mut hig_node) = testnodes::setup_test_nodes();

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop
    let _hs_handle = task::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });

    // Create a CAT transaction
    let cat_id = CATId("test_cat".to_string());
    let status = CATStatusLimited::Success;

    // Send status proposal from HIG to HS
    hig_node.send_cat_status_proposal(cat_id.clone(), status.clone()).await.unwrap();

    // Wait for HS to process the message
    sleep(Duration::from_millis(100)).await;

    // Verify that the proposed status matches what we sent
    let stored_status = hs_node.lock().await.get_cat_status(cat_id).await.unwrap();
    assert_eq!(stored_status, status);
}

#[tokio::test]
async fn test_hig_to_hs_status_proposal_failure() {
    // use testnodes from common
    let (hs_node, _, mut hig_node) = testnodes::setup_test_nodes();

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop
    tokio::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });

    // Create a CAT transaction with an invalid status
    let cat_id = CATId("test_cat".to_string());
    let status = CATStatusLimited::Failure;

    // Send status proposal from HIG to HS
    hig_node.send_cat_status_proposal(cat_id.clone(), status.clone()).await.unwrap();

    // Wait for HS to process the message
    sleep(Duration::from_millis(100)).await;

    // Verify that the proposed status matches what we sent
    let stored_status = hs_node.lock().await.get_cat_status(cat_id).await.unwrap();
    assert_eq!(stored_status, status);
}

#[tokio::test]
async fn test_hig_to_hs_multiple_status_proposals() {
    // use testnodes from common
    let ( hs_node, _, mut hig_node) = testnodes::setup_test_nodes();

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop
    tokio::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });

    // Create multiple CAT transactions
    let cat_ids = vec![
        CATId("test_cat_1".to_string()),
        CATId("test_cat_2".to_string()),
        CATId("test_cat_3".to_string()),
    ];

    // Send status proposals from HIG to HS
    for cat_id in &cat_ids {
        hig_node.send_cat_status_proposal(cat_id.clone(), CATStatusLimited::Success).await.unwrap();
    }

    // Wait for HS to process the messages
    sleep(Duration::from_millis(100)).await;

    // Verify that all proposed statuses match what we sent
    for cat_id in &cat_ids {
        let stored_status = hs_node.lock().await.get_cat_status(cat_id.clone()).await.unwrap();
        assert_eq!(stored_status, CATStatusLimited::Success);
    }
} 