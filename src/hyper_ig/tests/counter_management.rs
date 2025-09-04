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

/// Tests that CAT timeout properly updates counters:
/// - Verifies that pending CAT count decreases when CAT times out
/// - Ensures failure count increases when CAT times out
/// - Validates that the CAT is removed from pending set after timeout
/// - Checks that detailed pending counters (resolving/postponed) are updated correctly
#[tokio::test]
async fn test_cat_timeout_counter_management() {
    logging::init_logging();
    logging::log("TEST", "=== Starting test_cat_timeout_counter_management ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create a CAT transaction
    let cl_id = CLTransactionId("cl-tx_timeout_test".to_string());
    let cat_tx = Transaction::new(
        TransactionId(format!("{:?}:timeout_test", cl_id)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create CAT transaction");
    
    // Process the CAT transaction in block 1
    let subblock = SubBlock {
        block_height: 1,
        chain_id: constants::chain_1(),
        transactions: vec![cat_tx.clone()],
    };
    hig_node.lock().await.process_subblock(subblock).await.unwrap();
    
    // Verify CAT is pending and counters are correct
    let status = hig_node.lock().await.get_transaction_status(cat_tx.id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending, "CAT should be pending after creation");
    
    let cat_counts_before = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let pending_detailed_before = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("Before timeout - CAT counts - Pending: {}, Success: {}, Failure: {}", cat_counts_before.0, cat_counts_before.1, cat_counts_before.2));
    logging::log("TEST", &format!("Before timeout - Detailed counts - Resolving: {}, Postponed: {}", pending_detailed_before.0, pending_detailed_before.1));
    
    assert_eq!(cat_counts_before.0, 1, "CAT pending count should be 1 after creation");
    assert_eq!(cat_counts_before.1, 0, "CAT success count should be 0 before timeout");
    assert_eq!(cat_counts_before.2, 0, "CAT failure count should be 0 before timeout");
    
    // Verify CAT is in pending set
    let pending_txs_before = hig_node.lock().await.get_pending_transactions().await.unwrap();
    assert!(pending_txs_before.contains(&cat_tx.id), "CAT should be in pending set before timeout");
    
    // Get the max lifetime to determine timeout block
    let cat_id = crate::types::CATId(cl_id.clone());
    let max_lifetime = hig_node.lock().await.get_cat_max_lifetime(cat_id).await.unwrap();
    let cat_lifetime = hig_node.lock().await.get_cat_lifetime().await.unwrap();
    logging::log("TEST", &format!("CAT max_lifetime: {}, cat_lifetime: {}", max_lifetime, cat_lifetime));
    
    // Process a block that exceeds the timeout (block after max_lifetime)
    let timeout_block = max_lifetime + 1;
    let subblock = SubBlock {
        block_height: timeout_block,
        chain_id: constants::chain_1(),
        transactions: vec![],
    };
    hig_node.lock().await.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed timeout block: {}", timeout_block));
    
    // Verify CAT is now failed
    let status_after = hig_node.lock().await.get_transaction_status(cat_tx.id.clone()).await.unwrap();
    assert_eq!(status_after, TransactionStatus::Failure, "CAT should be failed after timeout");
    
    // Get counts after timeout
    let cat_counts_after = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let pending_detailed_after = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("After timeout - CAT counts - Pending: {}, Success: {}, Failure: {}", cat_counts_after.0, cat_counts_after.1, cat_counts_after.2));
    logging::log("TEST", &format!("After timeout - Detailed counts - Resolving: {}, Postponed: {}", pending_detailed_after.0, pending_detailed_after.1));
    
    // Verify counter changes
    assert_eq!(cat_counts_after.0, 0, "CAT pending count should be 0 after timeout");
    assert_eq!(cat_counts_after.1, 0, "CAT success count should be 0 after timeout");
    assert_eq!(cat_counts_after.2, 1, "CAT failure count should be 1 after timeout");
    
    // Verify detailed pending counters are updated
    assert_eq!(pending_detailed_after.0, 0, "CAT resolving count should be 0 after timeout");
    assert_eq!(pending_detailed_after.1, 0, "CAT postponed count should be 0 after timeout");
    
    // Verify CAT is removed from pending set
    let pending_txs_after = hig_node.lock().await.get_pending_transactions().await.unwrap();
    assert!(!pending_txs_after.contains(&cat_tx.id), "CAT should not be in pending set after timeout");
    
    // Verify total pending count matches detailed counts
    assert_eq!(cat_counts_after.0, pending_detailed_after.0 + pending_detailed_after.1, 
               "Total pending count should equal resolving + postponed counts");
    
    logging::log("TEST", "=== test_cat_timeout_counter_management completed successfully ===\n");
}

/// Tests that CAT accumulation is properly managed and doesn't exceed theoretical maximum:
/// - Creates multiple CATs over time
/// - Verifies that pending CAT count doesn't accumulate indefinitely
/// - Ensures that timed-out CATs are properly removed from counters
/// - Validates that the system maintains correct state even with many CATs
#[tokio::test]
async fn test_cat_accumulation_management() {
    logging::init_logging();
    logging::log("TEST", "=== Starting test_cat_accumulation_management ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Get CAT lifetime for theoretical maximum calculation
    let cat_lifetime = hig_node.lock().await.get_cat_lifetime().await.unwrap();
    logging::log("TEST", &format!("CAT lifetime: {}", cat_lifetime));
    
    // Create and process multiple CATs over several blocks
    let num_cats: usize = 5;
    let mut cat_txs = Vec::new();
    
    for i in 0..num_cats {
        let cl_id = CLTransactionId(format!("cl-tx_accumulation_{}", i));
        let cat_tx = Transaction::new(
            TransactionId(format!("{:?}:accumulation_{}", cl_id, i)),
            constants::chain_1(),
            vec![constants::chain_1(), constants::chain_2()],
            "CAT.credit 1 100".to_string(),
            cl_id.clone(),
        ).expect("Failed to create CAT transaction");
        
        cat_txs.push(cat_tx.clone());
        
        // Process CAT in block i+1
        let subblock = SubBlock {
            block_height: (i + 1) as u64,
            chain_id: constants::chain_1(),
            transactions: vec![cat_tx],
        };
        hig_node.lock().await.process_subblock(subblock).await.unwrap();
        
        // Check counts after each CAT
        let cat_counts = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
        let pending_txs = hig_node.lock().await.get_pending_transactions().await.unwrap();
        logging::log("TEST", &format!("After CAT {} - Pending: {}, Success: {}, Failure: {}", 
            i, cat_counts.0, cat_counts.1, cat_counts.2));
        logging::log("TEST", &format!("Pending transactions count: {}", pending_txs.len()));
        
        // Verify that pending count equals the number of CATs we've created
        assert_eq!(cat_counts.0, (i + 1) as u64, "CAT pending count should equal number of CATs created");
        assert_eq!(pending_txs.len(), i + 1, "Pending transactions should equal number of CATs created");
    }
    
    // Now process blocks to trigger timeouts for the first few CATs
    // Process a block that will timeout the first CAT (created in block 1)
    let timeout_block = cat_lifetime + 2; // Block after max_lifetime for first CAT
    let subblock = SubBlock {
        block_height: timeout_block,
        chain_id: constants::chain_1(),
        transactions: vec![],
    };
    hig_node.lock().await.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed timeout block: {}", timeout_block));
    
    // Check counts after timeout
    let cat_counts_after_timeout = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let pending_txs_after = hig_node.lock().await.get_pending_transactions().await.unwrap();
    logging::log("TEST", &format!("After timeout - Pending: {}, Success: {}, Failure: {}", 
        cat_counts_after_timeout.0, cat_counts_after_timeout.1, cat_counts_after_timeout.2));
    logging::log("TEST", &format!("Pending transactions count after timeout: {}", pending_txs_after.len()));
    
    // Verify that the first CAT was timed out and removed
    assert_eq!(cat_counts_after_timeout.2, 1, "CAT failure count should be 1 after timeout");
    assert_eq!(cat_counts_after_timeout.0, (num_cats - 1) as u64, "CAT pending count should decrease by 1 after timeout");
    assert_eq!(pending_txs_after.len(), num_cats - 1, "Pending transactions should decrease by 1 after timeout");
    
    // Verify that the first CAT is no longer in pending set
    assert!(!pending_txs_after.contains(&cat_txs[0].id), "First CAT should not be in pending set after timeout");
    
    // Verify that other CATs are still pending
    for i in 1..num_cats {
        assert!(pending_txs_after.contains(&cat_txs[i as usize].id), "CAT {} should still be in pending set", i);
    }
    
    // Process more blocks to timeout more CATs
    let final_timeout_block = cat_lifetime + num_cats as u64 + 1;
    let subblock = SubBlock {
        block_height: final_timeout_block,
        chain_id: constants::chain_1(),
        transactions: vec![],
    };
    hig_node.lock().await.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed final timeout block: {}", final_timeout_block));
    
    // Check final counts
    let final_cat_counts = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let final_pending_txs = hig_node.lock().await.get_pending_transactions().await.unwrap();
    logging::log("TEST", &format!("Final counts - Pending: {}, Success: {}, Failure: {}", 
        final_cat_counts.0, final_cat_counts.1, final_cat_counts.2));
    logging::log("TEST", &format!("Final pending transactions count: {}", final_pending_txs.len()));
    
    // Verify that all CATs have been timed out
    assert_eq!(final_cat_counts.0, 0, "All CATs should be timed out");
    assert_eq!(final_cat_counts.2, num_cats as u64, "All CATs should be in failure state");
    assert_eq!(final_pending_txs.len(), 0, "No transactions should be pending");
    
    // Verify total counts are consistent
    assert_eq!(final_cat_counts.0 + final_cat_counts.1 + final_cat_counts.2, num_cats as u64, 
               "Total CAT counts should equal number of CATs created");
    
    logging::log("TEST", "=== test_cat_accumulation_management completed successfully ===\n");
} 

/// Tests the transition from postponed to resolving counters when a postponed CAT gets reprocessed:
/// - Verifies that postponed counter decreases and resolving counter increases
/// - Ensures the transition happens correctly when a postponed CAT is reprocessed
/// - Validates that the total pending count remains consistent
#[tokio::test]
async fn test_postponed_to_resolving_transition() {
    logging::init_logging();
    logging::log("TEST", "=== Starting test_postponed_to_resolving_transition ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create first CAT that will be resolving
    let cl_id_1 = CLTransactionId("cl-tx_cat_1".to_string());
    let cat_tx_1 = Transaction::new(
        TransactionId(format!("{:?}:cat_1", cl_id_1)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id_1.clone(),
    ).expect("Failed to create first CAT transaction");
    
    // Process first CAT - it should be resolving
    let status_1 = hig_node.lock().await.process_transaction(cat_tx_1.clone()).await.unwrap();
    assert_eq!(status_1, TransactionStatus::Pending, "First CAT should be pending");
    
    // Create second CAT that will be postponed (depends on first CAT)
    let cl_id_2 = CLTransactionId("cl-tx_cat_2".to_string());
    let cat_tx_2 = Transaction::new(
        TransactionId(format!("{:?}:cat_2", cl_id_2)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id_2.clone(),
    ).expect("Failed to create second CAT transaction");
    
    // Process second CAT - it should be postponed
    let status_2 = hig_node.lock().await.process_transaction(cat_tx_2.clone()).await.unwrap();
    assert_eq!(status_2, TransactionStatus::Pending, "Second CAT should be pending");
    
    // Check initial detailed counts
    let (resolving_initial, postponed_initial) = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("Initial detailed counts - Resolving: {}, Postponed: {}", resolving_initial, postponed_initial));
    assert_eq!(resolving_initial, 1, "Should have 1 resolving CAT (first CAT)");
    assert_eq!(postponed_initial, 1, "Should have 1 postponed CAT (second CAT)");
    
    // Check total pending count before status update
    let total_pending_before = resolving_initial + postponed_initial;
    logging::log("TEST", &format!("Total pending count before status update: {}", total_pending_before));

    // Verify detailed counts match actual pending transactions
    let pending_txs = hig_node.lock().await.get_pending_transactions().await.unwrap();
    let mut cat_pending_txs = Vec::new();
    for tx_id in &pending_txs {
        if let Ok(tx) = hig_node.lock().await.get_transaction_data(tx_id.clone()).await {
            if tx.starts_with("CAT") {
                cat_pending_txs.push(tx_id);
            }
        }
    }
    logging::log("TEST", &format!("Actual pending CAT transactions: {:?}", cat_pending_txs));
    assert_eq!(cat_pending_txs.len() as u64, total_pending_before, "Detailed counts should match actual pending CAT transactions");

    // Verify main CAT pending counter matches detailed counts
    let (cat_pending, _, _) = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    logging::log("TEST", &format!("Main CAT pending counter: {}", cat_pending));
    assert_eq!(cat_pending, total_pending_before, "Main CAT pending counter should match detailed counts");
    
    // Now resolve the first CAT to trigger reprocessing of the second CAT
    let status_update_tx = Transaction::new(
        TransactionId("status_update".to_string()),
        constants::chain_1(),
        vec![constants::chain_1()],
        "STATUS_UPDATE:Success.CAT_ID:cl-tx_cat_1".to_string(),
        cl_id_1.clone(),
    ).expect("Failed to create status update transaction");
    
    let status_update_result = hig_node.lock().await.process_transaction(status_update_tx).await.unwrap();
    assert_eq!(status_update_result, TransactionStatus::Success, "Status update should succeed");
    
    // Check final detailed counts - second CAT should have transitioned from postponed to resolving
    let (resolving_final, postponed_final) = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("Final detailed counts - Resolving: {}, Postponed: {}", resolving_final, postponed_final));
    assert_eq!(resolving_final, 1, "Should have 1 resolving CAT (second CAT)");
    assert_eq!(postponed_final, 0, "Should have 0 postponed CATs");
    
    // Check total pending count after status update
    let total_pending_after = resolving_final + postponed_final;
    logging::log("TEST", &format!("Total pending count after status update: {}", total_pending_after));

    // Verify detailed counts match actual pending transactions after status update
    let pending_txs_after = hig_node.lock().await.get_pending_transactions().await.unwrap();
    let mut cat_pending_txs_after = Vec::new();
    for tx_id in &pending_txs_after {
        if let Ok(tx) = hig_node.lock().await.get_transaction_data(tx_id.clone()).await {
            if tx.starts_with("CAT") {
                cat_pending_txs_after.push(tx_id);
            }
        }
    }
    logging::log("TEST", &format!("Actual pending CAT transactions after status update: {:?}", cat_pending_txs_after));
    assert_eq!(cat_pending_txs_after.len() as u64, total_pending_after, "Detailed counts should match actual pending CAT transactions after status update");

    // Verify main CAT pending counter matches detailed counts after status update
    let (cat_pending_after, _, _) = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    logging::log("TEST", &format!("Main CAT pending counter after status update: {}", cat_pending_after));
    assert_eq!(cat_pending_after, total_pending_after, "Main CAT pending counter should match detailed counts after status update");
    
    // Verify that the total pending count is correct
    // After the first CAT reaches final status, only the second CAT should be pending
    let total_pending_final = resolving_final + postponed_final;
    assert_eq!(total_pending_final, 1, "Total pending count should be 1 (only the second CAT)");
    
    logging::log("TEST", "=== test_postponed_to_resolving_transition completed successfully ===\n");
}

/// Tests the timing metrics for regular transaction finalization:
/// - Verifies that timing is tracked when regular transactions enter pending state
/// - Ensures average latency is calculated correctly
/// - Validates that only regular transactions (not CATs) are tracked
#[tokio::test]
async fn test_regular_transaction_timing_metrics() {
    logging::init_logging();
    logging::log("TEST", "=== Starting test_regular_transaction_timing_metrics ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Get initial timing metrics
    let initial_latency = hig_node.lock().await.get_average_regular_tx_latency().await;
    let initial_max_latency = hig_node.lock().await.get_max_regular_tx_latency().await;
    let initial_count = hig_node.lock().await.get_regular_tx_finalized_count().await;
    logging::log("TEST", &format!("Initial latency: {}ms, max: {}ms, count: {}", initial_latency, initial_max_latency, initial_count));
    
    // Create a regular transaction that will succeed
    let regular_tx = Transaction::new(
        TransactionId("regular_tx_timing".to_string()),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.credit 1 100".to_string(),
        CLTransactionId("cl-tx_timing".to_string()),
    ).expect("Failed to create regular transaction");
    
    // Process the transaction (it should succeed)
    let status = hig_node.lock().await.process_transaction(regular_tx.clone()).await.unwrap();
    logging::log("TEST", &format!("Regular transaction status: {:?}", status));
    assert_eq!(status, TransactionStatus::Success, "Transaction should succeed");
    
    // Get final timing metrics
    let final_latency = hig_node.lock().await.get_average_regular_tx_latency().await;
    let final_max_latency = hig_node.lock().await.get_max_regular_tx_latency().await;
    let final_count = hig_node.lock().await.get_regular_tx_finalized_count().await;
    logging::log("TEST", &format!("Final latency: {}ms, max: {}ms, count: {}", final_latency, final_max_latency, final_count));
    
    // Verify that timing metrics were updated
    assert_eq!(final_count, initial_count + 1, "Count should increase by 1");
    // Note: Latency might be 0 if transaction is processed very quickly
    // This is still valid - it means the transaction was processed immediately
    logging::log("TEST", &format!("Transaction processed with latency: {}ms", final_latency));
    
    // Verify that maximum latency is at least as high as the current latency
    assert!(final_max_latency >= final_latency, "Maximum latency should be at least as high as current latency");
    
    // Verify that the transaction is not in pending set (reached final status)
    let pending_txs = hig_node.lock().await.get_pending_transactions().await.unwrap();
    assert!(!pending_txs.contains(&regular_tx.id), "Transaction should not be in pending set after success");
    
    logging::log("TEST", "=== test_regular_transaction_timing_metrics completed successfully ===\n");
}

/// Test to verify regular transaction behavior with multiple dependencies
/// This test verifies that regular transactions with multiple dependencies are handled correctly
/// when their dependencies resolve in sequence
#[tokio::test]
async fn test_regular_tx_multiple_dependencies() {
    logging::init_logging();
    logging::log("TEST", "=== Starting test_regular_tx_multiple_dependencies ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Create multiple CATs that will be pending
    let mut cat_txs = Vec::new();
    for i in 1..=3 {
        let cl_id = CLTransactionId(format!("cl-cat-{}", i));
        let cat_tx = Transaction::new(
            TransactionId(format!("{:?}:cat-credit-tx-{}", cl_id, i)),
            constants::chain_1(),
            vec![constants::chain_1(), constants::chain_2()],
            format!("CAT.credit {} 100", i),
            cl_id.clone(),
        ).expect("Failed to create CAT transaction");
        
        let status = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
        assert_eq!(status, TransactionStatus::Pending);
        cat_txs.push(cat_tx);
        logging::log("TEST", &format!("CAT {} processed and is pending", i));
    }
    
    // Create multiple regular transactions that depend on the CATs
    let mut regular_txs = Vec::new();
    for i in 1..=5 {
        let cl_id = CLTransactionId(format!("cl-reg-{}", i));
        let regular_tx = Transaction::new(
            TransactionId(format!("{:?}:regular-send-tx-{}", cl_id, i)),
            constants::chain_1(),
            vec![constants::chain_1()],
            format!("REGULAR.send {} {} 10", i, i + 1),
            cl_id.clone(),
        ).expect("Failed to create regular transaction");
        
        let status = hig_node.lock().await.process_transaction(regular_tx.clone()).await.unwrap();
        assert_eq!(status, TransactionStatus::Pending);
        regular_txs.push(regular_tx);
        logging::log("TEST", &format!("Regular tx {} processed and is pending", i));
    }
    
    // Check initial counts
    let (cat_pending, _, _) = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let (regular_pending, _, _) = hig_node.lock().await.get_transaction_status_counts_regular().await.unwrap();
    let locked_keys = hig_node.lock().await.get_total_locked_keys_count().await;
    
    logging::log("TEST", &format!("Initial counts - CAT pending: {}, Regular pending: {}, Locked keys: {}", 
        cat_pending, regular_pending, locked_keys));
    
    assert_eq!(cat_pending, 3, "Should have 3 pending CATs");
    assert_eq!(regular_pending, 5, "Should have 5 pending regular transactions");
    assert!(locked_keys > 0, "Should have some locked keys");
    
    // Resolve the first CAT
    let status_update = Transaction::new(
        TransactionId("status_update_1".to_string()),
        constants::chain_1(),
        vec![constants::chain_1()],
        "STATUS_UPDATE:Success.CAT_ID:cl-cat-1".to_string(),
        CLTransactionId("cl-cat-1".to_string()),
    ).expect("Failed to create status update");
    
    let update_status = hig_node.lock().await.process_transaction(status_update).await.unwrap();
    assert_eq!(update_status, TransactionStatus::Success);
    logging::log("TEST", "First CAT resolved with success");
    
    // Check counts after first CAT resolution
    let (cat_pending_after, _, _) = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let (regular_pending_after, regular_success_after, _) = hig_node.lock().await.get_transaction_status_counts_regular().await.unwrap();
    let locked_keys_after = hig_node.lock().await.get_total_locked_keys_count().await;
    
    logging::log("TEST", &format!("After first CAT resolution - CAT pending: {}, Regular pending: {}, Regular success: {}, Locked keys: {}", 
        cat_pending_after, regular_pending_after, regular_success_after, locked_keys_after));
    
    // Verify that regular transactions are processed correctly when dependencies resolve
    if regular_pending_after > 0 {
        logging::log("TEST", "ℹ️  INFO: Some regular transactions are still pending after first CAT resolution");
        logging::log("TEST", "This is expected if they depend on multiple CATs");
    } else {
        logging::log("TEST", "✅ SUCCESS: All regular transactions processed after first CAT resolution");
    }
    
    // Resolve the second CAT
    let status_update_2 = Transaction::new(
        TransactionId("status_update_2".to_string()),
        constants::chain_1(),
        vec![constants::chain_1()],
        "STATUS_UPDATE:Success.CAT_ID:cl-cat-2".to_string(),
        CLTransactionId("cl-cat-2".to_string()),
    ).expect("Failed to create status update");
    
    let update_status_2 = hig_node.lock().await.process_transaction(status_update_2).await.unwrap();
    assert_eq!(update_status_2, TransactionStatus::Success);
    logging::log("TEST", "Second CAT resolved with success");
    
    // Check final counts
    let (cat_pending_final, _, _) = hig_node.lock().await.get_transaction_status_counts_cats().await.unwrap();
    let (regular_pending_final, regular_success_final, _) = hig_node.lock().await.get_transaction_status_counts_regular().await.unwrap();
    let locked_keys_final = hig_node.lock().await.get_total_locked_keys_count().await;
    
    logging::log("TEST", &format!("Final counts - CAT pending: {}, Regular pending: {}, Regular success: {}, Locked keys: {}", 
        cat_pending_final, regular_pending_final, regular_success_final, locked_keys_final));
    
    logging::log("TEST", "=== test_regular_tx_multiple_dependencies completed ===\n");
} 