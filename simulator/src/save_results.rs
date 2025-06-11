use std::fs;
use serde_json;
use std::time::Instant;
use crate::account_selection::AccountSelectionStats;
use hyperplane::utils::logging;

/// Saves the simulation results to files
/// 
/// # Arguments
///
/// * `transactions_sent` - Total number of transactions sent
/// * `successful_transactions` - Number of successful transactions
/// * `failed_transactions` - Number of failed transactions
/// * `cat_transactions` - Number of CAT transactions
/// * `regular_transactions` - Number of regular transactions
/// * `initial_balance` - Initial balance for transactions
/// * `num_accounts` - Number of accounts
/// * `target_tps` - Target transactions per second
/// * `duration_seconds` - Duration of simulation in seconds
/// * `zipf_parameter` - Zipf parameter for account selection
/// * `ratio_cats` - Ratio of CAT transactions
/// * `block_interval` - Interval between blocks
/// * `chain_delays` - Chain delays for each node
/// * `chain_1_pending` - Pending transactions for chain 1
/// * `chain_2_pending` - Pending transactions for chain 2
/// * `account_stats` - Account selection statistics
/// * `start_time` - Start time of simulation
pub async fn save_results(
    transactions_sent: u64,
    successful_transactions: u64,
    failed_transactions: u64,
    cat_transactions: u64,
    regular_transactions: u64,
    initial_balance: u64,
    num_accounts: usize,
    target_tps: u64,
    duration_seconds: u64,
    zipf_parameter: f64,
    ratio_cats: f64,
    block_interval: f64,
    chain_delays: Vec<f64>,
    chain_1_pending: Vec<(u64, u64)>,
    chain_2_pending: Vec<(u64, u64)>,
    account_stats: AccountSelectionStats,
    start_time: Instant,
) -> Result<(), String> {

    // Print final statistics
    logging::log("SIMULATOR", "\n=== Simulation Statistics ===");
    logging::log("SIMULATOR", &format!("Total Transactions: {}", transactions_sent));
    logging::log("SIMULATOR", &format!("Successful Transactions: {}", successful_transactions));
    logging::log("SIMULATOR", &format!("Failed Transactions: {}", failed_transactions));
    logging::log("SIMULATOR", &format!("CAT Transactions: {}", cat_transactions));
    logging::log("SIMULATOR", &format!("Regular Transactions: {}", regular_transactions));
    logging::log("SIMULATOR", &format!("Actual TPS: {:.2}", transactions_sent as f64 / start_time.elapsed().as_secs_f64()));
    logging::log("SIMULATOR", "===========================");
    
    // Save statistics to JSON file
    let stats = serde_json::json!({
        "parameters": {
            "initial_balance": initial_balance,
            "num_accounts": num_accounts,
            "target_tps": target_tps,
            "duration_seconds": duration_seconds,
            "zipf_parameter": zipf_parameter,
            "ratio_cats": ratio_cats,
            "block_interval": block_interval,
            "chain_delays": chain_delays
        },
        "results": {
            "total_transactions": transactions_sent,
            "successful_transactions": successful_transactions,
            "failed_transactions": failed_transactions,
            "cat_transactions": cat_transactions,
            "regular_transactions": regular_transactions
        }
    });

    // Create results directories if they don't exist
    fs::create_dir_all("simulator/results/data").expect("Failed to create results directory");

    // Save simulation stats
    let stats_file = "simulator/results/data/simulation_stats.json";
    fs::write(stats_file, serde_json::to_string_pretty(&stats).expect("Failed to serialize stats")).map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Saved simulation statistics to {}", stats_file));

    // Save pending transactions data from chain 1
    let pending_txs_chain_1 = serde_json::json!({
        "chain_1_pending": chain_1_pending.iter().map(|(height, count)| {
            serde_json::json!({
                "height": height,
                "count": count
            })
        }).collect::<Vec<_>>()
    });
    let pending_file_chain_1 = "simulator/results/data/pending_transactions_chain_1.json";
    fs::write(pending_file_chain_1, serde_json::to_string_pretty(&pending_txs_chain_1).expect("Failed to serialize pending transactions")).map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Saved pending transactions data to {}", pending_file_chain_1));

    // Save pending transactions data from chain 2
    let pending_txs_chain_2 = serde_json::json!({
        "chain_2_pending": chain_2_pending.iter().map(|(height, count)| {
            serde_json::json!({
                "height": height,
                "count": count
            })
        }).collect::<Vec<_>>()
    });
    let pending_file_chain_2 = "simulator/results/data/pending_transactions_chain_2.json";
    fs::write(pending_file_chain_2, serde_json::to_string_pretty(&pending_txs_chain_2).expect("Failed to serialize pending transactions")).map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Saved pending transactions data to {}", pending_file_chain_2));

    // Save account selection data to files
    let (sender_json, receiver_json) = account_stats.to_json();
    let sender_file = "simulator/results/data/account_sender_selection.json";
    fs::write(sender_file, serde_json::to_string_pretty(&sender_json).expect("Failed to serialize sender stats")).map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Saved sender selection data to {}", sender_file));
    let receiver_file = "simulator/results/data/account_receiver_selection.json";
    fs::write(receiver_file, serde_json::to_string_pretty(&receiver_json).expect("Failed to serialize receiver stats")).map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Saved receiver selection data to {}", receiver_file));

    Ok(())
} 