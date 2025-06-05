#![cfg(feature = "test")]

use hyperplane::{
    types::{TransactionId, ChainId, CLTransaction, CATId, CATStatus, Transaction},
    confirmation_layer::ConfirmationLayer,
    HyperScheduler,
    utils::logging,
};
use super::super::common::testnodes;
use tokio::time::Duration;

/// Helper function: tests sending a CAT status proposal from CL to HS
/// - Submit a cat transaction to CL
/// - Wait for the transaction to be processed by the HIGs
/// - Check that the CAT status is set to the expected status in the HS
async fn run_test_one_cat(transaction_data: &str, expected_status: CATStatus) {
    logging::log("TEST", &format!("\n=== Starting test_one_cat with transaction: {} ===", transaction_data));
    let (hs_node, cl_node, _hig_node_1, _hig_node_2, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Create a CAT transaction
    let cat_id = CATId("test-cat".to_string());
    let tx_chain_1 = Transaction::new(
        TransactionId("test-tx".to_string()),
        chain_id_1.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        transaction_data.to_string(),
    ).expect("Failed to create transaction");
    let tx_chain_2 = Transaction::new(
        TransactionId("test-tx".to_string()),
        chain_id_2.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        transaction_data.to_string(),
    ).expect("Failed to create transaction");

    let cl_tx = CLTransaction::new(
        TransactionId("test-tx".to_string()),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        vec![tx_chain_1, tx_chain_2],
    ).expect("Failed to create CLTransaction");

    // Submit the transaction to CL
    logging::log("TEST", "Submitting transaction to CL...");
    // create a local scope (note the test fails without this)
    {
        let mut node = cl_node.lock().await;
        node.submit_transaction(cl_tx.clone()).await.expect("Failed to submit transaction");
    }
    logging::log("TEST", "Transaction submitted successfully");

    // Wait for block production in CL (cat-tx), processing in HIG and HS, and then block production in CL (status-update-tx)
    logging::log("TEST", "Waiting for block production and processing (200ms)...");
    tokio::time::sleep(Duration::from_millis(200)).await;
    logging::log("TEST", "Wait complete");

    // Verify block was produced
    {
        let node = cl_node.lock().await;
        let current_block = node.get_current_block().await.expect("Failed to get current block");
        logging::log("TEST", &format!("Current block height: {}", current_block));
        assert!(current_block >= 1, "No block was produced");
    }

    // Wait to make logs more readable
    tokio::time::sleep(Duration::from_millis(400)).await;

    // Verify the CAT status in HS
    logging::log("TEST", "Verifying CAT status in HS...");
    {
        let node = hs_node.lock().await;
        let status = node.get_cat_status(cat_id).await.expect("Failed to get CAT status");
        logging::log("TEST", &format!("Retrieved status: {:?}", status));
        assert_eq!(status, expected_status, "CAT status should be {:?}", expected_status);
    }
    logging::log("TEST", "Status verification successful");

    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests cat (success)
#[tokio::test]
async fn test_cat_one_cat_success() {
    logging::init_logging();
    run_test_one_cat("CAT.credit 1 100.CAT_ID:test-cat", CATStatus::Success).await;
}

/// Tests cat (failure) 
#[tokio::test]
async fn test_cat_one_cat_failure() {
    logging::init_logging();
    // the cat should fail because the sender has no balance
    run_test_one_cat("CAT.send 1 2 100.CAT_ID:test-cat", CATStatus::Failure).await;
}

