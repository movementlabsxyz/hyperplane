use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use hyperplane::hyper_scheduler::node::HyperSchedulerNode;
use hyperplane::types::{CATId, CLTransaction, CLTransactionId, constants, cat::CATStatus};
use hyperplane::utils::logging;

/// Tests that the HS node shutdown functionality works correctly:
/// - Verifies state is properly cleared after shutdown
/// - Tests that shutdown completes without errors
#[tokio::test]
async fn test_hs_node_shutdown() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_hs_node_shutdown ===");
    
    logging::log("TEST", "Creating channels for the HS node...");
    // Create channels for the HS node
    let (_sender_to_cl, _receiver_from_hs) = mpsc::channel::<CLTransaction>(100);
    
    logging::log("TEST", "Creating new HS node...");
    // Create a new HS node
    let node = Arc::new(Mutex::new(HyperSchedulerNode::new(_sender_to_cl)));
    
    logging::log("TEST", "Starting the node...");
    // Start the node
    HyperSchedulerNode::start(node.clone()).await;
    
    // Add some state to verify it gets cleared
    {
        logging::log("TEST", "Adding test state to verify it gets cleared...");
        let node_guard = node.lock().await;
        let mut state = node_guard.state.lock().await;
        
        // Add some test data using constants
        state.registered_chains.insert(constants::chain_1());
        state.registered_chains.insert(constants::chain_2());
        
        let cat_id = CATId(CLTransactionId("test_cat_1".to_string()));
        state.constituent_chains.insert(cat_id.clone(), vec![constants::chain_1(), constants::chain_2()]);
        state.cat_statuses.insert(cat_id.clone(), CATStatus::Pending);
        state.cat_chainwise_statuses.insert(cat_id.clone(), std::collections::HashMap::new());
        
        // Verify state exists
        assert!(state.registered_chains.contains(&constants::chain_1()));
        assert!(state.registered_chains.contains(&constants::chain_2()));
        assert!(state.constituent_chains.contains_key(&cat_id));
        assert!(state.cat_statuses.contains_key(&cat_id));
        assert!(state.cat_chainwise_statuses.contains_key(&cat_id));
        logging::log("TEST", "✓ Test state added and verified");
    }
    
    logging::log("TEST", "Shutting down the node...");
    // Shutdown the node
    HyperSchedulerNode::shutdown(node.clone()).await;
    
    // Verify state is cleared
    {
        logging::log("TEST", "Verifying state is cleared...");
        let node_guard = node.lock().await;
        let state = node_guard.state.lock().await;
        
        // Verify all state is cleared
        assert!(state.registered_chains.is_empty());
        assert!(state.constituent_chains.is_empty());
        assert!(state.cat_statuses.is_empty());
        assert!(state.cat_chainwise_statuses.is_empty());
        logging::log("TEST", "✓ All state cleared successfully");
    }
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that multiple shutdown calls are safe and don't cause errors:
/// - Verifies that calling shutdown multiple times doesn't cause issues
/// - Ensures state remains cleared after multiple shutdowns
#[tokio::test]
async fn test_hs_node_shutdown_multiple_times() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_hs_node_shutdown_multiple_times ===");
    
    logging::log("TEST", "Creating channels for the HS node...");
    // Create channels for the HS node
    let (_sender_to_cl, _receiver_from_hs) = mpsc::channel::<CLTransaction>(100);
    
    logging::log("TEST", "Creating new HS node...");
    // Create a new HS node
    let node = Arc::new(Mutex::new(HyperSchedulerNode::new(_sender_to_cl)));
    
    logging::log("TEST", "Starting the node...");
    // Start the node
    HyperSchedulerNode::start(node.clone()).await;
    
    // Add some state
    {
        logging::log("TEST", "Adding test state...");
        let node_guard = node.lock().await;
        let mut state = node_guard.state.lock().await;
        state.registered_chains.insert(constants::chain_1());
        state.registered_chains.insert(constants::chain_2());
        logging::log("TEST", "✓ Test state added");
    }
    
    logging::log("TEST", "Calling shutdown multiple times...");
    // Shutdown multiple times (should be safe)
    HyperSchedulerNode::shutdown(node.clone()).await;
    HyperSchedulerNode::shutdown(node.clone()).await;
    HyperSchedulerNode::shutdown(node.clone()).await;
    logging::log("TEST", "✓ Multiple shutdowns completed without error");
    
    // Verify state is still cleared
    {
        logging::log("TEST", "Verifying state is still cleared...");
        let node_guard = node.lock().await;
        let state = node_guard.state.lock().await;
        assert!(state.registered_chains.is_empty());
        assert!(state.constituent_chains.is_empty());
        assert!(state.cat_statuses.is_empty());
        assert!(state.cat_chainwise_statuses.is_empty());
        logging::log("TEST", "✓ State remains cleared after multiple shutdowns");
    }
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that the HS node can be restarted after shutdown:
/// - Verifies that shutdown clears all state
/// - Tests that node can be restarted and accept new state
/// - Ensures old state doesn't persist between restarts
#[tokio::test]
async fn test_hs_node_restart_after_shutdown() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_hs_node_restart_after_shutdown ===");
    
    logging::log("TEST", "Creating channels for the HS node...");
    // Create channels for the HS node
    let (_sender_to_cl, _receiver_from_hs) = mpsc::channel::<CLTransaction>(100);
    
    logging::log("TEST", "Creating new HS node...");
    // Create a new HS node
    let node = Arc::new(Mutex::new(HyperSchedulerNode::new(_sender_to_cl)));
    
    logging::log("TEST", "Starting the node...");
    // Start the node
    HyperSchedulerNode::start(node.clone()).await;
    
    // Add some initial state
    {
        logging::log("TEST", "Adding initial test state...");
        let node_guard = node.lock().await;
        let mut state = node_guard.state.lock().await;
        state.registered_chains.insert(constants::chain_1());
        
        let cat_id = CATId(CLTransactionId("test_cat_1".to_string()));
        state.constituent_chains.insert(cat_id.clone(), vec![constants::chain_1()]);
        state.cat_statuses.insert(cat_id.clone(), CATStatus::Pending);
        logging::log("TEST", "✓ Initial state added");
    }
    
    logging::log("TEST", "Shutting down the node...");
    // Shutdown the node
    HyperSchedulerNode::shutdown(node.clone()).await;
    
    // Verify state is cleared
    {
        logging::log("TEST", "Verifying state is cleared after shutdown...");
        let node_guard = node.lock().await;
        let state = node_guard.state.lock().await;
        assert!(state.registered_chains.is_empty());
        assert!(state.constituent_chains.is_empty());
        assert!(state.cat_statuses.is_empty());
        logging::log("TEST", "✓ State cleared after shutdown");
    }
    
    logging::log("TEST", "Starting the node again...");
    // Start the node again
    HyperSchedulerNode::start(node.clone()).await;
    
    // Add new state
    {
        logging::log("TEST", "Adding new test state after restart...");
        let node_guard = node.lock().await;
        let mut state = node_guard.state.lock().await;
        state.registered_chains.insert(constants::chain_2());
        
        let cat_id = CATId(CLTransactionId("test_cat_2".to_string()));
        state.constituent_chains.insert(cat_id.clone(), vec![constants::chain_2()]);
        state.cat_statuses.insert(cat_id.clone(), CATStatus::Success);
        logging::log("TEST", "✓ New state added after restart");
    }
    
    // Verify new state exists and old state doesn't persist
    {
        logging::log("TEST", "Verifying new state exists and old state doesn't persist...");
        let node_guard = node.lock().await;
        let state = node_guard.state.lock().await;
        
        // Verify new state exists
        assert!(state.registered_chains.contains(&constants::chain_2()));
        assert!(state.constituent_chains.contains_key(&CATId(CLTransactionId("test_cat_2".to_string()))));
        assert!(state.cat_statuses.contains_key(&CATId(CLTransactionId("test_cat_2".to_string()))));
        logging::log("TEST", "✓ New state exists");
        
        // Verify old state doesn't persist
        assert!(!state.registered_chains.contains(&constants::chain_1()));
        assert!(!state.constituent_chains.contains_key(&CATId(CLTransactionId("test_cat_1".to_string()))));
        assert!(!state.cat_statuses.contains_key(&CATId(CLTransactionId("test_cat_1".to_string()))));
        logging::log("TEST", "✓ Old state doesn't persist");
    }
    
    logging::log("TEST", "Shutting down again...");
    // Shutdown again
    HyperSchedulerNode::shutdown(node.clone()).await;
    
    logging::log("TEST", "=== Test completed successfully ===\n");
} 