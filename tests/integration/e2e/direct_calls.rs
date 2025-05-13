use hyperplane::{
    types::{Transaction, TransactionId, TransactionStatus, CATStatusLimited, ChainId, BlockId, CLTransaction, CATId},
    hyper_ig::HyperIG,
    hyper_scheduler::HyperScheduler,
    confirmation_layer::ConfirmationLayer,
};
use crate::common::testnodes;
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
    let (mut hs_node, mut cl_node, mut hig_node) = testnodes::setup_test_nodes();

    // Register chain in CL
    let chain_id = ChainId("test-chain".to_string());
    println!("[test.Setup] Registering chain in CL...");
    cl_node.register_chain(chain_id.clone()).await.expect("Failed to register chain");

    // Connect components
    println!("[test.Setup] Connecting components...");
    hs_node.set_confirmation_layer(Box::new(cl_node));

    // Register chain in HS
    println!("[test.Setup] Registering chain in HS...");
    hs_node.set_chain_id(chain_id.clone());

    // - - - - - - - - - CL processes CAT transaction - - - - - - - - -
    println!("\n[test.CL] Submitting CAT transaction...");
    // Create and submit CAT transaction to CL
    let cat_tx = Transaction {
        id: TransactionId("cat-tx".to_string()),
        data: "CAT.SIMULATION.SUCCESS".to_string(),
    };
    hs_node.confirmation_layer_mut().unwrap().submit_transaction(CLTransaction {
        id: cat_tx.id.clone(),
        data: cat_tx.data.clone(),
        chain_id: chain_id.clone(),
    })
        .await
        .expect("Failed to submit transaction");

    // Wait for block production and get the current block
    println!("[test.CL] Waiting for block production...");
    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
    let current_block = hs_node.confirmation_layer_mut().unwrap().get_current_block().await.expect("Failed to get current block");
    let current_block_num = current_block.0.parse::<u64>().unwrap();
    println!("[test.CL] Current block number after CAT tx: {}", current_block_num);

    // Look for CAT transaction in all blocks up to the current one
    println!("[test.CL] Searching for CAT transaction in blocks...");
    let mut found_subblock = None;
    for block_num in 0..current_block_num {
        let block_id = BlockId(block_num.to_string());
        let subblock = hs_node.confirmation_layer_mut().unwrap().get_subblock(chain_id.clone(), block_id.clone())
            .await
            .expect(&format!("Failed to get subblock for block {}", block_num));
        println!("[test.CL] Checking block {} for CAT tx: tx_count={}", block_num, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("[test.CL]   Transaction: id={}, data={}", tx.id.0, tx.data);
        }
        if subblock.transactions.iter().any(|tx| tx.id == cat_tx.id) {
            println!("[test.CL] Found CAT transaction in block {}", block_num);
            found_subblock = Some(subblock);
            break;
        }
    }

    let subblock = found_subblock.expect("Did not find subblock containing CAT transaction");
    assert_eq!(subblock.transactions.len(), 1, "Subblock should contain 1 transaction");
    assert!(subblock.transactions.iter().any(|tx| tx.id == cat_tx.id), "Subblock should contain CAT transaction");
    
    // - - - - - - - - - HIG processes subblock - - - - - - - - -
    println!("\n[test.HIG] Processing subblock...");
    // Receive the subblock from CL at HIG
    // TODO: we have no direct connection between CL at this point, so we just call the method for now
    hig_node.process_subblock(subblock)
        .await
        .expect("Failed to process subblock");

    // Verify HIG has pending status and success proposal
    println!("[test.HIG] Verifying transaction status...");
    let status = hig_node.get_transaction_status(cat_tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(status, TransactionStatus::Pending));

    let proposed_status = hig_node.get_proposed_status(cat_tx.id.clone())
        .await
        .expect("Failed to get proposed status");
    assert!(matches!(proposed_status, CATStatusLimited::Success));

    // Immediately send the CAT status proposal from HIG to HS
    println!("[test.HIG] Sending CAT status proposal to HS...");
    hig_node.send_cat_status_proposal(CATId(cat_tx.id.0.clone()), CATStatusLimited::Success)
        .await
        .expect("Failed to propose status update");

    // - - - - - - - - - HS processes status update - - - - - - - - -
    println!("\n[test.HS] Verifying CAT status...");
    // Check that HS has the correct status
    let hs_status = hs_node.get_cat_status(CATId(cat_tx.id.0.clone())).await.unwrap();
    assert_eq!(hs_status, CATStatusLimited::Success);

    // HS sends the status update to CL
    println!("[test.HS] Sending status update to CL...");
    hs_node.send_cat_status_update(CATId(cat_tx.id.0.clone()), CATStatusLimited::Success)
        .await
        .expect("Failed to send status update");

    // - - - - - - - - - CL processes status update - - - - - - - - -
    // Wait for block production (2x block interval to be safe)
    println!("\n[test.CL] Waiting for block production after status update...");
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    let current_block = hs_node.confirmation_layer_mut().unwrap().get_current_block().await.expect("Failed to get current block");
    let current_block_num = current_block.0.parse::<u64>().unwrap();
    println!("[test.CL] Current block number after status update: {}", current_block_num);

    // Look for status update transaction in all blocks up to the current one
    println!("[test.CL] Searching for status update transaction in blocks...");
    let mut found_subblock = None;
    for block_num in 0..current_block_num {
        let block_id = BlockId(block_num.to_string());
        let subblock = hs_node.confirmation_layer_mut().unwrap().get_subblock(chain_id.clone(), block_id.clone())
            .await
            .expect(&format!("Failed to get subblock for block {}", block_num));
        println!("[test.CL] Checking block {} for status update: tx_count={}", block_num, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("[test.CL]   Transaction: id={}, data={}", tx.id.0, tx.data);
        }
        if subblock.transactions.iter().any(|tx| tx.id == TransactionId(cat_tx.id.0.clone() + ".UPDATE")) {
            println!("[test.CL] Found status update transaction in block {}", block_num);
            found_subblock = Some(subblock);
            break;
        }
    }   

    let subblock = found_subblock.expect("Did not find subblock containing status update transaction");
    assert_eq!(subblock.transactions.len(), 1, "Subblock should contain 1 transaction");
    assert!(subblock.transactions.iter().any(|tx| tx.id == TransactionId(cat_tx.id.0.clone() + ".UPDATE")), "Subblock should contain status update transaction");

    // - - - - - - - - - HIG processes subblock - - - - - - - - -
    println!("\n[test.HIG] Processing status update subblock...");
    hig_node.process_subblock(subblock)
        .await
        .expect("Failed to process subblock");
    
    // Verify HIG has success status
    println!("[test.HIG] Verifying final transaction status...");
    let status = hig_node.get_transaction_status(cat_tx.id.clone())
        .await
        .expect("Failed to get transaction status");
    assert!(matches!(status, TransactionStatus::Success));
    
    println!("\n=== Test completed successfully ===\n");
}
