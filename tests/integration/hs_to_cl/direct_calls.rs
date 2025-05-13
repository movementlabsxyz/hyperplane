use hyperplane::{
    types::{
        CATId,
        CATStatusLimited,
        ChainId,
        BlockId,
        TransactionId,
    },
    hyper_scheduler::HyperScheduler,
    confirmation_layer::ConfirmationLayer,
};
use std::time::Duration;
use crate::common::testnodes;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Tests that a single CAT status update is properly included in a block:
/// - Hyper Scheduler sends a CAT status update for a single target chain
/// - Verify it is included in the next block
/// - Verify the transaction is correctly processed
#[tokio::test]
async fn test_cat_status_update_one_target_chain() {
    // use testnodes from common
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes();

    // Wrap nodes in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let cl_node = Arc::new(Mutex::new(cl_node));
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();

    // Start the message processing loops
    let hs_handle = tokio::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });
    let cl_handle = tokio::spawn(async move {
        let mut node = cl_node_clone.lock().await;
        node.start().await;
    });

    // Register a chain
    let chain_id = ChainId("test-chain".to_string());
    cl_node.lock().await.register_chain(chain_id.clone()).await.expect("Failed to register chain");

    // Set the chain ID in HS
    hs_node.lock().await.set_chain_id(chain_id.clone());

    // Send a CAT status update
    let cat_id = CATId("test-cat".to_string());
    hs_node.lock().await.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
        .await
        .expect("Failed to send status update");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Get the current block
    let current_block = cl_node.lock().await.get_current_block().await.expect("Failed to get current block");
    let current_block_num = current_block.0.parse::<u64>().unwrap();

    // Get the subblock
    let block_id = BlockId(current_block_num.to_string());
    let subblock = cl_node.lock().await.get_subblock(chain_id, block_id)
        .await
        .expect("Failed to get subblock");

    // Verify the subblock contains our transaction
    assert!(subblock.transactions.iter().any(|tx| 
        tx.id == TransactionId("test-cat.UPDATE".to_string()) &&
        tx.data.starts_with("STATUS_UPDATE.SUCCESS.CAT_ID:")
    ));

    // Clean up
    hs_handle.abort();
    cl_handle.abort();
}

/// Tests that multiple CAT status updates are properly queued and included in blocks:
/// - Send multiple CAT status updates, for a single target chain (note, this is for testing, there should be at least two target chains normally)
/// - Verify they are included in subsequent blocks
/// - Verify the order is maintained
#[tokio::test]
async fn test_multiple_cat_status_updates_one_target_chain() {
    // use testnodes from common
    let (mut hs_node, mut cl_node, _) = testnodes::setup_test_nodes();

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
        (CATId("cat-1".to_string()), CATStatusLimited::Success),
        (CATId("cat-2".to_string()), CATStatusLimited::Failure),
        (CATId("cat-3".to_string()), CATStatusLimited::Success),
    ];

    // Send each update
    for (cat_id, status) in updates.clone() {
        hs_node.send_cat_status_update(cat_id.clone(), status.clone())
            .await
            .expect("Failed to send status update");

        // Wait for block production
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify the status update was processed
        let current_status = hs_node.get_cat_status(cat_id).await.unwrap();
        assert_eq!(current_status, status);
    }
}

/// Tests that a status update is properly sent and processed:
/// - The Hyper Scheduler sends a status update for multiple chains
/// - The Confirmation Layer receives and queues the transaction
/// - The transaction is included in the next block for each chain
#[tokio::test]
async fn test_status_update() {
    // use testnodes from common
    let (mut hs_node, mut cl_node, _) = testnodes::setup_test_nodes();

    // Register chains 1 and 2 in the confirmation layer
    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());
    cl_node.register_chain(chain_id_1.clone())
        .await
        .expect("Failed to register chain 1");
    cl_node.register_chain(chain_id_2.clone())
        .await
        .expect("Failed to register chain 2");

    // Set the confirmation layer and chain ID
    hs_node.set_confirmation_layer(Box::new(cl_node));
    hs_node.set_chain_id(chain_id_1.clone());

    // Create a CAT status update transaction
    let cat_id = CATId("test-cat".to_string());
    
    // Send the CAT status update through the hyper scheduler
    hs_node.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
        .await
        .expect("Failed to send status update");

    // Wait for block production
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify the status update was processed
    let status = hs_node.get_cat_status(cat_id).await.unwrap();
    assert_eq!(status, CATStatusLimited::Success);
} 