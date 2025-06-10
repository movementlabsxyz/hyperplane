use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use indicatif::{ProgressBar, ProgressStyle};
use hyperplane::{
    types::{TransactionId, Transaction, CLTransaction, CLTransactionId},
    confirmation_layer::{ConfirmationLayerNode, ConfirmationLayer},
    utils::logging,
};
use crate::account_selector::AccountSelector;
use std::collections::HashMap;
use rand::Rng;
use serde_json;
use std::fs;

/// Runs the simulation for the specified duration
///
/// # Arguments
///
/// * `nodes` - A reference to a vector of Arc<Mutex<ConfirmationLayerNode>>, the confirmation layer nodes
/// * `duration_seconds` - A u64, the duration of the simulation in seconds
/// * `initial_balance` - A u64, the initial balance for transactions
/// * `num_accounts` - A usize, the number of accounts
/// * `target_tps` - A u64, the target TPS
/// * `zipf_parameter` - A f64, the Zipf parameter for account selection
/// * `chain_delays` - A Vec<f64>, the chain delays for each node
/// * `block_interval` - A f64, the interval between blocks
/// * `ratio_cats` - A f64, the ratio of CAT transactions
///
pub async fn run_simulation(
    nodes: Vec<Arc<Mutex<ConfirmationLayerNode>>>,
    duration_seconds: u64,
    _initial_balance: u64,
    num_accounts: usize,
    target_tps: u64,
    zipf_parameter: f64,
    _chain_delays: Vec<f64>,
    block_interval: f64,
    ratio_cats: f64,
) -> Result<(), String> {
    // Wait for initialization transactions to be processed
    logging::log("SIMULATOR", "Waiting 5 block for initialization transactions to be processed...");
    let wait = Duration::from_secs_f64(block_interval * 5.0);
    sleep(wait).await;
    logging::log("SIMULATOR", "Initialization complete, starting simulation");

    // Create account selector for transaction simulation
    let account_selector = AccountSelector::new(num_accounts, zipf_parameter);
    
    // Calculate transaction interval based on target TPS
    let transaction_interval = Duration::from_nanos(1_000_000_000 / target_tps);
    
    // Initialize statistics
    let mut transactions_sent = 0;
    let mut successful_transactions = 0;
    let mut failed_transactions = 0;
    let mut key_selection_counts = HashMap::new();
    let mut cat_transactions = 0;
    let mut regular_transactions = 0;
    
    // Create progress bar
    let pb = ProgressBar::new(duration_seconds);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} seconds")
        .unwrap()
        .progress_chars("##-"));
    
    let start_time = Instant::now();
    let mut rng = rand::thread_rng();
    
    // Main simulation loop
    while (Instant::now() - start_time).as_secs() < duration_seconds {
        // Select account for transaction using Zipf distribution
        let from_account = account_selector.select_account();
        *key_selection_counts.entry(from_account).or_insert(0) += 1;
        
        // Get registered chains for the first node (they should be the same for all nodes)
        let chains = nodes[0].lock().await.get_registered_chains().await.map_err(|e| e.to_string())?;
        
        // Determine if this should be a CAT transaction based on configured ratio
        let is_cat = rng.gen_bool(ratio_cats);
        let (chain_id, target_chain_id) = if is_cat {
            // For CAT, randomly select two different chains
            let idx1 = rng.gen_range(0..chains.len());
            let mut idx2 = rng.gen_range(0..chains.len());
            while idx2 == idx1 {
                idx2 = rng.gen_range(0..chains.len());
            }
            let source_chain = chains[idx1].clone();
            let target_chain = chains[idx2].clone();
            (source_chain.clone(), target_chain)
        } else {
            // For regular transaction, use the first chain
            let chain = chains[0].clone();
            (chain.clone(), chain)
        };
        
        let tx_data = if is_cat {
            format!("CAT.send {} {} 1", from_account, (from_account % num_accounts) + 1)
        } else {
            format!("REGULAR.send {} {} 1", from_account, (from_account % num_accounts) + 1)
        };
        
        logging::log("SIMULATOR", &format!("Creating {} transaction from account {} to account {} on chain {}", 
            if is_cat { "CAT" } else { "REGULAR" },
            from_account, 
            (from_account % num_accounts) + 1,
            chain_id.0
        ));
        logging::log("SIMULATOR", &format!("Transaction data: '{}'", tx_data));
        
        // Create and submit transaction
        let cl_id = CLTransactionId(format!("cl-tx_{}", transactions_sent));
        let tx = Transaction::new(
            TransactionId(format!("{:?}:tx", cl_id)),
            chain_id.clone(),
            vec![chain_id.clone(), target_chain_id.clone()], // Include both chains in constituent chains
            tx_data,
            cl_id.clone(),
        ).map_err(|e| {
            logging::log("SIMULATOR", &format!("Failed to create transaction: {}", e));
            logging::log("SIMULATOR", &format!("Transaction details: from={}, to={}, amount=1, chain={}, is_cat={}, target_chain={}", 
                from_account,
                (from_account % num_accounts) + 1,
                chain_id.0,
                is_cat,
                target_chain_id.0
            ));
            e.to_string()
        })?;

        logging::log("SIMULATOR", &format!("Created transaction: {}", tx.data));

        let cl_tx = CLTransaction::new(
            cl_id.clone(),
            vec![target_chain_id.clone()],
            vec![tx.clone()],
        ).map_err(|e| {
            logging::log("SIMULATOR", &format!("Failed to create CL transaction: {}", e));
            e.to_string()
        })?;

        logging::log("SIMULATOR", &format!("Created CL transaction with ID: {:?}", cl_id));

        // Submit transaction to all nodes
        let mut success = true;
        for (i, node) in nodes.iter().enumerate() {
            logging::log("SIMULATOR", &format!("Submitting transaction to node {}", i + 1));
            if let Err(e) = node.lock().await.submit_transaction(cl_tx.clone()).await {
                logging::log("SIMULATOR", &format!("Failed to submit transaction to node {}: {}", i + 1, e));
                success = false;
                break;
            }
            logging::log("SIMULATOR", &format!("Successfully submitted transaction to node {}", i + 1));
        }

        if success {
            successful_transactions += 1;
            if is_cat {
                cat_transactions += 1;
            } else {
                regular_transactions += 1;
            }
            logging::log("SIMULATOR", &format!("Transaction submitted successfully: {}", tx.data));
        } else {
            failed_transactions += 1;
            logging::log("SIMULATOR", &format!("Transaction failed: {}", tx.data));
        }
        
        transactions_sent += 1;
        pb.set_position((Instant::now() - start_time).as_secs());
        
        // Sleep for a short duration to prevent busy waiting
        sleep(transaction_interval).await;
    }
    
    // Print final statistics
    logging::log("SIMULATOR", "\n=== Simulation Results ===");
    logging::log("SIMULATOR", &format!("Total Transactions Sent: {}", transactions_sent));
    logging::log("SIMULATOR", &format!("Successful Transactions: {}", successful_transactions));
    logging::log("SIMULATOR", &format!("Failed Transactions: {}", failed_transactions));
    logging::log("SIMULATOR", &format!("Success Rate: {:.2}%", 
        (successful_transactions as f64 / transactions_sent as f64) * 100.0));
    logging::log("SIMULATOR", &format!("CAT Transactions: {} ({:.2}%)", 
        cat_transactions,
        (cat_transactions as f64 / transactions_sent as f64) * 100.0));
    logging::log("SIMULATOR", &format!("Regular Transactions: {} ({:.2}%)", 
        regular_transactions,
        (regular_transactions as f64 / transactions_sent as f64) * 100.0));
    
    // Print account selection distribution
    logging::log("SIMULATOR", "\nAccount Selection Distribution:");
    let mut sorted_counts: Vec<_> = key_selection_counts.into_iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(&a.1));
    for (account, count) in sorted_counts.iter().take(10) {
        logging::log("SIMULATOR", &format!("Account {}: {} transactions", account, count));
    }

    // Save statistics to JSON file
    let stats = serde_json::json!({
        "parameters": {
            "initial_balance": _initial_balance,
            "num_accounts": num_accounts,
            "target_tps": target_tps,
            "duration_seconds": duration_seconds,
            "zipf_parameter": zipf_parameter,
            "ratio_cats": ratio_cats,
            "block_interval": block_interval,
            "chain_delays": _chain_delays
        },
        "results": {
            "total_transactions": transactions_sent,
            "successful_transactions": successful_transactions,
            "failed_transactions": failed_transactions,
            "success_rate": (successful_transactions as f64 / transactions_sent as f64) * 100.0,
            "cat_transactions": {
                "count": cat_transactions,
                "percentage": (cat_transactions as f64 / transactions_sent as f64) * 100.0
            },
            "regular_transactions": {
                "count": regular_transactions,
                "percentage": (regular_transactions as f64 / transactions_sent as f64) * 100.0
            },
            "account_selection_distribution": sorted_counts.iter().take(10).map(|(account, count)| {
                serde_json::json!({
                    "account": account,
                    "transactions": count
                })
            }).collect::<Vec<_>>()
        }
    });

    // Create results directory if it doesn't exist
    fs::create_dir_all("simulator/results").expect("Failed to create results directory");
    
    // Write stats to file
    fs::write(
        "simulator/results/simulation_stats.json",
        serde_json::to_string_pretty(&stats).expect("Failed to serialize stats")
    ).expect("Failed to write stats file");
    
    Ok(())
} 