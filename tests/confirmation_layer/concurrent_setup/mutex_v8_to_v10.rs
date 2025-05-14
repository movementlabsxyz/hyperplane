use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep, interval};
use tokio::sync::mpsc;

/// A simplified version of a chain ID
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ChainId(String);

/// A simplified version of a block ID
#[derive(Clone, Debug, PartialEq, Eq)]
struct BlockId(String);

/// A simplified version of a transaction ID
#[derive(Clone, Debug, PartialEq, Eq)]
struct TransactionId(String);

/// A simplified version of a transaction
#[derive(Clone, Debug)]
struct Transaction {
    id: TransactionId,
    data: String,
    chain_id: ChainId,
}

/// A simplified version of a subblock
#[derive(Clone, Debug)]
struct SubBlock {
    chain_id: String,
    #[allow(dead_code)]
    block_id: u64,
    messages: Vec<String>,
}

/// A simplified version of a block
#[derive(Clone, Debug)]
struct Block {
    #[allow(dead_code)]
    id: u64,
    messages: Vec<String>,
}

// - - - - - - - - - - - - - - - - - - - - - - - 
// V8: Adds error handling (like CL node)
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V8: Adds error handling (like CL node)
/// - Adds proper error types
/// - Handles chain not found
/// - Handles duplicate registration
/// - Validates block intervals
/// - Still keeps the simple mutex pattern
#[tokio::test]
async fn test_mutex_concurrent_access_v8() {
    println!("\n=== Starting test_mutex_concurrent_access_v8 ===");
    
    // Create channels for messages and subblocks
    let (msg_sender, msg_receiver) = mpsc::channel(100);
    let (subblock_sender, mut subblock_receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV8::new(
        msg_receiver,
        subblock_sender,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v8(state_for_processor).await;
    });

    // Register chains first
    println!("[Test] Registering chains...");
    {
        let mut state = state.lock().await;
        state.register_chain("chain1").expect("Failed to register chain1");
        state.register_chain("chain2").expect("Failed to register chain2");
        
        // Try to register chain1 again (should fail)
        match state.register_chain("chain1") {
            Ok(_) => panic!("Should not be able to register chain1 twice"),
            Err(e) => println!("[Test] Expected error when registering chain1 twice: {}", e),
        }

        // Try to get subblock for unregistered chain
        match state.get_subblock("chain3", 0) {
            Ok(_) => panic!("Should not be able to get subblock for unregistered chain"),
            Err(e) => println!("[Test] Expected error when getting subblock for unregistered chain: {}", e),
        }
    }

    // Spawn tasks to add messages for different chains
    let sender_for_chain1 = msg_sender.clone();
    let _adder_handle1 = tokio::spawn(async move {
        run_adder_v8(sender_for_chain1, "chain1").await;
    });

    let sender_for_chain2 = msg_sender.clone();
    let _adder_handle2 = tokio::spawn(async move {
        run_adder_v8(sender_for_chain2, "chain2").await;
    });

    // Try to add messages for an unregistered chain
    let sender_for_chain3 = msg_sender.clone();
    let _adder_handle3 = tokio::spawn(async move {
        run_adder_v8(sender_for_chain3, "chain3").await;
    });

    // Spawn a task to receive and verify subblocks
    let _receiver_handle = tokio::spawn(async move {
        let mut received_blocks = 0;
        while let Some(subblock) = subblock_receiver.recv().await {
            print!("[Receiver] received subblock for chain {} with {} messages", 
                subblock.chain_id, subblock.messages.len());
            for msg in &subblock.messages {
                print!("  - \"{}\"", msg);
            }
            println!();
            received_blocks += 1;
        }
        println!("[Receiver] received {} subblocks total", received_blocks);
    });
    
    // Wait for a few seconds to let the processor run
    println!("Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    println!("Main task: current block is {}", state_guard.current_block);
    println!("Main task: processed {} messages", state_guard.processed_messages.len());
    println!("Main task: {} messages still pending", state_guard.pending_messages.len());
    println!("Main task: produced {} blocks", state_guard.blocks.len());
    println!("Main task: registered chains: {:?}", state_guard.registered_chains);
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_messages.is_empty(), "Should have processed some messages");
    assert!(!state_guard.blocks.is_empty(), "Should have produced some blocks");
    assert_eq!(state_guard.registered_chains.len(), 2, "Should have exactly 2 registered chains");
    
    // Test getting subblock for registered chain
    match state_guard.get_subblock("chain1", 0) {
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
    let processed_count = state_guard.processed_messages.len();
    let block_count = state_guard.blocks.len();
    println!("Main task: final check - block is {}, processed {} messages in {} blocks", 
        current_block, processed_count, block_count);
    
    // Ensure the processor is still running and processing messages
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 messages per chain (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 10, "Should have processed more than 10 messages in 2 seconds (5 per chain)");
    assert!(block_count > 15, "Should have produced more than 15 blocks in 2 seconds");
    
    println!("=== Test completed successfully ===\n");
}

/// Error types for the CL node
#[derive(Debug)]
#[allow(dead_code)]
enum NodeError {
    ChainNotFound(String),
    ChainAlreadyRegistered(String),
    InvalidBlockInterval,
    BlockNotFound(u64),
}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeError::ChainNotFound(chain_id) => write!(f, "Chain {} not found", chain_id),
            NodeError::ChainAlreadyRegistered(chain_id) => write!(f, "Chain {} is already registered", chain_id),
            NodeError::InvalidBlockInterval => write!(f, "Invalid block interval"),
            NodeError::BlockNotFound(block_id) => write!(f, "Block {} not found", block_id),
        }
    }
}

/// A simplified version of ConfirmationLayerNode's state with message channel and block production
struct TestNodeStateV8 {
    current_block: u64,
    pending_messages: Vec<(String, String)>, // (chain_id, message)
    processed_messages: Vec<(String, String)>, // (chain_id, message)
    message_receiver: mpsc::Receiver<(String, String)>, // (chain_id, message)
    subblock_sender: mpsc::Sender<SubBlock>,
    blocks: Vec<Block>,
    block_interval: Duration,
    registered_chains: std::collections::HashSet<String>,
}

impl TestNodeStateV8 {
    fn new(
        message_receiver: mpsc::Receiver<(String, String)>,
        subblock_sender: mpsc::Sender<SubBlock>,
        block_interval: Duration,
    ) -> Self {
        Self {
            current_block: 0,
            pending_messages: Vec::new(),
            processed_messages: Vec::new(),
            message_receiver,
            subblock_sender,
            blocks: Vec::new(),
            block_interval,
            registered_chains: std::collections::HashSet::new(),
        }
    }

    fn register_chain(&mut self, chain_id: &str) -> Result<(), NodeError> {
        if self.registered_chains.contains(chain_id) {
            return Err(NodeError::ChainAlreadyRegistered(chain_id.to_string()));
        }
        self.registered_chains.insert(chain_id.to_string());
        Ok(())
    }

    fn is_chain_registered(&self, chain_id: &str) -> bool {
        self.registered_chains.contains(chain_id)
    }

    fn get_subblock(&self, chain_id: &str, block_id: u64) -> Result<SubBlock, NodeError> {
        if !self.is_chain_registered(chain_id) {
            return Err(NodeError::ChainNotFound(chain_id.to_string()));
        }

        if block_id >= self.current_block {
            return Err(NodeError::BlockNotFound(block_id));
        }

        // Find messages for this chain in the block
        let block = &self.blocks[block_id as usize];
        let messages: Vec<String> = block.messages
            .iter()
            .filter(|msg| msg.starts_with(&format!("[{}]", chain_id)))
            .map(|msg| msg.split("] ").nth(1).unwrap_or("").to_string())
            .collect();

        Ok(SubBlock {
            chain_id: chain_id.to_string(),
            block_id,
            messages,
        })
    }
}

/// A function that continuously processes messages and updates state
async fn run_processor_v8(state: Arc<Mutex<TestNodeStateV8>>) {
    println!("[Processor] task started");
    
    // Get the block interval
    let block_interval = {
        let state = state.lock().await;
        state.block_interval
    };
    
    // Create an interval for block production
    let mut interval = interval(block_interval);
    
    loop {
        // Wait for the next block interval
        interval.tick().await;
        
        // Acquire the lock and process messages
        let mut state = state.lock().await;
        
        // Check for new messages from channel
        while let Ok((chain_id, message)) = state.message_receiver.try_recv() {
            if state.is_chain_registered(&chain_id) {
                println!("[Processor] received message from chain {}: {}", chain_id, message);
                state.pending_messages.push((chain_id, message));
            } else {
                println!("[Processor] ignoring message from unregistered chain {}: {}", chain_id, message);
            }
        }
        
        // Create a new block
        let block_id = state.current_block;
        let mut block = Block {
            id: block_id,
            messages: Vec::new(),
        };
        
        // Group messages by chain
        let mut chain_messages: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        
        // Move pending messages to the block and group by chain
        while !state.pending_messages.is_empty() {
            let (chain_id, message) = state.pending_messages.remove(0);
            let formatted_message = format!("[{}] {}", chain_id, message);
            block.messages.push(formatted_message.clone());
            state.processed_messages.push((chain_id.clone(), message.clone()));
            
            // Group messages by chain for subblocks
            chain_messages.entry(chain_id).or_default().push(message);
        }
        
        // Store the block
        state.blocks.push(block.clone());
        state.current_block += 1;
        
        // Create and send subblocks for each chain
        for (chain_id, messages) in chain_messages {
            if !messages.is_empty() {
                let subblock = SubBlock {
                    chain_id: chain_id.clone(),
                    block_id,
                    messages,
                };
                
                // Send the subblock
                if let Err(e) = state.subblock_sender.send(subblock.clone()).await {
                    println!("[Processor] failed to send subblock for chain {}: {}", chain_id, e);
                }
            }
        }
        
        // Print block status
        if !block.messages.is_empty() {
            print!("[Processor] produced block {} with {} messages", block_id, block.messages.len());
            for msg in &block.messages {
                print!("  - \"{}\"", msg);
            }
            println!();
        } else {
            println!("[Processor] produced empty block {}", block_id);
        }
        
        // Release the lock by dropping state
        drop(state);
    }
}

/// A function that gradually adds messages to the state through channel
async fn run_adder_v8(sender: mpsc::Sender<(String, String)>, chain_id: &str) {
    println!("[Adder-{}] task started", chain_id);
    for i in 1..=7 {
        // Wait for ~3 blocks (300ms) before adding next message
        sleep(Duration::from_millis(300)).await;
        let message = format!("message{}", i);
        if let Err(e) = sender.send((chain_id.to_string(), message.clone())).await {
            println!("[Adder-{}] failed to send message: {}", chain_id, e);
            break;
        }
        println!("[Adder-{}] sent message{}", chain_id, i);
    }
    println!("[Adder-{}] task completed", chain_id);
}

// - - - - - - - - - - - - - - - - - - - - - - - 
// V9: Adds proper types (like CL node)
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V9: Adds proper types (like CL node)
/// - Uses proper BlockId type
/// - Uses proper Transaction type
/// - Uses proper ChainId type
/// - Still keeps the simple mutex pattern
#[tokio::test]
async fn test_mutex_concurrent_access_v9() {
    println!("\n=== Starting test_mutex_concurrent_access_v9 ===");
    
    // Create channels for messages and subblocks
    let (msg_sender, msg_receiver) = mpsc::channel(100);
    let (subblock_sender, mut subblock_receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV9::new(
        msg_receiver,
        subblock_sender,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v9(state_for_processor).await;
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

    // Spawn tasks to add messages for different chains
    let sender_for_chain1 = msg_sender.clone();
    let _adder_handle1 = tokio::spawn(async move {
        run_adder_v9(sender_for_chain1, ChainId("chain1".to_string())).await;
    });

    let sender_for_chain2 = msg_sender.clone();
    let _adder_handle2 = tokio::spawn(async move {
        run_adder_v9(sender_for_chain2, ChainId("chain2".to_string())).await;
    });

    // Try to add messages for an unregistered chain
    let sender_for_chain3 = msg_sender.clone();
    let _adder_handle3 = tokio::spawn(async move {
        run_adder_v9(sender_for_chain3, ChainId("chain3".to_string())).await;
    });

    // Spawn a task to receive and verify subblocks
    let _receiver_handle = tokio::spawn(async move {
        let mut received_blocks = 0;
        while let Some(subblock) = subblock_receiver.recv().await {
            print!("[Receiver] received subblock for chain {} with {} transactions", 
                subblock.chain_id.0, subblock.transactions.len());
            for tx in &subblock.transactions {
                print!("  - id={}, data={}", tx.id.0, tx.data);
            }
            println!();
            received_blocks += 1;
        }
        println!("[Receiver] received {} subblocks total", received_blocks);
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

/// A simplified version of a subblock with proper types
#[derive(Clone, Debug)]
struct SubBlockV9 {
    chain_id: ChainId,
    #[allow(dead_code)]
    block_id: BlockId,
    transactions: Vec<Transaction>,
}

/// A simplified version of a block with proper types
#[derive(Clone, Debug)]
struct BlockV9 {
    #[allow(dead_code)]
    id: BlockId,
    transactions: Vec<Transaction>,
}

/// A simplified version of ConfirmationLayerNode's state with proper types
struct TestNodeStateV9 {
    current_block: u64,
    pending_transactions: Vec<(ChainId, Transaction)>, // (chain_id, transaction)
    processed_transactions: Vec<(ChainId, Transaction)>, // (chain_id, transaction)
    message_receiver: mpsc::Receiver<(ChainId, Transaction)>, // (chain_id, transaction)
    subblock_sender: mpsc::Sender<SubBlockV9>,
    blocks: Vec<BlockV9>,
    block_interval: Duration,
    registered_chains: std::collections::HashSet<ChainId>,
}

impl TestNodeStateV9 {
    fn new(
        message_receiver: mpsc::Receiver<(ChainId, Transaction)>,
        subblock_sender: mpsc::Sender<SubBlockV9>,
        block_interval: Duration,
    ) -> Self {
        Self {
            current_block: 0,
            pending_transactions: Vec::new(),
            processed_transactions: Vec::new(),
            message_receiver,
            subblock_sender,
            blocks: Vec::new(),
            block_interval,
            registered_chains: std::collections::HashSet::new(),
        }
    }

    fn register_chain(&mut self, chain_id: ChainId) -> Result<(), NodeError> {
        if self.registered_chains.contains(&chain_id) {
            return Err(NodeError::ChainAlreadyRegistered(chain_id.0));
        }
        self.registered_chains.insert(chain_id);
        Ok(())
    }

    fn is_chain_registered(&self, chain_id: &ChainId) -> bool {
        self.registered_chains.contains(chain_id)
    }

    fn get_subblock(&self, chain_id: ChainId, block_id: BlockId) -> Result<SubBlockV9, NodeError> {
        if !self.is_chain_registered(&chain_id) {
            return Err(NodeError::ChainNotFound(chain_id.0));
        }

        let block_num = block_id.0.parse::<u64>().map_err(|_| NodeError::BlockNotFound(0))?;
        if block_num >= self.current_block {
            return Err(NodeError::BlockNotFound(block_num));
        }

        // Find transactions for this chain in the block
        let block = &self.blocks[block_num as usize];
        let transactions: Vec<Transaction> = block.transactions
            .iter()
            .filter(|tx| tx.chain_id == chain_id)
            .cloned()
            .collect();

        Ok(SubBlockV9 {
            chain_id,
            block_id,
            transactions,
        })
    }
}

/// A function that continuously processes messages and updates state
async fn run_processor_v9(state: Arc<Mutex<TestNodeStateV9>>) {
    println!("[Processor] task started");
    
    // Get the block interval
    let block_interval = {
        let state = state.lock().await;
        state.block_interval
    };
    
    // Create an interval for block production
    let mut interval = interval(block_interval);
    
    loop {
        // Wait for the next block interval
        interval.tick().await;
        
        // Acquire the lock and process messages
        let mut state = state.lock().await;
        
        // Check for new messages from channel
        while let Ok((chain_id, transaction)) = state.message_receiver.try_recv() {
            if state.is_chain_registered(&chain_id) {
                println!("[Processor] received transaction from chain {}: {}", chain_id.0, transaction.data);
                state.pending_transactions.push((chain_id, transaction));
            } else {
                println!("[Processor] ignoring transaction from unregistered chain {}: {}", chain_id.0, transaction.data);
            }
        }
        
        // Create a new block
        let block_id = BlockId(state.current_block.to_string());
        let mut block = BlockV9 {
            id: block_id.clone(),
            transactions: Vec::new(),
        };
        
        // Group transactions by chain
        let mut chain_transactions: std::collections::HashMap<ChainId, Vec<Transaction>> = std::collections::HashMap::new();
        
        // Move pending transactions to the block and group by chain
        while !state.pending_transactions.is_empty() {
            let (chain_id, transaction) = state.pending_transactions.remove(0);
            block.transactions.push(transaction.clone());
            state.processed_transactions.push((chain_id.clone(), transaction.clone()));
            
            // Group transactions by chain for subblocks
            chain_transactions.entry(chain_id).or_default().push(transaction);
        }
        
        // Store the block
        state.blocks.push(block.clone());
        state.current_block += 1;
        
        // Create and send subblocks for each chain
        for (chain_id, transactions) in chain_transactions {
            if !transactions.is_empty() {
                let subblock = SubBlockV9 {
                    chain_id: chain_id.clone(),
                    block_id: block_id.clone(),
                    transactions,
                };
                
                // Send the subblock
                if let Err(e) = state.subblock_sender.send(subblock.clone()).await {
                    println!("[Processor] failed to send subblock for chain {}: {}", chain_id.0, e);
                }
            }
        }
        
        // Print block status
        if !block.transactions.is_empty() {
            print!("[Processor] produced block {} with {} transactions", block_id.0, block.transactions.len());
            for tx in &block.transactions {
                print!("  - id={}, data={}", tx.id.0, tx.data);
            }
            println!();
        } else {
            println!("[Processor] produced empty block {}", block_id.0);
        }
        
        // Release the lock by dropping state
        drop(state);
    }
}

/// A function that gradually adds messages to the state through channel
async fn run_adder_v9(sender: mpsc::Sender<(ChainId, Transaction)>, chain_id: ChainId) {
    println!("[Adder-{}] task started", chain_id.0);
    for i in 1..=7 {
        // Wait for ~3 blocks (300ms) before adding next message
        sleep(Duration::from_millis(300)).await;
        let transaction = Transaction {
            id: TransactionId(format!("tx{}", i)),
            data: format!("message{}.{}", i, chain_id.0),
            chain_id: chain_id.clone(),
        };
        if let Err(e) = sender.send((chain_id.clone(), transaction.clone())).await {
            println!("[Adder-{}] failed to send transaction: {}", chain_id.0, e);
            break;
        }
        println!("[Adder-{}] sent transaction{}", chain_id.0, i);
    }
    println!("[Adder-{}] task completed", chain_id.0);
}

// - - - - - - - - - - - - - - - - - - - - - - - 
// V10: Adds full CL node functionality
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V10: Adds full CL node functionality
/// - Combines all previous features
/// - Matches test_basic functionality
/// - Still keeps the simple mutex pattern
#[tokio::test]
async fn test_mutex_concurrent_access_v10() {
    println!("\n=== Starting test_mutex_concurrent_access_v10 ===");
    
    // Create channels for messages and subblocks
    let (msg_sender, msg_receiver) = mpsc::channel(100);
    let (subblock_sender, mut subblock_receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV10::new(
        msg_receiver,
        subblock_sender,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v10(state_for_processor).await;
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
        run_adder_v10(sender_for_chain1, ChainId("chain1".to_string())).await;
    });

    let sender_for_chain2 = msg_sender.clone();
    let _adder_handle2 = tokio::spawn(async move {
        run_adder_v10(sender_for_chain2, ChainId("chain2".to_string())).await;
    });

    // Spawn a task to receive and verify subblocks
    let _receiver_handle = tokio::spawn(async move {
        let mut received_blocks = 0;
        while let Some(subblock) = subblock_receiver.recv().await {
            print!("[Receiver] received subblock for chain {} with {} transactions", 
                subblock.chain_id.0, subblock.transactions.len());
            for tx in &subblock.transactions {
                print!("  - id={}, data={}", tx.id.0, tx.data);
            }
            println!();
            received_blocks += 1;
        }
        println!("[Receiver] received {} subblocks total", received_blocks);
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

/// A simplified version of a CL transaction
#[derive(Clone, Debug)]
struct CLTransaction {
    id: TransactionId,
    data: String,
    chain_id: ChainId,
}

/// A simplified version of a subblock with proper types
#[derive(Clone, Debug)]
struct SubBlockV10 {
    chain_id: ChainId,
    #[allow(dead_code)]
    block_id: BlockId,
    transactions: Vec<CLTransaction>,
}

/// A simplified version of a block with proper types
#[derive(Clone, Debug)]
struct BlockV10 {
    #[allow(dead_code)]
    id: BlockId,
    transactions: Vec<CLTransaction>,
}

/// A simplified version of ConfirmationLayerNode's state with proper types
struct TestNodeStateV10 {
    current_block: u64,
    pending_transactions: Vec<CLTransaction>,
    processed_transactions: Vec<CLTransaction>,
    message_receiver: mpsc::Receiver<CLTransaction>,
    subblock_sender: mpsc::Sender<SubBlockV10>,
    blocks: Vec<BlockV10>,
    block_interval: Duration,
    registered_chains: std::collections::HashSet<ChainId>,
}

impl TestNodeStateV10 {
    fn new(
        message_receiver: mpsc::Receiver<CLTransaction>,
        subblock_sender: mpsc::Sender<SubBlockV10>,
        block_interval: Duration,
    ) -> Self {
        Self {
            current_block: 0,
            pending_transactions: Vec::new(),
            processed_transactions: Vec::new(),
            message_receiver,
            subblock_sender,
            blocks: Vec::new(),
            block_interval,
            registered_chains: std::collections::HashSet::new(),
        }
    }

    fn register_chain(&mut self, chain_id: ChainId) -> Result<(), NodeError> {
        if self.registered_chains.contains(&chain_id) {
            return Err(NodeError::ChainAlreadyRegistered(chain_id.0));
        }
        self.registered_chains.insert(chain_id);
        Ok(())
    }

    fn is_chain_registered(&self, chain_id: &ChainId) -> bool {
        self.registered_chains.contains(chain_id)
    }

    fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), NodeError> {
        if !self.is_chain_registered(&transaction.chain_id) {
            return Err(NodeError::ChainNotFound(transaction.chain_id.0));
        }
        self.pending_transactions.push(transaction);
        Ok(())
    }

    fn get_subblock(&self, chain_id: ChainId, block_id: BlockId) -> Result<SubBlockV10, NodeError> {
        if !self.is_chain_registered(&chain_id) {
            return Err(NodeError::ChainNotFound(chain_id.0));
        }

        let block_num = block_id.0.parse::<u64>().map_err(|_| NodeError::BlockNotFound(0))?;
        if block_num >= self.current_block {
            return Err(NodeError::BlockNotFound(block_num));
        }

        // Find transactions for this chain in the block
        let block = &self.blocks[block_num as usize];
        let transactions: Vec<CLTransaction> = block.transactions
            .iter()
            .filter(|tx| tx.chain_id == chain_id)
            .cloned()
            .collect();

        Ok(SubBlockV10 {
            chain_id,
            block_id,
            transactions,
        })
    }
}

/// A function that continuously processes messages and updates state
async fn run_processor_v10(state: Arc<Mutex<TestNodeStateV10>>) {
    println!("[Processor] task started");
    
    // Get the block interval
    let block_interval = {
        let state = state.lock().await;
        state.block_interval
    };
    
    // Create an interval for block production
    let mut interval = interval(block_interval);
    
    loop {
        // Wait for the next block interval
        interval.tick().await;
        
        // Acquire the lock and process messages
        let mut state = state.lock().await;
        
        // Check for new messages from channel
        while let Ok(transaction) = state.message_receiver.try_recv() {
            if state.is_chain_registered(&transaction.chain_id) {
                println!("[Processor] received transaction from chain {}: {}", transaction.chain_id.0, transaction.data);
                state.pending_transactions.push(transaction);
            } else {
                println!("[Processor] ignoring transaction from unregistered chain {}: {}", transaction.chain_id.0, transaction.data);
            }
        }
        
        // Create a new block
        let block_id = BlockId(state.current_block.to_string());
        let mut block = BlockV10 {
            id: block_id.clone(),
            transactions: Vec::new(),
        };
        
        // Group transactions by chain
        let mut chain_transactions: std::collections::HashMap<ChainId, Vec<CLTransaction>> = std::collections::HashMap::new();
        
        // Move pending transactions to the block and group by chain
        while !state.pending_transactions.is_empty() {
            let transaction = state.pending_transactions.remove(0);
            block.transactions.push(transaction.clone());
            state.processed_transactions.push(transaction.clone());
            
            // Group transactions by chain for subblocks
            chain_transactions.entry(transaction.chain_id.clone()).or_default().push(transaction);
        }
        
        // Store the block
        state.blocks.push(block.clone());
        state.current_block += 1;
        
        // Create and send subblocks for each chain
        for (chain_id, transactions) in chain_transactions {
            if !transactions.is_empty() {
                let subblock = SubBlockV10 {
                    chain_id: chain_id.clone(),
                    block_id: block_id.clone(),
                    transactions,
                };
                
                // Send the subblock
                if let Err(e) = state.subblock_sender.send(subblock.clone()).await {
                    println!("[Processor] failed to send subblock for chain {}: {}", chain_id.0, e);
                }
            }
        }
        
        // Print block status
        if !block.transactions.is_empty() {
            print!("[Processor] produced block {} with {} transactions", block_id.0, block.transactions.len());
            for tx in &block.transactions {
                print!("  - id={}, data={}", tx.id.0, tx.data);
            }
            println!();
        } else {
            println!("[Processor] produced empty block {}", block_id.0);
        }
        
        // Release the lock by dropping state
        drop(state);
    }
}

/// A function that gradually adds messages to the state through channel
async fn run_adder_v10(sender: mpsc::Sender<CLTransaction>, chain_id: ChainId) {
    println!("[Adder-{}] task started", chain_id.0);
    for i in 1..=7 {
        // Wait for ~3 blocks (300ms) before adding next message
        sleep(Duration::from_millis(300)).await;
        let transaction = CLTransaction {
            id: TransactionId(format!("tx{}", i)),
            data: format!("message{}.{}", i, chain_id.0),
            chain_id: chain_id.clone(),
        };
        if let Err(e) = sender.send(transaction.clone()).await {
            println!("[Adder-{}] failed to send transaction: {}", chain_id.0, e);
            break;
        }
        println!("[Adder-{}] sent transaction{}", chain_id.0, i);
    }
    println!("[Adder-{}] task completed", chain_id.0);
}
