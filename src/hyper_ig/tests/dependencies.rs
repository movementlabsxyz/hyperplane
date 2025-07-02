use crate::hyper_ig::node::HyperIGNode;
use crate::types::{Transaction, TransactionId, TransactionStatus, ChainId, CLTransactionId};
use crate::utils::logging;
use crate::hyper_ig::HyperIG;
use crate::hyper_ig::tests::basic::setup_test_hig_node;



/// Runs a dependency test scenario where a transaction depends on a CAT transaction.
/// 
/// # Arguments
/// * `cat_status` - The final status to set for the CAT transaction (Success/Failure)
/// * `expected_result` - The expected final status of the dependent transaction
/// 
/// # Test Flow
/// 1. Creates a CAT transaction that credits key "1"
/// 2. Creates a dependent transaction that sends from key "1" to key "2"
/// 3. Processes both transactions (both start as Pending)
/// 4. Verifies the dependency is correctly established
/// 5. Resolves the CAT with the given status
/// 6. Verifies the dependent transaction reaches the expected result
/// 
/// # Returns
/// The HyperIG node and its VM state for further testing
async fn run_cat_credit_and_dependent_tx(
    cat_status: TransactionStatus, 
    expected_result: TransactionStatus
) -> std::sync::Arc<tokio::sync::Mutex<HyperIGNode>> {
    logging::init_logging();
    logging::log("TEST", &format!("\n=== Starting test with CAT status: {:?}, expected result: {:?} ===", cat_status, expected_result));
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;

    // Create a transaction that is part of a CAT that credits key "1"
    let cl_id_1 = CLTransactionId("cl-tx_1".to_string());
    let cat_tx = Transaction::new(
        TransactionId(format!("{:?}:cat-credit-tx", cl_id_1)),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())],
        "CAT.credit 1 100".to_string(),
        cl_id_1.clone(),
    ).expect("Failed to create CAT transaction");

    // Create a regular transaction that depends on the CAT
    let cl_id_2 = CLTransactionId("cl-tx_2".to_string());
    let dependent_tx = Transaction::new(
        TransactionId(format!("{:?}:dependent-send-tx", cl_id_2)),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 2 50".to_string(),
        cl_id_2.clone(),
    ).expect("Failed to create dependent transaction");
    
    // Process the CAT first
    let status = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Process the dependent transaction
    let status = hig_node.lock().await.process_transaction(dependent_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);

    // Check the correctness of the dependency
    let dependencies = hig_node.lock().await.get_transaction_dependencies(dependent_tx.id.clone()).await.unwrap();
    logging::log("TEST", &format!("Dependencies: {:?}", dependencies));
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0], cat_tx.id.clone());

    // Resolve the CAT with the given status
    let status_str = match cat_status {
        TransactionStatus::Success => "Success",
        TransactionStatus::Failure => "Failure",
        _ => panic!("Invalid status for test"),
    };
    let status_update = Transaction::new(
        TransactionId(format!("{:?}:status-1", cl_id_1)),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        format!("STATUS_UPDATE:{}.CAT_ID:{}", status_str, cl_id_1.0),
        cl_id_1.clone(),
    ).expect("Failed to create transaction");
    hig_node.lock().await.process_transaction(status_update).await.unwrap();

    // Check that dependencies are cleared
    let dependencies = hig_node.lock().await.get_transaction_dependencies(dependent_tx.id.clone()).await.unwrap();
    logging::log("TEST", &format!("Dependencies: {:?}", dependencies));
    assert_eq!(dependencies.len(), 0);
    
    // Verify the dependent transaction has the expected result
    let status = hig_node.lock().await.get_transaction_status(dependent_tx.id.clone()).await.unwrap();
    assert_eq!(status, expected_result);
    
    logging::log("TEST", "=== Test completed successfully ===\n");
    
    hig_node
}

/// Tests that a transaction succeeds when its CAT dependency succeeds.
/// 
/// This test verifies that when a CAT transaction with credit succeeds, a dependent transaction
/// that were waiting on the credit will also succeed.
#[tokio::test]
pub async fn test_success_dependency() {
    run_cat_credit_and_dependent_tx(TransactionStatus::Success, TransactionStatus::Success).await;
}

/// Tests that a transaction fails when its CAT dependency fails.
/// 
/// This test verifies that when a CAT transaction with credit fails, a dependent transactions
/// that were waiting on the credit will also fail.
#[tokio::test]
pub async fn test_failed_dependency() {
    run_cat_credit_and_dependent_tx(TransactionStatus::Failure, TransactionStatus::Failure).await;
}

/// Tests that multiple transactions waiting on the same key are processed in order.
/// 
/// This test verifies that when multiple transactions are waiting on the same key,
/// they are processed in the order they were submitted, maintaining transaction
/// ordering guarantees.
/// 1. A cat with credit will be created. 
/// 2. A transaction will be created that sends from 1 to 2.
/// 3. A transaction will be created that sends from 1 to 3.
/// 4. The first transaction will be processed and succeed because the cat's key has enough credit.
/// 5. The second transaction will be processed and fail because the cat's key does not have enough credit.
#[tokio::test]
pub async fn test_multiple_transactions_same_key_fail() {
    let hig_node = run_cat_credit_and_dependent_tx(TransactionStatus::Success, TransactionStatus::Success).await;

    let cl_id_2 = CLTransactionId("cl-tx_2".to_string());
    let dependent_tx_2 = Transaction::new(
        TransactionId(format!("{:?}:dependent-tx-2", cl_id_2)),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 2 60".to_string(),
        cl_id_2.clone(),
    ).expect("Failed to create dependent transaction");

    // the second transaction will fail because the cat's key does not have enough credit
    let status = hig_node.lock().await.process_transaction(dependent_tx_2.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Failure);

    logging::log("TEST", "=== Test completed successfully ===\n");
} 


/// Tests that multiple transactions waiting on the same key are processed in order.
/// 
/// This test verifies that when multiple transactions are waiting on the same key,
/// they are processed in the order they were submitted, maintaining transaction
/// ordering guarantees.
/// 1. A cat with credit will be created. 
/// 2. A transaction will be created that sends from 1 to 2.
/// 3. A transaction will be created that sends from 1 to 3.
/// 4. The first transaction will be processed and succeed because the cat's key has enough credit.
/// 5. The second transaction will be processed and also succeed because the cat's key has enough credit.
#[tokio::test]
pub async fn test_multiple_transactions_same_key_success() {
    let hig_node = run_cat_credit_and_dependent_tx(TransactionStatus::Success, TransactionStatus::Success).await;

    let cl_id_2 = CLTransactionId("cl-tx_2".to_string());
    let dependent_tx_2 = Transaction::new(
        TransactionId(format!("{:?}:dependent-tx-2", cl_id_2)),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 2 40".to_string(),
        cl_id_2.clone(),
    ).expect("Failed to create dependent transaction");

    // the second transaction will succeed because the cat's key has enough credit
    let status = hig_node.lock().await.process_transaction(dependent_tx_2.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Success);

    logging::log("TEST", "=== Test completed successfully ===\n");
} 

/// Tests that locks are properly released when a CAT transaction is marked as successful.
/// 
/// This test verifies the critical lock release mechanism:
/// 1. A CAT transaction acquires locks on keys
/// 2. The CAT is marked as success
/// 3. A status update is processed
/// 4. The locks are properly released
/// 5. Subsequent transactions can access the previously locked keys
/// 
/// This test would catch the issue where CATs get stuck in a "success" state
/// without releasing their locks, which was observed in the simulation.
#[tokio::test]
pub async fn test_cat_lock_release_on_success() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_lock_release_on_success ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;

    // Step 1: Create a CAT transaction that will lock key "1"
    let cl_id_1 = CLTransactionId("cl-cat-tx-1".to_string());
    let cat_tx = Transaction::new(
        TransactionId(format!("{:?}:cat-credit-tx", cl_id_1)),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())],
        "CAT.credit 1 100".to_string(),
        cl_id_1.clone(),
    ).expect("Failed to create CAT transaction");

    // Step 2: Process the CAT transaction - it should be pending and lock key "1"
    let status = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    logging::log("TEST", "CAT transaction processed and set to pending");

    // Step 3: Verify that key "1" is locked by the CAT
    let locked_keys = hig_node.lock().await.get_locked_keys_by_transaction(cat_tx.id.clone()).await;
    assert!(locked_keys.contains(&"1".to_string()), "Key '1' should be locked by the CAT");
    logging::log("TEST", &format!("Verified that keys {:?} are locked by the CAT", locked_keys));

    // Step 4: Create a regular transaction that tries to access key "1" - it should be blocked
    let cl_id_2 = CLTransactionId("cl-reg-tx-1".to_string());
    let blocked_tx = Transaction::new(
        TransactionId(format!("{:?}:blocked-send-tx", cl_id_2)),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 3 10".to_string(),
        cl_id_2.clone(),
    ).expect("Failed to create blocked transaction");

    let blocked_status = hig_node.lock().await.process_transaction(blocked_tx.clone()).await.unwrap();
    assert_eq!(blocked_status, TransactionStatus::Pending, "Transaction should be blocked and pending");
    logging::log("TEST", "Verified that regular transaction is blocked by the CAT");

    // Step 5: Create a status update transaction that marks the CAT as successful
    let status_update = Transaction::new(
        TransactionId(format!("{:?}:status-update", cl_id_1)),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        format!("STATUS_UPDATE:Success.CAT_ID:{}", cl_id_1.0),
        cl_id_1.clone(),
    ).expect("Failed to create status update transaction");

    // Step 6: Process the status update - this should release the locks
    let update_status = hig_node.lock().await.process_transaction(status_update).await.unwrap();
    assert_eq!(update_status, TransactionStatus::Success, "Status update should be successful");
    logging::log("TEST", "Status update processed successfully");

    // Step 7: Verify that key "1" is no longer locked
    let remaining_locked_keys = hig_node.lock().await.get_locked_keys_by_transaction(cat_tx.id.clone()).await;
    assert!(remaining_locked_keys.is_empty(), "Key '1' should no longer be locked by the CAT");
    logging::log("TEST", "Verified that locks have been released");

    // Step 8: Verify that the blocked transaction is now processed
    let final_blocked_status = hig_node.lock().await.get_transaction_status(blocked_tx.id.clone()).await.unwrap();
    assert_eq!(final_blocked_status, TransactionStatus::Success, "Blocked transaction should now be successful");
    logging::log("TEST", "Verified that blocked transaction is now processed successfully");

    // Step 9: Create another transaction that accesses key "1" - it should succeed immediately
    let cl_id_3 = CLTransactionId("cl-reg-tx-2".to_string());
    let new_tx = Transaction::new(
        TransactionId(format!("{:?}:new-send-tx", cl_id_3)),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 4 5".to_string(),
        cl_id_3.clone(),
    ).expect("Failed to create new transaction");

    let new_status = hig_node.lock().await.process_transaction(new_tx.clone()).await.unwrap();
    assert_eq!(new_status, TransactionStatus::Success, "New transaction should succeed immediately");
    logging::log("TEST", "Verified that new transaction can access key '1' immediately");

    logging::log("TEST", "=== test_cat_lock_release_on_success completed successfully ===\n");
} 
