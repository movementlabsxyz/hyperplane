use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, CATStatusUpdate, TransactionStatusUpdate},
    hyper_ig::{HyperIG, HyperIGNode},
};

/// Tests normal transaction success path in HyperIG:
/// - Non-dependent transaction execution
/// - Success status verification
/// - Status persistence
#[tokio::test]
async fn test_normal_transaction_success() {
    let mut hig = HyperIGNode::new();
    
    // Create a normal transaction with non-dependent data
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        data: "any data".to_string(),
    };
    
    // Execute the transaction
    let status = hig.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify it was successful (normal transactions with non-dependent data are successful)
    assert!(matches!(status, TransactionStatus::Success));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig.get_transaction_status(tx.id.clone())
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
    let mut hig = HyperIGNode::new();
    
    // Create a normal transaction with dependent data
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        data: "DEPENDENT".to_string(),
    };
    
    // Execute the transaction
    let status = hig.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify it stays pending (normal transactions with dependent data stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
}

/// Tests CAT transaction success-proposal path in HyperIG:
/// - CAT transaction with success data
/// - Pending status verification
/// - Success proposed status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_cat_success_proposal() {
    let mut hig = HyperIGNode::new();
    
    // Create a CAT transaction with success data
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    
    // Execute the transaction
    let status = hig.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify status is pending (CAT transactions always stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
    
    // Verify proposed status is Success
    let proposed_status = hig.get_proposed_status(tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    assert!(matches!(proposed_status, CATStatusUpdate::Success));
}

/// Tests CAT transaction failure-proposal path in HyperIG:
/// - CAT transaction with failure data
/// - Pending status verification
/// - Failure proposed status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_cat_failure_proposal() {
    let mut hig = HyperIGNode::new();
    
    // Create a CAT transaction with non-success data
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.FAILURE".to_string(),
    };
    
    // Execute the transaction
    let status = hig.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute transaction");
    
    // Verify status is pending (CAT transactions always stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
    
    // Verify proposed status is Failure
    let proposed_status = hig.get_proposed_status(tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    assert!(matches!(proposed_status, CATStatusUpdate::Failure));
}

/// Tests status update success path in HyperIG:
/// - CAT transaction submission
/// - Status update submission
/// - Status change verification
/// - Pending list removal
#[tokio::test]
async fn test_status_update_success() {
    let mut hig = HyperIGNode::new();
    
    // First submit a CAT transaction
    let cat_id = TransactionId("cat-tx".to_string());
    let tx = Transaction {
        id: cat_id.clone(),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    
    // Execute the CAT transaction
    hig.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute CAT transaction");
    
    // Verify it's pending
    let status = hig.get_transaction_status(cat_id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Submit a success status update
    let status_update = TransactionStatusUpdate {
        transaction_id: cat_id.clone(),
        status: TransactionStatus::Success,
    };
    
    // Submit the status update
    hig.submit_cat_status_proposal(status_update)
        .await
        .expect("Failed to submit status update");
    
    // Verify the CAT transaction status was updated to Success
    let retrieved_status = hig.get_transaction_status(cat_id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Success));
    
    // Verify it's no longer in the pending list
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(!pending.contains(&cat_id));
}

/// Tests status update failure path in HyperIG:
/// - CAT transaction submission
/// - Status update submission
/// - Status change verification
/// - Pending list removal
#[tokio::test]
async fn test_status_update_failure() {
    let mut hig = HyperIGNode::new();
    
    // First submit a CAT transaction
    let cat_id = TransactionId("cat-tx".to_string());
    let tx = Transaction {
        id: cat_id.clone(),
        data: "CAT.SIMULATION.FAILURE".to_string(),
    };
    
    // Execute the CAT transaction
    hig.execute_transaction(tx.clone())
        .await
        .expect("Failed to execute CAT transaction");
    
    // Verify it's pending
    let status = hig.get_transaction_status(cat_id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Submit a failure status update
    let status_update = TransactionStatusUpdate {
        transaction_id: cat_id.clone(),
        status: TransactionStatus::Failure,
    };
    
    // Submit the status update
    hig.submit_cat_status_proposal(status_update)
        .await
        .expect("Failed to submit status update");
    
    // Verify the CAT transaction status was updated to Failure
    let retrieved_status = hig.get_transaction_status(cat_id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Failure));
    
    // Verify it's no longer in the pending list
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(!pending.contains(&cat_id));
} 