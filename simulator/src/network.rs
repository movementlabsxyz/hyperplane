use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use hyperplane::{
    types::{ChainId, TransactionId, Transaction, CLTransaction, CATStatusUpdate, SubBlock, CLTransactionId},
    confirmation_layer::{ConfirmationLayerNode, ConfirmationLayer},
    hyper_scheduler::node::HyperSchedulerNode,
    hyper_ig::node::HyperIGNode,
    types::constants::{chain_1, chain_2},
    utils::logging,
};
use std::collections::HashMap;

// ------------------------------------------------------------------------------------------------
// Network Setup
// ------------------------------------------------------------------------------------------------

/// Sets up the network nodes (CL, HS, HIG) with appropriate channels and configurations
pub async fn setup_nodes() -> Vec<Arc<Mutex<ConfirmationLayerNode>>> {
    logging::log("SIMULATOR", "Setting up network nodes...");

    // Create channels
    let (sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel::<CLTransaction>(100);

    // Initialize nodes
    let cl_node = Arc::new(Mutex::new(ConfirmationLayerNode::new_with_block_interval(receiver_hs_to_cl, Duration::from_secs(1)).unwrap()));
    let hs_node = Arc::new(Mutex::new(HyperSchedulerNode::new(sender_hs_to_cl)));

    // Store HIG nodes by chain_id
    let hig_nodes: Arc<Mutex<HashMap<ChainId, Arc<Mutex<HyperIGNode>>>>> = Arc::new(Mutex::new(HashMap::new()));

    // Start the nodes
    ConfirmationLayerNode::start(cl_node.clone()).await;
    HyperSchedulerNode::start(hs_node.clone()).await;

    // Create 2 default chains
    logging::log("SIMULATOR", "Creating 2 default chains...");
    let default_chains = [chain_1(), chain_2()];
    for chain_id in default_chains {
        logging::log("SIMULATOR", &format!("Adding chain: {}", chain_id.0));
        // Channels for CL <-> HIG
        let (sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel::<SubBlock>(100);
        // Channels for HIG <-> HS
        let (sender_hig_to_hs, receiver_hig_to_hs) = mpsc::channel::<CATStatusUpdate>(100);
        // Create HIG node
        let hig_node = Arc::new(Mutex::new(HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs, chain_id.clone())));
        // Register chain with CL
        let mut cl_node_guard = cl_node.lock().await;
        cl_node_guard.register_chain(chain_id.clone(), sender_cl_to_hig).await.expect("Failed to register chain with CL");
        drop(cl_node_guard);
        // Register chain with HS
        let mut hs_node_guard = hs_node.lock().await;
        hs_node_guard.register_chain(chain_id.clone(), receiver_hig_to_hs).await.expect("Failed to register chain with HS");
        drop(hs_node_guard);
        // Store HIG node
        hig_nodes.lock().await.insert(chain_id.clone(), hig_node.clone());
        // Start HIG node
        HyperIGNode::start(hig_node).await;
        logging::log("SIMULATOR", &format!("Chain {} registered successfully.", chain_id.0));
    }

    logging::log("SIMULATOR", "Network setup complete");
    vec![cl_node]
}

/// Initializes accounts on each chain with the specified initial balance
pub async fn initialize_accounts(cl_nodes: &[Arc<Mutex<ConfirmationLayerNode>>], initial_balance: i64) {
    logging::log("SIMULATOR", &format!("Initializing accounts with {} tokens each...", initial_balance));

    for (i, node) in cl_nodes.iter().enumerate() {
        let chain_id = ChainId(format!("chain-{}", i + 1));
        logging::log("SIMULATOR", &format!("Initializing accounts for chain {}...", chain_id.0));
        
        // Create 100 accounts with initial balance
        for account_id in 1..=100 {
            let tx = Transaction::new(
                TransactionId(format!("credit-{}", account_id)),
                chain_id.clone(),
                vec![chain_id.clone()],
                format!("REGULAR.credit {} {}", account_id, initial_balance),
                CLTransactionId(format!("cl-credit-{}", account_id)),
            ).expect("Failed to create transaction");

            let cl_tx = CLTransaction::new(
                CLTransactionId(format!("cl-credit-{}", account_id)),
                vec![chain_id.clone()],
                vec![tx],
            ).expect("Failed to create CL transaction");

            let mut node_guard = node.lock().await;
            if let Ok(_status) = node_guard.submit_transaction(cl_tx).await {
                logging::log("SIMULATOR", &format!("Account {} created successfully", account_id));
            } else {
                logging::log("SIMULATOR", &format!("Failed to create account {}", account_id));
            }
        }
        logging::log("SIMULATOR", &format!("Chain {} account initialization complete", chain_id.0));
    }
    logging::log("SIMULATOR", "All accounts initialized");
} 