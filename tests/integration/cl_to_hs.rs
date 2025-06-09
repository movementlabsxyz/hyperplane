#![cfg(feature = "test")]

use hyperplane::{
    types::{CATId, CATStatus, CLTransactionId},
    confirmation_layer::ConfirmationLayer,
    HyperScheduler,
    utils::logging,
    types::constants,
};
use crate::integration::common::{testnodes, submit_transactions};
use tokio::time::Duration;

/// Helper function: tests sending a CAT status proposal from CL to HS
/// - Submit a cat transaction to CL
/// - Wait for the transaction to be processed by the HIGs
/// - Check that the CAT status is set to the expected status in the HS
async fn run_test_one_cat(transaction_data: &str, expected_status: CATStatus) {
    logging::log("TEST", &format!("\n=== Starting test_one_cat with transaction: {} ===", transaction_data));
    let (hs_node, cl_node, _hig_node_1, _hig_node_2, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");

    // Submit the CAT transaction
    let _cl_tx = submit_transactions::create_and_submit_cat_transaction(
        &cl_node,
        transaction_data,
        "test-cat"
    ).await.expect("Failed to submit CAT transaction");

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
        let status = node.get_cat_status(CATId(CLTransactionId("test-cat".to_string()))).await.expect("Failed to get CAT status");
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
    run_test_one_cat("credit 1 100", CATStatus::Success).await;
}

/// Tests cat (failure) 
#[tokio::test]
async fn test_cat_one_cat_failure() {
    logging::init_logging();
    // the cat should fail because the sender has no balance
    run_test_one_cat("send 1 2 100", CATStatus::Failure).await;
}

/// Tests that HIG delays work correctly across multiple chains:
/// - Set chain-1 HIG delay to 100ms
/// - Set chain-2 HIG delay to 300ms
/// - Submit a CAT transaction
/// - Verify that after 200ms:
///   - Chain-1 HIG has submitted its status
///   - Chain-2 HIG has not submitted its status
///   - CAT is not processed in HS
/// - Verify that after 400ms:
///   - Chain-2 HIG has submitted its status
///   - CAT is processed in HS
#[tokio::test]
async fn test_hig_delays() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_hig_delays ===");
    
    // Set up test nodes
    let (hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    logging::log("TEST", "Test nodes initialized successfully");
    
    // Set delays for both HIGs
    hig_node_1.lock().await.set_hs_message_delay(Duration::from_millis(0));
    hig_node_2.lock().await.set_hs_message_delay(Duration::from_millis(300));
    logging::log("TEST", "Set HIG-chain-1 delay to 0ms and HIG-chain-2 delay to 300ms");
    
    // Submit a CAT transaction
    let cl_tx = submit_transactions::create_and_submit_cat_transaction(
        &cl_node,
        "CAT.credit 1 100",
        "test-cat"
    ).await.expect("Failed to submit CAT transaction");
    logging::log("TEST", "Submitted CAT transaction");
    
    // Wait 200ms and check status
    logging::log("TEST", "Waiting 200ms...");
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Check HS state
    let node_guard = hs_node.lock().await;
    let hs_state = node_guard.state.lock().await;
    let cat_id = CATId(cl_tx.id.clone());
    
    // Verify that after 200ms:
    // 1. Chain-1 HIG has submitted its status
    let chain_1_status = hs_state.cat_chainwise_statuses.get(&cat_id)
        .and_then(|statuses| statuses.get(&constants::chain_1()));
    assert!(chain_1_status.is_some(), "Chain-1 HIG should have submitted its status");
    
    // 2. Chain-2 HIG has not submitted its status
    let chain_2_status = hs_state.cat_chainwise_statuses.get(&cat_id)
        .and_then(|statuses| statuses.get(&constants::chain_2()));
    assert!(chain_2_status.is_none(), "Chain-2 HIG should not have submitted its status yet");
    
    // 3. CAT is not processed in HS
    let cat_status = hs_state.cat_statuses.get(&cat_id);
    assert!(cat_status.is_none(), "CAT should not be processed in HS yet");
    
    logging::log("TEST", "Verified state after 200ms");
    
    // Wait another 200ms (total 400ms) and check final status
    logging::log("TEST", "Waiting another 200ms...");
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Check final HS state
    let node_guard = hs_node.lock().await;
    let hs_state = node_guard.state.lock().await;
    
    // Verify that after 400ms:
    // 1. Chain-2 HIG has submitted its status
    let chain_2_status = hs_state.cat_chainwise_statuses.get(&cat_id)
        .and_then(|statuses| statuses.get(&constants::chain_2()));
    assert!(chain_2_status.is_some(), "Chain-2 HIG should have submitted its status");
    
    // 2. CAT is processed in HS
    let cat_status = hs_state.cat_statuses.get(&cat_id);
    assert!(cat_status.is_some(), "CAT should be processed in HS");
    
    logging::log("TEST", "Verified final state after 400ms");
    logging::log("TEST", "=== Test completed successfully ===\n");
}

