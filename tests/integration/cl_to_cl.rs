#![cfg(feature = "test")]

use hyperplane::{
    types::{CATStatus},
    confirmation_layer::ConfirmationLayer,
    utils::logging,
};
use crate::integration::common::{testnodes, submit_transactions};
use hyperplane::types::constants;
use tokio::time::Duration;

/// Helper function to run a two chain CAT test
/// - CL: Send a CAT transaction to the CL and produce a block
/// - HIG: Process the CAT transaction (pending) and send a status update to the HS
/// - HS: Process the status update and send a status update to the CL
/// - CL: Verify the status update
async fn run_two_chain_cat_test(transaction_data: &str, expected_status: CATStatus) {
    logging::log("TEST", &format!("\n=== Starting CAT test with transaction: {} ===", transaction_data));
    
    // Initialize components with 100ms block interval
    logging::log("TEST", "Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, _hig_node, _, start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    logging::log("TEST", &format!("Using chains: {} and {}", constants::CHAIN_1, constants::CHAIN_2));

    // Submit the CAT transaction
    let _cl_tx = submit_transactions::create_and_submit_cat_transaction(
        &cl_node,
        transaction_data,
        "test-cat"
    ).await.expect("Failed to submit CAT transaction");

    // Wait for block production in CL (cat-tx), processing in HIG and HS, and then block production in CL (status-update-tx)
    logging::log("TEST", "Waiting for block production in CL and processing in HIG and HS (500ms)...");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check the subblocks for a status update
    logging::log("TEST", "Verifying transaction status in CL...");

    // Get the subblock from CL
    // make a loop over the subblocks and check if the status update is included
    let status_data = format!("STATUS_UPDATE:{:?}.CAT_ID:test-cat", expected_status);
    let mut found_tx = false;
    for i in 0..20 {
        let subblock = {
            let node = cl_node.lock().await;
            node.get_subblock(constants::chain_2(), start_block_height+1+i).await.expect("Failed to get subblock")
        };
        let tx_count = subblock.transactions.len();
        // Find our transaction in the subblock
        for tx in subblock.transactions {
            if tx.data.contains(&status_data) {
                found_tx = true;
                logging::log("TEST", &format!("Found status update in subblock: block_id={}, chain_id={}, tx_count={} with tx id:{} and data: {}", 
                    subblock.block_height, subblock.chain_id.0, tx_count, tx.id, tx.data));    
                break;
            }
        }
    }
    assert!(found_tx, "Transaction with data '{}' not found in subblock", transaction_data);
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests single chain CAT success
#[tokio::test]
async fn test_two_chain_cat_success() {
    logging::init_logging();
    run_two_chain_cat_test("credit 1 100", CATStatus::Success).await;
}

/// Tests single chain CAT failure
#[tokio::test]
async fn test_two_chain_cat_failure() {
    logging::init_logging();
    // the cat should fail because the sender has no balance
    run_two_chain_cat_test("send 1 2 100", CATStatus::Failure).await;
}
