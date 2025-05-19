use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, ChainId, CLTransaction},
    hyper_ig::HyperIG,
    confirmation_layer::ConfirmationLayer,
};
use tokio::time::Duration;
use crate::common::testnodes;

/// Helper function: Test that a subblock with new transactions is properly processed by the HIG:
/// - Submit a regular transaction to the CL
/// - The CL sends a subblock to the HIG
/// - The HIG processes the transaction in the subblock
/// - Verify the transaction status is correctly set to Pending
async fn run_test_process_subblock(
    transaction_data: &str,
    expected_status: TransactionStatus,
) {
    println!("\n[TEST]   === Starting test_process_subblock ===");
    
    // Initialize components with 100ms block interval
    println!("[TEST]   Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, hig_node, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register chain
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST]   Registering chain: {}", chain_id.0);
    // create a local scope (note the test currently fails without this)
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id.clone()).await.expect("Failed to register chain");
    }
    println!("[TEST]   Chain registered successfully");

    // Submit regulartransaction to CL
    let tx = Transaction::new(
        TransactionId("test-tx".to_string()),
        transaction_data.to_string()
    ).expect("Failed to create transaction");
    println!("[TEST]   Submitting transaction with ID: {}", tx.id.0);
    // create a local scope (note the test currently fails without this)
    {
        let mut node = cl_node.lock().await;
        node.submit_transaction(CLTransaction::new(
            tx.id.clone(),
            chain_id.clone(),
            tx.data.clone()
        ).expect("Failed to create CLTransaction")).await.expect("Failed to submit transaction");
    }
    println!("[TEST]   Transaction submitted successfully");

    // Wait for block production and processing (150ms to ensure block is produced and processed)
    println!("[TEST]   Waiting for block production and processing (150ms)...");
    tokio::time::sleep(Duration::from_millis(150)).await;

    // create a local scope (note the test currently fails without this)
    {
        let node = cl_node.lock().await;
        let current_block = node.get_current_block().await.expect("Failed to get current block");
        println!("[TEST]   Current block height: {}", current_block);
        assert!(current_block >= 1, "No block was produced");
    }

    // Wait for the transaction to be processed (we see it in block 3 in the logs)
    println!("[TEST]   Waiting for transaction to be processed...");
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify transaction status
    println!("[TEST]   Verifying transaction status...");
    let node = hig_node.lock().await;
    let status = node.get_transaction_status(tx.id).await.unwrap();
    println!("[TEST]   Retrieved transaction status: {:?}", status);
    assert_eq!(status, expected_status, "Transaction status is not {:?}", expected_status);
    println!("[TEST]   Transaction status verification successful");
    
    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests that a subblock with a regular transaction (success) is properly processed by the HIG
#[tokio::test]
async fn test_process_subblock_with_regular_transaction_success() {
    run_test_process_subblock("REGULAR.SIMULATION.Success", TransactionStatus::Success).await;
}

/// Tests that a subblock with a regular transaction (failure) is properly processed by the HIG
#[tokio::test]
async fn test_process_subblock_with_regular_transaction_failure() {
    run_test_process_subblock("REGULAR.SIMULATION.Failure", TransactionStatus::Failure).await;
}

/// Tests that a subblock with a CAT transaction is properly processed by the HIG
#[tokio::test]
async fn test_process_subblock_with_cat_transaction() {
    run_test_process_subblock("CAT.SIMULATION.Success.CAT_ID:test-cat", TransactionStatus::Pending).await;
}

