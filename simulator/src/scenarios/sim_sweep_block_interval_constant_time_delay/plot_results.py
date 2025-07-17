#!/usr/bin/env python3
"""
Plotting script for Block Interval Constant Time Delay Sweep Simulation

This script generates plots for the block interval sweep where the second chain
delay is constant in time (0.5 seconds) regardless of block interval.
"""

import sys
import os

# Add the scripts directory to the Python path to import plot_utils
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from plot_utils import generate_all_plots

def main():
    """Main function to generate plots for block interval sweep simulation (constant time delay)."""
    # Configuration for this specific sweep
    param_name = 'block_interval'
    results_dir = 'simulator/results/sim_sweep_block_interval_constant_time_delay'
    sweep_type = 'Block Interval (Constant Time Delay)'
    
    # Generate all plots using the generic utility
    # Data flow: run_average folders -> sweep_results_averaged.json -> plots
    generate_all_plots(results_dir, param_name, sweep_type)

if __name__ == "__main__":
    main() 