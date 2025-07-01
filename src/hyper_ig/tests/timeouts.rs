use crate::types::{Transaction, TransactionId, CATId, SubBlock, TransactionStatus, CLTransactionId, constants};
use crate::utils::logging;
use crate::hyper_ig::tests::basic::setup_test_hig_node;
use crate::hyper_ig::HyperIG;
use crate::hyper_ig::node::HyperIGNode;
use crate::types::CATStatusUpdate;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// Helper function to run a CAT timeout test with specific parameters
async fn run_cat_timeout_test(second_block_height: u64, expected_status: TransactionStatus) -> (Arc<Mutex<HyperIGNode>>, mpsc::Receiver<CATStatusUpdate>) {
    logging::init_logging();
    logging::log("TEST", &format!("\n=== Starting CAT timeout test with block height {} and expected status {:?} ===", 
        second_block_height, expected_status));
    
    // Create node
    let (mut hig_node, receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a CAT transaction
    let cl_id = CLTransactionId("cl-tx".to_string());
    logging::log("TEST", &format!("Creating cl-id='{}'", cl_id.0));
    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
    logging::log("TEST", &format!("Created tx-id='{}'", tx_id.0));
    let cat_tx = Transaction::new(
        tx_id,
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    // Process the CAT in block 1
    let subblock = SubBlock {
        block_height: 1,
        chain_id: constants::chain_1(),
        transactions: vec![cat_tx.clone()],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed block height=1"));
    
    // Verify CAT is pending
    let status = hig_node.get_transaction_status(cat_tx.id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Get the max lifetime
    let cat_id = CATId(cl_id.clone());
    let max_lifetime = hig_node.get_cat_max_lifetime(cat_id).await.unwrap();
    logging::log("TEST", &format!("Max lifetime='{}'", max_lifetime));
    
    // Process the second block at the specified height
    let subblock = SubBlock {
        block_height: second_block_height,
        chain_id: constants::chain_1(),
        transactions: vec![],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed block height={}", second_block_height));

    // Get current block height
    let block_height = hig_node.get_current_block_height().await.unwrap();
    logging::log("TEST", &format!("Current block height='{}'", block_height));
    
    // Verify CAT has the expected status
    let status = hig_node.get_transaction_status(cat_tx.id).await.unwrap();
    assert_eq!(status, expected_status);
    
    logging::log("TEST", "=== Test completed successfully ===\n");
    
    (hig_node, receiver_hig_to_hs)
}

/// Tests that a CAT transaction expires correctly when its lifetime is exceeded.
/// 
/// This test verifies that a CAT transaction is properly marked as failed when
/// the current block height exceeds its maximum lifetime. This is a critical
/// safety mechanism to ensure CATs don't remain pending indefinitely.
/// 
/// Test flow:
/// 1. Creates a CAT transaction in block 1
/// 2. Processes block 6 (which is after max lifetime of 5)
/// 3. Verifies the CAT is marked as failed
#[tokio::test]
async fn test_cat_timeout() {
    // Create a CAT in block 1, then process block 6 (which is after max lifetime)
    run_cat_timeout_test(6, TransactionStatus::Failure).await;
}

/// Tests that a CAT transaction remains pending for a block height less than its expiration.
/// 
/// This test verifies that a CAT transaction stays in pending state as long as
/// the current block height is less than its maximum lifetime. This ensures
/// that valid CATs have enough time to complete their execution.
/// 
/// Test flow:
/// 1. Creates a CAT transaction in block 1
/// 2. Processes block 5 (which is before max lifetime)
/// 3. Verifies the CAT remains in pending state
#[tokio::test]
async fn test_cat_timeout_not() {
    // Create a CAT in block 1, then process block 5 (which is before max lifetime)
    run_cat_timeout_test(5, TransactionStatus::Pending).await;
}

/// Tests that a timed-out CAT cannot be updated to success.
/// 
/// This test verifies that once a CAT transaction is marked as failed due to timeout,
/// its status becomes irreversible - it cannot be updated to success even if a status
/// update arrives later. This is important for maintaining consistency in the system
/// and preventing race conditions where a late status update could override a timeout.
/// 
/// Test flow:
/// 1. Creates a CAT transaction in block 1
/// 2. Processes block 6 to trigger timeout (max lifetime is 5)
/// 3. Verifies the CAT is marked as failed
/// 4. Attempts to update the CAT to success via a status update
/// 5. Verifies the CAT remains failed despite the status update
#[tokio::test]
async fn test_cat_timeout_irreversible() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_timeout_irreversible ===");
    
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
    
    // Use run_cat_timeout_test to set up and trigger timeout
    let (mut hig_node, _receiver_hig_to_hs) = run_cat_timeout_test(6, TransactionStatus::Failure).await;
    
    // ensure the cat is marked as failed
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Failure, "CAT should be marked as failed");
    
    // Try to update the CAT to success
    let cl_id_2 = CLTransactionId("cl-tx.UPDATE".to_string());
    let status_update = Transaction::new(
        TransactionId(format!("{}:tx", cl_id_2.0)),
        constants::chain_1(),
        vec![constants::chain_1()],
        format!("STATUS_UPDATE:Success.CAT_ID:{}", cl_id.0),
        cl_id.clone(),
    ).expect("Failed to create status update");
    
    // Process the status update
    let _status = hig_node.process_transaction(status_update).await.unwrap();
    
    // Verify CAT is still failed
    let status = hig_node.get_transaction_status(tx_id).await.unwrap();
    assert_eq!(status, TransactionStatus::Failure, "CAT should remain failed even after status update");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a CAT transaction that succeeds should not be timed out.
/// 
/// This test verifies that if a CAT transaction receives a success status update,
/// it should not be marked as failed due to timeout, even if the timeout period
/// has passed. This is important for ensuring that successful CATs are not
/// incorrectly marked as failed.
/// 
/// Test flow:
/// 1. Creates a CAT transaction in block 1
/// 2. Processes block 6 (which is after max lifetime of 5)
/// 3. Sends a success status update
/// 4. Verifies the CAT remains successful and is not timed out
#[tokio::test]
async fn test_cat_success_should_not_timeout() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_success_should_not_timeout ===");
    
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
    
    // Create node
    let (mut hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a CAT transaction
    let cat_tx = Transaction::new(
        tx_id.clone(),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    // Process the CAT in block 1
    let subblock = SubBlock {
        block_height: 1,
        chain_id: constants::chain_1(),
        transactions: vec![cat_tx.clone()],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", "Processed block height=1");
    
    // Verify CAT is pending
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);

    // Create a subblock that provides a status update with success
    let status_update = Transaction::new(
        TransactionId(format!("{}:status_update", cl_id.0)),
        constants::chain_1(),
        vec![constants::chain_1()],
        format!("STATUS_UPDATE:Success.CAT_ID:{}", cl_id.0),
        cl_id.clone(),
    ).expect("Failed to create status update");
    // Process the status update in block 2
    let subblock = SubBlock {
        block_height: 2,
        chain_id: constants::chain_1(),
        transactions: vec![status_update],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", "Processed block height=2");

    // Verify the CAT is successful
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Success, "CAT should be successful after status update");
    
    // get the max lifetime
    let max_lifetime = hig_node.get_cat_lifetime().await.unwrap();
    logging::log("TEST", &format!("Max lifetime='{}'", max_lifetime));

    // Process block after max lifetime
    let subblock = SubBlock {
        block_height: max_lifetime + 2,
        chain_id: constants::chain_1(),
        transactions: vec![],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed block height={}", max_lifetime + 2));
    
    // Verify CAT is still successful
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Success, "CAT should still be successful after timeout check");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a status update arriving before timeout should be processed, not ignored.
/// 
/// This test verifies that if a status update arrives for a CAT that has NOT timed out yet,
/// the status update should be processed normally, not ignored due to incorrect timeout logic.
/// This is critical for ensuring that valid status updates are not lost due to timing issues.
/// 
/// Test flow:
/// 1. Creates a CAT transaction in block 1
/// 2. Processes block 2 (which is before max lifetime)
/// 3. Sends a status update in block 2
/// 4. Verifies the CAT status is updated correctly
/// 5. Processes block 3 to trigger timeout check
/// 6. Verifies the CAT remains in the correct status (not timed out)
#[tokio::test]
async fn test_status_update_before_timeout_should_process() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_status_update_before_timeout_should_process ===");
    
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
    
    // Create node
    let (mut hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a CAT transaction
    let cat_tx = Transaction::new(
        tx_id.clone(),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    // Process the CAT in block 1
    let subblock = SubBlock {
        block_height: 1,
        chain_id: constants::chain_1(),
        transactions: vec![cat_tx.clone()],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", "Processed block height=1");
    
    // Verify CAT is pending
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending, "CAT should be pending after creation");
    
    // Get the max lifetime to verify timing
    let cat_id = CATId(cl_id.clone());
    let max_lifetime = hig_node.get_cat_max_lifetime(cat_id).await.unwrap();
    let cat_lifetime = hig_node.get_cat_lifetime().await.unwrap();
    logging::log("TEST", &format!("CAT max_lifetime: {}, cat_lifetime: {}", max_lifetime, cat_lifetime));
    
    // Create a status update transaction
    let status_update = Transaction::new(
        TransactionId(format!("{}:status_update", cl_id.0)),
        constants::chain_1(),
        vec![constants::chain_1()],
        format!("STATUS_UPDATE:Success.CAT_ID:{}", cl_id.0),
        cl_id.clone(),
    ).expect("Failed to create status update");
    
    // Process the status update in block 2 (before timeout)
    let subblock = SubBlock {
        block_height: 2,
        chain_id: constants::chain_1(),
        transactions: vec![status_update],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", "Processed block height=2 with status update");
    
    // Verify the CAT is now successful (status update should be processed)
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Success, "CAT should be successful after status update");
    
    // Process block 3 to trigger timeout check
    let subblock = SubBlock {
        block_height: 3,
        chain_id: constants::chain_1(),
        transactions: vec![],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", "Processed block height=3");
    
    // Verify CAT is still successful (should not be timed out)
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Success, "CAT should still be successful after timeout check");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a status update arriving exactly at the timeout boundary should be processed.
/// 
/// This test verifies edge case behavior when a status update arrives at the exact
/// block height where the CAT would timeout. The status update should be processed
/// before the timeout check occurs.
/// 
/// Test flow:
/// 1. Creates a CAT transaction in block 1
/// 2. Sends a status update in the exact block where timeout would occur
/// 3. Verifies the CAT status is updated correctly
/// 4. Verifies the CAT is not timed out
#[tokio::test]
async fn test_status_update_at_timeout_boundary_should_process() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_status_update_at_timeout_boundary_should_process ===");
    
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
    
    // Create node
    let (mut hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a CAT transaction
    let cat_tx = Transaction::new(
        tx_id.clone(),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    // Process the CAT in block 1
    let subblock = SubBlock {
        block_height: 1,
        chain_id: constants::chain_1(),
        transactions: vec![cat_tx.clone()],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", "Processed block height=1");
    
    // Get the max lifetime to determine timeout boundary
    let cat_id = CATId(cl_id.clone());
    let max_lifetime = hig_node.get_cat_max_lifetime(cat_id).await.unwrap();
    let cat_lifetime = hig_node.get_cat_lifetime().await.unwrap();
    logging::log("TEST", &format!("CAT max_lifetime: {}, cat_lifetime: {}", max_lifetime, cat_lifetime));
    
    // Create a status update transaction
    let status_update = Transaction::new(
        TransactionId(format!("{}:status_update", cl_id.0)),
        constants::chain_1(),
        vec![constants::chain_1()],
        format!("STATUS_UPDATE:Success.CAT_ID:{}", cl_id.0),
        cl_id.clone(),
    ).expect("Failed to create status update");
    
    // Process the status update in the exact block where timeout would occur
    let subblock = SubBlock {
        block_height: max_lifetime,
        chain_id: constants::chain_1(),
        transactions: vec![status_update],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed block height={} with status update", max_lifetime));
    
    // Verify the CAT is now successful (status update should be processed)
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Success, "CAT should be successful after status update");
    
    // Process the next block to trigger timeout check
    let subblock = SubBlock {
        block_height: max_lifetime + 1,
        chain_id: constants::chain_1(),
        transactions: vec![],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed block height={}", max_lifetime + 1));
    
    // Verify CAT is still successful (should not be timed out)
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Success, "CAT should still be successful after timeout check");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a Success status update for a timed-out CAT triggers a panic.
/// 
/// This test verifies that if a CAT transaction times out (is marked as Failed due to timeout)
/// and then receives a Success status update, the system correctly panics with a
/// critical error message. This is important for catching logical errors in the system.
/// 
/// Test flow:
/// 1. Creates a CAT transaction in block 1
/// 2. Processes a subblock with height > CAT lifetime to trigger timeout
/// 3. Verifies the CAT is marked as Failed due to timeout
/// 4. Sends a Success status update
/// 5. Verifies that the system panics with the correct error message
#[tokio::test]
#[should_panic(expected = "🚨 CRITICAL ERROR: Received Success status update for CAT tx-id=")]
async fn test_success_status_update_for_failed_cat_should_panic() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_success_status_update_for_failed_cat_should_panic ===");
    
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
    
    // Create node
    let (mut hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a CAT transaction
    let cat_tx = Transaction::new(
        tx_id.clone(),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    // Process the CAT in block 1
    let subblock = SubBlock {
        block_height: 1,
        chain_id: constants::chain_1(),
        transactions: vec![cat_tx.clone()],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", "Processed block height=1");
    
    // Verify CAT is pending
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending, "CAT should be pending after creation");

    // Process a subblock with height 6 to trigger timeout (CAT lifetime is 4, so max_lifetime = 5)
    let timeout_subblock = SubBlock {
        block_height: 6,
        chain_id: constants::chain_1(),
        transactions: vec![],
    };
    hig_node.process_subblock(timeout_subblock).await.unwrap();
    logging::log("TEST", "Processed block height=6 to trigger timeout");
    
    // Verify CAT is now failed due to timeout
    let status = hig_node.get_transaction_status(tx_id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Failure, "CAT should be failed due to timeout");

    // Create a Success status update (this should trigger the panic)
    let success_update = Transaction::new(
        TransactionId(format!("{}:success_update", cl_id.0)),
        constants::chain_1(),
        vec![constants::chain_1()],
        format!("STATUS_UPDATE:Success.CAT_ID:{}", cl_id.0),
        cl_id.clone(),
    ).expect("Failed to create success update");

    // This should panic with the critical error message
    hig_node.process_transaction(success_update).await.expect("Should have panicked");
}
