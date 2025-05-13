use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, ChainId, CLTransaction},
    hyper_ig::HyperIG,
    confirmation_layer::ConfirmationLayer,
};
use tokio::time::Duration;
use crate::common::testnodes;

/// Tests that a subblock with new transactions is properly processed by the Hyper IG:
/// - The Confirmation Layer sends a subblock to the Hyper IG
/// - The Hyper IG processes the transactions in the subblock
/// - Verify the transaction statuses are correctly updated
#[tokio::test]
async fn test_process_subblock() {
    // Initialize components
    let (_, mut cl_node, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Register chain
    let chain_id = ChainId("test-chain".to_string());
    cl_node.register_chain(chain_id.clone()).await.expect("Failed to register chain");

    // Submit transaction to CL
    let tx = Transaction {
        id: TransactionId("test-tx".to_string()),
        data: "test data".to_string(),
    };
    cl_node.submit_transaction(CLTransaction {
        id: tx.id.clone(),
        data: tx.data.clone(),
        chain_id: chain_id.clone(),
    })
    .await
    .expect("Failed to submit transaction");

    // Wait for block production
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");

    // Get subblock
    let subblock = cl_node.get_subblock(chain_id, current_block)
        .await
        .expect("Failed to get subblock");

    // Process subblock in HIG
    hig_node.process_subblock(subblock)
        .await
        .expect("Failed to process subblock");

    // Verify transaction status
    let status = hig_node.get_transaction_status(tx.id).await.unwrap();
    assert!(matches!(status, TransactionStatus::Success));
}

/// Tests that a subblock with a CAT transaction is properly processed by the Hyper IG:
/// - The Confirmation Layer sends a subblock with a CAT transaction to the Hyper IG
/// - The Hyper IG processes the CAT transaction
/// - Verify the CAT transaction status is correctly updated
#[tokio::test]
async fn test_process_cat_subblock() {
    // Initialize components
    let (_, mut cl_node, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Register chain
    let chain_id = ChainId("test-chain".to_string());
    cl_node.register_chain(chain_id.clone()).await.expect("Failed to register chain");

    // Submit CAT transaction to CL
    let tx = Transaction {
        id: TransactionId("test-cat".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    cl_node.submit_transaction(CLTransaction {
        id: tx.id.clone(),
        data: tx.data.clone(),
        chain_id: chain_id.clone(),
    })
    .await
    .expect("Failed to submit transaction");

    // Wait for block production
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");

    // Get subblock
    let subblock = cl_node.get_subblock(chain_id, current_block)
        .await
        .expect("Failed to get subblock");

    // Process subblock in HIG
    hig_node.process_subblock(subblock)
        .await
        .expect("Failed to process subblock");

    // Verify transaction status
    let status = hig_node.get_transaction_status(tx.id).await.unwrap();
    assert!(matches!(status, TransactionStatus::Pending));
}

/// Tests that multiple subblocks with new transactions are properly processed by the Hyper IG:
/// - The Confirmation Layer sends multiple subblocks to the Hyper IG
/// - The Hyper IG processes the transactions in each subblock
/// - Verify the transaction statuses are correctly updated for each subblock
#[tokio::test]
async fn test_process_multiple_subblocks_new_transactions() {
    // Create HIG and CL nodes
    let (_, mut cl_node, mut hig_node) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Register a test chain
    let chain_id = ChainId("test-chain".to_string());
    cl_node.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Create test transactions for first subblock
    let tx1 = Transaction {
        id: TransactionId("tx1".to_string()),
        data: "SUCCESS".to_string(),
    };
    // submit the transactions to the CL
    cl_node.submit_transaction(CLTransaction {
        id: tx1.id.clone(),
        data: tx1.data.clone(),
        chain_id: chain_id.clone(),
    }).await.expect("Failed to submit transaction");

    // Wait for block production and get the current block
    tokio::time::sleep(Duration::from_millis(200)).await;
    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");
    println!("Current block number after tx1: {}", current_block);

    // Look for tx1 in all blocks up to the current one
    let mut found_subblock1 = None;
    for block_num in 0..=current_block {
        if let Ok(subblock) = cl_node.get_subblock(chain_id.clone(), block_num).await {
            if subblock.transactions.iter().any(|tx| tx.id == tx1.id) {
                found_subblock1 = Some(subblock);
                break;
            }
        }
    }
    let subblock1 = found_subblock1.expect("Failed to find subblock containing tx1");

    // Process the first subblock
    hig_node.process_subblock(subblock1)
        .await
        .expect("Failed to process first subblock");

    // Verify tx1 status
    let status1 = hig_node.get_transaction_status(tx1.id.clone())
        .await
        .expect("Failed to get tx1 status");
    assert!(matches!(status1, TransactionStatus::Success));

    // Create test transactions for second subblock
    let tx2 = Transaction {
        id: TransactionId("tx2".to_string()),
        data: "SUCCESS".to_string(),
    };
    cl_node.submit_transaction(CLTransaction {
        id: tx2.id.clone(),
        data: tx2.data.clone(),
        chain_id: chain_id.clone(),
    }).await.expect("Failed to submit transaction");

    // Wait for block production and get the current block
    tokio::time::sleep(Duration::from_millis(200)).await;
    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");
    println!("Current block number after tx2: {}", current_block);

    // Look for tx2 in all blocks up to the current one
    let mut found_subblock2 = None;
    for block_num in 0..=current_block {
        if let Ok(subblock) = cl_node.get_subblock(chain_id.clone(), block_num).await {
            if subblock.transactions.iter().any(|tx| tx.id == tx2.id) {
                found_subblock2 = Some(subblock);
                break;
            }
        }
    }
    let subblock2 = found_subblock2.expect("Failed to find subblock containing tx2");

    // Process the second subblock
    hig_node.process_subblock(subblock2)
        .await
        .expect("Failed to process second subblock");

    // Verify tx2 status
    let status2 = hig_node.get_transaction_status(tx2.id.clone())
        .await
        .expect("Failed to get tx2 status");
    assert!(matches!(status2, TransactionStatus::Success));
}
