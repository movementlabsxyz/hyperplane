use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use hyperplane::hyper_ig::node::HyperIGNode;
use hyperplane::types::{constants, cat::CATStatusUpdate, SubBlock};
use hyperplane::utils::logging;

/// Tests that the HIG node shutdown functionality works correctly:
/// - Verifies that shutdown completes without errors
/// - Tests that background tasks are properly stopped
#[tokio::test]
async fn test_hig_node_shutdown() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_hig_node_shutdown ===");
    
    logging::log("TEST", "Creating channels for the HIG node...");
    // Create channels for the HIG node
    let (_sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel::<SubBlock>(100);
    let (sender_hig_to_hs, _receiver_hig_to_hs) = mpsc::channel::<CATStatusUpdate>(100);
    
    logging::log("TEST", "Creating new HIG node...");
    // Create a new HIG node
    let node = Arc::new(Mutex::new(HyperIGNode::new(
        receiver_cl_to_hig,
        sender_hig_to_hs,
        constants::chain_1(),
        10, // cat_lifetime
        true, // allow_cat_pending_dependencies
    )));
    
    logging::log("TEST", "Starting the node...");
    // Start the node
    HyperIGNode::start(node.clone()).await;
    
    logging::log("TEST", "Shutting down the node...");
    // Shutdown the node
    HyperIGNode::shutdown(node.clone()).await;
    
    logging::log("TEST", "✓ Shutdown completed without error");
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that multiple shutdown calls are safe and don't cause errors:
/// - Verifies that calling shutdown multiple times doesn't cause issues
/// - Ensures background tasks are properly managed
#[tokio::test]
async fn test_hig_node_shutdown_multiple_times() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_hig_node_shutdown_multiple_times ===");
    
    logging::log("TEST", "Creating channels for the HIG node...");
    // Create channels for the HIG node
    let (_sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel::<SubBlock>(100);
    let (sender_hig_to_hs, _receiver_hig_to_hs) = mpsc::channel::<CATStatusUpdate>(100);
    
    logging::log("TEST", "Creating new HIG node...");
    // Create a new HIG node
    let node = Arc::new(Mutex::new(HyperIGNode::new(
        receiver_cl_to_hig,
        sender_hig_to_hs,
        constants::chain_1(),
        10, // cat_lifetime
        true, // allow_cat_pending_dependencies
    )));
    
    logging::log("TEST", "Starting the node...");
    // Start the node
    HyperIGNode::start(node.clone()).await;
    
    logging::log("TEST", "Calling shutdown multiple times...");
    // Shutdown multiple times (should be safe)
    HyperIGNode::shutdown(node.clone()).await;
    HyperIGNode::shutdown(node.clone()).await;
    HyperIGNode::shutdown(node.clone()).await;
    
    logging::log("TEST", "✓ Multiple shutdowns completed without error");
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that the HIG node can be restarted after shutdown:
/// - Verifies that shutdown completes without errors
/// - Tests that node can be restarted successfully
/// - Ensures background tasks are properly managed across restarts
#[tokio::test]
async fn test_hig_node_restart_after_shutdown() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_hig_node_restart_after_shutdown ===");
    
    logging::log("TEST", "Creating channels for the HIG node...");
    // Create channels for the HIG node
    let (_sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel::<SubBlock>(100);
    let (sender_hig_to_hs, _receiver_hig_to_hs) = mpsc::channel::<CATStatusUpdate>(100);
    
    logging::log("TEST", "Creating new HIG node...");
    // Create a new HIG node
    let node = Arc::new(Mutex::new(HyperIGNode::new(
        receiver_cl_to_hig,
        sender_hig_to_hs,
        constants::chain_1(),
        10, // cat_lifetime
        true, // allow_cat_pending_dependencies
    )));
    
    logging::log("TEST", "Starting the node...");
    // Start the node
    HyperIGNode::start(node.clone()).await;
    
    logging::log("TEST", "Shutting down the node...");
    // Shutdown the node
    HyperIGNode::shutdown(node.clone()).await;
    
    logging::log("TEST", "Starting the node again...");
    // Start the node again
    HyperIGNode::start(node.clone()).await;
    
    logging::log("TEST", "Shutting down again...");
    // Shutdown again
    HyperIGNode::shutdown(node.clone()).await;
    
    logging::log("TEST", "✓ Restart after shutdown completed successfully");
    logging::log("TEST", "=== Test completed successfully ===\n");
} 