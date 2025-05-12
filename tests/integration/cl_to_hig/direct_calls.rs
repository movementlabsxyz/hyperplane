use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, ChainId, SubBlock, CATStatusUpdate, BlockId},
    hyper_ig::{HyperIG, HyperIGNode},
    confirmation_layer::{ConfirmationLayer, ConfirmationNode},
};
use tokio::time::Duration;

/// Tests that a subblock is properly processed by the Hyper IG:
/// - The Confirmation Layer sends a subblock to the Hyper IG
/// - The Hyper IG processes the transactions in the subblock
/// - Verify the transaction statuses are correctly updated
#[tokio::test]
async fn test_process_subblock() {
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
        data: "any data".to_string(),
    };
    let tx2 = Transaction {
        id: TransactionId("tx2".to_string()),
        data: "DEPENDENT".to_string(),
    };
    let tx3 = Transaction {
        id: TransactionId("tx3".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };

    // Create a subblock with these transactions
    let subblock = SubBlock {
        chain_id: chain_id.clone(),
        block_id: BlockId("block1".to_string()),
        transactions: vec![tx1.clone(), tx2.clone(), tx3.clone()],
    };

    // Send the subblock to HIG
    hig.process_subblock(subblock)
        .await
        .expect("Failed to process subblock");

    // Wait a bit for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify transaction statuses
    let status1 = hig.get_transaction_status(tx1.id.clone())
        .await
        .expect("Failed to get status for tx1");
    assert!(matches!(status1, TransactionStatus::Success), "tx1 should be successful");

    let status2 = hig.get_transaction_status(tx2.id.clone())
        .await
        .expect("Failed to get status for tx2");
    assert!(matches!(status2, TransactionStatus::Pending), "tx2 should be pending (dependent)");

    let status3 = hig.get_transaction_status(tx3.id.clone())
        .await
        .expect("Failed to get status for tx3");
    assert!(matches!(status3, TransactionStatus::Pending), "tx3 should be pending (CAT)");

    // Verify tx3 is in pending transactions
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx3.id), "tx3 should be in pending transactions");

    // Verify tx3 has a proposed success status
    let proposed_status = hig.get_proposed_status(tx3.id.clone())
        .await
        .expect("Failed to get proposed status");
    assert!(matches!(proposed_status, CATStatusUpdate::Success), "tx3 should have proposed Success status");
}

/// Tests that multiple subblocks are properly processed by the Hyper IG:
/// - The Confirmation Layer sends multiple subblocks to the Hyper IG
/// - The Hyper IG processes the transactions in each subblock
/// - Verify the transaction statuses are correctly updated for each subblock
#[tokio::test]
async fn test_process_multiple_subblocks() {
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
        data: "any data".to_string(),
    };
    let tx2 = Transaction {
        id: TransactionId("tx2".to_string()),
        data: "DEPENDENT".to_string(),
    };

    // Create first subblock
    let subblock1 = SubBlock {
        chain_id: chain_id.clone(),
        block_id: BlockId("block1".to_string()),
        transactions: vec![tx1.clone(), tx2.clone()],
    };

    // Send first subblock to HIG
    hig.process_subblock(subblock1)
        .await
        .expect("Failed to process first subblock");

    // Create test transactions for second subblock
    let tx3 = Transaction {
        id: TransactionId("tx3".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    let tx4 = Transaction {
        id: TransactionId("tx4".to_string()),
        data: "CAT.SIMULATION.FAILURE".to_string(),
    };

    // Create second subblock
    let subblock2 = SubBlock {
        chain_id: chain_id.clone(),
        block_id: BlockId("block2".to_string()),
        transactions: vec![tx3.clone(), tx4.clone()],
    };

    // Send second subblock to HIG
    hig.process_subblock(subblock2)
        .await
        .expect("Failed to process second subblock");

    // Wait a bit for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify all transaction statuses
    let status1 = hig.get_transaction_status(tx1.id.clone())
        .await
        .expect("Failed to get status for tx1");
    assert!(matches!(status1, TransactionStatus::Success), "tx1 should be successful");

    let status2 = hig.get_transaction_status(tx2.id.clone())
        .await
        .expect("Failed to get status for tx2");
    assert!(matches!(status2, TransactionStatus::Pending), "tx2 should be pending (dependent)");

    let status3 = hig.get_transaction_status(tx3.id.clone())
        .await
        .expect("Failed to get status for tx3");
    assert!(matches!(status3, TransactionStatus::Pending), "tx3 should be pending (CAT)");

    let status4 = hig.get_transaction_status(tx4.id.clone())
        .await
        .expect("Failed to get status for tx4");
    assert!(matches!(status4, TransactionStatus::Pending), "tx4 should be pending (CAT)");

    // Verify CAT transactions are in pending transactions
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx3.id), "tx3 should be in pending transactions");
    assert!(pending.contains(&tx4.id), "tx4 should be in pending transactions");

    // Verify proposed statuses for CAT transactions
    let proposed_status3 = hig.get_proposed_status(tx3.id.clone())
        .await
        .expect("Failed to get proposed status for tx3");
    assert!(matches!(proposed_status3, CATStatusUpdate::Success), "tx3 should have proposed Success status");

    let proposed_status4 = hig.get_proposed_status(tx4.id.clone())
        .await
        .expect("Failed to get proposed status for tx4");
    assert!(matches!(proposed_status4, CATStatusUpdate::Failure), "tx4 should have proposed Failure status");
} 