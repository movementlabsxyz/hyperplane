# Hyperplane Simulator

This simulator is designed to test the Hyperplane protocol under various conditions and configurations.

## Overview

The simulator creates a test environment with multiple chains and accounts, then spams transactions.


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
- Save results in `simulator/results/`

When running with logs enabled, the simulator will write detailed logs. But be careful, these logs are memory intensive.

You can track the logs in real-time by running in a separate terminal:

```bash
tail -f simulator/results/<simulation_type>/simulation.log
```

## Configuration

You can modify the simulation parameters by editing the configuration files in `simulator/src/scenarios/`. The simulator supports multiple simulation types including simple simulations and various parameter sweep scenarios.

## Features

- Creates multiple chains with registered nodes
- Initializes accounts with configurable balances
- Generates transactions with selection of receivers using a Zipf distribution
- Measures and reports performance metrics (tps, pending transactions, success, failure) over time
- Generates visualization plots for transaction analysis

For an introduction to the plots, see the [sim_simple](./src/scenarios/sim_simple/README.md) scenario.

#### Zipf-based Account Selection Model

The simulator uses a account selection model where:

- **Senders** are selected randomly (uniform distribution)
- **Receivers** are selected using Zipf distribution

This models real-world scenarios where:

- Any account can initiate a transaction (random senders)
- Some accounts are more popular destinations than others (Zipf receivers)
- The popularity of receiving accounts follows a power law distribution

The Zipf parameter controls how skewed the distribution is - higher values mean more concentration of transactions to popular accounts.

For more details, see the [sim_simple](./src/scenarios/sim_simple/README.md) scenario.

#### Transaction Symmetry

The simulator enforces transaction symmetry across chains:

- For regular transactions: The same transaction is submitted to both chains (chain-1 and chain-2) as separate CL transactions
- For CAT transactions: A single CL transaction is created with two constituent transactions (one for each chain)

This symmetry ensures that both chains process the same workload, making it easier to analyze performance and behavior.

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
