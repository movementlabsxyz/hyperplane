use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use indicatif::{ProgressBar, ProgressStyle};
use hyperplane::{
    types::{TransactionId, Transaction, CLTransaction, CLTransactionId, ChainId},
    confirmation_layer::{ConfirmationLayerNode, ConfirmationLayer},
    hyper_ig::node::HyperIGNode,
    utils::logging,
};
use crate::account_selector::AccountSelector;
use rand::Rng;
use serde_json;
use std::fs;
use crate::account_selection::AccountSelectionStats;

/// Runs the simulation for the specified duration
///
/// # Arguments
///
/// * `nodes` - A tuple containing two vectors:
///   - The first vector contains Arc<Mutex<ConfirmationLayerNode>>, the confirmation layer nodes
///   - The second vector contains Arc<Mutex<HyperIGNode>>, the HyperIG nodes
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
    cl_node: Arc<Mutex<ConfirmationLayerNode>>,
    hig_nodes: Vec<Arc<Mutex<HyperIGNode>>>,
    duration_seconds: u64,
    initial_balance: u64,
    num_accounts: usize,
    target_tps: u64,
    zipf_parameter: f64,
    chain_delays: Vec<f64>,
    block_interval: f64,
    ratio_cats: f64,
) -> Result<(), String> {
    
    // Wait for initialization transactions to be processed
    logging::log("SIMULATOR", "Waiting 5 block for initialization transactions to be processed...");
    let wait = Duration::from_secs_f64(block_interval * 5.0);
    sleep(wait).await;
    logging::log("SIMULATOR", "Initialization complete, starting simulation");

    // Initialize random number generator
    let mut rng = rand::thread_rng();
    
    // Initialize sender account selector with uniform distribution
    let account_selector_sender = AccountSelector::new(num_accounts, 0.0);    
    // Initialize receiver account selector with Zipf distribution
    let account_selector_receiver = AccountSelector::new(num_accounts, zipf_parameter);
    
    // Initialize account statistics
    let mut account_stats = AccountSelectionStats::new();
    
    // Calculate total number of transactions to send
    let total_transactions = target_tps * duration_seconds;
    
    // Create progress bar
    let pb = ProgressBar::new(total_transactions);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} transactions ({eta})")
        .unwrap()
        .progress_chars("##-"));

    // Set HIG delays
    for (i, delay) in chain_delays.iter().enumerate() {
        hig_nodes[i].lock().await.set_hs_message_delay(Duration::from_secs_f64(*delay));
    }
    
    // Initialize counters
    let mut transactions_sent = 0;
    let mut successful_transactions = 0;
    let mut failed_transactions = 0;
    let mut cat_transactions = 0;
    let mut regular_transactions = 0;
    let mut pending_transactions_by_height = Vec::new();
    let mut current_block = 0;
    let mut last_pending_count = 0;
    
    // Main simulation loop
    let start_time = std::time::Instant::now();
    while transactions_sent < total_transactions {
        // Select accounts for transaction
        let from_account = account_selector_sender.select_account(&mut rng);
        let to_account = account_selector_receiver.select_account(&mut rng);
        
        // Record transaction in account statistics
        account_stats.record_transaction(from_account as u64, to_account as u64);
        
        // Get registered chains
        let chains = cl_node.lock().await.get_registered_chains().await.map_err(|e| e.to_string())?;
        let chain_id_1 = chains[0].clone();
        let chain_id_2 = chains[1].clone();
        
        // Determine if this should be a CAT transaction based on configured ratio
        let is_cat = rng.gen_bool(ratio_cats);
        
        // Create transaction data
        let tx_data = format!("{}.send {} {} 1", 
            if is_cat { "CAT" } else { "REGULAR" },
            from_account,
            to_account
        );
        
        logging::log("SIMULATOR", &format!("Transaction data: '{}'", tx_data));

        // Create and submit transaction
        let cl_id = CLTransactionId(format!("cl-tx_{}", transactions_sent));
        
        let (success, _) = if is_cat {
            create_and_submit_cat_transaction(
                &cl_node,
                cl_id,
                chain_id_1,
                chain_id_2,
                tx_data,
            ).await?
        } else {
            create_and_submit_regular_transaction(
                &cl_node,
                cl_id,
                chain_id_1,
                chain_id_2,
                tx_data,
            ).await?
        };

        if success {
            successful_transactions += 1;
            if is_cat {
                cat_transactions += 1;
            } else {
                regular_transactions += 1;
            }
        } else {
            failed_transactions += 1;
        }
        
        transactions_sent += 1;
        pb.inc(1);
        
        // Get current block height and pending transactions
        let new_block = cl_node.lock().await.get_current_block().await.map_err(|e| e.to_string())?;
        let pending_txs = cl_node.lock().await.get_pending_transactions().await.map_err(|e| e.to_string())?;
        
        // Only record pending count if we've moved to a new block
        if new_block != current_block {
            if current_block > 0 {  // Don't record for block 0
                pending_transactions_by_height.push((current_block, last_pending_count));
            }
            current_block = new_block;
        }
        last_pending_count = pending_txs;
        
        // Calculate sleep time to maintain target TPS
        let elapsed = start_time.elapsed();
        let target_milliseconds = (transactions_sent as f64 / target_tps as f64) * 1000.0;
        let target_elapsed = Duration::from_millis(target_milliseconds as u64);
        if elapsed < target_elapsed {
            tokio::time::sleep(target_elapsed - elapsed).await;
        }
    }
    
    // Add the final pending count for the last block
    if current_block > 0 {
        pending_transactions_by_height.push((current_block, last_pending_count));
    }
    
    // Get sorted account selection counts
    let (_sorted_sender_counts, _sorted_receiver_counts) = account_stats.get_sorted_counts();
    
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

    // Save pending transactions data
    let pending_txs = serde_json::json!({
        "pending_transactions_by_height": pending_transactions_by_height.iter().map(|(block, pending)| {
            serde_json::json!({
                "block": block,
                "pending_count": pending
            })
        }).collect::<Vec<_>>()
    });
    let pending_file = "simulator/results/data/pending_transactions.json";
    fs::write(pending_file, serde_json::to_string_pretty(&pending_txs).expect("Failed to serialize pending transactions")).map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Saved pending transactions data to {}", pending_file));

    // Save account selection data to separate files
    let (sender_json, receiver_json) = account_stats.to_json();
    
    let sender_file = "simulator/results/data/account_sender_selection.json";
    fs::write(sender_file, serde_json::to_string_pretty(&sender_json).expect("Failed to serialize sender stats")).map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Saved sender selection data to {}", sender_file));

    let receiver_file = "simulator/results/data/account_receiver_selection.json";
    fs::write(receiver_file, serde_json::to_string_pretty(&receiver_json).expect("Failed to serialize receiver stats")).map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Saved receiver selection data to {}", receiver_file));
    
    Ok(())
}

/// Creates and submits a CAT transaction
/// 
/// # Arguments
///
/// * `cl_node` - A reference to the confirmation layer node
/// * `cl_id` - A CLTransactionId, the ID of the CL transaction
/// * `chain_id_1` - A ChainId, the ID of the first chain
/// * `chain_id_2` - A ChainId, the ID of the second chain
/// * `tx_data` - A String, the data of the transaction
async fn create_and_submit_cat_transaction(
    cl_node: &Arc<Mutex<ConfirmationLayerNode>>,
    cl_id: CLTransactionId,
    chain_id_1: ChainId,
    chain_id_2: ChainId,
    tx_data: String,
) -> Result<(bool, String), String> {
    // Create transactions for both chains
    let tx1 = Transaction::new(
        TransactionId(format!("{:?}:tx1", cl_id)),
        chain_id_1.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        tx_data.clone(),
        cl_id.clone(),
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create transaction: {}", e));
        e.to_string()
    })?;

    let tx2 = Transaction::new(
        TransactionId(format!("{:?}:tx2", cl_id)),
        chain_id_2.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        tx_data.clone(),
        cl_id.clone(),
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create transaction: {}", e));
        e.to_string()
    })?;

    // Create the CL transaction
    let cl_tx = CLTransaction::new(
        cl_id.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        vec![tx1, tx2],
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create CL transaction: {}", e));
        e.to_string()
    })?;

    logging::log("SIMULATOR", &format!("Created CL transaction with ID: {:?}", cl_id));

    // Submit transaction to CL node
    match cl_node.lock().await.submit_transaction(cl_tx.clone()).await {
        Ok(_) => {
            logging::log("SIMULATOR", &format!("CAT transaction submitted successfully: {}", tx_data));
            Ok((true, tx_data))
        }
        Err(e) => {
            logging::log("SIMULATOR", &format!("Failed to submit CAT transaction: {}", e));
            logging::log("SIMULATOR", &format!("CAT transaction failed: {}", tx_data));
            Ok((false, tx_data))
        }
    }
}

/// Creates and submits a regular transaction
///
/// # Arguments
///
/// * `cl_node` - A reference to the confirmation layer node
/// * `cl_id` - A CLTransactionId, the ID of the CL transaction
/// * `chain_id_1` - A ChainId, the ID of the first chain
/// * `chain_id_2` - A ChainId, the ID of the second chain
async fn create_and_submit_regular_transaction(
    cl_node: &Arc<Mutex<ConfirmationLayerNode>>,
    cl_id: CLTransactionId,
    chain_id_1: ChainId,
    chain_id_2: ChainId,
    tx_data: String,
) -> Result<(bool, String), String> {
    // Create and submit CL transaction for chain-1
    let cl_id_1 = CLTransactionId(format!("{:?}_1", cl_id));
    let tx_1 = Transaction::new(
        TransactionId(format!("{:?}_1:tx", cl_id.clone())),
        chain_id_1.clone(),
        vec![chain_id_1.clone()],
        tx_data.clone(),
        cl_id_1.clone(),
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create transaction: {}", e));
        e.to_string()
    })?;

    let cl_tx_1 = CLTransaction::new(
        cl_id_1.clone(),
        vec![chain_id_1.clone()],
        vec![tx_1],
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create CL transaction: {}", e));
        e.to_string()
    })?;

    // Create and submit CL transaction for chain-2
    let cl_id_2 = CLTransactionId(format!("{:?}_2", cl_id));
    let tx_2 = Transaction::new(
        TransactionId(format!("{:?}:tx", cl_id_2.clone())),
        chain_id_2.clone(),
        vec![chain_id_2.clone()],
        tx_data.clone(),
        cl_id_2.clone(),
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create transaction: {}", e));
        e.to_string()
    })?;

    let cl_tx_2 = CLTransaction::new(
        cl_id_2.clone(),
        vec![chain_id_2.clone()],
        vec![tx_2],
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create CL transaction: {}", e));
        e.to_string()
    })?;

    logging::log("SIMULATOR", &format!("Created CL transactions with IDs: {:?}_1 and {:?}_2", cl_id, cl_id));

    // Submit both transactions to CL node
    let success1 = match cl_node.lock().await.submit_transaction(cl_tx_1.clone()).await {
        Ok(_) => {
            logging::log("SIMULATOR", &format!("Regular transaction submitted successfully to chain-1: {}", tx_data));
            true
        }
        Err(e) => {
            logging::log("SIMULATOR", &format!("Failed to submit regular transaction to chain-1: {}", e));
            logging::log("SIMULATOR", &format!("Regular transaction failed on chain-1: {}", tx_data));
            false
        }
    };

    let success2 = match cl_node.lock().await.submit_transaction(cl_tx_2.clone()).await {
        Ok(_) => {
            logging::log("SIMULATOR", &format!("Regular transaction submitted successfully to chain-2: {}", tx_data));
            true
        }
        Err(e) => {
            logging::log("SIMULATOR", &format!("Failed to submit regular transaction to chain-2: {}", e));
            logging::log("SIMULATOR", &format!("Regular transaction failed on chain-2: {}", tx_data));
            false
        }
    };

    Ok((success1 && success2, tx_data))
} 