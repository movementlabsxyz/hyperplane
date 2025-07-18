#![cfg(feature = "test")]

use hyperplane::{
    types::{CATStatusLimited, TransactionStatus, CATId, CATStatus},
    confirmation_layer::ConfirmationLayer,
    hyper_ig::HyperIG,
    utils::logging,
};
use crate::integration::common::{testnodes, submit_transactions};
use hyperplane::types::constants;
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

    // Submit the CAT transaction
    let cl_tx = submit_transactions::create_and_submit_cat_transaction(
        &cl_node,
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
        let local_cl_node = cl_node.lock().await;
        let current_block = local_cl_node.get_current_block().await.expect("Failed to get current block");
        logging::log("TEST", &format!("Current block height: {}", current_block));
        assert!(current_block >= start_block_height + 1, "No block was produced");
    }

    // Verify that HIG-chain-1 has updated the status of the original CAT transaction
    logging::log("TEST", &format!("Verifying transaction status in HIG-chain-1 for original tx-id='{}'...", cl_tx.id.clone()));
    // Get the transaction ID from the CL transaction
    let tx_id = cl_tx.transactions[0].id.clone();
    let status = {
        let local_hig_node = hig_node_1.lock().await;
        local_hig_node.get_transaction_status(tx_id)
            .await
            .expect("Failed to get transaction status")
    };
    logging::log("TEST", &format!("Transaction status in HIG-chain-1: {:?}", status));
    
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
/// * `chain_1_credit` - Whether to credit chain-1
/// * `chain_2_credit` - Whether to credit chain-2
/// * `transaction_data_in_cat` - The transaction data to test that is wrapped in a CAT transaction
/// * `expected_status` - Expected CAT status (Success or Failure)
async fn run_two_chain_cat_test_with_credits(
    chain_1_credit: bool,
    chain_2_credit: bool,
    transaction_data_in_cat: &str,
    expected_status: CATStatusLimited
) {
    logging::log("TEST", &format!("=== Starting CAT test with credits: chain1={}, chain2={}, cat={} ===", 
        chain_1_credit, chain_2_credit, transaction_data_in_cat));
    
    // Initialize components with 100ms block interval
    logging::log("TEST", "Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, hig_node_1, _hig_node_2, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    // Credit chains if needed
    logging::log("TEST", "Crediting chains...");
    if chain_1_credit {
        submit_transactions::credit_account(&cl_node, &constants::chain_1(), "1").await.expect("Failed to submit credit transaction for chain-1");
    }
    if chain_2_credit {
        submit_transactions::credit_account(&cl_node, &constants::chain_2(), "1").await.expect("Failed to submit credit transaction for chain-2");
    }

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Submit the CAT transaction
    let cl_tx = submit_transactions::create_and_submit_cat_transaction(
        &cl_node,
        transaction_data_in_cat,
        "test-cat"
    ).await.expect("Failed to submit CAT transaction");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify that HIG of chain-1 has updated the status
    let tx_id = cl_tx.transactions[0].id.clone();
    let status = {
        let local_hig_node = hig_node_1.lock().await;
        local_hig_node.get_transaction_status(tx_id)
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
async fn test_cat_with_only_chain_1_credit() {
    timeout(Duration::from_secs(4), run_two_chain_cat_test_with_credits(
        true,  // chain-1 has funds
        false, // chain-2 has no funds
        "send 1 2 50",
        CATStatusLimited::Failure
    ))
    .await
    .expect("Test timed out after 4 seconds");
}

/// Tests that a CAT send succeeds when both chains have funds
#[tokio::test]
async fn test_cat_with_both_chains_credit() {
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

    // wait for 50ms to have one block already produced
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 1. Submit CAT credit transaction
    logging::log("TEST", "Submitting CAT credit transaction...");
    let _cat_tx = submit_transactions::create_and_submit_cat_transaction(
        &cl_node,
        "credit 1 100",
        "cat1"
    ).await.expect("Failed to submit CAT credit transaction");

    // 2. Immediately submit regular send transaction (in same block such that the CAT is still pending)
    logging::log("TEST", "Submitting regular send transaction...");
    let cl_tx_send = submit_transactions::create_and_submit_regular_transaction(
        &cl_node,
        &constants::chain_1(),
        "send 1 2 50",
        "send1"
    ).await.expect("Failed to submit regular send transaction");

    // Wait for block production (100ms block interval, we are now in the middle between blocks) and processing
    // Then check that send is pending after the block was produced 
    tokio::time::sleep(Duration::from_millis(100)).await;

    let tx_id = cl_tx_send.transactions[0].id.clone();
    let initial_status = {
        let local_hig_node = hig_node_1.lock().await;
        local_hig_node.get_transaction_status(tx_id.clone())
            .await
            .expect("Failed to get transaction status")
    };
    assert_eq!(initial_status, TransactionStatus::Pending, "Send transaction should be pending initially");

    // Wait for block production and processing
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check that send succeeded after the cat has been resolved (requires status update going through CL)
    let final_status = {
        let local_hig_node = hig_node_1.lock().await;
        local_hig_node.get_transaction_status(tx_id.clone())
            .await
            .expect("Failed to get transaction status")
    };
    assert_eq!(final_status, TransactionStatus::Success, "Send transaction should succeed after credit");

    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that HIG delays work correctly across multiple chains in an e2e scenario:
/// - Set chain-1 HIG delay to 200ms (slow)
/// - Set chain-2 HIG delay to 0ms (fast)
/// - Submit a CAT transaction
/// - Verify that after 200ms: (100ms block)
///   - Chain-2 HIG has submitted its status
///   - Chain-1 HIG has not submitted its status
///   - CAT is pending in HS
/// - Verify that after 500ms: (100ms block + 200ms delay + 100ms block)
///   - Chain-1 HIG has submitted its status
///   - CAT is processed in HS
#[tokio::test]
async fn test_hig_delays() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting e2e test_hig_delays ===");
    
    // Set up test nodes
    let (hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");
    
    // Set delays for both HIGs - chain-1 is slow, chain-2 is fast
    hig_node_1.lock().await.set_hs_message_delay(Duration::from_millis(200));
    hig_node_2.lock().await.set_hs_message_delay(Duration::from_millis(0));
    logging::log("TEST", "Set HIG-chain-1 delay to 200ms and HIG-chain-2 delay to 0ms");
    
    // Submit a CAT transaction
    logging::log("TEST", "Submitting CAT transaction...");
    let cl_tx = submit_transactions::create_and_submit_cat_transaction(
        &cl_node,
        "credit 1 100",
        "test-cat"
    ).await.expect("Failed to submit CAT transaction");
    logging::log("TEST", "CAT transaction submitted.");

    // Wait 200ms and check status
    logging::log("TEST", "Waiting 200ms before first status check...");
    tokio::time::sleep(Duration::from_millis(200)).await;
    logging::log("TEST", "Checking HS state after 200ms...");

    // Check HS state
    let cat_id = CATId(cl_tx.id.clone());
    let (chain_1_status, chain_2_status, cat_status) = {
        let node_guard = hs_node.lock().await;
        let hs_state = node_guard.state.lock().await;
        (
            hs_state.cat_chainwise_statuses.get(&cat_id)
                .and_then(|statuses| statuses.get(&constants::chain_1())).cloned(),
            hs_state.cat_chainwise_statuses.get(&cat_id)
                .and_then(|statuses| statuses.get(&constants::chain_2())).cloned(),
            hs_state.cat_statuses.get(&cat_id).cloned()
        )
    };

    // Verify that after 200ms:
    // 1. Chain-1 HIG has not submitted its status
    assert!(chain_1_status.is_none(), "Chain-1 HIG should not have submitted its status yet");
    logging::log("TEST", &format!("Chain-1 status after 200ms: {:?}", chain_1_status));

    // 2. Chain-2 HIG has submitted its status
    assert!(chain_2_status.is_some(), "Chain-2 HIG should have submitted its status");
    logging::log("TEST", &format!("Chain-2 status after 200ms: {:?}", chain_2_status));

    // 3. CAT is pending in HS
    logging::log("TEST", &format!("CAT status after 200ms: {:?}", cat_status));
    assert!(cat_status.is_some(), "CAT should be processed in HS");
    assert_eq!(cat_status.unwrap(), CATStatus::Pending, "CAT should be pending");

    logging::log("TEST", "Verified state after 200ms");

    // Wait another 300ms (total 500ms) and check final status
    logging::log("TEST", "Waiting another 300ms before final status check...");
    tokio::time::sleep(Duration::from_millis(300)).await;
    logging::log("TEST", "Checking HS state after 500ms...");

    // Check final HS state
    let (chain_1_status, cat_status) = {
        let node_guard = hs_node.lock().await;
        let hs_state = node_guard.state.lock().await;
        (
            hs_state.cat_chainwise_statuses.get(&cat_id)
                .and_then(|statuses| statuses.get(&constants::chain_1())).cloned(),
            hs_state.cat_statuses.get(&cat_id).cloned()
        )
    };

    // Verify that after 300ms:
    // 1. Chain-1 HIG has submitted its status
    assert!(chain_1_status.is_some(), "Chain-1 HIG should have submitted its status");
    logging::log("TEST", &format!("Chain-1 status after 300ms: {:?}", chain_1_status));

    // 2. CAT is processed in HS
    assert!(cat_status.is_some(), "CAT should be processed in HS");
    logging::log("TEST", &format!("CAT status after 300ms: {:?}", cat_status));

    logging::log("TEST", "Verified final state after 300ms");
    logging::log("TEST", "=== Test completed successfully ===\n");
}

