use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use hyperplane::{
    types::{ChainId, TransactionId, Transaction, CLTransaction, CLTransactionId, SubBlock},
    confirmation_layer::{ConfirmationLayerNode, ConfirmationLayer},
    hyper_ig::HyperIG,
    utils::logging,
};

// ------------------------------------------------------------------------------------------------
// Network Setup
// ------------------------------------------------------------------------------------------------


/// Creates a network of nodes with the specified number of nodes and chains
pub async fn create_network(num_nodes: usize, num_chains: usize) -> Vec<Arc<Mutex<ConfirmationLayerNode>>> {
    let mut nodes = Vec::new();
    let mut senders = Vec::new();
    
    // Create nodes
    for _i in 0..num_nodes {
        let (tx, rx) = mpsc::channel::<CLTransaction>(100);
        let node = ConfirmationLayerNode::new(rx);
        nodes.push(Arc::new(Mutex::new(node)));
        senders.push(tx);
    }
    
    // Register chains on each node
    for i in 0..num_chains {
        let chain_id = ChainId(format!("chain-{}", i));
        for (node, _sender) in nodes.iter().zip(senders.iter()) {
            let mut node = node.lock().await;
            let (subblock_tx, _) = mpsc::channel::<SubBlock>(100);
            node.register_chain(chain_id.clone(), subblock_tx).await.expect("Failed to register chain");
        }
    }
    
    nodes
}

// ------------------------------------------------------------------------------------------------
// Account Initialization
// ------------------------------------------------------------------------------------------------

/// Initializes accounts with the specified initial balance and verifies the balances
pub async fn initialize_accounts(
    nodes: &[Arc<Mutex<ConfirmationLayerNode>>], 
    initial_balance: u64, 
    num_accounts: usize,
    hig_nodes: Option<&[Arc<Mutex<hyperplane::hyper_ig::node::HyperIGNode>>]>,
    block_interval: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    logging::log("SIMULATOR", &format!("Initializing accounts with {} tokens each...", initial_balance));

    for node in nodes {
        let mut node = node.lock().await;
        
        // Get registered chains
        let chains = node.get_registered_chains().await.expect("Failed to get registered chains");
        
        // Initialize accounts on each chain
        for chain_id in chains {
            logging::log("SIMULATOR", &format!("Initializing accounts for chain {}...", chain_id.0));
            
            // Create accounts with initial balance sequentially
            for account_id in 1..=num_accounts {
                let tx = Transaction::new(
                    TransactionId(format!("init-credit-{}", account_id)),
                    chain_id.clone(),
                    vec![chain_id.clone()],
                    format!("REGULAR.credit {} {}", account_id, initial_balance),
                    CLTransactionId(format!("init-credit-{}", account_id)),
                ).expect("Failed to create transaction");

                let cl_tx = CLTransaction::new(
                    CLTransactionId(format!("init-credit-{}", account_id)),
                    vec![chain_id.clone()],
                    vec![tx.clone()],
                ).expect("Failed to create CL transaction");

                if let Ok(_status) = node.submit_transaction(cl_tx).await {
                    logging::log("SIMULATOR", &format!("Account {} credit submitted to CL node: {}", account_id, tx.data));
                } else {
                    logging::log("SIMULATOR", &format!("Failed to submit credit transaction for account {}: {}", account_id, tx.data));
                }
            }
            logging::log("SIMULATOR", &format!("Chain {} all credit transactions submitted to CL node", chain_id.0));
        }
    }
    logging::log("SIMULATOR", "All credit transactions submitted to CL nodes");

    // If HIG nodes are provided, wait for funding to complete and verify balances
    if let Some(hig_nodes) = hig_nodes {
        // Wait for funding transactions to complete
        // Wait for 5 blocks to ensure accounts are funded
        let wait_blocks = 5;
        
        let current_block = nodes[0].lock().await.get_current_block().await.map_err(|e| e.to_string())?;
        let funding_target_block = current_block + wait_blocks;
        
        logging::log("SIMULATOR", &format!("Waiting for {} funding transactions to complete...", num_accounts * 2));
        logging::log("SIMULATOR", &format!("Block interval: {:.3}s, Waiting for {} blocks ({:.1}s)...", 
            block_interval, wait_blocks, block_interval * wait_blocks as f64));
        
        // Wait for funding transactions to complete
        loop {
            let current_block = nodes[0].lock().await.get_current_block().await.map_err(|e| e.to_string())?;
            logging::log("SIMULATOR", &format!("Waiting for funding transactions to complete... Current block: {}", current_block));
            if current_block >= funding_target_block {
                break;
            }
            // Sleep for the block interval before checking again
            tokio::time::sleep(std::time::Duration::from_secs_f64(block_interval)).await;
        }
        
        // Verify that all accounts have been funded correctly
        logging::log("SIMULATOR", "================================================");
        logging::log("SIMULATOR", "VERIFYING ACCOUNT BALANCES AFTER FUNDING...");
        logging::log("SIMULATOR", "================================================");
        let expected_balance = initial_balance as i64;
        
        for (chain_index, hig_node) in hig_nodes.iter().enumerate() {
            let chain_state = hig_node.lock().await.get_chain_state().await.map_err(|e| e.to_string())?;
            logging::log("SIMULATOR", &format!("Chain {} state: {:?}", chain_index + 1, chain_state));
            
            // Check that all accounts from 1 to num_accounts have the expected balance
            for account_id in 1..=num_accounts {
                let account_key = account_id.to_string();
                let actual_balance = chain_state.get(&account_key).copied().unwrap_or(0);
                
                if actual_balance != expected_balance {
                    let error_msg = format!(
                        "Account {} on chain {} has incorrect balance: expected {}, got {}",
                        account_id, chain_index + 1, expected_balance, actual_balance
                    );
                    logging::log("SIMULATOR", "================================================");
                    logging::log("SIMULATOR", &format!("ERROR: {}", error_msg));
                    logging::log("SIMULATOR", "================================================");
                    return Err(error_msg.into());
                }
            }
            
            logging::log("SIMULATOR", &format!("✓ Chain {} account verification successful - all {} accounts have balance {}", 
                chain_index + 1, num_accounts, expected_balance));
        }
        
        logging::log("SIMULATOR", "================================================");
        logging::log("SIMULATOR", "✓ ALL ACCOUNT BALANCES VERIFIED SUCCESSFULLY!");
        logging::log("SIMULATOR", &format!("✓ All {} accounts on both chains have balance {}", num_accounts, expected_balance));
        logging::log("SIMULATOR", "✓ Funding transactions completed successfully");
        logging::log("SIMULATOR", "================================================");
    }

    Ok(())
} 