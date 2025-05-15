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
    println!("\n[TEST]   === Starting test_process_subblock ===");
    
    // Initialize components with 100ms block interval
    println!("[TEST]   Setting up test nodes with 100ms block interval...");
    let (_, mut cl_node, hig_node,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register chain
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST]   Registering chain: {}", chain_id.0);
    cl_node.register_chain(chain_id.clone()).await.expect("Failed to register chain");
    println!("[TEST]   Chain registered successfully");

    // Submit transaction to CL
    let tx = Transaction {
        id: TransactionId("test-tx".to_string()),
        data: "test data".to_string(),
    };
    println!("[TEST]   Submitting transaction with ID: {}", tx.id.0);
    cl_node.submit_transaction(CLTransaction {
        id: tx.id.clone(),
        data: tx.data.clone(),
        chain_id: chain_id.clone(),
    })
    .await
    .expect("Failed to submit transaction");
    println!("[TEST]   Transaction submitted successfully");

    // Wait for block production and processing (150ms to ensure block is produced and processed)
    println!("[TEST]   Waiting for block production and processing (150ms)...");
    tokio::time::sleep(Duration::from_millis(150)).await;
    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");
    println!("[TEST]   Current block height: {}", current_block);
    assert!(current_block >= 1, "No block was produced");

    // Verify transaction status
    println!("[TEST]   Verifying transaction status...");
    let status = hig_node.lock().await.get_transaction_status(tx.id).await.unwrap();
    println!("[TEST]   Retrieved transaction status: {:?}", status);
    assert!(matches!(status, TransactionStatus::Success));
    println!("[TEST]   Transaction status verification successful");
    
    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests that a subblock with a CAT transaction is properly processed by the Hyper IG:
/// - The Confirmation Layer sends a subblock with a CAT transaction to the Hyper IG
/// - The Hyper IG processes the CAT transaction
/// - Verify the CAT transaction status is correctly updated
#[tokio::test]
async fn test_process_cat_subblock() {
    // Initialize components with 100ms block interval
    let (_hs_node, mut cl_node, hig_node,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;

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

    // Wait for block production (150ms to ensure block is produced)
    tokio::time::sleep(Duration::from_millis(150)).await;
    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");
    assert!(current_block >= 1, "No block was produced");

    // Get the block containing our transaction (should be block 1)
    let subblock = cl_node.get_subblock(chain_id, 1)
        .await
        .expect("Failed to get subblock");

    // Process subblock in HIG
    hig_node.lock().await.process_subblock(subblock)
        .await
        .expect("Failed to process subblock");

    // Verify transaction status
    let status = hig_node.lock().await.get_transaction_status(tx.id).await.unwrap();
    assert!(matches!(status, TransactionStatus::Pending));
}

/// Tests that multiple subblocks with new transactions are properly processed by the Hyper IG:
/// - The Confirmation Layer sends multiple subblocks to the Hyper IG
/// - The Hyper IG processes the transactions in each subblock
/// - Verify the transaction statuses are correctly updated for each subblock
#[tokio::test]
async fn test_process_multiple_subblocks_new_transactions() {
    // Create HIG and CL nodes with 100ms block interval
    let (_, mut cl_node, hig_node,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;

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

    // Wait for block production and get the current block (150ms to ensure block is produced)
    tokio::time::sleep(Duration::from_millis(150)).await;
    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");
    assert!(current_block >= 1, "No block was produced for tx1");
    println!("[TEST]   Current block number after tx1: {}", current_block);

    // Get subblock for tx1 (should be block 1)
    let subblock1 = cl_node.get_subblock(chain_id.clone(), 1)
        .await
        .expect("Failed to get subblock for tx1");

    // Process the first subblock
    hig_node.lock().await.process_subblock(subblock1)
        .await
        .expect("Failed to process first subblock");

    // Verify tx1 status
    let status1 = hig_node.lock().await.get_transaction_status(tx1.id.clone())
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

    // Wait for block production and get the current block (150ms to ensure block is produced)
    tokio::time::sleep(Duration::from_millis(150)).await;
    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");
    assert!(current_block >= 3, "No new block was produced for tx2");
    println!("[TEST]   Current block number after tx2: {}", current_block);

    // Get subblock for tx2 (should be block 3)
    let subblock2 = cl_node.get_subblock(chain_id.clone(), 3)
        .await
        .expect("Failed to get subblock for tx2");

    // Process the second subblock
    hig_node.lock().await.process_subblock(subblock2)
        .await
        .expect("Failed to process second subblock");

    // Verify tx2 status
    let status2 = hig_node.lock().await.get_transaction_status(tx2.id.clone())
        .await
        .expect("Failed to get tx2 status");
    assert!(matches!(status2, TransactionStatus::Success));
}
