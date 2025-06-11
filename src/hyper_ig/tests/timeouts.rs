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
    let (mut hig_node, receiver_hig_to_hs) = setup_test_hig_node().await;
    
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
