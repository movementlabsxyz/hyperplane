#![cfg(feature = "test")]

use tokio::time::{Duration, sleep};
use tokio::sync::mpsc;
use hyperplane::{
    types::{TransactionId, ChainId, CLTransaction},
    confirmation_layer::ConfirmationLayer,
};
use super::super::integration::common::testnodes;

// - - - - - - - - - - - - - - - - - - - - - - - 
// V13: Integrates closer to actual node setup
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V13: Integrates closer to actual node setup
#[tokio::test]
async fn test_v13() {
    println!("\n=== Starting test_v13 ===");
    
    // Get the test nodes using our new helper function
    let (hs_node, cl_node, _hig_node, _, _start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    
    // Test initial state
    println!("[TEST]   Testing initial state...");
    {
        let cl_node_with_lock = cl_node.lock().await;
        let current_block = cl_node_with_lock.get_current_block().await.unwrap();
        println!("[TEST]   Initial block number: {}", current_block);
        assert_eq!(current_block, 2, "Initial block should be 2 since block production starts immediately");
    }

    // Register chains first
    println!("[TEST]   Registering chains...");
    {
        let mut cl_node_with_lock = cl_node.lock().await;
        cl_node_with_lock.register_chain(ChainId("chain-1".to_string())).await.expect("Failed to register chain-1");
        cl_node_with_lock.register_chain(ChainId("chain-2".to_string())).await.expect("Failed to register chain-2");
        
        // Try to register chain-1 again (should fail)
        match cl_node_with_lock.register_chain(ChainId("chain-1".to_string())).await {
            Ok(_) => panic!("Should not be able to register chain-1 twice"),
            Err(e) => println!("[TEST]   Expected error when registering chain-1 twice: '{}'", e),
        }

        // Try to get subblock for unregistered chain
        match cl_node_with_lock.get_subblock(ChainId("chain3".to_string()), 0).await {
            Ok(_) => panic!("Should not be able to get subblock for unregistered chain"),
            Err(e) => println!("[TEST]   Expected error when getting subblock for unregistered chain: '{}'", e),
        }
    }

    // Verify chain registration and get subblock for registered chain
    println!("[TEST]   Verifying chain registration and subblock retrieval...");
    {
        let cl_node_with_lock = cl_node.lock().await;
        // Verify registered chains
        let registered_chains = cl_node_with_lock.get_registered_chains().await.unwrap();
        assert_eq!(registered_chains.len(), 2, "Should have exactly 2 registered chains");
        assert!(registered_chains.contains(&ChainId("chain-1".to_string())), "chain-1 should be registered");
        assert!(registered_chains.contains(&ChainId("chain-2".to_string())), "chain-2 should be registered");

        // Get subblock for registered chain
        match cl_node_with_lock.get_subblock(ChainId("chain-1".to_string()), 0).await {
            Ok(subblock) => {
                println!("[TEST]   Successfully got subblock for chain-1: {:?}", subblock);
                assert_eq!(subblock.chain_id, ChainId("chain-1".to_string()), "Subblock should be for chain-1");
                assert_eq!(subblock.block_height, 0, "Subblock should be for block 0");
                assert!(subblock.transactions.is_empty(), "Initial subblock should be empty");
            },
            Err(e) => panic!("Failed to get subblock for chain-1: '{}'", e),
        }
    }

    // Submit transactions for different chains
    println!("[TEST]   Submitting transactions...");
    {
        let mut cl_node_with_lock_2 = cl_node.lock().await;
        
        // Submit a transaction for chain-1
        let tx1 = CLTransaction {
            id: TransactionId("tx1".to_string()),
            data: "message1.chain-1".to_string(),
            constituent_chains: vec![ChainId("chain-1".to_string())],
        };
        cl_node_with_lock_2.submit_transaction(tx1).await.expect("Failed to submit transaction for chain-1");
        
        // Submit a transaction for chain-2
        let tx2 = CLTransaction {
            id: TransactionId("tx2".to_string()),
            data: "message1.chain-2".to_string(),
            constituent_chains: vec![ChainId("chain-2".to_string())],
        };
        cl_node_with_lock_2.submit_transaction(tx2).await.expect("Failed to submit transaction for chain-2");
        
        // Try to submit a transaction for unregistered chain (should fail)
        let tx3 = CLTransaction {
            id: TransactionId("tx3".to_string()),
            data: "message1.chain3".to_string(),
            constituent_chains: vec![ChainId("chain3".to_string())],
        };
        match cl_node_with_lock_2.submit_transaction(tx3).await {
            Ok(_) => panic!("Should not be able to submit transaction for unregistered chain"),
            Err(e) => println!("[TEST]   Expected error when submitting transaction for unregistered chain: '{}'", e),
        }
    }

    // wait for 1 second
    sleep(Duration::from_secs(1)).await;

    // Spawn tasks to add more transactions for different chains
    let sender_for_chain_1 = hs_node.lock().await.get_sender_to_cl().await;
    let _adder_handle1 = tokio::spawn(async move {
        run_spammer(sender_for_chain_1, ChainId("chain-1".to_string())).await;
    });

    let sender_for_chain_2 = hs_node.lock().await.get_sender_to_cl().await;
    let _adder_handle2 = tokio::spawn(async move {
        run_spammer(sender_for_chain_2, ChainId("chain-2".to_string())).await;
    });

    // Wait for a few seconds to let the processor run
    println!("Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let cl_node_with_lock_3 = cl_node.lock().await;
    let current_block = cl_node_with_lock_3.get_current_block().await.unwrap();
    println!("Main task: current block is {}", current_block);
    println!("Main task: processed {} transactions", cl_node_with_lock_3.state.lock().await.processed_transactions.len());
    println!("Main task: {} transactions still pending", cl_node_with_lock_3.state.lock().await.pending_transactions.len());
    println!("Main task: produced {} blocks", cl_node_with_lock_3.state.lock().await.blocks.len());
    let registered_chains = cl_node_with_lock_3.get_registered_chains().await.unwrap();
    println!("Main task: registered chains: {:?}", registered_chains);
    
    // Verify the state has been updated
    assert!(current_block > 0, "Block should have been incremented");
    assert!(!cl_node_with_lock_3.state.lock().await.processed_transactions.is_empty(), "Should have processed some transactions");
    assert!(!cl_node_with_lock_3.state.lock().await.blocks.is_empty(), "Should have produced some blocks");
    assert_eq!(registered_chains.len(), 2, "Should have exactly 2 registered chains");
    
    // Test getting subblock for registered chain
    match cl_node_with_lock_3.get_subblock(ChainId("chain-1".to_string()), 0).await {
        Ok(subblock) => println!("[TEST]   Successfully got subblock for chain-1: {:?}", subblock),
        Err(e) => panic!("Failed to get subblock for chain-1: '{}'", e),
    }
    
    // Drop the first state lock
    drop(cl_node_with_lock_3);
    
    // Wait for a bit more to let transactions be processed
    sleep(Duration::from_secs(1)).await;
    
    // Make sure the processor task is still running by checking the state again
    let state_guard = cl_node.lock().await;
    let current_block = state_guard.get_current_block().await.unwrap();
    let processed_count = state_guard.state.lock().await.processed_transactions.len();
    let block_count = state_guard.state.lock().await.blocks.len();
    println!("Main task: final check - block is {}, processed {} transactions in {} blocks", 
        current_block, processed_count, block_count);
    
    // Ensure the processor is still running and processing transactions
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 transactions per chain (one every 3 blocks)
    assert!(current_block > 25, "Block should have been incremented more than 25 times in 3 seconds, did {}", current_block);
    assert!(processed_count > 15, "Should have processed more than 15 transactions in 3 seconds (5 per chain), did {}", processed_count);
    assert!(block_count > 25, "Should have produced more than 25 blocks in 3 seconds, did {}", block_count);
    
    println!("=== Test completed successfully ===\n");
}

/// Helper function to run the adder task
async fn run_spammer(sender: mpsc::Sender<CLTransaction>, chain_id: ChainId) {
    for i in 1..=10 {
        let tx = CLTransaction {
            id: TransactionId(format!("tx{}.{}", i, chain_id.0)),
            data: format!("message{}.{}", i, chain_id.0),
            constituent_chains: vec![chain_id.clone()],
        };
        if let Err(e) = sender.send(tx).await {
            println!("  [TEST] [Adder] Error sending transaction: '{}'", e);
            break;
        }
        // wait for 300ms before sending next transaction
        sleep(Duration::from_millis(300)).await;
    }
}

