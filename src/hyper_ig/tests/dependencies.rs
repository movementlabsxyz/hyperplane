use crate::hyper_ig::node::HyperIGNode;
use crate::types::{Transaction, TransactionId, TransactionStatus, ChainId};
use crate::utils::logging;
use crate::hyper_ig::HyperIG;
use tokio::sync::mpsc;
use std::time::Duration;

/// Helper function to set up a test HyperIG node
async fn setup_test_hig_node() -> std::sync::Arc<tokio::sync::Mutex<HyperIGNode>> {
    let (_sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel(100);
    let (sender_hig_to_hs, receiver_hig_to_hs) = mpsc::channel(100);
    let hig_node = HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs, ChainId("chain-1".to_string()));
    let hig_node = std::sync::Arc::new(tokio::sync::Mutex::new(hig_node));
    
    // Spawn a task to keep the receiver alive
    let mut receiver = receiver_hig_to_hs;
    tokio::spawn(async move {
        while let Some(_msg) = receiver.recv().await {
            // Keep receiving messages to prevent channel closure
        }
    });
    
    HyperIGNode::start(hig_node.clone()).await;
    hig_node
}

/// Tests that a transaction with a single dependency waits for the dependency to be resolved
#[tokio::test]
pub async fn test_single_dependency() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_single_dependency ===");
    
    let hig_node = setup_test_hig_node().await;

    // Create a CAT that locks key "1"
    let cat_tx = Transaction::new(
        TransactionId("cat-tx".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())],
        "CAT.credit 1 100.CAT_ID:cat-1".to_string(),
    ).expect("Failed to create transaction");

    // Create a transaction that depends on the CAT
    let dependent_tx = Transaction::new(
        TransactionId("dependent-tx".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 2 50".to_string(),
    ).expect("Failed to create transaction");
    
    // Process the CAT first
    let status = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Process the dependent transaction
    let status = hig_node.lock().await.process_transaction(dependent_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);

    // check the correctness of the dependency
    let dependencies = hig_node.lock().await.get_transaction_dependencies(dependent_tx.id.clone()).await.unwrap();
    logging::log("TEST", &format!("Dependencies: {:?}", dependencies));
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0], cat_tx.id.clone());

    // Resolve the CAT
    let status_update = Transaction::new(
        TransactionId("status-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "STATUS_UPDATE:Success.CAT_ID:cat-1".to_string(),
    ).expect("Failed to create transaction");
    hig_node.lock().await.process_transaction(status_update).await.unwrap();

    // check the correctness of the dependency
    let dependencies = hig_node.lock().await.get_transaction_dependencies(dependent_tx.id.clone()).await.unwrap();
    logging::log("TEST", &format!("Dependencies: {:?}", dependencies));
    assert_eq!(dependencies.len(), 0);
    
    // Verify the dependent transaction is now successful
    let status = hig_node.lock().await.get_transaction_status(dependent_tx.id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Success);
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a transaction with multiple dependencies waits for all dependencies to be resolved
#[tokio::test]
pub async fn test_multiple_dependencies() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_multiple_dependencies ===");
    
    let hig_node = setup_test_hig_node().await;
    
    // First create a CAT that locks key "1"
    let cat_tx_1 = Transaction::new(
        TransactionId("cat-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())],
        "CAT.credit 1 100.CAT_ID:cat-1".to_string(),
    ).expect("Failed to create transaction");
    
    // Create another CAT that locks key "2"
    let cat_tx_2 = Transaction::new(
        TransactionId("cat-2".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())],
        "CAT.credit 2 100.CAT_ID:cat-2".to_string(),
    ).expect("Failed to create transaction");
    
    // Create a transaction that depends on both keys
    let dependent_tx = Transaction::new(
        TransactionId("dependent-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 2 50".to_string(),
    ).expect("Failed to create transaction");
    
    // Process the CATs first
    let status_1 = hig_node.lock().await.process_transaction(cat_tx_1.clone()).await.unwrap();
    let status_2 = hig_node.lock().await.process_transaction(cat_tx_2.clone()).await.unwrap();
    assert_eq!(status_1, TransactionStatus::Pending);
    assert_eq!(status_2, TransactionStatus::Pending);
    
    // Process the dependent transaction
    let status = hig_node.lock().await.process_transaction(dependent_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Verify the dependent transaction is still pending
    let status = hig_node.lock().await.get_transaction_status(dependent_tx.id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Resolve the first CAT
    let status_update_1 = Transaction::new(
        TransactionId("status-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "STATUS_UPDATE:Success.CAT_ID:cat-1".to_string(),
    ).expect("Failed to create transaction");
    hig_node.lock().await.process_transaction(status_update_1).await.unwrap();
    
    // Verify the dependent transaction is still pending (waiting for second CAT)
    let status = hig_node.lock().await.get_transaction_status(dependent_tx.id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Resolve the second CAT
    let status_update_2 = Transaction::new(
        TransactionId("status-2".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "STATUS_UPDATE:Success.CAT_ID:cat-2".to_string(),
    ).expect("Failed to create transaction");
    hig_node.lock().await.process_transaction(status_update_2).await.unwrap();
    
    // Verify the dependent transaction is now successful
    let status = hig_node.lock().await.get_transaction_status(dependent_tx.id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Success);
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a transaction with a failed dependency remains pending
#[tokio::test]
pub async fn test_failed_dependency() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_failed_dependency ===");
    
    let hig_node = setup_test_hig_node().await;
    
    // Create a CAT that locks key "1"
    let cat_tx = Transaction::new(
        TransactionId("cat-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())],
        "CAT.credit 1 100.CAT_ID:cat-1".to_string(),
    ).expect("Failed to create transaction");
    
    // Create a transaction that depends on the key
    let dependent_tx = Transaction::new(
        TransactionId("dependent-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 2 50".to_string(),
    ).expect("Failed to create transaction");
    
    // Process the CAT first
    let status = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Process the dependent transaction
    let status = hig_node.lock().await.process_transaction(dependent_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Resolve the CAT with Failure
    let status_update = Transaction::new(
        TransactionId("status-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "STATUS_UPDATE:Failure.CAT_ID:cat-1".to_string(),
    ).expect("Failed to create transaction");
    hig_node.lock().await.process_transaction(status_update).await.unwrap();
    
    // Verify the dependent transaction is still pending
    let status = hig_node.lock().await.get_transaction_status(dependent_tx.id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that multiple transactions waiting on the same key are processed in order
#[tokio::test]
pub async fn test_multiple_transactions_same_key() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_multiple_transactions_same_key ===");
    
    let hig_node = setup_test_hig_node().await;
    
    // Create a CAT that locks key "1"
    let cat_tx = Transaction::new(
        TransactionId("cat-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())],
        "CAT.credit 1 100.CAT_ID:cat-1".to_string(),
    ).expect("Failed to create transaction");
    
    // Create multiple transactions that depend on the same key
    let dependent_tx_1 = Transaction::new(
        TransactionId("dependent-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 2 20".to_string(),
    ).expect("Failed to create transaction");
    
    let dependent_tx_2 = Transaction::new(
        TransactionId("dependent-2".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "REGULAR.send 1 2 30".to_string(),
    ).expect("Failed to create transaction");
    
    // Process the CAT first
    let status = hig_node.lock().await.process_transaction(cat_tx.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Process the dependent transactions
    let status_1 = hig_node.lock().await.process_transaction(dependent_tx_1.clone()).await.unwrap();
    let status_2 = hig_node.lock().await.process_transaction(dependent_tx_2.clone()).await.unwrap();
    assert_eq!(status_1, TransactionStatus::Pending);
    assert_eq!(status_2, TransactionStatus::Pending);
    
    // Resolve the CAT
    let status_update = Transaction::new(
        TransactionId("status-1".to_string()),
        ChainId("chain-1".to_string()),
        vec![ChainId("chain-1".to_string())],
        "STATUS_UPDATE:Success.CAT_ID:cat-1".to_string(),
    ).expect("Failed to create transaction");
    hig_node.lock().await.process_transaction(status_update).await.unwrap();
    
    // Wait a bit for transactions to be processed
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify both dependent transactions are now successful
    let status_1 = hig_node.lock().await.get_transaction_status(dependent_tx_1.id.clone()).await.unwrap();
    let status_2 = hig_node.lock().await.get_transaction_status(dependent_tx_2.id.clone()).await.unwrap();
    assert_eq!(status_1, TransactionStatus::Success);
    assert_eq!(status_2, TransactionStatus::Success);
    
    logging::log("TEST", "=== Test completed successfully ===\n");
} 