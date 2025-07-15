#!/usr/bin/env python3
"""
Plotting script for Block Interval All Scaled Sweep Simulation

This script generates plots for the block interval sweep where TPS is scaled
to maintain constant transactions per block across all simulations.
"""

import sys
import os

# Add the scripts directory to the Python path to import plot_utils
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from plot_utils import generate_all_plots

def main():
    """Main function to generate plots for block interval sweep simulation (all scaled)."""
    # Configuration for this specific sweep
    results_path = 'simulator/results/sim_sweep_block_interval_all_scaled/data/sweep_results.json'
    param_name = 'block_interval'
    results_dir = 'simulator/results/sim_sweep_block_interval_all_scaled'
    sweep_type = 'Block Interval (All Scaled)'
    
    # Generate all plots using the generic utility
    generate_all_plots(results_path, param_name, results_dir, sweep_type)

if __name__ == "__main__":
    main() 