use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::io::Write;
use hyperplane::{
    types::{ChainId, TransactionId, Transaction, CLTransaction, CATStatusUpdate, SubBlock, TransactionStatus},
    confirmation_layer::{ConfirmationLayerNode, ConfirmationLayer},
    hyper_scheduler::node::HyperSchedulerNode,
    hyper_ig::node::HyperIGNode,
    hyper_ig::HyperIG,
};

// Store transaction statuses
struct TransactionTracker {
    transactions: HashMap<TransactionId, TransactionStatus>,
}

impl TransactionTracker {
    fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    fn add_transaction(&mut self, tx_id: TransactionId) {
        self.transactions.insert(tx_id, TransactionStatus::Pending);
    }
}

#[tokio::main]
async fn main() {
    println!("=== Hyperplane Shell ===");
    println!("Type 'help' for commands.");
    print!("> ");
    std::io::stdout().flush().unwrap();

    // Set up channel for HS <-> CL
    let (sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel::<CLTransaction>(100);

    // Set block time to 1 second
    let block_time = Duration::from_secs(1);

    // Initialize nodes
    let cl_node = Arc::new(Mutex::new(ConfirmationLayerNode::new_with_block_interval(receiver_hs_to_cl, block_time).unwrap()));
    let hs_node = Arc::new(Mutex::new(HyperSchedulerNode::new(sender_hs_to_cl)));

    // Store HIG nodes by chain_id
    let hig_nodes: Arc<Mutex<HashMap<ChainId, Arc<Mutex<HyperIGNode>>>>> = Arc::new(Mutex::new(HashMap::new()));
    
    // Initialize transaction tracker
    let transaction_tracker = Arc::new(Mutex::new(TransactionTracker::new()));

    // Start the nodes
    ConfirmationLayerNode::start(cl_node.clone()).await;
    HyperSchedulerNode::start(hs_node.clone()).await;

    // Start REPL
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let input = line.trim();
        if input == "exit" || input == "quit" {
            println!("Exiting shell.");
            break;
        }
        if input == "help" {
            println!("Commands:\n  add-chain <chain_id>\n  default-chains <n>\n  send-tx <chain_id> <data>\n  send-cat <chain_id1,chain_id2,...> <data>\n  status\n  exit");
            println!("\nValid transaction data formats:");
            println!("  Regular: REGULAR.SIMULATION:Success or REGULAR.SIMULATION:Failure");
            println!("  Dependent: DEPENDENT.SIMULATION:Success.CAT_ID:<id> or DEPENDENT.SIMULATION:Failure.CAT_ID:<id>");
            println!("  CAT: CAT.SIMULATION:Success.CAT_ID:<id> or CAT.SIMULATION:Failure.CAT_ID:<id>");
            println!("  Status Update: STATUS_UPDATE:Success.CAT_ID:<id> or STATUS_UPDATE:Failure.CAT_ID:<id>");
            println!("\nExamples:");
            println!("  default-chains 3  # Creates chain1, chain2, chain3");
            println!("  send-tx chain1 REGULAR.SIMULATION:Success");
            println!("  send-cat chain1,chain2 CAT.SIMULATION:Success.CAT_ID:cat123");
            println!("  send-tx chain1 DEPENDENT.SIMULATION:Success.CAT_ID:cat123");
            println!("  send-tx chain1 STATUS_UPDATE:Success.CAT_ID:cat123");
            continue;
        }
        let mut parts = input.split_whitespace();
        match parts.next() {
            Some("default-chains") => {
                if let Some(n_str) = parts.next() {
                    match n_str.parse::<usize>() {
                        Ok(n) if n > 0 => {
                            println!("[shell] Creating {} default chains...", n);
                            for i in 1..=n {
                                let chain_id = ChainId(format!("chain{}", i));
                                println!("[shell] Adding chain: {}", chain_id.0);
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
                                println!("[shell] Chain {} registered successfully.", chain_id.0);
                            }
                        }
                        Ok(_) => println!("[shell] Error: Number of chains must be positive"),
                        Err(_) => println!("[shell] Error: Invalid number of chains"),
                    }
                } else {
                    println!("Usage: default-chains <n>");
                }
            }
            Some("status") => {
                let chains = hig_nodes.lock().await;
                let transactions = transaction_tracker.lock().await;
                println!("=== System Status ===");
                println!("Registered chains: {}", chains.len());
                for (chain_id, _) in chains.iter() {
                    println!("  - {}", chain_id.0);
                }
                println!("\nTransaction Status:");
                for (tx_id, _) in transactions.transactions.iter() {
                    println!("  - {}:", tx_id.0);
                    // Show status for each chain
                    for (chain_id, hig_node) in chains.iter() {
                        match hig_node.lock().await.get_resolution_status(tx_id.clone()).await {
                            Ok(status) => println!("    {}: {:?}", chain_id.0, status),
                            Err(_) => println!("    {}: Unknown", chain_id.0),
                        }
                    }
                }
                println!("===================");
            }
            Some("add-chain") => {
                if let Some(chain_id_str) = parts.next() {
                    let chain_id = ChainId(chain_id_str.to_string());
                    println!("[shell] Adding chain: {}", chain_id.0);
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
                    println!("[shell] Chain {} registered successfully.", chain_id.0);
                } else {
                    println!("Usage: add-chain <chain_id>");
                }
            }
            Some("send-tx") => {
                if let (Some(chain_id), Some(data)) = (parts.next(), parts.next()) {
                    let data = data.trim_matches('"');  // Remove quotes if present
                    let tx_id = TransactionId(format!("tx-{}", chain_id));
                    println!("[shell] Sending tx to {}: {}", chain_id, data);
                    match Transaction::new(
                        tx_id.clone(),
                        ChainId(chain_id.to_string()),
                        vec![ChainId(chain_id.to_string())],
                        data.to_string(),
                    ) {
                        Ok(tx) => {
                            match CLTransaction::new(
                                tx_id.clone(),
                                vec![ChainId(chain_id.to_string())],
                                vec![tx],
                            ) {
                                Ok(cl_tx) => {
                                    let mut cl_node_guard = cl_node.lock().await;
                                    if let Err(e) = cl_node_guard.submit_transaction(cl_tx).await {
                                        println!("[shell] Error: Failed to submit transaction: {}", e);
                                    } else {
                                        transaction_tracker.lock().await.add_transaction(tx_id.clone());
                                        println!("[shell] Transaction sent successfully. ID: {}", tx_id.0);
                                    }
                                }
                                Err(e) => println!("[shell] Error: Failed to create CL transaction: {}", e),
                            }
                        }
                        Err(e) => println!("[shell] Error: Failed to create transaction: {}", e),
                    }
                } else {
                    println!("Usage: send-tx <chain_id> <data>");
                }
            }
            Some("send-cat") => {
                if let (Some(chains), Some(data)) = (parts.next(), parts.next()) {
                    let data = data.trim_matches('"');  // Remove quotes if present
                    // Extract CAT ID from the data
                    let cat_id = if let Some(cat_id_start) = data.find("CAT_ID:") {
                        let cat_id = &data[cat_id_start + 7..];
                        TransactionId(cat_id.to_string())
                    } else {
                        TransactionId("cat-tx".to_string())
                    };
                    println!("[shell] Sending CAT to [{}]: {}", chains, data);
                    let chain_ids: Vec<ChainId> = chains.split(',').map(|c| ChainId(c.to_string())).collect();
                    match Transaction::new(
                        cat_id.clone(),
                        chain_ids[0].clone(),
                        chain_ids.clone(),
                        data.to_string(),
                    ) {
                        Ok(tx) => {
                            match CLTransaction::new(
                                cat_id.clone(),
                                chain_ids.clone(),
                                vec![tx],
                            ) {
                                Ok(cl_tx) => {
                                    let mut cl_node_guard = cl_node.lock().await;
                                    if let Err(e) = cl_node_guard.submit_transaction(cl_tx).await {
                                        println!("[shell] Error: Failed to submit CAT transaction: {}", e);
                                    } else {
                                        transaction_tracker.lock().await.add_transaction(cat_id.clone());
                                        println!("[shell] CAT transaction sent successfully. ID: {}", cat_id.0);
                                    }
                                }
                                Err(e) => println!("[shell] Error: Failed to create CL transaction: {}", e),
                            }
                        }
                        Err(e) => println!("[shell] Error: Failed to create transaction: {}", e),
                    }
                } else {
                    println!("Usage: send-cat <chain_id1,chain_id2,...> <data>");
                }
            }
            Some(cmd) => {
                println!("Unknown command: {}", cmd);
            }
            None => {}
        }
        print!("> ");
        std::io::stdout().flush().unwrap();
    }
} 