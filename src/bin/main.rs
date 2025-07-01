use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::io::Write;
use hyperplane::{
    types::{ChainId, TransactionId, Transaction, CLTransaction, CATStatusUpdate, SubBlock, TransactionStatus, CLTransactionId},
    confirmation_layer::{ConfirmationLayerNode, ConfirmationLayer, ConfirmationLayerError},
    hyper_scheduler::node::HyperSchedulerNode,
    hyper_ig::node::HyperIGNode,
    hyper_ig::HyperIG,
    types::constants::{chain_1, chain_2, chain_3},
};

mod config;

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

    // Initialize nodes
    let cl_node = Arc::new(Mutex::new(ConfirmationLayerNode::new_with_block_interval(receiver_hs_to_cl, config::BLOCK_TIME).unwrap()));
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
        let hig_node = Arc::new(Mutex::new(HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs, chain_id.clone(), config::CAT_MAX_LIFETIME_BLOCKS, config::ALLOW_CAT_PENDING_DEPENDENCIES)));
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
            println!("  set-delay <chain_id> <milliseconds>");
            println!("  set-block-interval <milliseconds>");
            println!("  status");
            println!("  exit");
            println!("\nValid transaction data formats:");
            println!("  Regular: credit <account> <amount>");
            println!("  Regular: send <from> <to> <amount>");
            println!("  CAT: CAT.send <from> <to> <amount>");
            println!("  CAT: CAT.credit <account> <amount>");
            println!("\nExamples:");
            println!("  send-tx chain-1 credit 1 100");
            println!("  send-tx chain-1 send 1 2 50");
            println!("  send-cat chain-1,chain-2 CAT.send 1 2 50");
            println!("  send-cat chain-1,chain-2 CAT.credit 1 100");
            println!("  set-delay chain-1 200");
            println!("  set-block-interval 500");
            println!("\n⚠️  CONFIGURATION NOTE:");
            println!("  Some settings (like CAT lifetime, allow_cat_pending_dependencies)");
            println!("  must be changed in src/bin/config.rs and require restarting the shell.");
            println!("  Check the config file for available options.");
            println!(" ");
            continue;
        }
        let mut parts = input.split_whitespace();
        match parts.next() {
            Some("set-delay") => {
                if let (Some(chain_id_str), Some(ms_str)) = (parts.next(), parts.next()) {
                    if let Ok(ms) = ms_str.parse::<u64>() {
                        let chain_id = ChainId(chain_id_str.to_string());
                        let hig_nodes_guard = hig_nodes.lock().await;
                        if let Some(node) = hig_nodes_guard.get(&chain_id) {
                            node.lock().await.set_hs_message_delay(Duration::from_millis(ms));
                            println!("[shell] Set message delay for chain {} to {}ms", chain_id.0, ms);
                        } else {
                            println!("[shell] Error: Chain {} not found", chain_id.0);
                        }
                    } else {
                        println!("[shell] Error: Invalid milliseconds value");
                    }
                } else {
                    println!("Usage: set-delay <chain_id> <milliseconds>");
                }
            }
            Some("set-block-interval") => {
                if let Some(ms_str) = parts.next() {
                    if let Ok(ms) = ms_str.parse::<u64>() {
                        let mut cl_node_guard = cl_node.lock().await;
                        if let Err(e) = cl_node_guard.set_block_interval(Duration::from_millis(ms)).await {
                            println!("[shell] Error: Failed to set block interval: {}", e);
                        } else {
                            println!("[shell] Set CL block interval to {}ms", ms);
                        }
                    } else {
                        println!("[shell] Error: Invalid milliseconds value");
                    }
                } else {
                    println!("Usage: set-block-interval <milliseconds>");
                }
            }
            Some("status") => {
                let chains = hig_nodes.lock().await;
                let transactions = transaction_tracker.lock().await;
                println!("=== System Status ===");
                let mut chain_list: Vec<String> = chains.keys().map(|c| c.0.clone()).collect();
                chain_list.sort();  // Sort chains alphabetically
                println!("Registered chains: {}", chain_list.join(", "));
                
                // Display configuration information
                println!("\nConfiguration:");
                
                // Get block time from CL node
                let block_time_ms = match cl_node.lock().await.get_block_interval().await {
                    Ok(interval) => interval.as_millis() as u64,
                    Err(_) => config::BLOCK_TIME_MILLISECONDS, // fallback to config
                };
                println!("  Block Time: {}ms", block_time_ms);
                
                // Get CAT timeout from one of the HIG nodes
                let cat_timeout_blocks = if let Some((_, node)) = chains.iter().next() {
                    match node.lock().await.get_cat_lifetime().await {
                        Ok(lifetime) => lifetime,
                        Err(_) => config::CAT_MAX_LIFETIME_BLOCKS, // fallback to config
                    }
                } else {
                    config::CAT_MAX_LIFETIME_BLOCKS // fallback to config
                };
                println!("  CAT Max Lifetime: {} blocks ({}ms)", cat_timeout_blocks, cat_timeout_blocks * block_time_ms);
                
                // Get CL block time and interval
                let cl_block = cl_node.lock().await.get_current_block().await.unwrap();
                let cl_interval = cl_node.lock().await.get_block_interval().await.unwrap();
                println!("\nCL Block Height: {} (Interval: {}ms)", cl_block, cl_interval.as_millis());
                
                // Show state for each chain
                println!("\nChain States:");
                let mut chain_states: Vec<_> = chains.iter().collect();
                chain_states.sort_by(|a, b| a.0.0.cmp(&b.0.0));  // Sort by chain ID
                for (chain_id, node) in chain_states {
                    let state = node.lock().await.get_chain_state().await.unwrap_or_default();
                    let delay = node.lock().await.get_hs_message_delay().as_millis();
                    // Convert state to sorted string representation
                    let mut sorted_state: Vec<_> = state.iter().collect();
                    sorted_state.sort_by(|a, b| a.0.cmp(b.0));  // Sort by account ID
                    let state_str = format!("{{{}}}", sorted_state.iter()
                        .map(|(k, v)| format!("\"{}\": {}", k, v))
                        .collect::<Vec<_>>()
                        .join(", "));
                    println!("  {}: {} (delay: {}ms)", chain_id.0, state_str, delay);
                }
                
                println!("\nTransaction Status:");
                
                // Collect all chain nodes and transaction IDs first
                let chain_nodes: Vec<(ChainId, Arc<Mutex<HyperIGNode>>)> = chains.iter()
                    .map(|(id, node)| (id.clone(), node.clone()))
                    .collect();
                let mut tx_ids: Vec<TransactionId> = transactions.transactions.keys()
                    .cloned()
                    .collect();
                
                // Sort by transaction ID string for consistent display
                tx_ids.sort_by(|a, b| a.0.cmp(&b.0));
                
                // Release the locks before making async calls
                drop(chains);
                drop(transactions);

                // Process each transaction
                for tx_id in tx_ids {
                    // Extract CL ID from transaction ID by removing the ":tx" suffix
                    let cl_id = if tx_id.0.ends_with(":tx") {
                        &tx_id.0[..tx_id.0.len()-3]
                    } else {
                        &tx_id.0
                    };
                    println!("  - {}:", cl_id);
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
                println!(" ");
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
                    let hig_node = Arc::new(Mutex::new(HyperIGNode::new(receiver_cl_to_hig, sender_hig_to_hs, chain_id.clone(), config::CAT_MAX_LIFETIME_BLOCKS, config::ALLOW_CAT_PENDING_DEPENDENCIES)));
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
                    let cl_id = CLTransactionId(format!("cl-tx_{}", timestamp));
                    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
                    println!("[shell] Sending tx to {}: {}", chain_id, data);
                    match Transaction::new(
                        tx_id.clone(),
                        ChainId(chain_id.to_string()),
                        vec![ChainId(chain_id.to_string())],
                        format!("REGULAR.{}", data),  // Add REGULAR. prefix for regular transactions
                        cl_id.clone(),
                    ) {
                        Ok(tx) => {
                            match CLTransaction::new(
                                cl_id.clone(),
                                vec![ChainId(chain_id.to_string())],
                                vec![tx],
                            ) {
                                Ok(cl_tx) => {
                                    let mut cl_node_guard = cl_node.lock().await;
                                    if let Err(e) = cl_node_guard.submit_transaction(cl_tx).await {
                                        match e {
                                            ConfirmationLayerError::TransactionAlreadyProcessed(id) => {
                                                println!("[shell] Error: Transaction rejected - transaction {} has already been processed", id);
                                            }
                                            _ => {
                                                println!("[shell] Error: Failed to submit transaction: {}", e);
                                            }
                                        }
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
                    
                    // Generate unique CAT ID with timestamp
                    let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                    let cl_id = CLTransactionId(format!("cl-tx_cat_{}", timestamp));
                    // construct the transaction id
                    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
                    println!("[shell] Sending CAT to [{}]: {}", chains, data);
                    let chain_ids: Vec<ChainId> = chains.split(',').map(|c| ChainId(c.to_string())).collect();
                    
                    // Create a transaction for each chain
                    let mut transactions = Vec::new();
                    for chain_id in &chain_ids {
                        match Transaction::new(
                            tx_id.clone(),
                            chain_id.clone(),
                            chain_ids.clone(),
                            data.to_string(),  // Use the data as is, without adding REGULAR. prefix
                            cl_id.clone(),
                        ) {
                            Ok(tx) => transactions.push(tx),
                            Err(e) => {
                                println!("[shell] Error: Failed to create transaction for chain {}: {}", chain_id.0, e);
                                continue;
                            }
                        }
                    }

                    // TODO check and explain the following again
                    if !transactions.is_empty() {
                        match CLTransaction::new(
                            cl_id.clone(),
                            chain_ids.clone(),
                            transactions,
                        ) {
                            Ok(cl_tx) => {
                                let mut cl_node_guard = cl_node.lock().await;
                                if let Err(e) = cl_node_guard.submit_transaction(cl_tx).await {
                                    println!("[shell] Error: Failed to submit CAT transaction: {}", e);
                                } else {
                                    let tx_id = TransactionId(format!("{}:tx", cl_id.0));
                                    transaction_tracker.lock().await.add_transaction(tx_id);
                                    println!("[shell] CAT transaction sent successfully. CL-ID: '{}'", cl_id.0);
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
