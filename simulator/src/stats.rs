//! Transaction statistics tracking for the Hyperplane simulator.
//! Tracks transaction counts, TPS, cancellation rates, and block timing during simulations.


use std::time::Instant;
use hyperplane::types::TransactionStatus;

// ------------------------------------------------------------------------------------------------
// Statistics Tracking
// ------------------------------------------------------------------------------------------------

/// Tracks statistics for the simulation including total transactions, cancellations, and timing
pub struct SimulatorStats {
    /// Total number of transactions processed during the simulation
    total_transactions: usize,
    /// Number of transactions that were cancelled or failed
    cancelled_transactions: usize,
    /// When the simulation started, used to calculate total duration
    start_time: Instant,
    /// When the last block was processed, used to calculate block intervals
    last_block_time: Instant,
    /// Vector tracking the number of transactions processed in each block
    transactions_per_block: Vec<usize>,
}

impl SimulatorStats {
    /// Creates a new SimulatorStats instance with initialized counters and start time
    pub fn new() -> Self {
        Self {
            total_transactions: 0,
            cancelled_transactions: 0,
            start_time: Instant::now(),
            last_block_time: Instant::now(),
            transactions_per_block: Vec::new(),
        }
    }

    /// Records a transaction and its status, incrementing appropriate counters
    pub fn record_transaction(&mut self, status: &TransactionStatus) {
        self.total_transactions += 1;
        if matches!(status, TransactionStatus::Failure) {
            self.cancelled_transactions += 1;
        }
    }

    /// Records a block and calculates TPS
    pub fn record_block(&mut self) {
        let now = Instant::now();
        let block_duration = now.duration_since(self.last_block_time);
        let tps = self.total_transactions as f64 / block_duration.as_secs_f64();
        let cancellation_rate = if self.total_transactions > 0 {
            (self.cancelled_transactions as f64 / self.total_transactions as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "SIMULATOR: Block completed in {:.2}s, TPS: {:.2}, Total: {}, Cancelled: {} ({:.1}%)",
            block_duration.as_secs_f64(),
            tps,
            self.total_transactions,
            self.cancelled_transactions,
            cancellation_rate
        );

        self.last_block_time = now;
        self.transactions_per_block.push(self.total_transactions);
    }

    /// Prints the final statistics including totals and rates
    pub fn print_final_stats(&self) {
        let duration = self.start_time.elapsed();
        let tps = self.total_transactions as f64 / duration.as_secs_f64();
        let cancellation_rate = if self.total_transactions > 0 {
            (self.cancelled_transactions as f64 / self.total_transactions as f64) * 100.0
        } else {
            0.0
        };

        println!("SIMULATOR: === Final Statistics ===");
        println!("SIMULATOR: Duration: {:.2}s", duration.as_secs_f64());
        println!("SIMULATOR: Total Transactions: {}", self.total_transactions);
        println!("SIMULATOR: Cancelled Transactions: {}", self.cancelled_transactions);
        println!("SIMULATOR: Cancellation Rate: {:.1}%", cancellation_rate);
        println!("SIMULATOR: Average TPS: {:.2}", tps);
    }
} 