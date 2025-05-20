use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, StatusLimited},
    hyper_ig::HyperIG,
};
use crate::common::testnodes;
use std::sync::Arc;
use tokio::sync::Mutex;
use hyperplane::types::{CATId, ChainId};

/// Helper function: Tests regular non-dependent transaction path in HyperIG
/// - Status verification
/// - Status persistence
async fn run_test_regular_transaction_status(expected_status: TransactionStatus) {
    println!("\n=== Starting regular non-dependent transaction test with status {:?}===", expected_status);
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (_, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;
    println!("[TEST]   Test nodes setup complete");

    let tx_id = "test-tx";
    println!("\n[TEST]   Processing transaction: {}", tx_id);
    let tx = Transaction::new(
        TransactionId(tx_id.to_string()),
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
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (_, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;
    println!("[TEST]   Test nodes setup complete");
    
    // Create a regular transaction that depends on a CAT transaction
    println!("[TEST]   Creating dependent transaction...");
    let tx = Transaction::new(
        TransactionId("REGULAR.SIMULATION:Success".to_string()),
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

/// Helper function to test CAT status proposal
async fn run_test_single_chain_cat(expected_status: StatusLimited) {
    println!("\n=== Starting test_single_chain_cat ({:?}) ===", expected_status);
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (hs_node, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;

    // Wrap hs_node in Arc<Mutex>
    println!("[TEST]   Wrapping HS node in Arc<Mutex>...");
    let _hs_node = Arc::new(Mutex::new(hs_node));
    println!("[TEST]   HS node wrapped successfully");
    
    // Create a CAT transaction
    println!("[TEST]   Creating CAT transaction...");
    let tx = Transaction::new(
        TransactionId("test-tx".to_string()),
        vec![ChainId("chain-1".to_string())],
        format!("CAT.SIMULATION:{:?}.CAT_ID:test-cat-tx", expected_status),
    ).expect("Failed to create transaction");
    println!("[TEST]   CAT transaction created with tx-id='{}' : data='{}'", tx.id, tx.data);
    
    // Execute the transaction
    println!("[TEST]   Executing CAT transaction...");
    let status = hig_node.lock().await.process_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    println!("[TEST]   Transaction status: {:?}", status);
    
    // Verify it's pending
    assert!(matches!(status, TransactionStatus::Pending));
    println!("[TEST]   Verified transaction is pending");
    
    // Verify we can retrieve the same status
    println!("[TEST]   Verifying transaction status...");
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
    
    // Verify the proposed status
    println!("[TEST]   Verifying proposed status...");
    let proposed_status = hig_node.lock().await.get_proposed_status(tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    println!("[TEST]   Proposed status: {:?}", proposed_status);
    assert_eq!(proposed_status, expected_status);
    println!("[TEST]   Verified proposed status is {:?}", expected_status);
    
    // Send the status proposal to HS
    println!("[TEST]   Sending status proposal to HS...");
    // we only have one chain for now, so we create a vector with one element
    let chain_id = vec![ChainId("chain-1".to_string())];
    hig_node.lock().await.send_cat_status_proposal(CATId(tx.id.0.clone()), expected_status, chain_id)
        .await
        .expect("Failed to send status proposal");
    println!("[TEST]   Status proposal sent to HS");
    
    println!("=== Test completed successfully ===\n");
}

/// Tests CAT transaction success proposal path in HyperIG
#[tokio::test]
#[allow(unused_variables)]
async fn test_cat_success_proposal() {
    run_test_single_chain_cat(StatusLimited::Success).await;
}

/// Tests CAT transaction failure proposal path in HyperIG
#[tokio::test]
#[allow(unused_variables)]
async fn test_cat_failure_proposal() {
    run_test_single_chain_cat(StatusLimited::Failure).await;
}

/// Tests get pending transactions functionality:
/// - Get pending transactions when none exist
/// - Get pending transactions after adding some
#[tokio::test]
async fn test_get_pending_transactions() {
    println!("\n=== Starting test_get_pending_transactions ===");
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (_, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;
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



