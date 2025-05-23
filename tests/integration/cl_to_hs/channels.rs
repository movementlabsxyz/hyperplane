#![cfg(feature = "test")]

use hyperplane::{
    types::{TransactionId, CATStatusLimited, ChainId, CLTransaction, CATId, CATStatus},
    confirmation_layer::ConfirmationLayer,
    HyperScheduler,
};
use super::super::common::testnodes;
use tokio::time::Duration;
use tokio::sync::mpsc;

/// Helper function: tests sending a CAT status proposal from CL to HS
/// - Submit a cat transaction to CL
/// - Wait for the transaction to be processed by the HIGs
/// - Check that the CAT status is set to the expected status in the HS
async fn run_test_one_cat(proposed_status: CATStatusLimited, expected_status: CATStatus) {
    println!("\n[TEST]   === Starting test_one_cat ===");
    let (hs_node, cl_node, _, _, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register chain in CL
    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());
    println!("[TEST]   Registering chains: {} and {}", chain_id_1.0, chain_id_2.0);
    {
        let mut cl_node_guard = cl_node.lock().await;
        let (sender_1, _receiver_1) = mpsc::channel(10);
        let (sender_2, _receiver_2) = mpsc::channel(10);
        cl_node_guard.register_chain(chain_id_1.clone(), sender_1).await.expect("Failed to register chain");
        cl_node_guard.register_chain(chain_id_2.clone(), sender_2).await.expect("Failed to register chain");
    }
    println!("[TEST]   Chains registered successfully");

    // Create a CAT transaction
    let cat_id = CATId("test-cat".to_string());
    let cl_tx = CLTransaction::new(
        TransactionId("test-tx".to_string()),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        format!("CAT.SIMULATION:{:?}.CAT_ID:{}", proposed_status, cat_id.0)
    ).expect("Failed to create CLTransaction");

    // Submit the transaction to CL
    println!("[TEST]   Submitting transaction to CL...");
    // create a local scope (note the test fails without this)
    {
        let mut node = cl_node.lock().await;
        node.submit_transaction(cl_tx.clone()).await.expect("Failed to submit transaction");
    }
    println!("[TEST]   Transaction submitted successfully");

    // Wait for message processing
    println!("[TEST]   Waiting for message processing (200ms)...");
    tokio::time::sleep(Duration::from_millis(200)).await;
    println!("[TEST]   Wait complete");

    // create a local scope (note the test fails without this)
    {
        let node = hs_node.lock().await;
        let status = node.get_cat_status(cat_id).await.expect("Failed to get CAT status");
        println!("[TEST]   Retrieved status: {:?}", status);
        assert_eq!(status, expected_status, "CAT status should be {:?}", expected_status);
    }
    println!("[TEST]   Status verification successful");

    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests cat (success)
#[tokio::test]
async fn test_cat_one_cat_success() {
    run_test_one_cat(CATStatusLimited::Success, CATStatus::Success).await;
}

/// Tests cat (failure) 
#[tokio::test]
async fn test_cat_one_cat_failure() {
    run_test_one_cat(CATStatusLimited::Failure, CATStatus::Failure).await;
}

