use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::io::Write;
use hyperplane::{
    types::{ChainId, TransactionId, Transaction, CLTransaction, CATStatusUpdate, SubBlock},
    confirmation_layer::{ConfirmationLayerNode, ConfirmationLayer},
    hyper_scheduler::node::HyperSchedulerNode,
    hyper_ig::node::HyperIGNode,
};

async fn run_cl_node(_cl_node: Arc<Mutex<ConfirmationLayerNode>>) {
    // Implement the CL node loop here
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn run_hs_node(_hs_node: Arc<Mutex<HyperSchedulerNode>>) {
    // Implement the HS node loop here
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() {
    println!("=== Hyperplane Shell ===");
    println!("Type 'help' for commands.");

    // Set up channel for HS <-> CL
    let (sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel::<CLTransaction>(100);

    // Set block time to 1 second
    let block_time = Duration::from_secs(1);

    // Initialize nodes
    let cl_node = Arc::new(Mutex::new(ConfirmationLayerNode::new_with_block_interval(receiver_hs_to_cl, block_time).unwrap()));
    let hs_node = Arc::new(Mutex::new(HyperSchedulerNode::new(sender_hs_to_cl)));

    // Store HIG nodes by chain_id
    let hig_nodes: Arc<Mutex<HashMap<ChainId, Arc<Mutex<HyperIGNode>>>>> = Arc::new(Mutex::new(HashMap::new()));

    // Spawn node tasks (replace with your actual node loops)
    tokio::spawn(run_cl_node(cl_node.clone()));
    tokio::spawn(run_hs_node(hs_node.clone()));

    // Start REPL
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    while let Ok(Some(line)) = {
        print!("> ");
        std::io::stdout().flush().unwrap();
        lines.next_line().await
    } {
        let input = line.trim();
        if input == "exit" || input == "quit" {
            println!("Exiting shell.");
            break;
        }
        if input == "help" {
            println!("Commands:\n  add-chain <chain_id>\n  send-tx <chain_id> <data>\n  send-cat <chain_id1,chain_id2,...> <data>\n  status\n  exit");
            println!("\nValid transaction data formats:");
            println!("  Regular: REGULAR.SIMULATION:Success or REGULAR.SIMULATION:Failure");
            println!("  Dependent: DEPENDENT.SIMULATION:Success.CAT_ID:<id> or DEPENDENT.SIMULATION:Failure.CAT_ID:<id>");
            println!("  CAT: CAT.SIMULATION:Success.CAT_ID:<id> or CAT.SIMULATION:Failure.CAT_ID:<id>");
            println!("  Status Update: STATUS_UPDATE:Success.CAT_ID:<id> or STATUS_UPDATE:Failure.CAT_ID:<id>");
            println!("\nExamples:");
            println!("  send-tx chain1 REGULAR.SIMULATION:Success");
            println!("  send-cat chain1,chain2 CAT.SIMULATION:Success.CAT_ID:cat123");
            println!("  send-tx chain1 DEPENDENT.SIMULATION:Success.CAT_ID:cat123");
            println!("  send-tx chain1 STATUS_UPDATE:Success.CAT_ID:cat123");
            continue;
        }
        let mut parts = input.split_whitespace();
        match parts.next() {
            Some("status") => {
                let chains = hig_nodes.lock().await;
                println!("=== System Status ===");
                println!("Registered chains: {}", chains.len());
                for (chain_id, _) in chains.iter() {
                    println!("  - {}", chain_id.0);
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
                    // Start HIG node message loop
                    tokio::spawn(HyperIGNode::start(hig_node));
                    println!("[shell] Chain {} registered successfully.", chain_id.0);
                } else {
                    println!("Usage: add-chain <chain_id>");
                }
            }
            Some("send-tx") => {
                if let (Some(chain_id), Some(data)) = (parts.next(), parts.next()) {
                    let data = data.trim_matches('"');  // Remove quotes if present
                    println!("[shell] Sending tx to {}: {}", chain_id, data);
                    match Transaction::new(
                        TransactionId(format!("tx-{}", chain_id)),
                        ChainId(chain_id.to_string()),
                        vec![ChainId(chain_id.to_string())],
                        data.to_string(),
                    ) {
                        Ok(tx) => {
                            match CLTransaction::new(
                                TransactionId(format!("tx-{}", chain_id)),
                                vec![ChainId(chain_id.to_string())],
                                vec![tx],
                            ) {
                                Ok(cl_tx) => {
                                    let mut cl_node_guard = cl_node.lock().await;
                                    if let Err(e) = cl_node_guard.submit_transaction(cl_tx).await {
                                        println!("[shell] Error: Failed to submit transaction: {}", e);
                                    } else {
                                        println!("[shell] Transaction sent successfully.");
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
                    println!("[shell] Sending CAT to [{}]: {}", chains, data);
                    let chain_ids: Vec<ChainId> = chains.split(',').map(|c| ChainId(c.to_string())).collect();
                    match Transaction::new(
                        TransactionId("cat-tx".to_string()),
                        chain_ids[0].clone(),
                        chain_ids.clone(),
                        data.to_string(),
                    ) {
                        Ok(tx) => {
                            match CLTransaction::new(
                                TransactionId("cat-tx".to_string()),
                                chain_ids.clone(),
                                vec![tx],
                            ) {
                                Ok(cl_tx) => {
                                    let mut cl_node_guard = cl_node.lock().await;
                                    if let Err(e) = cl_node_guard.submit_transaction(cl_tx).await {
                                        println!("[shell] Error: Failed to submit CAT transaction: {}", e);
                                    } else {
                                        println!("[shell] CAT transaction sent successfully.");
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
    }
} 