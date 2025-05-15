use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited},
    hyper_ig::HyperIG,
};
use crate::common::testnodes;
use tokio::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use hyperplane::types::{CATId};

/// Tests normal transaction success path in HyperIG:
/// - Non-dependent transaction execution
/// - Success status verification
/// - Status persistence
#[tokio::test]
async fn test_normal_transaction_success() {
    // use testnodes from common
    let (_, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;
    
    // Create a normal transaction with non-dependent data
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        data: "any data".to_string(),
    };
    
    // Execute the transaction
    let status = hig_node.lock().await.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify it was successful (normal transactions with non-dependent data are successful)
    assert!(matches!(status, TransactionStatus::Success));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Success));
}

/// Tests normal transaction pending path in HyperIG:
/// - Regular transaction that depends on a CAT transaction
/// - Pending status verification (stays pending until CAT is resolved)
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_normal_transaction_pending() {
    // use testnodes from common
    let (_, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;
    
    // Create a regular transaction that depends on a CAT transaction
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        data: "DEPENDENT_ON_CAT.tx-cat".to_string(), // Depends on a CAT transaction that doesn't exist yet
    };
    
    // Execute the transaction
    let status = hig_node.lock().await.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify it stays pending (transactions depending on unresolved CATs stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig_node.lock().await.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
}

/// Tests CAT transaction success proposal path in HyperIG:
/// - CAT transaction execution
/// - Success proposal verification
/// - Success proposal sending to Hyper Scheduler
#[tokio::test]
#[allow(unused_variables)]
async fn test_cat_success_proposal() {
    println!("\n=== Starting test_cat_success_proposal ===");
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;
    println!("[TEST]   Test nodes setup complete");

    // Wrap hs_node in Arc<Mutex>
    println!("[TEST]   Wrapping HS node in Arc<Mutex>...");
    let hs_node = Arc::new(Mutex::new(hs_node));
    println!("[TEST]   HS node wrapped successfully");
    
    // Create a CAT transaction
    println!("[TEST]   Creating CAT transaction...");
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    println!("[TEST]   CAT transaction created with id: {}", tx.id.0);
    
    // Execute the transaction
    println!("[TEST]   Executing CAT transaction...");
    let status = hig_node.lock().await.execute_transaction(tx.clone())
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
    assert!(matches!(proposed_status, CATStatusLimited::Success));
    println!("[TEST]   Verified proposed status is Success");
    
    // Send the status proposal to HS
    println!("[TEST]   Sending status proposal to HS...");
    hig_node.lock().await.send_cat_status_proposal(CATId(tx.id.0.clone()), CATStatusLimited::Success)
        .await
        .expect("Failed to send status proposal");
    println!("[TEST]   Status proposal sent to HS");
    
    println!("=== Test completed successfully ===\n");
}

/// Tests CAT transaction failure proposal path in HyperIG:
/// - CAT transaction execution
/// - Failure proposal verification
/// - Failure proposal sending to Hyper Scheduler
#[tokio::test]
#[allow(unused_variables)]
async fn test_cat_failure_proposal() {
    println!("\n=== Starting test_cat_failure_proposal ===");
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Wrap hs_node in Arc<Mutex>
    println!("[TEST]   Wrapping HS node in Arc<Mutex>...");
    let hs_node = Arc::new(Mutex::new(hs_node));
    println!("[TEST]   HS node wrapped successfully");
    
    // Create a CAT transaction
    println!("[TEST]   Creating CAT transaction...");
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.FAILURE".to_string(),
    };
    
    // Execute the transaction
    println!("[TEST]   Executing CAT transaction...");
    let status = hig_node.lock().await.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    println!("[TEST]   Transaction status: {:?}", status);
    
    // Verify it's pending
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    println!("[TEST]   Verifying transaction status...");
    let retrieved_status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    println!("[TEST]   Retrieved status: {:?}", retrieved_status);
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    println!("[TEST]   Verifying pending transactions list...");
    let pending = hig_node.lock().await.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    println!("[TEST]   Pending transactions: {:?}", pending);
    assert!(pending.contains(&tx.id));
    
    // Verify the proposed status
    println!("[TEST]   Verifying proposed status...");
    let proposed_status = hig_node.lock().await.get_proposed_status(tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    println!("[TEST]   Proposed status: {:?}", proposed_status);
    assert!(matches!(proposed_status, CATStatusLimited::Failure));
    
    // Send the status proposal to HS
    println!("[TEST]   Sending status proposal to HS...");
    hig_node.lock().await.send_cat_status_proposal(CATId(tx.id.0.clone()), CATStatusLimited::Failure)
        .await
        .expect("Failed to send status proposal");
    println!("[TEST]   Status proposal sent to HS");
    
    println!("=== Test completed successfully ===\n");
} 

/// Test CAT transaction success-update path in HyperIG (subblock received from the Confirmation Layer):
/// - CAT transaction with success data
/// - Success update verification
/// - Success status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_cat_success_update() {
    // use testnodes from common
    let (_, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Create a CAT transaction with success data
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "STATUS_UPDATE.SUCCESS".to_string(),
    };
    
    // Execute the transaction
    let status = hig_node.lock().await.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");

    // Verify status is success
    assert!(matches!(status, TransactionStatus::Success));

    // Verify update is successful
    let get_status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(get_status, TransactionStatus::Success));
}

/// Test transaction execution path in HyperIG:
/// - Regular transaction execution (success)
/// - CAT transaction execution (pending)
/// - Transaction status verification
/// - Pending transaction list inclusion
#[tokio::test]
#[allow(unused_variables)]
async fn test_execute_transactions() {
    // use testnodes from common
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));

    // Test regular transaction
    let tx = Transaction {
        id: TransactionId("test-tx".to_string()),
        data: "test data".to_string(),
    };
    let status = hig_node.lock().await.execute_transaction(tx).await.unwrap();
    assert!(matches!(status, TransactionStatus::Success));

    // Test CAT transaction
    let cat_tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    let status = hig_node.lock().await.execute_transaction(cat_tx.clone()).await.unwrap();
    assert!(matches!(status, TransactionStatus::Pending));

    // Verify CAT transaction is in pending list
    let pending = hig_node.lock().await.get_pending_transactions().await.unwrap();
    assert!(pending.contains(&cat_tx.id));
}

/// Test transaction status retrieval in HyperIG:
/// - Non-existent transaction status retrieval
/// - Existing transaction status retrieval
/// - Transaction status verification
#[tokio::test]
async fn test_get_transaction_status() {
    // use testnodes from common
    let (_, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Test non-existent transaction
    let tx_id = TransactionId("non-existent".to_string());
    let result = hig_node.lock().await.get_transaction_status(tx_id).await;
    assert!(result.is_err());

    // Test existing transaction
    let tx = Transaction {
        id: TransactionId("test-tx".to_string()),
        data: "test data".to_string(),
    };
    hig_node.lock().await.execute_transaction(tx.clone()).await.unwrap();
    let status = hig_node.lock().await.get_transaction_status(tx.id).await.unwrap();
    assert!(matches!(status, TransactionStatus::Success));
}

/// Test pending transaction retrieval in HyperIG:
/// - No pending transactions retrieval
/// - Pending transaction retrieval
/// - Pending transaction list inclusion
#[tokio::test]
#[allow(unused_variables)]
async fn test_get_pending_transactions() {
    // use testnodes from common
    let (hs_node, _, hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Wrap hs_node in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));

    // Initially no pending transactions
    let pending = hig_node.lock().await.get_pending_transactions().await.unwrap();
    assert!(pending.is_empty());

    // Add a pending transaction
    let tx = Transaction {
        id: TransactionId("pending-tx".to_string()),
        data: "DEPENDENT_ON_CAT".to_string(), // Make it dependent to ensure it stays pending
    };
    hig_node.lock().await.execute_transaction(tx.clone()).await.unwrap();
    let pending = hig_node.lock().await.get_pending_transactions().await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0], tx.id);
}



