#![cfg(feature = "test")]

use hyperplane::{
    types::{CATStatusLimited, CATId},
    confirmation_layer::ConfirmationLayer,
    hyper_ig::HyperIG,
    utils::logging,
};
use crate::integration::common::testnodes;
use hyperplane::types::constants;
use tokio::time::Duration;

/// Tests that the HIG waits for all proposals before sending a status update:
/// - HIG-1 sends Success proposal
/// - HIG-2 sends Failure proposal
/// - HS should wait for both proposals
/// - HS should send only one status update to CL with Failure status
#[tokio::test]
async fn test_hs_waits_for_all_proposals() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_hs_waits_for_all_proposals ===");
    let (_hs_node, cl_node, mut hig_node_1, mut hig_node_2, start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    // Create a CAT transaction
    let cat_id = CATId("test-cat".to_string());
    let constituent_chains = vec![constants::chain_1(), constants::chain_2()];
    logging::log("TEST", &format!("Created CAT '{}' with chains {:?}", cat_id.0, constituent_chains));

    // Send proposals from both HIGs
    logging::log("TEST", "Sending proposals from both HIGs...");
    
    // HIG-1 sends Success proposal
    HyperIG::send_cat_status_proposal(
        &mut hig_node_1,
        cat_id.clone(),
        CATStatusLimited::Success,
        constituent_chains.clone()
    ).await.expect("Failed to send proposal from HIG-1");
    logging::log("TEST", "Success proposal sent from HIG-1");

    // HIG-2 sends Failure proposal
    HyperIG::send_cat_status_proposal(
        &mut hig_node_2,
        cat_id.clone(),
        CATStatusLimited::Failure,
        constituent_chains.clone()
    ).await.expect("Failed to send proposal from HIG-2");
    logging::log("TEST", "Failure proposal sent from HIG-2");

    // Wait for block production and processing
    logging::log("TEST", "Waiting for block production and processing (500ms)...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    logging::log("TEST", "Wait complete");

    // Verify that only one status update was sent to CL with Failure status
    logging::log("TEST", "Verifying status updates in blocks...");
    let mut found_success = false;
    let mut found_failure = false;
    for block_id in start_block_height + 1..=start_block_height + 6 {
        let subblock = {
            let node = cl_node.lock().await;
            node.get_subblock(constants::chain_1(), block_id)
                .await
                .expect("Failed to get subblock")
        };
        logging::log("TEST", &format!("Checking block {} with {} transactions", block_id, subblock.transactions.len()));
        // print the transactions in the subblock
        for tx in &subblock.transactions {
            logging::log("TEST", &format!("Transaction: {:?}", tx));
        }
        
        for tx in &subblock.transactions {
            if tx.data == format!("STATUS_UPDATE:Success.CAT_ID:{}", cat_id.0) {
                found_success = true;
            }
            if tx.data == format!("STATUS_UPDATE:Failure.CAT_ID:{}", cat_id.0) {
                found_failure = true;
            }
        }
    }

    // Verify that only the Failure status update was sent
    assert!(!found_success, "Found Success status update when it should not have been sent");
    assert!(found_failure, "Did not find Failure status update");
    logging::log("TEST", "Status update verification successful");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
} 