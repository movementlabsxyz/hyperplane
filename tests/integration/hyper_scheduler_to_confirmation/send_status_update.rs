/// Integration: HS sends a status update message for two chains, CL verifies the message is queued and subblocks contain it
#[tokio::test]
async fn test_hs_to_cl_status_update() {
    let mut hs = HyperSchedulerNode::new();
    let mut cl = ConfirmationNode::with_block_interval(Duration::from_millis(100))
        .expect("Failed to create confirmation node");

    // Register chains 1 and 2 in the confirmation layer
    let chain1 = ChainId("chain1".to_string());
    let chain2 = ChainId("chain2".to_string());
    cl.register_chain(chain1.clone()).await.expect("Failed to register chain1");
    cl.register_chain(chain2.clone()).await.expect("Failed to register chain2");

    // Connect HS to CL
    hs.set_confirmation_layer(Box::new(cl.clone()));
    
    // Set chain IDs in HS
    hs.set_chain_id(chain1.clone());
    hs.set_chain_id(chain2.clone());

    // Create a status update message for chains 1 and 2
    let cat_id1 = CATId("cat1".to_string());
    let cat_id2 = CATId("cat2".to_string());
    let status = CATStatusLimited::Success;

    // HS sends the status update message for chain1
    hs.send_cat_status_update(cat_id1.clone(), status.clone())
        .await
        .expect("Failed to send status update for chain1");

    // HS sends the status update message for chain2
    hs.send_cat_status_update(cat_id2.clone(), status.clone())
        .await
        .expect("Failed to send status update for chain2");

    // Wait for block production (2x block interval to be safe)
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify that subblocks for both chains contain the status update message
    let subblock1 = hs.confirmation_layer().unwrap().get_subblock(chain1.clone(), BlockId("0".to_string())).await.expect("Failed to get subblock for chain1");
    let subblock2 = hs.confirmation_layer().unwrap().get_subblock(chain2.clone(), BlockId("0".to_string())).await.expect("Failed to get subblock for chain2");

    assert!(subblock1.transactions.iter().any(|tx| tx.id == TransactionId(cat_id1.0.clone())));
    assert!(subblock2.transactions.iter().any(|tx| tx.id == TransactionId(cat_id2.0.clone())));
} 