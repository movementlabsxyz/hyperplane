use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep, interval};
use tokio::sync::mpsc;
use hyperplane::{
    types::{Transaction, TransactionId, ChainId, CLTransaction, SubBlock, CATStatusUpdate, CLTransactionId},
};
use std::collections::HashSet;
use hyperplane::utils::logging;

// - - - - - - - - - - - - - - - - - - - - - - - 
// V11: copies v1 but uses correct types
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V11: copies v10 but uses correct types
/// - changes to real types
#[tokio::test]
async fn test_v11() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_v11 ===");
    
    // Create channels for messages and subblocks
    let (sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel(100);
    let (sender_cl_to_hig, _receiver_cl_to_hig) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV11::new(
        receiver_hs_to_cl,
        sender_cl_to_hig,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v11(state_for_processor).await;
    });

    // Register chains first
    logging::log("TEST", "[TEST]   Registering chains...");
    {
        let mut state = state.lock().await;
        state.register_chain(ChainId("chain-1".to_string())).expect("Failed to register chain-1");
        state.register_chain(ChainId("chain-2".to_string())).expect("Failed to register chain-2");
        
        // Try to register chain-1 again (should fail)
        match state.register_chain(ChainId("chain-1".to_string())) {
            Ok(_) => panic!("Should not be able to register chain-1 twice"),
            Err(e) => logging::log("TEST", &format!("[TEST]   Expected error when registering chain-1 twice: '{}'", e)),
        }

        // Try to get subblock for unregistered chain
        match state.get_subblock(ChainId("chain-3".to_string()), 0) {
            Ok(_) => panic!("Should not be able to get subblock for unregistered chain"),
            Err(e) => logging::log("TEST", &format!("[TEST]   Expected error when getting subblock for unregistered chain: '{}'", e)),
        }
    }

    // Submit transactions for different chains
    logging::log("TEST", "[TEST]   Submitting transactions...");
    {
        let mut state = state.lock().await;
        
        // Submit a transaction for chain1
        let cl_id_1 = CLTransactionId("cl-tx_1".to_string());
        let tx_chain_1 = Transaction::new(
            TransactionId("tx1".to_string()),
            ChainId("chain-1".to_string()),
            vec![ChainId("chain-1".to_string())],
            "REGULAR.credit 1 100".to_string(),
            cl_id_1.clone(),
        ).expect("Failed to create transaction");
        let cl_tx_chain_1 = CLTransaction::new(
            cl_id_1.clone(),
            vec![ChainId("chain-1".to_string())],
            vec![tx_chain_1],
        ).expect("Failed to create CL transaction");
        state.submit_transaction(cl_tx_chain_1).expect("Failed to submit transaction for chain-1");
        
        // Submit a transaction for chain2
        let cl_id_2 = CLTransactionId("cl-tx_2".to_string());
        let tx_chain_2 = Transaction::new(
            TransactionId(format!("{:?}:tx_2", cl_id_2)),
            ChainId("chain-2".to_string()),
            vec![ChainId("chain-2".to_string())],
            "REGULAR.credit 1 100".to_string(),
            cl_id_2.clone(),
        ).expect("Failed to create transaction");
        let cl_tx_chain_2 = CLTransaction::new(
            cl_id_2.clone(),
            vec![ChainId("chain-2".to_string())],
            vec![tx_chain_2],
        ).expect("Failed to create CL transaction");
        state.submit_transaction(cl_tx_chain_2).expect("Failed to submit transaction for chain-2");
        
        // Try to submit a transaction for unregistered chain (should fail)
        let cl_id_3 = CLTransactionId("cl-tx_3".to_string());
        let tx_chain_3 = Transaction::new(
            TransactionId(format!("{:?}:tx_3", cl_id_3)),
            ChainId("chain-3".to_string()),
            vec![ChainId("chain-3".to_string())],
            "REGULAR.credit 1 100".to_string(),
            cl_id_3.clone(),
        ).expect("Failed to create transaction");
        let cl_tx_chain_3 = CLTransaction::new(
            cl_id_3.clone(),
            vec![ChainId("chain-3".to_string())],
            vec![tx_chain_3],
        ).expect("Failed to create CL transaction");
        match state.submit_transaction(cl_tx_chain_3) {
            Ok(_) => panic!("Should not be able to submit transaction for unregistered chain"),
            Err(e) => logging::log("TEST", &format!("  [TEST]   Expected error when submitting transaction for unregistered chain: '{}'", e)),
        }
    }

    // Spawn tasks to add more transactions for different chains
    let sender_for_chain1 = sender_hs_to_cl.clone();
    let _adder_handle1 = tokio::spawn(async move {
        run_adder_v11(sender_for_chain1, ChainId("chain-1".to_string())).await;
    });

    let sender_for_chain2 = sender_hs_to_cl.clone();
    let _adder_handle2 = tokio::spawn(async move {
        run_adder_v11(sender_for_chain2, ChainId("chain-2".to_string())).await;
    });

    // Wait for a few seconds to let the processor run
    logging::log("TEST", "Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    logging::log("TEST", &format!("Main task: current block is {}", state_guard.current_block));
    logging::log("TEST", &format!("Main task: processed {} transactions", state_guard.processed_transactions.len()));
    logging::log("TEST", &format!("Main task: {} transactions still pending", state_guard.pending_transactions.len()));
    logging::log("TEST", &format!("Main task: produced {} blocks", state_guard.blocks.len()));
    logging::log("TEST", &format!("Main task: registered chains: {:?}", state_guard.registered_chains));
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_transactions.is_empty(), "Should have processed some transactions");
    assert!(!state_guard.blocks.is_empty(), "Should have produced some blocks");
    assert_eq!(state_guard.registered_chains.len(), 2, "Should have exactly 2 registered chains");
    
    // Test getting subblock for registered chain
    match state_guard.get_subblock(ChainId("chain-1".to_string()), 0) {
        Ok(subblock) => logging::log("TEST", &format!("  [TEST]   Successfully got subblock for chain-1: {:?}", subblock)),
        Err(e) => panic!("Failed to get subblock for chain-1: {}", e),
    }
    
    // Drop the first state lock
    drop(state_guard);
    
    // The processor task will continue running until the test ends
    sleep(Duration::from_secs(1)).await;
    
    // Make sure the processor task is still running by checking the state again
    let state_guard = state.lock().await;
    let current_block = state_guard.current_block;
    let processed_count = state_guard.processed_transactions.len();
    let block_count = state_guard.blocks.len();
    logging::log("TEST", &format!("Main task: final check - block is {}, processed {} transactions in {} blocks", 
        current_block, processed_count, block_count));
    
    // Ensure the processor is still running and processing transactions
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 transactions per chain (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 10, "Should have processed more than 10 transactions in 2 seconds (5 per chain)");
    assert!(block_count > 15, "Should have produced more than 15 blocks in 2 seconds");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// V11: State struct that matches CL node functionality
struct TestNodeStateV11 {
    receiver_hs_to_cl: mpsc::Receiver<CLTransaction>,
    sender_cl_to_hig: mpsc::Sender<SubBlock>,
    block_interval: Duration,
    current_block: u64,
    processed_transactions: Vec<(ChainId, CLTransaction)>,
    pending_transactions: Vec<CLTransaction>,
    blocks: Vec<u64>,
    registered_chains: Vec<ChainId>,
}

impl TestNodeStateV11 {
    fn new(
        receiver_hs_to_cl: mpsc::Receiver<CLTransaction>,
        sender_cl_to_hig: mpsc::Sender<SubBlock>,
        block_interval: Duration,
    ) -> Self {
        Self {
            receiver_hs_to_cl,
            sender_cl_to_hig,
            block_interval,
            current_block: 0,
            processed_transactions: Vec::new(),
            pending_transactions: Vec::new(),
            blocks: Vec::new(),
            registered_chains: Vec::new(),
        }
    }

    fn register_chain(&mut self, chain_id: ChainId) -> Result<(), String> {
        if self.registered_chains.contains(&chain_id) {
            return Err(format!("Chain {} is already registered", chain_id.0));
        }
        self.registered_chains.push(chain_id);
        Ok(())
    }

    fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), String> {
        if !transaction.constituent_chains.iter().all(|c| self.registered_chains.contains(c)) {
            return Err(format!("Chain {} is not registered", transaction.constituent_chains[0].0));
        }
        self.pending_transactions.push(transaction);
        Ok(())
    }

    fn get_subblock(&self, chain_id: ChainId, block_height: u64) -> Result<SubBlock, String> {
        if !self.registered_chains.contains(&chain_id) {
            return Err(format!("Chain {} is not registered", chain_id.0));
        }
        // For simplicity, just return a dummy subblock
        Ok(SubBlock {
            chain_id: chain_id.clone(),
            block_height,
            transactions: self.processed_transactions
                .iter()
                .filter(|(cid, _)| cid == &chain_id)
                .map(|(_, cl_tx)| Transaction::new(
                    cl_tx.transactions[0].id.clone(),
                    cl_tx.transactions[0].target_chain_id.clone(),
                    cl_tx.transactions[0].constituent_chains.clone(),
                    cl_tx.transactions[0].data.clone(),
                    cl_tx.transactions[0].cl_id.clone(),
                ).expect("Failed to create transaction"))
                .collect(),
        })
    }
}

/// Helper function to run the processor task
async fn run_processor_v11(state: Arc<Mutex<TestNodeStateV11>>) {
    let mut interval = interval(state.lock().await.block_interval);
    loop {
        interval.tick().await;
        
        let mut state = state.lock().await;
        
        // Process any new transactions from the channel
        while let Ok(cl_tx) = state.receiver_hs_to_cl.try_recv() {
            logging::log("TEST", &format!("  [TEST] [Processor] received transaction for chains {:?}: {}", cl_tx.constituent_chains, cl_tx.transactions[0].data));
            if cl_tx.constituent_chains.iter().all(|c| state.registered_chains.contains(c)) {
                state.pending_transactions.push(cl_tx);
            }
        }
        
        state.current_block += 1;
        
        // Process pending transactions for this block
        let mut processed_this_block = Vec::new();
        let mut remaining = Vec::new();
        let registered_chains = state.registered_chains.clone();
        for tx in state.pending_transactions.drain(..) {
            if tx.constituent_chains.iter().all(|c| registered_chains.contains(c)) {
                processed_this_block.push((tx.constituent_chains[0].clone(), tx.clone()));
            } else {
                remaining.push(tx);
            }
        }
        state.pending_transactions = remaining;
        
        // Create a block
        let block_height = 0;
        state.blocks.push(block_height);
        
        // Send subblocks for each chain with only this block's transactions
        for chain_id in &state.registered_chains {
            let subblock = SubBlock {
                chain_id: chain_id.clone(),
                block_height,
                transactions: processed_this_block
                    .iter()
                    .filter(|(cid, _)| cid == chain_id)
                    .map(|(_, cl_tx)| Transaction::new(
                        cl_tx.transactions[0].id.clone(),
                        cl_tx.transactions[0].target_chain_id.clone(),
                        cl_tx.transactions[0].constituent_chains.clone(),
                        cl_tx.transactions[0].data.clone(),
                        cl_tx.transactions[0].cl_id.clone(),
                    ).expect("Failed to create transaction"))
                    .collect(),
            };
            if let Err(e) = state.sender_cl_to_hig.send(subblock).await {
                logging::log("TEST", &format!("  [TEST] [Processor] Error sending subblock: {}", e));
                break;
            }
        }
        state.processed_transactions.extend(processed_this_block.iter().cloned());
        
        // Print block status
        if !processed_this_block.is_empty() {
            let mut block_status = format!("  [TEST] [Processor] produced block {} with {} transactions", state.current_block, processed_this_block.len());
            for (_, cl_tx) in &processed_this_block {
                block_status.push_str(&format!("\n  - id={}, data={}", cl_tx.transactions[0].id.0, cl_tx.transactions[0].data));
            }
            logging::log("TEST", &block_status);
        } else {
            logging::log("TEST", &format!("  [TEST] [Processor] produced empty block {}", state.current_block));
        }
    }
}

/// Helper function to run the adder task
async fn run_adder_v11(sender: mpsc::Sender<CLTransaction>, chain_id: ChainId) {
    for i in 1..=10 {
        let cl_id = CLTransactionId(format!("cl-tx_{}.{}", i, chain_id.0));
        let tx = Transaction::new(
            TransactionId(format!("{}.tx", cl_id.0)),
            chain_id.clone(),
            vec![chain_id.clone()],
            "REGULAR.credit 1 100".to_string(),
            cl_id.clone(),
        ).expect("Failed to create transaction");
        let cl_tx = CLTransaction::new(
            cl_id.clone(),
            vec![chain_id.clone()],
            vec![tx],
        ).expect("Failed to create CL transaction");
        if let Err(e) = sender.send(cl_tx).await {
            logging::log("TEST", &format!("  [TEST] [Adder] Error sending transaction: {}", e));
            break;
        }
        sleep(Duration::from_millis(300)).await;
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - 
// V12: Integrates closer to actual node setup
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V12: Integrates closer to actual node setup
#[tokio::test]
async fn test_v12() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_v12 ===");
    
    // Get the test nodes using our new helper function
    let (hs_node, cl_node, _hig_node) = setup_test_nodes(Duration::from_millis(100)).await;
    
    // Test initial state
    logging::log("TEST", "[TEST]   Testing initial state...");
    {
        let cl_node_with_lock = cl_node.lock().await;
        let current_block = cl_node_with_lock.get_current_block().await.unwrap();
        logging::log("TEST", &format!("[TEST]   Initial block number: {}", current_block));
        assert_eq!(current_block, 0, "Initial block should be 0");
    }

    // Register chains first
    logging::log("TEST", "[TEST]   Registering chains...");
    {
        let mut cl_node_with_lock = cl_node.lock().await;
        cl_node_with_lock.register_chain(ChainId("chain-1".to_string())).expect("Failed to register chain-1");
        cl_node_with_lock.register_chain(ChainId("chain-2".to_string())).expect("Failed to register chain-2");
        
        // Try to register chain1 again (should fail)
        match cl_node_with_lock.register_chain(ChainId("chain-1".to_string())) {
            Ok(_) => panic!("Should not be able to register chain-1 twice"),
            Err(e) => logging::log("TEST", &format!("[TEST]   Expected error when registering chain-1 twice: {}", e)),
        }

        // Try to get subblock for unregistered chain
        match cl_node_with_lock.get_subblock(ChainId("chain-3".to_string()), 0) {
            Ok(_) => panic!("Should not be able to get subblock for unregistered chain"),
            Err(e) => logging::log("TEST", &format!("[TEST]   Expected error when getting subblock for unregistered chain: {}", e)),
        }
    }

    // Verify chain registration and get subblock for registered chain
    logging::log("TEST", "[TEST]   Verifying chain registration and subblock retrieval...");
    {
        let cl_node_with_lock = cl_node.lock().await;
        // Verify registered chains
        assert_eq!(cl_node_with_lock.registered_chains.len(), 2, "Should have exactly 2 registered chains");
        assert!(cl_node_with_lock.registered_chains.contains(&ChainId("chain-1".to_string())), "chain-1 should be registered");
        assert!(cl_node_with_lock.registered_chains.contains(&ChainId("chain-2".to_string())), "chain-2 should be registered");

        // Get subblock for registered chain
        match cl_node_with_lock.get_subblock(ChainId("chain-1".to_string()), 0) {
            Ok(subblock) => {
                logging::log("TEST", &format!("[TEST]   Successfully got subblock for chain-1: {:?}", subblock));
                assert_eq!(subblock.chain_id, ChainId("chain-1".to_string()), "Subblock should be for chain-1");
                assert_eq!(subblock.block_height, 0, "Subblock should be for block 0");
                assert!(subblock.transactions.is_empty(), "Initial subblock should be empty");
            },
            Err(e) => panic!("Failed to get subblock for chain-1: {}", e),
        }
    }

    // Submit transactions for different chains
    logging::log("TEST", "[TEST]   Submitting transactions...");
    {
        let mut cl_node_with_lock_2 = cl_node.lock().await;
        
        // Submit a transaction for chain1
        let cl_id_1 = CLTransactionId("cl-tx_1".to_string());
        let tx_chain_1 = Transaction::new(
            TransactionId(format!("{:?}:tx_1", cl_id_1)),
            ChainId("chain-1".to_string()),
            vec![ChainId("chain-1".to_string())],
            "REGULAR.credit 1 100".to_string(),
            cl_id_1.clone(),
        ).expect("Failed to create transaction");
        let cl_tx_chain_1 = CLTransaction::new(
            cl_id_1.clone(),
            vec![ChainId("chain-1".to_string())],
            vec![tx_chain_1],
        ).expect("Failed to create CL transaction");
        cl_node_with_lock_2.submit_transaction(cl_tx_chain_1).expect("Failed to submit transaction for chain-1");
        
        // Submit a transaction for chain2
        let cl_id_2 = CLTransactionId("cl-tx_2".to_string());
        let tx_chain_2 = Transaction::new(
            TransactionId(format!("{:?}:tx_2", cl_id_2)),
            ChainId("chain-2".to_string()),
            vec![ChainId("chain-2".to_string())],
            "REGULAR.credit 1 100".to_string(),
            cl_id_2.clone(),
        ).expect("Failed to create transaction");
        let cl_tx_chain_2 = CLTransaction::new(
            cl_id_2.clone(),
            vec![ChainId("chain-2".to_string())],
            vec![tx_chain_2],
        ).expect("Failed to create CL transaction");
        cl_node_with_lock_2.submit_transaction(cl_tx_chain_2).expect("Failed to submit transaction for chain-2");
        
        // Try to submit a transaction for unregistered chain (should fail)
        let cl_id_3 = CLTransactionId("cl-tx_3".to_string());
        let tx_chain_3 = Transaction::new(
            TransactionId(format!("{:?}:tx_3", cl_id_3)),
            ChainId("chain-3".to_string()),
            vec![ChainId("chain-3".to_string())],
            "REGULAR.credit 1 100".to_string(),
            cl_id_3.clone(),
        ).expect("Failed to create transaction");
        let cl_tx_chain_3 = CLTransaction::new(
            cl_id_3.clone(),
            vec![ChainId("chain-3".to_string())],
            vec![tx_chain_3],
        ).expect("Failed to create CL transaction");
        match cl_node_with_lock_2.submit_transaction(cl_tx_chain_3) {
            Ok(_) => panic!("Should not be able to submit transaction for unregistered chain"),
            Err(e) => logging::log("TEST", &format!("[TEST]   Expected error when submitting transaction for unregistered chain: {}", e)),
        }
    }

    // wait for 1 second
    sleep(Duration::from_secs(1)).await;

    // Spawn tasks to add more transactions for different chains
    let sender_for_chain1 = hs_node.get_sender_to_cl();
    let _adder_handle1 = tokio::spawn(async move {
        run_spammer_v12(sender_for_chain1, ChainId("chain-1".to_string())).await;
    });

    let sender_for_chain2 = hs_node.get_sender_to_cl();
    let _adder_handle2 = tokio::spawn(async move {
        run_spammer_v12(sender_for_chain2, ChainId("chain-2".to_string())).await;
    });

    // Wait for a few seconds to let the processor run
    logging::log("TEST", "Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let cl_node_with_lock_3 = cl_node.lock().await;
    logging::log("TEST", &format!("Main task: current block is {}", cl_node_with_lock_3.current_block));
    logging::log("TEST", &format!("Main task: processed {} transactions", cl_node_with_lock_3.processed_transactions.len()));
    logging::log("TEST", &format!("Main task: {} transactions still pending", cl_node_with_lock_3.pending_transactions.len()));
    logging::log("TEST", &format!("Main task: produced {} blocks", cl_node_with_lock_3.blocks.len()));
    logging::log("TEST", &format!("Main task: registered chains: {:?}", cl_node_with_lock_3.registered_chains));
    
    // Verify the state has been updated
    assert!(cl_node_with_lock_3.current_block > 0, "Block should have been incremented");
    assert!(!cl_node_with_lock_3.processed_transactions.is_empty(), "Should have processed some transactions");
    assert!(!cl_node_with_lock_3.blocks.is_empty(), "Should have produced some blocks");
    assert_eq!(cl_node_with_lock_3.registered_chains.len(), 2, "Should have exactly 2 registered chains");
    
    // Test getting subblock for registered chain
    match cl_node_with_lock_3.get_subblock(ChainId("chain-1".to_string()), 0) {
        Ok(subblock) => logging::log("TEST", &format!("  [TEST]   Successfully got subblock for chain-1: {:?}", subblock)),
        Err(e) => panic!("Failed to get subblock for chain-1: {}", e),
    }
    
    // Drop the first state lock
    drop(cl_node_with_lock_3);
    
    // Wait for a bit more to let transactions be processed
    sleep(Duration::from_secs(1)).await;
    
    // Make sure the processor task is still running by checking the state again
    let state_guard = cl_node.lock().await;
    let current_block = state_guard.current_block;
    let processed_count = state_guard.processed_transactions.len();
    let block_count = state_guard.blocks.len();
    logging::log("TEST", &format!("Main task: final check - block is {}, processed {} transactions in {} blocks", 
        current_block, processed_count, block_count));
    
    // Ensure the processor is still running and processing transactions
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 transactions per chain (one every 3 blocks)
    assert!(current_block > 25, "Block should have been incremented more than 25 times in 3 seconds, did {}", current_block);
    assert!(processed_count > 15, "Should have processed more than 15 transactions in 3 seconds (5 per chain), did {}", processed_count);
    assert!(block_count > 25, "Should have produced more than 25 blocks in 3 seconds, did {}", block_count);
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// v12: Node that matches CL node functionality
struct TestConfirmationLayerNode {
    msg_receiver: mpsc::Receiver<CLTransaction>,
    subblock_sender: mpsc::Sender<SubBlock>,
    block_interval: Duration,
    current_block: u64,
    processed_transactions: Vec<(ChainId, CLTransaction)>,
    pending_transactions: Vec<CLTransaction>,
    blocks: Vec<u64>,
    registered_chains: Vec<ChainId>,
}

impl TestConfirmationLayerNode {
    fn new(
        receiver_hs_to_cl: mpsc::Receiver<CLTransaction>,
        sender_cl_to_hig: mpsc::Sender<SubBlock>,
        block_interval: Duration,
    ) -> Self {
        Self {
            msg_receiver: receiver_hs_to_cl,
            subblock_sender: sender_cl_to_hig,
            block_interval,
            current_block: 0,
            processed_transactions: Vec::new(),
            pending_transactions: Vec::new(),
            blocks: Vec::new(),
            registered_chains: Vec::new(),
        }
    }

    fn register_chain(&mut self, chain_id: ChainId) -> Result<(), String> {
        if self.registered_chains.contains(&chain_id) {
            return Err(format!("Chain {} is already registered", chain_id.0));
        }
        self.registered_chains.push(chain_id);
        Ok(())
    }

    fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), String> {
        if !transaction.constituent_chains.iter().all(|c| self.registered_chains.contains(c)) {
            return Err(format!("Chain {} is not registered", transaction.constituent_chains[0].0));
        }
        self.pending_transactions.push(transaction);
        Ok(())
    }

    fn get_subblock(&self, chain_id: ChainId, block_height: u64) -> Result<SubBlock, String> {
        if !self.registered_chains.contains(&chain_id) {
            return Err(format!("Chain {} is not registered", chain_id.0));
        }
        // For simplicity, just return a dummy subblock
        Ok(SubBlock {
            chain_id: chain_id.clone(),
            block_height,
            transactions: self.processed_transactions
                .iter()
                .filter(|(cid, _)| cid == &chain_id)
                .map(|(_, cl_tx)| Transaction::new(
                    cl_tx.transactions[0].id.clone(),
                    cl_tx.transactions[0].target_chain_id.clone(),
                    cl_tx.transactions[0].constituent_chains.clone(),
                    cl_tx.transactions[0].data.clone(),
                    cl_tx.transactions[0].cl_id.clone(),
                ).expect("Failed to create transaction"))
                .collect(),
        })
    }

    async fn get_current_block(&self) -> Result<u64, String> {
        Ok(self.current_block)
    }
}

/// v12: Hyper Scheduler node
struct TestHSNodev12 {
    sender_hs_to_cl: mpsc::Sender<CLTransaction>,
    #[allow(dead_code)]
    receiver_hig_to_hs: mpsc::Receiver<CATStatusUpdate>,
}

impl TestHSNodev12 {
    fn new(
        sender_hs_to_cl: mpsc::Sender<CLTransaction>,
        receiver_hig_to_hs: mpsc::Receiver<CATStatusUpdate>,
    ) -> Self {
        Self {
            sender_hs_to_cl,
            receiver_hig_to_hs,
        }
    }

    pub fn get_sender_to_cl(&self) -> mpsc::Sender<CLTransaction> {
        self.sender_hs_to_cl.clone()
    }
}

/// v12: Hyper IG node
struct TestHIGNodev12 {
    #[allow(dead_code)]
    receiver_cl_to_hig: mpsc::Receiver<SubBlock>,
    #[allow(dead_code)]
    sender_hig_to_hs: mpsc::Sender<CATStatusUpdate>,
}

impl TestHIGNodev12 {
    fn new(
        receiver_cl_to_hig: mpsc::Receiver<SubBlock>,
        sender_hig_to_hs: mpsc::Sender<CATStatusUpdate>,
    ) -> Self {
        Self {
            receiver_cl_to_hig,
            sender_hig_to_hs,
        }
    }
}

/// Helper function to create test nodes with basic setup
async fn setup_test_nodes( interval: Duration) -> (TestHSNodev12, Arc<Mutex<TestConfirmationLayerNode>>, TestHIGNodev12) {
    // Create channels for communication
    let (sender_hs_to_cl, receiver_hs_to_cl) = mpsc::channel(100);
    let (sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel(100);
    let (sender_hig_to_hs, receiver_hig_to_hs) = mpsc::channel(100);
    
    // Create nodes with their channels
    let hs_node = TestHSNodev12::new(sender_hs_to_cl, receiver_hig_to_hs);
    let cl_node = Arc::new(Mutex::new(TestConfirmationLayerNode::new(
        receiver_hs_to_cl,
        sender_cl_to_hig,
        interval,
    )));
    let hig_node = TestHIGNodev12::new(receiver_cl_to_hig, sender_hig_to_hs);
    
    // Clone the state for the processor task
    let cl_node_for_processor = cl_node.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_transaction_processor_v12(cl_node_for_processor).await;
    });

    (hs_node, cl_node, hig_node)
}

/// Helper function to run the processor task
async fn run_transaction_processor_v12(cl_node: Arc<Mutex<TestConfirmationLayerNode>>) {
    let mut interval = interval(cl_node.lock().await.block_interval);
    loop {
        interval.tick().await;
        
        let mut state = cl_node.lock().await;
        
        // Process any new transactions from the channel
        while let Ok(cl_tx) = state.msg_receiver.try_recv() {
            logging::log("TEST", &format!("  [TEST] [Processor] received transaction for chains {:?}: {}", cl_tx.constituent_chains, cl_tx.transactions[0].data));
            if cl_tx.constituent_chains.iter().all(|c| state.registered_chains.contains(c)) {
                state.pending_transactions.push(cl_tx);
            }
        }
        
        state.current_block += 1;
        
        // Process pending transactions for this block
        let mut processed_this_block = Vec::new();
        let mut remaining = Vec::new();
        let registered_chains = state.registered_chains.clone();
        for tx in state.pending_transactions.drain(..) {
            if tx.constituent_chains.iter().all(|c| registered_chains.contains(c)) {
                processed_this_block.push((tx.constituent_chains[0].clone(), tx.clone()));
            } else {
                remaining.push(tx);
            }
        }
        state.pending_transactions = remaining;
        
        // Create a block
        let block_height = 0;
        state.blocks.push(block_height);
        
        // Send subblocks for each chain with only this block's transactions
        for chain_id in &state.registered_chains {
            let subblock = SubBlock {
                chain_id: chain_id.clone(),
                block_height,
                transactions: processed_this_block
                    .iter()
                    .filter(|(cid, _)| cid == chain_id)
                    .map(|(_, cl_tx)| Transaction::new(
                        cl_tx.transactions[0].id.clone(),
                        cl_tx.transactions[0].target_chain_id.clone(),
                        cl_tx.transactions[0].constituent_chains.clone(),
                        cl_tx.transactions[0].data.clone(),
                        cl_tx.transactions[0].cl_id.clone(),
                    ).expect("Failed to create transaction"))
                    .collect(),
            };
            if let Err(e) = state.subblock_sender.send(subblock).await {
                logging::log("TEST", &format!("  [TEST] [Processor] Error sending subblock: {}", e));
                break;
            }
        }
        state.processed_transactions.extend(processed_this_block.iter().cloned());
        
        // Print block status
        if !processed_this_block.is_empty() {
            let mut block_status = format!("  [TEST] [Processor] produced block {} with {} transactions", state.current_block, processed_this_block.len());
            for (_, cl_tx) in &processed_this_block {
                block_status.push_str(&format!("\n  - id={}, data={}", cl_tx.transactions[0].id.0, cl_tx.transactions[0].data));
            }
            logging::log("TEST", &block_status);
        } else {
            logging::log("TEST", &format!("  [TEST] [Processor] produced empty block {}", state.current_block));
        }
    }
}

/// Helper function to run the adder task
async fn run_spammer_v12(sender: mpsc::Sender<CLTransaction>, chain_id: ChainId) {
    for i in 1..=10 {
        let cl_id = CLTransactionId(format!("cl-tx_{}.{}", i, chain_id.0));
        let tx = Transaction::new(
            TransactionId(format!("{:?}:tx", cl_id)),
            chain_id.clone(),
            vec![chain_id.clone()],
            "REGULAR.credit 1 100".to_string(),
            cl_id.clone(),
        ).expect("Failed to create transaction");
        let cl_tx = CLTransaction::new(
            cl_id.clone(),
            vec![chain_id.clone()],
            vec![tx],
        ).expect("Failed to create CL transaction");
        if let Err(e) = sender.send(cl_tx).await {
            logging::log("TEST", &format!("  [TEST] [Adder] Error sending transaction: {}", e));
            break;
        }
        // wait for 100ms before sending next transaction (reduced from 300ms)
        sleep(Duration::from_millis(100)).await;
    }
}

#[derive(Debug, Clone)]
struct TestConfirmationLayer {
    registered_chains: HashSet<ChainId>,
    current_block: u64,
    processed_transactions: Vec<(ChainId, Transaction)>,
}

impl TestConfirmationLayer {
    fn new() -> Self {
        Self {
            registered_chains: HashSet::new(),
            current_block: 0,
            processed_transactions: Vec::new(),
        }
    }

    fn get_current_block(&self) -> Result<u64, String> {
        Ok(self.current_block)
    }

    fn register_chain(&mut self, chain_id: ChainId) -> Result<u64, String> {
        if self.registered_chains.contains(&chain_id) {
            return Err(format!("Chain {} is already registered", chain_id.0));
        }
        self.registered_chains.insert(chain_id);
        Ok(self.current_block)
    }

    fn get_subblock(&self, chain_id: ChainId, block_height: u64) -> Result<SubBlock, String> {
        if !self.registered_chains.contains(&chain_id) {
            return Err(format!("Chain {} is not registered", chain_id.0));
        }
        Ok(SubBlock {
            chain_id: chain_id.clone(),
            block_height,
            transactions: self.processed_transactions
                .iter()
                .filter(|(cid, _)| cid == &chain_id)
                .map(|(_, tx)| Transaction::new(
                    tx.id.clone(),
                    tx.target_chain_id.clone(),
                    tx.constituent_chains.clone(),
                    tx.data.clone(),
                    tx.cl_id.clone(),
                ).expect("Failed to create transaction"))
                .collect(),
        })
    }
}

#[tokio::test]
async fn test_confirmation_layer() {
    let mut cl = TestConfirmationLayer::new();
    let chain_id = ChainId("1".to_string());
    
    // Test registering a chain
    let block_height = cl.register_chain(chain_id.clone()).unwrap();
    assert_eq!(block_height, 0);
    
    // Test getting current block
    let current_block = cl.get_current_block().unwrap();
    assert_eq!(current_block, 0);
    
    // Test getting subblock
    let subblock = cl.get_subblock(chain_id.clone(), block_height).unwrap();
    assert_eq!(subblock.chain_id, chain_id);
    assert_eq!(subblock.block_height, block_height);
    assert!(subblock.transactions.is_empty());
}

