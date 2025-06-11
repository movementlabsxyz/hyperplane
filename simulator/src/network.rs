use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use hyperplane::{
    types::{ChainId, TransactionId, Transaction, CLTransaction, CLTransactionId, SubBlock},
    confirmation_layer::{ConfirmationLayerNode, ConfirmationLayer},
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

/// Initializes accounts with the specified initial balance
pub async fn initialize_accounts(nodes: &[Arc<Mutex<ConfirmationLayerNode>>], initial_balance: u64, num_accounts: usize) {
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
                    logging::log("SIMULATOR", &format!("Account {} credited successfully: {}", account_id, tx.data));
                } else {
                    logging::log("SIMULATOR", &format!("Failed to credit account {}: {}", account_id, tx.data));
                }
            }
            logging::log("SIMULATOR", &format!("Chain {} account initialization complete", chain_id.0));
        }
    }
    logging::log("SIMULATOR", "All accounts initialized");
} 