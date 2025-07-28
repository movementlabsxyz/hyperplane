//! Core simulation logic for the Hyperplane simulator.
//! 
//! Handles transaction generation, submission, and status tracking during simulations.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use indicatif::{ProgressBar, ProgressStyle};
use hyperplane::{
    types::{TransactionId, Transaction, CLTransaction, CLTransactionId, ChainId},
    confirmation_layer::{ConfirmationLayerNode, ConfirmationLayer},
    hyper_ig::node::HyperIGNode,
    hyper_ig::HyperIG,
    utils::logging,
};
use crate::zipf_account_selection::AccountSelector;
use rand::Rng;
use crate::SimulationResults;
use std::time::Instant;

// ------------------------------------------------------------------------------------------------
// Main Simulation Function
// ------------------------------------------------------------------------------------------------

/// Runs the simulation for the specified number of blocks
///
/// # Arguments
///
/// * `nodes` - A tuple containing two vectors:
///   - The first vector contains Arc<Mutex<ConfirmationLayerNode>>, the confirmation layer nodes
///   - The second vector contains Arc<Mutex<HyperIGNode>>, the HyperIG nodes
/// * `sim_total_block_number` - A u64, the total number of blocks to simulate
/// * `initial_balance` - A u64, the initial balance for transactions
/// * `num_accounts` - A usize, the number of accounts
/// * `target_tps` - A u64, the target TPS
/// * `zipf_parameter` - A f64, the Zipf parameter for account selection
/// * `chain_delays` - A Vec<u64>, the chain delays for each node (in blocks)
/// * `block_interval` - A f64, the interval between blocks
/// * `ratio_cats` - A f64, the ratio of CAT transactions
///
pub async fn run_simulation(
    cl_node: Arc<Mutex<ConfirmationLayerNode>>,
    hig_nodes: Vec<Arc<Mutex<HyperIGNode>>>,
    results: &mut SimulationResults,
) -> Result<(), String> {
    run_simulation_with_message(cl_node, hig_nodes, results, None).await
}

pub async fn run_simulation_with_message(
    cl_node: Arc<Mutex<ConfirmationLayerNode>>,
    hig_nodes: Vec<Arc<Mutex<HyperIGNode>>>,
    results: &mut SimulationResults,
    run_message: Option<String>,
) -> Result<(), String> {
    run_simulation_with_message_and_retries(cl_node, hig_nodes, results, run_message, None).await
}

pub async fn run_simulation_with_message_and_retries(
    cl_node: Arc<Mutex<ConfirmationLayerNode>>,
    hig_nodes: Vec<Arc<Mutex<HyperIGNode>>>,
    results: &mut SimulationResults,
    run_message: Option<String>,
    retry_count: Option<usize>,
) -> Result<(), String> {
    
    // Get the current block at the start
    let start_block = cl_node.lock().await.get_current_block().await.map_err(|e| e.to_string())?;
    
    let target_block = start_block + results.initialization_wait_blocks;
    
    logging::log("SIMULATOR", &format!("Starting at block {}, waiting until block {} for initialization and stable block production...", start_block, target_block));
    
    // Wait until we reach the target block
    loop {
        let current_block = cl_node.lock().await.get_current_block().await.map_err(|e| e.to_string())?;
        if current_block >= target_block {
            break;
        }
        // Sleep for a short duration before checking again
        sleep(Duration::from_millis(100)).await;
    }
    
    // Get the current block after initialization
    let initial_block = cl_node.lock().await.get_current_block().await.map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Initialization complete, starting simulation at block {}", initial_block));

    // Record the start time for transaction sending (after initialization)
    let _transaction_spam_start_time = Instant::now();

    // Initialize random number generator
    let mut rng = rand::thread_rng();
    
    // Initialize sender account selector with uniform distribution
    let mut account_selector_sender = AccountSelector::new(results.num_accounts, 0.0);    
    // Initialize receiver account selector with Zipf distribution
    let mut account_selector_receiver = AccountSelector::new(results.num_accounts, results.zipf_parameter);
    
    // Calculate target block number for simulation termination
    let final_simulation_block = initial_block + results.sim_total_block_number;
    
    // Create progress bar for blocks
    let progress_bar = ProgressBar::new(results.sim_total_block_number);
    let template = if let Some(ref msg) = run_message {
        let retry_suffix = if let Some(retries) = retry_count {
            if retries > 0 {
                format!(" [RETRY {}]", retries)
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        format!("[{{elapsed_precise}}] {{bar:40.cyan/blue}} Block {{pos}}/{{len}} ({{eta}}) {}{}", msg, retry_suffix)
    } else {
        let retry_suffix = if let Some(retries) = retry_count {
            if retries > 0 {
                format!(" [RETRY {}]", retries)
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        format!("[{{elapsed_precise}}] {{bar:40.cyan/blue}} Block {{pos}}/{{len}} ({{eta}}){}", retry_suffix)
    };
    progress_bar.set_style(ProgressStyle::default_bar()
        .template(&template)
        .unwrap()
        .progress_chars("##-"));
    

    // Now set the actual chain delays for the main simulation
    logging::log("SIMULATOR", "Setting actual chain delays for main simulation...");
    for (i, delay_blocks) in results.chain_delays.iter().enumerate() {
        let delay_time = Duration::from_secs_f64(results.block_interval * *delay_blocks as f64);
        hig_nodes[i].lock().await.set_hs_message_delay(delay_time);
        logging::log("SIMULATOR", &format!("Set chain {} delay to {} blocks ({:?})", i + 1, delay_blocks, delay_time));
    }
    
    // Track transaction amounts per chain by height. In the chain the tx is either pending, success, or failure.
    let mut current_block = initial_block;
    
    // Get registered chains
    let chains = cl_node.lock().await.get_registered_chains().await.map_err(|e| e.to_string())?;
    let chain_id_1 = chains[0].clone();
    let chain_id_2 = chains[1].clone();
    
    // ------- main simulation loop -------

    // Calculate transactions per block based on target TPS and block interval
    let transactions_per_block = (results.target_tps as f64 * results.block_interval) as u64;
    logging::log("SIMULATOR", &format!("Target TPS: {}, Block interval: {}s, Transactions per block: {}", 
        results.target_tps, results.block_interval, transactions_per_block));
    

    // a counter to track how many times we have entered the following block without releasing transactions
    let mut block_counter = 0;

    // Main simulation loop - waits for new blocks and releases transactions in batches
    while current_block < final_simulation_block {
        // Get current block height from CL
        let new_block = cl_node.lock().await.get_current_block().await.map_err(|e| e.to_string())?;
        
        // Stop processing transactions if we've reached the target block
        if new_block >= final_simulation_block {
            logging::log("SIMULATOR", &format!("Reached target block {}, stopping transaction processing", final_simulation_block));
            break;
        }

        // Check if we've moved to a new block
        if new_block != current_block {

            // Record the block counter for the previous block before resetting it
            results.loop_steps_without_tx_issuance.push((current_block, block_counter));
            // reset the block counter
            block_counter = 0;

            logging::log("SIMULATOR", &format!("🎯 NEW BLOCK CREATED - Height: {} 🎯", new_block));
            
            // Process and record all data for this block
            process_block_data(
                &cl_node,
                &hig_nodes,
                results,
                new_block,
                chain_id_1.clone(),
                chain_id_2.clone(),
            ).await?;
            
            current_block = new_block;
            
            // Update progress bar for new block
            let blocks_completed = new_block - initial_block;
            progress_bar.set_position(blocks_completed);
            
            // Release all transactions for this block at once
            release_transactions_for_block(
                &cl_node,
                &mut rng,
                &mut account_selector_sender,
                &mut account_selector_receiver,
                results,
                chain_id_1.clone(),
                chain_id_2.clone(),
                transactions_per_block,
                new_block,
            ).await?;
        } else {
            // increment the block counter
            block_counter += 1;

            // Wait in intervals of 1/5 of the block interval until next block
            let wait_interval = Duration::from_secs_f64(results.block_interval / 5.0);
            tokio::time::sleep(wait_interval).await;
        }
    }
 
    // Save results - removed for sweep simulations that handle their own saving
    // results.save().await?;
    
    Ok(())
}

// ------------------------------------------------------------------------------------------------
// Data Processing Functions
// ------------------------------------------------------------------------------------------------

/// Processes and records all data for a single block
async fn process_block_data(
    cl_node: &Arc<Mutex<ConfirmationLayerNode>>,
    hig_nodes: &[Arc<Mutex<HyperIGNode>>],
    results: &mut SimulationResults,
    block_height: u64,
    chain_id_1: ChainId,
    chain_id_2: ChainId,
) -> Result<(), String> {
    // Get CAT transaction status counts
    let (chain_1_cat_pending, chain_1_cat_success, chain_1_cat_failure) = hig_nodes[0].lock().await.get_transaction_status_counts_cats().await.map_err(|e| e.to_string())?;
    let (chain_2_cat_pending, chain_2_cat_success, chain_2_cat_failure) = hig_nodes[1].lock().await.get_transaction_status_counts_cats().await.map_err(|e| e.to_string())?;
    
    // Get regular transaction status counts
    let (chain_1_regular_pending, chain_1_regular_success, chain_1_regular_failure) = hig_nodes[0].lock().await.get_transaction_status_counts_regular().await.map_err(|e| e.to_string())?;
    let (chain_2_regular_pending, chain_2_regular_success, chain_2_regular_failure) = hig_nodes[1].lock().await.get_transaction_status_counts_regular().await.map_err(|e| e.to_string())?;
    
    // Get locked keys counts
    let chain_1_locked_keys = hig_nodes[0].lock().await.get_total_locked_keys_count().await;
    let chain_2_locked_keys = hig_nodes[1].lock().await.get_total_locked_keys_count().await;
    
    // Calculate combined totals for backward compatibility
    let chain_1_pending = chain_1_cat_pending + chain_1_regular_pending;
    let chain_1_success = chain_1_cat_success + chain_1_regular_success;
    let chain_1_failure = chain_1_cat_failure + chain_1_regular_failure;
    let chain_2_pending = chain_2_cat_pending + chain_2_regular_pending;
    let chain_2_success = chain_2_cat_success + chain_2_regular_success;
    let chain_2_failure = chain_2_cat_failure + chain_2_regular_failure;
    
    // Subtract initialization transactions from success counts
    // Each account gets one initialization credit transaction per chain
    let init_tx_count = results.num_accounts as u64;
    let chain_1_success_filtered = chain_1_success.saturating_sub(init_tx_count);
    let chain_2_success_filtered = chain_2_success.saturating_sub(init_tx_count);
    
    // Get transactions per block for current block (only once per block)
    let chain_1_tx_per_block = cl_node.lock().await.get_subblock(chain_id_1.clone(), block_height).await
        .map(|subblock| subblock.transactions.len() as u64)
        .unwrap_or(0);
    let chain_2_tx_per_block = cl_node.lock().await.get_subblock(chain_id_2.clone(), block_height).await
        .map(|subblock| subblock.transactions.len() as u64)
        .unwrap_or(0);
    
    // Record combined totals (for backward compatibility)
    results.chain_1_pending.push((block_height, chain_1_pending));    
    results.chain_2_pending.push((block_height, chain_2_pending));
    results.chain_1_success.push((block_height, chain_1_success_filtered));
    results.chain_2_success.push((block_height, chain_2_success_filtered));
    results.chain_1_failure.push((block_height, chain_1_failure));
    results.chain_2_failure.push((block_height, chain_2_failure));
    
    // Record CAT transaction data
    results.chain_1_cat_pending.push((block_height, chain_1_cat_pending));
    results.chain_2_cat_pending.push((block_height, chain_2_cat_pending));
    results.chain_1_cat_success.push((block_height, chain_1_cat_success));
    results.chain_2_cat_success.push((block_height, chain_2_cat_success));
    results.chain_1_cat_failure.push((block_height, chain_1_cat_failure));
    results.chain_2_cat_failure.push((block_height, chain_2_cat_failure));
    
    // Record regular transaction data
    results.chain_1_regular_pending.push((block_height, chain_1_regular_pending));
    results.chain_2_regular_pending.push((block_height, chain_2_regular_pending));
    results.chain_1_regular_success.push((block_height, chain_1_regular_success));
    results.chain_2_regular_success.push((block_height, chain_2_regular_success));
    results.chain_1_regular_failure.push((block_height, chain_1_regular_failure));
    results.chain_2_regular_failure.push((block_height, chain_2_regular_failure));
    
    // Record locked keys data
    results.chain_1_locked_keys.push((block_height, chain_1_locked_keys));
    results.chain_2_locked_keys.push((block_height, chain_2_locked_keys));
    
    // Record transactions per block data
    results.chain_1_tx_per_block.push((block_height, chain_1_tx_per_block));
    results.chain_2_tx_per_block.push((block_height, chain_2_tx_per_block));
    
    // Record memory usage for this block
    let memory_usage = crate::SimulationResults::get_current_memory_usage();
    results.memory_usage.push((block_height, memory_usage));
    
    // Record total RAM usage for this block
            let total_memory = crate::SimulationResults::get_current_total_memory();
        results.total_memory.push((block_height, total_memory));
    
    // Record process CPU usage for this block
    let cpu_usage = crate::SimulationResults::get_current_cpu_usage();
    results.cpu_usage.push((block_height, cpu_usage));
    
    // Record total system CPU usage for this block
    let total_cpu_usage = crate::SimulationResults::get_current_total_cpu_usage();
    results.total_cpu_usage.push((block_height, total_cpu_usage));
    
    Ok(())
}

// ------------------------------------------------------------------------------------------------
// Transaction Release Functions
// ------------------------------------------------------------------------------------------------

/// Releases all transactions for a single block
async fn release_transactions_for_block(
    cl_node: &Arc<Mutex<ConfirmationLayerNode>>,
    rng: &mut rand::rngs::ThreadRng,
    account_selector_sender: &mut AccountSelector,
    account_selector_receiver: &mut AccountSelector,
    results: &mut SimulationResults,
    chain_id_1: ChainId,
    chain_id_2: ChainId,
    transactions_per_block: u64,
    current_block: u64,
) -> Result<(), String> {
    for tx_index in 0..transactions_per_block {
        // Select accounts for transaction
        let from_account = account_selector_sender.select_account(rng);
        let to_account = account_selector_receiver.select_account(rng);
        
        // Record transaction in account statistics
        results.account_stats.record_transaction(from_account as u64, to_account as u64);
        
        // Determine if this should be a CAT transaction based on configured ratio
        let is_cat = rng.gen_bool(results.ratio_cats);
        
        // Create transaction data
        let tx_data = format!("{}.send {} {} 1", 
            if is_cat { "CAT" } else { "REGULAR" },
            from_account,
            to_account
        );
        
        // Create and submit transaction
        let cl_id = CLTransactionId(format!("cl-{}-tx_{}", 
            if is_cat { "cat" } else { "reg" }, 
            results.transactions_sent
        ));
        
        let (success, _) = if is_cat {
            results.cat_transactions += 1;
            create_and_submit_cat_transaction(
                cl_node,
                cl_id,
                chain_id_1.clone(),
                chain_id_2.clone(),
                tx_data.clone(),
            ).await?
        } else {
            results.regular_transactions += 1;
            create_and_submit_regular_transaction(
                cl_node,
                cl_id,
                chain_id_1.clone(),
                chain_id_2.clone(),
                tx_data.clone(),
            ).await?
        };

        if success {
            logging::log("SIMULATOR", &format!("Transaction {} successful: {}", tx_index + 1, tx_data));
        } else {
            logging::log("SIMULATOR", &format!("Transaction {} failed: {}", tx_index + 1, tx_data));
            panic!("Transaction failed submitted to CL");
        }
        
        results.transactions_sent += 1;
    }
    
    Ok(())
}

// ------------------------------------------------------------------------------------------------
// Transaction Creation and Submission
// ------------------------------------------------------------------------------------------------

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
        logging::log("SIMULATOR", &format!("Failed to create CAT-sub-transaction 1: {}", e));
        e.to_string()
    })?;

    let tx2 = Transaction::new(
        TransactionId(format!("{:?}:tx2", cl_id)),
        chain_id_2.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        tx_data.clone(),
        cl_id.clone(),
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create CAT-sub-transaction 2: {}", e));
        e.to_string()
    })?;

    // Create the CL transaction
    let cl_tx = CLTransaction::new(
        cl_id.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        vec![tx1, tx2],
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create CAT CL transaction: {}", e));
        e.to_string()
    })?;

    logging::log("SIMULATOR", &format!("Created CAT CL transaction with ID: {:?}", cl_id));

    // Submit transaction to CL node
    match cl_node.lock().await.submit_transaction(cl_tx.clone()).await {
        Ok(_) => {
            logging::log("SIMULATOR", &format!("CAT transaction submitted successfully: {}", tx_data));
            Ok((true, tx_data))
        }
        Err(e) => {
            logging::log("SIMULATOR", &format!("Failed to submit CAT CL transaction: {}", e));
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
        logging::log("SIMULATOR", &format!("Failed to create regular transaction for chain-1: {}", e));
        e.to_string()
    })?;

    let cl_tx_1 = CLTransaction::new(
        cl_id_1.clone(),
        vec![chain_id_1.clone()],
        vec![tx_1],
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create regular CL transaction for chain-1: {}", e));
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
        logging::log("SIMULATOR", &format!("Failed to create regular transaction for chain-2: {}", e));
        e.to_string()
    })?;

    let cl_tx_2 = CLTransaction::new(
        cl_id_2.clone(),
        vec![chain_id_2.clone()],
        vec![tx_2],
    ).map_err(|e| {
        logging::log("SIMULATOR", &format!("Failed to create regular CL transaction for chain-2: {}", e));
        e.to_string()
    })?;

    logging::log("SIMULATOR", &format!("Created regular CL transactions with IDs: {:?}_1 and {:?}_2", cl_id, cl_id));

    // Submit both transactions to CL node
    let success1 = match cl_node.lock().await.submit_transaction(cl_tx_1.clone()).await {
        Ok(_) => {
            logging::log("SIMULATOR", &format!("Regular transaction submitted successfully to chain-1: {}", tx_data));
            true
        }
        Err(e) => {
            logging::log("SIMULATOR", &format!("Failed to submit regular transaction to CL node: {}", e));
            logging::log("SIMULATOR", &format!("Regular transaction failed to submit: {}", tx_data));
            false
        }
    };

    let success2 = match cl_node.lock().await.submit_transaction(cl_tx_2.clone()).await {
        Ok(_) => {
            logging::log("SIMULATOR", &format!("Regular transaction submitted successfully: {}", tx_data));
            true
        }
        Err(e) => {
            logging::log("SIMULATOR", &format!("Failed to submit regular transaction to CL node: {}", e));
            logging::log("SIMULATOR", &format!("Regular transaction failed to submit: {}", tx_data));
            false
        }
    };

    Ok((success1 && success2, tx_data))
} 