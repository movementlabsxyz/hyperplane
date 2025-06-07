#![cfg(feature = "test")]

use hyperplane::{
    types::{TransactionStatus},
    confirmation_layer::ConfirmationLayer,
    hyper_ig::HyperIG,
};
use crate::integration::common::{testnodes, submit_transactions, constants};
use tokio::time::Duration;
use hyperplane::utils::logging;

/// Helper function: Test that a subblock with new transactions is properly processed by the HIG:
/// - Submit a regular transaction to the CL
/// - The CL sends a subblock to the HIG
/// - The HIG processes the transaction in the subblock
/// - Verify the transaction status is correctly set to Pending
async fn run_process_subblock_regular_tx(
    transaction_data: &str,
    expected_status: TransactionStatus,
) {
    logging::log("TEST", "\n=== Starting test_process_subblock ===");
    
    // Initialize components with 100ms block interval
    logging::log("TEST", "Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, hig_node, _, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    // Submit regular transaction using helper function
    let cl_tx = submit_transactions::create_and_submit_regular_transaction(
        &cl_node,
        &constants::chain_1(),
        transaction_data,
        "test-tx"
    ).await.expect("Failed to submit transaction");

    // Wait for block production and processing (150ms to ensure block is produced and processed)
    logging::log("TEST", "Waiting for block production and processing (150ms)...");
    tokio::time::sleep(Duration::from_millis(150)).await;

    // create a local scope (note the test currently fails without this)
    {
        let node = cl_node.lock().await;
        let current_block = node.get_current_block().await.expect("Failed to get current block");
        logging::log("TEST", &format!("Current block height: {}", current_block));
        assert!(current_block >= 1, "No block was produced");
    }

    // Wait for the transaction to be processed (we see it in block 3 in the logs)
    logging::log("TEST", "Waiting for transaction to be processed...");
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify transaction status
    logging::log("TEST", "Verifying transaction status...");
    let node = hig_node.lock().await;
    let status = node.get_transaction_status(cl_tx.id).await.unwrap();
    logging::log("TEST", &format!("Retrieved transaction status: {:?}", status));
    assert_eq!(status, expected_status, "Transaction status is not {:?}", expected_status);
    logging::log("TEST", "Transaction status verification successful");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a subblock with a regular transaction (success) is properly processed by the HIG
#[tokio::test]
async fn test_process_subblock_with_regular_transaction_success() {
    logging::init_logging();
    run_process_subblock_regular_tx("credit 1 100", TransactionStatus::Success).await;
}

/// Tests that a subblock with a regular transaction (failure) is properly processed by the HIG
#[tokio::test]
async fn test_process_subblock_with_regular_transaction_failure() {
    logging::init_logging();
    run_process_subblock_regular_tx("send 1 2 100", TransactionStatus::Failure).await;
}
