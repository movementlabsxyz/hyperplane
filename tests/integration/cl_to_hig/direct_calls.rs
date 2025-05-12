use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, ChainId, BlockId, CLTransaction},
    hyper_ig::{HyperIG, HyperIGNode},
    confirmation_layer::{ConfirmationNode, ConfirmationLayer},
};
use tokio::time::Duration;

/// Tests that a subblock with new transactions is properly processed by the Hyper IG:
/// - The Confirmation Layer sends a subblock to the Hyper IG
/// - The Hyper IG processes the transactions in the subblock
/// - Verify the transaction statuses are correctly updated
#[tokio::test]
async fn test_process_subblock_normal_transactions() {
    // Create HIG and CL nodes
    let mut hig = HyperIGNode::new();
    let mut cl = ConfirmationNode::with_block_interval(Duration::from_millis(100))
        .expect("Failed to create confirmation node");

    // Register a test chain
    let chain_id = ChainId("test-chain".to_string());
    cl.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Create test transactions
    let tx1 = Transaction {
        id: TransactionId("tx1".to_string()),
        data: "SUCCESS".to_string(),
    };
    let tx2 = Transaction {
        id: TransactionId("tx2".to_string()),
        data: "DEPENDENT".to_string(),
    };

    // submit the transactions to the CL
    cl.submit_transaction(CLTransaction {
        id: tx1.id.clone(),
        data: tx1.data.clone(),
        chain_id: chain_id.clone(),
    }).await.expect("Failed to submit transaction");
    cl.submit_transaction(CLTransaction {
        id: tx2.id.clone(),
        data: tx2.data.clone(),
        chain_id: chain_id.clone(),
    }).await.expect("Failed to submit transaction");

    // Wait for block production and get the current block
    tokio::time::sleep(Duration::from_millis(200)).await;
    let current_block = cl.get_current_block().await.expect("Failed to get current block");
    let current_block_num = current_block.0.parse::<u64>().unwrap();
    println!("Current block number: {}", current_block_num);

    // Look for our transactions in all blocks up to the current one
    let mut found_subblock = None;
    for block_num in 0..current_block_num {
        let block_id = BlockId(block_num.to_string());
        let subblock = cl.get_subblock(chain_id.clone(), block_id.clone())
            .await
            .expect(&format!("Failed to get subblock for block {}", block_num));
        println!("Checking block {}: tx_count={}", block_num, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("  Transaction: id={}, data={}", tx.id.0, tx.data);
        }
        if subblock.transactions.iter().any(|tx| tx.id == tx1.id) && 
           subblock.transactions.iter().any(|tx| tx.id == tx2.id) {
            found_subblock = Some(subblock);
            break;
        }
    }

    let subblock = found_subblock.expect("Did not find subblock containing our transactions");
    assert_eq!(subblock.transactions.len(), 2, "Subblock should contain 2 transactions");
    assert!(subblock.transactions.iter().any(|tx| tx.id == tx1.id), "Subblock should contain tx1");
    assert!(subblock.transactions.iter().any(|tx| tx.id == tx2.id), "Subblock should contain tx2");

    // Send the subblock to HIG
    // TODO: we have no direct connection between CL at this point, so we just call the method for now
    // TODO: hig should just be listening for CL events
    hig.process_subblock(subblock)
        .await
        .expect("Failed to process subblock");

    // Verify transaction statuses
    let status1 = hig.get_transaction_status(tx1.id.clone())
        .await
        .expect("Failed to get status for tx1");
    assert!(matches!(status1, TransactionStatus::Success), "tx1 should be successful");
    let status2 = hig.get_transaction_status(tx2.id.clone())
        .await
        .expect("Failed to get status for tx2");
    assert!(matches!(status2, TransactionStatus::Pending), "tx2 should be pending (dependent)");
}

/// Tests that multiple subblocks with new transactions are properly processed by the Hyper IG:
/// - The Confirmation Layer sends multiple subblocks to the Hyper IG
/// - The Hyper IG processes the transactions in each subblock
/// - Verify the transaction statuses are correctly updated for each subblock
#[tokio::test]
async fn test_process_multiple_subblocks_new_transactions() {
    // Create HIG and CL nodes
    let mut hig = HyperIGNode::new();
    let mut cl = ConfirmationNode::with_block_interval(Duration::from_millis(100))
        .expect("Failed to create confirmation node");

    // Register a test chain
    let chain_id = ChainId("test-chain".to_string());
    cl.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Create test transactions for first subblock
    let tx1 = Transaction {
        id: TransactionId("tx1".to_string()),
        data: "SUCCESS".to_string(),
    };
    // submit the transactions to the CL
    cl.submit_transaction(CLTransaction {
        id: tx1.id.clone(),
        data: tx1.data.clone(),
        chain_id: chain_id.clone(),
    }).await.expect("Failed to submit transaction");

    // Wait for block production and get the current block
    tokio::time::sleep(Duration::from_millis(200)).await;
    let current_block = cl.get_current_block().await.expect("Failed to get current block");
    let current_block_num = current_block.0.parse::<u64>().unwrap();
    println!("Current block number after tx1: {}", current_block_num);

    // Look for tx1 in all blocks up to the current one
    let mut found_subblock1 = None;
    for block_num in 0..current_block_num {
        let block_id = BlockId(block_num.to_string());
        let subblock = cl.get_subblock(chain_id.clone(), block_id.clone())
            .await
            .expect(&format!("Failed to get subblock for block {}", block_num));
        println!("Checking block {} for tx1: tx_count={}", block_num, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("  Transaction: id={}, data={}", tx.id.0, tx.data);
        }
        if subblock.transactions.iter().any(|tx| tx.id == tx1.id) {
            found_subblock1 = Some(subblock);
            break;
        }
    }

    let subblock1 = found_subblock1.expect("Did not find subblock containing tx1");
    assert_eq!(subblock1.transactions.len(), 1, "First subblock should contain 1 transaction");
    assert!(subblock1.transactions.iter().any(|tx| tx.id == tx1.id), "First subblock should contain tx1");

    // Process first subblock
    hig.process_subblock(subblock1)
        .await
        .expect("Failed to process first subblock");

    // Create test transactions for second subblock
    let tx2 = Transaction {
        id: TransactionId("tx2".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    // submit the transactions to the CL
    cl.submit_transaction(CLTransaction {
        id: tx2.id.clone(),
        data: tx2.data.clone(),
        chain_id: chain_id.clone(),
    }).await.expect("Failed to submit transaction");

    // Wait for block production and get the current block
    tokio::time::sleep(Duration::from_millis(200)).await;
    let current_block = cl.get_current_block().await.expect("Failed to get current block");
    let current_block_num = current_block.0.parse::<u64>().unwrap();
    println!("Current block number after tx2: {}", current_block_num);

    // Look for tx2 in all blocks up to the current one
    let mut found_subblock2 = None;
    for block_num in 0..current_block_num {
        let block_id = BlockId(block_num.to_string());
        let subblock = cl.get_subblock(chain_id.clone(), block_id.clone())
            .await
            .expect(&format!("Failed to get subblock for block {}", block_num));
        println!("Checking block {} for tx2: tx_count={}", block_num, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("  Transaction: id={}, data={}", tx.id.0, tx.data);
        }
        if subblock.transactions.iter().any(|tx| tx.id == tx2.id) {
            found_subblock2 = Some(subblock);
            break;
        }
    }

    let subblock2 = found_subblock2.expect("Did not find subblock containing tx2");
    assert_eq!(subblock2.transactions.len(), 1, "Second subblock should contain 1 transaction");
    assert!(subblock2.transactions.iter().any(|tx| tx.id == tx2.id), "Second subblock should contain tx2");

    // Process second subblock
    hig.process_subblock(subblock2)
        .await
        .expect("Failed to process second subblock");

    // Verify all transaction statuses
    let status1 = hig.get_transaction_status(tx1.id.clone())
        .await
        .expect("Failed to get status for tx1");
    assert!(matches!(status1, TransactionStatus::Success), "tx1 should be successful");

    let status2 = hig.get_transaction_status(tx2.id.clone())
        .await
        .expect("Failed to get status for tx2");
    assert!(matches!(status2, TransactionStatus::Pending), "tx2 should be pending (dependent)");

    // Verify CAT transaction is in pending transactions
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx2.id), "tx2 should be in pending transactions");
} 