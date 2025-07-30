#!/usr/bin/env python3
"""
Plotting script for CAT Lifetime Sweep Simulation

This script generates plots for the CAT lifetime sweep using the generic
plotting utilities to eliminate code duplication.

Usage:
    python plot_results.py                    # Plot all simulations
    python plot_results.py 5                 # Plot only simulation 5 (0-indexed)
"""

import sys
import os
import json

# Add the scripts directory to the Python path to import plot_utils
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from plot_utils import generate_all_plots
from plot_utils_percentage import plot_transaction_percentage

def plot_specific_simulation(sim_number: int, results_dir: str, param_name: str, sweep_type: str):
    """Plot a specific simulation by number."""
    # Load metadata to get parameter values
    metadata_path = f'{results_dir}/data/metadata.json'
    with open(metadata_path, 'r') as f:
        metadata = json.load(f)
    
    if sim_number >= len(metadata['parameter_values']):
        print(f"Error: Simulation {sim_number} does not exist. Available simulations: 0-{len(metadata['parameter_values'])-1}")
        return
    
    param_value = metadata['parameter_values'][sim_number]
    print(f"Plotting simulation {sim_number} with {param_name} = {param_value}")
    
    # Load the specific simulation data
    sim_data_path = f'{results_dir}/data/sim_{sim_number}/run_average/simulation_stats.json'
    if not os.path.exists(sim_data_path):
        print(f"Error: No data found for simulation {sim_number}")
        return
    
    with open(sim_data_path, 'r') as f:
        sim_data = json.load(f)
    
    # Create individual results structure
    individual_results = [{
        param_name: param_value,
        **sim_data['results']
    }]
    
    # Create data structure for plotting
    data = {
        'individual_results': individual_results
    }
    
    # Create results directory
    os.makedirs(f'{results_dir}/figs', exist_ok=True)
    
    # Plot percentage plots for this specific simulation
    print("Generating percentage plots...")
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat', 'success')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat', 'failure')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat', 'pending')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'regular', 'success')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'regular', 'failure')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'regular', 'pending')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'sumtypes', 'success')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'sumtypes', 'failure')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'sumtypes', 'pending')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat_pending_resolving', 'pending')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat_pending_postponed', 'pending')
    
    print(f"Plots generated for simulation {sim_number} in {results_dir}/figs/")

def main():
    """Main function to generate plots for CAT lifetime sweep simulation."""
    # Configuration for this specific sweep
    param_name = 'cat_lifetime'
    results_dir = 'simulator/results/sim_sweep_cat_lifetime'
    sweep_type = 'CAT Lifetime'
    
    # Check if a specific simulation number was provided
    if len(sys.argv) > 1:
        try:
            sim_number = int(sys.argv[1])
            plot_specific_simulation(sim_number, results_dir, param_name, sweep_type)
        except ValueError:
            print("Error: Please provide a valid simulation number (integer)")
            print("Usage: python plot_results.py [simulation_number]")
    else:
        # Generate all plots using the generic utility
        # Data flow: run_average folders -> sweep_results_averaged.json -> plots
        generate_all_plots(results_dir, param_name, sweep_type)

if __name__ == "__main__":
    main() 