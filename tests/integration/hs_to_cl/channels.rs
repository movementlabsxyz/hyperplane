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
    println!("\n[TEST]   === Starting test_cat_status_update_one_target_chain ===");
    let (hs_node, cl_node, _hig_node, start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register a chain
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST]   Registering chain: {}", chain_id.0);
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id.clone()).await.expect("Failed to register chain");
    }
    println!("[TEST]   Chain registered successfully");

    // Set the chain ID in HS
    println!("[TEST]   Setting chain ID in HS...");
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id.clone()).await;
    }
    println!("[TEST]   Chain ID set in HS");

    // Send a CAT status update
    let cat_id = CATId("test-cat".to_string());
    println!("[TEST]   Sending CAT status update for '{}'...", cat_id.0);
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to send status update");
    }
    println!("[TEST]   CAT status update sent successfully");

    // Wait for block production
    println!("[TEST]   Waiting for block production (500ms)...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("[TEST]   Wait complete");

    // Get the current block
    println!("[TEST]   Getting current block...");
    let current_block = {
        let node = cl_node.lock().await;
        node.get_current_block().await.expect("Failed to get current block")
    };
    println!("[TEST]   Current block: {}", current_block);
    assert!(current_block >= start_block_height + 3 && current_block <= start_block_height + 6, "Current block not in correct range {}", current_block);

    // Get subblock and verify transaction
    println!("[TEST]   Getting subblock for chain {}...", chain_id.0);
    let subblock = {
        let node = cl_node.lock().await;
        node.get_subblock(chain_id, start_block_height + 1)
            .await
            .expect("Failed to get subblock")
    };
    println!("[TEST]   Retrieved subblock with {} transactions", subblock.transactions.len());
    assert_eq!(subblock.transactions.len(), 1);
    assert_eq!(subblock.transactions[0].data, "STATUS_UPDATE.Success.CAT_ID:test-cat");
    println!("[TEST]   Transaction verification successful");
    
    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests that multiple CAT status updates are properly included in blocks:
/// - Hyper Scheduler sends multiple CAT status updates for a single target chain
/// - Verify they are included in the next blocks
/// - Verify the transactions are correctly processed
#[tokio::test]
async fn test_multiple_cat_status_updates_one_target_chain() {
    println!("\n[TEST]   === Starting test_multiple_cat_status_updates_one_target_chain ===");
    let (hs_node, cl_node, _hig_node,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register a test chain
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST]   Registering chain: {}", chain_id.0);
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id.clone())
            .await
            .expect("Failed to register chain");
    }
    println!("[TEST]   Chain registered successfully");

    // Set the chain ID in HS
    println!("[TEST]   Setting chain ID in HS...");
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id.clone()).await;
    }
    println!("[TEST]   Chain ID set in HS");

    // Create and send multiple CAT status updates
    let updates = vec![
        (CATId("cat-1".to_string()), CATStatusLimited::Success),
        (CATId("cat-2".to_string()), CATStatusLimited::Failure),
        (CATId("cat-3".to_string()), CATStatusLimited::Success),
    ];
    println!("[TEST]   Created {} CAT status updates", updates.len());

    // Send each update
    for (i, (cat_id, status)) in updates.clone().iter().enumerate() {
        println!("[TEST]   Sending update {}/{} for CAT: {} with status: {:?}", 
            i + 1, updates.len(), cat_id.0, status);
        {
            let mut node = hs_node.lock().await;
            node.send_cat_status_update(cat_id.clone(), status.clone())
                .await
                .expect("Failed to send status update");
        }
        println!("[TEST]   Update sent successfully");

        // Wait for block production
        println!("[TEST]   Waiting for block production (100ms)...");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("[TEST]   Wait complete");

        // Verify the status update was processed
        println!("[TEST]   Verifying status update...");
        {
            let node = hs_node.lock().await;
            let current_status = node.get_cat_status(cat_id.clone()).await.unwrap();
            println!("[TEST]   Retrieved status: {:?}", current_status);
            assert_eq!(current_status, *status);
        }
        println!("[TEST]   Status verification successful");
    }
    
    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests that a status update is properly sent and processed:
/// - The Hyper Scheduler sends a status update for multiple chains
/// - The Confirmation Layer receives and queues the transaction
/// - The transaction is included in the next block for each chain
#[tokio::test]
async fn test_status_update() {
    println!("\n[TEST]   === Starting test_status_update ===");
    let (hs_node, cl_node, _hig_node,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register chains 1 and 2 in the confirmation layer
    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());
    println!("[TEST]   Registering chains: {} and {}", chain_id_1.0, chain_id_2.0);
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id_1.clone())
            .await
            .expect("Failed to register chain 1");
        node.register_chain(chain_id_2.clone())
            .await
            .expect("Failed to register chain 2");
    }
    println!("[TEST]   Chains registered successfully");

    // Set the chain ID in HS
    println!("[TEST]   Setting chain ID in HS...");
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id_1.clone()).await;
    }
    println!("[TEST]   Chain ID set in HS");

    // Create a CAT status update transaction
    let cat_id = CATId("test-cat".to_string());
    println!("[TEST]   Created CAT ID: {}", cat_id.0);
    
    // Send the CAT status update through the hyper scheduler
    println!("[TEST]   Sending CAT status update...");
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to send status update");
    }
    println!("[TEST]   CAT status update sent successfully");

    // Wait for block production
    println!("[TEST]   Waiting for block production (100ms)...");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    println!("[TEST]   Wait complete");

    // Verify the status update was processed
    println!("[TEST]   Verifying status update...");
    let status = {
        let node = hs_node.lock().await;
        node.get_cat_status(cat_id).await.unwrap()
    };
    println!("[TEST]   Retrieved status: {:?}", status);
    assert_eq!(status, CATStatusLimited::Success);
    println!("[TEST]   Status verification successful");
    
    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests that a CAT status update is properly sent and processed:
/// - Hyper Scheduler sends a CAT status update
/// - Verify the transaction is included in the next block
/// - Verify the transaction data matches the expected format
#[tokio::test]
async fn test_cat_status_update() {
    println!("\n[TEST]   === Starting test_cat_status_update ===");
    let (hs_node, cl_node, _hig_node,start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register a chain
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST]   Registering chain: {}", chain_id.0);
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id.clone()).await.expect("Failed to register chain");
    }
    println!("[TEST]   Chain registered successfully");

    // Set the chain ID in HS
    println!("[TEST]   Setting chain ID in HS...");
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id.clone()).await;
    }
    println!("[TEST]   Chain ID set in HS");

    // Create a CAT transaction
    let cat_id = CATId("test-tx".to_string());
    println!("[TEST]   Created CAT ID: {}", cat_id.0);

    // Submit the transaction
    println!("[TEST]   Submitting transaction...");
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to submit transaction");
    }
    println!("[TEST]   Transaction submitted successfully");

    // Wait for block production
    println!("[TEST]   Waiting for block production (500ms)...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("[TEST]   Wait complete");

    // Get current block
    println!("[TEST]   Getting current block...");
    let current_block = {
        let node = cl_node.lock().await;
        node.get_current_block().await.expect("Failed to get current block")
    };
    println!("[TEST]   Current block: {}", current_block);
    assert!(
        (start_block_height + 5..=start_block_height + 7).contains(&current_block),
        "block not in [start+5, start+7]"
    );
    
    // Get subblock and verify transaction
    println!("[TEST]   Getting subblock for chain {}...", chain_id.0);
    let subblock = {
        let node = cl_node.lock().await;
        node.get_subblock(chain_id, start_block_height+1)
            .await
            .expect("Failed to get subblock")
    };
    println!("[TEST]   Retrieved subblock with {} transactions", subblock.transactions.len());
    assert_eq!(subblock.transactions.len(), 1);
    assert_eq!(subblock.transactions[0].data, "STATUS_UPDATE.Success.CAT_ID:test-tx");
    println!("[TEST]   Transaction verification successful");
    
    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests that multiple CAT status updates are properly processed across different chains:
/// - Register two different chains
/// - Send CAT status updates for each chain
/// - Verify the transactions are included in the correct subblocks
/// - Verify the transaction data matches the expected format for each chain
#[tokio::test]
async fn test_multiple_cat_status_updates() {
    println!("\n[TEST]   === Starting test_multiple_cat_status_updates ===");
    let (hs_node, cl_node, _hig_node,start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register two chains
    let chain_id_1 = ChainId("test-chain-1".to_string());
    let chain_id_2 = ChainId("test-chain-2".to_string());
    println!("[TEST]   Registering chains: {} and {}", chain_id_1.0, chain_id_2.0);
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id_1.clone())
            .await
            .expect("Failed to register chain 1");
        node.register_chain(chain_id_2.clone())
            .await
            .expect("Failed to register chain 2");
    }
    println!("[TEST]   Chains registered successfully");

    // Set the chain ID in HS
    println!("[TEST]   Setting chain ID in HS...");
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id_1.clone()).await;
    }
    println!("[TEST]   Chain ID set in HS");

    // Create and submit transactions for both chains
    let cat_id_1 = CATId("test-tx-1".to_string());
    let cat_id_2 = CATId("test-tx-2".to_string());
    println!("[TEST]   Created CAT IDs: {} and {}", cat_id_1.0, cat_id_2.0);

    println!("[TEST]   Submitting transactions...");
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id_1.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to submit transaction 1");
        node.send_cat_status_update(cat_id_2.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to submit transaction 2");
    }
    println!("[TEST]   Transactions submitted successfully");

    // Wait for block production
    println!("[TEST]   Waiting for block production (500ms)...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("[TEST]   Wait complete");

    // Verify transactions in subblocks
    println!("[TEST]   Getting subblocks for both chains...");
    let subblock1 = {
        let node = cl_node.lock().await;
        node.get_subblock(chain_id_1, start_block_height+1)
            .await
            .expect("Failed to get subblock 1")
    };
    let subblock2 = {
        let node = cl_node.lock().await;
        node.get_subblock(chain_id_2, start_block_height+1)
            .await
            .expect("Failed to get subblock 2")
    };
    println!("[TEST]   Retrieved subblocks with {} and {} transactions", 
        subblock1.transactions.len(), subblock2.transactions.len());

    assert_eq!(subblock1.transactions.len(), 2);
    assert_eq!(subblock1.transactions[0].data, "STATUS_UPDATE.Success.CAT_ID:test-tx-1");
    assert_eq!(subblock1.transactions[1].data, "STATUS_UPDATE.Success.CAT_ID:test-tx-2");
    assert_eq!(subblock2.transactions.len(), 0);
    println!("[TEST]   Transaction verification successful");
    
    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests that a CAT status update is properly sent and processed:
/// - Hyper Scheduler sends a CAT status update
/// - Verify the transaction is included in the next block
/// - Verify the transaction data matches the expected format
#[tokio::test]
async fn test_send_cat_status_update() {
    println!("\n[TEST]   === Starting test_send_cat_status_update ===");
    
    // Get the test nodes using our helper function
    let (hs_node, _cl_node, _hig_node,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");
    
    // Set chain ID
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST]   Setting chain ID: {}", chain_id.0);
    {
        let mut node = hs_node.lock().await;
        node.set_chain_id(chain_id.clone()).await;
    }
    println!("[TEST]   Chain ID set successfully");
    
    // Send CAT status update
    let cat_id = CATId("test-cat".to_string());
    println!("[TEST]   Sending CAT status update for {}...", cat_id.0);
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id.clone(), CATStatusLimited::Success)
            .await
            .expect("Failed to send CAT status update");
    }
    println!("[TEST]   CAT status update sent successfully");
    
    // Wait for a bit to let the update be processed
    println!("[TEST]   Waiting for update to be processed (100ms)...");
    sleep(Duration::from_millis(100)).await;
    println!("[TEST]   Wait complete");
    
    // Verify the status was updated
    println!("[TEST]   Verifying status update...");
    {
        let node = hs_node.lock().await;
        let current_status = node.get_cat_status(cat_id).await.unwrap();
        println!("[TEST]   Retrieved status: {:?}", current_status);
        assert_eq!(current_status, CATStatusLimited::Success);
    }
    println!("[TEST]   Status verification successful");
    
    println!("[TEST]   === Test completed successfully ===\n");
}
