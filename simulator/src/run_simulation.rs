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

// ------------------------------------------------------------------------------------------------
// Main Simulation Function
// ------------------------------------------------------------------------------------------------

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
    results: &mut SimulationResults,
) -> Result<(), String> {
    
    // Wait for initialization transactions to be processed
    logging::log("SIMULATOR", "Waiting 5 block for initialization transactions to be processed...");
    let wait = Duration::from_secs_f64(results.block_interval * 5.0);
    sleep(wait).await;
    logging::log("SIMULATOR", "Initialization complete, starting simulation");

    // Initialize random number generator
    let mut rng = rand::thread_rng();
    
    // Initialize sender account selector with uniform distribution
    let account_selector_sender = AccountSelector::new(results.num_accounts, 0.0);    
    // Initialize receiver account selector with Zipf distribution
    let account_selector_receiver = AccountSelector::new(results.num_accounts, results.zipf_parameter);
    
    // Calculate total number of transactions to send
    let total_transactions = results.target_tps * results.duration_seconds;
    
    // Create progress bar
    let progress_bar = ProgressBar::new(total_transactions);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} transactions ({eta})")
        .unwrap()
        .progress_chars("##-"));

    // Set HIG delays
    for (i, delay) in results.chain_delays.iter().enumerate() {
        hig_nodes[i].lock().await.set_hs_message_delay(*delay);
    }
    
    // Track transaction amounts per chain by height. In the chain the tx is either pending, success, or failure.
    let mut current_block = 0;
    
    // Get registered chains
    let chains = cl_node.lock().await.get_registered_chains().await.map_err(|e| e.to_string())?;
    let chain_id_1 = chains[0].clone();
    let chain_id_2 = chains[1].clone();
    
    // Main simulation loop
    while results.transactions_sent < total_transactions {
        // Select accounts for transaction
        let from_account = account_selector_sender.select_account(&mut rng);
        let to_account = account_selector_receiver.select_account(&mut rng);
        
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
        
        logging::log("SIMULATOR", &format!("Transaction data: '{}'", tx_data));

        // Create and submit transaction
        let cl_id = CLTransactionId(format!("cl-tx_{}", results.transactions_sent));
        
        let (success, _) = if is_cat {
            results.cat_transactions += 1;
            create_and_submit_cat_transaction(
                &cl_node,
                cl_id,
                chain_id_1.clone(),
                chain_id_2.clone(),
                tx_data.clone(),
            ).await?
        } else {
            results.regular_transactions += 1;
            create_and_submit_regular_transaction(
                &cl_node,
                cl_id,
                chain_id_1.clone(),
                chain_id_2.clone(),
                tx_data.clone(),
            ).await?
        };

        if success {
            logging::log("SIMULATOR", &format!("Transaction successful submitted to CL  : {}", tx_data));
        } else {
            logging::log("SIMULATOR", &format!("Transaction failed submitted to CL  : {}", tx_data));
            // we should crash
            panic!("Transaction failed submitted to CL");
        }
        
        results.transactions_sent += 1;
        progress_bar.inc(1);
        
        // Get current block height and transaction status counts
        let new_block = cl_node.lock().await.get_current_block().await.map_err(|e| e.to_string())?;
        
        // Get success and failure transaction counts
        let (chain_1_pending, chain_1_success, chain_1_failure) = hig_nodes[0].lock().await.get_transaction_status_counts().await.map_err(|e| e.to_string())?;
        let (chain_2_pending, chain_2_success, chain_2_failure) = hig_nodes[1].lock().await.get_transaction_status_counts().await.map_err(|e| e.to_string())?;
        
        // Only record counts if we've moved to a new block
        if new_block != current_block {
            if current_block > 0 {  // Don't record for block 0
                results.chain_1_pending.push((current_block, chain_1_pending));    
                results.chain_2_pending.push((current_block, chain_2_pending));
                results.chain_1_success.push((current_block, chain_1_success));
                results.chain_2_success.push((current_block, chain_2_success));
                results.chain_1_failure.push((current_block, chain_1_failure));
                results.chain_2_failure.push((current_block, chain_2_failure));
            }
            current_block = new_block;
        }
        
        // Calculate sleep time to maintain target TPS
        let elapsed = results.start_time.elapsed();
        let target_milliseconds = (results.transactions_sent as f64 / results.target_tps as f64) * 1000.0;
        let target_elapsed = Duration::from_millis(target_milliseconds as u64);
        if elapsed < target_elapsed {
            tokio::time::sleep(target_elapsed - elapsed).await;
        }
    }
 
    // Save results
    results.save().await?;
    
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