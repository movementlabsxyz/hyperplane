//! Simulation results storage and serialization.
//! 
//! Handles saving simulation data to JSON files for analysis.

use std::time::Instant;
use std::fs;
use serde_json;
use crate::account_selection::AccountSelectionStats;
use hyperplane::utils::logging;
use sysinfo::System;
use std::sync::Mutex;
use lazy_static::lazy_static;

// ------------------------------------------------------------------------------------------------
// Global System Instance
// ------------------------------------------------------------------------------------------------

lazy_static! {
    static ref SYSTEM: Mutex<System> = Mutex::new(System::new_all());
}

fn get_system() -> &'static Mutex<System> {
    &SYSTEM
}

// ------------------------------------------------------------------------------------------------
// Data Structures
// ------------------------------------------------------------------------------------------------

/// Stores all simulation results and statistics
#[derive(Debug, Clone)]
pub struct SimulationResults {
    // Transaction counts
    pub transactions_sent: u64,
    pub cat_transactions: u64,
    pub regular_transactions: u64,
    
    // Simulation parameters
    pub initial_balance: u64,
    pub num_accounts: usize,
    pub target_tpb: u64,
    pub sim_total_block_number: u64,  // Total number of blocks to simulate
    pub zipf_parameter: f64,
    pub ratio_cats: f64,
    pub block_interval: f64,
    pub cat_lifetime: u64,
    pub initialization_wait_blocks: u64,
    pub chain_delays: Vec<f64>,  // Chain delays in blocks
    
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
    
    // Chain data - Detailed CAT pending states
    pub chain_1_cat_pending_resolving: Vec<(u64, u64)>,
    pub chain_2_cat_pending_resolving: Vec<(u64, u64)>,
    pub chain_1_cat_pending_postponed: Vec<(u64, u64)>,
    pub chain_2_cat_pending_postponed: Vec<(u64, u64)>,
    
    // Chain data - Regular transactions
    pub chain_1_regular_pending: Vec<(u64, u64)>,
    pub chain_2_regular_pending: Vec<(u64, u64)>,
    pub chain_1_regular_success: Vec<(u64, u64)>,
    pub chain_2_regular_success: Vec<(u64, u64)>,
    pub chain_1_regular_failure: Vec<(u64, u64)>,
    pub chain_2_regular_failure: Vec<(u64, u64)>,
    
    // Chain data - Locked keys
    pub chain_1_locked_keys: Vec<(u64, u64)>,
    pub chain_2_locked_keys: Vec<(u64, u64)>,
    
    // Chain data - Transactions per block
    pub chain_1_tx_per_block: Vec<(u64, u64)>,
    pub chain_2_tx_per_block: Vec<(u64, u64)>,
    
        // Memory usage tracking
    pub memory_usage: Vec<(u64, u64)>, // (block_height, memory_usage_bytes)
    pub total_memory: Vec<(u64, u64)>, // (block_height, total_memory_bytes)
    
    // CPU usage tracking
    pub cpu_usage: Vec<(u64, f64)>, // (block_height, process_cpu_usage_percent)
    pub total_cpu_usage: Vec<(u64, f64)>, // (block_height, total_system_cpu_usage_percent)
    
    // Loop steps without transaction issuance tracking
    pub loop_steps_without_tx_issuance: Vec<(u64, u64)>, // (block_height, loop_steps_count)
    
    // CL queue length tracking
    pub cl_queue_length: Vec<(u64, u64)>, // (block_height, queue_length)
    
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
            target_tpb: 0,
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
            chain_1_cat_pending_resolving: Vec::new(),
            chain_2_cat_pending_resolving: Vec::new(),
            chain_1_cat_pending_postponed: Vec::new(),
            chain_2_cat_pending_postponed: Vec::new(),
            chain_1_regular_pending: Vec::new(),
            chain_2_regular_pending: Vec::new(),
            chain_1_regular_success: Vec::new(),
            chain_2_regular_success: Vec::new(),
            chain_1_regular_failure: Vec::new(),
            chain_2_regular_failure: Vec::new(),
            chain_1_locked_keys: Vec::new(),
            chain_2_locked_keys: Vec::new(),
            chain_1_tx_per_block: Vec::new(),
            chain_2_tx_per_block: Vec::new(),
            memory_usage: Vec::new(),
            total_memory: Vec::new(),
            cpu_usage: Vec::new(),
            total_cpu_usage: Vec::new(),
            loop_steps_without_tx_issuance: Vec::new(),
            cl_queue_length: Vec::new(),
            account_stats: AccountSelectionStats::new(),
            start_time: Instant::now(),
        }
    }
}

impl SimulationResults {
    /// Gets the current memory usage in bytes
    pub fn get_current_memory_usage() -> u64 {
        // Use sysinfo crate or similar for more accurate memory measurement
        // For now, use a simple approximation based on process memory
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
                for line in contents.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
            0 // Fallback if we can't read the file or find VmRSS
        }
        
        #[cfg(target_os = "macos")]
        {
            // On macOS, try to get memory usage using libc
            #[allow(deprecated)]
            unsafe {
                use std::mem;
                let mut info: libc::mach_task_basic_info = mem::zeroed();
                let mut count = mem::size_of::<libc::mach_task_basic_info>() as libc::mach_msg_type_number_t;
                
                let result = libc::task_info(
                    libc::mach_task_self(),
                    libc::MACH_TASK_BASIC_INFO,
                    &mut info as *mut _ as *mut i32,
                    &mut count,
                );
                
                if result == libc::KERN_SUCCESS {
                    return info.resident_size;
                }
            }
            
            // Fallback: try to read from /proc/self/statm if available
            if let Ok(contents) = std::fs::read_to_string("/proc/self/statm") {
                if let Some(first) = contents.split_whitespace().next() {
                    if let Ok(pages) = first.parse::<u64>() {
                        return pages * 4096; // Convert pages to bytes (assuming 4KB pages)
                    }
                }
            }
            
            // Final fallback: return a reasonable estimate
            // Since we can't easily get heap usage, return 0 for now
            0
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            // On other platforms, try to read from /proc/self/statm if available
            if let Ok(contents) = std::fs::read_to_string("/proc/self/statm") {
                if let Some(first) = contents.split_whitespace().next() {
                    if let Ok(pages) = first.parse::<u64>() {
                        return pages * 4096; // Convert pages to bytes (assuming 4KB pages)
                    }
                }
            }
            
            // Fallback: return 0 for now
            0
        }
    }

    /// Gets the current total RAM usage in bytes
    pub fn get_current_total_memory() -> u64 {
        let system = get_system();
        if let Ok(mut sys) = system.lock() {
            // Refresh memory information
            sys.refresh_memory();
            
            return sys.used_memory(); // sysinfo returns bytes directly
        }
        
        // Fallback if we can't get system info
        0
    }

    /// Gets the current process CPU usage as a percentage
    /// Uses sysinfo crate for accurate cross-platform CPU measurement
    pub fn get_current_cpu_usage() -> f64 {
        let system = get_system();
        if let Ok(mut sys) = system.lock() {
            // Refresh process information
            sys.refresh_processes();
            
            // Get current process ID
            let pid = sysinfo::Pid::from(std::process::id() as usize);
            
            // Get process CPU usage percentage
            if let Some(process) = sys.process(pid) {
                return process.cpu_usage() as f64;
            }
        }
        
        // Fallback if we can't get process info
        0.0
    }

    /// Gets the current total system CPU usage as a percentage
    /// Uses sysinfo crate for accurate cross-platform CPU measurement
    pub fn get_current_total_cpu_usage() -> f64 {
        let system = get_system();
        if let Ok(mut sys) = system.lock() {
            // Refresh CPU information
            sys.refresh_cpu();
            
            // Get overall CPU usage percentage
            let cpu_usage = sys.global_cpu_info().cpu_usage();
            return cpu_usage as f64;
        }
        
        // Fallback if we can't get system info
        0.0
    }

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
                "target_tpb": self.target_tpb,
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

        // Save detailed CAT pending states data from chain 1
        let cat_pending_resolving_txs_chain_1 = serde_json::json!({
            "chain_1_cat_pending_resolving": self.chain_1_cat_pending_resolving.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_pending_resolving_file_chain_1 = format!("{}/data/cat_pending_resolving_transactions_chain_1.json", base_dir);
        fs::write(&cat_pending_resolving_file_chain_1, serde_json::to_string_pretty(&cat_pending_resolving_txs_chain_1).expect("Failed to serialize CAT pending resolving transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT pending resolving transactions data to {}", cat_pending_resolving_file_chain_1));

        let cat_pending_postponed_txs_chain_1 = serde_json::json!({
            "chain_1_cat_pending_postponed": self.chain_1_cat_pending_postponed.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_pending_postponed_file_chain_1 = format!("{}/data/cat_pending_postponed_transactions_chain_1.json", base_dir);
        fs::write(&cat_pending_postponed_file_chain_1, serde_json::to_string_pretty(&cat_pending_postponed_txs_chain_1).expect("Failed to serialize CAT pending postponed transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT pending postponed transactions data to {}", cat_pending_postponed_file_chain_1));

        // Save detailed CAT pending states data from chain 2
        let cat_pending_resolving_txs_chain_2 = serde_json::json!({
            "chain_2_cat_pending_resolving": self.chain_2_cat_pending_resolving.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_pending_resolving_file_chain_2 = format!("{}/data/cat_pending_resolving_transactions_chain_2.json", base_dir);
        fs::write(&cat_pending_resolving_file_chain_2, serde_json::to_string_pretty(&cat_pending_resolving_txs_chain_2).expect("Failed to serialize CAT pending resolving transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT pending resolving transactions data to {}", cat_pending_resolving_file_chain_2));

        let cat_pending_postponed_txs_chain_2 = serde_json::json!({
            "chain_2_cat_pending_postponed": self.chain_2_cat_pending_postponed.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cat_pending_postponed_file_chain_2 = format!("{}/data/cat_pending_postponed_transactions_chain_2.json", base_dir);
        fs::write(&cat_pending_postponed_file_chain_2, serde_json::to_string_pretty(&cat_pending_postponed_txs_chain_2).expect("Failed to serialize CAT pending postponed transactions")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CAT pending postponed transactions data to {}", cat_pending_postponed_file_chain_2));

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

        // Save locked keys data from chain 1
        let locked_keys_chain_1 = serde_json::json!({
            "chain_1_locked_keys": self.chain_1_locked_keys.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let locked_keys_file_chain_1 = format!("{}/data/locked_keys_chain_1.json", base_dir);
        fs::write(&locked_keys_file_chain_1, serde_json::to_string_pretty(&locked_keys_chain_1).expect("Failed to serialize locked keys")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved locked keys data to {}", locked_keys_file_chain_1));

        // Save locked keys data from chain 2
        let locked_keys_chain_2 = serde_json::json!({
            "chain_2_locked_keys": self.chain_2_locked_keys.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let locked_keys_file_chain_2 = format!("{}/data/locked_keys_chain_2.json", base_dir);
        fs::write(&locked_keys_file_chain_2, serde_json::to_string_pretty(&locked_keys_chain_2).expect("Failed to serialize locked keys")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved locked keys data to {}", locked_keys_file_chain_2));

        // Save transactions per block data from chain 1
        let tx_per_block_chain_1 = serde_json::json!({
            "chain_1_tx_per_block": self.chain_1_tx_per_block.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let tx_per_block_file_chain_1 = format!("{}/data/tx_per_block_chain_1.json", base_dir);
        fs::write(&tx_per_block_file_chain_1, serde_json::to_string_pretty(&tx_per_block_chain_1).expect("Failed to serialize transactions per block")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved transactions per block data to {}", tx_per_block_file_chain_1));

        // Save transactions per block data from chain 2
        let tx_per_block_chain_2 = serde_json::json!({
            "chain_2_tx_per_block": self.chain_2_tx_per_block.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let tx_per_block_file_chain_2 = format!("{}/data/tx_per_block_chain_2.json", base_dir);
        fs::write(&tx_per_block_file_chain_2, serde_json::to_string_pretty(&tx_per_block_chain_2).expect("Failed to serialize transactions per block")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved transactions per block data to {}", tx_per_block_file_chain_2));

        // Save account selection data to files
        let (sender_json, receiver_json) = self.account_stats.to_json();
        let sender_file = format!("{}/data/account_sender_selection.json", base_dir);
        fs::write(&sender_file, serde_json::to_string_pretty(&sender_json).expect("Failed to serialize sender stats")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved sender selection data to {}", sender_file));
        let receiver_file = format!("{}/data/account_receiver_selection.json", base_dir);
        fs::write(&receiver_file, serde_json::to_string_pretty(&receiver_json).expect("Failed to serialize receiver stats")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved receiver selection data to {}", receiver_file));

        // Save system memory usage data
        let system_memory_data = serde_json::json!({
            "system_memory": self.memory_usage.iter().map(|(height, bytes)| {
                serde_json::json!({
                    "height": height,
                    "bytes": bytes
                })
            }).collect::<Vec<_>>()
        });
        let system_memory_file = format!("{}/data/system_memory.json", base_dir);
        fs::write(&system_memory_file, serde_json::to_string_pretty(&system_memory_data).expect("Failed to serialize system memory")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved system memory data to {}", system_memory_file));

        // Save system total memory usage data
        let system_total_memory_data = serde_json::json!({
            "system_total_memory": self.total_memory.iter().map(|(height, bytes)| {
                serde_json::json!({
                    "height": height,
                    "bytes": bytes
                })
            }).collect::<Vec<_>>()
        });
        let system_total_memory_file = format!("{}/data/system_total_memory.json", base_dir);
        fs::write(&system_total_memory_file, serde_json::to_string_pretty(&system_total_memory_data).expect("Failed to serialize system total memory")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved system total memory data to {}", system_total_memory_file));

        // Save system CPU usage data
        let system_cpu_data = serde_json::json!({
            "system_cpu": self.cpu_usage.iter().map(|(height, percent)| {
                serde_json::json!({
                    "height": height,
                    "percent": percent
                })
            }).collect::<Vec<_>>()
        });
        let system_cpu_file = format!("{}/data/system_cpu.json", base_dir);
        fs::write(&system_cpu_file, serde_json::to_string_pretty(&system_cpu_data).expect("Failed to serialize system CPU")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved system CPU data to {}", system_cpu_file));

        // Save system total CPU usage data
        let system_total_cpu_data = serde_json::json!({
            "system_total_cpu": self.total_cpu_usage.iter().map(|(height, percent)| {
                serde_json::json!({
                    "height": height,
                    "percent": percent
                })
            }).collect::<Vec<_>>()
        });
        let system_total_cpu_file = format!("{}/data/system_total_cpu.json", base_dir);
        fs::write(&system_total_cpu_file, serde_json::to_string_pretty(&system_total_cpu_data).expect("Failed to serialize system total CPU")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved system total CPU data to {}", system_total_cpu_file));

        // Save loop steps without transaction issuance data
        let loop_steps_data = serde_json::json!({
            "loop_steps_without_tx_issuance": self.loop_steps_without_tx_issuance.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let loop_steps_file = format!("{}/data/loop_steps_without_tx_issuance.json", base_dir);
        fs::write(&loop_steps_file, serde_json::to_string_pretty(&loop_steps_data).expect("Failed to serialize loop steps data")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved loop steps data to {}", loop_steps_file));

        // Save CL queue length data
        let cl_queue_length_data = serde_json::json!({
            "cl_queue_length": self.cl_queue_length.iter().map(|(height, count)| {
                serde_json::json!({
                    "height": height,
                    "count": count
                })
            }).collect::<Vec<_>>()
        });
        let cl_queue_length_file = format!("{}/data/cl_queue_length.json", base_dir);
        fs::write(&cl_queue_length_file, serde_json::to_string_pretty(&cl_queue_length_data).expect("Failed to serialize CL queue length data")).map_err(|e| e.to_string())?;
        logging::log("SIMULATOR", &format!("Saved CL queue length data to {}", cl_queue_length_file));

        Ok(())
    }
} 