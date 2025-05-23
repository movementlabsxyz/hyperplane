use crate::{
    types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited, SubBlock, ChainId, CATId},
    hyper_ig::{HyperIG, node::HyperIGNode},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

/// Helper function to set up a test HIG node
async fn setup_test_hig_node() -> Arc<Mutex<HyperIGNode>> {
    let (_sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel(100);
    let (sender_hig_to_hs, mut receiver_hig_to_hs) = mpsc::channel(100);
    
    // Spawn a task to keep the receiver alive
    tokio::spawn(async move {
        while let Some(_) = receiver_hig_to_hs.recv().await {
            // Just consume the messages to keep the channel alive
        }
    });
    
    let hig_node = HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs, ChainId("chain-1".to_string()));
    Arc::new(Mutex::new(hig_node))
}

/// Helper function: Tests regular non-dependent transaction path in HyperIG
/// - Status verification
/// - Status persistence
async fn run_test_regular_transaction_status(expected_status: TransactionStatus) {
    println!("\n=== Starting regular non-dependent transaction test with status {:?}===", expected_status);
    
    println!("[TEST]   Setting up test nodes...");
    let hig_node = setup_test_hig_node().await;
    println!("[TEST]   Test nodes setup complete");

    let tx_id = "test-tx";
    println!("\n[TEST]   Processing transaction: {}", tx_id);
    let tx = Transaction::new(
        TransactionId(tx_id.to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        format!("REGULAR.SIMULATION:{:?}", expected_status),
    ).expect("Failed to create transaction");
    
    // Process transaction and verify initial status
    let status = hig_node.lock().await.process_transaction(tx.clone())
        .await
        .expect("Failed to process transaction");
    println!("[TEST]   Transaction status: {:?}", status);
    assert_eq!(status, expected_status, "Transaction should have status {:?}", expected_status);
    
    // Verify status persistence
    let get_status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert_eq!(get_status, expected_status, "Retrieved status should be {:?}", expected_status);
    println!("[TEST]   Verified status persistence");
    
    println!("=== Test completed successfully ===\n");
}

/// Tests regular non-dependent transaction success path in HyperIG:
#[tokio::test]
async fn test_regular_transaction_success() {
    run_test_regular_transaction_status(TransactionStatus::Success).await;
}

/// Tests regular non-dependent transaction success path in HyperIG:
#[tokio::test]
async fn test_regular_transaction_failure() {
    run_test_regular_transaction_status(TransactionStatus::Failure).await;
}

/// Tests regular transaction pending path in HyperIG:
/// - Regular transaction that depends on a CAT transaction
/// - Pending status verification (stays pending until CAT is resolved)
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_regular_transaction_pending() {
    println!("\n=== Starting test_regular_transaction_pending ===");
    
    println!("[TEST]   Setting up test nodes...");
    let hig_node = setup_test_hig_node().await;
    println!("[TEST]   Test nodes setup complete");
    
    // Create a regular transaction that depends on a CAT transaction
    println!("[TEST]   Creating dependent transaction...");
    let tx = Transaction::new(
        TransactionId("REGULAR.SIMULATION:Success".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "DEPENDENT.SIMULATION:Success.CAT_ID:test-cat-tx".to_string(),
    ).expect("Failed to create transaction");
    println!("[TEST]   Transaction created with tx-id='{}'", tx.id);
    
    // Execute the transaction
    println!("[TEST]   Executing transaction...");
    let status = hig_node.lock().await.process_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    println!("[TEST]   Transaction status: {:?}", status);
    
    // Verify it stays pending (transactions depending on unresolved CATs stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    println!("[TEST]   Verified transaction is pending");
    
    // Verify we can retrieve the same status
    println!("[TEST]   Verifying transaction status persistence...");
    let retrieved_status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    println!("[TEST]   Retrieved status: {:?}", retrieved_status);
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    println!("[TEST]   Verified retrieved status is pending");
    
    // Verify it's in the pending transactions list
    println!("[TEST]   Verifying pending transactions list...");
    let pending = hig_node.lock().await.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    println!("[TEST]   Pending transactions: {:?}", pending);
    assert!(pending.contains(&tx.id));
    println!("[TEST]   Verified transaction is in pending list");
    
    println!("=== Test completed successfully ===\n");
}

/// Helper function to test sending a CAT status proposal
async fn run_process_and_send_cat(expected_status: CATStatusLimited) {    
    println!("[TEST]   Setting up test nodes...");
    let hig_node = setup_test_hig_node().await;
    println!("[TEST]   Test nodes setup complete");
    
    // Create necessary parts of a CAT transaction
    let cat_id = CATId("test-cat-tx".to_string());
    let tx_chain_1 = Transaction::new(
        TransactionId("tx_chain_1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())],
        format!("CAT.SIMULATION:{:?}.CAT_ID:{}", expected_status, cat_id.0),
    ).expect("Failed to create transaction");

    // Execute the transaction
    println!("[TEST]   Executing chain-level transaction of a CLCAT transaction...");
    let status = hig_node.lock().await.process_transaction(tx_chain_1.clone())
        .await
        .expect("Failed to execute transaction");
    println!("[TEST]   Transaction status: {:?}", status);
    
    // Verify it's pending
    assert!(matches!(status, TransactionStatus::Pending));
    println!("[TEST]   Verified transaction is pending");
    
    // Verify we can retrieve the same status
    println!("[TEST]   Verifying transaction status...");
    let retrieved_status = hig_node.lock().await.get_transaction_status(tx_chain_1.id.clone())
        .await
        .expect("Failed to get transaction status");
    println!("[TEST]   Retrieved status: {:?}", retrieved_status);
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    println!("[TEST]   Verified retrieved status is pending");
    
    // Verify it's in the pending transactions list
    println!("[TEST]   Verifying pending transactions list...");
    let pending = hig_node.lock().await.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    println!("[TEST]   Pending transactions: {:?}", pending);
    assert!(pending.contains(&tx_chain_1.id));
    println!("[TEST]   Verified transaction is in pending list");
    
    // Verify the proposed status
    println!("[TEST]   Verifying proposed status...");
    let proposed_status = hig_node.lock().await.get_proposed_status(tx_chain_1.id.clone())
        .await
        .expect("Failed to get proposed status");
    println!("[TEST]   Proposed status: {:?}", proposed_status);
    assert_eq!(proposed_status, expected_status);
    println!("[TEST]   Verified proposed status is {:?}", expected_status);
    
    // Send the status proposal to HS
    println!("[TEST]   Sending status proposal to HS...");
    // we only have one chain for now, so we create a vector with one element
    let chain_id = vec![ChainId("chain-1".to_string())];
    hig_node.lock().await.send_cat_status_proposal(cat_id.clone(), expected_status, chain_id)
        .await
        .expect("Failed to send status proposal");
    println!("[TEST]   Status proposal sent to HS");
    
    println!("=== Test completed successfully ===\n");
}

/// Tests CAT transaction success proposal path in HyperIG
#[tokio::test]
#[allow(unused_variables)]
async fn test_cat_process_and_send_success() {
    println!("\n=== Starting test_cat_process_and_send_success ===");
    run_process_and_send_cat(CATStatusLimited::Success).await;
}

/// Tests CAT transaction failure proposal path in HyperIG
#[tokio::test]
#[allow(unused_variables)]
async fn test_cat_process_and_send_failure() {
    println!("\n=== Starting test_cat_process_and_send_failure ===");
    run_process_and_send_cat(CATStatusLimited::Failure).await;
}

/// Tests get pending transactions functionality:
/// - Get pending transactions when none exist
/// - Get pending transactions after adding some
#[tokio::test]
async fn test_get_pending_transactions() {
    println!("\n=== Starting test_get_pending_transactions ===");
    
    println!("[TEST]   Setting up test nodes...");
    let hig_node = setup_test_hig_node().await;
    println!("[TEST]   Test nodes setup complete");

    // Get pending transactions when none exist
    println!("[TEST]   Checking pending transactions (empty)...");
    let pending = hig_node.lock().await.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    println!("[TEST]   Pending transactions: {:?}", pending);
    assert!(pending.is_empty());

    // Create and execute a dependent transaction
    println!("[TEST]   Creating dependent transaction...");
    let tx = Transaction::new(
        TransactionId("pending-tx".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "DEPENDENT.SIMULATION:Success.CAT_ID:test-cat-tx".to_string(),
    ).expect("Failed to create transaction");
    println!("[TEST]   Executing transaction...");
    hig_node.lock().await.process_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");

    // Get pending transactions after adding one
    println!("[TEST]   Checking pending transactions (with one)...");
    let pending = hig_node.lock().await.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    println!("[TEST]   Pending transactions: {:?}", pending);
    assert!(pending.contains(&tx.id));
    
    println!("=== Test completed successfully ===\n");
}

/// Tests that a subblock with a wrong chain ID should not happen
/// - Only the subblock with the correct chain ID should be received.
#[tokio::test]
async fn test_wrong_chain_subblock() {
    // Create channels
    let (_sender_cl_to_hig, receiver_cl_to_hig) = tokio::sync::mpsc::channel(100);
    let (sender_hig_to_hs, _receiver_hig_to_hs) = tokio::sync::mpsc::channel(100);

    // Create HIG node
    let hig_node = Arc::new(Mutex::new(HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs, ChainId("chain-1".to_string()))));

    // Start the node
    HyperIGNode::start(hig_node.clone()).await;

    // Create a subblock with a different chain ID
    let wrong_chain_subblock = SubBlock {
        block_height: 1,
        chain_id: ChainId("wrong-chain".to_string()),
        transactions: vec![Transaction {
            id: TransactionId("test-tx".to_string()),
            target_chain_id: ChainId("wrong-chain".to_string()),
            data: "REGULAR.SIMULATION:Success".to_string(),
            constituent_chains: vec![],
        }],
    };

    // process the subblock and expect the error WrongChainId
    let result = hig_node.lock().await.process_subblock(wrong_chain_subblock).await;
    assert!(result.is_err(), "Expected error when receiving subblock from wrong chain");
    
    println!("=== Test completed successfully ===\n");
}



