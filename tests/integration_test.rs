use hyperplane::{
    confirmation::{ConfirmationLayer, ConfirmationNode},
    types::{BlockId, ChainId, ChainRegistration, SubBlock},
};
use std::time::Duration;

#[tokio::test]
async fn test_confirmation_node_basic() {
    // Create a new confirmation node
    let mut node = ConfirmationNode::new();

    // Test initial state
    let block_interval = node.get_block_interval().await.expect("Failed to get block interval");
    assert_eq!(block_interval, Duration::from_secs(1));

    let current_block = node.get_current_block().await.expect("Failed to get current block");
    assert_eq!(current_block, BlockId(0));

    // Register a chain
    let chain_id = ChainId("test-chain".to_string());
    let registration_block = node.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Verify chain registration
    let chains = node.get_registered_chains().await.expect("Failed to get registered chains");
    assert_eq!(chains.len(), 1);
    assert_eq!(chains[0].chain_id, chain_id);
    assert_eq!(chains[0].registration_block, registration_block);
    assert!(chains[0].active);

    // Test subblock retrieval
    let subblock = node.get_subblock(chain_id.clone(), BlockId(0))
        .await
        .expect("Failed to get subblock");
    assert_eq!(subblock.block_id, BlockId(0));
    assert_eq!(subblock.chain_id, chain_id);
    assert!(subblock.transactions.is_empty());

    // Test duplicate registration
    let result = node.register_chain(chain_id.clone()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_confirmation_node_block_interval() {
    let mut node = ConfirmationNode::new();

    // Test invalid interval
    let result = node.set_block_interval(Duration::from_secs(0)).await;
    assert!(result.is_err());

    // Test valid interval
    let new_interval = Duration::from_millis(500);
    node.set_block_interval(new_interval)
        .await
        .expect("Failed to set block interval");

    let current_interval = node.get_block_interval().await.expect("Failed to get block interval");
    assert_eq!(current_interval, new_interval);
} 