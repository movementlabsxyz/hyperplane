use tokio::time::{Duration, sleep};
use crate::{
    types::{TransactionId, ChainId, CLTransaction, Transaction},
    confirmation_layer::{ConfirmationLayer, ConfirmationLayerError, node::ConfirmationLayerNode},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

/// Helper function to set up a test CL node
async fn setup_cl_node(block_interval: Duration) -> Arc<Mutex<ConfirmationLayerNode>> {
    let (_sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel(100);
    let cl_node = ConfirmationLayerNode::new_with_block_interval(
        receiver_hs_to_cl,
        block_interval
    ).expect("Failed to create CL node");
    let cl_node = Arc::new(Mutex::new(cl_node));
    ConfirmationLayerNode::start(cl_node.clone()).await;
    cl_node
}

/// Helper function to set up a test CL node with a chain already registered
async fn setup_cl_node_with_registration(block_interval: Duration) -> Arc<Mutex<ConfirmationLayerNode>> {
    let cl_node = setup_cl_node(block_interval).await;
    let chain_id_1 = ChainId("chain-1".to_string());
    let chain_id_2 = ChainId("chain-2".to_string());

    // Create mock channels for the chains
    let (sender_1, _receiver_1) = mpsc::channel(10);
    let (sender_2, _receiver_2) = mpsc::channel(10);

    // Register the chains with their channels
    cl_node.lock().await.register_chain(chain_id_1.clone(), sender_1).await.expect("Failed to register chain-1");
    cl_node.lock().await.register_chain(chain_id_2.clone(), sender_2).await.expect("Failed to register chain-2");

    cl_node
}

/// Tests block interval functionality:
/// - Verify initial block interval
/// - Set and verify valid block interval
/// - Attempt to set invalid intervals (zero, too short, too long)
/// - Verify interval persistence after invalid attempts
#[tokio::test]
async fn test_block_interval() {
    println!("\n=== Starting test_block_interval ===");
    let cl_node = setup_cl_node_with_registration(Duration::from_secs(1)).await;
    
    // Test initial interval
    println!("[TEST]   Verifying initial block interval...");
    let initial_interval = cl_node.lock().await.get_block_interval().await.unwrap();
    assert_eq!(initial_interval, Duration::from_secs(1), "Initial block interval should be 1 second");
    println!("[TEST]   Initial block interval verified");
    
    // Test setting valid interval
    println!("[TEST]   Setting valid block interval...");
    let new_interval = Duration::from_millis(200);
    let result = cl_node.lock().await.set_block_interval(new_interval).await;
    assert!(result.is_ok(), "Failed to set valid block interval");
    
    // Verify new interval
    let current_interval = cl_node.lock().await.get_block_interval().await.unwrap();
    assert_eq!(current_interval, new_interval, "Block interval should be updated to 200ms");
    println!("[TEST]   Valid block interval update verified");
    
    // Test setting invalid intervals
    println!("[TEST]   Testing invalid block intervals...");
    
    // Test zero interval
    let result = cl_node.lock().await.set_block_interval(Duration::from_secs(0)).await;
    assert!(matches!(result, Err(ConfirmationLayerError::InvalidBlockInterval(_))), 
        "Should not be able to set zero interval");
    println!("[TEST]   Zero interval correctly rejected");

    // Verify interval hasn't changed after invalid attempts
    let final_interval = cl_node.lock().await.get_block_interval().await.unwrap();
    assert_eq!(final_interval, new_interval, "Block interval should remain unchanged after invalid attempts");
    println!("[TEST]   Block interval persistence verified");
    
    println!("=== Test completed successfully ===\n");
}

/// Tests transaction submission functionality:
/// - Submit a transaction
/// - Verify transaction is included in blocks
/// - Verify block production timing
#[tokio::test]
async fn test_transaction_submission() {
    println!("\n=== Starting test_transaction_submission ===");
    let cl_node = setup_cl_node_with_registration(Duration::from_millis(100)).await;

    // Create and submit a transaction
    println!("[TEST]   Submitting transaction...");
    let chain_id = ChainId("chain-1".to_string());
    let tx = Transaction::new(
        TransactionId("regular-tx".to_string()),
        chain_id.clone(),
        vec![chain_id.clone()],
        "REGULAR.SIMULATION:Success".to_string(),
    ).expect("Failed to create transaction");
    let cl_tx = CLTransaction::new(
        TransactionId("regular-tx".to_string()),
        vec![chain_id.clone()],
        vec![tx],
    ).expect("Failed to create CL transaction");
    let result = cl_node.lock().await.submit_transaction(cl_tx).await;
    assert!(result.is_ok(), "Transaction submission should succeed");
    println!("[TEST]   Transaction submitted successfully");

    // Wait for block production
    println!("[TEST]   Waiting for block production...");
    sleep(Duration::from_millis(500)).await;

    // Verify block production timing
    println!("[TEST]   Verifying block production...");
    let current_block = cl_node.lock().await.get_current_block().await.expect("Failed to get current block");
    assert!(current_block >= 5 && current_block <= 9, 
        "Should have produced between 5 and 9 blocks, but have produced {}", current_block);
    println!("[TEST]   Block production timing verified");

    // Verify transaction inclusion
    println!("[TEST]   Verifying transaction inclusion...");
    let mut found = false;
    for block_id in 1..=4 {
        let subblock = cl_node.lock().await.get_subblock(chain_id.clone(), block_id)
            .await
            .expect("Failed to get subblock");
        println!("[TEST]   Subblock transactions for block {}: {:?}", block_id, subblock.transactions);
        if subblock.transactions.iter().any(|tx| tx.data == "REGULAR.SIMULATION:Success") {
            found = true;
            break;
        }
    }
    assert!(found, "Transaction should be present in one of the blocks 1 to 4");
    println!("[TEST]   Transaction inclusion verified");

    println!("=== Test completed successfully ===\n");
}

/// Tests chain registration functionality:
/// - Register a new chain
/// - Verify chain is registered
/// - Verify registration block is returned
/// - Verify duplicate registration is rejected
/// - Verify subblock retrieval for registered chain
#[tokio::test]
async fn test_chain_registration() {
    println!("\n=== Starting test_chain_registration ===");
    let cl_node = setup_cl_node(Duration::from_millis(100)).await;

    // Register a chain
    println!("[TEST]   Registering chain...");
    let chain_id = ChainId("chain-1".to_string());
    let (sender, _receiver) = mpsc::channel(10);
    let result = cl_node.lock().await.register_chain(chain_id.clone(), sender).await;
    assert!(result.is_ok(), "Failed to register chain");
    println!("[TEST]   Chain registered successfully");

    // Verify chain is registered
    println!("[TEST]   Verifying chain registration...");
    let chains = cl_node.lock().await.get_registered_chains().await.unwrap();
    assert_eq!(chains.len(), 1, "Should have exactly 1 registered chain");
    assert_eq!(chains[0], chain_id, "Registered chain should match");
    println!("[TEST]   Chain verification successful");

    // Try to register the same chain again
    println!("[TEST]   Attempting duplicate registration...");
    let (sender_again, _receiver_again) = mpsc::channel(10);
    let result = cl_node.lock().await.register_chain(chain_id.clone(),sender_again).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainAlreadyRegistered(_))), 
        "Should not be able to register chain twice");
    println!("[TEST]   Duplicate registration correctly rejected");

    // Get subblock for the chain
    println!("[TEST]   Verifying subblock retrieval...");
    let subblock = cl_node.lock().await.get_subblock(chain_id.clone(), 0).await.unwrap();
    assert_eq!(subblock.chain_id, chain_id, "Subblock should be for registered chain");
    assert_eq!(subblock.block_height, 0, "Subblock should be for block 0");
    assert!(subblock.transactions.is_empty(), "Initial subblock should be empty");
    println!("[TEST]   Subblock retrieval successful");

    println!("=== Test completed successfully ===\n");
}

/// Tests current block retrieval:
/// - Get initial block number
/// - Verify block number format
#[tokio::test]
async fn test_get_current_block() {
    println!("\n=== Starting test_get_current_block ===");
    let cl_node = setup_cl_node_with_registration(Duration::from_millis(100)).await;
    
    let initial_block = cl_node.lock().await.get_current_block().await.unwrap();
    assert_eq!(initial_block, 0, "Initial block should be 0 since we check it immediately after startup");

    // wait for 500ms
    sleep(Duration::from_millis(500)).await;

    // Get current block
    let block = cl_node.lock().await.get_current_block().await.unwrap();
    assert_eq!(block, 5, "Initial block should be 5 since we wait 500ms after startup");
}

/// Tests subblock retrieval functionality:
/// - Get subblock for non-existent block
/// - Verify empty subblock is returned
#[tokio::test]
async fn test_get_subblock() {
    println!("\n=== Starting test_get_subblock ===");
    let cl_node = setup_cl_node_with_registration(Duration::from_millis(100)).await;
    
    // Get subblock for non-existent block
    let chain_id = ChainId("chain-1".to_string());
    let block_id = 999;
    let subblock = cl_node.lock().await.get_subblock(chain_id.clone(), block_id).await.unwrap();
    assert_eq!(subblock.block_height, block_id);
    assert_eq!(subblock.chain_id, chain_id);
    assert!(subblock.transactions.is_empty());
}

/// Tests chain not found error handling:
/// - Attempt to get subblock for non-existent chain
/// - Verify appropriate error is returned
#[tokio::test]
async fn test_chain_not_found() {
    println!("\n=== Starting test_chain_not_found ===");
    let cl_node = setup_cl_node(Duration::from_millis(100)).await;
    
    // Try to get subblock for non-existent chain
    let chain_id = ChainId("non_existent".to_string());
    let block_id = 0;
    let result = cl_node.lock().await.get_subblock(chain_id, block_id).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainNotFound(_))));
}

/// Tests register a third chain functionality:
/// - Register a third chain
/// - Verify registered chains are returned
#[tokio::test]
async fn test_get_registered_chains() {
    println!("\n=== Starting test_get_registered_chains ===");
    let cl_node = setup_cl_node_with_registration(Duration::from_millis(100)).await;

    // Register a third chain
    let chain_id = ChainId("chain-3".to_string());
    let (sender, _receiver) = mpsc::channel(10);
    cl_node.lock().await.register_chain(chain_id.clone(), sender).await.expect("Failed to register chain-3");

    // Verify registered chain is returned
    let chains = cl_node.lock().await.get_registered_chains().await.unwrap();
    assert_eq!(chains.len(), 3);
    assert!(chains.contains(&chain_id.clone()), "Chain-3 should be registered");
}

/// Tests get block interval functionality:
/// - Register a chain
/// - Verify block interval is returned
#[tokio::test]
async fn test_get_block_interval() {
    println!("\n=== Starting test_get_block_interval ===");
    let cl_node = setup_cl_node(Duration::from_millis(200)).await;
    let interval = cl_node.lock().await.get_block_interval().await.unwrap();
    assert_eq!(interval, Duration::from_millis(200));
}

/// Tests submit transaction functionality for a chain not registered:
/// - Attempt to submit a transaction for a chain not registered
/// - Verify appropriate error is returned
#[tokio::test]
async fn test_submit_transaction_chain_not_registered() {
    println!("\n=== Starting test_submit_transaction_chain_not_registered ===");
    let cl_node = setup_cl_node_with_registration(Duration::from_millis(100)).await;

    // Attempt to submit a transaction for a chain not registered
    let chain_id = ChainId("chain-3".to_string());
    let tx = Transaction::new(
        TransactionId("test-tx".to_string()),
        chain_id.clone(),
        vec![chain_id.clone()],
        "REGULAR.SIMULATION:Success".to_string(),
    ).expect("Failed to create transaction");
    let cl_tx = CLTransaction::new(
        TransactionId("test-tx".to_string()),
        vec![chain_id.clone()],
        vec![tx],
    ).expect("Failed to create CL transaction");
    let result = cl_node.lock().await.submit_transaction(cl_tx).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainNotFound(_))), "Should not be able to submit transaction for unregistered chain");
}

/// Tests submit a transaction destined for two registered chains
#[tokio::test]
async fn test_submit_cl_transaction_for_two_chains() {
    println!("\n=== Starting test_submit_cl_transaction_for_two_chains ===");
    let cl_node = setup_cl_node_with_registration(Duration::from_millis(100)).await;

    // Create and submit a transaction for both chains
    println!("[TEST]   Submitting transaction for both chains...");
    let chain1_id = ChainId("chain-1".to_string());
    let chain2_id = ChainId("chain-2".to_string());
    
    let tx1 = Transaction::new(
        TransactionId("multi-tx-1".to_string()),
        chain1_id.clone(),
        vec![chain1_id.clone(), chain2_id.clone()],
        "REGULAR.SIMULATION:Success".to_string(),
    ).expect("Failed to create transaction");
    let tx2 = Transaction::new(
        TransactionId("multi-tx-2".to_string()),
        chain2_id.clone(),
        vec![chain1_id.clone(), chain2_id.clone()],
        "REGULAR.SIMULATION:Success".to_string(),
    ).expect("Failed to create transaction");
    
    let cl_tx = CLTransaction::new(
        TransactionId("multi-tx".to_string()),
        vec![chain1_id.clone(), chain2_id.clone()],
        vec![tx1, tx2],
    ).expect("Failed to create CL transaction");
    
    let result = cl_node.lock().await.submit_transaction(cl_tx).await;
    assert!(result.is_ok(), "Transaction submission should succeed");
    println!("[TEST]   Transaction submitted successfully");

    // Wait for block production
    println!("[TEST]   Waiting for block production...");
    sleep(Duration::from_millis(500)).await;

    // Verify transaction inclusion in both chains
    println!("[TEST]   Verifying transaction inclusion in both chains...");
    let mut found_chain1 = false;
    let mut found_chain2 = false;

    for block_id in 1..=4 {
        let subblock1 = cl_node.lock().await.get_subblock(chain1_id.clone(), block_id)
            .await
            .expect("Failed to get subblock for chain 1");
        let subblock2 = cl_node.lock().await.get_subblock(chain2_id.clone(), block_id)
            .await
            .expect("Failed to get subblock for chain 2");

        if subblock1.transactions.iter().any(|tx| tx.data == "REGULAR.SIMULATION:Success") {
            found_chain1 = true;
        }
        if subblock2.transactions.iter().any(|tx| tx.data == "REGULAR.SIMULATION:Success") {
            found_chain2 = true;
        }

        if found_chain1 && found_chain2 {
            break;
        }
    }

    assert!(found_chain1, "Transaction should be present in chain 1");
    assert!(found_chain2, "Transaction should be present in chain 2");
    println!("[TEST]   Transaction inclusion verified for both chains");

    println!("=== Test completed successfully ===\n");
}

/// Tests dynamic channel registration and message delivery to dynamically registered HIG nodes
#[tokio::test]
async fn test_dynamic_channel_registration() {
    println!("\n=== Starting test_dynamic_channel_registration ===");
    let cl_node = setup_cl_node_with_registration(Duration::from_millis(100)).await;

    // Register a new chain dynamically
    println!("[TEST]   Registering new chain dynamically...");
    let dynamic_chain_id = ChainId("dynamic-chain".to_string());
    let (tx, mut rx) = mpsc::channel(100);
    let _ = cl_node.lock().await.register_chain(dynamic_chain_id.clone(), tx).await;
    println!("[TEST]   Chain registered successfully");

    // Create and submit a transaction for the dynamic chain
    println!("[TEST]   Submitting transaction for dynamic chain...");
    let tx = Transaction::new(
        TransactionId("test-tx".to_string()),
        dynamic_chain_id.clone(),
        vec![dynamic_chain_id.clone()],
        "REGULAR.SIMULATION:Success".to_string(),
    ).expect("Failed to create transaction");
    let cl_tx = CLTransaction::new(
        TransactionId("test-tx".to_string()),
        vec![dynamic_chain_id.clone()],
        vec![tx],
    ).expect("Failed to create CL transaction");
    let result = cl_node.lock().await.submit_transaction(cl_tx).await;
    assert!(result.is_ok(), "Transaction submission should succeed");
    println!("[TEST]   Transaction submitted successfully");

    // Wait for block production
    println!("[TEST]   Waiting for block production...");
    sleep(Duration::from_millis(500)).await;

    // Verify the subblock was received
    println!("[TEST]   Verifying subblock reception...");
    let mut received = false;
    for _ in 0..10 {
        if let Ok(subblock) = rx.try_recv() {
            assert_eq!(subblock.chain_id, dynamic_chain_id);
            assert_eq!(subblock.block_height, 1);
            assert_eq!(subblock.transactions.len(), 1);
            assert_eq!(subblock.transactions[0].data, "REGULAR.SIMULATION:Success");
            received = true;
            break;
        }
        sleep(Duration::from_millis(50)).await;
    }
    assert!(received, "Should receive subblock for dynamic chain");
    println!("[TEST]   Subblock received and verified");

    println!("=== Test completed successfully ===\n");
}
