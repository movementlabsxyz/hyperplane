#![cfg(feature = "test")]

use hyperplane::{
    types::{CATStatusLimited, ChainId, TransactionStatus},
    confirmation_layer::ConfirmationLayer,
    hyper_ig::HyperIG,
    utils::logging,
};
use crate::integration::common::{testnodes, submit_transactions};
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
    let cl_tx = submit_transactions::create_and_submit_cat_transaction(
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
/// * `chain1_credit` - Whether to credit chain-1
/// * `chain2_credit` - Whether to credit chain-2
/// * `transaction_data_in_cat` - The transaction data to test that is wrapped in a CAT transaction
/// * `expected_status` - Expected CAT status (Success or Failure)
async fn run_two_chain_cat_test_with_credits(
    chain1_credit: bool,
    chain2_credit: bool,
    transaction_data_in_cat: &str,
    expected_status: CATStatusLimited
) {
    logging::log("TEST", &format!("=== Starting CAT test with credits: chain1={}, chain2={}, cat={} ===", 
        chain1_credit, chain2_credit, transaction_data_in_cat));
    
    // Initialize components with 100ms block interval
    logging::log("TEST", "Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, hig_node_1, _hig_node_2, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Credit chains if needed
    logging::log("TEST", "Crediting chains...");
    if chain1_credit {
        submit_transactions::credit_account(&cl_node, &chain_id_1, "1").await.expect("Failed to submit credit transaction for chain-1");
    }
    if chain2_credit {
        submit_transactions::credit_account(&cl_node, &chain_id_2, "1").await.expect("Failed to submit credit transaction for chain-2");
    }

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Submit the CAT transaction
    let cl_tx = submit_transactions::create_and_submit_cat_transaction(
        &cl_node,
        &chain_id_1,
        &chain_id_2,
        transaction_data_in_cat,
        "test-cat"
    ).await.expect("Failed to submit CAT transaction");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify that HIG of chain-1 has updated the status
    let status = {
        let node = hig_node_1.lock().await;
        node.get_transaction_status(cl_tx.id.clone())
            .await
            .expect("Failed to get transaction status")
    };
    logging::log("TEST", &format!("Transaction status in HIG of chain-1: {:?}", status));
    
    // The status should match the expected status from the CAT transaction
    let expected_tx_status = match expected_status {
        CATStatusLimited::Success => TransactionStatus::Success,
        CATStatusLimited::Failure => TransactionStatus::Failure,
    };
    assert_eq!(status, expected_tx_status, "Transaction status should match the expected status from CAT transaction. Should: {:?}, Got: {:?}", expected_tx_status, status);
    
    logging::log("TEST", "=== Test completed successfully ===");
}

/// Tests two chain CAT success
#[tokio::test]
async fn test_two_chain_cat_success() {
    timeout(Duration::from_secs(2), run_two_chain_cat_test("credit 1 100", CATStatusLimited::Success))
        .await
        .expect("Test timed out after 2 seconds");
}

/// Tests two chain CAT failure
#[tokio::test]
async fn test_two_chain_cat_failure() {
    // This test fails because we try to send 100 tokens from account 1 to account 2,
    // but account 1 has no balance (it was never credited)
    timeout(Duration::from_secs(2), run_two_chain_cat_test("send 1 2 100", CATStatusLimited::Failure))
        .await
        .expect("Test timed out after 2 seconds");
}

/// Tests that a CAT send fails when only chain-1 has funds
#[tokio::test]
async fn test_cat_send_chain1_only() {
    timeout(Duration::from_secs(4), run_two_chain_cat_test_with_credits(
        true,  // chain-1 has funds
        false,    // chain-2 has no funds
        "send 1 2 50",
        CATStatusLimited::Failure
    ))
    .await
    .expect("Test timed out after 4 seconds");
}

/// Tests that a CAT send succeeds when both chains have funds
#[tokio::test]
async fn test_cat_send_both_chains() {
    timeout(Duration::from_secs(4), run_two_chain_cat_test_with_credits(
        true,  // chain-1 has funds
        true,  // chain-2 has funds
        "send 1 2 50",
        CATStatusLimited::Success
    ))
    .await
    .expect("Test timed out after 4 seconds");
}

/// Tests a sequence of transactions:
/// 1. CAT credit transaction
/// 2. Regular send transaction (should be pending after one block is produced)
/// 3. After 500ms, the send should succeed
#[tokio::test]
async fn test_cat_credit_then_send() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cat_credit_then_send ===");
    
    // Initialize components with 100ms block interval
    logging::log("TEST", "Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, hig_node_1, _hig_node_2, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // wait for 50ms to have one block already produced
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 1. Submit CAT credit transaction
    logging::log("TEST", "Submitting CAT credit transaction...");
    let _cat_tx = submit_transactions::create_and_submit_cat_transaction(
        &cl_node,
        &chain_id_1,
        &chain_id_2,
        "credit 1 100",
        "cat1"
    ).await.expect("Failed to submit CAT credit transaction");

    // 2. Immediately submit regular send transaction (in same block such that the CAT is still pending)
    logging::log("TEST", "Submitting regular send transaction...");
    let send_tx = submit_transactions::create_and_submit_regular_transaction(
        &cl_node,
        &chain_id_1,
        "send 1 2 50",
        "send1"
    ).await.expect("Failed to submit regular send transaction");

    // Wait for block production (100ms block interval, we are now in the middle between blocks) and processing
    // Then check that send is pending after the block was produced 
    tokio::time::sleep(Duration::from_millis(100)).await;

    let initial_status = {
        let node = hig_node_1.lock().await;
        node.get_transaction_status(send_tx.id.clone())
            .await
            .expect("Failed to get transaction status")
    };
    assert_eq!(initial_status, TransactionStatus::Pending, "Send transaction should be pending initially");

    // Wait for block production and processing
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check that send succeeded after the cat has been resolved (requires status update going through CL)
    let final_status = {
        let node = hig_node_1.lock().await;
        node.get_transaction_status(send_tx.id.clone())
            .await
            .expect("Failed to get transaction status")
    };
    assert_eq!(final_status, TransactionStatus::Success, "Send transaction should succeed after credit");

    logging::log("TEST", "=== Test completed successfully ===\n");
}

