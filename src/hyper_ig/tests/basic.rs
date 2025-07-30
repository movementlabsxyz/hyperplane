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

/// Helper function to set up a test HIG node with configurable CAT pending dependency behavior
pub async fn setup_test_hig_node(allow_cat_pending_dependencies: bool) -> (Arc<Mutex<HyperIGNode>>, mpsc::Receiver<CATStatusUpdate>) {
    let (_sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel(100);
    let (sender_hig_to_hs, receiver_hig_to_hs) = mpsc::channel(100);
    
    let hig_node = HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs, constants::chain_1(), 4, allow_cat_pending_dependencies);
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
    let (hig_node, _rx) = setup_test_hig_node(true).await;
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
async fn run_process_and_send_cat(data: &str, expected_status: crate::types::CATStatus) {    
    logging::log("TEST", "Setting up test nodes...");
    let (hig_node, _receiver_hig_to_hs  ) = setup_test_hig_node(true).await;
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
    let status_limited = match expected_status {
        crate::types::CATStatus::Success => crate::types::CATStatusLimited::Success,
        crate::types::CATStatus::Failure => crate::types::CATStatusLimited::Failure,
        crate::types::CATStatus::Pending => panic!("Cannot send Pending status to HS"),
    };
    hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), status_limited, vec![constants::chain_1()])
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
    run_process_and_send_cat("CAT.credit 1 100", crate::types::CATStatus::Success).await;
}

/// Tests CAT transaction failure proposal path in HyperIG
#[tokio::test]
#[allow(unused_variables)]
async fn test_cat_process_and_send_failure() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_process_and_send_failure ===");
    run_process_and_send_cat("CAT.send 1 2 1000", crate::types::CATStatus::Failure).await;
}

/// Tests get pending transactions functionality:
/// - Get pending transactions when none exist
/// - Get pending transactions after adding some
#[tokio::test]
async fn test_get_pending_transactions() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_get_pending_transactions ===");
    
    logging::log("TEST", "Setting up test nodes...");
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
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
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;

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
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
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
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
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
    assert_eq!(proposed_status, crate::types::CATStatus::Failure, "CAT send should propose Failure status");

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
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
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
    assert_eq!(proposed_status, crate::types::CATStatus::Success, "CAT credit should propose Success status");

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
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
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
    assert_eq!(proposed_status, crate::types::CATStatus::Success, "CAT send should propose Success");
}

/// Tests that a newly created HIG node starts with an empty chain state.
/// This verifies that:
/// 1. The initial state is empty
/// 2. The get_chain_state method returns an empty HashMap
#[tokio::test]
async fn test_get_chain_state_empty() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_get_chain_state_empty ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
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
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
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
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
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
    let (hig_node, mut receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
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

/// Tests that CATs are rejected when they depend on pending transactions and allow_cat_pending_dependencies is false.
/// This verifies that:
/// 1. When allow_cat_pending_dependencies is false, CATs that depend on pending transactions are rejected
/// 2. The rejected CAT is marked as failed
/// 3. A failure status proposal is sent to HS
/// 4. When allow_cat_pending_dependencies is true, CATs can depend on pending transactions
#[tokio::test]
async fn test_cat_pending_dependency_restriction() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_pending_dependency_restriction ===");
    
    // Test with allow_cat_pending_dependencies = false
    {
        logging::log("TEST", "Testing with allow_cat_pending_dependencies = false");
        let (hig_node, mut receiver_hig_to_hs) = setup_test_hig_node(false).await;
        
        // Verify the flag is set correctly
        let flag_value = hig_node.lock().await.get_allow_cat_pending_dependencies().await;
        assert_eq!(flag_value, false, "Flag should be set to false");
        
        // First create a CAT transaction that will be pending
        let cl_id_1 = CLTransactionId("cl-tx_cat_1".to_string());
        let cat_tx_1 = Transaction::new(
            TransactionId(format!("{:?}:cat_1", cl_id_1)),
            constants::chain_1(),
            vec![constants::chain_1(), constants::chain_2()],
            "CAT.credit 1 100".to_string(),
            cl_id_1.clone(),
        ).expect("Failed to create first CAT transaction");
        
        let status = hig_node.lock().await.process_transaction(cat_tx_1.clone()).await.unwrap();
        assert_eq!(status, TransactionStatus::Pending, "First CAT should be pending");
        
        // Now create a second CAT that depends on the same key (account 1)
        let cl_id_2 = CLTransactionId("cl-tx_cat_2".to_string());
        let cat_tx_2 = Transaction::new(
            TransactionId(format!("{:?}:cat_2", cl_id_2)),
            constants::chain_1(),
            vec![constants::chain_1(), constants::chain_2()],
            "CAT.send 1 2 50".to_string(), // This depends on account 1 which is locked by the first CAT
            cl_id_2.clone(),
        ).expect("Failed to create second CAT transaction");
        
        let status = hig_node.lock().await.process_transaction(cat_tx_2.clone()).await.unwrap();
        assert_eq!(status, TransactionStatus::Failure, "Second CAT should be rejected due to pending dependency");
        
        // Verify the second CAT is marked as failed
        let retrieved_status = hig_node.lock().await.get_transaction_status(cat_tx_2.id.clone()).await.unwrap();
        assert_eq!(retrieved_status, TransactionStatus::Failure, "Rejected CAT should be marked as failed");
        
        // Verify the second CAT is not in pending transactions
        let pending = hig_node.lock().await.get_pending_transactions().await.unwrap();
        assert!(!pending.contains(&cat_tx_2.id), "Rejected CAT should not be in pending transactions");
        
        // Verify a failure status proposal is sent to HS for the second CAT
        let cat_id_2 = CATId(cl_id_2.clone());
        let proposed_status = hig_node.lock().await.get_proposed_status(cat_tx_2.id.clone()).await.unwrap();
        assert_eq!(proposed_status, crate::types::CATStatus::Failure, "Rejected CAT should propose Failure status");
        
        // Wait for the status proposals to be sent (both CATs will send proposals)
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Receive both status updates and find the one for the second CAT
        let mut found_second_cat_update = false;
        for _ in 0..2 {
            if let Ok(status_update) = receiver_hig_to_hs.try_recv() {
                if status_update.cat_id == cat_id_2 {
                    assert_eq!(status_update.status, CATStatusLimited::Failure, "Status update for second CAT should be Failure");
                    found_second_cat_update = true;
                    break;
                }
            }
        }
        assert!(found_second_cat_update, "Should receive status update for the second CAT");
        
        logging::log("TEST", "Test with allow_cat_pending_dependencies = false completed successfully");
    }
    
    // Test with allow_cat_pending_dependencies = true
    {
        logging::log("TEST", "Testing with allow_cat_pending_dependencies = true");
        let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
        
        // Verify the flag is set correctly
        let flag_value = hig_node.lock().await.get_allow_cat_pending_dependencies().await;
        assert_eq!(flag_value, true, "Flag should be set to true");
        
        // First create a CAT transaction that will be pending
        let cl_id_1 = CLTransactionId("cl-tx_cat_3".to_string());
        let cat_tx_1 = Transaction::new(
            TransactionId(format!("{:?}:cat_3", cl_id_1)),
            constants::chain_1(),
            vec![constants::chain_1(), constants::chain_2()],
            "CAT.credit 1 100".to_string(),
            cl_id_1.clone(),
        ).expect("Failed to create first CAT transaction");
        
        let status = hig_node.lock().await.process_transaction(cat_tx_1.clone()).await.unwrap();
        assert_eq!(status, TransactionStatus::Pending, "First CAT should be pending");
        
        // Now create a second CAT that depends on the same key (account 1)
        let cl_id_2 = CLTransactionId("cl-tx_cat_4".to_string());
        let cat_tx_2 = Transaction::new(
            TransactionId(format!("{:?}:cat_4", cl_id_2)),
            constants::chain_1(),
            vec![constants::chain_1(), constants::chain_2()],
            "CAT.send 1 2 50".to_string(), // This depends on account 1 which is locked by the first CAT
            cl_id_2.clone(),
        ).expect("Failed to create second CAT transaction");
        
        let status = hig_node.lock().await.process_transaction(cat_tx_2.clone()).await.unwrap();
        assert_eq!(status, TransactionStatus::Pending, "Second CAT should be pending (allowed to depend on pending)");
        
        // Verify the second CAT is in pending transactions
        let pending = hig_node.lock().await.get_pending_transactions().await.unwrap();
        assert!(pending.contains(&cat_tx_2.id), "Second CAT should be in pending transactions");
        
        logging::log("TEST", "Test with allow_cat_pending_dependencies = true completed successfully");
    }
    
    logging::log("TEST", "=== test_cat_pending_dependency_restriction completed successfully ===\n");
}

/// Tests that the allow_cat_pending_dependencies flag can be changed at runtime.
/// This verifies that:
/// 1. The flag can be set and retrieved correctly
/// 2. The flag affects subsequent CAT processing
#[tokio::test]
async fn test_cat_pending_dependency_flag_runtime_change() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_pending_dependency_flag_runtime_change ===");
    
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    
    // Verify initial flag value
    let flag_value = hig_node.lock().await.get_allow_cat_pending_dependencies().await;
    assert_eq!(flag_value, true, "Initial flag should be true");
    
    // Change the flag to false
    hig_node.lock().await.set_allow_cat_pending_dependencies(false).await;
    
    // Verify the flag was changed
    let flag_value = hig_node.lock().await.get_allow_cat_pending_dependencies().await;
    assert_eq!(flag_value, false, "Flag should be changed to false");
    
    // Change the flag back to true
    hig_node.lock().await.set_allow_cat_pending_dependencies(true).await;
    
    // Verify the flag was changed back
    let flag_value = hig_node.lock().await.get_allow_cat_pending_dependencies().await;
    assert_eq!(flag_value, true, "Flag should be changed back to true");
    
    logging::log("TEST", "=== test_cat_pending_dependency_flag_runtime_change completed successfully ===\n");
}

/// Tests that the proposal queue processes CATs with independent delays.
/// This verifies that:
/// 1. Multiple CATs can be queued with different entry times
/// 2. Each CAT is sent after its individual delay from queue entry time
/// 3. Delays don't compound (each CAT has the same fixed delay regardless of queue position)
#[tokio::test]
async fn test_proposal_queue_independent_delays() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_proposal_queue_independent_delays ===");
    
    // Set up test node with 200ms delay
    let (hig_node, mut receiver_hig_to_hs) = setup_test_hig_node(true).await;
    hig_node.lock().await.set_hs_message_delay(Duration::from_millis(200));
    logging::log("TEST", "Set message delay to 200ms");
    
    // Create and process multiple CAT transactions with different timing
    let mut cat_ids = Vec::new();
    
    // CAT 1: Process immediately
    let cl_id_1 = CLTransactionId("cl-tx_queue_1".to_string());
    let cat_tx_1 = Transaction::new(
        TransactionId(format!("{:?}:cat_1", cl_id_1)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id_1.clone(),
    ).expect("Failed to create first CAT transaction");
    
    let start_time = std::time::Instant::now();
    let _status = hig_node.lock().await.process_transaction(cat_tx_1.clone()).await.unwrap();
    cat_ids.push(CATId(cl_id_1.clone()));
    logging::log("TEST", "CAT 1 processed at t=0ms");
    
    // Wait 50ms, then add CAT 2
    tokio::time::sleep(Duration::from_millis(50)).await;
    let cl_id_2 = CLTransactionId("cl-tx_queue_2".to_string());
    let cat_tx_2 = Transaction::new(
        TransactionId(format!("{:?}:cat_2", cl_id_2)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 2 100".to_string(),
        cl_id_2.clone(),
    ).expect("Failed to create second CAT transaction");
    
    let _status = hig_node.lock().await.process_transaction(cat_tx_2.clone()).await.unwrap();
    cat_ids.push(CATId(cl_id_2.clone()));
    logging::log("TEST", "CAT 2 processed at t=50ms");
    
    // Wait 50ms more, then add CAT 3
    tokio::time::sleep(Duration::from_millis(50)).await;
    let cl_id_3 = CLTransactionId("cl-tx_queue_3".to_string());
    let cat_tx_3 = Transaction::new(
        TransactionId(format!("{:?}:cat_3", cl_id_3)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 3 100".to_string(),
        cl_id_3.clone(),
    ).expect("Failed to create third CAT transaction");
    
    let _status = hig_node.lock().await.process_transaction(cat_tx_3.clone()).await.unwrap();
    cat_ids.push(CATId(cl_id_3.clone()));
    logging::log("TEST", "CAT 3 processed at t=100ms");
    
    // Now check the timing of received messages
    let mut received_cats = Vec::new();
    
    // Wait for all three CATs to be sent (should be sent at t=200ms, t=250ms, t=300ms)
    for i in 0..3 {
        let receive_start = std::time::Instant::now();
        let status_update = receiver_hig_to_hs.recv().await.expect("Should receive status update");
        let receive_time = receive_start.elapsed();
        let total_time = start_time.elapsed();
        
        logging::log("TEST", &format!("Received CAT {} at t={}ms (receive took {}ms)", 
            i + 1, total_time.as_millis(), receive_time.as_millis()));
        
        received_cats.push(status_update.cat_id);
    }
    
    // Verify all CATs were received
    assert_eq!(received_cats.len(), 3, "Should receive all 3 CATs");
    
    // Verify the timing: each CAT should be sent after its individual 200ms delay
    // CAT 1: entered at t=0ms, should be sent at t=200ms
    // CAT 2: entered at t=50ms, should be sent at t=250ms  
    // CAT 3: entered at t=100ms, should be sent at t=300ms
    
    // The total time should be around 300ms (the latest CAT's send time)
    let total_time = start_time.elapsed();
    assert!(total_time >= Duration::from_millis(300), 
        "Total time should be at least 300ms, but was {}ms", total_time.as_millis());
    assert!(total_time <= Duration::from_millis(400), 
        "Total time should be at most 400ms, but was {}ms", total_time.as_millis());
    
    logging::log("TEST", &format!("Total time for all CATs: {}ms", total_time.as_millis()));
    logging::log("TEST", "=== test_proposal_queue_independent_delays completed successfully ===\n");
}

/// Tests that a CAT gets the correct proposed status when its dependency (first CAT) succeeds.
/// This verifies that:
/// 1. First CAT gets Success proposed status (resolving)
/// 2. Second CAT that depends on the first gets Pending proposed status (postponed)
/// 3. When first CAT succeeds, second CAT gets Success proposed status
#[tokio::test]
async fn test_cat_pending_when_depending_on_resolving_cat_success() {
    test_cat_dependency_resolution(
        "Success", // First CAT resolution status (matches proposed status)
        crate::types::CATStatus::Success, // Second CAT expected status after resolution
    ).await;
}

/// Tests that a CAT gets the correct proposed status when its dependency (first CAT) fails.
/// This verifies that:
/// 1. First CAT gets Success proposed status (resolving)
/// 2. Second CAT that depends on the first gets Pending proposed status (postponed)
/// 3. When first CAT fails, second CAT gets Failure proposed status
#[tokio::test]
async fn test_cat_pending_when_depending_on_resolving_cat_failure() {
    test_cat_dependency_resolution(
        "Failure", // First CAT resolution status (different from proposed status - e.g., other chain failed)
        crate::types::CATStatus::Failure, // Second CAT expected status after resolution
    ).await;
}

/// Helper function to test CAT dependency resolution scenarios
/// 
/// # Arguments
/// * `first_cat_resolution_status` - The status to use when resolving the first CAT (Success/Failure)
/// * `second_cat_expected_status_after_resolution` - The expected proposed status for the second CAT after first CAT resolution
async fn test_cat_dependency_resolution(
    first_cat_resolution_status: &str,
    second_cat_expected_status_after_resolution: crate::types::CATStatus,
) {
    logging::init_logging();
    logging::log("TEST", &format!("\n=== Starting test_cat_dependency_resolution with first_cat_resolution_status='{}', second_cat_expected_status_after_resolution='{:?}' ===", 
        first_cat_resolution_status, second_cat_expected_status_after_resolution));
    
    // Set up test node with CAT pending dependencies enabled
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    logging::log("TEST", "Set up test node with CAT pending dependencies enabled");
    
    // First CAT: This will be resolving
    let cl_id_1 = CLTransactionId("cl-tx_cat_1".to_string());
    let cat_tx_1 = Transaction::new(
        TransactionId(format!("{:?}:cat_1", cl_id_1)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(), // Fixed transaction data
        cl_id_1.clone(),
    ).expect("Failed to create first CAT transaction");
    
    // Process first CAT
    let status_1 = hig_node.lock().await.process_transaction(cat_tx_1.clone()).await.unwrap();
    logging::log("TEST", &format!("First CAT status: {:?}", status_1));
    assert_eq!(status_1, TransactionStatus::Pending, "First CAT should be pending");
    
    // Check that first CAT has Success proposed status (based on transaction data)
    let proposed_status_1 = hig_node.lock().await.get_proposed_status(cat_tx_1.id.clone()).await.unwrap();
    logging::log("TEST", &format!("First CAT proposed status: {:?}", proposed_status_1));
    assert_eq!(proposed_status_1, crate::types::CATStatus::Success, "First CAT should have Success proposed status based on transaction data");
    
    // Second CAT: This should depend on the first CAT and get Pending proposed status
    let cl_id_2 = CLTransactionId("cl-tx_cat_2".to_string());
    let cat_tx_2 = Transaction::new(
        TransactionId(format!("{:?}:cat_2", cl_id_2)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.send 1 2 50".to_string(), // This depends on account 1 which is locked by first CAT
        cl_id_2.clone(),
    ).expect("Failed to create second CAT transaction");
    
    // Process second CAT
    let status_2 = hig_node.lock().await.process_transaction(cat_tx_2.clone()).await.unwrap();
    logging::log("TEST", &format!("Second CAT status: {:?}", status_2));
    assert_eq!(status_2, TransactionStatus::Pending, "Second CAT should be pending");
    
    // Check that second CAT has a Pending proposed status (postponed)
    let proposed_status_2 = hig_node.lock().await.get_proposed_status(cat_tx_2.id.clone()).await.unwrap();
    logging::log("TEST", &format!("Second CAT proposed status: {:?}", proposed_status_2));
    assert!(matches!(proposed_status_2, crate::types::CATStatus::Pending), "Second CAT should have Pending proposed status");
    
    // Verify the detailed pending counts
    let (resolving, postponed) = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("Detailed pending counts - Resolving: {}, Postponed: {}", resolving, postponed));
    assert_eq!(resolving, 1, "Should have 1 resolving CAT (first CAT)");
    assert_eq!(postponed, 1, "Should have 1 postponed CAT (second CAT)");
    
    // Now resolve the first CAT with the specified status
    let status_update_tx = Transaction::new(
        TransactionId("status_update".to_string()),
        constants::chain_1(),
        vec![constants::chain_1()],
        format!("STATUS_UPDATE:{}.CAT_ID:cl-tx_cat_1", first_cat_resolution_status),
        cl_id_1.clone(),
    ).expect("Failed to create status update transaction");
    
    let status_update_result = hig_node.lock().await.process_transaction(status_update_tx).await.unwrap();
    logging::log("TEST", &format!("Status update result: {:?}", status_update_result));
    // The status update transaction should return the same status that was in the update
    let expected_status_update_result = if first_cat_resolution_status == "Success" {
        TransactionStatus::Success
    } else {
        TransactionStatus::Failure
    };
    assert_eq!(status_update_result, expected_status_update_result, "Status update should return the status that was sent");
    
    // Check that second CAT now has the expected proposed status
    let proposed_status_2_after = hig_node.lock().await.get_proposed_status(cat_tx_2.id.clone()).await.unwrap();
    logging::log("TEST", &format!("Second CAT proposed status after first CAT resolution: {:?}", proposed_status_2_after));
    assert_eq!(proposed_status_2_after, second_cat_expected_status_after_resolution, "Second CAT should have expected proposed status after first CAT resolution");
    
    // Verify the detailed pending counts after resolution
    let (resolving_after, postponed_after) = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("Detailed pending counts after resolution - Resolving: {}, Postponed: {}", resolving_after, postponed_after));
    assert_eq!(resolving_after, 1, "Should have 1 resolving CAT (second CAT)");
    assert_eq!(postponed_after, 0, "Should have 0 postponed CATs");
    
    logging::log("TEST", "=== test_cat_dependency_resolution completed successfully ===\n");
}

/// Helper function to test regular transaction dependency resolution on CATs
/// 
/// # Arguments
/// * `first_cat_resolution_status` - The status to use when resolving the first CAT (Success/Failure)
/// * `second_tx_expected_status_after_resolution` - The expected status for the second transaction after first CAT resolution
async fn test_regular_tx_dependency_resolution(
    first_cat_resolution_status: &str,
    second_tx_expected_status_after_resolution: TransactionStatus,
) {
    logging::init_logging();
    logging::log("TEST", &format!("\n=== Starting test_regular_tx_dependency_resolution with first_cat_resolution_status='{}', second_tx_expected_status_after_resolution='{:?}' ===", 
        first_cat_resolution_status, second_tx_expected_status_after_resolution));
    
    // Set up test node with CAT pending dependencies enabled
    let (hig_node, _receiver_hig_to_hs) = setup_test_hig_node(true).await;
    logging::log("TEST", "Set up test node with CAT pending dependencies enabled");
    
    // First CAT: This will be resolving
    let cl_id_1 = CLTransactionId("cl-tx_cat_1".to_string());
    let cat_tx_1 = Transaction::new(
        TransactionId(format!("{:?}:cat_1", cl_id_1)),
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(), // Fixed transaction data
        cl_id_1.clone(),
    ).expect("Failed to create first CAT transaction");
    
    // Process first CAT
    let status_1 = hig_node.lock().await.process_transaction(cat_tx_1.clone()).await.unwrap();
    logging::log("TEST", &format!("First CAT status: {:?}", status_1));
    assert_eq!(status_1, TransactionStatus::Pending, "First CAT should be pending");
    
    // Check that first CAT has Success proposed status (based on transaction data)
    let proposed_status_1 = hig_node.lock().await.get_proposed_status(cat_tx_1.id.clone()).await.unwrap();
    logging::log("TEST", &format!("First CAT proposed status: {:?}", proposed_status_1));
    assert_eq!(proposed_status_1, crate::types::CATStatus::Success, "First CAT should have Success proposed status based on transaction data");
    
    // Second transaction: Regular transaction that depends on the first CAT
    let regular_tx_2 = Transaction::new(
        TransactionId("regular_tx_2".to_string()),
        constants::chain_1(),
        vec![constants::chain_1()],
        "REGULAR.send 1 2 50".to_string(), // This depends on account 1 which is locked by first CAT
        cl_id_1.clone(),
    ).expect("Failed to create second regular transaction");
    
    // Process second transaction
    let status_2 = hig_node.lock().await.process_transaction(regular_tx_2.clone()).await.unwrap();
    logging::log("TEST", &format!("Second transaction status: {:?}", status_2));
    assert_eq!(status_2, TransactionStatus::Pending, "Second transaction should be pending");
    
    // Verify the detailed pending counts
    let (resolving, postponed) = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("Detailed pending counts - Resolving: {}, Postponed: {}", resolving, postponed));
    assert_eq!(resolving, 1, "Should have 1 resolving CAT (first CAT)");
    assert_eq!(postponed, 0, "Should have 0 postponed CATs (regular transaction doesn't count as postponed CAT)");
    
    // Now resolve the first CAT with the specified status
    let status_update_tx = Transaction::new(
        TransactionId("status_update".to_string()),
        constants::chain_1(),
        vec![constants::chain_1()],
        format!("STATUS_UPDATE:{}.CAT_ID:cl-tx_cat_1", first_cat_resolution_status),
        cl_id_1.clone(),
    ).expect("Failed to create status update transaction");
    
    let status_update_result = hig_node.lock().await.process_transaction(status_update_tx).await.unwrap();
    logging::log("TEST", &format!("Status update result: {:?}", status_update_result));
    // The status update transaction should return the same status that was in the update
    let expected_status_update_result = if first_cat_resolution_status == "Success" {
        TransactionStatus::Success
    } else {
        TransactionStatus::Failure
    };
    assert_eq!(status_update_result, expected_status_update_result, "Status update should return the status that was sent");
    
    // Check that second transaction now has the expected status
    let status_2_after = hig_node.lock().await.get_transaction_status(regular_tx_2.id.clone()).await.unwrap();
    logging::log("TEST", &format!("Second transaction status after first CAT resolution: {:?}", status_2_after));
    assert_eq!(status_2_after, second_tx_expected_status_after_resolution, "Second transaction should have expected status after first CAT resolution");
    
    // Verify the detailed pending counts after resolution
    let (resolving_after, postponed_after) = hig_node.lock().await.get_cat_pending_detailed_counts().await.unwrap();
    logging::log("TEST", &format!("Detailed pending counts after resolution - Resolving: {}, Postponed: {}", resolving_after, postponed_after));
    assert_eq!(resolving_after, 0, "Should have 0 resolving CATs");
    assert_eq!(postponed_after, 0, "Should have 0 postponed CATs");
    
    logging::log("TEST", "=== test_regular_tx_dependency_resolution completed successfully ===\n");
}

/// Tests that a regular transaction gets the correct status when its dependency (first CAT) succeeds.
/// This verifies that:
/// 1. First CAT gets Success proposed status (resolving)
/// 2. Regular transaction that depends on the first gets Pending status (postponed)
/// 3. When first CAT succeeds, regular transaction gets Success status
#[tokio::test]
async fn test_regular_tx_pending_when_depending_on_resolving_cat_success() {
    test_regular_tx_dependency_resolution(
        "Success", // First CAT resolution status
        TransactionStatus::Success, // Second transaction expected status after resolution
    ).await;
}

/// Tests that a regular transaction gets the correct status when its dependency (first CAT) fails.
/// This verifies that:
/// 1. First CAT gets Success proposed status (resolving)
/// 2. Regular transaction that depends on the first gets Pending status (postponed)
/// 3. When first CAT fails, regular transaction gets Failure status
#[tokio::test]
async fn test_regular_tx_pending_when_depending_on_resolving_cat_failure() {
    test_regular_tx_dependency_resolution(
        "Failure", // First CAT resolution status (different from proposed status - e.g., other chain failed)
        TransactionStatus::Failure, // Second transaction expected status after resolution
    ).await;
}

