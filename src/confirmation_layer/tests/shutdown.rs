use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use hyperplane::confirmation_layer::node::ConfirmationLayerNode;
use hyperplane::types::{CLTransaction, CLTransactionId, Transaction, TransactionId, constants};
use hyperplane::utils::logging;

/// Tests that the CL node shutdown functionality works correctly:
/// - Verifies state is properly cleared after shutdown
/// - Tests that shutdown completes without errors
#[tokio::test]
async fn test_cl_node_shutdown() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cl_node_shutdown ===");
    
    logging::log("TEST", "Creating channels for the CL node...");
    // Create channels for the CL node
    let (_sender_to_cl, receiver_from_hs) = mpsc::channel::<CLTransaction>(100);
    
    logging::log("TEST", "Creating new CL node...");
    // Create a new CL node
    let node = Arc::new(Mutex::new(ConfirmationLayerNode::new(receiver_from_hs)));
    
    logging::log("TEST", "Starting the node...");
    // Start the node
    ConfirmationLayerNode::start(node.clone()).await;
    
    // Add some state to verify it gets cleared
    {
        logging::log("TEST", "Adding test state to verify it gets cleared...");
        let node_guard = node.lock().await;
        let mut state = node_guard.state.lock().await;
        
        // Add some test data using constants
        state.pending_transactions.push(CLTransaction {
            id: CLTransactionId("test_cl_1".to_string()),
            transactions: vec![Transaction::new(
                TransactionId("tx1".to_string()),
                constants::chain_1(),
                vec![constants::chain_1()],
                "REGULAR.credit 1 100".to_string(),
                CLTransactionId("test_cl_1".to_string())
            ).expect("Valid transaction")],
            constituent_chains: vec![constants::chain_1()],
        });
        state.current_block_height = 10;
        state.registered_chains.push(constants::chain_1());
        state.registered_chains.push(constants::chain_2());
        
        // Verify state exists
        assert!(!state.pending_transactions.is_empty());
        assert_eq!(state.current_block_height, 10);
        assert!(state.registered_chains.contains(&constants::chain_1()));
        assert!(state.registered_chains.contains(&constants::chain_2()));
        logging::log("TEST", "✓ Test state added and verified");
    }
    
    logging::log("TEST", "Shutting down the node...");
    // Shutdown the node
    ConfirmationLayerNode::shutdown(node.clone()).await;
    
    // Verify state is cleared
    {
        logging::log("TEST", "Verifying state is cleared...");
        let node_guard = node.lock().await;
        let state = node_guard.state.lock().await;
        
        // Verify all state is cleared
        assert!(state.pending_transactions.is_empty());
        assert_eq!(state.current_block_height, 0);
        assert!(state.registered_chains.is_empty());
        logging::log("TEST", "✓ All state cleared successfully");
    }
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that multiple shutdown calls are safe and don't cause errors:
/// - Verifies that calling shutdown multiple times doesn't cause issues
/// - Ensures state remains cleared after multiple shutdowns
#[tokio::test]
async fn test_cl_node_shutdown_multiple_times() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cl_node_shutdown_multiple_times ===");
    
    logging::log("TEST", "Creating channels for the CL node...");
    // Create channels for the CL node
    let (_sender_to_cl, receiver_from_hs) = mpsc::channel::<CLTransaction>(100);
    
    logging::log("TEST", "Creating new CL node...");
    // Create a new CL node
    let node = Arc::new(Mutex::new(ConfirmationLayerNode::new(receiver_from_hs)));
    
    logging::log("TEST", "Starting the node...");
    // Start the node
    ConfirmationLayerNode::start(node.clone()).await;
    
    // Add some state
    {
        logging::log("TEST", "Adding test state...");
        let node_guard = node.lock().await;
        let mut state = node_guard.state.lock().await;
        state.pending_transactions.push(CLTransaction {
            id: CLTransactionId("test_cl_1".to_string()),
            transactions: vec![Transaction::new(
                TransactionId("tx1".to_string()),
                constants::chain_1(),
                vec![constants::chain_1()],
                "REGULAR.credit 1 100".to_string(),
                CLTransactionId("test_cl_1".to_string())
            ).expect("Valid transaction")],
            constituent_chains: vec![constants::chain_1()],
        });
        state.registered_chains.push(constants::chain_1());
        logging::log("TEST", "✓ Test state added");
    }
    
    logging::log("TEST", "Calling shutdown multiple times...");
    // Shutdown multiple times (should be safe)
    ConfirmationLayerNode::shutdown(node.clone()).await;
    ConfirmationLayerNode::shutdown(node.clone()).await;
    ConfirmationLayerNode::shutdown(node.clone()).await;
    logging::log("TEST", "✓ Multiple shutdowns completed without error");
    
    // Verify state is still cleared
    {
        logging::log("TEST", "Verifying state is still cleared...");
        let node_guard = node.lock().await;
        let state = node_guard.state.lock().await;
        assert!(state.pending_transactions.is_empty());
        assert!(state.registered_chains.is_empty());
        logging::log("TEST", "✓ State remains cleared after multiple shutdowns");
    }
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that the CL node can be restarted after shutdown:
/// - Verifies that shutdown clears all state
/// - Tests that node can be restarted and accept new state
/// - Ensures old state doesn't persist between restarts
#[tokio::test]
async fn test_cl_node_restart_after_shutdown() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_cl_node_restart_after_shutdown ===");
    
    logging::log("TEST", "Creating channels for the CL node...");
    // Create channels for the CL node
    let (_sender_to_cl, receiver_from_hs) = mpsc::channel::<CLTransaction>(100);
    
    logging::log("TEST", "Creating new CL node...");
    // Create a new CL node
    let node = Arc::new(Mutex::new(ConfirmationLayerNode::new(receiver_from_hs)));
    
    logging::log("TEST", "Starting the node...");
    // Start the node
    ConfirmationLayerNode::start(node.clone()).await;
    
    // Add some initial state
    {
        logging::log("TEST", "Adding initial test state...");
        let node_guard = node.lock().await;
        let mut state = node_guard.state.lock().await;
        state.pending_transactions.push(CLTransaction {
            id: CLTransactionId("test_cl_1".to_string()),
            transactions: vec![Transaction::new(
                TransactionId("tx1".to_string()),
                constants::chain_1(),
                vec![constants::chain_1()],
                "REGULAR.credit 1 100".to_string(),
                CLTransactionId("test_cl_1".to_string())
            ).expect("Valid transaction")],
            constituent_chains: vec![constants::chain_1()],
        });
        state.registered_chains.push(constants::chain_1());
        logging::log("TEST", "✓ Initial state added");
    }
    
    logging::log("TEST", "Shutting down the node...");
    // Shutdown the node
    ConfirmationLayerNode::shutdown(node.clone()).await;
    
    // Verify state is cleared
    {
        logging::log("TEST", "Verifying state is cleared after shutdown...");
        let node_guard = node.lock().await;
        let state = node_guard.state.lock().await;
        assert!(state.pending_transactions.is_empty());
        assert!(state.registered_chains.is_empty());
        logging::log("TEST", "✓ State cleared after shutdown");
    }
    
    logging::log("TEST", "Starting the node again...");
    // Start the node again
    ConfirmationLayerNode::start(node.clone()).await;
    
    // Add new state
    {
        logging::log("TEST", "Adding new test state after restart...");
        let node_guard = node.lock().await;
        let mut state = node_guard.state.lock().await;
        state.pending_transactions.push(CLTransaction {
            id: CLTransactionId("test_cl_2".to_string()),
            transactions: vec![Transaction::new(
                TransactionId("tx2".to_string()),
                constants::chain_2(),
                vec![constants::chain_2()],
                "REGULAR.credit 1 100".to_string(),
                CLTransactionId("test_cl_2".to_string())
            ).expect("Valid transaction")],
            constituent_chains: vec![constants::chain_2()],
        });
        state.registered_chains.push(constants::chain_2());
        logging::log("TEST", "✓ New state added after restart");
    }
    
    // Verify new state exists and old state doesn't persist
    {
        logging::log("TEST", "Verifying new state exists and old state doesn't persist...");
        let node_guard = node.lock().await;
        let state = node_guard.state.lock().await;
        
        // Verify new state exists
        assert!(!state.pending_transactions.is_empty());
        let has_test2 = state.pending_transactions.iter().any(|tx| tx.id.0 == "test_cl_2");
        assert!(has_test2);
        assert!(state.registered_chains.contains(&constants::chain_2()));
        logging::log("TEST", "✓ New state exists");
        
        // Verify old state doesn't persist
        let has_test1 = state.pending_transactions.iter().any(|tx| tx.id.0 == "test_cl_1");
        assert!(!has_test1);
        assert!(!state.registered_chains.contains(&constants::chain_1()));
        logging::log("TEST", "✓ Old state doesn't persist");
    }
    
    logging::log("TEST", "Shutting down again...");
    // Shutdown again
    ConfirmationLayerNode::shutdown(node.clone()).await;
    
    logging::log("TEST", "=== Test completed successfully ===\n");
} 