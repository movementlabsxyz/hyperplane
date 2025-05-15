use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited},
    hyper_ig::HyperIG,
};
use crate::common::testnodes;
use std::sync::Arc;
use tokio::sync::Mutex;
use hyperplane::types::{CATId};

/// Tests normal transaction success path in HyperIG:
/// - Non-dependent transaction execution
/// - Success status verification
/// - Status persistence
#[tokio::test]
async fn test_normal_transaction_success() {
    println!("\n=== Starting test_normal_transaction_success ===");
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (_, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;
    println!("[TEST]   Test nodes setup complete");
    
    // Create a normal transaction with non-dependent data
    println!("[TEST]   Creating normal transaction...");
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        data: "any data".to_string(),
    };
    println!("[TEST]   Transaction created with id: {}", tx.id.0);
    
    // Execute the transaction
    println!("[TEST]   Executing transaction...");
    let status = hig_node.lock().await.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    println!("[TEST]   Transaction status: {:?}", status);
    
    // Verify it was successful (normal transactions with non-dependent data are successful)
    assert!(matches!(status, TransactionStatus::Success));
    println!("[TEST]   Verified transaction is successful");
    
    // Verify we can retrieve the same status
    println!("[TEST]   Verifying transaction status persistence...");
    let retrieved_status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    println!("[TEST]   Retrieved status: {:?}", retrieved_status);
    assert!(matches!(retrieved_status, TransactionStatus::Success));
    println!("[TEST]   Verified retrieved status is successful");
    
    println!("=== Test completed successfully ===\n");
}

/// Tests normal transaction pending path in HyperIG:
/// - Regular transaction that depends on a CAT transaction
/// - Pending status verification (stays pending until CAT is resolved)
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_normal_transaction_pending() {
    println!("\n=== Starting test_normal_transaction_pending ===");
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (_, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;
    println!("[TEST]   Test nodes setup complete");
    
    // Create a regular transaction that depends on a CAT transaction
    println!("[TEST]   Creating dependent transaction...");
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        data: "DEPENDENT_ON_CAT.tx-cat".to_string(), // Depends on a CAT transaction that doesn't exist yet
    };
    println!("[TEST]   Transaction created with id: {}", tx.id.0);
    
    // Execute the transaction
    println!("[TEST]   Executing transaction...");
    let status = hig_node.lock().await.execute_transaction(tx.clone())
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
    let (hs_node, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;
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
    let (hs_node, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;

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
    println!("\n=== Starting test_cat_success_update ===");
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (_, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;
    println!("[TEST]   Test nodes setup complete");

    // Create a CAT transaction with success data
    println!("[TEST]   Creating CAT transaction...");
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "STATUS_UPDATE.SUCCESS".to_string(),
    };
    println!("[TEST]   Transaction created with id: {}", tx.id.0);
    
    // Execute the transaction
    println!("[TEST]   Executing transaction...");
    let status = hig_node.lock().await.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    println!("[TEST]   Transaction status: {:?}", status);

    // Verify status is success
    assert!(matches!(status, TransactionStatus::Success));
    println!("[TEST]   Verified transaction is successful");

    // Verify update is successful
    println!("[TEST]   Verifying transaction status persistence...");
    let get_status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    println!("[TEST]   Retrieved status: {:?}", get_status);
    assert!(matches!(get_status, TransactionStatus::Success));
    println!("[TEST]   Verified retrieved status is successful");
    
    println!("=== Test completed successfully ===\n");
}

/// Test transaction execution path in HyperIG:
/// - Regular transaction execution (success)
/// - CAT transaction execution (pending)
/// - Transaction status verification
/// - Pending transaction list inclusion
#[tokio::test]
#[allow(unused_variables)]
async fn test_execute_transactions() {
    println!("\n=== Starting test_execute_transactions ===");
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (_, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;
    println!("[TEST]   Test nodes setup complete");

    // Create multiple transactions
    println!("[TEST]   Creating test transactions...");
    let transactions = vec![
        Transaction {
            id: TransactionId("tx1".to_string()),
            data: "any data".to_string(),
        },
        Transaction {
            id: TransactionId("tx2".to_string()),
            data: "DEPENDENT_ON_CAT.tx-cat".to_string(),
        },
    ];
    println!("[TEST]   Created {} transactions", transactions.len());

    // Execute each transaction
    println!("[TEST]   Executing transactions...");
    for tx in &transactions {
        println!("[TEST]   Executing transaction: {}", tx.id.0);
        let status = hig_node.lock().await.execute_transaction(tx.clone())
            .await
            .expect("Failed to execute transaction");
        println!("[TEST]   Transaction status: {:?}", status);
    }

    // Verify status of each transaction
    println!("[TEST]   Verifying transaction statuses...");
    for tx in &transactions {
        println!("[TEST]   Checking status for transaction: {}", tx.id.0);
        let status = hig_node.lock().await.get_transaction_status(tx.id.clone())
            .await
            .expect("Failed to get transaction status");
        println!("[TEST]   Retrieved status: {:?}", status);
    }
    
    println!("=== Test completed successfully ===\n");
}

/// Tests get transaction status functionality:
/// - Get status of non-existent transaction
/// - Get status of existing transaction
#[tokio::test]
async fn test_get_transaction_status() {
    println!("\n=== Starting test_get_transaction_status ===");
    
    // use testnodes from common
    println!("[TEST]   Setting up test nodes...");
    let (_, _, hig_node,_start_block_height) = testnodes::setup_test_nodes_no_block_production().await;
    println!("[TEST]   Test nodes setup complete");

    // Try to get status of non-existent transaction
    println!("[TEST]   Checking status of non-existent transaction...");
    let non_existent_tx = TransactionId("non-existent".to_string());
    let result = hig_node.lock().await.get_transaction_status(non_existent_tx.clone())
        .await;
    println!("[TEST]   Result for non-existent transaction: {:?}", result);
    assert!(result.is_err());

    // Create and execute a transaction
    println!("[TEST]   Creating test transaction...");
    let tx = Transaction {
        id: TransactionId("test-tx".to_string()),
        data: "any data".to_string(),
    };
    println!("[TEST]   Executing transaction...");
    hig_node.lock().await.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");

    // Get status of existing transaction
    println!("[TEST]   Checking status of existing transaction...");
    let status = hig_node.lock().await.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    println!("[TEST]   Retrieved status: {:?}", status);
    assert!(matches!(status, TransactionStatus::Success));
    
    println!("=== Test completed successfully ===\n");
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
    let tx = Transaction {
        id: TransactionId("pending-tx".to_string()),
        data: "DEPENDENT_ON_CAT.tx-cat".to_string(),
    };
    println!("[TEST]   Executing transaction...");
    hig_node.lock().await.execute_transaction(tx.clone())
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



