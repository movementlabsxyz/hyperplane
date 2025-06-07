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
    types::constants::{chain_1, chain_2, chain_3},
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
    // Initialize logging
    hyperplane::utils::logging::init_logging();

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

    // Create 3 default chains
    println!("[shell] Creating 3 default chains...");
    let default_chains = [chain_1(), chain_2(), chain_3()];
    for chain_id in default_chains {
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
            println!("Commands:");
            println!("  add-chain <chain_id>");
            println!("  send-tx <chain_id> <data>");
            println!("  send-cat <chain_id1,chain_id2,...> <data>");
            println!("  status");
            println!("  exit");
            println!("\nValid transaction data formats:");
            println!("  Regular: credit <account> <amount>");
            println!("  Regular: send <from> <to> <amount>");
            println!("  CAT: CAT.send <from> <to> <amount>.CAT_ID:<id>");
            println!("  CAT: CAT.credit <account> <amount>.CAT_ID:<id>");
            println!("\nExamples:");
            println!("  send-tx chain-1 credit 1 100");
            println!("  send-tx chain-1 send 1 2 50");
            println!("  send-cat chain-1,chain-2 CAT.send 1 2 50.CAT_ID:cat123");
            println!("  send-cat chain-1,chain-2 CAT.credit 1 100.CAT_ID:cat123");
            continue;
        }
        let mut parts = input.split_whitespace();
        match parts.next() {
            Some("status") => {
                let chains = hig_nodes.lock().await;
                let transactions = transaction_tracker.lock().await;
                println!("=== System Status ===");
                let chain_list: Vec<String> = chains.keys().map(|c| c.0.clone()).collect();
                println!("Registered chains: {}", chain_list.join(", "));
                
                // Show state for each chain
                println!("\nChain States:");
                for (chain_id, node) in chains.iter() {
                    let state = node.lock().await.get_chain_state().await.unwrap_or_default();
                    println!("  {}: {:?}", chain_id.0, state);
                }
                
                println!("\nTransaction Status:");
                
                // Collect all chain nodes and transaction IDs first
                let chain_nodes: Vec<(ChainId, Arc<Mutex<HyperIGNode>>)> = chains.iter()
                    .map(|(id, node)| (id.clone(), node.clone()))
                    .collect();
                let tx_ids: Vec<TransactionId> = transactions.transactions.keys().cloned().collect();
                
                // Release the locks before making async calls
                drop(chains);
                drop(transactions);

                // Process each transaction
                for tx_id in tx_ids {
                    println!("  - {}:", tx_id.0);
                    // Process each chain
                    for (chain_id, node) in &chain_nodes {
                        let node = node.lock().await;
                        if let Ok(status) = node.get_resolution_status(tx_id.clone()).await {
                            if let Ok(data) = node.get_transaction_data(tx_id.clone()).await {
                                println!("    {}: {:?} : {}", chain_id.0, status, data);
                            }
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
                if let (Some(chain_id), Some(_data)) = (parts.next(), parts.next()) {
                    // Get the rest of the input as the full data
                    let data = input.split_once("send-tx").unwrap().1
                        .split_once(chain_id).unwrap().1
                        .trim_start();
                    let data = data.trim_matches('"');  // Remove quotes if present
                    // Generate unique transaction ID with timestamp
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    let tx_id = TransactionId(format!("tx-{}-{}", chain_id, timestamp));
                    println!("[shell] Sending tx to {}: {}", chain_id, data);
                    match Transaction::new(
                        tx_id.clone(),
                        ChainId(chain_id.to_string()),
                        vec![ChainId(chain_id.to_string())],
                        format!("REGULAR.{}", data),  // Add REGULAR. prefix for regular transactions
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
                if let (Some(chains), Some(_data)) = (parts.next(), parts.next()) {
                    // Get the rest of the input as the full data
                    let data = input.split_once("send-cat").unwrap().1
                        .split_once(chains).unwrap().1
                        .trim_start();
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
                    
                    // Create a transaction for each chain
                    let mut transactions = Vec::new();
                    for chain_id in &chain_ids {
                        match Transaction::new(
                            cat_id.clone(),
                            chain_id.clone(),
                            chain_ids.clone(),
                            data.to_string(),  // Use the data as is, without adding REGULAR. prefix
                        ) {
                            Ok(tx) => transactions.push(tx),
                            Err(e) => {
                                println!("[shell] Error: Failed to create transaction for chain {}: {}", chain_id.0, e);
                                continue;
                            }
                        }
                    }

                    if !transactions.is_empty() {
                        match CLTransaction::new(
                            cat_id.clone(),
                            chain_ids.clone(),
                            transactions,
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
                    } else {
                        println!("[shell] Error: No valid transactions were created");
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