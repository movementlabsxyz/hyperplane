use hyperplane::{
    confirmation_layer::{ConfirmationLayer, ConfirmationLayerError},
    types::{BlockId, ChainId, TransactionId, CLTransaction},
};
use std::time::Duration;
use crate::common::testnodes;
/// Tests basic confirmation node functionality:
/// - Initial state (block interval, current block)
/// - Chain registration
/// - Subblock retrieval
/// - Duplicate registration handling
#[tokio::test]
async fn test_basic() {
    // use testnodes from common
    let (_, mut cl_node, _) = testnodes::setup_test_nodes();

    // Test initial state
    let block_interval = cl_node.get_block_interval().await.expect("Failed to get block interval");
    assert_eq!(block_interval, Duration::from_secs(1));

    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");
    assert_eq!(current_block, BlockId("0".to_string()));

    // Register a chain
    let chain_id = ChainId("test-chain".to_string());
    let _registration_block = cl_node.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Verify chain registration
    let chains = cl_node.get_registered_chains().await.expect("Failed to get registered chains");
    assert_eq!(chains.len(), 1);
    assert_eq!(chains[0], chain_id);

    // Test subblock retrieval
    let subblock = cl_node.get_subblock(chain_id.clone(), BlockId("0".to_string()))
        .await
        .expect("Failed to get subblock");
    assert_eq!(subblock.block_id, BlockId("0".to_string()));
    assert_eq!(subblock.chain_id, chain_id);
    assert!(subblock.transactions.is_empty());
}

/// Tests block interval configuration:
/// - Invalid interval rejection
/// - Valid interval setting and retrieval
#[tokio::test]
async fn test_block_interval() {
    let (_, mut cl_node, _) = testnodes::setup_test_nodes();

    // Test invalid interval
    let result = cl_node.set_block_interval(Duration::from_secs(0)).await;
    assert!(result.is_err());

    // Test valid interval
    let new_interval = Duration::from_millis(500);
    cl_node.set_block_interval(new_interval)
        .await
        .expect("Failed to set block interval");

    let current_interval = cl_node.get_block_interval().await.expect("Failed to get block interval");
    assert_eq!(current_interval, new_interval);
}

/// Tests normal transaction handling in confirmation node:
/// - Transaction submission
/// - Block production
/// - Transaction inclusion in subblocks
#[tokio::test]
async fn test_normal_transactions() {
    // Create a new confirmation node with a short block interval
    let (_, mut cl_node, _) = testnodes::setup_test_nodes();

    // Register a chain
    let chain_id = ChainId("test-chain".to_string());
    cl_node.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Create a normal transaction
    let tx = CLTransaction {
        id: TransactionId("test-tx".to_string()),
        chain_id: chain_id.clone(),
        data: "test data".to_string(),
    };

    // Submit the transaction
    cl_node.submit_transaction(tx.clone())
        .await
        .expect("Failed to submit transaction");

    // Wait for a block to be produced
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check that 5 blocks have been produced
    let current_block = cl_node.get_current_block().await.expect("Failed to get current block");
    assert_eq!(current_block, BlockId("5".to_string()));

    // Get the subblock for block 0 where the transaction was included
    let subblock = cl_node.get_subblock(chain_id.clone(), BlockId("0".to_string()))
        .await
        .expect("Failed to get subblock");
    println!("Subblock transactions: {:?}", subblock.transactions);

    // Verify the transaction was included
    assert_eq!(subblock.transactions.len(), 1);
    assert_eq!(subblock.transactions[0].data, "test data");
}

#[tokio::test]
async fn test_register_chain() {
    let (_, mut cl_node, _) = testnodes::setup_test_nodes();
    
    // Register a chain
    let chain_id = ChainId("test_chain".to_string());
    let result = cl_node.register_chain(chain_id.clone()).await;
    assert!(result.is_ok());
    
    // Verify chain is registered
    let chains = cl_node.get_registered_chains().await.unwrap();
    assert_eq!(chains.len(), 1);
    assert_eq!(chains[0], chain_id);
}

#[tokio::test]
async fn test_get_current_block() {
    let (_, cl_node, _) = testnodes::setup_test_nodes();
    
    // Get current block
    let block = cl_node.get_current_block().await.unwrap();
    assert_eq!(block, BlockId("0".to_string()));
}

#[tokio::test]
async fn test_get_subblock() {
    let (_, mut cl_node, _) = testnodes::setup_test_nodes();
    
    // Register a chain
    let chain_id = ChainId("test_chain".to_string());
    cl_node.register_chain(chain_id.clone()).await.unwrap();
    
    // Get subblock for non-existent block
    let block_id = BlockId("999".to_string());
    let subblock = cl_node.get_subblock(chain_id.clone(), block_id.clone()).await.unwrap();
    assert_eq!(subblock.block_id, block_id);
    assert_eq!(subblock.chain_id, chain_id);
    assert!(subblock.transactions.is_empty());
}

#[tokio::test]
async fn test_submit_transaction() {
    // Create a new confirmation node with a short block interval
    let (_, mut cl_node, _) = testnodes::setup_test_nodes();
    
    // Register a chain
    let chain_id = ChainId("test_chain".to_string());
    cl_node.register_chain(chain_id.clone()).await.unwrap();
    
    // Submit a transaction
    let transaction = CLTransaction {
        id: TransactionId("test-tx".to_string()),
        chain_id: chain_id.clone(),
        data: "test_data".to_string(),
    };
    let result = cl_node.submit_transaction(transaction).await;
    assert!(result.is_ok());
    
    // Wait for block production (500ms should be enough for 5 blocks)
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Get subblock and verify transaction
    let subblock = cl_node.get_subblock(chain_id, BlockId("0".to_string())).await.unwrap();
    assert_eq!(subblock.transactions.len(), 1);
    assert_eq!(subblock.transactions[0].data, "test_data");
}

#[tokio::test]
async fn test_set_block_interval() {
    let (_, mut cl_node, _) = testnodes::setup_test_nodes();
    
    // Set block interval
    let interval = Duration::from_secs(2);
    let result = cl_node.set_block_interval(interval).await;
    assert!(result.is_ok());
    
    // Verify block interval
    let current_interval = cl_node.get_block_interval().await.unwrap();
    assert_eq!(current_interval, interval);
}

#[tokio::test]
async fn test_invalid_block_interval() {
    let (_, mut cl_node, _) = testnodes::setup_test_nodes();
    
    // Try to set zero interval
    let result = cl_node.set_block_interval(Duration::from_secs(0)).await;
    assert!(matches!(result, Err(ConfirmationLayerError::InvalidBlockInterval(_))));
}

#[tokio::test]
async fn test_chain_not_found() {
    let (_, cl_node, _) = testnodes::setup_test_nodes();
    
    // Try to get subblock for non-existent chain
    let chain_id = ChainId("non_existent".to_string());
    let block_id = BlockId("0".to_string());
    let result = cl_node.get_subblock(chain_id, block_id).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainNotFound(_))));
}

#[tokio::test]
async fn test_chain_already_registered() {
    let (_, mut cl_node, _) = testnodes::setup_test_nodes();
    
    // Register a chain
    let chain_id = ChainId("test_chain".to_string());
    cl_node.register_chain(chain_id.clone()).await.unwrap();
    
    // Try to register the same chain again
    let result = cl_node.register_chain(chain_id).await;
    assert!(matches!(result, Err(ConfirmationLayerError::ChainAlreadyRegistered(_))));
} 