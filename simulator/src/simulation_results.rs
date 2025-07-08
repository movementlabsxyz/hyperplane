//! Simulation results storage and serialization.
//! 
//! Handles saving simulation data to JSON files for analysis.

use std::time::Instant;
use std::fs;
use serde_json;
use crate::account_selection::AccountSelectionStats;
use hyperplane::utils::logging;

// ------------------------------------------------------------------------------------------------
// Data Structures
// ------------------------------------------------------------------------------------------------

/// Stores all simulation results and statistics
#[derive(Debug)]
pub struct SimulationResults {
    // Transaction counts
    pub transactions_sent: u64,
    pub cat_transactions: u64,
    pub regular_transactions: u64,
    
    // Simulation parameters
    pub initial_balance: u64,
    pub num_accounts: usize,
    pub target_tps: u64,
    pub sim_total_block_number: u64,  // Total number of blocks to simulate
    pub zipf_parameter: f64,
    pub ratio_cats: f64,
    pub block_interval: f64,
    pub cat_lifetime: u64,
    pub initialization_wait_blocks: u64,
    pub chain_delays: Vec<u64>,  // Chain delays in blocks
    
    // Chain data - Combined totals (for backward compatibility)
    pub chain_1_pending: Vec<(u64, u64)>,
    pub chain_2_pending: Vec<(u64, u64)>,
    pub chain_1_success: Vec<(u64, u64)>,
    pub chain_2_success: Vec<(u64, u64)>,
    pub chain_1_failure: Vec<(u64, u64)>,
    pub chain_2_failure: Vec<(u64, u64)>,
    
    // Chain data - CAT transactions
    pub chain_1_cat_pending: Vec<(u64, u64)>,
    pub chain_2_cat_pending: Vec<(u64, u64)>,
    pub chain_1_cat_success: Vec<(u64, u64)>,
    pub chain_2_cat_success: Vec<(u64, u64)>,
    pub chain_1_cat_failure: Vec<(u64, u64)>,
    pub chain_2_cat_failure: Vec<(u64, u64)>,
    
    // Chain data - Regular transactions
    pub chain_1_regular_pending: Vec<(u64, u64)>,
    pub chain_2_regular_pending: Vec<(u64, u64)>,
    pub chain_1_regular_success: Vec<(u64, u64)>,
    pub chain_2_regular_success: Vec<(u64, u64)>,
    pub chain_1_regular_failure: Vec<(u64, u64)>,
    pub chain_2_regular_failure: Vec<(u64, u64)>,
    
    // Statistics
    pub account_stats: AccountSelectionStats,
    pub start_time: Instant,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Default for SimulationResults {
    fn default() -> Self {
        Self {
            transactions_sent: 0,
            cat_transactions: 0,
            regular_transactions: 0,
            initial_balance: 0,
            num_accounts: 0,
            target_tps: 0,
            sim_total_block_number: 0,
            zipf_parameter: 0.0,
            ratio_cats: 0.0,
            block_interval: 0.0,
            cat_lifetime: 0,
            initialization_wait_blocks: 0,
            chain_delays: Vec::new(),
            chain_1_pending: Vec::new(),
            chain_2_pending: Vec::new(),
            chain_1_success: Vec::new(),
            chain_2_success: Vec::new(),
            chain_1_failure: Vec::new(),
            chain_2_failure: Vec::new(),
            chain_1_cat_pending: Vec::new(),
            chain_2_cat_pending: Vec::new(),
            chain_1_cat_success: Vec::new(),
            chain_2_cat_success: Vec::new(),
            chain_1_cat_failure: Vec::new(),
            chain_2_cat_failure: Vec::new(),
            chain_1_regular_pending: Vec::new(),
            chain_2_regular_pending: Vec::new(),
            chain_1_regular_success: Vec::new(),
            chain_2_regular_success: Vec::new(),
            chain_1_regular_failure: Vec::new(),
            chain_2_regular_failure: Vec::new(),
            account_stats: AccountSelectionStats::new(),
            start_time: Instant::now(),
        }
    }
}

impl SimulationResults {
    /// Saves results to the default directory
    pub async fn save(&self) -> Result<(), String> {
        self.save_to_directory("simulator/results/sim_simple").await
    }

    /// Saves all simulation data to JSON files in the specified directory
    pub async fn save_to_directory(&self, base_dir: &str) -> Result<(), String> {
        // Print final statistics
        logging::log("SIMULATOR", "\n=== Simulation Statistics ===");
        logging::log("SIMULATOR", &format!("Total Transactions: {}", self.transactions_sent));
        logging::log("SIMULATOR", &format!("CAT Transactions: {}", self.cat_transactions));
        logging::log("SIMULATOR", &format!("Regular Transactions: {}", self.regular_transactions));
        logging::log("SIMULATOR", &format!("Actual TPS: {:.2}", self.transactions_sent as f64 / self.start_time.elapsed().as_secs_f64()));
        logging::log("SIMULATOR", "===========================");
        
        // Save statistics to JSON file
        let stats = serde_json::json!({
            "parameters": {
                "initial_balance": self.initial_balance,
                "num_accounts": self.num_accounts,
                "target_tps": self.target_tps,
                "sim_total_block_number": self.sim_total_block_number,
                "zipf_parameter": self.zipf_parameter,
                "ratio_cats": self.ratio_cats,
                "block_interval": self.block_interval,
                "chain_delays": self.chain_delays.clone()
            },
            "results": {
                "total_transactions": self.transactions_sent,
                "cat_transactions": self.cat_transactions,
                "regular_transactions": self.regular_transactions
            }
        });

        // Create results directories if they don't exist
        fs::create_dir_all(&format!("{}/data", base_dir)).expect("Failed to create results directory");

        // Save simulation stats
        let stats_file = format!("{}/data/simulation_stats.json", base_dir);
        fs::write(&stats_file, serde_json::to_string_pretty(&stats).expect("Failed to serialize stats")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved simulation statistics to {}", stats_file));

        // Save pending transactions data from chain 1
        let pending_txs_chain_1 = serde_json::json!({
            "chain_1_pending": self.chain_1_pending.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let pending_file_chain_1 = format!("{}/data/pending_transactions_chain_1.json", base_dir);
        fs::write(&pending_file_chain_1, serde_json::to_string_pretty(&pending_txs_chain_1).expect("Failed to serialize pending transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved pending transactions data to {}", pending_file_chain_1));

        // Save pending transactions data from chain 2
        let pending_txs_chain_2 = serde_json::json!({
            "chain_2_pending": self.chain_2_pending.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let pending_file_chain_2 = format!("{}/data/pending_transactions_chain_2.json", base_dir);
        fs::write(&pending_file_chain_2, serde_json::to_string_pretty(&pending_txs_chain_2).expect("Failed to serialize pending transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved pending transactions data to {}", pending_file_chain_2));

        // Save success transactions data from chain 1
        let success_txs_chain_1 = serde_json::json!({
            "chain_1_success": self.chain_1_success.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let success_file_chain_1 = format!("{}/data/success_transactions_chain_1.json", base_dir);
        fs::write(&success_file_chain_1, serde_json::to_string_pretty(&success_txs_chain_1).expect("Failed to serialize success transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved success transactions data to {}", success_file_chain_1));

        // Save success transactions data from chain 2
        let success_txs_chain_2 = serde_json::json!({
            "chain_2_success": self.chain_2_success.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let success_file_chain_2 = format!("{}/data/success_transactions_chain_2.json", base_dir);
        fs::write(&success_file_chain_2, serde_json::to_string_pretty(&success_txs_chain_2).expect("Failed to serialize success transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved success transactions data to {}", success_file_chain_2));

        // Save failure transactions data from chain 1
        let failure_txs_chain_1 = serde_json::json!({
            "chain_1_failure": self.chain_1_failure.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let failure_file_chain_1 = format!("{}/data/failure_transactions_chain_1.json", base_dir);
        fs::write(&failure_file_chain_1, serde_json::to_string_pretty(&failure_txs_chain_1).expect("Failed to serialize failure transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved failure transactions data to {}", failure_file_chain_1));

        // Save failure transactions data from chain 2
        let failure_txs_chain_2 = serde_json::json!({
            "chain_2_failure": self.chain_2_failure.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let failure_file_chain_2 = format!("{}/data/failure_transactions_chain_2.json", base_dir);
        fs::write(&failure_file_chain_2, serde_json::to_string_pretty(&failure_txs_chain_2).expect("Failed to serialize failure transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved failure transactions data to {}", failure_file_chain_2));

        // Save CAT failure transactions data from chain 1
        let cat_failure_txs_chain_1 = serde_json::json!({
            "chain_1_cat_failure": self.chain_1_cat_failure.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_failure_file_chain_1 = format!("{}/data/cat_failure_transactions_chain_1.json", base_dir);
        fs::write(&cat_failure_file_chain_1, serde_json::to_string_pretty(&cat_failure_txs_chain_1).expect("Failed to serialize CAT failure transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT failure transactions data to {}", cat_failure_file_chain_1));

        // Save CAT pending transactions data from chain 1
        let cat_pending_txs_chain_1 = serde_json::json!({
            "chain_1_cat_pending": self.chain_1_cat_pending.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_pending_file_chain_1 = format!("{}/data/cat_pending_transactions_chain_1.json", base_dir);
        fs::write(&cat_pending_file_chain_1, serde_json::to_string_pretty(&cat_pending_txs_chain_1).expect("Failed to serialize CAT pending transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT pending transactions data to {}", cat_pending_file_chain_1));

        // Save CAT pending transactions data from chain 2
        let cat_pending_txs_chain_2 = serde_json::json!({
            "chain_2_cat_pending": self.chain_2_cat_pending.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_pending_file_chain_2 = format!("{}/data/cat_pending_transactions_chain_2.json", base_dir);
        fs::write(&cat_pending_file_chain_2, serde_json::to_string_pretty(&cat_pending_txs_chain_2).expect("Failed to serialize CAT pending transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT pending transactions data to {}", cat_pending_file_chain_2));

        // Save CAT success transactions data from chain 1
        let cat_success_txs_chain_1 = serde_json::json!({
            "chain_1_cat_success": self.chain_1_cat_success.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_success_file_chain_1 = format!("{}/data/cat_success_transactions_chain_1.json", base_dir);
        fs::write(&cat_success_file_chain_1, serde_json::to_string_pretty(&cat_success_txs_chain_1).expect("Failed to serialize CAT success transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT success transactions data to {}", cat_success_file_chain_1));

        // Save CAT success transactions data from chain 2
        let cat_success_txs_chain_2 = serde_json::json!({
            "chain_2_cat_success": self.chain_2_cat_success.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_success_file_chain_2 = format!("{}/data/cat_success_transactions_chain_2.json", base_dir);
        fs::write(&cat_success_file_chain_2, serde_json::to_string_pretty(&cat_success_txs_chain_2).expect("Failed to serialize CAT success transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT success transactions data to {}", cat_success_file_chain_2));

        // Save CAT failure transactions data from chain 1
        let cat_failure_txs_chain_1 = serde_json::json!({
            "chain_1_cat_failure": self.chain_1_cat_failure.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_failure_file_chain_1 = format!("{}/data/cat_failure_transactions_chain_1.json", base_dir);
        fs::write(&cat_failure_file_chain_1, serde_json::to_string_pretty(&cat_failure_txs_chain_1).expect("Failed to serialize CAT failure transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT failure transactions data to {}", cat_failure_file_chain_1));

        // Save CAT failure transactions data from chain 2
        let cat_failure_txs_chain_2 = serde_json::json!({
            "chain_2_cat_failure": self.chain_2_cat_failure.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_failure_file_chain_2 = format!("{}/data/cat_failure_transactions_chain_2.json", base_dir);
        fs::write(&cat_failure_file_chain_2, serde_json::to_string_pretty(&cat_failure_txs_chain_2).expect("Failed to serialize CAT failure transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT failure transactions data to {}", cat_failure_file_chain_2));

        // Save regular pending transactions data from chain 1
        let regular_pending_txs_chain_1 = serde_json::json!({
            "chain_1_regular_pending": self.chain_1_regular_pending.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let regular_pending_file_chain_1 = format!("{}/data/regular_pending_transactions_chain_1.json", base_dir);
        fs::write(&regular_pending_file_chain_1, serde_json::to_string_pretty(&regular_pending_txs_chain_1).expect("Failed to serialize regular pending transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved regular pending transactions data to {}", regular_pending_file_chain_1));

        // Save regular pending transactions data from chain 2
        let regular_pending_txs_chain_2 = serde_json::json!({
            "chain_2_regular_pending": self.chain_2_regular_pending.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let regular_pending_file_chain_2 = format!("{}/data/regular_pending_transactions_chain_2.json", base_dir);
        fs::write(&regular_pending_file_chain_2, serde_json::to_string_pretty(&regular_pending_txs_chain_2).expect("Failed to serialize regular pending transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved regular pending transactions data to {}", regular_pending_file_chain_2));

        // Save regular success transactions data from chain 1
        let regular_success_txs_chain_1 = serde_json::json!({
            "chain_1_regular_success": self.chain_1_regular_success.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let regular_success_file_chain_1 = format!("{}/data/regular_success_transactions_chain_1.json", base_dir);
        fs::write(&regular_success_file_chain_1, serde_json::to_string_pretty(&regular_success_txs_chain_1).expect("Failed to serialize regular success transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved regular success transactions data to {}", regular_success_file_chain_1));

        // Save regular success transactions data from chain 2
        let regular_success_txs_chain_2 = serde_json::json!({
            "chain_2_regular_success": self.chain_2_regular_success.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let regular_success_file_chain_2 = format!("{}/data/regular_success_transactions_chain_2.json", base_dir);
        fs::write(&regular_success_file_chain_2, serde_json::to_string_pretty(&regular_success_txs_chain_2).expect("Failed to serialize regular success transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved regular success transactions data to {}", regular_success_file_chain_2));

        // Save regular failure transactions data from chain 1
        let regular_failure_txs_chain_1 = serde_json::json!({
            "chain_1_regular_failure": self.chain_1_regular_failure.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let regular_failure_file_chain_1 = format!("{}/data/regular_failure_transactions_chain_1.json", base_dir);
        fs::write(&regular_failure_file_chain_1, serde_json::to_string_pretty(&regular_failure_txs_chain_1).expect("Failed to serialize regular failure transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved regular failure transactions data to {}", regular_failure_file_chain_1));

        // Save regular failure transactions data from chain 2
        let regular_failure_txs_chain_2 = serde_json::json!({
            "chain_2_regular_failure": self.chain_2_regular_failure.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let regular_failure_file_chain_2 = format!("{}/data/regular_failure_transactions_chain_2.json", base_dir);
        fs::write(&regular_failure_file_chain_2, serde_json::to_string_pretty(&regular_failure_txs_chain_2).expect("Failed to serialize regular failure transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved regular failure transactions data to {}", regular_failure_file_chain_2));

        // Save account selection data to files
        let (sender_json, receiver_json) = self.account_stats.to_json();
        let sender_file = format!("{}/data/account_sender_selection.json", base_dir);
        fs::write(&sender_file, serde_json::to_string_pretty(&sender_json).expect("Failed to serialize sender stats")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved sender selection data to {}", sender_file));
        let receiver_file = format!("{}/data/account_receiver_selection.json", base_dir);
        fs::write(&receiver_file, serde_json::to_string_pretty(&receiver_json).expect("Failed to serialize receiver stats")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved receiver selection data to {}", receiver_file));

        Ok(())
    }
} 