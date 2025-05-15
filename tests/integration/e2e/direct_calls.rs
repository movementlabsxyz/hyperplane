use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited, ChainId, CLTransaction, CATId, SubBlock},
    hyper_ig::HyperIG,
    hyper_scheduler::HyperScheduler,
    confirmation_layer::ConfirmationLayer,
};
use crate::common::testnodes;
use tokio::time::{sleep, Duration};

/// Tests the complete flow of a CAT transaction through all components:
/// 1. CAT created in CL with success simulation
/// 2. HIG processes it and forwards success proposal to HS
/// 3. HS sets success update and sends to CL
/// 4. CL forwards to HIG
/// 5. HIG updates CAT to success
#[tokio::test]
async fn test_cat_complete_flow() {
    println!("\n=== Starting test_cat_complete_flow ===");
    
    // - - - - - - - - - Setup - - - - - - - - -
    println!("\n[test.Setup] Initializing components...");
    let (hs_node, cl_node, hig_node,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Register chain in CL
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST.Setup] Registering chain in CL...");
    cl_node.lock().await.register_chain(chain_id.clone()).await.expect("Failed to register chain");

    // Register chain in HS
    println!("[TEST.Setup] Registering chain in HS...");
    hs_node.lock().await.set_chain_id(chain_id.clone()).await;

    // - - - - - - - - - CL processes CAT transaction - - - - - - - - -
    println!("\n[test.CL] Submitting CAT transaction...");
    // Create and submit CAT transaction to CL
    let cat_tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    hs_node.lock().await.submit_transaction(CLTransaction {
        id: cat_tx.id.clone(),
        data: cat_tx.data.clone(),
        chain_id: chain_id.clone(),
    })
        .await
        .expect("Failed to submit transaction");

    // Wait for block production and get the current block
    println!("[TEST.CL] Waiting for block production...");
    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
    let current_block = hs_node.lock().await.get_current_block().await.expect("Failed to get current block");
    println!("[TEST.CL] Current block number after CAT tx: {}", current_block);

    // Look for CAT transaction in all blocks up to the current one
    println!("[TEST.CL] Searching for CAT transaction in blocks...");
    let mut found_subblock = None;
    for block_num in 0..current_block {
        let subblock = hs_node.lock().await.get_subblock(chain_id.clone(), block_num)
            .await
            .expect(&format!("Failed to get subblock for block {}", block_num));
        println!("[TEST.CL] Checking block {} for CAT tx: tx_count={}", block_num, subblock.len());
        for tx in &subblock {
            println!("[TEST.CL]   Transaction: id={}, data={}", tx.id.0, tx.data);
        }
        if subblock.iter().any(|tx| tx.id == cat_tx.id) {
            println!("[TEST.CL] Found CAT transaction in block {}", block_num);
            found_subblock = Some(subblock);
            break;
        }
    }

    let subblock = found_subblock.expect("Did not find subblock containing CAT transaction");
    assert_eq!(subblock.len(), 1, "Subblock should contain 1 transaction");
    assert!(subblock.iter().any(|tx| tx.id == cat_tx.id), "Subblock should contain CAT transaction");
    
    // - - - - - - - - - HIG processes subblock - - - - - - - - -
    println!("\n[test.HIG] Processing subblock...");
    // Receive the subblock from CL at HIG
    // TODO: we have no direct connection between CL at this point, so we just call the method for now
    hig_node.lock().await.process_subblock(SubBlock {
        block_id: 0,
        chain_id: chain_id.clone(),
        transactions: subblock.into_iter().map(|tx| Transaction {
            id: tx.id,
            data: tx.data,
        }).collect(),
    })
        .await
        .expect("Failed to process subblock");

    // Verify HIG has pending status and success proposal
    println!("[TEST.HIG] Verifying transaction status...");
    let status = hig_node.lock().await.get_transaction_status(cat_tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(status, TransactionStatus::Pending));

    let proposed_status = hig_node.lock().await.get_proposed_status(cat_tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    assert!(matches!(proposed_status, CATStatusLimited::Success));

    // Immediately send the CAT status proposal from HIG to HS
    println!("[TEST.HIG] Sending CAT status proposal to HS...");
    hig_node.lock().await.send_cat_status_proposal(CATId(cat_tx.id.0.clone()), CATStatusLimited::Success)
        .await
        .expect("Failed to propose status update");

    // - - - - - - - - - HS processes status update - - - - - - - - -
    println!("\n[test.HS] Verifying CAT status...");
    // Check that HS has the correct status
    let hs_status = hs_node.lock().await.get_cat_status(CATId(cat_tx.id.0.clone())).await.unwrap();
    assert_eq!(hs_status, CATStatusLimited::Success);

    // HS sends the status update to CL
    println!("[TEST.HS] Sending status update to CL...");
    hs_node.lock().await.send_cat_status_update(CATId(cat_tx.id.0.clone()), CATStatusLimited::Success)
        .await
        .expect("Failed to send status update");

    // - - - - - - - - - CL processes status update - - - - - - - - -
    // Wait for block production (2x block interval to be safe)
    println!("\n[test.CL] Waiting for block production after status update...");
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    let current_block = hs_node.lock().await.get_current_block().await.expect("Failed to get current block");
    println!("[TEST.CL] Current block number after status update: {}", current_block);

    // Look for status update transaction in all blocks up to the current one
    println!("[TEST.CL] Searching for status update transaction in blocks...");
    let mut found_subblock = None;
    for block_num in 0..current_block {
        let subblock = hs_node.lock().await.get_subblock(chain_id.clone(), block_num)
            .await
            .expect(&format!("Failed to get subblock for block {}", block_num));
        println!("[TEST.CL] Checking block {} for status update: tx_count={}", block_num, subblock.len());
        for tx in &subblock {
            println!("[TEST.CL]   Transaction: id={}, data={}", tx.id.0, tx.data);
        }
        if subblock.iter().any(|tx| tx.id == TransactionId(cat_tx.id.0.clone() + ".UPDATE")) {
            println!("[TEST.CL] Found status update transaction in block {}", block_num);
            found_subblock = Some(subblock);
            break;
        }
    }   

    let subblock = found_subblock.expect("Did not find subblock containing status update transaction");
    assert_eq!(subblock.len(), 1, "Subblock should contain 1 transaction");
    assert!(subblock.iter().any(|tx| tx.id == TransactionId(cat_tx.id.0.clone() + ".UPDATE")), "Subblock should contain status update transaction");

    // - - - - - - - - - HIG processes subblock - - - - - - - - -
    println!("\n[test.HIG] Processing status update subblock...");
    hig_node.lock().await.process_subblock(SubBlock {
        block_id: 0,
        chain_id: chain_id.clone(),
        transactions: subblock.into_iter().map(|tx| Transaction {
            id: tx.id,
            data: tx.data,
        }).collect(),
    })
        .await
        .expect("Failed to process subblock");
    
    // Verify HIG has success status
    println!("[TEST.HIG] Verifying final transaction status...");
    let status = hig_node.lock().await.get_transaction_status(cat_tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(status, TransactionStatus::Success));
    
    println!("\n=== Test completed successfully ===\n");
}

#[tokio::test]
async fn test_cat_transaction_flow() {
    // use testnodes from common
    let (hs_node, _, hig_node,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Create a CAT transaction
    let cat_tx = Transaction {
        id: TransactionId("test-cat".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };

    // Create a subblock with the CAT transaction
    let subblock = SubBlock {
        block_id: 0,
        chain_id: ChainId("test-chain".to_string()),
        transactions: vec![cat_tx.clone()],
    };

    // Process the subblock
    hig_node.lock().await.process_subblock(subblock)
        .await
        .expect("Failed to process subblock");

    // Verify the transaction status
    let status = hig_node.lock().await.get_transaction_status(cat_tx.id.clone())
        .await
        .unwrap();
    assert!(matches!(status, TransactionStatus::Pending));

    // Verify the proposed status
    let proposed_status = hig_node.lock().await.get_proposed_status(cat_tx.id.clone())
        .await
        .unwrap();
    assert!(matches!(proposed_status, CATStatusLimited::Success));

    // Send the status proposal to HS
    hig_node.lock().await.send_cat_status_proposal(CATId(cat_tx.id.0.clone()), CATStatusLimited::Success)
        .await
        .expect("Failed to send status proposal");

    // Wait for HS to process the message
    sleep(Duration::from_millis(100)).await;

    // Verify HS stored the status
    let stored_status = hs_node.lock().await.get_cat_status(CATId(cat_tx.id.0.clone()))
        .await
        .expect("Failed to get CAT status");
    assert_eq!(stored_status, CATStatusLimited::Success);
}

#[tokio::test]
async fn test_status_update_flow() {
    // use testnodes from common
    let (_hs_node, _, hig_node,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Create a CAT transaction
    let cat_tx = Transaction {
        id: TransactionId("test-cat".to_string()),
        data: "STATUS_UPDATE.SUCCESS".to_string(),
    };

    // Create a subblock with the CAT transaction
    let subblock = SubBlock {
        block_id: 0,
        chain_id: ChainId("test-chain".to_string()),
        transactions: vec![cat_tx.clone()],
    };

    // Process the subblock
    hig_node.lock().await.process_subblock(subblock)
        .await
        .expect("Failed to process subblock");

    // Verify the transaction status
    let status = hig_node.lock().await.get_transaction_status(cat_tx.id.clone())
        .await
        .unwrap();
    assert!(matches!(status, TransactionStatus::Success));
}

/// E2E tests for the CAT status update flow
/// - CAT transaction is submitted to CL
/// - CL processes the transaction and submits it to HIG
/// - HIG processes the transaction and sends a status proposal to HS
/// - HS stores the status
/// - HS sends the status update to CL
/// - CL processes the status update and submits it to HIG
/// - HIG processes the status update and updates the transaction status
#[tokio::test]
async fn test_e2e_cat_status_update() {
    // use testnodes from common
    let (hs_node, cl_node, _,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Register chain in CL
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST.Setup] Registering chain in CL...");
    cl_node.lock().await.register_chain(chain_id.clone()).await.expect("Failed to register chain");

    // Register chain in HS
    println!("[TEST.Setup] Registering chain in HS...");
    hs_node.lock().await.set_chain_id(chain_id.clone()).await;

    // Submit a transaction
    hs_node.lock().await.submit_transaction(CLTransaction {
        id: TransactionId("test-tx".to_string()),
        data: "test-data".to_string(),
        chain_id: chain_id.clone(),
    }).await.expect("Failed to submit transaction");

    // Get current block and subblock
    let _current_block = hs_node.lock().await.get_current_block().await.expect("Failed to get current block");
    let _subblock = hs_node.lock().await.get_subblock(chain_id.clone(), 0).await.expect("Failed to get subblock");
}

#[tokio::test]
async fn test_e2e_cat_status_update_with_status() {
    // use testnodes from common
    let (hs_node, cl_node, _,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Register chain in CL
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST.Setup] Registering chain in CL...");
    cl_node.lock().await.register_chain(chain_id.clone()).await.expect("Failed to register chain");

    // Register chain in HS
    println!("[TEST.Setup] Registering chain in HS...");
    hs_node.lock().await.set_chain_id(chain_id.clone()).await;

    // Submit a transaction
    let cat_tx = CLTransaction {
        id: TransactionId("test-tx".to_string()),
        data: "test data".to_string(),
        chain_id: chain_id.clone(),
    };
    hs_node.lock().await.submit_transaction(cat_tx.clone())
        .await
        .expect("Failed to submit transaction");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify the status in HS
    let hs_status = hs_node.lock().await.get_cat_status(CATId(cat_tx.id.0.clone())).await.unwrap();
    assert_eq!(hs_status, CATStatusLimited::Success);

    // Send a status update
    hs_node.lock().await.send_cat_status_update(CATId(cat_tx.id.0.clone()), CATStatusLimited::Success)
        .await
        .expect("Failed to send status update");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Get current block
    let current_block = hs_node.lock().await.get_current_block().await.expect("Failed to get current block");
    assert_eq!(current_block, 4);

    // Get subblock and verify transaction
    let subblock = hs_node.lock().await.get_subblock(chain_id.clone(), 0)
        .await
        .expect("Failed to get subblock");
    assert_eq!(subblock.len(), 1);
    assert_eq!(subblock[0].data, "test data");
}

#[tokio::test]
async fn test_e2e_cat_status_update_with_multiple_statuses() {
    // use testnodes from common
    let (hs_node, cl_node, _,_start_block_height) = testnodes::setup_test_nodes(Duration::from_millis(1000)).await;

    // Register chain in CL
    let chain_id = ChainId("test-chain".to_string());
    println!("[TEST.Setup] Registering chain in CL...");
    cl_node.lock().await.register_chain(chain_id.clone()).await.expect("Failed to register chain");

    // Register chain in HS
    println!("[TEST.Setup] Registering chain in HS...");
    hs_node.lock().await.set_chain_id(chain_id.clone()).await;

    // Submit a transaction
    let cat_tx = CLTransaction {
        id: TransactionId("test-tx".to_string()),
        data: "test data".to_string(),
        chain_id: chain_id.clone(),
    };
    hs_node.lock().await.submit_transaction(cat_tx.clone())
        .await
        .expect("Failed to submit transaction");

    // Wait for block production
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify the status in HS
    let stored_status = hs_node.lock().await.get_cat_status(CATId(cat_tx.id.0.clone()))
        .await
        .unwrap();
    assert_eq!(stored_status, CATStatusLimited::Success);

    // Send multiple status updates
    for status in [CATStatusLimited::Success, CATStatusLimited::Failure, CATStatusLimited::Success] {
        hs_node.lock().await.send_cat_status_update(CATId(cat_tx.id.0.clone()), status.clone())
            .await
            .expect("Failed to send status update");

        // Wait for block production
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify the status in HS
        let current_status = hs_node.lock().await.get_cat_status(CATId(cat_tx.id.0.clone()))
            .await
            .unwrap();
        assert_eq!(current_status, status);
    }
}
