use std::env;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json;

/// Generic sweep runner that eliminates duplication across sweep simulations
pub struct SweepRunner<T> {
    sweep_name: String,
    results_dir: String,
    parameter_name: String,
    parameter_values: Vec<T>,
    config_loader: Box<dyn Fn() -> Result<Box<dyn SweepConfigTrait>, crate::config::ConfigError>>,
    config_modifier: Box<dyn Fn(&Box<dyn SweepConfigTrait>, T) -> crate::config::Config>,
    result_saver: Box<dyn Fn(&str, &[(T, crate::SimulationResults)]) -> Result<(), crate::config::ConfigError>>,
}

/// Trait for sweep configurations to allow generic handling
pub trait SweepConfigTrait {
    fn get_num_simulations(&self) -> usize;
    fn get_network(&self) -> &crate::config::NetworkConfig;
    fn get_num_accounts(&self) -> &crate::config::AccountConfig;
    fn get_transactions(&self) -> &crate::config::TransactionConfig;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Implementation for different sweep config types
impl SweepConfigTrait for crate::config::SweepConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network(&self) -> &crate::config::NetworkConfig { &self.network_config }
    fn get_num_accounts(&self) -> &crate::config::AccountConfig { &self.account_config }
    fn get_transactions(&self) -> &crate::config::TransactionConfig { &self.transaction_config }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl SweepConfigTrait for crate::config::SweepZipfConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network(&self) -> &crate::config::NetworkConfig { &self.network }
    fn get_num_accounts(&self) -> &crate::config::AccountConfig { &self.num_accounts }
    fn get_transactions(&self) -> &crate::config::TransactionConfig { &self.transactions }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl SweepConfigTrait for crate::config::SweepChainDelayConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network(&self) -> &crate::config::NetworkConfig { &self.network }
    fn get_num_accounts(&self) -> &crate::config::AccountConfig { &self.num_accounts }
    fn get_transactions(&self) -> &crate::config::TransactionConfig { &self.transactions }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl SweepConfigTrait for crate::config::SweepDurationConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network(&self) -> &crate::config::NetworkConfig { &self.network }
    fn get_num_accounts(&self) -> &crate::config::AccountConfig { &self.num_accounts }
    fn get_transactions(&self) -> &crate::config::TransactionConfig { &self.transactions }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl SweepConfigTrait for crate::config::SweepCatLifetimeConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network(&self) -> &crate::config::NetworkConfig { &self.network }
    fn get_num_accounts(&self) -> &crate::config::AccountConfig { &self.num_accounts }
    fn get_transactions(&self) -> &crate::config::TransactionConfig { &self.transactions }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl<T: std::fmt::Debug + Clone> SweepRunner<T> {
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

    /// Runs the complete sweep simulation
    pub async fn run(&self) -> Result<(), crate::config::ConfigError> {
        // Create results directory if it doesn't exist
        self.create_directories();
        
        // Setup logging
        self.setup_logging();

        // Load sweep configuration
        let sweep_config = (self.config_loader)()?;

        // Log sweep start
        self.log_sweep_start(&sweep_config);

        // Display sweep name before progress bar
        println!("Running Sweep: {}", self.sweep_name);

        // Create progress bar for sweep
        let progress_bar = self.create_progress_bar(sweep_config.get_num_simulations());

        // Store results for each simulation
        let mut all_results = Vec::new();

        // Run each simulation with different parameter value
        for (sim_index, param_value) in self.parameter_values.iter().enumerate() {
            self.log_simulation_start(sim_index, sweep_config.get_num_simulations(), param_value);

            // Create a modified config with the current parameter value
            let sim_config = (self.config_modifier)(&sweep_config, param_value.clone());

            // Initialize simulation results
            let mut results = self.initialize_simulation_results(&sim_config, sim_index, param_value);

            // Setup test nodes
            let (_hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = crate::testnodes::setup_test_nodes(
                Duration::from_secs_f64(sim_config.network_config.block_interval),
                &sim_config.network_config.chain_delays,
                sim_config.transaction_config.allow_cat_pending_dependencies,
                sim_config.transaction_config.cat_lifetime_blocks,
            ).await;
            
            // Initialize accounts with initial balance
            crate::network::initialize_accounts(
                &[cl_node.clone()], 
                sim_config.account_config.initial_balance.try_into().unwrap(), 
                sim_config.account_config.num_accounts.try_into().unwrap(),
                Some(&[hig_node_1.clone(), hig_node_2.clone()]),
                sim_config.network_config.block_interval,
            ).await.map_err(|e| {
                let error_context = format!(
                    "Sweep '{}' failed during simulation {}/{} with {}: {:?}. Error: {}",
                    self.sweep_name,
                    sim_index + 1,
                    sweep_config.get_num_simulations(),
                    self.parameter_name,
                    param_value,
                    e
                );
                crate::config::ConfigError::ValidationError(error_context)
            })?;

            // Run simulation
            crate::run_simulation::run_simulation(
                cl_node,
                vec![hig_node_1, hig_node_2],
                &mut results,
            ).await.map_err(|e| {
                let error_context = format!(
                    "Sweep '{}' failed during simulation {}/{} with {}: {:?}. Error: {}",
                    self.sweep_name,
                    sim_index + 1,
                    sweep_config.get_num_simulations(),
                    self.parameter_name,
                    param_value,
                    e
                );
                crate::config::ConfigError::ValidationError(error_context)
            })?;

            // Save individual simulation results
            results.save_to_directory(&format!("simulator/results/{}/data/sim_{}", self.results_dir, sim_index))
                .await.map_err(|e| {
                    let error_context = format!(
                        "Sweep '{}' failed to save results for simulation {}/{} with {}: {:?}. Error: {}",
                        self.sweep_name,
                        sim_index + 1,
                        sweep_config.get_num_simulations(),
                        self.parameter_name,
                        param_value,
                        e
                    );
                    crate::config::ConfigError::ValidationError(error_context)
                })?;
            
            all_results.push((param_value.clone(), results));
            
            // Update progress bar and show completed simulation
            progress_bar.inc(1);
            progress_bar.set_message(self.format_progress_message(sim_index, sweep_config.get_num_simulations(), param_value));
        }

        // Finish progress bar with final state
        progress_bar.finish_with_message(self.format_progress_message(
            sweep_config.get_num_simulations() - 1, 
            sweep_config.get_num_simulations(), 
            self.parameter_values.last().unwrap()
        ));
        
        println!("Sweep simulation complete");

        // Save combined results
        (self.result_saver)(&self.results_dir, &all_results)?;

        logging::log("SIMULATOR", "=== Sweep Simulation Complete ===");
        logging::log("SIMULATOR", &format!("Total simulations completed: {}", all_results.len()));

        Ok(())
    }

    /// Creates the necessary directories for the sweep
    fn create_directories(&self) {
        fs::create_dir_all(&format!("simulator/results/{}", self.results_dir))
            .expect("Failed to create results directory");
        fs::create_dir_all(&format!("simulator/results/{}/data", self.results_dir))
            .expect("Failed to create data directory");
        fs::create_dir_all(&format!("simulator/results/{}/figs", self.results_dir))
            .expect("Failed to create figures directory");
    }

    /// Sets up logging if ENABLE_LOGS environment variable is set
    fn setup_logging(&self) {
        if env::var("ENABLE_LOGS").is_ok() {
            // Delete existing log file if it exists
            let log_path = format!("simulator/results/{}/simulation.log", self.results_dir);
            if let Err(e) = fs::remove_file(&log_path) {
                // Ignore error if file doesn't exist
                if e.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("Error deleting log file: {}", e);
                }
            }

            // Initialize logging with simulation-specific log file
            env::set_var("HYPERPLANE_LOGGING", "true");
            env::set_var("HYPERPLANE_LOG_TO_FILE", "true");
            env::set_var("HYPERPLANE_LOG_FILE", log_path);
            logging::init_logging();
        }
    }

    /// Creates a progress bar for the sweep
    fn create_progress_bar(&self, num_simulations: usize) -> ProgressBar {
        let progress_bar = ProgressBar::new(num_simulations as u64);
        progress_bar.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
            .unwrap()
            .progress_chars("+>-"));
        progress_bar
    }

    /// Logs the start of the sweep
    fn log_sweep_start(&self, sweep_config: &Box<dyn SweepConfigTrait>) {
        logging::log("SIMULATOR", &format!("=== Sweep {} Simulation ===", self.sweep_name));
        logging::log("SIMULATOR", &format!("Number of simulations: {}", sweep_config.get_num_simulations()));
        logging::log("SIMULATOR", &format!("{} values: {:?}", self.parameter_name, self.parameter_values));
        logging::log("SIMULATOR", "================================");
    }

    /// Logs the start of an individual simulation
    fn log_simulation_start(&self, sim_index: usize, total_sims: usize, param_value: &T) {
        logging::log("SIMULATOR", &format!("Running simulation {}/{} with {}: {:?}", 
            sim_index + 1, total_sims, self.parameter_name, param_value));
    }

    /// Formats the progress message
    fn format_progress_message(&self, sim_index: usize, total_sims: usize, param_value: &T) -> String {
        format!("Simulation {}/{} with {}: {:?}", 
            sim_index + 1, total_sims, self.parameter_name, param_value)
    }

    /// Initializes simulation results from configuration
    fn initialize_simulation_results(&self, config: &crate::config::Config, sim_index: usize, param_value: &T) -> crate::SimulationResults {
        let mut results = crate::SimulationResults::default();
        results.initial_balance = config.account_config.initial_balance.try_into().unwrap();
        results.num_accounts = config.account_config.num_accounts.try_into().unwrap();
        results.target_tps = config.transaction_config.target_tps as u64;
        results.sim_total_block_number = config.transaction_config.sim_total_block_number.try_into().unwrap();
        results.zipf_parameter = config.transaction_config.zipf_parameter;
        results.ratio_cats = config.transaction_config.ratio_cats;
        results.block_interval = config.network_config.block_interval;
        results.cat_lifetime = config.transaction_config.cat_lifetime_blocks;
        results.initialization_wait_blocks = config.transaction_config.initialization_wait_blocks;
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
        logging::log("SIMULATOR", &format!("Simulation Total Blocks: {}", config.transaction_config.sim_total_block_number));
        logging::log("SIMULATOR", &format!("Number of Chains: {}", config.network_config.num_chains));
        logging::log("SIMULATOR", &format!("Zipf Parameter: {}", config.transaction_config.zipf_parameter));
        logging::log("SIMULATOR", &format!("CAT Ratio: {}", config.transaction_config.ratio_cats));
        logging::log("SIMULATOR", &format!("CAT Lifetime: {} blocks", results.cat_lifetime));
        logging::log("SIMULATOR", &format!("Initialization Wait Blocks: {}", results.initialization_wait_blocks));
        for (i, delay) in config.network_config.chain_delays.iter().enumerate() {
            logging::log("SIMULATOR", &format!("Chain {} Delay: {} blocks", i + 1, delay));
        }
        logging::log("SIMULATOR", "=============================");

        results
    }
}

/// Generic function to save sweep results
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