#!/usr/bin/env python3
"""
Plotting script for Chain Delay Sweep Simulation

This script generates plots for the chain delay sweep using the generic
plotting utilities to eliminate code duplication.
"""

import sys
import os

# Add the scripts directory to the Python path to import plot_utils
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from plot_utils import generate_all_plots

def main():
    """Main function to generate plots for chain delay sweep simulation."""
    # Configuration for this specific sweep
    param_name = 'chain_delay'
    results_dir = 'simulator/results/sim_sweep_chain_delay'
    sweep_type = 'Chain Delay'
    
    # Generate all plots using the generic utility
    # Data flow: run_average folders -> sweep_results_averaged.json -> plots
    generate_all_plots(results_dir, param_name, sweep_type)

if __name__ == "__main__":
    main() 