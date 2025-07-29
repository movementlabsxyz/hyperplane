#!/usr/bin/env python3
"""
Generic plotting utilities for Hyperplane simulator sweep results.

This module provides reusable plotting functions that can be used by all
sweep simulation plotting scripts to eliminate code duplication.
"""

import os
import sys
import json
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.colors import LinearSegmentedColormap
from typing import Dict, List, Tuple, Any, Optional

# Global colormap setting - easily switch between different colormaps
# Options: 'viridis', 'RdYlBu_r', 'plasma', 'inferno', 'magma', 'cividis'
COLORMAP = 'viridis'  # Change this to switch colormaps globally

# ------------------------------------------------------------------------------------------------
# Utility Functions
# ------------------------------------------------------------------------------------------------

# Global parameter display names to avoid duplication
PARAM_DISPLAY_NAMES = {
    'zipf_parameter': 'Zipf Parameter',
    'block_interval': 'Block Interval (seconds)',
    'cat_rate': 'CAT Rate',
    'chain_delay': 'Chain Delay (blocks)',
    'duration': 'Duration (blocks)',
    'cat_lifetime': 'CAT Lifetime (blocks)',
    'allow_cat_pending_dependencies': 'Allow CAT Pending Dependencies'
}

def create_color_gradient(num_simulations: int) -> np.ndarray:
    """Create a color gradient using the global COLORMAP setting"""
    return plt.cm.get_cmap(COLORMAP)(np.linspace(0, 1, num_simulations))

def create_sweep_data_from_averaged_runs(results_dir_name: str) -> str:
    """Create sweep data structure from run_average directories."""
    base_dir = f'simulator/results/{results_dir_name}/data'
    
    # Load metadata to get parameter values
    with open(f'{base_dir}/metadata.json', 'r') as f:
        metadata = json.load(f)
    
    param_values = metadata['parameter_values']
    param_name = metadata['parameter_name']
    
    # Create sweep summary
    sweep_summary = {
        'num_simulations': metadata['num_simulations'],
        param_name: param_values,
        'total_transactions': [],
        'cat_transactions': [],
        'regular_transactions': []
    }
    
    # Create individual results
    individual_results = []
    
    for sim_index, param_value in enumerate(param_values):
        # Load averaged stats for this simulation
        stats_file = f'{base_dir}/sim_{sim_index}/run_average/simulation_stats.json'
        if os.path.exists(stats_file):
            with open(stats_file, 'r') as f:
                stats = json.load(f)
            
            # Add to sweep summary
            sweep_summary['total_transactions'].append(stats['results']['total_transactions'])
            sweep_summary['cat_transactions'].append(stats['results']['cat_transactions'])
            sweep_summary['regular_transactions'].append(stats['results']['regular_transactions'])
            
            # Create individual result entry
            result_entry = {
                param_name: param_value,
                'total_transactions': stats['results']['total_transactions'],
                'cat_transactions': stats['results']['cat_transactions'],
                'regular_transactions': stats['results']['regular_transactions']
            }
            
            # Load time series data
            time_series_files = [
                ('pending_transactions_chain_1.json', 'chain_1_pending'),
                ('pending_transactions_chain_2.json', 'chain_2_pending'),
                ('success_transactions_chain_1.json', 'chain_1_success'),
                ('success_transactions_chain_2.json', 'chain_2_success'),
                ('failure_transactions_chain_1.json', 'chain_1_failure'),
                ('failure_transactions_chain_2.json', 'chain_2_failure'),
                ('cat_pending_transactions_chain_1.json', 'chain_1_cat_pending'),
                ('cat_pending_transactions_chain_2.json', 'chain_2_cat_pending'),
                ('cat_success_transactions_chain_1.json', 'chain_1_cat_success'),
                ('cat_success_transactions_chain_2.json', 'chain_2_cat_success'),
                ('cat_failure_transactions_chain_1.json', 'chain_1_cat_failure'),
                ('cat_failure_transactions_chain_2.json', 'chain_2_cat_failure'),
                ('cat_pending_resolving_transactions_chain_1.json', 'chain_1_cat_pending_resolving'),
                ('cat_pending_resolving_transactions_chain_2.json', 'chain_2_cat_pending_resolving'),
                ('cat_pending_postponed_transactions_chain_1.json', 'chain_1_cat_pending_postponed'),
                ('cat_pending_postponed_transactions_chain_2.json', 'chain_2_cat_pending_postponed'),
                ('regular_pending_transactions_chain_1.json', 'chain_1_regular_pending'),
                ('regular_pending_transactions_chain_2.json', 'chain_2_regular_pending'),
                ('regular_success_transactions_chain_1.json', 'chain_1_regular_success'),
                ('regular_success_transactions_chain_2.json', 'chain_2_regular_success'),
                ('regular_failure_transactions_chain_1.json', 'chain_1_regular_failure'),
                ('regular_failure_transactions_chain_2.json', 'chain_2_regular_failure'),
                ('locked_keys_chain_1.json', 'chain_1_locked_keys'),
                ('locked_keys_chain_2.json', 'chain_2_locked_keys'),
                ('tx_per_block_chain_1.json', 'chain_1_tx_per_block'),
                ('tx_per_block_chain_2.json', 'chain_2_tx_per_block'),
            ]
            
            for filename, key_name in time_series_files:
                file_path = f'{base_dir}/sim_{sim_index}/run_average/{filename}'
                if os.path.exists(file_path):
                    with open(file_path, 'r') as f:
                        data = json.load(f)
                        # Convert from dict format to list of tuples for plotting
                        if key_name in data:
                            time_series_data = []
                            for entry in data[key_name]:
                                time_series_data.append((entry['height'], entry['count']))
                            result_entry[key_name] = time_series_data
            
            individual_results.append(result_entry)
    
    # Create the complete data structure
    sweep_data = {
        'sweep_summary': sweep_summary,
        'individual_results': individual_results
    }
    
    # Save the combined data for plotting
    output_file = f'{base_dir}/sweep_results_averaged.json'
    with open(output_file, 'w') as f:
        json.dump(sweep_data, f, indent=2)
    
    return output_file

def load_sweep_data(results_path: str) -> Dict[str, Any]:
    """Load the combined sweep results data from a given path"""
    try:
        with open(results_path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        return {"individual_results": [], "sweep_summary": {}}
    except json.JSONDecodeError as e:
        print(f"Warning: Invalid JSON in sweep results file: {results_path}")
        print(f"Error: {e}")
        return {"individual_results": [], "sweep_summary": {}}

def extract_parameter_value(result: Dict[str, Any], param_name: str) -> float:
    """Extract parameter value from result dict"""
    return result[param_name]

def create_parameter_label(param_name: str, param_value: float) -> str:
    """Create a label for the parameter based on its name and value"""
    if param_name == 'zipf_parameter':
        return f'Zipf: {param_value:.3f}'
    elif param_name == 'block_interval':
        return f'Block Interval: {param_value:.3f}s'
    elif param_name == 'cat_rate':
        return f'CAT Rate: {param_value:.3f}'
    elif param_name == 'chain_delay':
        return f'Chain Delay: {param_value:.1f} blocks'
    elif param_name == 'duration':
        return f'Duration: {param_value:.0f} blocks'
    elif param_name == 'cat_lifetime':
        return f'CAT Lifetime: {param_value:.0f} blocks'
    else:
        return f'{param_name}: {param_value:.3f}'

def create_sweep_title(param_name: str, sweep_type: str) -> str:
    """Create a title for the sweep based on parameter name and type"""
    # Remove units from display name for titles
    param_display = PARAM_DISPLAY_NAMES.get(param_name, param_name.replace('_', ' ').title())
    # Remove units in parentheses for cleaner titles
    param_display = param_display.split(' (')[0]
    return f'{param_display} Sweep'

def trim_time_series_data(time_series_data: List[Tuple[int, int]], cutoff_percentage: float = 0.1) -> List[Tuple[int, int]]:
    """Trim the last cutoff_percentage of time series data to avoid edge effects"""
    if not time_series_data:
        return time_series_data
    
    # Calculate the cutoff point (remove last 10% by default)
    cutoff_index = int(len(time_series_data) * (1 - cutoff_percentage))
    
    # Return data up to the cutoff point
    return time_series_data[:cutoff_index]

# ------------------------------------------------------------------------------------------------
# Transaction Overlay Plotting
# ------------------------------------------------------------------------------------------------

def plot_transactions_overlay(
    data: Dict[str, Any],
    param_name: str,
    transaction_type: str,
    results_dir: str,
    sweep_type: str
) -> None:
    """Plot transaction overlay for a specific transaction type"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping {transaction_type} transactions plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's chain 1 transactions
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # For main transaction types (pending, success, failure), calculate as CAT + regular
            if transaction_type in ['pending', 'success', 'failure']:
                cat_data = result.get(f'chain_1_cat_{transaction_type}', [])
                regular_data = result.get(f'chain_1_regular_{transaction_type}', [])
                
                # Create a combined dataset by summing CAT and regular at each height
                combined_data = {}
                
                # Add CAT data
                for height, count in cat_data:
                    combined_data[height] = combined_data.get(height, 0) + count
                
                # Add regular data
                for height, count in regular_data:
                    combined_data[height] = combined_data.get(height, 0) + count
                
                # Convert back to sorted list of tuples
                chain_data = sorted(combined_data.items())
            else:
                # For CAT and regular specific types, use the data directly
                chain_data = result[f'chain_1_{transaction_type}']
            
            if not chain_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            chain_data = trim_time_series_data(chain_data, 0.1)
            
            if not chain_data:
                continue
                
            # Extract data - chain_data is a list of tuples (height, count)
            heights = [entry[0] for entry in chain_data]
            counts = [entry[1] for entry in chain_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, counts, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        # Create title and filename based on transaction type
        if transaction_type in ['pending', 'success', 'failure']:
            # Combined totals
            title = f'SumTypes {transaction_type.title()} Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}_sumTypes.png'
        elif transaction_type.startswith('cat_'):
            # CAT transactions
            status = transaction_type.replace('cat_', '')
            if status == 'pending_resolving':
                title = f'CAT Pending Resolving Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_resolving.png'
            elif status == 'pending_postponed':
                title = f'CAT Pending Postponed Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_postponed.png'
            else:
                title = f'CAT {status.title()} Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat.png'
        elif transaction_type.startswith('regular_'):
            # Regular transactions
            status = transaction_type.replace('regular_', '')
            title = f'Regular {status.title()} Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{status}_regular.png'
        else:
            # Fallback
            title = f'{transaction_type.title()} Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}.png'
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel(f'Number of {transaction_type.title()} Transactions')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing {transaction_type} transactions data: {e}")
        return

def plot_cat_pending_resolving(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending resolving transactions overlay"""
    plot_transactions_overlay(data, param_name, 'cat_pending_resolving', results_dir, sweep_type)

def plot_cat_pending_postponed(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending postponed transactions overlay"""
    plot_transactions_overlay(data, param_name, 'cat_pending_postponed', results_dir, sweep_type)

# ------------------------------------------------------------------------------------------------
# Summary Chart Plotting
# ------------------------------------------------------------------------------------------------

# Note: Transaction status chart functions removed - now using individual_curves_plots.py

# ------------------------------------------------------------------------------------------------
# Sweep Summary Plotting
# ------------------------------------------------------------------------------------------------

def plot_sweep_summary(
    data: Dict[str, Any],
    param_name: str,
    results_dir: str,
    sweep_type: str
) -> None:
    """Plot summary statistics across all parameter values"""
    try:
        sweep_summary = data['sweep_summary']
        
        if not sweep_summary:
            print("Warning: No sweep summary found, skipping summary plot")
            return
        
        # Handle both singular and plural parameter names
        param_key = param_name
        if param_key not in sweep_summary:
            # Try plural version
            param_key = f"{param_name}s"
            if param_key not in sweep_summary:
                print(f"Warning: Parameter {param_name} not found in sweep summary")
                return
        
        param_values = sweep_summary[param_key]
        total_transactions = sweep_summary['total_transactions']
        cat_transactions = sweep_summary['cat_transactions']
        regular_transactions = sweep_summary['regular_transactions']
        
        # Create subplots - 1x2 grid for summary analysis
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))
        
        xlabel = PARAM_DISPLAY_NAMES.get(param_name, param_name.replace('_', ' ').title())
        
        # Plot 1: Total transactions
        ax1.plot(param_values, total_transactions, 'bo-', linewidth=2, markersize=6)
        ax1.set_title(f'Total Transactions vs {xlabel}')
        ax1.set_xlabel(xlabel)
        ax1.set_ylabel('Total Transactions')
        ax1.grid(True, alpha=0.3)
        
        # Plot 2: Transaction type distribution
        ax2.plot(param_values, cat_transactions, 'ro-', linewidth=2, markersize=6, label='CAT Transactions')
        ax2.plot(param_values, regular_transactions, 'go-', linewidth=2, markersize=6, label='Regular Transactions')
        ax2.set_title(f'Transaction Distribution vs {xlabel}')
        ax2.set_xlabel(xlabel)
        ax2.set_ylabel('Number of Transactions')
        ax2.legend()
        ax2.grid(True, alpha=0.3)
        ax2.set_ylim(bottom=0)
        
        # Note: Individual transaction plots are now generated by generate_individual_curves_plots
        # instead of these summary charts to maintain consistency with simple simulation
        
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/sweep_summary.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing sweep summary data: {e}")
        return

# ------------------------------------------------------------------------------------------------
# Main Plot Generation
# ------------------------------------------------------------------------------------------------

def generate_all_plots(
    results_dir: str,
    param_name: str,
    sweep_type: str
) -> None:
    """
    Generate all plots for a sweep simulation.
    
    This function:
    1. Runs the averaging script to create run_average folders from individual runs
    2. Creates a combined sweep_results_averaged.json from the run_average data
    3. Generates all plots from the combined data
    
    Args:
        results_dir: The full path to the results directory (e.g., 'simulator/results/sim_sweep_cat_rate')
        param_name: The parameter name being swept (e.g., 'cat_rate')
        sweep_type: The display name for the sweep (e.g., 'CAT Rate')
    """
    import subprocess
    
    # Extract the results directory name from the full path
    results_dir_name = results_dir.replace('simulator/results/', '')
    
    # First, run the averaging script to create averaged data from run_average folders
    print("Running averaging script...")
    try:
        # The average_runs.py script is in simulator/src/
        average_script_path = os.path.join(os.path.dirname(__file__), '..', 'average_runs.py')
        # Use relative path from simulator directory (where the script runs from)
        results_path = f'results/{results_dir_name}'
        # Run from the simulator root directory
        simulator_root = os.path.join(os.path.dirname(__file__), '..', '..')
        result = subprocess.run([sys.executable, average_script_path, results_path], 
                               cwd=simulator_root, capture_output=True, text=True)
        
        if result.returncode == 0:
            print("Averaging completed successfully!")
        else:
            # Check if it's a "no data" error (metadata.json not found)
            # The error message can be in either stdout or stderr
            error_output = result.stdout + result.stderr
            if "metadata.json not found" in error_output:
                print("No simulation data found - skipping plotting")
                return
            else:
                print(f"Error running averaging script: {error_output}")
                return
    except Exception as e:
        print(f"Error during averaging: {e}")
        return
    
    # Create combined sweep data from run_average folders
    sweep_data_path = create_sweep_data_from_averaged_runs(results_dir_name)
    
    # Load the combined data for plotting
    data = load_sweep_data(sweep_data_path)
    
    # Check if we have any data to plot
    if not data.get('individual_results'):
        print(f"No data found for {sweep_type} simulation. Skipping plot generation.")
        return
    
    # Create results directory only if we have data
    os.makedirs(f'{results_dir}/figs', exist_ok=True)
    
    # Plot all transaction overlays (combined totals) - these show how transactions change across parameter values
    plot_transactions_overlay(data, param_name, 'pending', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'success', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'failure', results_dir, sweep_type)
    
    # Plot CAT transaction overlays
    plot_transactions_overlay(data, param_name, 'cat_pending', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'cat_success', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'cat_failure', results_dir, sweep_type)
    
    # Plot detailed CAT pending state overlays
    plot_cat_pending_resolving(data, param_name, results_dir, sweep_type)
    plot_cat_pending_postponed(data, param_name, results_dir, sweep_type)
    
    # Plot regular transaction overlays
    plot_transactions_overlay(data, param_name, 'regular_pending', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'regular_success', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'regular_failure', results_dir, sweep_type)
    
    # Plot sweep summary
    plot_sweep_summary(data, param_name, results_dir, sweep_type)
    
    # Plot locked keys data
    plot_sweep_locked_keys(data, param_name, results_dir, sweep_type)
    plot_sweep_locked_keys_with_pending(data, param_name, results_dir, sweep_type)
    
    # Plot transactions per block data
    plot_sweep_transactions_per_block(data, param_name, results_dir, sweep_type)
    
    # Note: Individual curves plots are now generated by generate_individual_curves_plots
    # instead of plot_individual_sweep_tps to maintain consistency with simple simulation
    
    # Plot system memory usage over time
    plot_system_memory(data, param_name, results_dir, sweep_type)
    
    # Plot system total memory usage over time
    plot_system_total_memory(data, param_name, results_dir, sweep_type)
    
    # Plot system CPU usage over time
    plot_system_cpu(data, param_name, results_dir, sweep_type)
    
    # Plot filtered system CPU usage over time (removes spikes above 30%)
    plot_system_cpu_filtered(data, param_name, results_dir, sweep_type)
    
    # Plot system total CPU usage over time
    plot_system_total_cpu(data, param_name, results_dir, sweep_type)
    
    # Plot loop steps without transaction issuance
    plot_loop_steps_without_tx_issuance(data, param_name, results_dir, sweep_type)
    
    # Plot CAT success percentage over time
    # Import and call from plot_utils_percentage.py
    from plot_utils_percentage import (
        plot_cat_success_percentage, plot_cat_failure_percentage, plot_cat_pending_percentage,
        plot_regular_success_percentage, plot_regular_failure_percentage, plot_regular_pending_percentage,
        plot_sumtypes_success_percentage, plot_sumtypes_failure_percentage, plot_sumtypes_pending_percentage,
        plot_cat_pending_resolving_percentage, plot_cat_pending_postponed_percentage
    )
    plot_cat_success_percentage(data, param_name, results_dir, sweep_type)
    plot_cat_failure_percentage(data, param_name, results_dir, sweep_type)
    plot_cat_pending_percentage(data, param_name, results_dir, sweep_type)
    plot_regular_success_percentage(data, param_name, results_dir, sweep_type)
    plot_regular_failure_percentage(data, param_name, results_dir, sweep_type)
    plot_regular_pending_percentage(data, param_name, results_dir, sweep_type)
    plot_sumtypes_success_percentage(data, param_name, results_dir, sweep_type)
    plot_sumtypes_failure_percentage(data, param_name, results_dir, sweep_type)
    plot_sumtypes_pending_percentage(data, param_name, results_dir, sweep_type)
    plot_cat_pending_resolving_percentage(data, param_name, results_dir, sweep_type)
    plot_cat_pending_postponed_percentage(data, param_name, results_dir, sweep_type)
    
    # Generate individual curves plots for each simulation in the sweep
    generate_individual_curves_plots(data, param_name, results_dir, sweep_type)
    
    print(f"{sweep_type} simulation plots generated successfully!") 

# ------------------------------------------------------------------------------------------------
# Locked Keys Plotting
# ------------------------------------------------------------------------------------------------

def plot_sweep_locked_keys(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot locked keys overlay for sweep simulations.
    
    # Arguments
    * `data` - The sweep data containing individual results
    * `param_name` - Name of the parameter being swept
    * `results_dir` - Directory name of the sweep (e.g., 'sim_sweep_cat_rate')
    * `sweep_type` - Type of sweep simulation
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping locked keys overlay plot")
            return
        
        # Create figure
        plt.figure(figsize=(12, 8))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Plot each simulation's chain 1 locked keys
        for i, result in enumerate(individual_results):
            # Get the parameter value (first key that's not a standard metric)
            param_value = None
            for key, value in result.items():
                if key not in ['total_transactions', 'cat_transactions', 'regular_transactions', 
                              'chain_1_pending', 'chain_1_success', 'chain_1_failure',
                              'chain_1_cat_pending', 'chain_1_cat_success', 'chain_1_cat_failure',
                              'chain_1_regular_pending', 'chain_1_regular_success', 'chain_1_regular_failure',
                              'chain_1_locked_keys', 'chain_2_locked_keys']:
                    param_value = value
                    break
            
            if param_value is None:
                continue
                
            chain_data = result.get('chain_1_locked_keys', [])
            
            if not chain_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            chain_data = trim_time_series_data(chain_data, 0.1)
            
            if not chain_data:
                continue
                
            # Extract data - chain_data is a list of tuples (height, count)
            heights = [entry[0] for entry in chain_data]
            counts = [entry[1] for entry in chain_data]
            
            # Plot with color based on parameter
            label = create_parameter_label(key, param_value)
            plt.plot(heights, counts, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Create title using the same pattern as other overlays
        title = f'Locked Keys by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel('Number of Locked Keys')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/locked_keys.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing sweep locked keys data for {results_dir}: {e}")
        return

def plot_sweep_locked_keys_with_pending(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot locked keys alongside pending transactions for sweep simulations.
    Creates a two-panel plot similar to the simple simulation's locked_keys_and_tx_pending.
    
    # Arguments
    * `data` - The sweep data containing individual results
    * `param_name` - Name of the parameter being swept
    * `results_dir` - Directory name of the sweep (e.g., 'sim_sweep_cat_rate')
    * `sweep_type` - Type of sweep simulation
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping locked keys with pending plot")
            return
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Create subplots
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 10), sharex=True)
        
        # Plot each simulation's data
        for i, result in enumerate(individual_results):
            # Get the parameter value
            param_value = None
            for key, value in result.items():
                if key not in ['total_transactions', 'cat_transactions', 'regular_transactions', 
                              'chain_1_pending', 'chain_1_success', 'chain_1_failure',
                              'chain_1_cat_pending', 'chain_1_cat_success', 'chain_1_cat_failure',
                              'chain_1_regular_pending', 'chain_1_regular_success', 'chain_1_regular_failure',
                              'chain_1_locked_keys', 'chain_2_locked_keys']:
                    param_value = value
                    break
            
            if param_value is None:
                continue
            
            # Get locked keys data
            locked_keys_data = result.get('chain_1_locked_keys', [])
            if not locked_keys_data:
                continue
            
            # Get CAT pending data
            cat_pending_data = result.get('chain_1_cat_pending', [])
            if not cat_pending_data:
                continue
            
            # Get regular pending data
            regular_pending_data = result.get('chain_1_regular_pending', [])
            if not regular_pending_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            locked_keys_data = trim_time_series_data(locked_keys_data, 0.1)
            cat_pending_data = trim_time_series_data(cat_pending_data, 0.1)
            regular_pending_data = trim_time_series_data(regular_pending_data, 0.1)
            
            if not locked_keys_data or not cat_pending_data or not regular_pending_data:
                continue
            
            # Extract data - all data is list of tuples (height, count)
            heights = [entry[0] for entry in locked_keys_data]
            locked_keys = [entry[1] for entry in locked_keys_data]
            cat_pending = [entry[1] for entry in cat_pending_data]
            regular_pending = [entry[1] for entry in regular_pending_data]
            
            # Create label
            label = create_parameter_label(param_name, param_value)
            
            # Plot locked keys vs CAT pending (top panel)
            ax1.plot(heights, locked_keys, color=colors[i], alpha=0.7, 
                    label=f'Locked Keys - {label}', linewidth=1.5)
            ax1.plot(heights, cat_pending, color=colors[i], alpha=0.7, 
                    linestyle='--', label=f'CAT Pending - {label}', linewidth=1.5)
            
            # Plot pending transactions breakdown (bottom panel)
            ax2.plot(heights, cat_pending, color=colors[i], alpha=0.7, 
                    label=f'CAT Pending - {label}', linewidth=1.5)
            ax2.plot(heights, regular_pending, color=colors[i], alpha=0.7, 
                    linestyle='--', label=f'Regular Pending - {label}', linewidth=1.5)
        
        # Set up top panel (locked keys vs CAT pending)
        ax1.set_ylabel('Count')
        ax1.set_title(f'Locked Keys vs Pending Transactions (Chain 1) - {create_sweep_title(param_name, sweep_type)}')
        ax1.grid(True, alpha=0.3)
        ax1.legend(loc="upper right")
        
        # Set up bottom panel (pending transactions breakdown)
        ax2.set_xlabel('Block Height')
        ax2.set_ylabel('Number of Pending Transactions')
        ax2.grid(True, alpha=0.3)
        ax2.legend(loc="upper right")
        
        plt.tight_layout()
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/locked_keys_and_tx_pending.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing sweep locked keys with pending data for {results_dir}: {e}")
        return 

def plot_sweep_transactions_per_block(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot transactions per block and TPS for sweep simulations.
    
    # Arguments
    * `data` - The sweep data containing individual results
    * `param_name` - Name of the parameter being swept
    * `results_dir` - Directory name of the sweep (e.g., 'sim_sweep_cat_rate')
    * `sweep_type` - Type of sweep simulation
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping transactions per block overlay plot")
            return
        
        # Create figure with subplots
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 10), sharex=True)
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Plot each simulation's data
        for i, result in enumerate(individual_results):
            # Get the parameter value
            param_value = None
            for key, value in result.items():
                if key not in ['total_transactions', 'cat_transactions', 'regular_transactions', 
                              'chain_1_pending', 'chain_1_success', 'chain_1_failure',
                              'chain_1_cat_pending', 'chain_1_cat_success', 'chain_1_cat_failure',
                              'chain_1_regular_pending', 'chain_1_regular_success', 'chain_1_regular_failure',
                              'chain_1_locked_keys', 'chain_2_locked_keys',
                              'chain_1_tx_per_block', 'chain_2_tx_per_block']:
                    param_value = value
                    break
            
            if param_value is None:
                continue
            
            # Get transactions per block data
            tx_per_block_data = result.get('chain_1_tx_per_block', [])
            if not tx_per_block_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            tx_per_block_data = trim_time_series_data(tx_per_block_data, 0.1)
            
            if not tx_per_block_data:
                continue
            
            # Extract data - data is list of tuples (height, count)
            heights = [entry[0] for entry in tx_per_block_data]
            tx_per_block = [entry[1] for entry in tx_per_block_data]
            
            # Calculate TPS using the parameter value (block_interval) for this simulation
            # For block interval sweeps, the parameter value IS the block interval
            if param_name == 'block_interval':
                block_interval = param_value
            else:
                # For other sweeps, try to get block_interval from simulation_stats.json
                try:
                    # Extract just the directory name from the full path
                    results_dir_name = results_dir.replace('simulator/results/', '')
                    # Use simulation_stats.json from the first simulation's run_average directory
                    stats_file = f'simulator/results/{results_dir_name}/data/sim_0/run_average/simulation_stats.json'
                    with open(stats_file, 'r') as f:
                        stats_data = json.load(f)
                    block_interval = stats_data['parameters']['block_interval']
                except (FileNotFoundError, KeyError) as e:
                    raise ValueError(f"Could not determine block_interval for TPS calculation. "
                                  f"Simulation stats file not found or missing block_interval parameter: {e}")
            
            tps = [tx_count / block_interval for tx_count in tx_per_block]
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            ax1.plot(heights, tx_per_block, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
            ax2.plot(heights, tps, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Create titles
        title = f'Transactions per Block (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
        ax1.set_title(title)
        ax1.set_ylabel('Number of Transactions')
        ax1.grid(True, alpha=0.3)
        ax1.legend(loc="upper right")
        
        ax2.set_title(f'Transactions per Second (Chain 1) - {create_sweep_title(param_name, sweep_type)}')
        ax2.set_xlabel('Block Height')
        ax2.set_ylabel('TPS')
        ax2.grid(True, alpha=0.3)
        ax2.legend(loc="upper right")
        
        plt.tight_layout()
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/tps.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error processing transactions per block data for sweep: {e}")
        return

def calculate_running_average(data: List[float], window_size: int = 10) -> List[float]:
    """
    Calculate a running average over a window of specified size.
    
    # Arguments
    * `data` - List of values to average
    * `window_size` - Size of the averaging window (default: 10)
    
    # Returns
    * List of averaged values (same length as input, with edge handling)
    """
    if len(data) < window_size:
        return data  # Return original data if too short
    
    averaged_data = []
    half_window = window_size // 2
    
    for i in range(len(data)):
        start = max(0, i - half_window)
        end = min(len(data), i + half_window + 1)
        window_data = data[start:end]
        averaged_data.append(sum(window_data) / len(window_data))
    
    return averaged_data

# Note: plot_individual_sweep_tps function removed - now using generate_individual_curves_plots
# The old function created sim_x/ directories which are no longer needed

# ------------------------------------------------------------------------------------------------
# Generic Sweep Plotting
# ------------------------------------------------------------------------------------------------

def plot_system_memory(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot system memory usage over time for each simulation"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping system memory plot")
            return
        
        # Create figure
        fig, ax = plt.subplots(figsize=(12, 8))
        
        # Get parameter values for coloring
        param_values = [result[param_name] for result in individual_results]
        colors = create_color_gradient(len(param_values))
        
        # Plot each simulation's system memory usage data
        missing_files = []
        for sim_index, (result, color) in enumerate(zip(individual_results, colors)):
            param_value = result[param_name]
            label = create_parameter_label(param_name, param_value)
            
            # Load system memory usage data for this simulation
            # Extract just the directory name from the full path
            results_dir_name = results_dir.replace('simulator/results/', '')
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            
            # Use averaged system memory usage data
            memory_file = f'{sim_data_dir}/run_average/system_memory.json'
            if os.path.exists(memory_file):
                with open(memory_file, 'r') as f:
                    memory_data = json.load(f)
                
                # Extract system memory usage data
                if 'system_memory' in memory_data:
                    memory_entries = memory_data['system_memory']
                    if memory_entries:
                        # Extract block heights and memory usage values
                        heights = [entry['height'] for entry in memory_entries]
                        memory_values = [entry['bytes'] / (1024 * 1024) for entry in memory_entries]  # Convert to MB
                        
                        # Ensure heights and memory_values have the same length
                        if len(heights) != len(memory_values):
                            print(f"Warning: Heights ({len(heights)}) and memory values ({len(memory_values)}) have different lengths for simulation {sim_index}")
                            # Use the shorter length
                            min_length = min(len(heights), len(memory_values))
                            heights = heights[:min_length]
                            memory_values = memory_values[:min_length]
                        
                        # Plot the averaged data directly (no additional smoothing needed)
                        ax.plot(heights, memory_values, color=color, alpha=0.7, linewidth=2)
                    else:
                        print(f"Warning: No system memory entries found for simulation {sim_index}")
                else:
                    print(f"Warning: No system_memory key found in {memory_file}")
            else:
                missing_files.append(memory_file)
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Print summary warning for missing files
        if missing_files:
            print(f"Warning: {len(missing_files)} system_memory.json files not found across all simulations")
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('System Memory Usage (MB)')
        ax.set_title(f'System Memory Usage Over Time by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(loc="upper right")
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/system_memory.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting system memory data: {e}")
        import traceback
        traceback.print_exc()

def plot_system_total_memory(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot system total RAM usage over time for each simulation"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping system total RAM plot")
            return
        
        # Create figure
        fig, ax = plt.subplots(figsize=(12, 8))
        
        # Get parameter values for coloring
        param_values = [result[param_name] for result in individual_results]
        colors = create_color_gradient(len(param_values))
        
        # Plot each simulation's system total RAM usage data
        missing_files = []
        for sim_index, (result, color) in enumerate(zip(individual_results, colors)):
            param_value = result[param_name]
            label = create_parameter_label(param_name, param_value)
            
            # Load system total RAM usage data for this simulation
            # Extract just the directory name from the full path
            results_dir_name = results_dir.replace('simulator/results/', '')
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            
            # Use averaged system total memory usage data
            system_total_memory_file = f'{sim_data_dir}/run_average/system_total_memory.json'
            if os.path.exists(system_total_memory_file):
                with open(system_total_memory_file, 'r') as f:
                    system_total_memory_data = json.load(f)
                
                # Extract system total memory usage data
                if 'system_total_memory' in system_total_memory_data:
                    system_total_memory_entries = system_total_memory_data['system_total_memory']
                    if system_total_memory_entries:
                        # Extract block heights and system total memory usage values
                        heights = [entry['height'] for entry in system_total_memory_entries]
                        system_total_memory_values = [entry['bytes'] / (1024 * 1024 * 1024) for entry in system_total_memory_entries]  # Convert to GB
                        
                        # Ensure heights and system_total_memory_values have the same length
                        if len(heights) != len(system_total_memory_values):
                            print(f"Warning: Heights ({len(heights)}) and system total memory values ({len(system_total_memory_values)}) have different lengths for simulation {sim_index}")
                            # Use the shorter length
                            min_length = min(len(heights), len(system_total_memory_values))
                            heights = heights[:min_length]
                            system_total_memory_values = system_total_memory_values[:min_length]
                        
                        # Plot the averaged data directly (no additional smoothing needed)
                        ax.plot(heights, system_total_memory_values, color=color, alpha=0.7, linewidth=2)
                    else:
                        print(f"Warning: No system total memory entries found for simulation {sim_index}")
                else:
                    print(f"Warning: No system_total_memory key found in {system_total_memory_file}")
            else:
                missing_files.append(system_total_memory_file)
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Print summary warning for missing files
        if missing_files:
            print(f"Warning: {len(missing_files)} system_total_memory.json files not found across all simulations")
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('System Total Memory Usage (GB)')
        ax.set_title(f'System Total Memory Usage Over Time by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(loc="upper right")
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/system_total_memory.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting system total memory data: {e}")
        import traceback
        traceback.print_exc()

def plot_system_cpu(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot system CPU usage over time for each simulation"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping system CPU plot")
            return
        
        # Create figure
        fig, ax = plt.subplots(figsize=(12, 8))
        
        # Get parameter values for coloring
        param_values = [result[param_name] for result in individual_results]
        colors = create_color_gradient(len(param_values))
        
        # Plot each simulation's system CPU usage data
        missing_files = []
        for sim_index, (result, color) in enumerate(zip(individual_results, colors)):
            param_value = result[param_name]
            label = create_parameter_label(param_name, param_value)
            
            # Load system CPU usage data for this simulation
            # Extract just the directory name from the full path
            results_dir_name = results_dir.replace('simulator/results/', '')
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            
            # Use averaged system CPU usage data
            cpu_file = f'{sim_data_dir}/run_average/system_cpu.json'
            if os.path.exists(cpu_file):
                with open(cpu_file, 'r') as f:
                    cpu_data = json.load(f)
                
                # Extract system CPU usage data
                if 'system_cpu' in cpu_data:
                    cpu_entries = cpu_data['system_cpu']
                    if cpu_entries:
                        # Extract block heights and CPU usage values
                        heights = [entry['height'] for entry in cpu_entries]
                        cpu_values = [entry['percent'] for entry in cpu_entries]  # Already in percent
                        
                        # Ensure heights and cpu_values have the same length
                        if len(heights) != len(cpu_values):
                            print(f"Warning: Heights ({len(heights)}) and CPU values ({len(cpu_values)}) have different lengths for simulation {sim_index}")
                            # Use the shorter length
                            min_length = min(len(heights), len(cpu_values))
                            heights = heights[:min_length]
                            cpu_values = cpu_values[:min_length]
                        
                        # Plot the averaged data directly (no additional smoothing needed)
                        ax.plot(heights, cpu_values, color=color, alpha=0.7, linewidth=2)
                    else:
                        print(f"Warning: No system CPU entries found for simulation {sim_index}")
                else:
                    print(f"Warning: No system_cpu key found in {cpu_file}")
            else:
                missing_files.append(cpu_file)
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Print summary warning for missing files
        if missing_files:
            print(f"Warning: {len(missing_files)} system_cpu.json files not found across all simulations")
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('System CPU Usage (%)')
        ax.set_title(f'System CPU Usage Over Time by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(loc="upper right")
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/system_cpu.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting system CPU data: {e}")
        import traceback
        traceback.print_exc()

def plot_system_cpu_filtered(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot system CPU usage over time for each simulation with spikes above 30% filtered out"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping filtered system CPU plot")
            return
        
        # Create figure
        fig, ax = plt.subplots(figsize=(12, 8))
        
        # Get parameter values for coloring
        param_values = [result[param_name] for result in individual_results]
        colors = create_color_gradient(len(param_values))
        
        # Plot each simulation's system CPU usage data
        missing_files = []
        for sim_index, (result, color) in enumerate(zip(individual_results, colors)):
            param_value = result[param_name]
            label = create_parameter_label(param_name, param_value)
            
            # Load system CPU usage data for this simulation
            # Extract just the directory name from the full path
            results_dir_name = results_dir.replace('simulator/results/', '')
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            
            # Use averaged system CPU usage data
            cpu_file = f'{sim_data_dir}/run_average/system_cpu.json'
            if os.path.exists(cpu_file):
                with open(cpu_file, 'r') as f:
                    cpu_data = json.load(f)
                
                # Extract system CPU usage data
                if 'system_cpu' in cpu_data:
                    cpu_entries = cpu_data['system_cpu']
                    if cpu_entries:
                        # Extract block heights and CPU usage values
                        heights = [entry['height'] for entry in cpu_entries]
                        cpu_values = [entry['percent'] for entry in cpu_entries]  # Already in percent
                        
                        # Filter out spikes above 30%
                        filtered_heights = []
                        filtered_cpu_values = []
                        for height, cpu_value in zip(heights, cpu_values):
                            if cpu_value <= 30.0:
                                filtered_heights.append(height)
                                filtered_cpu_values.append(cpu_value)
                        
                        # Ensure heights and cpu_values have the same length
                        if len(filtered_heights) != len(filtered_cpu_values):
                            print(f"Warning: Heights ({len(filtered_heights)}) and filtered CPU values ({len(filtered_cpu_values)}) have different lengths for simulation {sim_index}")
                            # Use the shorter length
                            min_length = min(len(filtered_heights), len(filtered_cpu_values))
                            filtered_heights = filtered_heights[:min_length]
                            filtered_cpu_values = filtered_cpu_values[:min_length]
                        
                        # Plot the filtered data
                        if filtered_heights and filtered_cpu_values:
                            ax.plot(filtered_heights, filtered_cpu_values, color=color, alpha=0.7, linewidth=2)
                    else:
                        print(f"Warning: No system CPU entries found for simulation {sim_index}")
                else:
                    print(f"Warning: No system_cpu key found in {cpu_file}")
            else:
                missing_files.append(cpu_file)
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Print summary warning for missing files
        if missing_files:
            print(f"Warning: {len(missing_files)} system_cpu.json files not found across all simulations")
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('System CPU Usage (%)')
        ax.set_title(f'System CPU Usage Over Time (Filtered ≤30%) by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(loc="upper right")
        ax.grid(True, alpha=0.3)
        

        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/system_cpu_filtered.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting filtered system CPU data: {e}")
        import traceback
        traceback.print_exc()

def plot_system_total_cpu(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot system total CPU usage over time for each simulation"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping system total CPU plot")
            return
        
        # Create figure
        fig, ax = plt.subplots(figsize=(12, 8))
        
        # Get parameter values for coloring
        param_values = [result[param_name] for result in individual_results]
        colors = create_color_gradient(len(param_values))
        
        # Plot each simulation's system total CPU usage data
        missing_files = []
        for sim_index, (result, color) in enumerate(zip(individual_results, colors)):
            param_value = result[param_name]
            label = create_parameter_label(param_name, param_value)
            
            # Load system total CPU usage data for this simulation
            # Extract just the directory name from the full path
            results_dir_name = results_dir.replace('simulator/results/', '')
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            
            # Use averaged system total CPU usage data
            cpu_file = f'{sim_data_dir}/run_average/system_total_cpu.json'
            if os.path.exists(cpu_file):
                with open(cpu_file, 'r') as f:
                    cpu_data = json.load(f)
                
                # Extract system total CPU usage data
                if 'system_total_cpu' in cpu_data:
                    cpu_entries = cpu_data['system_total_cpu']
                    if cpu_entries:
                        # Extract block heights and CPU usage values
                        heights = [entry['height'] for entry in cpu_entries]
                        cpu_values = [entry['percent'] for entry in cpu_entries]  # Already in percent
                        
                        # Ensure heights and cpu_values have the same length
                        if len(heights) != len(cpu_values):
                            print(f"Warning: Heights ({len(heights)}) and total CPU values ({len(cpu_values)}) have different lengths for simulation {sim_index}")
                            # Use the shorter length
                            min_length = min(len(heights), len(cpu_values))
                            heights = heights[:min_length]
                            cpu_values = cpu_values[:min_length]
                        
                        # Plot the averaged data directly (no additional smoothing needed)
                        ax.plot(heights, cpu_values, color=color, alpha=0.7, linewidth=2)
                    else:
                        print(f"Warning: No system total CPU entries found for simulation {sim_index}")
                else:
                    print(f"Warning: No system_total_cpu key found in {cpu_file}")
            else:
                missing_files.append(cpu_file)
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Print summary warning for missing files
        if missing_files:
            print(f"Warning: {len(missing_files)} system_total_cpu.json files not found across all simulations")
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('System Total CPU Usage (%)')
        ax.set_title(f'System Total CPU Usage Over Time by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(loc="upper right")
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/system_total_cpu.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting system total CPU data: {e}")
        import traceback
        traceback.print_exc()

def plot_loop_steps_without_tx_issuance(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot loop steps without transaction issuance over time for each simulation"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping loop steps plot")
            return
        
        # Create figure
        fig, ax = plt.subplots(figsize=(12, 8))
        
        # Get parameter values for coloring
        param_values = [result[param_name] for result in individual_results]
        colors = create_color_gradient(len(param_values))
        
        # Plot each simulation's loop steps data
        missing_files = []
        for sim_index, (result, color) in enumerate(zip(individual_results, colors)):
            param_value = result[param_name]
            label = create_parameter_label(param_name, param_value)
            
            # Load loop steps data for this simulation
            # Extract just the directory name from the full path
            results_dir_name = results_dir.replace('simulator/results/', '')
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            
            # Use averaged loop steps data
            loop_steps_file = f'{sim_data_dir}/run_average/loop_steps_without_tx_issuance.json'
            if os.path.exists(loop_steps_file):
                with open(loop_steps_file, 'r') as f:
                    loop_steps_data = json.load(f)
                
                # Extract loop steps data
                if 'loop_steps_without_tx_issuance' in loop_steps_data:
                    loop_steps_entries = loop_steps_data['loop_steps_without_tx_issuance']
                    if loop_steps_entries:
                        # Extract block heights and loop steps values
                        heights = [entry['height'] for entry in loop_steps_entries]
                        loop_steps_values = [entry['count'] for entry in loop_steps_entries]
                        
                        # Ensure heights and loop_steps_values have the same length
                        if len(heights) != len(loop_steps_values):
                            print(f"Warning: Heights ({len(heights)}) and loop steps values ({len(loop_steps_values)}) have different lengths for simulation {sim_index}")
                            # Use the shorter length
                            min_length = min(len(heights), len(loop_steps_values))
                            heights = heights[:min_length]
                            loop_steps_values = loop_steps_values[:min_length]
                        
                        # Plot the averaged data directly (no additional smoothing needed)
                        ax.plot(heights, loop_steps_values, color=color, alpha=0.7, linewidth=2)
                    else:
                        print(f"Warning: No loop steps entries found for simulation {sim_index}")
                else:
                    print(f"Warning: No loop_steps_without_tx_issuance key found in {loop_steps_file}")
            else:
                missing_files.append(loop_steps_file)
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Print summary warning for missing files
        if missing_files:
            print(f"Warning: {len(missing_files)} loop_steps_without_tx_issuance.json files not found across all simulations")
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('Loop Steps Count')
        ax.set_title(f'Loop Steps Without Transaction Issuance Over Time by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(loc="upper right")
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/loop_steps_without_tx_issuance.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting loop steps data: {e}")
        import traceback
        traceback.print_exc()

def generate_individual_curves_plots(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Generate individual curves plots for each simulation in the sweep.
    
    This function uses the individual_curves_plots module to create per-run plots
    for each simulation in the sweep, showing individual runs with different colors.
    
    Args:
        data: The sweep data containing individual results
        param_name: The parameter name being swept
        results_dir: The full path to the results directory
        sweep_type: The display name for the sweep
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping individual curves plots")
            return
        
        # Import the individual curves plotting module
        sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__))))
        from individual_curves_plots import create_per_run_plots
        
        # Extract the results directory name from the full path
        results_dir_name = results_dir.replace('simulator/results/', '')
        
        # Generate individual curves plots for each simulation
        for sim_index, result in enumerate(individual_results):
            print(f"Generating individual curves plots for simulation {sim_index}...")
            
            # Get the parameter value for this simulation
            param_value = result[param_name]
            
            # Set up paths for this simulation
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            sim_figs_dir = f'{results_dir}/figs/sim_{sim_index}'
            
            # Load block interval from simulation stats to calculate TPS
            block_interval = None
            try:
                stats_file = f'{sim_data_dir}/run_average/simulation_stats.json'
                if os.path.exists(stats_file):
                    with open(stats_file, 'r') as f:
                        stats_data = json.load(f)
                    block_interval = stats_data['parameters']['block_interval']  # in seconds
            except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
                print(f"Warning: Could not load block interval for simulation {sim_index}: {e}")
            
            # Create individual curves plots for this simulation
            create_per_run_plots(sim_data_dir, sim_figs_dir, block_interval)
            
        print(f"Individual curves plots generated for all simulations in {sweep_type} sweep!")
        
    except Exception as e:
        print(f"Error generating individual curves plots: {e}")
        import traceback
        traceback.print_exc()



def run_sweep_plots(sweep_name: str, param_name: str, sweep_type: str) -> None:
    """
    Generic function to run plots for any sweep simulation.
    
    This function:
    1. Runs the averaging script to create run_average folders from individual runs
    2. Creates a combined sweep_results_averaged.json from the run_average data
    3. Generates all plots from the combined data
    
    Args:
        sweep_name: The name of the sweep directory (e.g., 'sim_sweep_cat_rate')
        param_name: The parameter name being swept (e.g., 'cat_rate')
        sweep_type: The display name for the sweep (e.g., 'CAT Rate')
    """
    results_dir = f'simulator/results/{sweep_name}'
    
    # Generate all plots using the generic utility
    # Data flow: run_average folders -> sweep_results_averaged.json -> plots
    generate_all_plots(results_dir, param_name, sweep_type)

 