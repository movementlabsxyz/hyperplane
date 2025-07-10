# Simple Repeated Simulation

This scenario runs the simple simulation multiple times and averages the results to provide more statistically reliable data.

## Overview

The simple repeated simulation:
1. Runs the basic simulation multiple times with the same parameters
2. Saves each individual run's data separately in `run_0/`, `run_1/`, etc.
3. Provides a separate averaging script to combine results
4. Generates plots from the averaged data

## Workflow

### 1. Run the Simulation
```bash
# From the simulator directory
cargo run --bin simulator
# Select "Simple Repeated" from the menu
```

This will:
- Run the simulation multiple times (configurable in `config.toml`)
- Save each run's data in separate directories: `run_0/`, `run_1/`, etc.
- Create a `metadata.json` file with simulation parameters and run information

### 2. Generate Plots (Averaging is Automatic)
```bash
# From the sim_simple_repeated directory
python3 plot_results.py
```

This will:
- **Automatically run averaging** if averaged data doesn't exist
- Read averaged data from `run_average/` directory
- Generate all plots with "- Averaged" in the titles
- Save plots in `figs/` directory

### Manual Averaging (Optional)
If you want to run averaging separately:
```bash
python3 average_runs.py
```

## Configuration

Edit `config.toml` to configure:
- `num_runs`: Number of times to run the simulation (default: 5)
- All other simulation parameters (same as simple simulation)

## Data Structure

### Individual Runs
```
data/
├── metadata.json              # Simulation parameters and run info
├── run_0/                     # First run data
│   ├── simulation_stats.json
│   ├── pending_transactions_chain_1.json
│   ├── success_transactions_chain_1.json
│   └── ... (all data files)
├── run_1/                     # Second run data
│   └── ... (same structure)
└── ... (more runs)
```

### Averaged Data
```
data/
└── run_average/               # Created by average_runs.py
    ├── simulation_stats.json  # Averaged statistics
    ├── pending_transactions_chain_1.json
    ├── success_transactions_chain_1.json
    └── ... (all averaged data files)
```

## Files

- `simulation.rs`: Rust simulation implementation
- `config.toml`: Configuration file
- `average_runs.py`: Script to average individual run data
- `plot_results.py`: Main plotting script (uses averaged data)
- `plot_miscellaneous.py`: Transaction plotting functions
- `plot_account_selection.py`: Account selection distribution plots
- `test_averaging.py`: Test script to verify workflow
- `README.md`: This documentation

## Benefits

1. **Statistical Reliability**: Multiple runs provide more reliable results
2. **Data Preservation**: Individual run data is preserved for detailed analysis
3. **Flexible Analysis**: Can analyze individual runs or averaged results
4. **Float Precision**: Averaged results maintain decimal precision
5. **Separation of Concerns**: Rust handles simulation, Python handles averaging and plotting

## Testing

Run the test script to verify everything is set up correctly:
```bash
python3 test_averaging.py
```

This will check for:
- Required files and directories
- Data structure integrity
- Workflow readiness 