use hyperplane::{
    confirmation::{ConfirmationLayer, ConfirmationNode},
    types::{BlockId, ChainId, Transaction, TransactionId, TransactionWrapper, TransactionStatus, CATStatusProposal},
    hyper_ig::executor::{HyperIGNode, HyperIG},
};
use std::time::Duration;

/// Tests basic confirmation node functionality:
/// - Initial state (block interval, current block)
/// - Chain registration
/// - Subblock retrieval
/// - Duplicate registration handling
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

/// Tests block interval configuration:
/// - Invalid interval rejection
/// - Valid interval setting and retrieval
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

/// Tests normal transaction handling in confirmation node:
/// - Transaction submission
/// - Block production
/// - Transaction inclusion in subblocks
/// - Non-CAT transaction verification
#[tokio::test]
async fn test_confirmation_node_transactions() {
    // Create a new confirmation node with a short block interval
    let mut node = ConfirmationNode::with_block_interval(Duration::from_millis(100))
        .expect("Failed to create node");

    // Register a chain
    let chain_id = ChainId("test-chain".to_string());
    node.register_chain(chain_id.clone())
        .await
        .expect("Failed to register chain");

    // Create a normal transaction (not a CAT)
    let tx = Transaction {
        id: TransactionId("test-tx".to_string()),
        chain_id: chain_id.clone(),
        data: "test data".to_string(),
        timestamp: Duration::from_secs(0),
    };
    let tx_wrapper = TransactionWrapper {
        transaction: tx.clone(),
        is_cat: false,
    };

    // Submit the transaction
    node.submit_transaction(tx_wrapper)
        .await
        .expect("Failed to submit transaction");

    // Wait for a block to be produced
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check that 5 blocks have been produced
    let current_block = node.get_current_block().await.expect("Failed to get current block");
    assert_eq!(current_block, BlockId(5));

    // Get the subblock for block 0 where the transaction was included
    let subblock = node.get_subblock(chain_id.clone(), BlockId(0))
        .await
        .expect("Failed to get subblock");
    println!("Subblock transactions: {:?}", subblock.transactions);

    // Verify the transaction was included and is not a CAT
    assert_eq!(subblock.transactions.len(), 1);
    assert_eq!(subblock.transactions[0].transaction.id, tx.id);
    assert!(!subblock.transactions[0].is_cat);
}

/// Tests CAT transaction handling in confirmation node:
/// - Multiple chain registration
/// - CAT transaction submission to multiple chains
/// - CAT transaction inclusion in all relevant subblocks
/// - CAT flag verification
#[tokio::test]
async fn test_confirmation_node_cat_transactions() {
    // Create a new confirmation node with a short block interval
    let mut node = ConfirmationNode::with_block_interval(Duration::from_millis(100))
        .expect("Failed to create node");

    // Register two chains
    let chain1 = ChainId("chain-1".to_string());
    let chain2 = ChainId("chain-2".to_string());
    node.register_chain(chain1.clone())
        .await
        .expect("Failed to register chain 1");
    node.register_chain(chain2.clone())
        .await
        .expect("Failed to register chain 2");

    // Create CAT transactions for both chains
    let cat_id = TransactionId("cat-tx".to_string());
    let tx1 = Transaction {
        id: cat_id.clone(),
        chain_id: chain1.clone(),
        data: "cat data for chain 1".to_string(),
        timestamp: Duration::from_secs(0),
    };
    let tx2 = Transaction {
        id: cat_id.clone(),
        chain_id: chain2.clone(),
        data: "cat data for chain 2".to_string(),
        timestamp: Duration::from_secs(0),
    };

    // Create wrappers for both transactions
    let tx_wrapper1 = TransactionWrapper {
        transaction: tx1.clone(),
        is_cat: true,
    };
    let tx_wrapper2 = TransactionWrapper {
        transaction: tx2.clone(),
        is_cat: true,
    };

    // Submit both transactions
    node.submit_transaction(tx_wrapper1)
        .await
        .expect("Failed to submit CAT to chain 1");
    node.submit_transaction(tx_wrapper2)
        .await
        .expect("Failed to submit CAT to chain 2");

    // Wait for a block to be produced
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Get the subblocks for block 0
    let subblock1 = node.get_subblock(chain1.clone(), BlockId(0))
        .await
        .expect("Failed to get subblock for chain 1");
    let subblock2 = node.get_subblock(chain2.clone(), BlockId(0))
        .await
        .expect("Failed to get subblock for chain 2");

    println!("Chain 1 subblock transactions: {:?}", subblock1.transactions);
    println!("Chain 2 subblock transactions: {:?}", subblock2.transactions);

    // Verify both chains received the CAT
    assert_eq!(subblock1.transactions.len(), 1);
    assert_eq!(subblock2.transactions.len(), 1);
    assert_eq!(subblock1.transactions[0].transaction.id, cat_id);
    assert_eq!(subblock2.transactions[0].transaction.id, cat_id);
    assert!(subblock1.transactions[0].is_cat);
    assert!(subblock2.transactions[0].is_cat);
}

/// Tests normal transaction success path in HyperIG:
/// - Non-dependent transaction execution
/// - Success status verification
/// - Status persistence
#[tokio::test]
async fn test_hyper_ig_normal_transaction_success() {
    let mut hig = HyperIGNode::new();
    
    // Create a normal transaction with non-dependent data
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        chain_id: ChainId("test-chain".to_string()),
        data: "any data".to_string(),
        timestamp: Duration::from_secs(0),
    };
    
    // Execute the transaction
    let status = hig.execute_transaction_wrapper(TransactionWrapper {
        transaction: tx.clone(),
        is_cat: false,
    })
        .await
        .expect("Failed to execute transaction");
    
    // Verify it was successful (normal transactions with non-dependent data are successful)
    assert!(matches!(status, TransactionStatus::Success));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Success));
}

/// Tests normal transaction pending path in HyperIG:
/// - Dependent transaction execution
/// - Pending status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_hyper_ig_normal_transaction_pending() {
    let mut hig = HyperIGNode::new();
    
    // Create a normal transaction with dependent data
    let tx = Transaction {
        id: TransactionId("normal-tx".to_string()),
        chain_id: ChainId("test-chain".to_string()),
        data: "dependent".to_string(),
        timestamp: Duration::from_secs(0),
    };
    
    // Execute the transaction
    let status = hig.execute_transaction_wrapper(TransactionWrapper {
        transaction: tx.clone(),
        is_cat: false,
    })
        .await
        .expect("Failed to execute transaction");
    
    // Verify it stays pending (normal transactions with dependent data stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
}

/// Tests CAT transaction success-proposal path in HyperIG:
/// - CAT transaction with success data
/// - Pending status verification
/// - Success proposed status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_hyper_ig_cat_success_proposal() {
    let mut hig = HyperIGNode::new();
    
    // Create a CAT transaction with success data
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        chain_id: ChainId("test-chain".to_string()),
        data: "success".to_string(),
        timestamp: Duration::from_secs(0),
    };
    let tx_wrapper = TransactionWrapper {
        transaction: tx.clone(),
        is_cat: true,
    };
    
    // Execute the transaction
    let status = hig.execute_transaction_wrapper(tx_wrapper)
        .await
        .expect("Failed to execute transaction");
    
    // Verify status is pending (CAT transactions always stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
    
    // Verify proposed status is Success
    let proposed_status = hig.get_proposed_status(tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    assert!(matches!(proposed_status, CATStatusProposal::Success));
}

/// Tests CAT transaction failure-proposal path in HyperIG:
/// - CAT transaction with failure data
/// - Pending status verification
/// - Failure proposed status verification
/// - Pending transaction list inclusion
#[tokio::test]
async fn test_hyper_ig_cat_failure_proposal() {
    let mut hig = HyperIGNode::new();
    
    // Create a CAT transaction with non-success data
    let tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        chain_id: ChainId("test-chain".to_string()),
        data: "failure".to_string(),
        timestamp: Duration::from_secs(0),
    };
    let tx_wrapper = TransactionWrapper {
        transaction: tx.clone(),
        is_cat: true,
    };
    
    // Execute the transaction
    let status = hig.execute_transaction_wrapper(tx_wrapper)
        .await
        .expect("Failed to execute transaction");
    
    // Verify status is pending (CAT transactions always stay pending)
    assert!(matches!(status, TransactionStatus::Pending));
    
    // Verify we can retrieve the same status
    let retrieved_status = hig.get_transaction_status(tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(retrieved_status, TransactionStatus::Pending));
    
    // Verify it's in the pending transactions list
    let pending = hig.get_pending_transactions()
        .await
        .expect("Failed to get pending transactions");
    assert!(pending.contains(&tx.id));
    
    // Verify proposed status is Failure
    let proposed_status = hig.get_proposed_status(tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    assert!(matches!(proposed_status, CATStatusProposal::Failure));
}
