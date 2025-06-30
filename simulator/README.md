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

## Account Selection Model

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

This will execute the simulation and generate results in `simulator/results/`.

When running with logs enabled, the simulator will write detailed logs to `simulation.log` in the project root directory. The logs include:

- Network setup and initialization
- Account creation and balance updates
- Transaction submission and processing
- Performance metrics and statistics

You can track the logs in real-time by running in a separate terminal:

```bash
tail -f simulation.log
```

## Configuration

You can modify the simulation parameters by editing the `config.toml` file.

## Architecture

The simulator is organized into several modules:

```
simulator/
├── src/                # Simulator core logic
│   ├── run_simulation.rs    # Core simulation logic and transaction processing
│   ├── simulation_results.rs # Results tracking and data collection
│   ├── config.rs            # Configuration management
│   ├── network.rs           # Node setup and chain registration
│   ├── account_selector.rs  # Account selection using Zipf distribution
│   └── bin/simulator.rs     # Main entry point
├── scripts/            # Plotting and analysis scripts
│   ├── plot_results.py      # Main plotting script
│   ├── plot_miscellaneous.py # Transaction status plots
│   └── plot_account_selection.py # Account selection plots
├── results/            # Generated results and figures
│   ├── data/               # JSON data files
│   └── figs/               # Generated plot images
└── config.toml         # Configuration file
```

## Notes on Visualizing Results

The simulator generates results in the `simulator/results` directory. To visualize these results:

1. Install Python dependencies:

```bash
pip3 install -r simulator/scripts/requirements.txt
```

2. Run the visualization script:

```bash
python3 simulator/scripts/plot_results.py
```

This will generate several plots:

- **Transaction Status Plots**: `tx_count_pending.png`, `tx_count_success.png`, `tx_count_failure.png` - Shows transaction counts over time for each status
- **Account Selection Plots**: Distribution of sender and receiver account selection
- **Parameter Tracking**: Simulation configuration and parameters used
