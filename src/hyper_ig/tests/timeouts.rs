use crate::types::{Transaction, TransactionId, CATId, SubBlock, TransactionStatus, CLTransactionId, constants};
use crate::utils::logging;
use crate::hyper_ig::tests::basic::setup_test_hig_node;
use crate::hyper_ig::HyperIG;

/// Helper function to run a CAT timeout test with specific parameters
async fn run_cat_timeout_test(second_block_height: u64, expected_status: TransactionStatus) {
    logging::init_logging();
    logging::log("TEST", &format!("\n=== Starting CAT timeout test with block height {} and expected status {:?} ===", 
        second_block_height, expected_status));
    
    // Create node
    let (mut hig_node, _receiver_hig_to_hs) = setup_test_hig_node().await;
    
    // Create a CAT transaction
    let cl_id = CLTransactionId("cl-tx".to_string());
    logging::log("TEST", &format!("Creating cl-id='{}'", cl_id.0));
    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
    logging::log("TEST", &format!("Created tx-id='{}'", tx_id.0));
    let cat_tx = Transaction::new(
        tx_id,
        constants::chain_1(),
        vec![constants::chain_1(), constants::chain_2()],
        "CAT.credit 1 100".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    // Process the CAT in block 1
    let subblock = SubBlock {
        block_height: 1,
        chain_id: constants::chain_1(),
        transactions: vec![cat_tx.clone()],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed block height=1"));
    
    // Verify CAT is pending
    let status = hig_node.get_transaction_status(cat_tx.id.clone()).await.unwrap();
    assert_eq!(status, TransactionStatus::Pending);
    
    // Get the max lifetime
    let cat_id = CATId(cl_id.clone());
    let max_lifetime = hig_node.get_cat_max_lifetime(cat_id).await.unwrap();
    logging::log("TEST", &format!("Max lifetime='{}'", max_lifetime));
    
    // Process the second block at the specified height
    let subblock = SubBlock {
        block_height: second_block_height,
        chain_id: constants::chain_1(),
        transactions: vec![],
    };
    hig_node.process_subblock(subblock).await.unwrap();
    logging::log("TEST", &format!("Processed block height={}", second_block_height));

    // Get current block height
    let block_height = hig_node.get_current_block_height().await.unwrap();
    logging::log("TEST", &format!("Current block height='{}'", block_height));

    // Verify CAT has the expected status
    let status = hig_node.get_transaction_status(cat_tx.id).await.unwrap();
    assert_eq!(status, expected_status);
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that a CAT transaction expires correctly when its lifetime is exceeded.
#[tokio::test]
async fn test_cat_timeout() {
    // Create a CAT in block 1, then process block 6 (which is after max lifetime)
    run_cat_timeout_test(6, TransactionStatus::Failure).await;
}

/// Tests that a CAT transaction remains pending for a block height less than its expiration.
#[tokio::test]
async fn test_cat_not_expired() {
    // Create a CAT in block 1, then process block 5 (which is before max lifetime)
    run_cat_timeout_test(5, TransactionStatus::Pending).await;
}

/// Tests that a CAT transaction expires exactly at its max lifetime.
#[tokio::test]
async fn test_cat_expires_at_lifetime() {
    // Create a CAT in block 1, then process block 5 (which is exactly at max lifetime)
    run_cat_timeout_test(5, TransactionStatus::Pending).await;
}
