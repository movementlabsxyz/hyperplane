use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited},
    hyper_ig::HyperIG,
};
use crate::common::testnodes;
use tokio::time::Duration;
/// Tests normal transaction success path in HyperIG:
/// - Non-dependent transaction execution
/// - Success status verification
/// - Status persistence
#[tokio::test]
async fn test_normal_transaction_success() {
    // use testnodes from common
    let (_, _, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;
    
    // Create a normal transaction with non-dependent data
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        data: "any data".to_string(),
    };
    
    // Execute the transaction
    let status = hig_node.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify it was successful (normal transactions with non-dependent data are successful)
    assert!(matches!(status, TransactionStatus::Success));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig_node.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Success));
}

/// Tests normal transaction pending path in HyperIG:
/// - Dependent transaction execution
/// - Pending status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_normal_transaction_pending() {
    // use testnodes from common
    let (_, _, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;
    
    // Create a normal transaction with dependent data
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        data: "DEPENDENT".to_string(),
    };
    
    // Execute the transaction
    let status = hig_node.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify it stays pending (normal transactions with dependent data stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig_node.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig_node.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
}

/// Tests CAT transaction success-proposal path in HyperIG (destined for the Hyper Scheduler):
/// - CAT transaction with success data
/// - Pending status verification
/// - Success proposed status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_cat_success_proposal() {
    // use testnodes from common
    let (_, _, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;
    
    // Create a CAT transaction with success data
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    
    // Execute the transaction
    let status = hig_node.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify status is pending (CAT transactions always stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig_node.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig_node.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
    
    // Verify proposed status is Success
    let proposed_status = hig_node.get_proposed_status(tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    assert!(matches!(proposed_status, CATStatusLimited::Success));
}

/// Tests CAT transaction failure-proposal path in HyperIG (destined for the Hyper Scheduler):
/// - CAT transaction with failure data
/// - Pending status verification
/// - Failure proposed status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_cat_failure_proposal() {
    // use testnodes from common
    let (_, _, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;
    
    // Create a CAT transaction with non-success data
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.FAILURE".to_string(),
    };
    
    // Execute the transaction
    let status = hig_node.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify status is pending (CAT transactions always stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig_node.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig_node.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
    
    // Verify proposed status is Failure
    let proposed_status = hig_node.get_proposed_status(tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    assert!(matches!(proposed_status, CATStatusLimited::Failure));
} 

/// Test CAT transaction success-update path in HyperIG (subblock received from the Confirmation Layer):
/// - CAT transaction with success data
/// - Success update verification
/// - Success status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_cat_success_update() {
    // use testnodes from common
    let (_, _, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Create a CAT transaction with success data
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "STATUS_UPDATE.SUCCESS".to_string(),
    };
    
    // Execute the transaction
    let status = hig_node.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");

    // Verify status is success
    assert!(matches!(status, TransactionStatus::Success));

    // Verify update is successful
    let get_status = hig_node.get_transaction_status(tx.id.clone())
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
async fn test_execute_transactions() {
    // use testnodes from common
    let (_, _, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Test regular transaction
    let tx = Transaction {
        id: TransactionId("test-tx".to_string()),
        data: "test data".to_string(),
    };
    let status = hig_node.execute_transaction(tx).await.unwrap();
    assert!(matches!(status, TransactionStatus::Success));

    // Test CAT transaction
    let cat_tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    let status = hig_node.execute_transaction(cat_tx.clone()).await.unwrap();
    assert!(matches!(status, TransactionStatus::Pending));

    // Verify CAT transaction is in pending list
    let pending = hig_node.get_pending_transactions().await.unwrap();
    assert!(pending.contains(&cat_tx.id));
}

/// Test transaction status retrieval in HyperIG:
/// - Non-existent transaction status retrieval
/// - Existing transaction status retrieval
/// - Transaction status verification
#[tokio::test]
async fn test_get_transaction_status() {
    // use testnodes from common
    let (_, _, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Test non-existent transaction
    let tx_id = TransactionId("non-existent".to_string());
    let result = hig_node.get_transaction_status(tx_id).await;
    assert!(result.is_err());

    // Test existing transaction
    let tx = Transaction {
        id: TransactionId("test-tx".to_string()),
        data: "test data".to_string(),
    };
    hig_node.execute_transaction(tx.clone()).await.unwrap();
    let status = hig_node.get_transaction_status(tx.id).await.unwrap();
    assert!(matches!(status, TransactionStatus::Success));
}

/// Test pending transaction retrieval in HyperIG:
/// - No pending transactions retrieval
/// - Pending transaction retrieval
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_get_pending_transactions() {
    // use testnodes from common
    let (_, _, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Initially no pending transactions
    let pending = hig_node.get_pending_transactions().await.unwrap();
    assert!(pending.is_empty());

    // Add a pending transaction
    let tx = Transaction {
        id: TransactionId("pending-tx".to_string()),
        data: "DEPENDENT".to_string(),
    };
    hig_node.execute_transaction(tx).await.unwrap();
    let pending = hig_node.get_pending_transactions().await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0], TransactionId("pending-tx".to_string()));
}



