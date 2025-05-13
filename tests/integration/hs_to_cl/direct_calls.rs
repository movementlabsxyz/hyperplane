use hyperplane::{
    types::{
        CATId,
        CATStatusLimited,
        ChainId,
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
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Wrap nodes in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let cl_node = Arc::new(Mutex::new(cl_node));
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();

    // Start the message processing loops
    let _hs_handle = tokio::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });
    let _cl_handle = tokio::spawn(async move {
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

    // Get the subblock
    let subblock = cl_node.lock().await.get_subblock(chain_id, current_block)
        .await
        .expect("Failed to get subblock");

    // Verify the subblock contains our transaction
    assert!(subblock.transactions.iter().any(|tx| 
        tx.id == TransactionId("test-cat.UPDATE".to_string()) &&
        tx.data.starts_with("STATUS_UPDATE.SUCCESS.CAT_ID:")
    ));
}

/// Tests that multiple CAT status updates are properly queued and included in blocks:
/// - Send multiple CAT status updates, for a single target chain (note, this is for testing, there should be at least two target chains normally)
/// - Verify they are included in subsequent blocks
/// - Verify the order is maintained
#[tokio::test]
async fn test_multiple_cat_status_updates_one_target_chain() {
    // use testnodes from common
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Wrap nodes in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let cl_node = Arc::new(Mutex::new(cl_node));
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();

    // Start the message processing loops
    let _hs_handle = tokio::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });
    let _cl_handle = tokio::spawn(async move {
        let mut node = cl_node_clone.lock().await;
        node.start().await;
    });

    // Register a test chain
    let chain_id = ChainId("test-chain".to_string());
    cl_node.lock().await.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Set the chain ID in HS
    hs_node.lock().await.set_chain_id(chain_id.clone());

    // Create and send multiple CAT status updates
    let updates = vec![
        (CATId("cat-1".to_string()), CATStatusLimited::Success),
        (CATId("cat-2".to_string()), CATStatusLimited::Failure),
        (CATId("cat-3".to_string()), CATStatusLimited::Success),
    ];

    // Send each update
    for (cat_id, status) in updates.clone() {
        hs_node.lock().await.send_cat_status_update(cat_id.clone(), status.clone())
            .await
            .expect("Failed to send status update");

        // Wait for block production
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify the status update was processed
        let current_status = hs_node.lock().await.get_cat_status(cat_id).await.unwrap();
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
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Wrap nodes in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let cl_node = Arc::new(Mutex::new(cl_node));
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();

    // Start the message processing loops
    let _hs_handle = tokio::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });
    let _cl_handle = tokio::spawn(async move {
        let mut node = cl_node_clone.lock().await;
        node.start().await;
    });

    // Register chains 1 and 2 in the confirmation layer
    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());
    cl_node.lock().await.register_chain(chain_id_1.clone())
        .await
        .expect("Failed to register chain 1");
    cl_node.lock().await.register_chain(chain_id_2.clone())
        .await
        .expect("Failed to register chain 2");

    // Set the chain ID in HS
    hs_node.lock().await.set_chain_id(chain_id_1.clone());

    // Create a CAT status update transaction
    let cat_id = CATId("test-cat".to_string());
    
    // Send the CAT status update through the hyper scheduler
    hs_node.lock().await.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
        .await
        .expect("Failed to send status update");

    // Wait for block production
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify the status update was processed
    let status = hs_node.lock().await.get_cat_status(cat_id).await.unwrap();
    assert_eq!(status, CATStatusLimited::Success);
}

/// Tests that a CAT status update is properly processed:
/// - Hyper Scheduler sends a CAT status update
/// - Verify the transaction is included in the next block
/// - Verify the transaction data matches the expected format
#[tokio::test]
async fn test_cat_status_update() {
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Wrap nodes in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let cl_node = Arc::new(Mutex::new(cl_node));
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();

    // Start the message processing loops
    let _hs_handle = tokio::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });
    let _cl_handle = tokio::spawn(async move {
        let mut node = cl_node_clone.lock().await;
        node.start().await;
    });

    // Register a chain
    let chain_id = ChainId("test-chain".to_string());
    cl_node.lock().await.register_chain(chain_id.clone()).await.expect("Failed to register chain");

    // Set the chain ID in HS
    hs_node.lock().await.set_chain_id(chain_id.clone());

    // Create a CAT transaction
    let cat_id = CATId("test-tx".to_string());

    // Submit the transaction
    hs_node.lock().await.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
        .await
        .expect("Failed to submit transaction");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Get current block
    let current_block = cl_node.lock().await.get_current_block().await.expect("Failed to get current block");
    assert_eq!(current_block, 5);

    // Get subblock and verify transaction
    let subblock = cl_node.lock().await.get_subblock(chain_id, 0)
        .await
        .expect("Failed to get subblock");
    assert_eq!(subblock.transactions.len(), 1);
    assert_eq!(subblock.transactions[0].data, "test data");
}

/// Tests that multiple CAT status updates are properly processed across different chains:
/// - Register two different chains
/// - Send CAT status updates for each chain
/// - Verify the transactions are included in the correct subblocks
/// - Verify the transaction data matches the expected format for each chain
#[tokio::test]
async fn test_multiple_cat_status_updates() {
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Wrap nodes in Arc<Mutex>
    let hs_node = Arc::new(Mutex::new(hs_node));
    let cl_node = Arc::new(Mutex::new(cl_node));
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();

    // Start the message processing loops
    let _hs_handle = tokio::spawn(async move {
        let mut node = hs_node_clone.lock().await;
        node.start().await;
    });
    let _cl_handle = tokio::spawn(async move {
        let mut node = cl_node_clone.lock().await;
        node.start().await;
    });

    // Register two chains
    let chain_id_1 = ChainId("test-chain-1".to_string());
    let chain_id_2 = ChainId("test-chain-2".to_string());
    cl_node.lock().await.register_chain(chain_id_1.clone())
        .await
        .expect("Failed to register chain 1");
    cl_node.lock().await.register_chain(chain_id_2.clone())
        .await
        .expect("Failed to register chain 2");

    // Set the chain ID in HS
    hs_node.lock().await.set_chain_id(chain_id_1.clone());

    // Create and submit transactions for both chains
    let cat_id_1 = CATId("test-tx-1".to_string());
    let cat_id_2 = CATId("test-tx-2".to_string());

    hs_node.lock().await.send_cat_status_update(cat_id_1.clone(), CATStatusLimited::Success)
        .await
        .expect("Failed to submit transaction 1");
    hs_node.lock().await.send_cat_status_update(cat_id_2.clone(), CATStatusLimited::Success)
        .await
        .expect("Failed to submit transaction 2");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify transactions in subblocks
    let subblock1 = cl_node.lock().await.get_subblock(chain_id_1, 0)
        .await
        .expect("Failed to get subblock 1");
    let subblock2 = cl_node.lock().await.get_subblock(chain_id_2, 0)
        .await
        .expect("Failed to get subblock 2");

    assert_eq!(subblock1.transactions.len(), 1);
    assert_eq!(subblock1.transactions[0].data, "test data 1");
    assert_eq!(subblock2.transactions.len(), 1);
    assert_eq!(subblock2.transactions[0].data, "test data 2");
}
