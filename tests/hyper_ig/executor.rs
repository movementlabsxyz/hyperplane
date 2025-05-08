use hyperplane::{
    types::{ChainId, Transaction, TransactionId, TransactionWrapper, TransactionStatus, CATStatusProposal},
    hyper_ig::executor::{HyperIGNode, HyperIG},
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
        chain_id: ChainId("test-chain".to_string()),
        data: "any data".to_string(),
    };
    
    // Execute the transaction
    let status = hig.execute_transaction_wrapper(TransactionWrapper {
        transaction: tx.clone(),
        is_cat: false,
    })
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
        chain_id: ChainId("test-chain".to_string()),
        data: "dependent".to_string(),
    };
    
    // Execute the transaction
    let status = hig.execute_transaction_wrapper(TransactionWrapper {
        transaction: tx.clone(),
        is_cat: false,
    })
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
        chain_id: ChainId("test-chain".to_string()),
        data: "success".to_string(),
    };
    let tx_wrapper = TransactionWrapper {
        transaction: tx.clone(),
        is_cat: true,
    };
    
    // Execute the transaction
    let status = hig.execute_transaction_wrapper(tx_wrapper)
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
    assert!(matches!(proposed_status, CATStatusProposal::Success));
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
        chain_id: ChainId("test-chain".to_string()),
        data: "failure".to_string(),
    };
    let tx_wrapper = TransactionWrapper {
        transaction: tx.clone(),
        is_cat: true,
    };
    
    // Execute the transaction
    let status = hig.execute_transaction_wrapper(tx_wrapper)
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
    assert!(matches!(proposed_status, CATStatusProposal::Failure));
} 