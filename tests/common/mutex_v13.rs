use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep, interval};
use tokio::sync::mpsc;
use hyperplane::{
    types::{Transaction, TransactionId, ChainId, CLTransaction, SubBlock, CATStatusUpdate},
    confirmation_layer::{ConfirmationLayerError, ConfirmationLayer},
};
use std::collections::HashSet;
use std::collections::HashMap;


// - - - - - - - - - - - - - - - - - - - - - - - 
// V13: Integrates closer to actual node setup
// - - - - - - - - - - - - - - - - - - - - - - - 

/// V13: Integrates closer to actual node setup
#[tokio::test]
async fn test_mutex_concurrent_access_v13() {
    println!("\n=== Starting test_mutex_concurrent_access_v13 ===");
    
    // Get the test nodes using our new helper function
    let (hs_node, cl_node, _hig_node) = setup_test_nodes(Duration::from_millis(100)).await;
    
    // Test initial state
    println!("[Test] Testing initial state...");
    {
        let cl_node_with_lock = cl_node.lock().await;
        let current_block = cl_node_with_lock.get_current_block().await.unwrap();
        println!("[Test] Initial block number: {}", current_block);
        assert_eq!(current_block, 0, "Initial block should be 0");
    }

    // Register chains first
    println!("[Test] Registering chains...");
    {
        let mut cl_node_with_lock = cl_node.lock().await;
        cl_node_with_lock.register_chain(ChainId("chain1".to_string())).await.expect("Failed to register chain1");
        cl_node_with_lock.register_chain(ChainId("chain2".to_string())).await.expect("Failed to register chain2");
        
        // Try to register chain1 again (should fail)
        match cl_node_with_lock.register_chain(ChainId("chain1".to_string())).await {
            Ok(_) => panic!("Should not be able to register chain1 twice"),
            Err(e) => println!("[Test] Expected error when registering chain1 twice: {}", e),
        }

        // Try to get subblock for unregistered chain
        match cl_node_with_lock.get_subblock(ChainId("chain3".to_string()), 0).await {
            Ok(_) => panic!("Should not be able to get subblock for unregistered chain"),
            Err(e) => println!("[Test] Expected error when getting subblock for unregistered chain: {}", e),
        }
    }

    // Verify chain registration and get subblock for registered chain
    println!("[Test] Verifying chain registration and subblock retrieval...");
    {
        let cl_node_with_lock = cl_node.lock().await;
        // Verify registered chains
        assert_eq!(cl_node_with_lock.registered_chains.len(), 2, "Should have exactly 2 registered chains");
        assert!(cl_node_with_lock.registered_chains.contains(&ChainId("chain1".to_string())), "chain1 should be registered");
        assert!(cl_node_with_lock.registered_chains.contains(&ChainId("chain2".to_string())), "chain2 should be registered");

        // Get subblock for registered chain
        match cl_node_with_lock.get_subblock(ChainId("chain1".to_string()), 0).await {
            Ok(subblock) => {
                println!("[Test] Successfully got subblock for chain1: {:?}", subblock);
                assert_eq!(subblock.chain_id, ChainId("chain1".to_string()), "Subblock should be for chain1");
                assert_eq!(subblock.block_id, 0, "Subblock should be for block 0");
                assert!(subblock.transactions.is_empty(), "Initial subblock should be empty");
            },
            Err(e) => panic!("Failed to get subblock for chain1: {}", e),
        }
    }

    // Submit transactions for different chains
    println!("[Test] Submitting transactions...");
    {
        let mut cl_node_with_lock_2 = cl_node.lock().await;
        
        // Submit a transaction for chain1
        let tx1 = CLTransaction {
            id: TransactionId("tx1".to_string()),
            data: "message1.chain1".to_string(),
            chain_id: ChainId("chain1".to_string()),
        };
        cl_node_with_lock_2.submit_transaction(tx1).await.expect("Failed to submit transaction for chain1");
        
        // Submit a transaction for chain2
        let tx2 = CLTransaction {
            id: TransactionId("tx2".to_string()),
            data: "message1.chain2".to_string(),
            chain_id: ChainId("chain2".to_string()),
        };
        cl_node_with_lock_2.submit_transaction(tx2).await.expect("Failed to submit transaction for chain2");
        
        // Try to submit a transaction for unregistered chain (should fail)
        let tx3 = CLTransaction {
            id: TransactionId("tx3".to_string()),
            data: "message1.chain3".to_string(),
            chain_id: ChainId("chain3".to_string()),
        };
        match cl_node_with_lock_2.submit_transaction(tx3).await {
            Ok(_) => panic!("Should not be able to submit transaction for unregistered chain"),
            Err(e) => println!("[Test] Expected error when submitting transaction for unregistered chain: {}", e),
        }
    }

    // wait for 1 second
    sleep(Duration::from_secs(1)).await;

    // Spawn tasks to add more transactions for different chains
    let sender_for_chain1 = hs_node.get_sender_to_cl();
    let _adder_handle1 = tokio::spawn(async move {
        run_spammer_v12(sender_for_chain1, ChainId("chain1".to_string())).await;
    });

    let sender_for_chain2 = hs_node.get_sender_to_cl();
    let _adder_handle2 = tokio::spawn(async move {
        run_spammer_v12(sender_for_chain2, ChainId("chain2".to_string())).await;
    });

    // Wait for a few seconds to let the processor run
    println!("Main task: waiting for 1 second...");
    sleep(Duration::from_secs(1)).await;
    
    // Check the state
    let cl_node_with_lock_3 = cl_node.lock().await;
    println!("Main task: current block is {}", cl_node_with_lock_3.current_block);
    println!("Main task: processed {} transactions", cl_node_with_lock_3.processed_transactions.len());
    println!("Main task: {} transactions still pending", cl_node_with_lock_3.pending_transactions.len());
    println!("Main task: produced {} blocks", cl_node_with_lock_3.blocks.len());
    println!("Main task: registered chains: {:?}", cl_node_with_lock_3.registered_chains);
    
    // Verify the state has been updated
    assert!(cl_node_with_lock_3.current_block > 0, "Block should have been incremented");
    assert!(!cl_node_with_lock_3.processed_transactions.is_empty(), "Should have processed some transactions");
    assert!(!cl_node_with_lock_3.blocks.is_empty(), "Should have produced some blocks");
    assert_eq!(cl_node_with_lock_3.registered_chains.len(), 2, "Should have exactly 2 registered chains");
    
    // Test getting subblock for registered chain
    match cl_node_with_lock_3.get_subblock(ChainId("chain1".to_string()), 0).await {
        Ok(subblock) => println!("[Test] Successfully got subblock for chain1: {:?}", subblock),
        Err(e) => panic!("Failed to get subblock for chain1: {}", e),
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
    println!("Main task: final check - block is {}, processed {} transactions in {} blocks", 
        current_block, processed_count, block_count);
    
    // Ensure the processor is still running and processing transactions
    // With 100ms interval, we should process ~20 blocks in 2 seconds
    // But only ~7 transactions per chain (one every 3 blocks)
    assert!(current_block > 25, "Block should have been incremented more than 25 times in 3 seconds, did {}", current_block);
    assert!(processed_count > 15, "Should have processed more than 15 transactions in 3 seconds (5 per chain), did {}", processed_count);
    assert!(block_count > 25, "Should have produced more than 25 blocks in 3 seconds, did {}", block_count);
    
    println!("=== Test completed successfully ===\n");
}

/// v13: Node that matches CL node functionality exactly
struct TestConfirmationLayerNode {
    /// Currently registered chains
    pub registered_chains: Vec<ChainId>,
    /// Current block number
    pub current_block: u64,
    /// Block interval
    pub block_interval: Duration,
    /// Pending transactions
    pub pending_transactions: Vec<CLTransaction>,
    /// Processed transactions
    pub processed_transactions: Vec<(ChainId, CLTransaction)>,
    /// Block history
    pub blocks: Vec<u64>,
    /// Block to transactions mapping
    pub block_transactions: HashMap<u64, Vec<(ChainId, CLTransaction)>>,
    /// Receiver for messages from Hyper Scheduler
    pub receiver_hs_to_cl: mpsc::Receiver<CLTransaction>,
    /// Sender for messages to Hyper IG
    pub sender_cl_to_hig: mpsc::Sender<SubBlock>,
    /// Sender for transactions from Hyper Scheduler
    pub sender_hs_to_cl: mpsc::Sender<CLTransaction>,
}

impl TestConfirmationLayerNode {
    fn new(
        receiver_hs_to_cl: mpsc::Receiver<CLTransaction>,
        sender_cl_to_hig: mpsc::Sender<SubBlock>,
        block_interval: Duration,
    ) -> Self {
        let (sender_hs_to_cl, _) = mpsc::channel(100);
        Self {
            registered_chains: Vec::new(),
            current_block: 0,
            block_interval,
            pending_transactions: Vec::new(),
            processed_transactions: Vec::new(),
            blocks: Vec::new(),
            block_transactions: HashMap::new(),
            receiver_hs_to_cl,
            sender_cl_to_hig,
            sender_hs_to_cl,
        }
    }

    async fn register_chain(&mut self, chain_id: ChainId) -> Result<(), ConfirmationLayerError> {
        if self.registered_chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainAlreadyRegistered(chain_id));
        }
        self.registered_chains.push(chain_id);
        Ok(())
    }

    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        if !self.registered_chains.contains(&transaction.chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(transaction.chain_id));
        }
        self.pending_transactions.push(transaction);
        Ok(())
    }

    async fn get_subblock(&self, chain_id: ChainId, block_id: u64) -> Result<SubBlock, ConfirmationLayerError> {
        if !self.registered_chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(chain_id));
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

    async fn get_current_block(&self) -> Result<u64, ConfirmationLayerError> {
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
        run_transaction_processor_v13(cl_node_for_processor).await;
    });

    (hs_node, cl_node, hig_node)
}

/// Helper function to run the processor task
async fn run_transaction_processor_v13(cl_node: Arc<Mutex<TestConfirmationLayerNode>>) {
    let mut interval = interval(cl_node.lock().await.block_interval);
    loop {
        interval.tick().await;
        
        let mut state = cl_node.lock().await;
        
        // Process any new transactions from the channel
        while let Ok(transaction) = state.receiver_hs_to_cl.try_recv() {
            println!("[Processor] received transaction for chain {}: {}", transaction.chain_id.0, transaction.data);
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
        let block_id = state.current_block;
        state.blocks.push(block_id);
        
        // Store transactions for this block
        state.block_transactions.insert(block_id, processed_this_block.clone());
        
        // Add processed transactions
        state.processed_transactions.extend(processed_this_block.clone());
        
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
        
        // Send subblocks for each chain with only this block's transactions
        for chain_id in &state.registered_chains {
            let subblock = SubBlock {
                chain_id: chain_id.clone(),
                block_id,
                transactions: processed_this_block
                    .iter()
                    .filter(|(cid, _)| cid == chain_id)
                    .map(|(_, tx)| Transaction {
                        id: tx.id.clone(),
                        data: tx.data.clone(),
                    })
                    .collect(),
            };
            if let Err(e) = state.sender_cl_to_hig.send(subblock).await {
                println!("Error sending subblock: {}", e);
                break;
            }
        }
    }
}

/// Helper function to run the adder task
async fn run_spammer_v12(sender: mpsc::Sender<CLTransaction>, chain_id: ChainId) {
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
        // wait for 300ms before sending next transaction
        sleep(Duration::from_millis(300)).await;
    }
}

#[tokio::test]
async fn test_confirmation_layer() {
    let mut cl = TestConfirmationLayerNode::new(
        mpsc::channel(100).1,
        mpsc::channel(100).0,
        Duration::from_millis(100),
    );
    let chain_id = ChainId("1".to_string());
    
    // Test registering a chain
    cl.register_chain(chain_id.clone()).await.unwrap();
    
    // Test getting current block
    let current_block = cl.get_current_block().await.unwrap();
    assert_eq!(current_block, 0);
    
    // Test getting subblock
    let subblock = cl.get_subblock(chain_id.clone(), 0).await.unwrap();
    assert_eq!(subblock.chain_id, chain_id);
    assert_eq!(subblock.block_id, 0);
    assert!(subblock.transactions.is_empty());
}

