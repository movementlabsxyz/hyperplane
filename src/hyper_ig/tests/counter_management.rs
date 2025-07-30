use crate::hyper_ig::{HyperIGNode, HyperIG};
use crate::types::{Transaction, TransactionId, CLTransactionId, TransactionStatus, SubBlock};
use crate::types::constants;
use crate::utils::logging;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

/// Helper function to test CAT status updates with different credit amounts and expected regular transaction outcomes
/// 
/// # Arguments
/// * `cat_credit_amount` - The credit amount for the CAT transaction (e.g., 100 or 1000)
/// * `expected_regular_status` - The expected final status for the regular transaction (Success or Failure)
/// * `test_name` - A descriptive name for the test case
async fn test_cat_status_update_with_credit_amount(
    cat_credit_amount: u32,
    expected_regular_status: TransactionStatus,
    test_name: &str,
) {
    logging::init_logging();
    logging::log("TEST", &format!("=== Starting {} ===", test_name));
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a CAT transaction with the specified credit amount
    let cl_id = CLTransactionId(format!("cl-tx_cat_{}", cat_credit_amount));
    let cat_tx = Transaction::new(
        TransactionId(format!("{:?}:cat_{}", cl_id, cat_credit_amount)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        format!("CAT.credit 1 {}", cat_credit_amount),
        cl_id.clone(),
    ).expect("Failed to create CAT transaction");
    
    // Create a regular transaction that will succeed or fail based on the CAT's credit amount
    let regular_tx = Transaction::new(
        TransactionId(format!("regular_tx_{}", cat_credit_amount)),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.send 1 2 1000".to_string(), // This will succeed if CAT credits 1000, fail if CAT credits 100
        CLTransactionId(format!("cl-tx_regular_{}", cat_credit_amount)),
    ).expect("Failed to create regular transaction");
    
    // Process CAT transaction (should stay pending)
    logging::log("TEST", "Processing CAT transaction...");
    let cat_status = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
    logging::log("TEST", &format!("CAT transaction status: {:?}", cat_status));
    
    // Process regular transaction (should be blocked initially)
    logging::log("TEST", "Processing regular transaction...");
    let regular_status = hig_node.lock().await.process_transaction(regular_tx.clone()).await.unwrap();
    logging::log("TEST", &format!("Regular transaction status: {:?}", regular_status));
    
    // Check that both transactions are in pending set
    let pending_after_both = hig_node.lock().await.get_pending_transactions().await.unwrap();
    let cat_in_pending = pending_after_both.contains(&cat_tx.id);
    let regular_in_pending = pending_after_both.contains(&regular_tx.id);
    logging::log("TEST", &format!("CAT in pending after both processed: {}", cat_in_pending));
    logging::log("TEST", &format!("Regular in pending after both processed: {}", regular_in_pending));
    assert!(cat_in_pending, "CAT transaction should be in pending set");
    assert!(regular_in_pending, "Regular transaction should be in pending set");
    
    // Get counts after both transactions
    let cat_counts = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let regular_counts = hig_node.lock().await.get_transaction_status_counts_regular().await.unwrap();
    logging::log("TEST", &format!("After both - CAT counts - Pending: {}, Success: {}, Failure: {}", cat_counts.0, cat_counts.1, cat_counts.2));
    logging::log("TEST", &format!("After both - Regular counts - Pending: {}, Success: {}, Failure: {}", regular_counts.0, regular_counts.1, regular_counts.2));
    
    // Update CAT status to success via status update transaction
    logging::log("TEST", "Updating CAT status to success...");
    let status_update = Transaction::new(
        TransactionId(format!("{}:status_update", cl_id.0)),
        constants::chain_1(),
        vec![constants::chain_1()],
        format!("STATUS_UPDATE:Success.CAT_ID:{}", cl_id.0),
        cl_id.clone(),
    ).expect("Failed to create status update");
    
    // Process the status update in a subblock
    let subblock = SubBlock {
        block_height: 2,
        chain_id: constants::chain_1(),
        transactions: vec![status_update],
    };
    hig_node.lock().await.process_subblock(subblock).await.unwrap();
    
    // Get final counts after CAT status update
    let final_cat_counts = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let final_regular_counts = hig_node.lock().await.get_transaction_status_counts_regular().await.unwrap();
    let final_pending_txs = hig_node.lock().await.get_pending_transactions().await.unwrap();
    
    logging::log("TEST", &format!("Final CAT counts - Pending: {}, Success: {}, Failure: {}", final_cat_counts.0, final_cat_counts.1, final_cat_counts.2));
    logging::log("TEST", &format!("Final Regular counts - Pending: {}, Success: {}, Failure: {}", final_regular_counts.0, final_regular_counts.1, final_regular_counts.2));
    logging::log("TEST", &format!("Final pending transactions: {:?}", final_pending_txs));
    
    // Verify final state
    assert_eq!(final_cat_counts.0, 0, "CAT pending count should be 0 after status update");
    assert_eq!(final_cat_counts.1, 1, "CAT success count should be 1 after status update");
    assert_eq!(final_regular_counts.0, 0, "Regular pending count should be 0 (reached final status)");
    
    // Verify the expected regular transaction status
    match expected_regular_status {
        TransactionStatus::Success => {
            assert_eq!(final_regular_counts.1, 1, "Regular success count should be 1");
            assert_eq!(final_regular_counts.2, 0, "Regular failure count should be 0");
        }
        TransactionStatus::Failure => {
            assert_eq!(final_regular_counts.1, 0, "Regular success count should be 0");
            assert_eq!(final_regular_counts.2, 1, "Regular failure count should be 1");
        }
        _ => panic!("Expected status should be Success or Failure"),
    }
    
    // Verify that neither transaction is in pending set (both reached final status)
    assert!(!final_pending_txs.contains(&cat_tx.id), "CAT transaction should not be in pending set after success");
    assert!(!final_pending_txs.contains(&regular_tx.id), "Regular transaction should not be in pending set after final status");
    
    logging::log("TEST", &format!("=== {} completed successfully ===\n", test_name));
}

/// Helper function to set up a test HIG node
async fn setup_test_hig_node(allow_cat_pending_dependencies: bool) -> (Arc<Mutex<HyperIGNode>>, mpsc::Receiver<crate::types::CATStatusUpdate>) {
    let (sender_hig_to_hs, receiver_hig_to_hs) = mpsc::channel(100);
    let (_sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel(100);
    
    let hig_node = HyperIGNode::new(
        receiver_cl_to_hig,
        sender_hig_to_hs,
        constants::chain_1(),
        10, // cat_lifetime
        allow_cat_pending_dependencies,
    );
    
    (Arc::new(Mutex::new(hig_node)), receiver_hig_to_hs)
}

/// Tests the add_to_pending_and_increment_counter function:
/// - Verifies that pending counters are properly incremented when transactions are added to pending set
/// - Ensures CAT and regular transaction counters are updated correctly
/// - Validates that transactions are properly stored in the pending set
#[tokio::test]
async fn test_add_to_pending_and_increment_counter() {
    logging::init_logging();
    logging::log("TEST", "=== Starting test_add_to_pending_and_increment_counter ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a CAT transaction
    let cl_id = CLTransactionId("cl-tx_cat_1".to_string());
    let cat_tx = Transaction::new(
        TransactionId(format!("{:?}:cat_1", cl_id)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create CAT transaction");
    
    // Get initial counts
    let initial_counts = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let initial_pending = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("Initial counts - Pending: {}, Success: {}, Failure: {}", initial_counts.0, initial_counts.1, initial_counts.2));
    logging::log("TEST", &format!("Initial detailed counts - Resolving: {}, Postponed: {}", initial_pending.0, initial_pending.1));
    
    // Process the transaction
    let status = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
    logging::log("TEST", &format!("CAT transaction status: {:?}", status));
    
    // Get counts after processing
    let final_counts = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let final_pending = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("Final counts - Pending: {}, Success: {}, Failure: {}", final_counts.0, final_counts.1, final_counts.2));
    logging::log("TEST", &format!("Final detailed counts - Resolving: {}, Postponed: {}", final_pending.0, final_pending.1));
    
    // Verify that pending count increased by 1
    assert_eq!(final_counts.0, initial_counts.0 + 1, "CAT pending count should increase by 1");
    
    // Verify that the transaction is in the pending set
    let pending_txs = hig_node.lock().await.get_pending_transactions().await.unwrap();
    assert!(pending_txs.contains(&cat_tx.id), "Transaction should be in pending set");
    
    logging::log("TEST", "=== test_add_to_pending_and_increment_counter completed successfully ===\n");
}



/// Tests the update_to_final_status_and_update_counter function:
/// - Verifies that final status counters (Success/Failure) are properly incremented
/// - Ensures transaction status is correctly updated to final state
/// - Validates that transactions are removed from pending set when reaching final status
#[tokio::test]
async fn test_update_to_final_status_and_update_counter() {
    logging::init_logging();
    logging::log("TEST", "=== Starting test_update_to_final_status_and_update_counter ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a regular transaction that will fail
    let regular_tx = Transaction::new(
        TransactionId("regular_tx_2".to_string()),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.send 1 2 1000".to_string(), // This will fail due to insufficient funds
        CLTransactionId("cl-tx_regular_2".to_string()),
    ).expect("Failed to create regular transaction");
    
    // Process the transaction (it should fail)
    let status = hig_node.lock().await.process_transaction(regular_tx.clone()).await.unwrap();
    logging::log("TEST", &format!("Regular transaction status: {:?}", status));
    assert_eq!(status, TransactionStatus::Failure, "Transaction should fail");
    
    // Get counts after processing
    let final_counts = hig_node.lock().await.get_transaction_status_counts_regular().await.unwrap();
    logging::log("TEST", &format!("Final regular counts - Pending: {}, Success: {}, Failure: {}", final_counts.0, final_counts.1, final_counts.2));
    
    // Verify that counts are correct
    assert_eq!(final_counts.0, 0, "Regular pending count should be 0");
    assert_eq!(final_counts.1, 0, "Regular success count should be 0");
    assert_eq!(final_counts.2, 1, "Regular failure count should be 1");
    
    // Verify that the transaction is NOT in the pending set (it was removed)
    let pending_txs = hig_node.lock().await.get_pending_transactions().await.unwrap();
    assert!(!pending_txs.contains(&regular_tx.id), "Transaction should NOT be in pending set after failure");

    logging::log("TEST", "=== test_update_to_final_status_and_update_counter completed successfully ===\n");
}

/// Tests counter consistency when transactions are reprocessed:
/// - Verifies that counters are not double-incremented when the same transaction is processed multiple times
/// - Ensures that pending set and counter state remain consistent across reprocessing
/// - Validates that detailed CAT counters (resolving/postponed) maintain consistency
#[tokio::test]
async fn test_counter_consistency_on_reprocessing() {
    logging::init_logging();
    logging::log("TEST", "=== Starting test_counter_consistency_on_reprocessing ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a CAT transaction
    let cl_id = CLTransactionId("cl-tx_cat_2".to_string());
    let cat_tx = Transaction::new(
        TransactionId(format!("{:?}:cat_2", cl_id)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create CAT transaction");
    
    // Process the transaction first time
    let status1 = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
    logging::log("TEST", &format!("First processing status: {:?}", status1));
    
    // Get counts after first processing
    let counts1 = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let pending1 = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("After first processing - Pending: {}, Success: {}, Failure: {}", counts1.0, counts1.1, counts1.2));
    logging::log("TEST", &format!("After first processing - Resolving: {}, Postponed: {}", pending1.0, pending1.1));
    
    // Process the same transaction again (should not change counters)
    let status2 = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
    logging::log("TEST", &format!("Second processing status: {:?}", status2));
    
    // Get counts after second processing
    let counts2 = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let pending2 = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("After second processing - Pending: {}, Success: {}, Failure: {}", counts2.0, counts2.1, counts2.2));
    logging::log("TEST", &format!("After second processing - Resolving: {}, Postponed: {}", pending2.0, pending2.1));
    
    // Verify that counters didn't change (no double counting)
    assert_eq!(counts2.0, counts1.0, "CAT pending count should not change on reprocessing");
    assert_eq!(counts2.1, counts1.1, "CAT success count should not change on reprocessing");
    assert_eq!(counts2.2, counts1.2, "CAT failure count should not change on reprocessing");
    assert_eq!(pending2.0, pending1.0, "CAT resolving count should not change on reprocessing");
    assert_eq!(pending2.1, pending1.1, "CAT postponed count should not change on reprocessing");
    
    logging::log("TEST", "=== test_counter_consistency_on_reprocessing completed successfully ===\n");
}

/// Tests CAT status updates with CAT credit 100 - regular transaction should fail due to insufficient funds
/// - CAT credits 100, regular transaction tries to send 1000 (insufficient funds)
/// - Regular transaction should reach Failure status after CAT resolution
#[tokio::test]
async fn test_cat_status_update_with_credit_100_failure() {
    test_cat_status_update_with_credit_amount(100, TransactionStatus::Failure, "test_cat_status_update_with_credit_100_failure").await;
}

/// Tests CAT status updates with CAT credit 1000 - regular transaction should succeed due to sufficient funds
/// - CAT credits 1000, regular transaction tries to send 1000 (sufficient funds)
/// - Regular transaction should reach Success status after CAT resolution
#[tokio::test]
async fn test_cat_status_update_with_credit_1000_success() {
    test_cat_status_update_with_credit_amount(1000, TransactionStatus::Success, "test_cat_status_update_with_credit_1000_success").await;
} 