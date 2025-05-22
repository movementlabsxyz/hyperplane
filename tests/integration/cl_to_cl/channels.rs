#![cfg(feature = "test")]

use hyperplane::{
    types::{TransactionId, StatusLimited, ChainId, CLTransaction, CATStatus},
    confirmation_layer::ConfirmationLayer,
};
use hyperplane::common::testnodes;
use tokio::time::Duration;

/// Helper function to run a two chain CAT test
/// - CL: Send a CAT transaction to the CL and produce a block
/// - HIG: Process the CAT transaction (pending) and send a status update to the HS
/// - HS: Process the status update and send a status update to the CL
/// - CL: Verify the status update
async fn run_two_chain_cat_test(proposed_status: StatusLimited, expected_status: CATStatus) {
    println!("\n[TEST]   === Starting CAT test with proposed status: {:?} ===", proposed_status);
    
    // Initialize components with 100ms block interval
    println!("[TEST]   Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, _hig_node, _, start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register chains
    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id_1.clone()).await.expect("Failed to register chain");
        node.register_chain(chain_id_2.clone()).await.expect("Failed to register chain");
    }
    // Register chain in HS node
    {
        let mut node = _hs_node.lock().await;
        node.register_chain(chain_id_1.clone()).await.expect("Failed to register chain");
        node.register_chain(chain_id_2.clone()).await.expect("Failed to register chain");
    }
    println!("[TEST]   Chain registered successfully");

    // Submit CAT transaction to CL
    let cl_data = format!("CAT.SIMULATION:{:?}.CAT_ID:test-cat", proposed_status);
    let cl_tx = CLTransaction::new(
        TransactionId("test-cat".to_string()),
        vec![ChainId("chain-1".to_string()), ChainId("chain-2".to_string())],
        cl_data.clone()
    ).expect("Failed to create CLTransaction");
    println!("[TEST]   Submitting CAT transaction with ID: {}", cl_tx.id.0);
    {
        let mut node = cl_node.lock().await;
        node.submit_transaction(cl_tx.clone()).await.expect("Failed to submit transaction");
    }
    println!("[TEST]   CAT transaction submitted successfully");

    // Wait for block production in CL (cat-tx), processing in HIG and HS, and then block production in CL (status-update-tx)
    println!("[TEST]   Waiting for block production in CL and processing in HIG and HS (500ms)...");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check the subblocks for a status update
    println!("[TEST]   Verifying transaction status in CL...");

    // Get the subblock from CL
    // make a loop over the subblocks and check if the status update is included
    let status_data = format!("STATUS_UPDATE:{:?}.CAT_ID:test-cat", expected_status);
    let mut found_tx = false;
    for i in 0..20 {
        let subblock = {
            let node = cl_node.lock().await;
            node.get_subblock(chain_id_2.clone(), start_block_height+1+i).await.expect("Failed to get subblock")
        };
        let tx_count = subblock.transactions.len();
        // Find our transaction in the subblock
        for tx in subblock.transactions {
            if tx.data.contains(&status_data) {
                found_tx = true;
                println!("[TEST]   Found status update in subblock: block_id={}, chain_id={}, tx_count={} with tx id:{} and data: {}", 
                    subblock.block_height, subblock.chain_id.0, tx_count, tx.id, tx.data);    
                break;
            }
        }
    }
    assert!(found_tx, "Transaction with data '{}' not found in subblock", cl_data);
    
    println!("[TEST]   === Test completed successfully ===\n");
}

/// Tests single chain CAT success
#[tokio::test]
async fn test_two_chain_cat_success() {
    run_two_chain_cat_test(StatusLimited::Success, CATStatus::Success).await;
}

/// Tests single chain CAT failure
#[tokio::test]
async fn test_two_chain_cat_failure() {
    run_two_chain_cat_test(StatusLimited::Failure, CATStatus::Failure).await;
}
