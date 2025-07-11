use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json;



// ------------------------------------------------------------------------------------------------
// Core Types and Traits
// ------------------------------------------------------------------------------------------------

/// Generic sweep runner that eliminates duplication across sweep simulations.
/// 
/// This struct provides a unified interface for running parameter sweep simulations.
/// It handles the common workflow of loading configurations, running multiple simulations
/// with different parameter values, and saving results.
/// 
/// # Type Parameters
/// 
/// * `T` - The type of the parameter being swept (e.g., f64 for block intervals, u64 for delays)
/// 
/// # Fields
/// 
/// * `sweep_name` - Human-readable name for logging and display
/// * `results_dir` - Directory name for storing simulation results
/// * `parameter_name` - Name of the parameter being varied (for JSON output)
/// * `parameter_values` - List of parameter values to test
/// * `config_loader` - Function to load the sweep configuration
/// * `config_modifier` - Function to create a modified config for each simulation
/// * `result_saver` - Function to save combined results from all simulations
pub struct SweepRunner<T> {
    sweep_name: String,
    results_dir: String,
    parameter_name: String,
    parameter_values: Vec<T>,
    config_loader: Box<dyn Fn() -> Result<Box<dyn SweepConfigTrait>, crate::config::ConfigError>>,
    config_modifier: Box<dyn Fn(&Box<dyn SweepConfigTrait>, T) -> crate::config::Config>,
    result_saver: Box<dyn Fn(&str, &[(T, crate::SimulationResults)]) -> Result<(), crate::config::ConfigError>>,
}

/// Trait for sweep configurations to allow generic handling across different config types.
/// 
/// This trait provides a common interface for accessing configuration data regardless
/// of the specific sweep configuration type. It allows the SweepRunner to work with
/// any configuration that implements this trait.
pub trait SweepConfigTrait {
    /// Returns the number of simulations to run in this sweep
    fn get_num_simulations(&self) -> usize;
    
    /// Returns the number of runs per simulation
    fn get_num_runs(&self) -> u32;
    
    /// Returns a reference to the network configuration
    fn get_network_config(&self) -> &crate::config::NetworkConfig;
    
    /// Returns a reference to the account configuration
    fn get_account_config(&self) -> &crate::config::AccountConfig;
    
    /// Returns a reference to the transaction configuration
    fn get_transaction_config(&self) -> &crate::config::TransactionConfig;
    
    /// Returns a reference to the simulation configuration
    fn get_simulation_config(&self) -> Option<&crate::config::SimulationConfig>;
    
    /// Returns a reference to the underlying configuration as Any for type casting
    fn as_any(&self) -> &dyn std::any::Any;
}

// ------------------------------------------------------------------------------------------------
// SweepRunner Implementation
// ------------------------------------------------------------------------------------------------

/// Implementation of SweepRunner methods for parameter types that support Debug and Clone.
/// 
/// This implementation provides the core functionality for running sweep simulations.
/// The Debug and Clone bounds are required for logging and parameter handling.
impl<T: std::fmt::Debug + Clone> SweepRunner<T> {
    /// Creates a new SweepRunner instance.
    /// 
    /// # Arguments
    /// 
    /// * `sweep_name` - Human-readable name for the sweep (used in logging and display)
    /// * `results_dir` - Directory name where results will be stored
    /// * `parameter_name` - Name of the parameter being varied (used in JSON output)
    /// * `parameter_values` - Vector of parameter values to test
    /// * `config_loader` - Function that loads the sweep configuration
    /// * `config_modifier` - Function that creates a modified config for each simulation
    /// * `result_saver` - Function that saves combined results from all simulations
    /// 
    /// # Returns
    /// 
    /// A new SweepRunner instance ready to execute the sweep simulation
    pub fn new(
        sweep_name: &str,
        results_dir: &str,
        parameter_name: &str,
        parameter_values: Vec<T>,
        config_loader: Box<dyn Fn() -> Result<Box<dyn SweepConfigTrait>, crate::config::ConfigError>>,
        config_modifier: Box<dyn Fn(&Box<dyn SweepConfigTrait>, T) -> crate::config::Config>,
        result_saver: Box<dyn Fn(&str, &[(T, crate::SimulationResults)]) -> Result<(), crate::config::ConfigError>>,
    ) -> Self {
        Self {
            sweep_name: sweep_name.to_string(),
            results_dir: results_dir.to_string(),
            parameter_name: parameter_name.to_string(),
            parameter_values,
            config_loader,
            config_modifier,
            result_saver,
        }
    }

    // ------------------------------------------------------------------------------------------------
    // Main Simulation Execution
    // ------------------------------------------------------------------------------------------------
    
    /// Runs the sweep simulation.
    /// 
    /// This method orchestrates the entire sweep simulation process:
    /// 1. Creates necessary directories for results
    /// 2. Sets up logging if enabled
    /// 3. Loads the sweep configuration
    /// 4. Runs each simulation with different parameter values
    /// 5. Saves individual and combined results
    /// 6. Provides progress feedback
    /// 
    /// # Returns
    /// 
    /// Result indicating success or failure of the sweep simulation
    pub async fn run(&self) -> Result<(), crate::config::ConfigError>
    where
        T: serde::Serialize,
    {
        // Create results directory if it doesn't exist
        self.create_directories();
        
        // Setup logging (will be configured after loading config)
        // Note: setup_logging is called after config is loaded

        // Load sweep configuration
        let sweep_config = (self.config_loader)()?;

        // Setup logging with the first config (we'll use the first simulation's config for logging)
        let first_config = (self.config_modifier)(&sweep_config, self.parameter_values[0].clone());
        self.setup_logging(&first_config);

        // Write metadata.json for Python averaging script
        let metadata_path = format!("simulator/results/{}/data/metadata.json", self.results_dir);
        let metadata = serde_json::json!({
            "num_runs": sweep_config.get_num_runs(),
            "num_simulations": sweep_config.get_num_simulations(),
            "parameter_name": self.parameter_name,
            "parameter_values": self.parameter_values,
        });
        std::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap()).expect("Failed to write metadata.json");

        // Log sweep start
        self.log_sweep_start(&sweep_config);

        // Display sweep name before progress bar
        println!("Running Sweep: {}", self.sweep_name);

        // Create progress bar for sweep
        let progress_bar = self.create_progress_bar(sweep_config.get_num_simulations());

        // Store results for each simulation
        let mut all_results = Vec::new();

        // Get number of runs from config
        let num_runs = sweep_config.get_num_runs();

        // Run each simulation with different parameter value
        for (sim_index, param_value) in self.parameter_values.iter().enumerate() {
            self.log_simulation_start(sim_index, sweep_config.get_num_simulations(), param_value);

            // Create a modified config with the current parameter value
            let sim_config = (self.config_modifier)(&sweep_config, param_value.clone());

            // Store results for all runs of this parameter set
            let mut parameter_results = Vec::new();
            let mut total_retries_for_simulation = 0;

            // Run this parameter set multiple times
            for run in 1..=num_runs {
                let mut retry_count = 0;
                let max_retries = sim_config.simulation_config.max_retries;
                
                loop {
                    logging::log("SIMULATOR", &format!("=== Starting Run {}/{} for parameter {}: {:?} ===", 
                        run, num_runs, self.parameter_name, param_value));
                    
                    if retry_count > 0 {
                        logging::log("SIMULATOR", &format!("=== Retry attempt {}/{} for Run {}/{} ===", 
                            retry_count, max_retries, run, num_runs));
                        // Update progress bar message to show retry count
                        progress_bar.set_message(self.format_progress_message(sim_index, sweep_config.get_num_simulations(), param_value, Some(total_retries_for_simulation)));
                    }

                    // Initialize simulation results for this run
                    let mut results = self.initialize_simulation_results(&sim_config, sim_index, param_value);

                    // Setup test nodes
                    let (_hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = crate::testnodes::setup_test_nodes(
                        Duration::from_secs_f64(sim_config.network_config.block_interval),
                        &sim_config.network_config.chain_delays,
                        sim_config.transaction_config.allow_cat_pending_dependencies,
                        sim_config.transaction_config.cat_lifetime_blocks,
                    ).await;
                    
                    // Initialize accounts with initial balance
                    let account_init_result = crate::network::initialize_accounts(
                        &[cl_node.clone()], 
                        sim_config.account_config.initial_balance.try_into().unwrap(), 
                        sim_config.account_config.num_accounts.try_into().unwrap(),
                        Some(&[hig_node_1.clone(), hig_node_2.clone()]),
                        sim_config.network_config.block_interval,
                        sim_config.simulation_config.funding_wait_blocks,
                    ).await;

                    // Check if account initialization failed
                    if let Err(e) = account_init_result {
                        retry_count += 1;
                        total_retries_for_simulation += 1;
                        logging::log("SIMULATOR", &format!("DEBUG: Account init failed. retry_count: {}, total_retries: {}, max_retries: {}", retry_count, total_retries_for_simulation, max_retries));
                        if total_retries_for_simulation >= max_retries {
                            let error_context = format!(
                                "Sweep '{}' failed during simulation {}/{} run {}/{} with {}: {:?} after {} total retries. Error: {}",
                                self.sweep_name,
                                sim_index + 1,
                                sweep_config.get_num_simulations(),
                                run,
                                num_runs,
                                self.parameter_name,
                                param_value,
                                total_retries_for_simulation,
                                e
                            );
                            return Err(crate::config::ConfigError::ValidationError(error_context));
                        }
                        logging::log("SIMULATOR", &format!("Account initialization failed, retrying... (attempt {}/{})", retry_count, max_retries));
                        // Update progress message to show retry count
                        progress_bar.set_message(self.format_progress_message(sim_index, sweep_config.get_num_simulations(), param_value, Some(total_retries_for_simulation)));
                        continue;
                    }

                    // Run simulation with run message and retry count
                    let run_message = format!("Sim {} Run {}/{}", sim_index + 1, run, num_runs);
                    let simulation_result = crate::run_simulation::run_simulation_with_message_and_retries(
                        cl_node,
                        vec![hig_node_1, hig_node_2],
                        &mut results,
                        Some(run_message),
                        Some(total_retries_for_simulation),
                    ).await;

                    // Check if simulation failed
                    if let Err(e) = simulation_result {
                        retry_count += 1;
                        total_retries_for_simulation += 1;
                        logging::log("SIMULATOR", &format!("DEBUG: Simulation failed. retry_count: {}, total_retries: {}, max_retries: {}", retry_count, total_retries_for_simulation, max_retries));
                        if total_retries_for_simulation >= max_retries {
                            let error_context = format!(
                                "Sweep '{}' failed during simulation {}/{} run {}/{} with {}: {:?} after {} total retries. Error: {}",
                                self.sweep_name,
                                sim_index + 1,
                                sweep_config.get_num_simulations(),
                                run,
                                num_runs,
                                self.parameter_name,
                                param_value,
                                total_retries_for_simulation,
                                e
                            );
                            return Err(crate::config::ConfigError::ValidationError(error_context));
                        }
                        logging::log("SIMULATOR", &format!("Simulation failed, retrying... (attempt {}/{})", retry_count, max_retries));
                        // Update progress message to show retry count
                        progress_bar.set_message(self.format_progress_message(sim_index, sweep_config.get_num_simulations(), param_value, Some(total_retries_for_simulation)));
                        continue;
                    }

                    // Save this run's results to its own directory
                    let run_dir = format!("simulator/results/{}/data/sim_{}/run_{}", self.results_dir, sim_index, run - 1);
                    let save_result = results.save_to_directory(&run_dir).await;
                    
                    if let Err(e) = save_result {
                        retry_count += 1;
                        total_retries_for_simulation += 1;
                        logging::log("SIMULATOR", &format!("DEBUG: Save failed. retry_count: {}, total_retries: {}, max_retries: {}", retry_count, total_retries_for_simulation, max_retries));
                        if total_retries_for_simulation >= max_retries {
                            let error_context = format!(
                                "Sweep '{}' failed to save results for simulation {}/{} run {}/{} with {}: {:?} after {} total retries. Error: {}",
                                self.sweep_name,
                                sim_index + 1,
                                sweep_config.get_num_simulations(),
                                run,
                                num_runs,
                                self.parameter_name,
                                param_value,
                                total_retries_for_simulation,
                                e
                            );
                            return Err(crate::config::ConfigError::ValidationError(error_context));
                        }
                        logging::log("SIMULATOR", &format!("Result saving failed, retrying... (attempt {}/{})", retry_count, max_retries));
                        // Update progress message to show retry count
                        progress_bar.set_message(self.format_progress_message(sim_index, sweep_config.get_num_simulations(), param_value, Some(total_retries_for_simulation)));
                        continue;
                    }

                    // Success! Break out of retry loop
                    parameter_results.push(results);
                    
                    let completion_message = if retry_count > 0 {
                        format!("=== Completed Run {}/{} for parameter {}: {:?} (after {} retries) ===", 
                            run, num_runs, self.parameter_name, param_value, retry_count)
                    } else {
                        format!("=== Completed Run {}/{} for parameter {}: {:?} ===", 
                            run, num_runs, self.parameter_name, param_value)
                    };
                    logging::log("SIMULATOR", &completion_message);
                    break;
                }
            }

            // Use the first run's results for the sweep summary (individual runs are saved separately)
            all_results.push((param_value.clone(), parameter_results[0].clone()));
            
            // Update progress bar and show completed simulation with retry count
            progress_bar.inc(1);
            progress_bar.set_message(self.format_progress_message(sim_index, sweep_config.get_num_simulations(), param_value, Some(total_retries_for_simulation)));
        }

        // Finish progress bar with final state
        progress_bar.finish_with_message(self.format_progress_message(
            sweep_config.get_num_simulations() - 1, 
            sweep_config.get_num_simulations(), 
            self.parameter_values.last().unwrap(),
            None
        ));
        
        println!("Sweep simulation complete");

        // Save combined results
        (self.result_saver)(&self.results_dir, &all_results)?;

        logging::log("SIMULATOR", "=== Sweep Simulation Complete ===");
        logging::log("SIMULATOR", &format!("Total simulations completed: {}", all_results.len()));

        Ok(())
    }

    // ------------------------------------------------------------------------------------------------
    // Setup and Utility Methods
    // ------------------------------------------------------------------------------------------------

    /// Creates the necessary directories for storing sweep results.
    /// 
    /// This method creates the main results directory and subdirectories for data and figures.
    /// The directory structure follows the pattern: `simulator/results/{sweep_name}/{data|figs}/`
    fn create_directories(&self) {
        fs::create_dir_all(&format!("simulator/results/{}", self.results_dir))
            .expect("Failed to create results directory");
        fs::create_dir_all(&format!("simulator/results/{}/data", self.results_dir))
            .expect("Failed to create data directory");
        fs::create_dir_all(&format!("simulator/results/{}/figs", self.results_dir))
            .expect("Failed to create figures directory");
    }

    /// Sets up logging for the sweep simulation.
    /// 
    /// This method configures logging based on the configuration file.
    /// It creates a simulation-specific log file and initializes the logging system
    /// with appropriate configuration for the sweep.
    fn setup_logging(&self, config: &crate::config::Config) {
        // Delete existing log file if it exists and logging is enabled
        if config.logging_config.log_to_file {
            let log_path = format!("simulator/results/{}/simulation.log", self.results_dir);
            if let Err(e) = fs::remove_file(&log_path) {
                // Ignore error if file doesn't exist
                if e.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("Error deleting log file: {}", e);
                }
            }

            // Initialize logging with configuration
            logging::init_logging_with_config(
                true, // enabled
                true, // log_to_file
                Some(log_path)
            );
        } else {
            // Initialize logging with configuration (no file logging)
            logging::init_logging_with_config(
                false, // enabled
                false, // log_to_file
                None
            );
        }
    }

    /// Creates a progress bar for tracking sweep simulation progress.
    /// 
    /// This method creates a visual progress bar that shows the current simulation
    /// progress and estimated completion time. The progress bar displays the
    /// current simulation number and parameter value being tested.
    /// 
    /// # Arguments
    /// 
    /// * `num_simulations` - Total number of simulations to run in the sweep
    /// 
    /// # Returns
    /// 
    /// A configured ProgressBar instance ready for use
    fn create_progress_bar(&self, num_simulations: usize) -> ProgressBar {
        let progress_bar = ProgressBar::new(num_simulations as u64);
        progress_bar.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
            .unwrap()
            .progress_chars("+>-"));
        progress_bar
    }

    /// Logs the start of the sweep simulation with configuration details.
    /// 
    /// This method logs comprehensive information about the sweep including
    /// the sweep name, number of simulations, and all parameter values to be tested.
    /// This provides a clear record of what the sweep is testing.
    /// 
    /// # Arguments
    /// 
    /// * `sweep_config` - The sweep configuration containing simulation parameters
    fn log_sweep_start(&self, sweep_config: &Box<dyn SweepConfigTrait>) {
        logging::log("SIMULATOR", &format!("=== Sweep {} Simulation ===", self.sweep_name));
        logging::log("SIMULATOR", &format!("Number of simulations: {}", sweep_config.get_num_simulations()));
        logging::log("SIMULATOR", &format!("{} values: {:?}", self.parameter_name, self.parameter_values));
        logging::log("SIMULATOR", "================================");
    }

    /// Logs the start of an individual simulation within the sweep.
    /// 
    /// This method logs when a specific simulation begins, showing which simulation
    /// number is running out of the total and what parameter value is being tested.
    /// 
    /// # Arguments
    /// 
    /// * `sim_index` - Index of the current simulation (0-based)
    /// * `total_sims` - Total number of simulations in the sweep
    /// * `param_value` - The parameter value being tested in this simulation
    fn log_simulation_start(&self, sim_index: usize, total_sims: usize, param_value: &T) {
        logging::log("SIMULATOR", &format!("Running simulation {}/{} with {}: {:?}", 
            sim_index + 1, total_sims, self.parameter_name, param_value));
    }

    /// Formats a progress message for display in the progress bar.
    /// 
    /// This method creates a human-readable message showing the current simulation
    /// progress and the parameter value being tested. The message is used by the
    /// progress bar to show real-time status updates.
    /// 
    /// # Arguments
    /// 
    /// * `sim_index` - Index of the current simulation (0-based)
    /// * `total_sims` - Total number of simulations in the sweep
    /// * `param_value` - The parameter value being tested in this simulation
    /// * `retry_count` - Optional number of retries for the current simulation
    /// 
    /// # Returns
    /// 
    /// A formatted string describing the current simulation progress
    fn format_progress_message(&self, sim_index: usize, total_sims: usize, param_value: &T, retry_count: Option<usize>) -> String {
        let base_message = format!("Simulation {}/{} with {}: {:?}", 
            sim_index + 1, total_sims, self.parameter_name, param_value);
        
        if let Some(retries) = retry_count {
            if retries > 0 {
                format!("{} // Reattempts: {}", base_message, retries)
            } else {
                base_message
            }
        } else {
            base_message
        }
    }

    /// Initializes simulation results from the given configuration.
    /// 
    /// This method creates a new SimulationResults instance and populates it with
    /// configuration data. It also logs detailed configuration information for
    /// debugging and analysis purposes.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The configuration for this simulation
    /// * `sim_index` - Index of the current simulation (for logging)
    /// * `param_value` - The parameter value being tested (for logging)
    /// 
    /// # Returns
    /// 
    /// A SimulationResults instance initialized with configuration data
    fn initialize_simulation_results(&self, config: &crate::config::Config, sim_index: usize, param_value: &T) -> crate::SimulationResults {
        let mut results = crate::SimulationResults::default();
        results.initial_balance = config.account_config.initial_balance.try_into().unwrap();
        results.num_accounts = config.account_config.num_accounts.try_into().unwrap();
        results.target_tps = config.transaction_config.target_tps as u64;
        results.sim_total_block_number = config.simulation_config.sim_total_block_number.try_into().unwrap();
        results.zipf_parameter = config.transaction_config.zipf_parameter;
        results.ratio_cats = config.transaction_config.ratio_cats;
        results.block_interval = config.network_config.block_interval;
        results.cat_lifetime = config.transaction_config.cat_lifetime_blocks;
        results.initialization_wait_blocks = config.simulation_config.initialization_wait_blocks;
        results.chain_delays = config.network_config.chain_delays.clone();
        results.start_time = Instant::now();

        // Log configuration
        let start_time = Local::now();
        logging::log("SIMULATOR", &format!("=== Simulation {} Configuration ===", sim_index + 1));
        logging::log("SIMULATOR", &format!("Start Time: {}", start_time.format("%Y-%m-%d %H:%M:%S")));
        logging::log("SIMULATOR", &format!("{}: {:?}", self.parameter_name, param_value));
        logging::log("SIMULATOR", &format!("Initial Balance: {}", config.account_config.initial_balance));
        logging::log("SIMULATOR", &format!("Number of Accounts: {}", config.account_config.num_accounts));
        logging::log("SIMULATOR", &format!("Target TPS: {}", config.transaction_config.target_tps));
        logging::log("SIMULATOR", &format!("Simulation Total Blocks: {}", config.simulation_config.sim_total_block_number));
        logging::log("SIMULATOR", &format!("Number of Chains: {}", config.network_config.num_chains));
        logging::log("SIMULATOR", &format!("Zipf Parameter: {}", config.transaction_config.zipf_parameter));
        logging::log("SIMULATOR", &format!("CAT Ratio: {}", config.transaction_config.ratio_cats));
        logging::log("SIMULATOR", &format!("CAT Lifetime: {} blocks", results.cat_lifetime));
        logging::log("SIMULATOR", &format!("Initialization Wait Blocks: {}", results.initialization_wait_blocks));
        logging::log("SIMULATOR", &format!("Max Retries: {}", config.simulation_config.max_retries));
        for (i, delay) in config.network_config.chain_delays.iter().enumerate() {
            logging::log("SIMULATOR", &format!("Chain {} Delay: {} blocks", i + 1, delay));
        }
        logging::log("SIMULATOR", "=============================");

        results
    }
}

// ------------------------------------------------------------------------------------------------
// Helper Functions
// ------------------------------------------------------------------------------------------------

/// Generic function to save sweep results to JSON format.
/// 
/// This function takes the results from all simulations in a sweep and saves them
/// in a structured JSON format. It creates both a summary of all simulations and
/// detailed results for each individual simulation.
/// 
/// The JSON output includes:
/// - Sweep summary with parameter values and transaction counts
/// - Individual results for each simulation with detailed metrics
/// - Chain-specific transaction statistics
/// 
/// # Arguments
/// 
/// * `results_dir` - Directory where results should be saved
/// * `parameter_name` - Name of the parameter being swept (used for JSON field names)
/// * `all_results` - Vector of tuples containing (parameter_value, simulation_results)
/// 
/// # Returns
/// 
/// Result indicating success or failure of saving the results
pub fn save_generic_sweep_results<T: serde::Serialize>(
    results_dir: &str,
    parameter_name: &str,
    all_results: &[(T, crate::SimulationResults)]
) -> Result<(), crate::config::ConfigError> {
    // Map parameter names to the expected JSON field names for each sweep type
    let json_field_name = match parameter_name {
        "cat_ratio" => "cat_ratios",
        "chain_delay" => "chain_delays", 
        "duration" => "durations",
        "zipf_parameter" => "zipf_parameters",
        "cat_lifetime" => "cat_lifetimes",
        _ => parameter_name, // fallback to original name
    };

    // Create combined results structure
    let mut sweep_summary = serde_json::Map::new();
    sweep_summary.insert("num_simulations".to_string(), serde_json::to_value(all_results.len()).unwrap());
    sweep_summary.insert(json_field_name.to_string(), serde_json::to_value(all_results.iter().map(|(param, _)| param).collect::<Vec<_>>()).unwrap());
    sweep_summary.insert("total_transactions".to_string(), serde_json::to_value(all_results.iter().map(|(_, results)| results.transactions_sent).collect::<Vec<_>>()).unwrap());
    sweep_summary.insert("cat_transactions".to_string(), serde_json::to_value(all_results.iter().map(|(_, results)| results.cat_transactions).collect::<Vec<_>>()).unwrap());
    sweep_summary.insert("regular_transactions".to_string(), serde_json::to_value(all_results.iter().map(|(_, results)| results.regular_transactions).collect::<Vec<_>>()).unwrap());

    let combined_results = serde_json::json!({
        "sweep_summary": sweep_summary,
        "individual_results": all_results.iter().map(|(param, results)| {
            let mut json_obj = serde_json::Map::new();
            json_obj.insert(parameter_name.to_string(), serde_json::to_value(param).unwrap());
            json_obj.insert("total_transactions".to_string(), serde_json::to_value(results.transactions_sent).unwrap());
            json_obj.insert("cat_transactions".to_string(), serde_json::to_value(results.cat_transactions).unwrap());
            json_obj.insert("regular_transactions".to_string(), serde_json::to_value(results.regular_transactions).unwrap());
            json_obj.insert("chain_1_pending".to_string(), serde_json::to_value(&results.chain_1_pending).unwrap());
            json_obj.insert("chain_1_success".to_string(), serde_json::to_value(&results.chain_1_success).unwrap());
            json_obj.insert("chain_1_failure".to_string(), serde_json::to_value(&results.chain_1_failure).unwrap());
            json_obj.insert("chain_1_cat_pending".to_string(), serde_json::to_value(&results.chain_1_cat_pending).unwrap());
            json_obj.insert("chain_1_cat_success".to_string(), serde_json::to_value(&results.chain_1_cat_success).unwrap());
            json_obj.insert("chain_1_cat_failure".to_string(), serde_json::to_value(&results.chain_1_cat_failure).unwrap());
            json_obj.insert("chain_1_regular_pending".to_string(), serde_json::to_value(&results.chain_1_regular_pending).unwrap());
            json_obj.insert("chain_1_regular_success".to_string(), serde_json::to_value(&results.chain_1_regular_success).unwrap());
            json_obj.insert("chain_1_regular_failure".to_string(), serde_json::to_value(&results.chain_1_regular_failure).unwrap());
            json_obj.insert("chain_1_locked_keys".to_string(), serde_json::to_value(&results.chain_1_locked_keys).unwrap());
            json_obj.insert("chain_2_locked_keys".to_string(), serde_json::to_value(&results.chain_2_locked_keys).unwrap());
            serde_json::Value::Object(json_obj)
        }).collect::<Vec<_>>()
    });

    // Save combined results
    let combined_file = format!("simulator/results/{}/data/sweep_results.json", results_dir);
    fs::write(&combined_file, serde_json::to_string_pretty(&combined_results).expect("Failed to serialize combined results"))
        .map_err(|e| crate::config::ConfigError::ValidationError(e.to_string()))?;
    
    logging::log("SIMULATOR", &format!("Saved combined sweep results to {}", combined_file));

    Ok(())
} 

/// Helper function to create a config with a single field modified.
/// This reduces duplication across sweep implementations.
/// 
/// # Arguments
/// 
/// * `sweep_config` - The sweep configuration containing base parameters
/// * `field_updater` - A function that takes the base config and returns a modified config
/// 
/// # Returns
/// 
/// A new Config with the specified field modified
pub fn create_modified_config<F>(
    sweep_config: &Box<dyn SweepConfigTrait>,
    field_updater: F,
) -> crate::config::Config
where
    F: FnOnce(&crate::config::Config) -> crate::config::Config,
{
    // Create a base config from the sweep config
    let base_config = crate::config::Config {
        network_config: sweep_config.get_network_config().clone(),
        account_config: sweep_config.get_account_config().clone(),
        transaction_config: sweep_config.get_transaction_config().clone(),
        simulation_config: sweep_config.get_simulation_config().unwrap().clone(),
        logging_config: crate::config::LoggingConfig::default(),
    };
    
    // Apply the field updater to create the modified config
    field_updater(&base_config)
}

/// Helper function to generate a linear sequence of f64 values.
/// This reduces duplication in parameter generation across sweeps.
/// 
/// # Arguments
/// 
/// * `start` - The starting value
/// * `step` - The step size between values
/// * `count` - The number of values to generate
/// 
/// # Returns
/// 
/// A vector of f64 values in the sequence
pub fn generate_f64_sequence(start: f64, step: f64, count: usize) -> Vec<f64> {
    (0..count)
        .map(|i| start + (i as f64 * step))
        .collect()
}

/// Helper function to generate a linear sequence of u64 values.
/// This reduces duplication in parameter generation across sweeps.
/// 
/// # Arguments
/// 
/// * `start` - The starting value
/// * `step` - The step size between values
/// * `count` - The number of values to generate
/// 
/// # Returns
/// 
/// A vector of u64 values in the sequence
pub fn generate_u64_sequence(start: u64, step: u64, count: usize) -> Vec<u64> {
    (0..count)
        .map(|i| start + (i as u64 * step))
        .collect()
}

// All sweep configs now have their own trait implementations in their respective files 