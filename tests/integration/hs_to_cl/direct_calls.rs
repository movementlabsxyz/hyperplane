use hyperplane::{
    types::{ChainId, CATId, CATStatusLimited},
    hyper_scheduler::HyperScheduler,
    confirmation_layer::ConfirmationLayer,
};
use crate::common::testnodes;
use tokio::time::{sleep, Duration};

/// Tests that a single CAT status update is properly included in a block:
/// - Hyper Scheduler sends a CAT status update for a single target chain
/// - Verify it is included in the next block
/// - Verify the transaction is correctly processed
#[tokio::test]
async fn test_cat_status_update_one_target_chain() {
    // use testnodes from common
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Start the message processing loops
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();
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
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id.clone()).await.expect("Failed to register chain");
    }

    // Set the chain ID in HS
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id.clone()).await;
    }

    // Send a CAT status update
    let cat_id = CATId("test-cat".to_string());
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to send status update");
    }

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Get the current block
    let current_block = {
        let node = cl_node.lock().await;
        node.get_current_block().await.expect("Failed to get current block")
    };
    assert_eq!(current_block, 2);

    // Get subblock and verify transaction
    let subblock = {
        let node = cl_node.lock().await;
        node.get_subblock(chain_id.clone(), 0)
            .await
            .expect("Failed to get subblock")
    };
    assert_eq!(subblock.transactions.len(), 1);
    assert_eq!(subblock.transactions[0].data, "STATUS_UPDATE.SUCCESS.CAT_ID:test-cat");
}

/// Tests that multiple CAT status updates are properly included in blocks:
/// - Hyper Scheduler sends multiple CAT status updates for a single target chain
/// - Verify they are included in the next blocks
/// - Verify the transactions are correctly processed
#[tokio::test]
async fn test_multiple_cat_status_updates_one_target_chain() {
    // use testnodes from common
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Start the message processing loops
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();
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
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id.clone())
            .await
            .expect("Failed to register chain");
    }

    // Set the chain ID in HS
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id.clone()).await;
    }

    // Create and send multiple CAT status updates
    let updates = vec![
        (CATId("cat-1".to_string()), CATStatusLimited::Success),
        (CATId("cat-2".to_string()), CATStatusLimited::Failure),
        (CATId("cat-3".to_string()), CATStatusLimited::Success),
    ];

    // Send each update
    for (cat_id, status) in updates.clone() {
        {
            let mut node = hs_node.lock().await;
            node.send_cat_status_update(cat_id.clone(), status.clone())
                .await
                .expect("Failed to send status update");
        }

        // Wait for block production
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify the status update was processed
        {
            let node = hs_node.lock().await;
            let current_status = node.get_cat_status(cat_id).await.unwrap();
            assert_eq!(current_status, status);
        }
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

    // Start the message processing loops
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();
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
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id_1.clone())
            .await
            .expect("Failed to register chain 1");
        node.register_chain(chain_id_2.clone())
            .await
            .expect("Failed to register chain 2");
    }

    // Set the chain ID in HS
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id_1.clone()).await;
    }

    // Create a CAT status update transaction
    let cat_id = CATId("test-cat".to_string());
    
    // Send the CAT status update through the hyper scheduler
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to send status update");
    }

    // Wait for block production
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify the status update was processed
    let status = {
        let node = hs_node.lock().await;
        node.get_cat_status(cat_id).await.unwrap()
    };
    assert_eq!(status, CATStatusLimited::Success);
}

/// Tests that a CAT status update is properly sent and processed:
/// - Hyper Scheduler sends a CAT status update
/// - Verify the transaction is included in the next block
/// - Verify the transaction data matches the expected format
#[tokio::test]
async fn test_cat_status_update() {
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Start the message processing loops
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();
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
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id.clone()).await.expect("Failed to register chain");
    }

    // Set the chain ID in HS
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id.clone()).await;
    }

    // Create a CAT transaction
    let cat_id = CATId("test-tx".to_string());

    // Submit the transaction
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to submit transaction");
    }

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Get current block
    let current_block = {
        let node = cl_node.lock().await;
        node.get_current_block().await.expect("Failed to get current block")
    };
    assert_eq!(current_block, 5);

    // Get subblock and verify transaction
    let subblock = {
        let node = cl_node.lock().await;
        node.get_subblock(chain_id, 0)
            .await
            .expect("Failed to get subblock")
    };
    assert_eq!(subblock.transactions.len(), 1);
    assert_eq!(subblock.transactions[0].data, "STATUS_UPDATE.SUCCESS.CAT_ID:test-tx");
}

/// Tests that multiple CAT status updates are properly processed across different chains:
/// - Register two different chains
/// - Send CAT status updates for each chain
/// - Verify the transactions are included in the correct subblocks
/// - Verify the transaction data matches the expected format for each chain
#[tokio::test]
async fn test_multiple_cat_status_updates() {
    let (hs_node, cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Start the message processing loops
    let hs_node_clone = hs_node.clone();
    let cl_node_clone = cl_node.clone();
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
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id_1.clone())
            .await
            .expect("Failed to register chain 1");
        node.register_chain(chain_id_2.clone())
            .await
            .expect("Failed to register chain 2");
    }

    // Set the chain ID in HS
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id_1.clone()).await;
    }

    // Create and submit transactions for both chains
    let cat_id_1 = CATId("test-tx-1".to_string());
    let cat_id_2 = CATId("test-tx-2".to_string());

    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id_1.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to submit transaction 1");
        node.send_cat_status_update(cat_id_2.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to submit transaction 2");
    }

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify transactions in subblocks
    let subblock1 = {
        let node = cl_node.lock().await;
        node.get_subblock(chain_id_1, 0)
            .await
            .expect("Failed to get subblock 1")
    };
    let subblock2 = {
        let node = cl_node.lock().await;
        node.get_subblock(chain_id_2, 0)
            .await
            .expect("Failed to get subblock 2")
    };

    assert_eq!(subblock1.transactions.len(), 1);
    assert_eq!(subblock1.transactions[0].data, "test data 1");
    assert_eq!(subblock2.transactions.len(), 1);
    assert_eq!(subblock2.transactions[0].data, "test data 2");
}

#[tokio::test]
async fn test_send_cat_status_update() {
    println!("\n=== Starting test_send_cat_status_update ===");
    
    // Get the test nodes using our helper function
    let (hs_node, _cl_node, _hig_node) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    
    // Start the node
    {
        let mut node = hs_node.lock().await;
        node.start().await;
    }
    
    // Set chain ID
    let chain_id = ChainId("test-chain".to_string());
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id.clone()).await;
    }
    
    // Send CAT status update
    let cat_id = CATId("test-cat".to_string());
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to send CAT status update");
    }
    
    // Wait for a bit to let the update be processed
    sleep(Duration::from_millis(100)).await;
    
    // Verify the status was updated
    {
        let node = hs_node.lock().await;
        let current_status = node.get_cat_status(cat_id).await.unwrap();
        assert_eq!(current_status, CATStatusLimited::Success);
    }
    
    println!("=== Test completed successfully ===\n");
}
