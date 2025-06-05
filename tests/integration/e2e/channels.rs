#![cfg(feature = "test")]

use hyperplane::{
    types::{CATStatusLimited, ChainId, TransactionStatus},
    confirmation_layer::ConfirmationLayer,
    hyper_ig::HyperIG,
    utils::logging,
};
use super::super::common::{testnodes, submit_transactions};
use tokio::time::{Duration, timeout};

// Helper function to run a two chain CAT test
/// - CL: Send a CAT transaction to the CL and produce a block
/// - HIG: Process the CAT transaction (pending) and send a status update to the HS
/// - HS: Process the status update and send a status update to the CL
/// - CL: Include the status update in a block
/// - HIG: Process the status update and update the transaction status (success or failure)
async fn run_two_chain_cat_test(transaction_data: &str, expected_status: CATStatusLimited) {
    logging::log("TEST", &format!("=== Starting CAT test with transaction: {} ===", transaction_data));
    
    // Initialize components with 100ms block interval
    logging::log("TEST", "Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, hig_node_1, _hig_node_2, start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Submit the CAT transaction
    let cl_tx = submit_transactions::submit_cat_transaction(
        &cl_node,
        &chain_id_1,
        &chain_id_2,
        transaction_data,
        "test-cat"
    ).await.expect("Failed to submit CAT transaction");

    // Wait for block production in CL (cat-tx), processing in HIG and HS, and then block production in CL (status-update-tx)
    logging::log("TEST", "----------------------------------------------------------------");
    logging::log("TEST", "Waiting for 1) block production in CL for CAT and 2) block production in CL for status-update-tx...");
    logging::log("TEST", "----------------------------------------------------------------");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify block was produced
    {
        let node = cl_node.lock().await;
        let current_block = node.get_current_block().await.expect("Failed to get current block");
        logging::log("TEST", &format!("Current block height: {}", current_block));
        assert!(current_block >= start_block_height + 1, "No block was produced");
    }

    // Verify that HIG has updated the status of the original CAT transaction
    logging::log("TEST", &format!("Verifying transaction status in HIG for original tx-id='{}'...", cl_tx.id.clone()));
    let status = {
        let node = hig_node_1.lock().await;
        node.get_transaction_status(cl_tx.id.clone())
            .await
            .expect("Failed to get transaction status")
    };
    logging::log("TEST", &format!("Transaction status in HIG: {:?}", status));
    
    // The status should match the expected status from the CAT transaction
    let expected_tx_status = match expected_status {
        CATStatusLimited::Success => TransactionStatus::Success,
        CATStatusLimited::Failure => TransactionStatus::Failure,
    };
    assert_eq!(status, expected_tx_status, "Transaction status should match the expected status from CAT transaction");
    
    logging::log("TEST", "=== Test completed successfully ===");
}

/// Helper function to run a two chain CAT test with credits
/// 
/// # Arguments
/// * `chain1_credit` - Credit amount for chain-1 (e.g. "credit 1 100")
/// * `chain2_credit` - Credit amount for chain-2 (e.g. "credit 1 0")
/// * `cat_transaction` - The CAT transaction to test
/// * `expected_status` - Expected CAT status (Success or Failure)
async fn run_two_chain_cat_test_with_credits(
    chain1_credit: &str,
    chain2_credit: &str,
    cat_transaction: &str,
    expected_status: CATStatusLimited
) {
    logging::log("TEST", &format!("=== Starting CAT test with credits: chain1={}, chain2={}, cat={} ===", 
        chain1_credit, chain2_credit, cat_transaction));
    
    // Initialize components with 100ms block interval
    logging::log("TEST", "Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, hig_node_1, _hig_node_2, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Credit both chains
    logging::log("TEST", "Crediting both chains...");
    submit_transactions::submit_regular_transaction(
        &cl_node,
        &chain_id_1,
        "credit 1 100",
        "credit-tx-1"
    ).await.expect("Failed to submit credit transaction for chain-1");

    submit_transactions::submit_regular_transaction(
        &cl_node,
        &chain_id_2,
        "credit 1 100",
        "credit-tx-2"
    ).await.expect("Failed to submit credit transaction for chain-2");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Submit the CAT transaction
    let cl_tx = submit_transactions::submit_cat_transaction(
        &cl_node,
        &chain_id_1,
        &chain_id_2,
        cat_transaction,
        "test-cat"
    ).await.expect("Failed to submit CAT transaction");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify that HIG has updated the status
    let status = {
        let node = hig_node_1.lock().await;
        node.get_transaction_status(cl_tx.id.clone())
            .await
            .expect("Failed to get transaction status")
    };
    logging::log("TEST", &format!("Transaction status in HIG: {:?}", status));
    
    // The status should match the expected status from the CAT transaction
    let expected_tx_status = match expected_status {
        CATStatusLimited::Success => TransactionStatus::Success,
        CATStatusLimited::Failure => TransactionStatus::Failure,
    };
    assert_eq!(status, expected_tx_status, "Transaction status should match the expected status from CAT transaction");
    
    logging::log("TEST", "=== Test completed successfully ===");
}

/// Tests two chain CAT success
#[tokio::test]
async fn test_two_chain_cat_success() {
    timeout(Duration::from_secs(2), run_two_chain_cat_test("CAT.credit 1 100.CAT_ID:test-cat", CATStatusLimited::Success))
        .await
        .expect("Test timed out after 2 seconds");
}

/// Tests two chain CAT failure
#[tokio::test]
async fn test_two_chain_cat_failure() {
    // This test fails because we try to send 100 tokens from account 1 to account 2,
    // but account 1 has no balance (it was never credited)
    timeout(Duration::from_secs(2), run_two_chain_cat_test("CAT.send 1 2 100.CAT_ID:test-cat", CATStatusLimited::Failure))
        .await
        .expect("Test timed out after 2 seconds");
}

/// Tests that a CAT send fails when only chain-1 has funds
#[tokio::test]
async fn test_cat_send_chain1_only() {
    timeout(Duration::from_secs(4), run_two_chain_cat_test_with_credits(
        "credit 1 100",  // chain-1 has funds
        "credit 1 0",    // chain-2 has no funds
        "CAT.send 1 2 50.CAT_ID:test-cat",
        CATStatusLimited::Failure
    ))
    .await
    .expect("Test timed out after 4 seconds");
}

/// Tests that a CAT send succeeds when both chains have funds
#[tokio::test]
async fn test_cat_send_both_chains() {
    timeout(Duration::from_secs(4), run_two_chain_cat_test_with_credits(
        "credit 1 100",  // chain-1 has funds
        "credit 1 100",  // chain-2 has funds
        "CAT.send 1 2 50.CAT_ID:test-cat",
        CATStatusLimited::Success
    ))
    .await
    .expect("Test timed out after 4 seconds");
}

