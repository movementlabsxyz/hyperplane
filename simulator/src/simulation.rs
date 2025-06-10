use std::time::Instant;
use std::sync::Arc;
use std::fs::File;
use std::io::Write;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use hyperplane::{
    confirmation_layer::node::ConfirmationLayerNode,
    confirmation_layer::ConfirmationLayer,
    types::{Transaction, TransactionId, CLTransaction, CLTransactionId, ChainId},
    utils::logging,
};
use crate::account_selector::AccountSelector;

/// Runs the simulation for the specified duration
///
/// # Arguments
///
/// * `cl_nodes` - A reference to a vector of Arc<Mutex<ConfirmationLayerNode>>, the confirmation layer nodes
/// * `account_selector` - A mutable AccountSelector instance, the account selector
/// * `target_tps` - A f64, the target TPS
/// * `duration` - A Duration, the duration of the simulation
///
pub async fn run_simulation(
    cl_nodes: &[Arc<Mutex<ConfirmationLayerNode>>],
    mut account_selector: AccountSelector,
    target_tps: f64,
    duration: Duration,
) {
    logging::log("SIMULATOR", "Starting simulation...");
    
    // Calculate time between transactions to achieve target TPS
    let interval = Duration::from_secs_f64(1.0 / target_tps);
    let start_time = Instant::now();
    let end_time = start_time + duration;
    
    // Create progress bar
    let pb = ProgressBar::new(duration.as_secs());
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} seconds ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    
    let mut transactions_sent = 0;
    let mut successful_transactions = 0;
    let mut failed_transactions = 0;
    let mut key_selection_counts = std::collections::HashMap::new();
    
    while Instant::now() < end_time {
        // Select random account
        let account = account_selector.select_account();
        
        // Track key selection
        *key_selection_counts.entry(account.clone()).or_insert(0) += 1;
        
        // Create and submit transaction
        let tx = Transaction::new(
            TransactionId(format!("sim-tx-{}", transactions_sent)),
            ChainId("chain-1".to_string()),
            vec![ChainId("chain-1".to_string())],
            format!("REGULAR.send {} {} 1", account, (account.parse::<u32>().unwrap() + 1) % 100),
            CLTransactionId(format!("cl-sim-tx-{}", transactions_sent)),
        ).expect("Failed to create transaction");

        let cl_tx = CLTransaction::new(
            CLTransactionId(format!("cl-sim-tx-{}", transactions_sent)),
            vec![ChainId("chain-1".to_string())],
            vec![tx],
        ).expect("Failed to create CL transaction");

        if let Ok(_status) = cl_nodes[0].lock().await.submit_transaction(cl_tx).await {
            transactions_sent += 1;
            successful_transactions += 1;
            logging::log("SIMULATOR", &format!("Transaction {} submitted successfully", transactions_sent));
        } else {
            failed_transactions += 1;
            logging::log("SIMULATOR", &format!("Transaction {} failed to submit", transactions_sent));
        }

        // Update progress bar
        let elapsed = start_time.elapsed();
        pb.set_position(elapsed.as_secs());
        
        // Wait for next transaction
        sleep(interval).await;
    }
    
    // Finish progress bar
    pb.finish_with_message("Simulation complete");
    
    // Calculate final statistics
    let actual_duration = start_time.elapsed();
    let actual_tps = transactions_sent as f64 / actual_duration.as_secs_f64();
    
    // Create stats object
    let stats = json!({
        "simulation_config": {
            "target_tps": target_tps,
            "duration_seconds": duration.as_secs(),
            "initial_balance": 1000,
            "num_accounts": 100
        },
        "results": {
            "total_transactions": transactions_sent,
            "successful_transactions": successful_transactions,
            "failed_transactions": failed_transactions,
            "actual_duration_seconds": actual_duration.as_secs_f64(),
            "actual_tps": actual_tps,
            "success_rate": (successful_transactions as f64 / transactions_sent as f64) * 100.0
        },
        "timestamp": chrono::Local::now().to_rfc3339()
    });

    // Write simulation stats to file
    let stats_file = "simulator/results/simulation_stats.json";
    if let Ok(mut file) = File::create(stats_file) {
        if let Err(e) = writeln!(file, "{}", serde_json::to_string_pretty(&stats).unwrap()) {
            eprintln!("Error writing stats file: {}", e);
        }
    } else {
        eprintln!("Error creating stats file");
    }

    // Create key selection stats
    let mut key_stats: Vec<_> = key_selection_counts.into_iter().collect();
    key_stats.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending

    let key_stats_json = json!({
        "key_selection_distribution": key_stats.iter().map(|(key, count)| {
            json!({
                "key": key,
                "count": count,
                "percentage": (*count as f64 / transactions_sent as f64) * 100.0
            })
        }).collect::<Vec<_>>(),
        "total_transactions": transactions_sent,
        "unique_keys": key_stats.len(),
        "timestamp": chrono::Local::now().to_rfc3339()
    });

    // Write key selection stats to file
    let key_stats_file = "simulator/results/key_selection_stats.json";
    if let Ok(mut file) = File::create(key_stats_file) {
        if let Err(e) = writeln!(file, "{}", serde_json::to_string_pretty(&key_stats_json).unwrap()) {
            eprintln!("Error writing key stats file: {}", e);
        }
    } else {
        eprintln!("Error creating key stats file");
    }
    
    // Log final statistics
    logging::log("SIMULATOR", "=== Simulation Results ===");
    logging::log("SIMULATOR", &format!("Total Transactions: {}", transactions_sent));
    logging::log("SIMULATOR", &format!("Successful Transactions: {}", successful_transactions));
    logging::log("SIMULATOR", &format!("Failed Transactions: {}", failed_transactions));
    logging::log("SIMULATOR", &format!("Actual Duration: {:.2} seconds", actual_duration.as_secs_f64()));
    logging::log("SIMULATOR", &format!("Actual TPS: {:.2}", actual_tps));
    logging::log("SIMULATOR", &format!("Success Rate: {:.2}%", (successful_transactions as f64 / transactions_sent as f64) * 100.0));
    logging::log("SIMULATOR", "========================");
} 