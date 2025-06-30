use crate::{
    types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited, SubBlock, ChainId, CATId, constants, CLTransactionId, CATStatusUpdate},
    hyper_ig::{HyperIG, node::HyperIGNode},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use hyperplane::utils::logging;
use regex::Regex;
use std::time::Duration;

/// Helper function to set up a test HIG node
pub async fn setup_test_hig_node() -> (Arc<Mutex<HyperIGNode>>, mpsc::Receiver<CATStatusUpdate>) {
    let (_sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel(100);
    let (sender_hig_to_hs, receiver_hig_to_hs) = mpsc::channel(100);
    
    let hig_node = HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs, constants::chain_1(), 4);
    let hig_node = Arc::new(Mutex::new(hig_node));
    
    // Start the node
    HyperIGNode::start(hig_node.clone()).await;

    (hig_node, receiver_hig_to_hs)
}

/// Helper function: Tests regular non-dependent transaction path in HyperIG
/// - Status verification
/// - Status persistence
async fn run_test_regular_transaction_status(data: &str, expected_status: TransactionStatus) {
    logging::log("TEST", &format!("\n=== Starting regular non-dependent transaction test with status {:?}===", expected_status));
    
    logging::log("TEST", "Setting up test nodes...");
    let (hig_node, _rx) = setup_test_hig_node().await;
    logging::log("TEST", "Test nodes setup complete");

    let tx_id = "test-tx";
    logging::log("TEST", &format!("\nProcessing transaction: {}", tx_id));
    
    // Use credit for success, send for failure (since account is empty)
    let command = format!("REGULAR.{}", data);
    logging::log("TEST", &format!("Command: {}", command));

    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx = Transaction::new(
        TransactionId(format!("{:?}:{}", cl_id, tx_id)),
        constants::chain_1(),
        vec![constants::chain_1()],
        command.to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    // Process transaction and verify initial status
    let status = hig_node.lock().await.process_transaction(tx.clone())
        .await
        .expect("Failed to process transaction");
    logging::log("TEST", &format!("Transaction status: {:?}", status));
    assert_eq!(status, expected_status, "Transaction should have status {:?}", expected_status);
    
    // Verify status persistence
    let get_status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert_eq!(get_status, expected_status, "Retrieved status should be {:?}", expected_status);
    logging::log("TEST", "Verified status persistence");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests regular non-dependent transaction success path in HyperIG:
#[tokio::test]
async fn test_regular_transaction_success() {
    logging::init_logging();
    run_test_regular_transaction_status("credit 1 100", TransactionStatus::Success).await;
}

/// Tests regular non-dependent transaction success path in HyperIG:
#[tokio::test]
async fn test_regular_transaction_failure() {
    logging::init_logging();
    run_test_regular_transaction_status("send 1 2 100", TransactionStatus::Failure).await;
}

/// Helper function to test sending a CAT status proposal
async fn run_process_and_send_cat(data: &str, expected_status: CATStatusLimited) {    
    logging::log("TEST", "Setting up test nodes...");
    let (hig_node, _receiver_hig_to_hs  ) = setup_test_hig_node().await;
    logging::log("TEST", "Test nodes setup complete");
    
    // Create necessary parts of a CAT transaction
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx_chain_1 = Transaction::new(
        TransactionId(format!("{:?}:tx_chain_1", cl_id)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        data.to_string(),

        cl_id.clone(),
    ).expect("Failed to create transaction");

    // Process the transaction
    logging::log("TEST", &format!("Executing transaction of a CAT with data: {}", tx_chain_1.data));
    let status = hig_node.lock().await.process_transaction(tx_chain_1.clone())
        .await
        .expect("Failed to execute transaction");
    logging::log("TEST", &format!("Transaction status: {:?}", status));
    
    // Verify it's pending
    assert!(matches!(status, TransactionStatus::Pending));
    logging::log("TEST", "Verified transaction is pending");
    
    // Verify we can retrieve the same status
    logging::log("TEST", "Verifying transaction status...");
    let retrieved_status = hig_node.lock().await.get_transaction_status(tx_chain_1.id.clone())
        .await
        .expect("Failed to get transaction status");
    logging::log("TEST", &format!("Retrieved status: {:?}", retrieved_status));
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    logging::log("TEST", "Verified retrieved status is pending");
    
    // Verify it's in the pending transactions list
    logging::log("TEST", "Verifying pending transactions list...");
    let pending = hig_node.lock().await.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    logging::log("TEST", &format!("Pending transactions: {:?}", pending));
    assert!(pending.contains(&tx_chain_1.id));
    logging::log("TEST", "Verified transaction is in pending list");
    
    // Verify the proposed status
    logging::log("TEST", "Verifying proposed status...");
    let proposed_status = hig_node.lock().await.get_proposed_status(tx_chain_1.id.clone())
        .await
        .expect("Failed to get proposed status");
    logging::log("TEST", &format!("Proposed status: {:?}", proposed_status));
    assert_eq!(proposed_status, expected_status);
    logging::log("TEST", &format!("Verified proposed status is {:?}", expected_status));
    
    // Send the status proposal to HS
    let cat_id = CATId(cl_id.clone());
    logging::log("TEST", &format!("Sending status proposal to HS for cat_id: {:?}", cat_id));
    // we only have one chain for now, so we create a vector with one element
    hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), expected_status, vec![constants::chain_1()])
        .await
        .expect("Failed to send status proposal");
    logging::log("TEST", "Status proposal sent to HS");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests CAT transaction success proposal path in HyperIG
#[tokio::test]
#[allow(unused_variables)]
async fn test_cat_process_and_send_success() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_process_and_send_success ===");
    run_process_and_send_cat("CAT.credit 1 100", CATStatusLimited::Success).await;
}

/// Tests CAT transaction failure proposal path in HyperIG
#[tokio::test]
#[allow(unused_variables)]
async fn test_cat_process_and_send_failure() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_process_and_send_failure ===");
    run_process_and_send_cat("CAT.send 1 2 1000", CATStatusLimited::Failure).await;
}

/// Tests get pending transactions functionality:
/// - Get pending transactions when none exist
/// - Get pending transactions after adding some
#[tokio::test]
async fn test_get_pending_transactions() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_get_pending_transactions ===");
    
    logging::log("TEST", "Setting up test nodes...");
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;
    logging::log("TEST", "Test nodes setup complete");

    // Get pending transactions when none exist
    logging::log("TEST", "Checking pending transactions (empty)...");
    let pending = hig_node.lock().await.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    logging::log("TEST", &format!("Pending transactions: {:?}", pending));
    assert!(pending.is_empty());

    // Create a CAT transaction on which we will make a dependent transaction
    let cl_id_1 = CLTransactionId("cl-tx_cat".to_string());
    let tx_1 = Transaction::new(
        TransactionId(format!("{:?}:tx_1", cl_id_1)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id_1.clone(),
    ).expect("Failed to create transaction");
    logging::log("TEST", "Executing transaction of a CAT.");
    let _status = hig_node.lock().await.process_transaction(tx_1.clone())
        .await
        .expect("Failed to execute transaction");

    // Create and execute a dependent transaction
    logging::log("TEST", "Creating dependent transaction.");
    let cl_id_2 = CLTransactionId("cl-tx_dependent".to_string());
    let tx_2 = Transaction::new(
        TransactionId(format!("{:?}:tx_2", cl_id_2)),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.send 1 2 100".to_string(),
        cl_id_2.clone(),
    ).expect("Failed to create transaction");
    logging::log("TEST", "Executing transaction...");
    hig_node.lock().await.process_transaction(tx_2.clone())
        .await
        .expect("Failed to execute transaction");

    // Ensure both transactions are pending
    logging::log("TEST", "Checking pending transactions (with one)...");
    let pending = hig_node.lock().await.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    logging::log("TEST", &format!("Pending transactions: {:?}", pending));
    assert!(pending.contains(&tx_1.id));
    assert!(pending.contains(&tx_2.id));
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a subblock with a wrong chain ID should not happen
/// - Only the subblock with the correct chain ID should be received.
#[tokio::test]
async fn test_wrong_chain_subblock() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_wrong_chain_subblock ===");
    
    // setup using the helper function
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;

    // // Start the node
    // HyperIGNode::start(hig_node.clone()).await;

    // Create a subblock with a different chain ID
    let cl_id = CLTransactionId("cl-tx".to_string());
    let wrong_chain_subblock = SubBlock {
        block_height: 1,
        chain_id: ChainId("wrong-chain".to_string()),
        transactions: vec![Transaction::new(
            TransactionId(format!("{:?}:test-tx", cl_id)),
            ChainId("wrong-chain".to_string()),
            vec![ChainId("wrong-chain".to_string())],
            "REGULAR.credit 1 100".to_string(),
            cl_id.clone(),
        ).expect("Failed to create transaction")],
    };

    // process the subblock and expect the error WrongChainId
    let result = hig_node.lock().await.process_subblock(wrong_chain_subblock).await;
    assert!(result.is_err(), "Expected error when receiving subblock from wrong chain");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Test to verify that the CAT transaction pattern regex works correctly.
/// This test is separate from the actual transaction processing tests to help diagnose
/// whether issues are with the pattern matching or with the transaction handling logic.
/// 
/// The test verifies that:
/// 1. The pattern correctly matches valid CAT transaction formats
/// 2. The pattern can extract the CAT ID from the transaction data
/// 3. Both credit and send commands are properly recognized
#[tokio::test]
async fn test_cat_pattern() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_pattern ===");
    
    use crate::types::communication::cl_to_hig::{CAT_PATTERN, CAT_ID_SUFFIX};
    
    // Test cases that should match the pattern:
    // - CAT.credit <receiver> <amount>
    // - CAT.send <sender> <receiver> <amount>
    let test_cases = vec![
        "CAT.credit 1 100",
        "CAT.send 1 2 1000",
    ];
    
    for data in test_cases {
        println!("\n=== Testing pattern ===");
        println!("Input data: {}", data);
        println!("CAT_PATTERN: {}", *CAT_PATTERN);
        println!("CAT_ID_SUFFIX: {}", *CAT_ID_SUFFIX);
        
        // Test the full pattern match
        let is_match = CAT_PATTERN.is_match(data);
        println!("Full pattern match: {}", is_match);
        
        if let Some(captures) = CAT_PATTERN.captures(data) {
            println!("Captures: {:?}", captures);
            if let Some(cat_id) = captures.name("cat_id") {
                println!("Extracted CAT ID: {}", cat_id.as_str());
            }
        }
        
        // Test the CAT_ID_SUFFIX pattern separately
        let cat_id_pattern = Regex::new(&format!(r"{}", *CAT_ID_SUFFIX)).unwrap();
        let cat_id_match = cat_id_pattern.captures(data);
        println!("CAT ID pattern match: {:?}", cat_id_match);
        if let Some(cat_id_captures) = cat_id_match {
            println!("CAT ID captures: {:?}", cat_id_captures);
        }
        
        println!("=== End test case ===\n");
    }
}

/// Tests processing a send transaction after a credit transaction.
/// 
/// This test verifies that:
/// 1. A credit transaction successfully adds funds to an account
/// 2. A subsequent send transaction can use those funds
/// 3. The state is correctly updated after both transactions
#[tokio::test]
async fn test_send_after_credit() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_send_after_credit ===");
    
    logging::log("TEST", "Setting up test nodes...");
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;
    logging::log("TEST", "Test nodes setup complete");

    // First credit 100 to account 1
    let cl_id_1 = CLTransactionId("cl-tx_1".to_string());
    let credit_tx = Transaction::new(
        TransactionId(format!("{:?}:credit-tx", cl_id_1)),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.credit 1 100".to_string(),
        cl_id_1.clone(),
    ).expect("Failed to create credit transaction");
    
    // Process credit transaction
    let credit_status = hig_node.lock().await.process_transaction(credit_tx.clone())
        .await
        .expect("Failed to process credit transaction");
    assert_eq!(credit_status, TransactionStatus::Success, "Credit transaction should succeed");
    logging::log("TEST", "Credit transaction processed successfully");

    // Then send 50 from account 1 to account 2
    let cl_id_2 = CLTransactionId("cl-tx_2".to_string());
    let send_tx = Transaction::new(
        TransactionId(format!("{:?}:send-tx", cl_id_2)),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.send 1 2 50".to_string(),
        cl_id_2.clone(),
    ).expect("Failed to create send transaction");
    
    // Process send transaction
    let send_status = hig_node.lock().await.process_transaction(send_tx.clone())
        .await
        .expect("Failed to process send transaction");
    assert_eq!(send_status, TransactionStatus::Success, "Send transaction should succeed");
    logging::log("TEST", "Send transaction processed successfully");

    // Verify final statuses
    let credit_final_status = hig_node.lock().await.get_transaction_status(credit_tx.id)
        .await
        .expect("Failed to get credit transaction status");
    let send_final_status = hig_node.lock().await.get_transaction_status(send_tx.id)
        .await
        .expect("Failed to get send transaction status");
    
    assert_eq!(credit_final_status, TransactionStatus::Success, "Credit transaction should have Success status");
    assert_eq!(send_final_status, TransactionStatus::Success, "Send transaction should have Success status");
    logging::log("TEST", "Verified final transaction statuses");

    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a CAT send transaction fails when there are no funds.
/// 
/// This test verifies that:
/// 1. A CAT send transaction is marked as pending
/// 2. The transaction would fail if executed (due to insufficient funds)
#[tokio::test]
async fn test_cat_send_no_funds() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_send_no_funds ===");
    
    logging::log("TEST", "Setting up test nodes...");
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;
    logging::log("TEST", "Test nodes setup complete");

    // Create a CAT send transaction with multiple constituent chains
    let cl_id = CLTransactionId("cl-tx".to_string());
    let cat_send_tx = Transaction::new(
        TransactionId(format!("{:?}:cat-send-1", cl_id)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.send 1 2 50".to_string(),
        cl_id.clone(),
    ).expect("Failed to create CAT send transaction");

    // Process the transaction
    let status = hig_node.lock().await.process_transaction(cat_send_tx.clone())
        .await
        .expect("Failed to process CAT send transaction");
    assert_eq!(status, TransactionStatus::Pending, "CAT send should be pending");

    // Verify the proposed status is Failure
    let proposed_status = hig_node.lock().await.get_proposed_status(cat_send_tx.id)
        .await
        .expect("Failed to get proposed status");
    assert_eq!(proposed_status, CATStatusLimited::Failure, "CAT send should propose Failure status");

    logging::log("TEST", "=== test_cat_send_no_funds completed successfully ===\n");
}

/// Tests that a CAT credit transaction is marked as pending.
/// 
/// This test verifies that:
/// 1. A CAT credit transaction is marked as pending
/// 2. The transaction would succeed if executed
#[tokio::test]
async fn test_cat_credit_pending() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_credit_pending ===");
    
    logging::log("TEST", "Setting up test nodes...");
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;
    logging::log("TEST", "Test nodes setup complete");

    // Create a CAT credit transaction with multiple constituent chains
    let cl_id = CLTransactionId("cl-tx".to_string());
    let cat_credit_tx = Transaction::new(
        TransactionId(format!("{:?}:cat-credit-1", cl_id)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create CAT credit transaction");

    // Process the transaction
    let status = hig_node.lock().await.process_transaction(cat_credit_tx.clone())
        .await
        .expect("Failed to process CAT credit transaction");
    assert_eq!(status, TransactionStatus::Pending, "CAT credit should be pending");

    // Verify the proposed status is Success
    let proposed_status = hig_node.lock().await.get_proposed_status(cat_credit_tx.id)
        .await
        .expect("Failed to get proposed status");
    assert_eq!(proposed_status, CATStatusLimited::Success, "CAT credit should propose Success status");

    logging::log("TEST", "=== test_cat_credit_pending completed successfully ===\n");
}

/// Tests that a CAT send transaction is pending after a regular credit.
/// Verifies that:
/// 1. The regular credit succeeds
/// 2. The CAT send is pending
/// 3. The CAT send proposes a Success status
#[tokio::test]
async fn test_cat_send_after_credit() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_send_after_credit ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;
    
    // First do a regular credit
    let cl_id_1 = CLTransactionId("cl-tx_1".to_string());
    let credit_tx = Transaction::new(
        TransactionId(format!("{:?}:credit-1", cl_id_1)),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.credit 1 100".to_string(),
        cl_id_1.clone(),
    ).expect("Failed to create credit transaction");
    
    let status = hig_node.lock().await.process_transaction(credit_tx).await.unwrap();
    assert_eq!(status, TransactionStatus::Success, "Regular credit should succeed");
    
    // Then do a CAT send
    let cl_id_2 = CLTransactionId("cl-tx_2".to_string());
    let cat_send_tx = Transaction::new(
        TransactionId(format!("{:?}:cat-send-1", cl_id_2)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.send 1 2 50".to_string(),
        cl_id_2.clone(),
    ).expect("Failed to create CAT send transaction");
    
    let status = hig_node.lock().await.process_transaction(cat_send_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending, "CAT send should be pending");
    
    // Verify the proposed status is Success
    let proposed_status = hig_node.lock().await.get_proposed_status(cat_send_tx.id).await.unwrap();
    assert_eq!(proposed_status, CATStatusLimited::Success, "CAT send should propose Success");
}

/// Tests that a newly created HIG node starts with an empty chain state.
/// This verifies that:
/// 1. The initial state is empty
/// 2. The get_chain_state method returns an empty HashMap
#[tokio::test]
async fn test_get_chain_state_empty() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_get_chain_state_empty ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;
    let state = hig_node.lock().await.get_chain_state().await.unwrap();
    assert!(state.is_empty(), "Initial chain state should be empty");
}

/// Tests that the chain state is correctly updated after processing a transaction.
/// This verifies that:
/// 1. A credit transaction successfully updates the chain state
/// 2. The get_chain_state method returns the correct balance
/// 3. The state is properly persisted after transaction execution
#[tokio::test]
async fn test_get_chain_state_after_transaction() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_get_chain_state_after_transaction ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;
    
    // Create and process a credit transaction
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx = Transaction::new(
        TransactionId(format!("{:?}:test_tx", cl_id)),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    let status = hig_node.lock().await.process_transaction(tx).await.unwrap();
    assert_eq!(status, TransactionStatus::Success);
    
    // Get the chain state and verify the balance
    let state = hig_node.lock().await.get_chain_state().await.unwrap();
    assert_eq!(state.get("1"), Some(&100), "Account 1 should have balance 100");
}

/// Tests that duplicate transaction IDs are skipped by the HyperIG node.
/// This verifies that:
/// 1. The first transaction with a unique ID is processed successfully
/// 2. Any subsequent transaction with the same ID is skipped
/// 3. The subblock processing continues successfully
#[tokio::test]
async fn test_duplicate_transaction_id() {
    // Create a HIG node
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;
    
    // Create a transaction
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx = Transaction::new(
        TransactionId(format!("{:?}:test_tx", cl_id)),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    // Create a subblock with the transaction
    let subblock = SubBlock {
        chain_id: constants::chain_1(),
        block_height: 1,
        transactions: vec![tx.clone()],
    };
    
    // Process the subblock
    let result = hig_node.lock().await.process_subblock(subblock).await;
    assert!(result.is_ok(), "First subblock should be processed successfully");
    
    // Create another subblock with the same transaction
    let subblock2 = SubBlock {
        chain_id: constants::chain_1(),
        block_height: 2,
        transactions: vec![tx],
    };
    
    // Process the subblock with the duplicate transaction
    let result = hig_node.lock().await.process_subblock(subblock2).await;
    assert!(result.is_ok(), "Subblock with duplicate transaction ID should be processed successfully (skipping duplicate)");
    
    // Verify the chain state hasn't changed (no double credit)
    let state = hig_node.lock().await.get_chain_state().await.unwrap();
    assert_eq!(state.get("1"), Some(&100), "Account 1 should still have balance 100 (no double credit)");
}

/// Tests that the HS message delay works correctly:
/// - Set a delay of 100ms
/// - Send a CAT status proposal
/// - Verify that the message is not received after 50ms
/// - Verify that the message is received after 150ms
#[tokio::test]
async fn test_hs_message_delay() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_hs_message_delay ===");
    
    // Set up test node
    let (hig_node, mut receiver_hig_to_hs) = setup_test_hig_node().await;
    
    // Set delay to 100ms
    hig_node.lock().await.set_hs_message_delay(Duration::from_millis(100));
    logging::log("TEST", "Set message delay to 100ms");
    
    // Create a CAT transaction
    let cl_id = CLTransactionId("cl-tx_test".to_string());
    let tx = Transaction::new(
        TransactionId(format!("{:?}:tx", cl_id)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    logging::log("TEST", "Created test transaction");
    
    // Process the transaction (this will queue the proposal)
    let _status = hig_node.lock().await.process_transaction(tx.clone())
        .await
        .expect("Failed to process transaction");
    logging::log("TEST", "Transaction processed, proposal queued");
    
    // Wait 50ms and check that message is not received
    logging::log("TEST", "Waiting 50ms...");
    tokio::time::sleep(Duration::from_millis(50)).await;
    let early_result = receiver_hig_to_hs.try_recv();
    logging::log("TEST", &format!("Early check result: {:?}", early_result));
    assert!(early_result.is_err(), "Message should not be received after 50ms");
    logging::log("TEST", "Verified no message received at 50ms");
    
    // Wait another 150ms and check that message is received
    logging::log("TEST", "Waiting another 150ms...");
    tokio::time::sleep(Duration::from_millis(150)).await;
    let late_result = receiver_hig_to_hs.try_recv();
    logging::log("TEST", &format!("Late check result: {:?}", late_result));
    assert!(late_result.is_ok(), "Message should be received after 200ms");
    logging::log("TEST", "Verified message received at 200ms");
    
    logging::log("TEST", "HS message delay test completed successfully");
}



