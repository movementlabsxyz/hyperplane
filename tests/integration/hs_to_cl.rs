#![cfg(feature = "test")]

use hyperplane::{
    types::{CATStatusLimited, CATId},
    confirmation_layer::ConfirmationLayer,
    hyper_scheduler::HyperScheduler,
    utils::logging,
};
use crate::integration::common::testnodes;
use hyperplane::types::constants;
use tokio::time::Duration;

/// Tests that a single-chain CAT status update is properly included in a block:
/// - HS submits a single-chain CAT status update to CL
/// - Verify it is included in the next block
#[tokio::test]
async fn test_single_chain_cat_status_update() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_single_chain_cat_status_update ===");
    let (hs_node, cl_node, _hig_node, _, start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    // Send a CAT status update
    let cat_id = CATId("test-cat".to_string());
    logging::log("TEST", &format!("Sending CAT status update for '{}'...", cat_id.0));
    {
        let mut node = hs_node.lock().await;
        node.send_cat_status_update(cat_id.clone(), vec![constants::chain_1()], CATStatusLimited::Success)
            .await
            .expect("Failed to send status update");
    }
    logging::log("TEST", "CAT status update sent successfully");

    // Wait for block production
    logging::log("TEST", "Waiting for block production (500ms)...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    logging::log("TEST", "Wait complete");

    // Get the current block
    logging::log("TEST", "Getting current block...");
    let current_block = {
        let node = cl_node.lock().await;
        node.get_current_block().await.expect("Failed to get current block")
    };
    logging::log("TEST", &format!("Current block: {}", current_block));
    assert!(current_block >= start_block_height + 3 && current_block <= start_block_height + 6, "Current block not in correct range {}", current_block);

    // Get subblock and verify transaction
    logging::log("TEST", &format!("Getting subblock for chain {}...", constants::CHAIN_1));
    let subblock = {
        let node = cl_node.lock().await;
        node.get_subblock(constants::chain_1(), start_block_height + 1)
            .await
            .expect("Failed to get subblock")
    };
    logging::log("TEST", &format!("Retrieved subblock with {} transactions", subblock.transactions.len()));
    assert_eq!(subblock.transactions.len(), 1);
    assert_eq!(subblock.transactions[0].data, "STATUS_UPDATE:Success.CAT_ID:test-cat");
    logging::log("TEST", "Transaction verification successful");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that several single-chain CAT status updates are properly included in blocks:
/// - HS submits several single-chain CAT status updates to CL
/// - Verify they are included in the next blocks
#[tokio::test]
async fn test_several_single_chain_cat_status_updates() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_several_single_chain_cat_status_updates ===");
    let (hs_node, cl_node, _hig_node, _, start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    // Create and send multiple CAT status updates
    let updates = vec![
        (CATId("cat-1".to_string()), CATStatusLimited::Success),
        (CATId("cat-2".to_string()), CATStatusLimited::Failure),
        (CATId("cat-3".to_string()), CATStatusLimited::Success),
    ];
    logging::log("TEST", &format!("Created {} CAT status updates", updates.len()));

    // Send each update
    for (i, (cat_id, status)) in updates.clone().iter().enumerate() {
        logging::log("TEST", &format!("Sending update {}/{} for CAT: {} with status: {:?}", 
            i + 1, updates.len(), cat_id.0, status));
        {
            let mut node = hs_node.lock().await;
            node.send_cat_status_update(cat_id.clone(), vec![constants::chain_1()], status.clone())
                .await
                .expect("Failed to send status update");
        }
        logging::log("TEST", "Update sent successfully");

        // Wait for block production
        logging::log("TEST", "Waiting for block production (300ms)...");
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        logging::log("TEST", "Wait complete");

        // Verify the status update was included in the block
        logging::log("TEST", "Verifying status update was included in the block...");
        let mut found = false;
        for block_id in start_block_height + 1..=start_block_height + 9 {
            let subblock = {
                let node = cl_node.lock().await;
                node.get_subblock(constants::chain_1(), block_id)
                    .await
                    .expect("Failed to get subblock")
            };
            logging::log("TEST", &format!("Checking block {} with {} transactions", block_id, subblock.transactions.len()));
            if subblock.transactions.iter().any(|tx| tx.data == format!("STATUS_UPDATE:{:?}.CAT_ID:{}", status, cat_id.0)) {
                found = true;
                break;
            }
        }
        assert!(found, "Status update not found in any block");
        logging::log("TEST", "Status update verification successful");
    }

    logging::log("TEST", "=== Test completed successfully ===\n");
}




