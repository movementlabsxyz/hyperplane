# Hyperplane Simulator

This simulator is designed to test the Hyperplane protocol under various conditions and configurations.

## Overview

The simulator creates a test environment with multiple chains and accounts, then runs transactions between them to measure system performance.

## Features

- Creates multiple chains with registered nodes
- Initializes accounts with configurable balances
- Generates transactions using a Zipf distribution
- Measures and reports performance metrics
- Tracks transaction status (pending, success, failure) over time
- Generates visualization plots for transaction analysis
- Configurable number of accounts and chains
- Adjustable transaction rates and types (CAT vs REGULAR)
- Detailed statistics and visualization

## Zipf-based Account Selection Model

The simulator uses a realistic account selection model where:

- **Senders** are selected randomly (uniform distribution)
- **Receivers** are selected using Zipf distribution

This models real-world scenarios where:

- Any account can initiate a transaction (random senders)
- Some accounts are more popular destinations than others (Zipf receivers)
- The popularity of receiving accounts follows a power law distribution

The Zipf parameter controls how skewed the distribution is - higher values mean more concentration of transactions to popular accounts.

## Transaction Symmetry

The simulator enforces transaction symmetry across chains:

- For regular transactions: The same transaction is submitted to both chains (chain-1 and chain-2) as separate CL transactions
- For CAT transactions: A single CL transaction is created with two constituent transactions (one for each chain)

This symmetry ensures that both chains process the same workload, making it easier to analyze performance and behavior.

## Usage

From the root directory of the repository:

```bash
./simulator/run.sh
```

This will start the interactive simulator interface.

The simple simulation will:

- Run the simulation with the current configuration
- Display progress bar and real-time output
- Automatically generate plots after completion
- Save results in `simulator/results/sim_simple/`

When running with logs enabled, the simulator will write detailed logs to `simulator/results/sim_simple/simulation.log`. The logs include:

- Network setup and initialization
- Account creation and balance updates
- Transaction submission and processing
- Performance metrics and statistics

You can track the logs in real-time by running in a separate terminal:

```bash
tail -f simulator/results/sim_simple/simulation.log
```

## Running Plots After Simulation

If you want to regenerate plots after a simulation has been completed, you can run the plotting scripts directly from the root directory. For example

```bash
python3 simulator/src/scenarios/sim_simple/plot_results.py
```

## Configuration

You can modify the simulation parameters by editing the configuration files in `simulator/src/scenarios/`. The simulator supports multiple simulation types including simple simulations and various parameter sweep scenarios.



## Adding New Simulations

To add a new simulation to the simulator, follow these steps:

##### 1. `src/interface.rs` : Add Simulation Type
Add your simulation type to the `SimulationType` enum and update the `from_input()` method and menu text.

##### 2. `src/scenarios/mod.rs` : Add Module Declaration
Add the module declaration for your new simulation.

##### 3. `src/scenarios/sim_your_new_simulation.rs` : Create Simulation File
Create the simulation file with your simulation logic and a `register()` function.

##### 4. `src/scenarios/config_your_new_simulation.toml` : Create Configuration File
Create the configuration file with your simulation parameters.

##### 5. `src/scenarios/sim_your_new_simulation/plot_results.py` : Create Plot Script
Create the plot script directory and add the plotting script.

##### Reference Implementation
See `sim_sweep_zipf.rs` for a complete example of a sweep simulation with:
- Sweep-specific parameter struct definition
- Configuration loading and validation
- Parameter sequence generation  
- SweepRunner integration
- Proper error handling
- Registry registration

## Architecture

The simulator is organized into several modules:

```
simulator/
├── src/                    # Simulator core logic
│   ├── bin/
│   │   └── simulator.rs    # Main entry point with interactive interface
│   ├── scenarios/          # Simulation scenarios and configurations
│   │   ├── mod.rs          # Scenario module declarations
│   │   ├── sim_simple.rs   # Simple simulation implementation
│   │   ├── sweep_runner.rs # Generic sweep simulation runner
│   │   ├── run_all_tests.rs # Test orchestration
│   │   ├── config_simple.toml # Configuration for simple simulation
│   │   └── config_*.toml   # Configuration files for various sweep scenarios
│   ├── interface.rs        # Interface system for simulation selection
│   ├── simulation_registry.rs # Central registry for all simulations
│   ├── run_simulation.rs   # Core simulation logic and transaction processing
│   ├── simulation_results.rs # Results tracking and data collection
│   ├── config.rs           # Configuration management and validation
│   ├── network.rs          # Node setup and chain registration
│   ├── zipf_account_selection.rs # Account selection using Zipf distribution
│   ├── account_selection.rs # Account selection statistics tracking
│   ├── stats.rs            # Performance statistics collection
│   ├── logging.rs          # Logging utilities
│   └── lib.rs              # Module declarations and exports
├── scripts/                # Plotting and analysis scripts
│   ├── plot_utils.py       # Common plotting utilities
│   ├── sim_simple/         # Simple simulation plotting scripts
│   └── sim_sweep_*/        # Sweep simulation plotting scripts
├── results/                # Generated results and figures
└── run.sh                  # Launch script
```


