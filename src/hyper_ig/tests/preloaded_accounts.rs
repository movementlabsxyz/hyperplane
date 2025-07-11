use crate::{
    types::{Transaction, TransactionId, TransactionStatus, ChainId, CLTransactionId},
    hyper_ig::{HyperIG, node::HyperIGNode},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use hyperplane::utils::logging;

/// Helper function to set up a test HIG node with preloaded accounts
pub async fn setup_test_hig_node_with_preloaded_accounts(num_accounts: u32, preload_value: u32) -> Arc<Mutex<HyperIGNode>> {
    let (_sender_cl_to_hig, receiver_cl_to_hig) = mpsc::channel(100);
    let (sender_hig_to_hs, _receiver_hig_to_hs) = mpsc::channel(100);
    
    let hig_node = HyperIGNode::new_with_preloaded_accounts(
        receiver_cl_to_hig, 
        sender_hig_to_hs, 
        ChainId("test-chain".to_string()), 
        4, 
        true, 
        num_accounts, 
        preload_value
    );
    let hig_node = Arc::new(Mutex::new(hig_node));
    
    // Start the node
    HyperIGNode::start(hig_node.clone()).await;

    hig_node
}

/// Tests that accounts are properly preloaded with the specified value when creating a HyperIG node.
/// 
/// This test verifies the core preloading functionality by ensuring that when a HyperIG node
/// is created with preloaded accounts, all specified accounts are initialized with the correct
/// balance. This is essential for simulation scenarios where we need predictable initial states.
/// 
/// Test flow:
/// 1. Creates a HyperIG node with 5 accounts preloaded with 100 tokens each
/// 2. Retrieves the chain state to verify account balances
/// 3. Verifies each account from 1 to 5 has exactly 100 tokens
/// 4. Confirms no additional accounts exist beyond the specified count
/// 
/// This test ensures that the preloading mechanism works correctly and provides
/// a reliable foundation for simulation testing.
#[tokio::test]
async fn test_preloaded_accounts() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_preloaded_accounts ===");
    
    let num_accounts = 5;
    let preload_value = 100;
    
    logging::log("TEST", &format!("Setting up HyperIG node with {} accounts preloaded with {} tokens each", num_accounts, preload_value));
    let hig_node = setup_test_hig_node_with_preloaded_accounts(num_accounts, preload_value).await;
    logging::log("TEST", "HyperIG node setup complete");
    
    // Get the chain state and verify all accounts are preloaded
    logging::log("TEST", "Retrieving chain state to verify preloaded accounts...");
    let state = hig_node.lock().await.get_chain_state().await.unwrap();
    logging::log("TEST", &format!("Chain state: {:?}", state));
    
    // Verify all accounts from 1 to num_accounts have the preload_value
    logging::log("TEST", "Verifying account balances...");
    for account_id in 1..=num_accounts {
        let account_key = account_id.to_string();
        let balance = state.get(&account_key).copied().unwrap_or(0);
        assert_eq!(balance, preload_value as i64, "Account {} should have balance {}", account_id, preload_value);
        logging::log("TEST", &format!("✓ Account {} has correct balance: {}", account_id, balance));
    }
    
    // Verify no other accounts exist
    assert_eq!(state.len(), num_accounts as usize, "Should have exactly {} accounts", num_accounts);
    logging::log("TEST", &format!("✓ Verified exactly {} accounts exist", num_accounts));
    
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that send transactions work correctly with preloaded accounts.
/// 
/// This test verifies that the preloaded account functionality integrates properly
/// with the transaction processing system. It ensures that transactions can be
/// executed between preloaded accounts and that balances are updated correctly.
/// 
/// Test flow:
/// 1. Creates a HyperIG node with 3 accounts preloaded with 200 tokens each
/// 2. Executes a send transaction from account 1 to account 2 for 50 tokens
/// 3. Verifies the transaction succeeds
/// 4. Checks that account 1 balance is reduced to 150 (200 - 50)
/// 5. Checks that account 2 balance is increased to 250 (200 + 50)
/// 6. Confirms account 3 balance remains unchanged at 200
/// 
/// This test ensures that preloaded accounts can participate in normal transaction
/// processing without any issues.
#[tokio::test]
async fn test_transactions_with_preloaded_accounts() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_transactions_with_preloaded_accounts ===");
    
    let num_accounts = 3;
    let preload_value = 200;
    
    logging::log("TEST", &format!("Setting up HyperIG node with {} accounts preloaded with {} tokens each", num_accounts, preload_value));
    let hig_node = setup_test_hig_node_with_preloaded_accounts(num_accounts, preload_value).await;
    logging::log("TEST", "HyperIG node setup complete");
    
    // Test a send transaction between preloaded accounts
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx = Transaction::new(
        TransactionId(format!("{:?}:send_tx", cl_id)),
        ChainId("test-chain".to_string()),
        vec![ChainId("test-chain".to_string())],
        "REGULAR.send 1 2 50".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    logging::log("TEST", "Executing send transaction from account 1 to account 2 for 50 tokens...");
    let status = hig_node.lock().await.process_transaction(tx).await.unwrap();
    assert_eq!(status, TransactionStatus::Success, "Send transaction should succeed with sufficient funds");
    logging::log("TEST", "✓ Send transaction executed successfully");
    
    // Verify the balances are updated correctly
    logging::log("TEST", "Verifying updated account balances...");
    let state = hig_node.lock().await.get_chain_state().await.unwrap();
    assert_eq!(state.get("1"), Some(&150), "Account 1 should have balance 150 (200 - 50)");
    assert_eq!(state.get("2"), Some(&250), "Account 2 should have balance 250 (200 + 50)");
    assert_eq!(state.get("3"), Some(&200), "Account 3 should still have balance 200");
    
    logging::log("TEST", "✓ All account balances updated correctly");
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that no accounts are preloaded when the preload count is set to zero.
/// 
/// This test verifies the edge case where no preloading is requested, ensuring
/// that the system behaves correctly when preloading is disabled. This is important
/// for scenarios where we want to start with completely empty accounts.
/// 
/// Test flow:
/// 1. Creates a HyperIG node with 0 accounts to preload
/// 2. Retrieves the chain state
/// 3. Verifies that no accounts exist in the state
/// 
/// This test ensures that the preloading system gracefully handles the case
/// where no preloading is desired, maintaining clean state initialization.
#[tokio::test]
async fn test_no_preloaded_accounts() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_no_preloaded_accounts ===");
    
    logging::log("TEST", "Setting up HyperIG node with 0 preloaded accounts");
    let hig_node = setup_test_hig_node_with_preloaded_accounts(0, 100).await;
    logging::log("TEST", "HyperIG node setup complete");
    
    // Get the chain state and verify no accounts are preloaded
    logging::log("TEST", "Retrieving chain state to verify no accounts exist...");
    let state = hig_node.lock().await.get_chain_state().await.unwrap();
    assert!(state.is_empty(), "Should have no preloaded accounts when count is 0");
    
    logging::log("TEST", "✓ Verified no accounts exist in chain state");
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Tests that credit transactions work correctly with preloaded accounts.
/// 
/// This test verifies that credit transactions can be executed on preloaded accounts
/// and that the balances are updated correctly. This is important for scenarios
/// where we need to add additional funds to preloaded accounts during simulation.
/// 
/// Test flow:
/// 1. Creates a HyperIG node with 2 accounts preloaded with 50 tokens each
/// 2. Executes a credit transaction adding 75 tokens to account 1
/// 3. Verifies the transaction succeeds
/// 4. Checks that account 1 balance is increased to 125 (50 + 75)
/// 5. Confirms account 2 balance remains unchanged at 50
/// 
/// This test ensures that preloaded accounts can receive additional credits
/// and that the balance arithmetic works correctly.
#[tokio::test]
async fn test_credit_with_preloaded_accounts() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_credit_with_preloaded_accounts ===");
    
    let num_accounts = 2;
    let preload_value = 50;
    
    logging::log("TEST", &format!("Setting up HyperIG node with {} accounts preloaded with {} tokens each", num_accounts, preload_value));
    let hig_node = setup_test_hig_node_with_preloaded_accounts(num_accounts, preload_value).await;
    logging::log("TEST", "HyperIG node setup complete");
    
    // Test a credit transaction to an existing account
    let cl_id = CLTransactionId("cl-tx".to_string());
    let tx = Transaction::new(
        TransactionId(format!("{:?}:credit_tx", cl_id)),
        ChainId("test-chain".to_string()),
        vec![ChainId("test-chain".to_string())],
        "REGULAR.credit 1 75".to_string(),
        cl_id.clone(),
    ).expect("Failed to create transaction");
    
    logging::log("TEST", "Executing credit transaction adding 75 tokens to account 1...");
    let status = hig_node.lock().await.process_transaction(tx).await.unwrap();
    assert_eq!(status, TransactionStatus::Success, "Credit transaction should succeed");
    logging::log("TEST", "✓ Credit transaction executed successfully");
    
    // Verify the balance is updated correctly
    logging::log("TEST", "Verifying updated account balances...");
    let state = hig_node.lock().await.get_chain_state().await.unwrap();
    assert_eq!(state.get("1"), Some(&125), "Account 1 should have balance 125 (50 + 75)");
    assert_eq!(state.get("2"), Some(&50), "Account 2 should still have balance 50");
    
    logging::log("TEST", "✓ Account balances updated correctly after credit");
    logging::log("TEST", "=== Test completed successfully ===\n");
}

/// Comprehensive test demonstrating a realistic simulation scenario with preloaded accounts.
/// 
/// This test simulates a complex scenario with multiple accounts and transactions
/// to verify that the preloaded accounts functionality works correctly in realistic
/// simulation conditions. It tests various transaction types and verifies that
/// all balance calculations are accurate throughout the simulation.
/// 
/// Test flow:
/// 1. Creates a HyperIG node with 10 accounts preloaded with 1000 tokens each
/// 2. Executes a series of 5 transactions including sends and credits
/// 3. Verifies each transaction succeeds
/// 4. Checks final balances for all accounts against expected values
/// 5. Validates that the simulation maintains consistency throughout
/// 
/// This test ensures that preloaded accounts can handle complex simulation
/// scenarios with multiple transactions and maintain accurate state throughout.
#[tokio::test]
async fn test_simulation_with_preloaded_accounts() {
    logging::init_logging();
    logging::log("TEST", "\n=== Starting test_simulation_with_preloaded_accounts ===");
    
    // Simulate a scenario with 10 accounts, each preloaded with 1000 tokens
    let num_accounts = 10;
    let preload_value = 1000;
    
    logging::log("TEST", &format!("Setting up HyperIG node with {} accounts preloaded with {} tokens each", num_accounts, preload_value));
    let hig_node = setup_test_hig_node_with_preloaded_accounts(num_accounts, preload_value).await;
    logging::log("TEST", "HyperIG node setup complete");
    
    // Verify initial state
    logging::log("TEST", "Verifying initial account state...");
    let initial_state = hig_node.lock().await.get_chain_state().await.unwrap();
    logging::log("TEST", &format!("Initial state: {:?}", initial_state));
    
    // Simulate a series of transactions
    let transactions = vec![
        ("send 1 5 100", "Account 1 sends 100 to account 5"),
        ("send 2 8 200", "Account 2 sends 200 to account 8"),
        ("send 3 10 150", "Account 3 sends 150 to account 10"),
        ("credit 7 500", "Credit 500 to account 7"),
        ("send 4 6 75", "Account 4 sends 75 to account 6"),
    ];
    
    logging::log("TEST", "Executing series of transactions...");
    for (i, (tx_data, description)) in transactions.iter().enumerate() {
        let cl_id = CLTransactionId(format!("cl-tx-{}", i));
        let tx = Transaction::new(
            TransactionId(format!("{:?}:{}", cl_id, i)),
            ChainId("test-chain".to_string()),
            vec![ChainId("test-chain".to_string())],
            format!("REGULAR.{}", tx_data),
            cl_id.clone(),
        ).expect("Failed to create transaction");
        
        logging::log("TEST", &format!("Executing transaction {}: {}", i + 1, description));
        let status = hig_node.lock().await.process_transaction(tx).await.unwrap();
        assert_eq!(status, TransactionStatus::Success, "Transaction {} should succeed", i + 1);
        logging::log("TEST", &format!("✓ Transaction {} completed successfully", i + 1));
    }
    
    // Verify final state
    logging::log("TEST", "Verifying final account balances...");
    let final_state = hig_node.lock().await.get_chain_state().await.unwrap();
    logging::log("TEST", &format!("Final state: {:?}", final_state));
    
    // Verify specific account balances
    let expected_balances = vec![
        (1, 900),   // 1000 - 100
        (2, 800),   // 1000 - 200
        (3, 850),   // 1000 - 150
        (4, 925),   // 1000 - 75 + 500
        (5, 1100),  // 1000 + 100
        (6, 1075),  // 1000 + 75
        (7, 1500),  // 1000 + 500
        (8, 1200),  // 1000 + 200
        (9, 1000),  // unchanged
        (10, 1150), // 1000 + 150
    ];
    
    logging::log("TEST", "Validating final account balances...");
    for (account_id, expected_balance) in expected_balances {
        let actual_balance = final_state.get(&account_id.to_string()).copied().unwrap_or(0);
        assert_eq!(actual_balance, expected_balance as i64, 
            "Account {} should have balance {}", account_id, expected_balance);
        logging::log("TEST", &format!("✓ Account {} has correct balance: {}", account_id, actual_balance));
    }
    
    logging::log("TEST", "✓ Simulation with preloaded accounts completed successfully");
    logging::log("TEST", "=== Test completed successfully ===\n");
} 