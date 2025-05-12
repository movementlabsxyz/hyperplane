use hyperplane::{
    types::{CATId, ChainId, CATStatusUpdate, TransactionId, BlockId},
    hyper_scheduler::{HyperScheduler, HyperSchedulerNode},
    confirmation_layer::{ConfirmationLayer, ConfirmationNode},
};
use tokio::time::Duration;

/// Tests that a single CAT status update is properly included in a block:
/// - Hyper Scheduler sends a CAT status update for a single target chain
/// - Verify it is included in the next block
/// - Verify the transaction is correctly processed
#[tokio::test]
async fn test_cat_status_update_one_target_chain() {
    // Create a confirmation node with a short block interval for testing
    let mut cl_node = ConfirmationNode::with_block_interval(Duration::from_millis(100))
        .expect("Failed to create confirmation node");

    // Create a hyper scheduler node
    let mut hs_node = HyperSchedulerNode::new();

    // Register a test chain
    let chain_id = ChainId("test-chain".to_string());
    cl_node.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Set the confirmation layer and chain ID
    hs_node.set_confirmation_layer(Box::new(cl_node));
    hs_node.set_chain_id(chain_id.clone());

    // Create a CAT status update transaction
    let cat_id = CATId("test-cat".to_string());
    
    // Send the CAT status update through the hyper scheduler
    hs_node.send_cat_status_update(cat_id.clone(), CATStatusUpdate::Success)
        .await
        .expect("Failed to send CAT status update");

    // Wait for block production (2x block interval to be safe)
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Get the subblock for our chain in block 0
    let subblock = hs_node.confirmation_layer().unwrap().get_subblock(chain_id.clone(), hyperplane::types::BlockId("0".to_string()))
        .await
        .expect("Failed to get subblock");

    // Verify the transaction was included
    assert!(!subblock.transactions.is_empty(), "No transactions in subblock");
    
    // Verify the transaction data matches our CAT status update
    let tx = &subblock.transactions[0];
    assert_eq!(tx.data, format!("STATUS_UPDATE.SUCCESS.CAT_ID:{}", cat_id.0), "Transaction data does not match expected status update");
}

/// Tests that multiple CAT status updates are properly queued and included in blocks:
/// - Send multiple CAT status updates, for a single target chain (note, this is for testing, there should be at least two target chains normally)
/// - Verify they are included in subsequent blocks
/// - Verify the order is maintained
#[tokio::test]
async fn test_multiple_cat_status_updates_one_target_chain() {
    // Create a confirmation node with a short block interval
    let mut cl_node = ConfirmationNode::with_block_interval(Duration::from_millis(100))
        .expect("Failed to create confirmation node");

    // Create a hyper scheduler node
    let mut hs_node = HyperSchedulerNode::new();

    // Register a test chain
    let chain_id = ChainId("test-chain".to_string());
    cl_node.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Set the confirmation layer and chain ID
    hs_node.set_confirmation_layer(Box::new(cl_node));
    hs_node.set_chain_id(chain_id.clone());

    // Create and send multiple CAT status updates
    let updates = vec![
        (CATId("cat-1".to_string()), CATStatusUpdate::Success),
        (CATId("cat-2".to_string()), CATStatusUpdate::Failure),
        (CATId("cat-3".to_string()), CATStatusUpdate::Success),
    ];

    // Track which block each transaction was included in
    let mut transaction_blocks = Vec::new();

    for (_i, (cat_id, status)) in updates.iter().enumerate() {
        // Send the status update
        hs_node.send_cat_status_update(cat_id.clone(), status.clone())
            .await
            .expect("Failed to send CAT status update");

        // Wait for block production (2x block interval to be safe)
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Get current block after each update
        let current_block = hs_node.confirmation_layer().unwrap().get_current_block()
            .await
            .expect("Failed to get current block");

        // Find which block contains this transaction
        let mut found = false;
        for block_num in 0..current_block.0.parse::<u64>().unwrap() {
            let block_id = format!("{}", block_num);
            let subblock = hs_node.confirmation_layer().unwrap().get_subblock(chain_id.clone(), hyperplane::types::BlockId(block_id.clone()))
                .await
                .expect("Failed to get subblock");

            if !subblock.transactions.is_empty() {
                let tx = &subblock.transactions[0];
                // Check that the transaction data matches the status (Success/Failure) from our updates vector
                let expected_data = format!("STATUS_UPDATE.{}.CAT_ID:{}", 
                    match status {
                        CATStatusUpdate::Success => "SUCCESS",
                        CATStatusUpdate::Failure => "FAILURE",
                    },
                    cat_id.0
                );
                if tx.data == expected_data {
                    transaction_blocks.push(block_num);
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "Transaction for CAT {} not found in any block", cat_id.0);
    }

    // Verify the transactions were included in order
    for i in 1..transaction_blocks.len() {
        assert!(transaction_blocks[i] > transaction_blocks[i-1], 
            "Transaction {} was included in block {} before transaction {} in block {}", 
            i, transaction_blocks[i], i-1, transaction_blocks[i-1]);
    }
}

/// Tests that a status update is properly sent and processed:
/// - The Hyper Scheduler sends a status update for multiple chains
/// - The Confirmation Layer receives and queues the transaction
/// - The transaction is included in the next block for each chain
#[tokio::test]
async fn test_status_update() {
    let mut hs = HyperSchedulerNode::new();
    let mut cl = ConfirmationNode::with_block_interval(Duration::from_millis(100))
        .expect("Failed to create confirmation node");

    // Register chains 1 and 2 in the confirmation layer
    let chain1 = ChainId("chain1".to_string());
    let chain2 = ChainId("chain2".to_string());
    cl.register_chain(chain1.clone()).await.expect("Failed to register chain1");
    cl.register_chain(chain2.clone()).await.expect("Failed to register chain2");

    // Connect HS to CL
    hs.set_confirmation_layer(Box::new(cl));
    
    // Set chain IDs in HS
    hs.set_chain_id(chain1.clone());
    hs.set_chain_id(chain2.clone());

    // Create a status update message for chains 1 and 2
    let cat_id1 = CATId("cat1".to_string());
    let cat_id2 = CATId("cat2".to_string());
    let status = CATStatusUpdate::Success;

    // HS sends the status update message for chain1
    hs.send_cat_status_update(cat_id1.clone(), status.clone())
        .await
        .expect("Failed to send status update for chain1");

    // HS sends the status update message for chain2
    hs.send_cat_status_update(cat_id2.clone(), status.clone())
        .await
        .expect("Failed to send status update for chain2");

    // Wait for block production (2x block interval to be safe)
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify that subblocks for both chains contain the status update message
    let subblock1 = hs.confirmation_layer().unwrap().get_subblock(chain1.clone(), BlockId("0".to_string())).await.expect("Failed to get subblock for chain1");
    let subblock2 = hs.confirmation_layer().unwrap().get_subblock(chain2.clone(), BlockId("0".to_string())).await.expect("Failed to get subblock for chain2");

    assert!(subblock1.transactions.iter().any(|tx| tx.id == TransactionId(cat_id1.0.clone())));
    assert!(subblock2.transactions.iter().any(|tx| tx.id == TransactionId(cat_id2.0.clone())));
} 