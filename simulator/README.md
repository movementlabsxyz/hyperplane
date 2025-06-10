# Hyperplane Simulator

This simulator is designed to test the Hyperplane protocol under various conditions and configurations.

## Overview

The simulator creates a test environment with multiple chains and accounts, then runs transactions between them to measure system performance.

## Features

- Creates multiple chains with registered nodes
- Initializes accounts with configurable balances
- Generates transactions using a Zipf distribution
- Measures and reports performance metrics
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

## Usage

```bash
# Run without logs
cargo run -p simulator

# Run with detailed logging
ENABLE_LOGS=1 cargo run -p simulator
```

When running with logs enabled, the simulator will write detailed logs to `simulation.log` in the project root directory. The logs include:

- Network setup and initialization
- Account creation and balance updates
- Transaction submission and processing
- Performance metrics and statistics

You can track the logs in real-time by running in a separate terminal:

```bash
tail -f simulation.log
```

## Customization

You can modify the simulation parameters by editing the constants in `src/bin/simulator.rs`:

- `NUM_ACCOUNTS`: Number of accounts to create per chain
- `INITIAL_BALANCE`: Starting balance for each account
- `TARGET_TPS`: Target transactions per second
- `SIMULATION_DURATION`: How long to run the simulation (in seconds)

## Example Output

```
=== Simulation Results ===
Total Transactions: 600
Successful: 598 (99.67%)
Failed: 2 (0.33%)
Average TPS: 9.97
Total Duration: 60.12 seconds
```

## Architecture

The simulator is organized into several modules:

- `network.rs`: Handles node setup and chain registration
- `account_selector.rs`: Manages account selection using Zipf distribution
- `simulation.rs`: Core simulation logic and statistics tracking
- `bin/simulator.rs`: Main entry point and configuration

## Visualizing Results

The simulator generates results in the `simulator/results` directory. To visualize these results:

1. Install Python dependencies:

```bash
pip3 install -r simulator/scripts/requirements.txt
```

2. Run the visualization script:

```bash
python3 simulator/scripts/plot_results.py
```

## Configuration

The simulator can be configured through the `config.toml` file. Here are the available parameters:
