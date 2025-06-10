use hyperplane::confirmation_layer::ConfirmationLayer;
use hyperplane::utils::logging;
use hyperplane::types::{Transaction, TransactionId, CLTransaction, CLTransactionId, ChainId};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;
use crate::account_selector::AccountSelector;
use rand;

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
    cl_nodes: &[Arc<Mutex<hyperplane::confirmation_layer::ConfirmationLayerNode>>],
    account_selector: AccountSelector,
    target_tps: f64,
    duration: Duration,
) {
    let start_time = Instant::now();
    let end_time = start_time + duration;
    let mut transactions_sent = 0;
    let mut successful_transactions = 0;
    let mut failed_transactions = 0;
    let mut key_selection_counts = std::collections::HashMap::new();

    // Log simulation start with Zipf parameter
    logging::log("SIMULATOR", &format!("Starting simulation with Zipf parameter: {}", account_selector.get_zipf_parameter()));

    // Create progress bar
    let pb = ProgressBar::new(duration.as_secs());
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} seconds ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    while Instant::now() < end_time {
        let elapsed = Instant::now() - start_time;
        pb.set_position(elapsed.as_secs());

        // Calculate delay to maintain target TPS
        let target_interval = Duration::from_secs_f64(1.0 / target_tps);
        let elapsed_since_last = if transactions_sent > 0 {
            elapsed - Duration::from_secs_f64((transactions_sent as f64) / target_tps)
        } else {
            Duration::from_secs(0)
        };

        if elapsed_since_last < target_interval {
            sleep(target_interval - elapsed_since_last).await;
        }

        // Select random node and account
        let node_idx = rand::random::<usize>() % cl_nodes.len();
        let account = account_selector.select_account();  // This is now 1-based
        
        // Track key selection
        *key_selection_counts.entry(account).or_insert(0) += 1;

        // Create and submit transaction
        let tx = Transaction::new(
            TransactionId(format!("sim-tx-{}", transactions_sent)),
            ChainId("chain-1".to_string()),
            vec![ChainId("chain-1".to_string())],
            format!("REGULAR.send {} {} 1", account - 1, account % 100),  // Convert to 0-based for the transaction
            CLTransactionId(format!("cl-sim-tx-{}", transactions_sent)),
        ).expect("Failed to create transaction");

        let cl_tx = CLTransaction::new(
            CLTransactionId(format!("cl-sim-tx-{}", transactions_sent)),
            vec![ChainId("chain-1".to_string())],
            vec![tx],
        ).expect("Failed to create CL transaction");

        let result = cl_nodes[node_idx].lock().await.submit_transaction(cl_tx).await;

        match result {
            Ok(_) => {
                successful_transactions += 1;
                logging::log("SIMULATOR", &format!("Transaction submitted successfully: {}", transactions_sent));
            }
            Err(e) => {
                failed_transactions += 1;
                logging::log("SIMULATOR", &format!("Failed to submit transaction: {} - Error: {}", transactions_sent, e));
            }
        }

        transactions_sent += 1;
    }

    pb.finish_with_message("Simulation complete");

    // Calculate final statistics
    let actual_duration = Instant::now() - start_time;
    let actual_tps = transactions_sent as f64 / actual_duration.as_secs_f64();

    // Log final statistics
    logging::log("SIMULATOR", "=== Simulation Results ===");
    logging::log("SIMULATOR", &format!("Zipf Parameter: {}", account_selector.get_zipf_parameter()));
    logging::log("SIMULATOR", &format!("Total Transactions: {}", transactions_sent));
    logging::log("SIMULATOR", &format!("Successful Transactions: {}", successful_transactions));
    logging::log("SIMULATOR", &format!("Failed Transactions: {}", failed_transactions));
    logging::log("SIMULATOR", &format!("Actual Duration: {:.2} seconds", actual_duration.as_secs_f64()));
    logging::log("SIMULATOR", &format!("Actual TPS: {:.2}", actual_tps));
    logging::log("SIMULATOR", "=======================");

    // Write simulation statistics to file
    let mut sorted_distribution: Vec<_> = key_selection_counts.into_iter().collect();
    sorted_distribution.sort_by_key(|(key, _)| *key);

    // Format distribution as an array of objects
    let distribution_array: Vec<_> = sorted_distribution
        .iter()
        .map(|(key, count)| serde_json::json!({
            "key": key,
            "count": count
        }))
        .collect();

    let stats = serde_json::json!({
        "zipf_parameter": account_selector.get_zipf_parameter(),
        "total_transactions": transactions_sent,
        "successful_transactions": successful_transactions,
        "failed_transactions": failed_transactions,
        "actual_duration_seconds": actual_duration.as_secs_f64(),
        "actual_tps": actual_tps,
        "key_selection_distribution": distribution_array
    });

    // Create results directory if it doesn't exist
    std::fs::create_dir_all("simulator/results").expect("Failed to create results directory");

    // Write simulation stats
    let stats_file = std::fs::File::create("simulator/results/simulation_stats.json")
        .expect("Failed to create simulation stats file");
    serde_json::to_writer_pretty(stats_file, &stats)
        .expect("Failed to write simulation stats");
} 