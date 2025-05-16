use hyperplane::{
    types::{ChainId, CATId, CATStatusLimited},
    hyper_scheduler::HyperScheduler,
    confirmation_layer::ConfirmationLayer,
};
use crate::common::testnodes;
use tokio::time::{sleep, Duration};

/// Tests that a single-chain CAT status update is properly included in a block:
/// - HS submits a single-chain CAT status update to CL
/// - Verify it is included in the next block
#[tokio::test]
async fn test_single_chain_cat_status_update() {
    println!("\n[TEST]   === Starting test_single_chain_cat_status_update ===");
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

/// Tests that several single-chain CAT status updates are properly included in blocks:
/// - HS submits several single-chain CAT status updates to CL
/// - Verify they are included in the next blocks
#[tokio::test]
async fn test_several_single_chain_cat_status_updates() {
    println!("\n[TEST]   === Starting test_several_single_chain_cat_status_updates ===");
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


