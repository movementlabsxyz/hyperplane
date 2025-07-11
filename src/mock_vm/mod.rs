use std::collections::HashMap;
pub use x_chain_vm::transaction::{Transaction, TxSet1};
pub use x_chain_vm::execution::{Execution, Status};
pub use x_chain_vm::memtrace::MemTrace;
use x_chain_vm::parse_input;

/// A mock virtual machine that executes transactions using x-chain-vm
pub struct MockVM {
    state: HashMap<u32, u32>,
}

impl MockVM {
    /// Creates a new instance of MockVM
    /// 
    /// # Returns
    /// `MockVM` - A new instance with an empty state HashMap
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    /// Execute a transaction and return the execution result
    /// 
    /// # Arguments
    /// * `transaction` - A string containing the transaction command
    /// 
    /// # Returns
    /// `Execution<u32, u32>` containing:
    /// * `change_set`: HashMap<u32, u32> - The changes that should be applied to the state
    /// * `status`: Status - Either Success or Failure
    /// * `memory_trace`: MemTrace<u32, u32> - A trace of all memory operations performed
    pub fn execute_transaction(&mut self, transaction: &str) -> Result<Execution<u32, u32>, anyhow::Error> {
        // Parse the transaction using x-chain-vm's parser
        let tx = parse_input(transaction)
            .map_err(|e| anyhow::anyhow!("Failed to parse transaction: {}", e))?;
        
        // Execute the transaction
        let execution = tx.execute(&self.state);
        
        // Update the state if successful
        if execution.is_success() {
            self.state.extend(execution.change_set.clone());
        }

        Ok(execution)
    }

    /// Get the current state
    /// 
    /// # Returns
    /// `&HashMap<u32, u32>` - A reference to the current state map where:
    /// * Key (u32): Account ID
    /// * Value (u32): Account balance
    pub fn get_state(&self) -> &HashMap<u32, u32> {
        &self.state
    }

    /// Preload an account with a specified balance
    /// 
    /// # Arguments
    /// * `account_id` - The account ID to preload
    /// * `balance` - The balance to set for the account
    pub fn preload_account(&mut self, account_id: u32, balance: u32) {
        self.state.insert(account_id, balance);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test credit transactions
    /// 
    /// This test verifies that:
    /// 1. A credit transaction successfully adds the specified amount to an account
    /// 2. Multiple credit transactions can be executed in sequence
    /// 3. The state is correctly updated after each credit
    #[test]
    fn test_credit_transaction() {
        let mut vm = MockVM::new();
        
        // Credit 100 to account 1
        let execution = vm.execute_transaction("credit 1 100").unwrap();
        assert!(execution.is_success());
        assert_eq!(vm.get_state().get(&1), Some(&100));

        // Credit 50 to account 2
        let execution = vm.execute_transaction("credit 2 50").unwrap();
        assert!(execution.is_success());
        assert_eq!(vm.get_state().get(&2), Some(&50));
    }

    /// Test send transactions
    /// 
    /// This test verifies that:
    /// 1. A send transaction successfully transfers funds between accounts
    /// 2. The sender's balance is correctly decreased
    /// 3. The receiver's balance is correctly increased
    /// 4. A send transaction fails if the sender has insufficient funds
    /// 5. The state remains unchanged after a failed transaction
    #[test]
    fn test_send_transaction() {
        let mut vm = MockVM::new();
        
        // First credit 100 to account 1
        let execution = vm.execute_transaction("credit 1 100").unwrap();
        assert!(execution.is_success());
        
        // Send 50 from account 1 to account 2
        let execution = vm.execute_transaction("send 1 2 50").unwrap();
        assert!(execution.is_success());
        // Verify sender's balance is decreased by 50
        assert_eq!(vm.get_state().get(&1), Some(&50));
        // Verify receiver's balance is increased by 50
        assert_eq!(vm.get_state().get(&2), Some(&50));

        // Try to send 100 from account 1 (which now has 50)
        let execution = vm.execute_transaction("send 1 2 100").unwrap();
        assert!(execution.is_failure());
        // Verify sender's balance is unchanged after failed transaction
        assert_eq!(vm.get_state().get(&1), Some(&50));
    }
} 