use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep, interval};
use tokio::sync::mpsc;
use hyperplane::utils::logging;

/// A simplified version of ConfirmationLayerNode's state
struct TestNodeState {
    current_block: u64,
    pending_messages: Vec<String>,
    processed_messages: Vec<String>,
}

impl TestNodeState {
    fn new() -> Self {
        Self {
            current_block: 0,
            pending_messages: Vec::new(),
            processed_messages: Vec::new(),
        }
    }
}


// - - - - - - - - - - - - - - - - - - - - - - - 
// V1: Simple counter with incrementer
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V1 (current): Simple counter with incrementer
/// - Basic mutex usage
/// - Single value being updated
/// - Simple sleep-based yielding
#[tokio::test]
async fn test_v1() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_v1 ===");
    
    // Create a shared counter wrapped in Arc<Mutex>
    let counter = Arc::new(Mutex::new(0));
    
    // Clone the counter for the incrementer task
    let counter_for_incrementer = counter.clone();
    
    // Spawn the incrementer task
    let _incrementer_handle = tokio::spawn(async move {
        run_processer_v1(counter_for_incrementer).await;
    });
    
    // Wait for a few seconds to let the incrementer run
    logging::log("TEST", "Main task: waiting for 2 seconds...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the counter value
    let counter_value = *counter.lock().await;
    logging::log("TEST", &format!("Main task: counter is {}", counter_value));
    
    // Verify the counter has been incremented
    assert!(counter_value > 0, "Counter should have been incremented");
    assert!(counter_value <= 11, "Counter should not have incremented more than 11 times in 1 second (leaving a bit of buffer here)");
    
    // The incrementer task will continue running until the test ends

    // wait now for 3 seconds
    sleep(Duration::from_secs(1)).await;
    // make sure the incrementer task is still running, so check again the counter value
    let counter_value = *counter.lock().await;
    // ensure the counter value is still incrementing
    assert!(counter_value > 15, "Counter should have been incremented more than 15 times in 2 seconds");
    logging::log("TEST", &format!("Main task: counter is {}", counter_value));
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// A function that continuously increments a counter
async fn run_processer_v1(counter: Arc<Mutex<i32>>) {
    logging::log("TEST", "Incrementer task started");
    loop {
        // Acquire the lock and increment the counter
        let mut counter = counter.lock().await;
        *counter += 1;
        logging::log("TEST", &format!("Incrementer: counter is now {}", *counter));
        
        // Release the lock by dropping the counter
        drop(counter);
        
        // Sleep for a second
        sleep(Duration::from_millis(100)).await;
    }
}


// - - - - - - - - - - - - - - - - - - - - - - - 
// V2: Adds a more complex state structure (like ConfirmationLayerNode)
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V2: Adds a more complex state structure (like ConfirmationLayerNode)
/// - Uses a struct with multiple fields instead of just a counter
/// - Still keeps the simple incrementer pattern
#[tokio::test]
async fn test_v2() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_v2 ===");
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeState::new()));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v2(state_for_processor).await;
    });

    // Spawn a task to add messages gradually
    let state_for_adder = state.clone();
    let _adder_handle = tokio::spawn(async move {
        run_adder_v2(state_for_adder).await;
    });
    
    // Wait for a few seconds to let the processor run
    logging::log("TEST", "Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    logging::log("TEST", &format!("Main task: current block is {}", state_guard.current_block));
    logging::log("TEST", &format!("Main task: processed {} messages", state_guard.processed_messages.len()));
    logging::log("TEST", &format!("Main task: {} messages still pending", state_guard.pending_messages.len()));
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_messages.is_empty(), "Should have processed some messages");
    
    // Drop the first state lock
    drop(state_guard);
    
    // The processor task will continue running until the test ends
    sleep(Duration::from_secs(1)).await;
    
    // Make sure the processor task is still running by checking the state again
    let state_guard = state.lock().await;
    let current_block = state_guard.current_block;
    let processed_count = state_guard.processed_messages.len();
    logging::log("TEST", &format!("Main task: final check - block is {}, processed {} messages", current_block, processed_count));
    
    // Ensure the processor is still running and processing messages
    // With 100ms sleep, we should process ~20 blocks in 2 seconds
    // But only ~7 messages (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 5, "Should have processed more than 5 messages in 2 seconds");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// A function that continuously processes messages and updates state
async fn run_processor_v2(state: Arc<Mutex<TestNodeState>>) {
    logging::log("TEST", "  [TEST] [Processor] task started");
    loop {
        // Acquire the lock and process messages
        let mut state = state.lock().await;
        
        // Always increment block, even if no messages
        state.current_block += 1;
        logging::log("TEST", &format!("  [Processor] block is now {}", state.current_block));
        
        // Process any pending messages
        if !state.pending_messages.is_empty() {
            let message = state.pending_messages.remove(0);
            state.processed_messages.push(message.clone());
            logging::log("TEST", &format!("  [Processor] block is now {} with message: {}", state.current_block, message));
        } else {
            logging::log("TEST", &format!("  [Processor] block is now {} (no messages)", state.current_block));
        }
        
        // Release the lock by dropping state
        drop(state);
        
        // Sleep for a second
        sleep(Duration::from_millis(100)).await;
    }
}

/// A function that gradually adds messages to the state
async fn run_adder_v2(state: Arc<Mutex<TestNodeState>>) {
    logging::log("TEST", "  [TEST] [Adder] task started");
    for i in 1..=7 {
        // Wait for ~3 blocks (300ms) before adding next message
        sleep(Duration::from_millis(300)).await;
        let mut state = state.lock().await;
        state.pending_messages.push(format!("message{}", i));
        logging::log("TEST", &format!("  [Adder] added message{}", i));
    }
    logging::log("TEST", "  [Adder] task completed");
}

// - - - - - - - - - - - - - - - - - - - - - - - 
// V3: Adds a message processing (like CL node)
// - - - - - - - - - - - - - - - - - - - - - - - 


/// V3: Adds message processing (like CL node)
/// - Adds a channel for receiving messages
/// - Processes messages in the incrementer
/// - Still keeps the simple mutex pattern
#[tokio::test]
async fn test_v3() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_v3 ===");
    
    // Create a channel for messages
    let (sender, receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV3::new(receiver)));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v3(state_for_processor).await;
    });

    // Spawn a task to add messages gradually
    let _adder_handle = tokio::spawn(async move {
        run_adder_v3(sender).await;
    });
    
    // Wait for a few seconds to let the processor run
    logging::log("TEST", "Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    logging::log("TEST", &format!("Main task: current block is {}", state_guard.current_block));
    logging::log("TEST", &format!("Main task: processed {} messages", state_guard.processed_messages.len()));
    logging::log("TEST", &format!("Main task: {} messages still pending", state_guard.pending_messages.len()));
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_messages.is_empty(), "Should have processed some messages");
    
    // Drop the first state lock
    drop(state_guard);
    
    // The processor task will continue running until the test ends
    sleep(Duration::from_secs(1)).await;
    
    // Make sure the processor task is still running by checking the state again
    let state_guard = state.lock().await;
    let current_block = state_guard.current_block;
    let processed_count = state_guard.processed_messages.len();
    logging::log("TEST", &format!("Main task: final check - block is {}, processed {} messages", current_block, processed_count));
    
    // Ensure the processor is still running and processing messages
    // With 100ms sleep, we should process ~20 blocks in 2 seconds
    // But only ~7 messages (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 5, "Should have processed more than 5 messages in 2 seconds");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}


/// A simplified version of ConfirmationLayerNode's state with message channel
struct TestNodeStateV3 {
    current_block: u64,
    pending_messages: Vec<String>,
    processed_messages: Vec<String>,
    message_receiver: mpsc::Receiver<String>,
}

impl TestNodeStateV3 {
    fn new(message_receiver: mpsc::Receiver<String>) -> Self {
        Self {
            current_block: 0,
            pending_messages: Vec::new(),
            processed_messages: Vec::new(),
            message_receiver,
        }
    }
}

/// A function that continuously processes messages and updates state
async fn run_processor_v3(state: Arc<Mutex<TestNodeStateV3>>) {
    logging::log("TEST", "  [TEST] [Processor] task started");
    loop {
        // Acquire the lock and process messages
        let mut state = state.lock().await;
        
        // Check for new messages from channel
        while let Ok(message) = state.message_receiver.try_recv() {
            logging::log("TEST", &format!("  [TEST] [Processor] received message from channel: {}", message));
            state.pending_messages.push(message);
        }
        
        // Always increment block, even if no messages
        state.current_block += 1;
        logging::log("TEST", &format!("  [Processor] block is now {}", state.current_block));
        
        // Process any pending messages
        if !state.pending_messages.is_empty() {
            let message = state.pending_messages.remove(0);
            state.processed_messages.push(message.clone());
            logging::log("TEST", &format!("  [Processor] block is now {} with message: {}", state.current_block, message));
        } else {
            logging::log("TEST", &format!("  [Processor] block is now {} (no messages)", state.current_block));
        }
        
        // Release the lock by dropping state
        drop(state);
        
        // Sleep for a second
        sleep(Duration::from_millis(100)).await;
    }
}

/// A function that gradually adds messages to the state through channel
async fn run_adder_v3(sender: mpsc::Sender<String>) {
    logging::log("TEST", "  [TEST] [Adder] task started");
    for i in 1..=7 {
        // Wait for ~3 blocks (300ms) before adding next message
        sleep(Duration::from_millis(300)).await;
        let message = format!("message{}", i);
        if let Err(e) = sender.send(message.clone()).await {
            logging::log("TEST", &format!("  [TEST] [Adder] failed to send message: {}", e));
            break;
        }
        logging::log("TEST", &format!("  [TEST] [Adder] sent message{}", i));
    }
    logging::log("TEST", "  [TEST] [Adder] task completed");
}


// - - - - - - - - - - - - - - - - - - - - - - - 
// V4: Adds block production (like CL node)
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V4: Adds block production (like CL node)
/// - Adds interval-based block production
/// - Processes messages into blocks
/// - Still keeps the simple mutex pattern
#[tokio::test]
async fn test_v4() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_v4 ===");
    
    // Create a channel for messages
    let (sender, receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV4::new(
        receiver,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v4(state_for_processor).await;
    });

    // Spawn a task to add messages gradually
    let _adder_handle = tokio::spawn(async move {
        run_adder_v4(sender).await;
    });
    
    // Wait for a few seconds to let the processor run
    logging::log("TEST", "Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    logging::log("TEST", &format!("Main task: current block is {}", state_guard.current_block));
    logging::log("TEST", &format!("Main task: processed {} messages", state_guard.processed_messages.len()));
    logging::log("TEST", &format!("Main task: {} messages still pending", state_guard.pending_messages.len()));
    logging::log("TEST", &format!("Main task: produced {} blocks", state_guard.blocks.len()));
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_messages.is_empty(), "Should have processed some messages");
    assert!(!state_guard.blocks.is_empty(), "Should have produced some blocks");
    
    // Drop the first state lock
    drop(state_guard);
    
    // The processor task will continue running until the test ends
    sleep(Duration::from_secs(1)).await;
    
    // Make sure the processor task is still running by checking the state again
    let state_guard = state.lock().await;
    let current_block = state_guard.current_block;
    let processed_count = state_guard.processed_messages.len();
    let block_count = state_guard.blocks.len();
    logging::log("TEST", &format!("Main task: final check - block is {}, processed {} messages in {} blocks", 
        current_block, processed_count, block_count));
    
    // Ensure the processor is still running and processing messages
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 messages (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 5, "Should have processed more than 5 messages in 2 seconds");
    assert!(block_count > 15, "Should have produced more than 15 blocks in 2 seconds");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}


/// A simplified version of a block
#[derive(Clone, Debug)]
struct Block {
    #[allow(dead_code)]
    id: u64,
    messages: Vec<String>,
}

/// A simplified version of ConfirmationLayerNode's state with message channel and block production
struct TestNodeStateV4 {
    current_block: u64,
    pending_messages: Vec<String>,
    processed_messages: Vec<String>,
    message_receiver: mpsc::Receiver<String>,
    blocks: Vec<Block>,
    block_interval: Duration,
}

impl TestNodeStateV4 {
    fn new(message_receiver: mpsc::Receiver<String>, block_interval: Duration) -> Self {
        Self {
            current_block: 0,
            pending_messages: Vec::new(),
            processed_messages: Vec::new(),
            message_receiver,
            blocks: Vec::new(),
            block_interval,
        }
    }
}

/// A function that continuously processes messages and updates state
async fn run_processor_v4(state: Arc<Mutex<TestNodeStateV4>>) {
    logging::log("TEST", "  [TEST] [Processor] task started");
    
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
        while let Ok(message) = state.message_receiver.try_recv() {
            logging::log("TEST", &format!("  [TEST] [Processor] received message from channel: {}", message));
            state.pending_messages.push(message);
        }
        
        // Create a new block
        let block_id = state.current_block;
        let mut block = Block {
            id: block_id,
            messages: Vec::new(),
        };
        
        // Move pending messages to the block
        while !state.pending_messages.is_empty() {
            let message = state.pending_messages.remove(0);
            block.messages.push(message.clone());
            state.processed_messages.push(message);
        }
        
        // Store the block
        state.blocks.push(block.clone());
        state.current_block += 1;
        
        // Print block status
        if !block.messages.is_empty() {
            let mut block_status = format!("  [Processor] produced block {} with {} messages", block_id, block.messages.len());
            for msg in &block.messages {
                block_status.push_str(&format!("\n  - \"{}\"", msg));
            }
            logging::log("TEST", &block_status);
        } else {
            logging::log("TEST", &format!("  [Processor] produced empty block {}", block_id));
        }
        
        // Release the lock by dropping state
        drop(state);
    }
}

/// A function that gradually adds messages to the state through channel
async fn run_adder_v4(sender: mpsc::Sender<String>) {
    logging::log("TEST", "  [Adder] task started");
    for i in 1..=7 {
        // Wait for ~3 blocks (300ms) before adding next message
        sleep(Duration::from_millis(300)).await;
        let message = format!("message{}", i);
        if let Err(e) = sender.send(message.clone()).await {
            logging::log("TEST", &format!("  [Adder] failed to send message: {}", e));
            break;
        }
        logging::log("TEST", &format!("  [Adder] sent message{}", i));
    }
    logging::log("TEST", "  [Adder] task completed");
}

// - - - - - - - - - - - - - - - - - - - - - - - 
// V5: Adds multiple chains (like CL node)
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V5: Adds multiple chains (like CL node)
/// - Adds support for multiple chains
/// - Processes messages for different chains
/// - Still keeps the simple mutex pattern
#[tokio::test]
async fn test_v5() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_v5 ===");
    
    // Create a channel for messages
    let (sender, receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV5::new(
        receiver,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v5(state_for_processor).await;
    });

    // Spawn tasks to add messages for different chains
    let sender_for_chain1 = sender.clone();
    let _adder_handle1 = tokio::spawn(async move {
        run_adder_v5(sender_for_chain1, "chain1").await;
    });

    let sender_for_chain2 = sender.clone();
    let _adder_handle2 = tokio::spawn(async move {
        run_adder_v5(sender_for_chain2, "chain2").await;
    });
    
    // Wait for a few seconds to let the processor run
    logging::log("TEST", "Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    logging::log("TEST", &format!("Main task: current block is {}", state_guard.current_block));
    logging::log("TEST", &format!("Main task: processed {} messages", state_guard.processed_messages.len()));
    logging::log("TEST", &format!("Main task: {} messages still pending", state_guard.pending_messages.len()));
    logging::log("TEST", &format!("Main task: produced {} blocks", state_guard.blocks.len()));
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_messages.is_empty(), "Should have processed some messages");
    assert!(!state_guard.blocks.is_empty(), "Should have produced some blocks");
    
    // Drop the first state lock
    drop(state_guard);
    
    // The processor task will continue running until the test ends
    sleep(Duration::from_secs(1)).await;
    
    // Make sure the processor task is still running by checking the state again
    let state_guard = state.lock().await;
    let current_block = state_guard.current_block;
    let processed_count = state_guard.processed_messages.len();
    let block_count = state_guard.blocks.len();
    logging::log("TEST", &format!("Main task: final check - block is {}, processed {} messages in {} blocks", 
        current_block, processed_count, block_count));
    
    // Ensure the processor is still running and processing messages
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 messages per chain (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 10, "Should have processed more than 10 messages in 2 seconds (5 per chain)");
    assert!(block_count > 15, "Should have produced more than 15 blocks in 2 seconds");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// A simplified version of ConfirmationLayerNode's state with message channel and block production
struct TestNodeStateV5 {
    current_block: u64,
    pending_messages: Vec<(String, String)>, // (chain_id, message)
    processed_messages: Vec<(String, String)>, // (chain_id, message)
    message_receiver: mpsc::Receiver<(String, String)>, // (chain_id, message)
    blocks: Vec<Block>,
    block_interval: Duration,
}

impl TestNodeStateV5 {
    fn new(message_receiver: mpsc::Receiver<(String, String)>, block_interval: Duration) -> Self {
        Self {
            current_block: 0,
            pending_messages: Vec::new(),
            processed_messages: Vec::new(),
            message_receiver,
            blocks: Vec::new(),
            block_interval,
        }
    }
}

/// A function that continuously processes messages and updates state
async fn run_processor_v5(state: Arc<Mutex<TestNodeStateV5>>) {
    logging::log("TEST", "  [TEST] [Processor] task started");
    
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
            logging::log("TEST", &format!("  [TEST] [Processor] received message from chain {}: {}", chain_id, message));
            state.pending_messages.push((chain_id, message));
        }
        
        // Create a new block
        let block_id = state.current_block;
        let mut block = Block {
            id: block_id,
            messages: Vec::new(),
        };
        
        // Move pending messages to the block
        while !state.pending_messages.is_empty() {
            let (chain_id, message) = state.pending_messages.remove(0);
            let formatted_message = format!("  [{}] {}", chain_id, message);
            block.messages.push(formatted_message.clone());
            state.processed_messages.push((chain_id.clone(), message.clone()));
        }
        
        // Store the block
        state.blocks.push(block.clone());
        state.current_block += 1;
        
        // Print block status
        if !block.messages.is_empty() {
            let mut block_status = format!("  [Processor] produced block {} with {} messages", block_id, block.messages.len());
            for msg in &block.messages {
                block_status.push_str(&format!("\n  - \"{}\"", msg));
            }
            logging::log("TEST", &block_status);
        } else {
            logging::log("TEST", &format!("  [Processor] produced empty block {}", block_id));
        }
        
        // Release the lock by dropping state
        drop(state);
    }
}

/// A function that gradually adds messages to the state through channel
async fn run_adder_v5(sender: mpsc::Sender<(String, String)>, chain_id: &str) {
    logging::log("TEST", &format!("  [Adder-{}] task started", chain_id));
    for i in 1..=7 {
        // Wait for ~3 blocks (300ms) before adding next message
        sleep(Duration::from_millis(300)).await;
        let message = format!("message{}", i);
        if let Err(e) = sender.send((chain_id.to_string(), message.clone())).await {
            logging::log("TEST", &format!("  [Adder-{}] failed to send message: {}", chain_id, e));
            break;
        }
        logging::log("TEST", &format!("  [Adder-{}] sent message{}", chain_id, i));
    }
    logging::log("TEST", &format!("  [Adder-{}] task completed", chain_id));
}

// - - - - - - - - - - - - - - - - - - - - - - - 
// V6: Adds subblock creation (like CL node)
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V6: Adds subblock creation (like CL node)
/// - Creates subblocks from messages
/// - Sends subblocks to a receiver
/// - Still keeps the simple mutex pattern
#[tokio::test]
async fn test_v6() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_v6 ===");
    
    // Create channels for messages and subblocks
    let (msg_sender, msg_receiver) = mpsc::channel(100);
    let (subblock_sender, mut subblock_receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV6::new(
        msg_receiver,
        subblock_sender,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v6(state_for_processor).await;
    });

    // Spawn tasks to add messages for different chains
    let sender_for_chain1 = msg_sender.clone();
    let _adder_handle1 = tokio::spawn(async move {
        run_adder_v6(sender_for_chain1, "chain1").await;
    });

    let sender_for_chain2 = msg_sender.clone();
    let _adder_handle2 = tokio::spawn(async move {
        run_adder_v6(sender_for_chain2, "chain2").await;
    });

    // Spawn a task to receive and verify subblocks
    let _receiver_handle = tokio::spawn(async move {
        let received_blocks = 0;
        while let Some(subblock) = subblock_receiver.recv().await {
            logging::log("TEST", &format!("  [Receiver] received subblock for chain {} with {} messages", 
                subblock.chain_id, subblock.messages.len()));
            for msg in &subblock.messages {
                logging::log("TEST", &format!("  - \"{}\"", msg));
            }
        }
        logging::log("TEST", &format!("  [Receiver] received {} subblocks total", received_blocks));
    });
    
    // Wait for a few seconds to let the processor run
    logging::log("TEST", "Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    logging::log("TEST", &format!("Main task: current block is {}", state_guard.current_block));
    logging::log("TEST", &format!("Main task: processed {} messages", state_guard.processed_messages.len()));
    logging::log("TEST", &format!("Main task: {} messages still pending", state_guard.pending_messages.len()));
    logging::log("TEST", &format!("Main task: produced {} blocks", state_guard.blocks.len()));
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_messages.is_empty(), "Should have processed some messages");
    assert!(!state_guard.blocks.is_empty(), "Should have produced some blocks");
    
    // Drop the first state lock
    drop(state_guard);
    
    // The processor task will continue running until the test ends
    sleep(Duration::from_secs(1)).await;
    
    // Make sure the processor task is still running by checking the state again
    let state_guard = state.lock().await;
    let current_block = state_guard.current_block;
    let processed_count = state_guard.processed_messages.len();
    let block_count = state_guard.blocks.len();
    logging::log("TEST", &format!("Main task: final check - block is {}, processed {} messages in {} blocks", 
        current_block, processed_count, block_count));
    
    // Ensure the processor is still running and processing messages
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 messages per chain (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 10, "Should have processed more than 10 messages in 2 seconds (5 per chain)");
    assert!(block_count > 15, "Should have produced more than 15 blocks in 2 seconds");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// A simplified version of a subblock
#[derive(Clone, Debug)]
struct SubBlock {
    chain_id: String,
    #[allow(dead_code)]
    block_id: u64,
    messages: Vec<String>,
}

/// A simplified version of ConfirmationLayerNode's state with message channel and block production
struct TestNodeStateV6 {
    current_block: u64,
    pending_messages: Vec<(String, String)>, // (chain_id, message)
    processed_messages: Vec<(String, String)>, // (chain_id, message)
    message_receiver: mpsc::Receiver<(String, String)>, // (chain_id, message)
    subblock_sender: mpsc::Sender<SubBlock>,
    blocks: Vec<Block>,
    block_interval: Duration,
}

impl TestNodeStateV6 {
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
        }
    }
}

/// A function that continuously processes messages and updates state
async fn run_processor_v6(state: Arc<Mutex<TestNodeStateV6>>) {
    logging::log("TEST", "  [TEST] [Processor] task started");
    
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
            logging::log("TEST", &format!("  [TEST] [Processor] received message from chain {}: {}", chain_id, message));
            state.pending_messages.push((chain_id, message));
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
            let formatted_message = format!("  [{}] {}", chain_id, message);
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
                    logging::log("TEST", &format!("  [Processor] failed to send subblock for chain {}: {}", chain_id, e));
                }
            }
        }
        
        // Print block status
        if !block.messages.is_empty() {
            let mut block_status = format!("  [Processor] produced block {} with {} messages", block_id, block.messages.len());
            for msg in &block.messages {
                block_status.push_str(&format!("\n  - \"{}\"", msg));
            }
            logging::log("TEST", &block_status);
        } else {
            logging::log("TEST", &format!("  [Processor] produced empty block {}", block_id));
        }
        
        // Release the lock by dropping state
        drop(state);
    }
}

/// A function that gradually adds messages to the state through channel
async fn run_adder_v6(sender: mpsc::Sender<(String, String)>, chain_id: &str) {
    logging::log("TEST", &format!("  [Adder-{}] task started", chain_id));
    for i in 1..=7 {
        // Wait for ~3 blocks (300ms) before adding next message
        sleep(Duration::from_millis(300)).await;
        let message = format!("message{}", i);
        if let Err(e) = sender.send((chain_id.to_string(), message.clone())).await {
            logging::log("TEST", &format!("  [Adder-{}] failed to send message: {}", chain_id, e));
            break;
        }
        logging::log("TEST", &format!("  [Adder-{}] sent message{}", chain_id, i));
    }
    logging::log("TEST", &format!("  [Adder-{}] task completed", chain_id));
}

// - - - - - - - - - - - - - - - - - - - - - - - 
// V7: Adds chain registration (like CL node)
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V7: Adds chain registration (like CL node)
/// - Adds proper chain registration
/// - Tracks registered chains
/// - Validates chain existence
/// - Still keeps the simple mutex pattern
#[tokio::test]
async fn test_v7() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_v7 ===");
    
    // Create channels for messages and subblocks
    let (msg_sender, msg_receiver) = mpsc::channel(100);
    let (subblock_sender, mut subblock_receiver) = mpsc::channel(100);
    
    // Create a shared state wrapped in Arc<Mutex>
    let state = Arc::new(Mutex::new(TestNodeStateV7::new(
        msg_receiver,
        subblock_sender,
        Duration::from_millis(100), // 100ms block interval
    )));
    
    // Clone the state for the processor task
    let state_for_processor = state.clone();
    
    // Spawn the processor task
    let _processor_handle = tokio::spawn(async move {
        run_processor_v7(state_for_processor).await;
    });

    // Register chains first
    logging::log("TEST", "[TEST]   Registering chains...");
    {
        let mut state = state.lock().await;
        state.register_chain("chain-1").expect("Failed to register chain-1");
        state.register_chain("chain-2").expect("Failed to register chain-2");
        
        // Try to register chain1 again (should fail)
        match state.register_chain("chain-1") {
            Ok(_) => panic!("Should not be able to register chain-1 twice"),
            Err(e) => logging::log("TEST", &format!("[TEST]   Expected error when registering chain-1 twice: '{}'", e)),
        }
    }

    // Spawn tasks to add messages for different chains
    let sender_for_chain1 = msg_sender.clone();
    let _adder_handle1 = tokio::spawn(async move {
        run_adder_v7(sender_for_chain1, "chain-1").await;
    });

    let sender_for_chain2 = msg_sender.clone();
    let _adder_handle2 = tokio::spawn(async move {
        run_adder_v7(sender_for_chain2, "chain-2").await;
    });

    // Try to add messages for an unregistered chain
    let sender_for_chain3 = msg_sender.clone();
    let _adder_handle3 = tokio::spawn(async move {
        run_adder_v7(sender_for_chain3, "chain-3").await;
    });

    // Spawn a task to receive and verify subblocks
    let _receiver_handle = tokio::spawn(async move {
        let mut received_blocks = 0;
        while let Some(subblock) = subblock_receiver.recv().await {
            logging::log("TEST", &format!("  [Receiver] received subblock for chain {} with {} messages", 
                subblock.chain_id, subblock.messages.len()));
            for msg in &subblock.messages {
                logging::log("TEST", &format!("  - \"{}\"", msg));
            }
            received_blocks += 1;
        }
        logging::log("TEST", &format!("  [Receiver] received {} subblocks total", received_blocks));
    });
    
    // Wait for a few seconds to let the processor run
    logging::log("TEST", "Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let state_guard = state.lock().await;
    logging::log("TEST", &format!("Main task: current block is {}", state_guard.current_block));
    logging::log("TEST", &format!("Main task: processed {} messages", state_guard.processed_messages.len()));
    logging::log("TEST", &format!("Main task: {} messages still pending", state_guard.pending_messages.len()));
    logging::log("TEST", &format!("Main task: produced {} blocks", state_guard.blocks.len()));
    logging::log("TEST", &format!("Main task: registered chains: {:?}", state_guard.registered_chains));
    
    // Verify the state has been updated
    assert!(state_guard.current_block > 0, "Block should have been incremented");
    assert!(!state_guard.processed_messages.is_empty(), "Should have processed some messages");
    assert!(!state_guard.blocks.is_empty(), "Should have produced some blocks");
    assert_eq!(state_guard.registered_chains.len(), 2, "Should have exactly 2 registered chains");
    
    // Drop the first state lock
    drop(state_guard);
    
    // The processor task will continue running until the test ends
    sleep(Duration::from_secs(1)).await;
    
    // Make sure the processor task is still running by checking the state again
    let state_guard = state.lock().await;
    let current_block = state_guard.current_block;
    let processed_count = state_guard.processed_messages.len();
    let block_count = state_guard.blocks.len();
    logging::log("TEST", &format!("Main task: final check - block is {}, processed {} messages in {} blocks", 
        current_block, processed_count, block_count));
    
    // Ensure the processor is still running and processing messages
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 messages per chain (one every 3 blocks)
    assert!(current_block > 15, "Block should have been incremented more than 15 times in 2 seconds");
    assert!(processed_count > 10, "Should have processed more than 10 messages in 2 seconds (5 per chain)");
    assert!(block_count > 15, "Should have produced more than 15 blocks in 2 seconds");
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// A simplified version of ConfirmationLayerNode's state with message channel and block production
struct TestNodeStateV7 {
    current_block: u64,
    pending_messages: Vec<(String, String)>, // (chain_id, message)
    processed_messages: Vec<(String, String)>, // (chain_id, message)
    message_receiver: mpsc::Receiver<(String, String)>, // (chain_id, message)
    subblock_sender: mpsc::Sender<SubBlock>,
    blocks: Vec<Block>,
    block_interval: Duration,
    registered_chains: std::collections::HashSet<String>,
}

impl TestNodeStateV7 {
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

    fn register_chain(&mut self, chain_id: &str) -> Result<(), String> {
        if self.registered_chains.contains(chain_id) {
            return Err(format!("Chain {} is already registered", chain_id));
        }
        self.registered_chains.insert(chain_id.to_string());
        Ok(())
    }

    fn is_chain_registered(&self, chain_id: &str) -> bool {
        self.registered_chains.contains(chain_id)
    }
}

/// A function that continuously processes messages and updates state
async fn run_processor_v7(state: Arc<Mutex<TestNodeStateV7>>) {
    logging::log("TEST", "  [TEST] [Processor] task started");
    
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
                logging::log("TEST", &format!("  [TEST] [Processor] received message from chain {}: {}", chain_id, message));
                state.pending_messages.push((chain_id, message));
            } else {
                logging::log("TEST", &format!("  [TEST] [Processor] ignoring message from unregistered chain {}: {}", chain_id, message));
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
            let formatted_message = format!("  [{}] {}", chain_id, message);
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
                    logging::log("TEST", &format!("  [Processor] failed to send subblock for chain {}: {}", chain_id, e));
                }
            }
        }
        
        // Print block status
        if !block.messages.is_empty() {
            let mut block_status = format!("  [Processor] produced block {} with {} messages", block_id, block.messages.len());
            for msg in &block.messages {
                block_status.push_str(&format!("\n  - \"{}\"", msg));
            }
            logging::log("TEST", &block_status);
        } else {
            logging::log("TEST", &format!("  [Processor] produced empty block {}", block_id));
        }
        
        // Release the lock by dropping state
        drop(state);
    }
}

/// A function that gradually adds messages to the state through channel
async fn run_adder_v7(sender: mpsc::Sender<(String, String)>, chain_id: &str) {
    logging::log("TEST", &format!("  [Adder-{}] task started", chain_id));
    for i in 1..=7 {
        // Wait for ~3 blocks (300ms) before adding next message
        sleep(Duration::from_millis(300)).await;
        let message = format!("message{}", i);
        if let Err(e) = sender.send((chain_id.to_string(), message.clone())).await {
            logging::log("TEST", &format!("  [Adder-{}] failed to send message: {}", chain_id, e));
            break;
        }
        logging::log("TEST", &format!("  [Adder-{}] sent message{}", chain_id, i));
    }
    logging::log("TEST", &format!("  [Adder-{}] task completed", chain_id));
}
