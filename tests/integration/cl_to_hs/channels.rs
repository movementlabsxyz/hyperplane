use hyperplane::{
    types::{ChainId, CATId, CATStatusLimited, TransactionId, CLTransaction, Transaction},
    hyper_scheduler::{HyperScheduler},
    confirmation_layer::ConfirmationLayer,
};
use tokio::time::Duration;
use crate::common::testnodes;

/// Helper function: tests sending a CAT status proposal from CL to HS for single chain
/// - Submit a cat transaction to CL
/// - CL proposes a block with a Success status for a CAT
/// - HIG receives the block, processes the transaction, and proposes a status update for the CAT
/// - HS receives and stores the status
async fn run_test_single_chain_cat_success(expected_status: CATStatusLimited) {
    println!("\n[TEST]   === Starting test_single_chain_cat_success ===");
    let (hs_node, cl_node, _hig_node, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register chain in CL
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST]   Registering chain: {}", chain_id.0);
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id.clone()).await.expect("Failed to register chain");
    }
    println!("[TEST]   Chain registered successfully");

    // Create a CAT transaction with simulation success
    let cat_id = CATId("test-cat-tx".to_string());
    let tx = Transaction {
        id: TransactionId("cat-tx.chain-1".to_string()),
        data: format!("CAT.SIMULATION.{:?}.CAT_ID:{}", expected_status, cat_id.0),
    };

    // Submit the transaction to CL
    println!("[TEST]   Submitting transaction to CL...");
    // create a local scope (note the test fails without this)
    {
        let mut node = cl_node.lock().await;
        node.submit_transaction(CLTransaction {
            id: tx.id.clone(),
            data: tx.data.clone(),
            chain_id: chain_id.clone(),
        }).await.expect("Failed to submit transaction");
    }
    println!("[TEST]   Transaction submitted successfully");

    // Wait for message processing
    println!("[TEST]   Waiting for message processing (200ms)...");
    tokio::time::sleep(Duration::from_millis(200)).await;
    println!("[TEST]   Wait complete");

    // Verify the CAT status was updated in HS
    println!("[TEST]   Verifying CAT status in HS...");
    // create a local scope (note the test fails without this)
    {
        let node = hs_node.lock().await;
        let status = node.get_cat_status(cat_id).await.expect("Failed to get CAT status");
        println!("[TEST]   Retrieved status: {:?}", status);
        assert_eq!(status, expected_status);
    }
    println!("[TEST]   Status verification successful");

    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests cat (success) for single chain 
#[tokio::test]
async fn test_single_chain_cat_success() {
    run_test_single_chain_cat_success(CATStatusLimited::Success).await;
}

/// Tests cat (failure) for single chain 
#[tokio::test]
async fn test_single_chain_cat_failure() {
    run_test_single_chain_cat_success(CATStatusLimited::Failure).await;
}

