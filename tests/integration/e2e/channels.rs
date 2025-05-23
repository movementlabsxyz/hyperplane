#![cfg(feature = "test")]

use hyperplane::{
    types::{TransactionId, CATStatusLimited, ChainId, CLTransaction, TransactionStatus, Transaction},
    confirmation_layer::ConfirmationLayer,
    hyper_ig::HyperIG,
};
use super::super::common::testnodes;
use tokio::time::{Duration, timeout};

// Helper function to run a two chain CAT test
/// - CL: Send a CAT transaction to the CL and produce a block
/// - HIG: Process the CAT transaction (pending) and send a status update to the HS
/// - HS: Process the status update and send a status update to the CL
/// - CL: Include the status update in a block
/// - HIG: Process the status update and update the transaction status (success or failure)
async fn run_two_chain_cat_test(expected_status: CATStatusLimited) {
    println!("\n[TEST]   === Starting CAT test with expected status: {:?} ===", expected_status);
    
    // Initialize components with 100ms block interval
    println!("[TEST]   Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, hig_node_1, _hig_node_2, start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Create a transaction for each chain
    let tx_chain_1 = Transaction::new(
        TransactionId("test-cat".to_string()),
        chain_id_1.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        format!("CAT.SIMULATION:{:?}.CAT_ID:test-cat", expected_status),
    ).expect("Failed to create transaction");

    let tx_chain_2 = Transaction::new(
        TransactionId("test-cat".to_string()),
        chain_id_2.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        format!("CAT.SIMULATION:{:?}.CAT_ID:test-cat", expected_status),
    ).expect("Failed to create transaction");

    let cl_tx = CLTransaction::new(
        TransactionId("test-cat".to_string()),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        vec![tx_chain_1, tx_chain_2],
    ).expect("Failed to create CL transaction");

    println!("[TEST]   Submitting CAT transaction");
    {
        let mut node = cl_node.lock().await;
        node.submit_transaction(cl_tx.clone()).await.expect("Failed to submit transaction");
    }
    println!("[TEST]   CAT transaction submitted successfully");

    // Wait for block production in CL (cat-tx), processing in HIG and HS, and then block production in CL (status-update-tx)
    println!("----------------------------------------------------------------");
    println!("[TEST]   Waiting for 1) block production in CL for CAT and 2) block production in CL for status-update-tx...");
    println!("----------------------------------------------------------------");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify block was produced
    {
        let node = cl_node.lock().await;
        let current_block = node.get_current_block().await.expect("Failed to get current block");
        println!("[TEST]   Current block height: {}", current_block);
        assert!(current_block >= start_block_height + 1, "No block was produced");
    }

    // Verify that HIG has updated the status of the original CAT transaction
    println!("[TEST]   Verifying transaction status in HIG for original tx-id='{}'...", cl_tx.id.clone());
    let status = {
        let node = hig_node_1.lock().await;
        node.get_transaction_status(cl_tx.id.clone())
            .await
            .expect("Failed to get transaction status")
    };
    println!("[TEST]   Transaction status in HIG: {:?}", status);
    
    // The status should match the expected status from the CAT transaction
    let expected_tx_status = match expected_status {
        CATStatusLimited::Success => TransactionStatus::Success,
        CATStatusLimited::Failure => TransactionStatus::Failure,
    };
    assert_eq!(status, expected_tx_status, "Transaction status should match the expected status from CAT transaction");
    
    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests two chain CAT success
#[tokio::test]
async fn test_two_chain_cat_success() {
    timeout(Duration::from_secs(2), run_two_chain_cat_test(CATStatusLimited::Success))
        .await
        .expect("Test timed out after 2 seconds");
}

/// Tests two chain CAT failure
#[tokio::test]
async fn test_two_chain_cat_failure() {
    timeout(Duration::from_secs(2), run_two_chain_cat_test(CATStatusLimited::Failure))
        .await
        .expect("Test timed out after 2 seconds");
}

