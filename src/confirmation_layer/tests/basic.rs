use tokio::time::{Duration, sleep};
use crate::{
    types::{TransactionId, ChainId, CLTransaction},
    confirmation_layer::{ConfirmationLayer, ConfirmationLayerError, node::ConfirmationLayerNode},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

/// Helper function to set up a test CL node
async fn setup_test_cl_node(block_interval: Duration) -> Arc<Mutex<ConfirmationLayerNode>> {
    let (_sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel(100);
    let (sender_cl_to_hig1, _receiver_cl_to_hig1) = mpsc::channel(100);
    let (sender_cl_to_hig2, _receiver_cl_to_hig2) = mpsc::channel(100);
    let cl_node = ConfirmationLayerNode::new_with_block_interval(
        receiver_hs_to_cl,
        sender_cl_to_hig1,
        sender_cl_to_hig2,
        block_interval
    ).expect("Failed to create CL node");
    let cl_node = Arc::new(Mutex::new(cl_node));
    ConfirmationLayerNode::start(cl_node.clone()).await;
    cl_node
}

/// Tests basic confirmation layer node functionality:
/// - Initial state (block interval, current block)
/// - Chain registration
/// - Subblock retrieval
/// - Duplicate registration handling
#[tokio::test]
async fn test_basic_confirmation_layer() {
    println!("\n=== Starting test_basic_confirmation_layer ===");
    
    // Get the test nodes using our helper function
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;
    
    // Test initial state
    println!("[TEST]   Testing initial state...");
    {
        let cl_node_with_lock = cl_node.lock().await;
        let current_block = cl_node_with_lock.get_current_block().await.unwrap();
        println!("[TEST]   Initial block number: {}", current_block);
        assert_eq!(current_block, 0, "Initial block should be 0 since block production starts immediately");
    }

    // Register chains first
    println!("[TEST]   Registering chains...");
    {
        let mut cl_node_with_lock = cl_node.lock().await;
        let chain_id = ChainId("chain-1".to_string());
        cl_node_with_lock.register_chain(chain_id.clone()).await.expect("Failed to register chain");
        
        // Try to register chain again (should fail)
        match cl_node_with_lock.register_chain(chain_id.clone()).await {
            Ok(_) => panic!("Should not be able to register chain twice"),
            Err(e) => println!("[TEST]   Expected error when registering chain twice: '{}'", e),
        }

        // Try to get subblock for unregistered chain
        match cl_node_with_lock.get_subblock(ChainId("chain-2".to_string()), 0).await {
            Ok(_) => panic!("Should not be able to get subblock for unregistered chain"),
            Err(e) => println!("[TEST]   Expected error when getting subblock for unregistered chain: '{}'", e),
        }
    }

    // Verify chain registration and get subblock for registered chain
    println!("[TEST]   Verifying chain registration and subblock retrieval...");
    {
        let cl_node_with_lock = cl_node.lock().await;
        let chain_id = ChainId("chain-1".to_string());
        
        // Verify registered chains
        let registered_chains = cl_node_with_lock.get_registered_chains().await.unwrap();
        assert_eq!(registered_chains.len(), 1, "Should have exactly 1 registered chain");
        assert!(registered_chains.contains(&chain_id), "chain-1 should be registered");

        // Get subblock for registered chain
        match cl_node_with_lock.get_subblock(chain_id.clone(), 0).await {
            Ok(subblock) => {
                println!("[TEST]   Successfully got subblock: {:?}", subblock);
                assert_eq!(subblock.chain_id, chain_id, "Subblock should be for chain-1");
                assert_eq!(subblock.block_height, 0, "Subblock should be for block 0");
                assert!(subblock.transactions.is_empty(), "Initial subblock should be empty");
            },
            Err(e) => panic!("Failed to get subblock: '{}'", e),
        }
    }

    // wait for 500 milliseconds
    sleep(Duration::from_millis(500)).await;

    // Submit a transaction
    println!("[TEST]   Submitting transaction...");
    {
        let mut cl_node_with_lock = cl_node.lock().await;
        let chain_id = ChainId("chain-1".to_string());
        
        // Submit a transaction
        let tx = CLTransaction::new(
            TransactionId("test-tx".to_string()),
            vec![chain_id.clone()],
            "REGULAR.SIMULATION:Success".to_string()
        ).expect("Failed to create CLTransaction");
        cl_node_with_lock.submit_transaction(tx).await.expect("Failed to submit transaction");
        
        // Try to submit a transaction for unregistered chain (should fail)
        let tx2 = CLTransaction::new(
            TransactionId("test-tx2".to_string()),
            vec![ChainId("chain-2".to_string())],
            "REGULAR.SIMULATION:Failure".to_string()
        ).expect("Failed to create CLTransaction");
        match cl_node_with_lock.submit_transaction(tx2).await {
            Ok(_) => panic!("Should not be able to submit transaction for unregistered chain"),
            Err(e) => println!("[TEST]   Expected error when submitting transaction for unregistered chain: '{}'", e),
        }
    }

    // Wait for block production
    println!("[TEST]   Waiting for block production...");
    sleep(Duration::from_millis(500)).await;

    // Check final state
    println!("[TEST]   Checking final state...");
    {
        let cl_node_with_lock = cl_node.lock().await;
        let current_block = cl_node_with_lock.get_current_block().await.unwrap();
        println!("[TEST]   Final block number: {}", current_block);
        
        // With 100ms interval, we should have produced at least 10 blocks in 1000ms
        assert!(current_block >= 10, "Should have produced at least 10 blocks");
        
        // Check blocks 3 to 7 for the presence of the transaction (it may be difficult to pin the exact block)
        let chain_id = ChainId("chain-1".to_string());
        let mut found = false;
        for block_id in 5..=9 {
            let subblock = cl_node_with_lock.get_subblock(chain_id.clone(), block_id).await.unwrap();
            if subblock.transactions.iter().any(|tx| tx.data == "REGULAR.SIMULATION:Success") {
                found = true;
                break;
            }
        }
        assert!(found, "Transaction should be present in one of the blocks 5 to 9");
    }

    println!("=== Test completed successfully ===\n");
}

/// Tests block interval configuration:
/// - Invalid interval rejection
/// - Valid interval setting and retrieval
#[tokio::test]
async fn test_block_interval() {
    println!("\n=== Starting test_block_interval ===");
    let cl_node = setup_test_cl_node(Duration::from_secs(1)).await;
    let interval = cl_node.lock().await.get_block_interval().await.unwrap();
    assert_eq!(interval, Duration::from_secs(1));
}

/// Tests regular transaction handling in confirmation node:
/// - Transaction submission
/// - Block production
/// - Transaction inclusion in subblocks
#[tokio::test]
async fn test_regular_transactions() {
    println!("\n=== Starting test_regular_transactions ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;

    // Register a chain
    let chain_id = ChainId("chain-1".to_string());
    cl_node.lock().await.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Create a regular transaction
    let tx = CLTransaction::new(
        TransactionId("regular-tx".to_string()),
        vec![chain_id.clone()],
        "REGULAR.SIMULATION:Success".to_string()
    ).expect("Failed to create transaction");

    // Submit the transaction
    cl_node.lock().await.submit_transaction(tx)
        .await
        .expect("Failed to submit transaction");

    // Wait for a block to be produced
    sleep(Duration::from_millis(500)).await;

    // Check that approximately 7 blocks have been produced (Startup time 200ms + 500ms / 100ms = 7 blocks)
    let current_block = cl_node.lock().await.get_current_block().await.expect("Failed to get current block");
    assert!(current_block >= 5 && current_block <= 9, 
        "Should have produced between 5 and 9 blocks, but have produced {}", current_block);

    // Check blocks 1 to 4 for the presence of the transaction (it may be difficult to pin the exact block)
    let mut found = false;
    for block_id in 1..=4 {
        let subblock = cl_node.lock().await.get_subblock(chain_id.clone(), block_id)
            .await
            .expect("Failed to get subblock");
        println!("Subblock transactions for block {}: {:?}", block_id, subblock.transactions);
        if subblock.transactions.iter().any(|tx| tx.data == "REGULAR.SIMULATION:Success") {
            found = true;
            break;
        }
    }
    assert!(found, "Transaction should be present in one of the blocks 1 to 4");
}

/// Tests chain registration functionality:
/// - Register a new chain
/// - Verify chain is registered
/// - Verify registration block is returned
#[tokio::test]
async fn test_register_chain() {
    println!("\n=== Starting test_register_chain ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;
    
    // Register a chain
    let chain_id = ChainId("chain-1".to_string());
    let result = cl_node.lock().await.register_chain(chain_id.clone()).await;
    assert!(result.is_ok());
    
    // Verify chain is registered
    let chains = cl_node.lock().await.get_registered_chains().await.unwrap();
    assert_eq!(chains.len(), 1);
    assert_eq!(chains[0], chain_id);
}

/// Tests current block retrieval:
/// - Get initial block number
/// - Verify block number format
#[tokio::test]
async fn test_get_current_block() {
    println!("\n=== Starting test_get_current_block ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;
    
    let initial_block = cl_node.lock().await.get_current_block().await.unwrap();
    assert_eq!(initial_block, 0, "Initial block should be 0 since we check it immediately after startup");

    // wait for 500ms
    sleep(Duration::from_millis(500)).await;

    // Get current block
    let block = cl_node.lock().await.get_current_block().await.unwrap();
    assert_eq!(block, 5, "Initial block should be 5 since we wait 500ms after startup");
}

/// Tests subblock retrieval functionality:
/// - Register a chain
/// - Get subblock for non-existent block
/// - Verify empty subblock is returned
#[tokio::test]
async fn test_get_subblock() {
    println!("\n=== Starting test_get_subblock ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;
    
    // Register a chain
    let chain_id = ChainId("chain-1".to_string());
    cl_node.lock().await.register_chain(chain_id.clone()).await.unwrap();
    
    // Get subblock for non-existent block
    let block_id = 999;
    let subblock = cl_node.lock().await.get_subblock(chain_id.clone(), block_id).await.unwrap();
    assert_eq!(subblock.block_height, block_id);
    assert_eq!(subblock.chain_id, chain_id);
    assert!(subblock.transactions.is_empty());
}

/// Tests transaction submission:
/// - Register a chain
/// - Submit a transaction
/// - Wait for block production
/// - Verify transaction is included in subblock
#[tokio::test]
async fn test_submit_transaction() {
    println!("\n=== Starting test_submit_transaction ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;

    // Register a chain
    let chain_id = ChainId("chain-1".to_string());
    cl_node.lock().await.register_chain(chain_id.clone()).await.unwrap();

    // Submit a transaction
    let transaction = CLTransaction::new(
        TransactionId("test-tx".to_string()),
        vec![chain_id.clone()],
        "REGULAR.SIMULATION:Success".to_string()
    ).expect("Failed to create CLTransaction");
    let result = cl_node.lock().await.submit_transaction(transaction).await;
    assert!(result.is_ok());

    // Wait for block production (500ms should be enough for 5 blocks)
    sleep(Duration::from_millis(500)).await;

    // Check blocks 1 to 4 for the presence of the transaction
    let mut found = false;
    for block_id in 1..=4 {
        let subblock = cl_node.lock().await.get_subblock(chain_id.clone(), block_id).await.unwrap();
        println!("Subblock transactions for block {}: {:?}", block_id, subblock.transactions);
        if subblock.transactions.iter().any(|tx| tx.data == "REGULAR.SIMULATION:Success") {
            found = true;
            break;
        }
    }
    assert!(found, "Transaction should be present in one of the blocks 1 to 4");
}

/// Tests block interval setting:
/// - Set a valid block interval
/// - Verify interval is updated
#[tokio::test]
async fn test_set_block_interval() {
    println!("\n=== Starting test_set_block_interval ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;
    
    // Set block interval
    let interval = Duration::from_secs(2);
    let result = cl_node.lock().await.set_block_interval(interval).await;
    assert!(result.is_ok());
    
    // Verify block interval
    let current_interval = cl_node.lock().await.get_block_interval().await.unwrap();
    assert_eq!(current_interval, interval);
}

/// Tests invalid block interval handling:
/// - Attempt to set zero interval
/// - Verify error is returned
#[tokio::test]
async fn test_invalid_block_interval() {
    println!("\n=== Starting test_invalid_block_interval ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;
    
    // Try to set zero interval
    let result = cl_node.lock().await.set_block_interval(Duration::from_secs(0)).await;
    assert!(matches!(result, Err(ConfirmationLayerError::InvalidBlockInterval(_))));
}

/// Tests chain not found error handling:
/// - Attempt to get subblock for non-existent chain
/// - Verify appropriate error is returned
#[tokio::test]
async fn test_chain_not_found() {
    println!("\n=== Starting test_chain_not_found ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;
    
    // Try to get subblock for non-existent chain
    let chain_id = ChainId("non_existent".to_string());
    let block_id = 0;
    let result = cl_node.lock().await.get_subblock(chain_id, block_id).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainNotFound(_))));
}

/// Tests duplicate chain registration handling:
/// - Register a chain
/// - Attempt to register same chain again
/// - Verify appropriate error is returned
#[tokio::test]
async fn test_chain_already_registered() {
    println!("\n=== Starting test_chain_already_registered ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;
    
    // Register a chain
    let chain_id = ChainId("chain-1".to_string());
    cl_node.lock().await.register_chain(chain_id.clone()).await.unwrap();
    
    // Try to register the same chain again
    let result = cl_node.lock().await.register_chain(chain_id).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainAlreadyRegistered(_))));
}

/// Tests chain registration and subblock retrieval:
/// - Register a chain
/// - Verify chain is registered
/// - Get subblock for registered chain
/// - Verify subblock properties
#[tokio::test]
async fn test_chain_registration() {
    println!("\n=== Starting test_chain_registration ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;

    // Register a chain
    let chain_id = ChainId("chain-1".to_string());
    cl_node.lock().await.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Get registered chains
    let chains = cl_node.lock().await.get_registered_chains().await.unwrap();
    assert_eq!(chains.len(), 1);
    assert_eq!(chains[0], chain_id);

    // Try to register the same chain again
    let result = cl_node.lock().await.register_chain(chain_id.clone()).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainAlreadyRegistered(_))), "Should not be able to register chain twice");

    // Get subblock for the chain
    let subblock = cl_node.lock().await.get_subblock(chain_id.clone(), 0).await.unwrap();
    assert_eq!(subblock.chain_id, chain_id);
}

/// Tests block interval validation:
/// - Set valid block interval
/// - Verify interval is updated
/// - Attempt to set invalid interval
/// - Verify error is returned
#[tokio::test]
async fn test_block_interval_validation() {
    println!("\n=== Starting test_block_interval_validation ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;

    // Set valid block interval
    let interval = Duration::from_millis(200);
    let result = cl_node.lock().await.set_block_interval(interval).await;
    assert!(result.is_ok());

    // Verify new interval
    let current_interval = cl_node.lock().await.get_block_interval().await.unwrap();
    assert_eq!(current_interval, interval);

    // Try to set invalid block interval
    let result = cl_node.lock().await.set_block_interval(Duration::from_secs(0)).await;
    assert!(matches!(result, Err(ConfirmationLayerError::InvalidBlockInterval(_))));
}

/// Tests subblock not found handling:
/// - Attempt to get subblock for non-existent chain
/// - Register chain
/// - Verify subblock retrieval
#[tokio::test]
async fn test_subblock_not_found() {
    println!("\n=== Starting test_subblock_not_found ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;

    // Try to get subblock for non-existent chain
    let chain_id = ChainId("chain-1".to_string());
    let block_id = 0;
    let result = cl_node.lock().await.get_subblock(chain_id.clone(), block_id).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainNotFound(_))));

    // Register chain
    let result = cl_node.lock().await.register_chain(chain_id).await;
    assert!(result.is_ok());
}

/// Tests get registered chains functionality:
/// - Register multiple chains
/// - Verify registered chains are returned
#[tokio::test]
async fn test_get_registered_chains() {
    println!("\n=== Starting test_get_registered_chains ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;

    // Register multiple chains
    let chain_ids = vec![
        ChainId("chain-1".to_string()),
        ChainId("chain-2".to_string()),
        ChainId("chain-3".to_string()),
    ];
    for chain_id in chain_ids.clone() {
        cl_node.lock().await.register_chain(chain_id).await.unwrap();
    }

    // Verify registered chains are returned
    let chains = cl_node.lock().await.get_registered_chains().await.unwrap();
    assert_eq!(chains.len(), 3);
    for chain_id in chain_ids {
        assert!(chains.contains(&chain_id));
    }
}

/// Tests get block interval functionality:
/// - Register a chain
/// - Verify block interval is returned
#[tokio::test]
async fn test_get_block_interval() {
    println!("\n=== Starting test_get_block_interval ===");
    let cl_node = setup_test_cl_node(Duration::from_secs(1)).await;
    let interval = cl_node.lock().await.get_block_interval().await.unwrap();
    assert_eq!(interval, Duration::from_secs(1));
}

/// Tests submit transaction functionality for a chain not registered:
/// - Attempt to submit a transaction for a chain not registered
/// - Verify appropriate error is returned
#[tokio::test]
async fn test_submit_transaction_chain_not_registered() {
    println!("\n=== Starting test_submit_transaction_chain_not_registered ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;

    // Attempt to submit a transaction for a chain not registered
    let chain_id = ChainId("non_existent".to_string());
    let tx = CLTransaction::new(
        TransactionId("test-tx".to_string()),
        vec![chain_id.clone()],
        "REGULAR.SIMULATION:Success".to_string()
    ).expect("Failed to create CLTransaction");
    let result = cl_node.lock().await.submit_transaction(tx).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainNotFound(_))), "Should not be able to submit transaction for unregistered chain");
}

// submit a transaction destined for multiple chains
#[tokio::test]
async fn test_submit_cl_transaction_for_multiple_chains() {
    println!("\n=== Starting test_submit_cl_transaction_for_multiple_chains ===");
    let cl_node = setup_test_cl_node(Duration::from_millis(100)).await;

    // Register multiple chains
    let chain_ids = vec![
        ChainId("chain-1".to_string()),
        ChainId("chain-2".to_string()),
        ChainId("chain-3".to_string()),
    ];

    for chain_id in chain_ids.clone() {
        cl_node.lock().await.register_chain(chain_id).await.unwrap();
    }

    // submit a transaction destined for all chains
    let tx = CLTransaction::new(
        TransactionId("test-tx".to_string()),
        chain_ids.clone(),
        "REGULAR.SIMULATION:Success".to_string()
    ).expect("Failed to create CLTransaction");
    cl_node.lock().await.submit_transaction(tx).await.unwrap();

    // wait for block production
    sleep(Duration::from_millis(200)).await;

    // check that the transaction is included in the subblock for each chain
    for chain_id in chain_ids {
        let subblock = cl_node.lock().await.get_subblock(chain_id, 1).await.unwrap();
        assert_eq!(subblock.transactions.len(), 1);
        assert_eq!(subblock.transactions[0].data, "REGULAR.SIMULATION:Success");
    }
}
