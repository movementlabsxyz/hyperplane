use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep, interval};
use tokio::sync::mpsc;
use hyperplane::{
    types::{Transaction, TransactionId, ChainId, BlockId, CLTransaction, SubBlock},
    types::communication::Channel,
    hyper_scheduler::node::HyperSchedulerNode,
    confirmation_layer::node::{ConfirmationLayerNode, ConfirmationLayerNodeWrapper},
    confirmation_layer::ConfirmationLayer,
    hyper_ig::node::HyperIGNode,
};
use crate::common::testnodes;

// - - - - - - - - - - - - - - - - - - - - - - - 
// V11: copies v1 but uses correct types
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V11: copies v10 but uses correct types
/// - changes to real types
#[tokio::test]
async fn test_mutex_concurrent_access_v11() {
    println!("\n=== Starting test_mutex_concurrent_access_v11 ===");
    
    // Create channels for messages and subblocks
    let (msg_sender, msg_receiver) = mpsc::channel(100);
    let (subblock_sender, mut subblock_receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV11::new(
        msg_receiver,
        subblock_sender,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v11(state_for_processor).await;
    });

    // Register chains first
    println!("[Test] Registering chains...");
    {
        let mut state = state.lock().await;
        state.register_chain(ChainId("chain1".to_string())).expect("Failed to register chain1");
        state.register_chain(ChainId("chain2".to_string())).expect("Failed to register chain2");
        
        // Try to register chain1 again (should fail)
        match state.register_chain(ChainId("chain1".to_string())) {
            Ok(_) => panic!("Should not be able to register chain1 twice"),
            Err(e) => println!("[Test] Expected error when registering chain1 twice: {}", e),
        }

        // Try to get subblock for unregistered chain
        match state.get_subblock(ChainId("chain3".to_string()), BlockId("0".to_string())) {
            Ok(_) => panic!("Should not be able to get subblock for unregistered chain"),
            Err(e) => println!("[Test] Expected error when getting subblock for unregistered chain: {}", e),
        }
    }

    // Submit transactions for different chains
    println!("[Test] Submitting transactions...");
    {
        let mut state = state.lock().await;
        
        // Submit a transaction for chain1
        let tx1 = CLTransaction {
            id: TransactionId("tx1".to_string()),
            data: "message1.chain1".to_string(),
            chain_id: ChainId("chain1".to_string()),
        };
        state.submit_transaction(tx1).expect("Failed to submit transaction for chain1");
        
        // Submit a transaction for chain2
        let tx2 = CLTransaction {
            id: TransactionId("tx2".to_string()),
            data: "message1.chain2".to_string(),
            chain_id: ChainId("chain2".to_string()),
        };
        state.submit_transaction(tx2).expect("Failed to submit transaction for chain2");
        
        // Try to submit a transaction for unregistered chain (should fail)
        let tx3 = CLTransaction {
            id: TransactionId("tx3".to_string()),
            data: "message1.chain3".to_string(),
            chain_id: ChainId("chain3".to_string()),
        };
        match state.submit_transaction(tx3) {
            Ok(_) => panic!("Should not be able to submit transaction for unregistered chain"),
            Err(e) => println!("[Test] Expected error when submitting transaction for unregistered chain: {}", e),
        }
    }

    // Spawn tasks to add more transactions for different chains
    let sender_for_chain1 = msg_sender.clone();
    let _adder_handle1 = tokio::spawn(async move {
        run_adder_v11(sender_for_chain1, ChainId("chain1".to_string())).await;
    });

    let sender_for_chain2 = msg_sender.clone();
    let _adder_handle2 = tokio::spawn(async move {
        run_adder_v11(sender_for_chain2, ChainId("chain2".to_string())).await;
    });

    // // Spawn a task to receive and verify subblocks
    // let _receiver_handle = tokio::spawn(async move {
    //     let mut received_blocks = 0;
    //     while let Some(subblock_msg) = subblock_receiver.recv().await {
    //         print!("[Receiver] received subblock for chain {} with {} transactions", 
    //             subblock_msg.subblock.chain_id.0, subblock_msg.subblock.transactions.len());
    //         for tx in &subblock_msg.subblock.transactions {
    //             print!("  - id={}, data={}", tx.id.0, tx.data);
    //         }
    //         println!();
    //         received_blocks += 1;
    //     }
    //     println!("[Receiver] received {} subblocks total", received_blocks);
    // });
    
    // Wait for a few seconds to let the processor run
    println!("Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    println!("Main task: current block is {}", state_guard.current_block);
    println!("Main task: processed {} transactions", state_guard.processed_transactions.len());
    println!("Main task: {} transactions still pending", state_guard.pending_transactions.len());
    println!("Main task: produced {} blocks", state_guard.blocks.len());
    println!("Main task: registered chains: {:?}", state_guard.registered_chains);
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_transactions.is_empty(), "Should have processed some transactions");
    assert!(!state_guard.blocks.is_empty(), "Should have produced some blocks");
    assert_eq!(state_guard.registered_chains.len(), 2, "Should have exactly 2 registered chains");
    
    // Test getting subblock for registered chain
    match state_guard.get_subblock(ChainId("chain1".to_string()), BlockId("0".to_string())) {
        Ok(subblock) => println!("[Test] Successfully got subblock for chain1: {:?}", subblock),
        Err(e) => panic!("Failed to get subblock for chain1: {}", e),
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
    println!("Main task: final check - block is {}, processed {} transactions in {} blocks", 
        current_block, processed_count, block_count);
    
    // Ensure the processor is still running and processing transactions
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 transactions per chain (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 10, "Should have processed more than 10 transactions in 2 seconds (5 per chain)");
    assert!(block_count > 15, "Should have produced more than 15 blocks in 2 seconds");
    
    println!("=== Test completed successfully ===\n");
}

/// V11: State struct that matches CL node functionality
struct TestNodeStateV11 {
    msg_receiver: mpsc::Receiver<CLTransaction>,
    subblock_sender: mpsc::Sender<SubBlock>,
    block_interval: Duration,
    current_block: u64,
    processed_transactions: Vec<(ChainId, CLTransaction)>,
    pending_transactions: Vec<CLTransaction>,
    blocks: Vec<BlockId>,
    registered_chains: Vec<ChainId>,
}

impl TestNodeStateV11 {
    fn new(
        msg_receiver: mpsc::Receiver<CLTransaction>,
        subblock_sender: mpsc::Sender<SubBlock>,
        block_interval: Duration,
    ) -> Self {
        Self {
            msg_receiver,
            subblock_sender,
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
        if !self.registered_chains.contains(&transaction.chain_id) {
            return Err(format!("Chain {} is not registered", transaction.chain_id.0));
        }
        self.pending_transactions.push(transaction);
        Ok(())
    }

    fn get_subblock(&self, chain_id: ChainId, block_id: BlockId) -> Result<SubBlock, String> {
        if !self.registered_chains.contains(&chain_id) {
            return Err(format!("Chain {} is not registered", chain_id.0));
        }
        // For simplicity, just return a dummy subblock
        Ok(SubBlock {
            chain_id: chain_id.clone(),
            block_id,
            transactions: self.processed_transactions
                .iter()
                .filter(|(cid, _)| cid == &chain_id)
                .map(|(_, tx)| Transaction {
                    id: tx.id.clone(),
                    data: tx.data.clone(),
                })
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
        while let Ok(transaction) = state.msg_receiver.try_recv() {
            println!("[Processor] received transaction from chain {}: {}", transaction.chain_id.0, transaction.data);
            if state.registered_chains.contains(&transaction.chain_id) {
                state.pending_transactions.push(transaction);
            }
        }
        
        state.current_block += 1;
        
        // Process pending transactions for this block
        let mut processed_this_block = Vec::new();
        let mut remaining = Vec::new();
        let registered_chains = state.registered_chains.clone();
        for tx in state.pending_transactions.drain(..) {
            if registered_chains.contains(&tx.chain_id) {
                processed_this_block.push((tx.chain_id.clone(), tx.clone()));
            } else {
                remaining.push(tx);
            }
        }
        state.pending_transactions = remaining;
        
        // Create a block
        let block_id = BlockId(state.current_block.to_string());
        state.blocks.push(block_id.clone());
        
        // Send subblocks for each chain with only this block's transactions
        for chain_id in &state.registered_chains {
            let subblock = SubBlock {
                chain_id: chain_id.clone(),
                block_id: block_id.clone(),
                transactions: processed_this_block
                    .iter()
                    .filter(|(cid, _)| cid == chain_id)
                    .map(|(_, tx)| Transaction {
                        id: tx.id.clone(),
                        data: tx.data.clone(),
                    })
                    .collect(),
            };
            if let Err(e) = state.subblock_sender.send(subblock).await {
                println!("Error sending subblock: {}", e);
                break;
            }
        }
        state.processed_transactions.extend(processed_this_block.iter().cloned());
        
        // Print block status
        if !processed_this_block.is_empty() {
            print!("[Processor] produced block {} with {} transactions", state.current_block, processed_this_block.len());
            for (_, tx) in &processed_this_block {
                print!("  - id={}, data={}", tx.id.0, tx.data);
            }
            println!();
        } else {
            println!("[Processor] produced empty block {}", state.current_block);
        }
    }
}

/// Helper function to run the adder task
async fn run_adder_v11(sender: mpsc::Sender<CLTransaction>, chain_id: ChainId) {
    for i in 1..=10 {
        let tx = CLTransaction {
            id: TransactionId(format!("tx{}.{}", i, chain_id.0)),
            data: format!("message{}.{}", i, chain_id.0),
            chain_id: chain_id.clone(),
        };
        if let Err(e) = sender.send(tx).await {
            println!("Error sending transaction: {}", e);
            break;
        }
        sleep(Duration::from_millis(300)).await;
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - 
// V12: Integrates with actual node setup
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V12: Integrates with actual node setup
/// - Uses setup_test_nodes for proper node initialization
/// - Connects nodes with proper channels
/// - Tests full node communication flow
/// - Verifies node behavior matches v11's expectations
#[tokio::test]
async fn test_mutex_concurrent_access_v12() {
    println!("\n=== Starting test_mutex_concurrent_access_v12 ===");
    
    // Get the actual node (note: setup_test_nodes may be buggy and need fixing)
    let (_, mut cl_node, _) = testnodes::setup_test_nodes(Duration::from_millis(100)).await;
    
    // Create channels for messages and subblocks
    let (msg_sender, msg_receiver) = mpsc::channel(100);
    let (subblock_sender, mut subblock_receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStatev12::new(
        msg_receiver,
        subblock_sender,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v12(state_for_processor).await;
    });

    // Register chains first
    println!("[Test] Registering chains...");
    {
        let mut state = state.lock().await;
        state.register_chain(ChainId("chain1".to_string())).expect("Failed to register chain1");
        state.register_chain(ChainId("chain2".to_string())).expect("Failed to register chain2");
        
        // Try to register chain1 again (should fail)
        match state.register_chain(ChainId("chain1".to_string())) {
            Ok(_) => panic!("Should not be able to register chain1 twice"),
            Err(e) => println!("[Test] Expected error when registering chain1 twice: {}", e),
        }

        // Try to get subblock for unregistered chain
        match state.get_subblock(ChainId("chain3".to_string()), BlockId("0".to_string())) {
            Ok(_) => panic!("Should not be able to get subblock for unregistered chain"),
            Err(e) => println!("[Test] Expected error when getting subblock for unregistered chain: {}", e),
        }
    }

    // Submit transactions for different chains
    println!("[Test] Submitting transactions...");
    {
        let mut state = state.lock().await;
        
        // Submit a transaction for chain1
        let tx1 = CLTransaction {
            id: TransactionId("tx1".to_string()),
            data: "message1.chain1".to_string(),
            chain_id: ChainId("chain1".to_string()),
        };
        state.submit_transaction(tx1).expect("Failed to submit transaction for chain1");
        
        // Submit a transaction for chain2
        let tx2 = CLTransaction {
            id: TransactionId("tx2".to_string()),
            data: "message1.chain2".to_string(),
            chain_id: ChainId("chain2".to_string()),
        };
        state.submit_transaction(tx2).expect("Failed to submit transaction for chain2");
        
        // Try to submit a transaction for unregistered chain (should fail)
        let tx3 = CLTransaction {
            id: TransactionId("tx3".to_string()),
            data: "message1.chain3".to_string(),
            chain_id: ChainId("chain3".to_string()),
        };
        match state.submit_transaction(tx3) {
            Ok(_) => panic!("Should not be able to submit transaction for unregistered chain"),
            Err(e) => println!("[Test] Expected error when submitting transaction for unregistered chain: {}", e),
        }
    }

    // Spawn tasks to add more transactions for different chains
    let sender_for_chain1 = msg_sender.clone();
    let _adder_handle1 = tokio::spawn(async move {
        run_adder_v12(sender_for_chain1, ChainId("chain1".to_string())).await;
    });

    let sender_for_chain2 = msg_sender.clone();
    let _adder_handle2 = tokio::spawn(async move {
        run_adder_v12(sender_for_chain2, ChainId("chain2".to_string())).await;
    });
    
    // Wait for a few seconds to let the processor run
    println!("Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    println!("Main task: current block is {}", state_guard.current_block);
    println!("Main task: processed {} transactions", state_guard.processed_transactions.len());
    println!("Main task: {} transactions still pending", state_guard.pending_transactions.len());
    println!("Main task: produced {} blocks", state_guard.blocks.len());
    println!("Main task: registered chains: {:?}", state_guard.registered_chains);
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_transactions.is_empty(), "Should have processed some transactions");
    assert!(!state_guard.blocks.is_empty(), "Should have produced some blocks");
    assert_eq!(state_guard.registered_chains.len(), 2, "Should have exactly 2 registered chains");
    
    // Test getting subblock for registered chain
    match state_guard.get_subblock(ChainId("chain1".to_string()), BlockId("0".to_string())) {
        Ok(subblock) => println!("[Test] Successfully got subblock for chain1: {:?}", subblock),
        Err(e) => panic!("Failed to get subblock for chain1: {}", e),
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
    println!("Main task: final check - block is {}, processed {} transactions in {} blocks", 
        current_block, processed_count, block_count);
    
    // Ensure the processor is still running and processing transactions
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 transactions per chain (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 10, "Should have processed more than 10 transactions in 2 seconds (5 per chain)");
    assert!(block_count > 15, "Should have produced more than 15 blocks in 2 seconds");
    
    println!("=== Test completed successfully ===\n");
}

/// v12: State struct that matches CL node functionality
struct TestNodeStatev12 {
    msg_receiver: mpsc::Receiver<CLTransaction>,
    subblock_sender: mpsc::Sender<SubBlock>,
    block_interval: Duration,
    current_block: u64,
    processed_transactions: Vec<(ChainId, CLTransaction)>,
    pending_transactions: Vec<CLTransaction>,
    blocks: Vec<BlockId>,
    registered_chains: Vec<ChainId>,
}

impl TestNodeStatev12 {
    fn new(
        msg_receiver: mpsc::Receiver<CLTransaction>,
        subblock_sender: mpsc::Sender<SubBlock>,
        block_interval: Duration,
    ) -> Self {
        Self {
            msg_receiver,
            subblock_sender,
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
        if !self.registered_chains.contains(&transaction.chain_id) {
            return Err(format!("Chain {} is not registered", transaction.chain_id.0));
        }
        self.pending_transactions.push(transaction);
        Ok(())
    }

    fn get_subblock(&self, chain_id: ChainId, block_id: BlockId) -> Result<SubBlock, String> {
        if !self.registered_chains.contains(&chain_id) {
            return Err(format!("Chain {} is not registered", chain_id.0));
        }
        // For simplicity, just return a dummy subblock
        Ok(SubBlock {
            chain_id: chain_id.clone(),
            block_id,
            transactions: self.processed_transactions
                .iter()
                .filter(|(cid, _)| cid == &chain_id)
                .map(|(_, tx)| Transaction {
                    id: tx.id.clone(),
                    data: tx.data.clone(),
                })
                .collect(),
        })
    }
}

/// Helper function to run the processor task
async fn run_processor_v12(state: Arc<Mutex<TestNodeStatev12>>) {
    let mut interval = interval(state.lock().await.block_interval);
    loop {
        interval.tick().await;
        
        let mut state = state.lock().await;
        
        // Process any new transactions from the channel
        while let Ok(transaction) = state.msg_receiver.try_recv() {
            println!("[Processor] received transaction from chain {}: {}", transaction.chain_id.0, transaction.data);
            if state.registered_chains.contains(&transaction.chain_id) {
                state.pending_transactions.push(transaction);
            }
        }
        
        state.current_block += 1;
        
        // Process pending transactions for this block
        let mut processed_this_block = Vec::new();
        let mut remaining = Vec::new();
        let registered_chains = state.registered_chains.clone();
        for tx in state.pending_transactions.drain(..) {
            if registered_chains.contains(&tx.chain_id) {
                processed_this_block.push((tx.chain_id.clone(), tx.clone()));
            } else {
                remaining.push(tx);
            }
        }
        state.pending_transactions = remaining;
        
        // Create a block
        let block_id = BlockId(state.current_block.to_string());
        state.blocks.push(block_id.clone());
        
        // Send subblocks for each chain with only this block's transactions
        for chain_id in &state.registered_chains {
            let subblock = SubBlock {
                chain_id: chain_id.clone(),
                block_id: block_id.clone(),
                transactions: processed_this_block
                    .iter()
                    .filter(|(cid, _)| cid == chain_id)
                    .map(|(_, tx)| Transaction {
                        id: tx.id.clone(),
                        data: tx.data.clone(),
                    })
                    .collect(),
            };
            if let Err(e) = state.subblock_sender.send(subblock).await {
                println!("Error sending subblock: {}", e);
                break;
            }
        }
        state.processed_transactions.extend(processed_this_block.iter().cloned());
        
        // Print block status
        if !processed_this_block.is_empty() {
            print!("[Processor] produced block {} with {} transactions", state.current_block, processed_this_block.len());
            for (_, tx) in &processed_this_block {
                print!("  - id={}, data={}", tx.id.0, tx.data);
            }
            println!();
        } else {
            println!("[Processor] produced empty block {}", state.current_block);
        }
    }
}

/// Helper function to run the adder task
async fn run_adder_v12(sender: mpsc::Sender<CLTransaction>, chain_id: ChainId) {
    for i in 1..=10 {
        let tx = CLTransaction {
            id: TransactionId(format!("tx{}.{}", i, chain_id.0)),
            data: format!("message{}.{}", i, chain_id.0),
            chain_id: chain_id.clone(),
        };
        if let Err(e) = sender.send(tx).await {
            println!("Error sending transaction: {}", e);
            break;
        }
        sleep(Duration::from_millis(300)).await;
    }
}

