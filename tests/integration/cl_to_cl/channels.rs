use hyperplane::{
    types::{Transaction, TransactionId, ChainId, CLTransaction},
    confirmation_layer::ConfirmationLayer,
};
use tokio::time::Duration;
use crate::common::testnodes;

/// Tests the full flow of a CAT transaction:
/// 1. CL sends CAT transaction to HIG
/// 2. HIG processes it and sends status proposal to HS
/// 3. HS processes the proposal and sends status update back to CL
/// 4. Verify the final status in CL
#[tokio::test]
async fn test_cat_transaction_flow() {
    println!("\n[TEST]   === Starting test_cat_transaction_flow ===");
    
    // Initialize components with 100ms block interval
    println!("[TEST]   Setting up test nodes with 100ms block interval...");
    let (_hs_node, cl_node, _hig_node, start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    println!("[TEST]   Test nodes initialized successfully");

    // Register chain
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST]   Registering chain: {}", chain_id.0);
    {
        let mut node = cl_node.lock().await;
        node.register_chain(chain_id.clone()).await.expect("Failed to register chain");
    }
    // Register chain in HS node
    {
        let mut node = _hs_node.lock().await;
        node.set_chain_id(chain_id.clone()).await;
    }
    println!("[TEST]   Chain registered successfully");

    // Submit CAT transaction to CL
    let tx = Transaction {
        id: TransactionId("test-cat".to_string()),
        data: "CAT.SIMULATION.Success:test-cat".to_string(),
    };
    println!("[TEST]   Submitting CAT transaction with ID: {}", tx.id.0);
    {
        let mut node = cl_node.lock().await;
        node.submit_transaction(CLTransaction {
            id: tx.id.clone(),
            data: tx.data.clone(),
            chain_id: chain_id.clone(),
        }).await.expect("Failed to submit transaction");
    }
    println!("[TEST]   CAT transaction submitted successfully");

    // Wait for block production in CL (cat-tx), processing in HIG and HS, and then block production in CL (status-update-tx)
    println!("[TEST]   Waiting for block production in CL and processing in HIG and HS (500ms)...");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify block was produced
    {
        let node = cl_node.lock().await;
        let current_block = node.get_current_block().await.expect("Failed to get current block");
        println!("[TEST]   Current block height: {}", current_block);
        assert!(current_block >= start_block_height + 1, "No block was produced");
    }

    // Check the subblocks for a status update
    println!("[TEST]   Verifying transaction status in CL...");

    // Get the subblock from CL
    // make a loop over the subblocks and check if the status update is included
    let mut found_tx = false;
    for i in 0..20 {
        let subblock = {
            let node = cl_node.lock().await;
            node.get_subblock(chain_id.clone(), start_block_height+1+i).await.expect("Failed to get subblock")
        };
        let tx_count = subblock.transactions.len();
        // Find our transaction in the subblock
        for tx in subblock.transactions {
            if tx.data.contains("STATUS_UPDATE.Success.CAT_ID:test-cat") {
                found_tx = true;
                println!("[TEST]   Found status update in subblock: block_id={}, chain_id={}, tx_count={} with tx id:{} and data: {}", 
                    subblock.block_id, subblock.chain_id.0, tx_count, tx.id, tx.data);    
                break;
            }
        }
    }
    assert!(found_tx, "Transaction not found in subblock");
    
    println!("[TEST]   === Test completed successfully ===\n");
}
