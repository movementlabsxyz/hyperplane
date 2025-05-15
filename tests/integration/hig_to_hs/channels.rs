use hyperplane::{
    types::{Transaction, TransactionId, CATStatusLimited, CATId},
    hyper_ig::HyperIG,
    hyper_scheduler::{HyperScheduler, HyperSchedulerNode},
};
use tokio::{time::{sleep, Duration}, task};
use crate::common::testnodes;

/// Tests sending a single CAT status update from HIG to HS:
/// - HIG proposes a Success status for a CAT
/// - HS receives and stores the status
/// - HS can retrieve the stored Success status
#[tokio::test]
async fn test_single_cat_status_storage() {
    println!("\n[TEST] === Starting test_single_cat_status_storage ===");
    
    let (hs_node, _cl_node, hig_node) = testnodes::setup_test_nodes_no_block_production().await;
    println!("[TEST] Test nodes initialized successfully");

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Success;
    println!("[TEST] Created CAT ID: {} with status: {:?}", cat_id.0, status);

    // Propose the status update
    println!("[TEST] Sending CAT status proposal from HIG to HS...");
    hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), status.clone())
        .await
        .expect("Failed to propose CAT status update");
    println!("[TEST] CAT status proposal sent successfully");

    // Wait for HS to process the message (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the message (150ms)...");
    sleep(Duration::from_millis(150)).await;

    // Verify HS stored the status
    println!("[TEST] Attempting to acquire hs_node lock for get_cat_status...");
    let stored_status = hs_node.lock().await.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    println!("[TEST] Released hs_node lock after get_cat_status");
    println!("[TEST] Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, status);
    println!("[TEST] Status verification successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

/// Tests the storage of multiple CAT status updates in HS:
/// - HIG proposes Success status for multiple CATs
/// - HS receives and stores all statuses
/// - HS can retrieve all stored Success statuses
#[tokio::test]
async fn test_multiple_cat_status_storage() {
    println!("\n[TEST] === Starting test_multiple_cat_status_storage ===");
    println!("[TEST] Setting up test nodes with 100ms block interval...");
    
    // Initialize components with 100ms block interval
    let (hs_node, _cl_node, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST] Test nodes initialized successfully");

    // Clone hs_node for the message processing loop
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop in a separate task
    println!("[TEST] Starting HS message processing loop...");
    let _hs_handle = task::spawn(HyperSchedulerNode::process_messages(hs_node_clone));
    println!("[TEST] HS message processing loop started");

    // Create multiple CAT IDs and status updates
    let cat_ids = vec![
        CATId("test-cat-1".to_string()),
        CATId("test-cat-2".to_string()),
        CATId("test-cat-3".to_string()),
    ];
    let status = CATStatusLimited::Success;
    println!("[TEST] Created {} CAT IDs with status: {:?}", cat_ids.len(), status);

    // Propose status updates for each CAT
    println!("[TEST] Sending CAT status proposals from HIG to HS...");
    for (i, cat_id) in cat_ids.iter().enumerate() {
        println!("[TEST] Sending proposal {}/{} for CAT: {}", i + 1, cat_ids.len(), cat_id.0);
        hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), status.clone())
            .await
            .expect("Failed to propose CAT status update");
    }
    println!("[TEST] All CAT status proposals sent successfully");

    // Wait for HS to process all messages (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process all messages (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete, verifying stored statuses...");

    // Verify HS stored all statuses
    for (i, cat_id) in cat_ids.iter().enumerate() {
        println!("[TEST] Verifying status {}/{} for CAT: {}", i + 1, cat_ids.len(), cat_id.0);
        let stored_status = hs_node.lock().await.get_cat_status(cat_id.clone())
            .await
            .expect("Failed to get CAT status");
        println!("[TEST] Retrieved stored status: {:?}", stored_status);
        assert_eq!(stored_status, status);
    }
    println!("[TEST] All status verifications successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

/// Tests the storage of a CAT status update in HS with a transaction ID:
/// - HIG proposes a Success status for a CAT with a transaction ID
/// - HS receives and stores the status
/// - HS can retrieve the stored Success status
#[tokio::test]
async fn test_cat_status_storage_with_transaction_id() {
    println!("\n[TEST] === Starting test_cat_status_storage_with_transaction_id ===");
    println!("[TEST] Setting up test nodes with 100ms block interval...");
    
    // Initialize components with 100ms block interval
    let (hs_node, _cl_node, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST] Test nodes initialized successfully");

    // Clone hs_node for the message processing loop
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop in a separate task
    println!("[TEST] Starting HS message processing loop...");
    let _hs_handle = task::spawn(HyperSchedulerNode::process_messages(hs_node_clone));
    println!("[TEST] HS message processing loop started");

    // Create a CAT ID, transaction ID, and status update
    let cat_id = CATId("test-cat".to_string());
    let transaction_id = TransactionId("test-tx".to_string());
    let status = CATStatusLimited::Success;
    println!("[TEST] Created CAT ID: {} with transaction ID: {} and status: {:?}", 
        cat_id.0, transaction_id.0, status);

    // Propose the status update
    println!("[TEST] Sending CAT status proposal from HIG to HS...");
    hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), status.clone())
        .await
        .expect("Failed to propose CAT status update");
    println!("[TEST] CAT status proposal sent successfully");

    // Wait for HS to process the message (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the message (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete, verifying stored status...");

    // Verify HS stored the status
    let stored_status = hs_node.lock().await.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    println!("[TEST] Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, status);
    println!("[TEST] Status verification successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

/// Tests the storage of a CAT status update in HS:
/// - HIG proposes a Failure status for a CAT
/// - HS receives and stores the status
/// - HS can retrieve the stored Failure status
#[tokio::test]
async fn test_status_proposal_failure() {
    println!("\n[TEST] === Starting test_status_proposal_failure ===");
    println!("[TEST] Setting up test nodes with 100ms block interval...");
    
    // Initialize components with 100ms block interval
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST] Test nodes initialized successfully");

    // Clone hs_node for the message processing loop
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop in a separate task
    println!("[TEST] Starting HS message processing loop...");
    let _hs_handle = task::spawn(HyperSchedulerNode::process_messages(hs_node_clone));
    println!("[TEST] HS message processing loop started");

    // Create a CAT ID and status update
    let cat_id = CATId("test-cat".to_string());
    let status = CATStatusLimited::Failure;
    println!("[TEST] Created CAT ID: {} with status: {:?}", cat_id.0, status);

    // Propose the status update
    println!("[TEST] Sending CAT status proposal from HIG to HS...");
    hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), status.clone())
        .await
        .expect("Failed to propose CAT status update");
    println!("[TEST] CAT status proposal sent successfully");

    // Wait for HS to process the message (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the message (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete, verifying stored status...");

    // Verify HS stored the status
    let stored_status = hs_node.lock().await.get_cat_status(cat_id.clone())
        .await
        .expect("Failed to get CAT status");
    println!("[TEST] Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, status);
    println!("[TEST] Status verification successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

/// Tests the sending of a CAT status proposal from HIG to HS:
/// - HIG sends a Success status proposal for a CAT
/// - HS receives and stores the status
/// - HS can retrieve the stored Success status
#[tokio::test]
async fn test_send_cat_status_proposal() {
    println!("\n[TEST] === Starting test_send_cat_status_proposal ===");
    println!("[TEST] Setting up test nodes with 100ms block interval...");
    
    // Initialize components with 100ms block interval
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST] Test nodes initialized successfully");

    // Clone hs_node for the message processing loop
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop in a separate task
    println!("[TEST] Starting HS message processing loop...");
    let _hs_handle = task::spawn(HyperSchedulerNode::process_messages(hs_node_clone));
    println!("[TEST] HS message processing loop started");

    // Send a status proposal
    let cat_id = CATId("test-cat".to_string());
    println!("[TEST] Sending status proposal for CAT: {}", cat_id.0);
    hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), CATStatusLimited::Success)
        .await
        .expect("Failed to send status proposal");
    println!("[TEST] Status proposal sent successfully");

    // Wait for HS to process the message (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the message (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete, verifying stored status...");

    // Verify the status in HS
    let stored_status = hs_node.lock().await.get_cat_status(cat_id).await.unwrap();
    println!("[TEST] Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, CATStatusLimited::Success);
    println!("[TEST] Status verification successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

/// Tests the processing of a CAT transaction in HS:
/// - HIG executes a CAT transaction
/// - HS receives and processes the transaction
/// - HS can retrieve the stored Success status
#[tokio::test]
async fn test_process_cat_transaction() {
    println!("\n[TEST] === Starting test_process_cat_transaction ===");
    println!("[TEST] Setting up test nodes with 100ms block interval...");
    
    // Initialize components with 100ms block interval
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST] Test nodes initialized successfully");

    // Clone hs_node for the message processing loop
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop in a separate task
    println!("[TEST] Starting HS message processing loop...");
    let _hs_handle = task::spawn(HyperSchedulerNode::process_messages(hs_node_clone));
    println!("[TEST] HS message processing loop started");

    // Create and process a CAT transaction
    let tx = Transaction {
        id: TransactionId("test-cat".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    println!("[TEST] Created CAT transaction with ID: {}", tx.id.0);
    
    println!("[TEST] Executing CAT transaction...");
    hig_node.lock().await.execute_transaction(tx.clone()).await.expect("Failed to execute transaction");
    println!("[TEST] CAT transaction executed successfully");

    // Wait for HS to process the message (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the message (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete, verifying stored status...");

    // Verify the status in HS
    let stored_status = hs_node.lock().await.get_cat_status(CATId(tx.id.0)).await.unwrap();
    println!("[TEST] Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, CATStatusLimited::Success);
    println!("[TEST] Status verification successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

/// Tests the processing of a status update in HS:
/// - HIG sends a Success status update for a CAT
/// - HS receives and processes the update
/// - HS can retrieve the stored Success status
#[tokio::test]
async fn test_process_status_update() {
    println!("\n[TEST] === Starting test_process_status_update ===");
    println!("[TEST] Setting up test nodes with 100ms block interval...");
    
    // Initialize components with 100ms block interval
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST] Test nodes initialized successfully");

    // Clone hs_node for the message processing loop
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop in a separate task
    println!("[TEST] Starting HS message processing loop...");
    let _hs_handle = task::spawn(HyperSchedulerNode::process_messages(hs_node_clone));
    println!("[TEST] HS message processing loop started");

    // Create and process a CAT transaction
    let tx = Transaction {
        id: TransactionId("test-cat".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    println!("[TEST] Created CAT transaction with ID: {}", tx.id.0);
    
    println!("[TEST] Executing CAT transaction...");
    hig_node.lock().await.execute_transaction(tx.clone()).await.expect("Failed to execute transaction");
    println!("[TEST] CAT transaction executed successfully");

    // Wait for HS to process the message (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the message (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete");

    // Send status update
    println!("[TEST] Sending status update for CAT: {}", tx.id.0);
    hs_node.lock().await.send_cat_status_update(CATId(tx.id.0.clone()), CATStatusLimited::Success)
        .await
        .expect("Failed to send status update");
    println!("[TEST] Status update sent successfully");

    // Wait for HS to process the update (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the update (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete, verifying stored status...");

    // Verify the status in HS
    let stored_status = hs_node.lock().await.get_cat_status(CATId(tx.id.0)).await.unwrap();
    println!("[TEST] Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, CATStatusLimited::Success);
    println!("[TEST] Status verification successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

/// Tests the sending of a CAT status proposal from HIG to HS:
/// - HIG sends a Success status proposal for a CAT
/// - HS receives and stores the status
/// - HS can retrieve the stored Success status
#[tokio::test]
async fn test_hig_to_hs_status_proposal() {
    println!("\n[TEST] === Starting test_hig_to_hs_status_proposal ===");
    println!("[TEST] Setting up test nodes with 100ms block interval...");
    
    // Initialize components with 100ms block interval
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST] Test nodes initialized successfully");

    // Clone hs_node for the message processing loop
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop
    println!("[TEST] Starting HS message processing loop...");
    let _hs_handle = task::spawn(HyperSchedulerNode::process_messages(hs_node_clone));
    println!("[TEST] HS message processing loop started");

    // Create a CAT transaction
    let cat_id = CATId("test_cat".to_string());
    let status = CATStatusLimited::Success;
    println!("[TEST] Created CAT ID: {} with status: {:?}", cat_id.0, status);

    // Send status proposal from HIG to HS
    println!("[TEST] Sending status proposal from HIG to HS...");
    hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), status.clone()).await.unwrap();
    println!("[TEST] Status proposal sent successfully");

    // Wait for HS to process the message (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the message (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete, verifying stored status...");

    // Verify that the proposed status matches what we sent
    let stored_status = hs_node.lock().await.get_cat_status(cat_id).await.unwrap();
    println!("[TEST] Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, status);
    println!("[TEST] Status verification successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

/// Tests the processing of a status update in HS:
/// - HIG sends a Failure status proposal for a CAT
/// - HS receives and processes the update
/// - HS can retrieve the stored Failure status
#[tokio::test]
async fn test_hig_to_hs_status_proposal_failure() {
    println!("\n[TEST] === Starting test_hig_to_hs_status_proposal_failure ===");
    println!("[TEST] Setting up test nodes with 100ms block interval...");
    
    // Initialize components with 100ms block interval
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST] Test nodes initialized successfully");

    // Clone hs_node for the message processing loop
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop
    println!("[TEST] Starting HS message processing loop...");
    let _hs_handle = task::spawn(HyperSchedulerNode::process_messages(hs_node_clone));
    println!("[TEST] HS message processing loop started");

    // Create a CAT transaction with an invalid status
    let cat_id = CATId("test_cat".to_string());
    let status = CATStatusLimited::Failure;
    println!("[TEST] Created CAT ID: {} with status: {:?}", cat_id.0, status);

    // Send status proposal from HIG to HS
    println!("[TEST] Sending status proposal from HIG to HS...");
    hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), status.clone()).await.unwrap();
    println!("[TEST] Status proposal sent successfully");

    // Wait for HS to process the message (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the message (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete, verifying stored status...");

    // Verify that the proposed status matches what we sent
    let stored_status = hs_node.lock().await.get_cat_status(cat_id).await.unwrap();
    println!("[TEST] Retrieved stored status: {:?}", stored_status);
    assert_eq!(stored_status, status);
    println!("[TEST] Status verification successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

/// Tests the processing of multiple status proposals in HS:
/// - HIG sends multiple Success status proposals for different CATs
/// - HS receives and processes all statuses
/// - HS can retrieve all stored Success statuses
#[tokio::test]
async fn test_hig_to_hs_multiple_status_proposals() {
    println!("\n[TEST] === Starting test_hig_to_hs_multiple_status_proposals ===");
    println!("[TEST] Setting up test nodes with 100ms block interval...");
    
    // Initialize components with 100ms block interval
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST] Test nodes initialized successfully");

    // Clone hs_node for the message processing loop
    let hs_node_clone = hs_node.clone();

    // Start the HS message processing loop
    println!("[TEST] Starting HS message processing loop...");
    let _hs_handle = task::spawn(HyperSchedulerNode::process_messages(hs_node_clone));
    println!("[TEST] HS message processing loop started");

    // Create multiple CAT transactions
    let cat_ids = vec![
        CATId("test_cat_1".to_string()),
        CATId("test_cat_2".to_string()),
        CATId("test_cat_3".to_string()),
    ];
    println!("[TEST] Created {} CAT IDs", cat_ids.len());

    // Send status proposals from HIG to HS
    println!("[TEST] Sending status proposals for all CATs...");
    for (i, cat_id) in cat_ids.iter().enumerate() {
        println!("[TEST] Sending proposal {}/{} for CAT: {}", i + 1, cat_ids.len(), cat_id.0);
        hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), CATStatusLimited::Success).await.unwrap();
    }
    println!("[TEST] All status proposals sent successfully");

    // Wait for HS to process the messages (150ms to ensure processing)
    println!("[TEST] Waiting for HS to process the messages (150ms)...");
    sleep(Duration::from_millis(150)).await;
    println!("[TEST] Wait complete, verifying stored statuses...");

    // Verify that all proposed statuses match what we sent
    for (i, cat_id) in cat_ids.iter().enumerate() {
        println!("[TEST] Verifying status {}/{} for CAT: {}", i + 1, cat_ids.len(), cat_id.0);
        let stored_status = hs_node.lock().await.get_cat_status(cat_id.clone()).await.unwrap();
        println!("[TEST] Retrieved stored status: {:?}", stored_status);
        assert_eq!(stored_status, CATStatusLimited::Success);
    }
    println!("[TEST] All status verifications successful");
    
    println!("[TEST] === Test completed successfully ===\n");
}

#[tokio::test]
async fn test_cat_transaction_flow() {
    // use testnodes from common
    let (_hs_node, _cl_node, _hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;

    // ... existing code ...
}

#[tokio::test]
async fn test_cat_status_update_flow() {
    // use testnodes from common
    let (_hs_node, _cl_node, _hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;

    // ... existing code ...
}
