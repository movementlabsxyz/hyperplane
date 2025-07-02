#!/usr/bin/env python3
"""
Plotting script for CAT Pending Dependencies Sweep Simulation

This script generates plots for the CAT pending dependencies sweep using the generic
plotting utilities to eliminate code duplication.
"""

import sys
import os

# Add the scripts directory to the Python path to import plot_utils
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from plot_utils import generate_all_plots

def main():
    """Main function"""
    # Configuration for this specific sweep
    results_path = 'simulator/results/sim_sweep_cat_pending_dependencies/data/sweep_results.json'
    param_name = 'allow_cat_pending_dependencies'
    results_dir = 'simulator/results/sim_sweep_cat_pending_dependencies'
    sweep_type = 'CAT Pending Dependencies'
    
    # Generate all plots using the generic utility
    generate_all_plots(results_path, param_name, results_dir, sweep_type)

if __name__ == "__main__":
    main() 