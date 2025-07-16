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
    """Create a color gradient from red (0) to blue (max)"""
    return plt.cm.RdYlBu_r(np.linspace(0, 1, num_simulations))

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
        return f'Chain Delay: {param_value:.0f} blocks'
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
        plt.figure(figsize=(12, 8))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
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
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, counts, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Create title and filename based on transaction type
        if transaction_type in ['pending', 'success', 'failure']:
            # Combined totals
            title = f'SumTypes {transaction_type.title()} Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}_sumTypes.png'
        elif transaction_type.startswith('cat_'):
            # CAT transactions
            status = transaction_type.replace('cat_', '')
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
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing {transaction_type} transactions data: {e}")
        return

# ------------------------------------------------------------------------------------------------
# Summary Chart Plotting
# ------------------------------------------------------------------------------------------------

def plot_transaction_status_chart(ax: plt.Axes, data: Dict[str, Any], param_name: str) -> None:
    """Create a line chart showing failed/success/pending data vs parameter"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            return
        
        # Extract data for the chart
        param_values = []
        success_counts = []
        failure_counts = []
        pending_counts = []
        
        for result in individual_results:
            param_values.append(extract_parameter_value(result, param_name))
            
            # Calculate total success, failure, and pending from chain_1 data (trimmed)
            success_data = trim_time_series_data(result['chain_1_success'], 0.1)
            failure_data = trim_time_series_data(result['chain_1_failure'], 0.1)
            pending_data = trim_time_series_data(result['chain_1_pending'], 0.1)
            
            success_total = sum(count for _, count in success_data)
            failure_total = sum(count for _, count in failure_data)
            pending_total = sum(count for _, count in pending_data)
            
            success_counts.append(success_total)
            failure_counts.append(failure_total)
            pending_counts.append(pending_total)
        
        # Create the line chart
        ax.plot(param_values, success_counts, 'go-', linewidth=2, markersize=6, label='Success')
        ax.plot(param_values, failure_counts, 'ro-', linewidth=2, markersize=6, label='Failed')
        ax.plot(param_values, pending_counts, 'yo-', linewidth=2, markersize=6, label='Pending')
        
        xlabel = PARAM_DISPLAY_NAMES.get(param_name, param_name.replace('_', ' ').title())
        ax.set_title(f'Transaction Status vs {xlabel}')
        ax.set_xlabel(xlabel)
        ax.set_ylabel('Number of Transactions')
        ax.legend()
        ax.grid(True, alpha=0.3)
        ax.set_ylim(bottom=0)
        
    except (KeyError, IndexError) as e:
        print(f"Warning: Error creating transaction status chart: {e}")
        ax.text(0.5, 0.5, 'Error creating chart', ha='center', va='center', transform=ax.transAxes)
        ax.axis('off')

def plot_failure_breakdown_chart(ax: plt.Axes, data: Dict[str, Any], param_name: str) -> None:
    """Create a line chart showing CAT vs regular failure breakdown vs parameter"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            return
        
        # Extract data for the chart
        param_values = []
        cat_failure_counts = []
        regular_failure_counts = []
        
        for result in individual_results:
            param_values.append(extract_parameter_value(result, param_name))
            
            # Calculate total CAT and regular failures from chain_1 data (trimmed)
            cat_failure_data = trim_time_series_data(result.get('chain_1_cat_failure', []), 0.1)
            regular_failure_data = trim_time_series_data(result.get('chain_1_regular_failure', []), 0.1)
            
            cat_failure_total = sum(count for _, count in cat_failure_data)
            regular_failure_total = sum(count for _, count in regular_failure_data)
            
            cat_failure_counts.append(cat_failure_total)
            regular_failure_counts.append(regular_failure_total)
        
        # Create the line chart
        ax.plot(param_values, cat_failure_counts, 'ro-', linewidth=2, markersize=6, label='CAT Failures')
        ax.plot(param_values, regular_failure_counts, 'mo-', linewidth=2, markersize=6, label='Regular Failures')
        
        xlabel = PARAM_DISPLAY_NAMES.get(param_name, param_name.replace('_', ' ').title())
        ax.set_title(f'Failure Breakdown vs {xlabel}')
        ax.set_xlabel(xlabel)
        ax.set_ylabel('Number of Failed Transactions')
        ax.legend()
        ax.grid(True, alpha=0.3)
        ax.set_ylim(bottom=0)
        
    except (KeyError, IndexError) as e:
        print(f"Warning: Error creating failure breakdown chart: {e}")
        ax.text(0.5, 0.5, 'Error creating chart', ha='center', va='center', transform=ax.transAxes)
        ax.axis('off')

def plot_transaction_status_chart_separate(ax: plt.Axes, data: Dict[str, Any], param_name: str, transaction_type: str) -> None:
    """Create a line chart showing pending/success/failure data vs parameter for CAT or regular transactions"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            return
        
        # Extract data for the chart
        param_values = []
        success_counts = []
        failure_counts = []
        pending_counts = []
        
        for result in individual_results:
            param_values.append(extract_parameter_value(result, param_name))
            
            # Calculate totals from chain_1 data for the specified transaction type (trimmed)
            success_data = trim_time_series_data(result.get(f'chain_1_{transaction_type}_success', []), 0.1)
            failure_data = trim_time_series_data(result.get(f'chain_1_{transaction_type}_failure', []), 0.1)
            pending_data = trim_time_series_data(result.get(f'chain_1_{transaction_type}_pending', []), 0.1)
            
            success_total = sum(count for _, count in success_data)
            failure_total = sum(count for _, count in failure_data)
            pending_total = sum(count for _, count in pending_data)
            
            success_counts.append(success_total)
            failure_counts.append(failure_total)
            pending_counts.append(pending_total)
        
        # Create the line chart
        ax.plot(param_values, success_counts, 'go-', linewidth=2, markersize=6, label='Success')
        ax.plot(param_values, failure_counts, 'ro-', linewidth=2, markersize=6, label='Failed')
        ax.plot(param_values, pending_counts, 'yo-', linewidth=2, markersize=6, label='Pending')
        
        xlabel = PARAM_DISPLAY_NAMES.get(param_name, param_name.replace('_', ' ').title())
        transaction_display = transaction_type.replace('_', ' ').title()
        ax.set_title(f'{transaction_display} Transaction Status vs {xlabel}')
        ax.set_xlabel(xlabel)
        ax.set_ylabel('Number of Transactions')
        ax.legend()
        ax.grid(True, alpha=0.3)
        ax.set_ylim(bottom=0)
        
    except (KeyError, IndexError) as e:
        print(f"Warning: Error creating {transaction_type} transaction status chart: {e}")
        ax.text(0.5, 0.5, 'Error creating chart', ha='center', va='center', transform=ax.transAxes)
        ax.axis('off')

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
        
        # Create subplots - 3x2 grid for more detailed analysis
        fig, ((ax1, ax2), (ax3, ax4), (ax5, ax6)) = plt.subplots(3, 2, figsize=(15, 15))
        
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
        
        # Plot 3: Combined transaction status chart
        plot_transaction_status_chart(ax3, data, param_name)
        
        # Plot 4: Failure breakdown chart
        plot_failure_breakdown_chart(ax4, data, param_name)
        
        # Plot 5: CAT transaction status chart
        plot_transaction_status_chart_separate(ax5, data, param_name, 'cat')
        
        # Plot 6: Regular transaction status chart
        plot_transaction_status_chart_separate(ax6, data, param_name, 'regular')
        
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
    results_path: str,
    param_name: str,
    results_dir: str,
    sweep_type: str
) -> None:
    """Generate all plots for a sweep simulation"""
    import subprocess
    
    # First, run the averaging script to create averaged data
    print("Running averaging script...")
    try:
        # Extract the results directory name from the full path
        results_dir_name = results_dir.replace('simulator/results/', '')
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
    
    # Create sweep data from averaged runs
    results_path = create_sweep_data_from_averaged_runs(results_dir_name)
    
    # Load data
    data = load_sweep_data(results_path)
    
    # Check if we have any data to plot
    if not data.get('individual_results'):
        print(f"No data found for {sweep_type} simulation. Skipping plot generation.")
        return
    
    # Create results directory only if we have data
    os.makedirs(f'{results_dir}/figs', exist_ok=True)
    
    # Plot all transaction overlays (combined totals)
    plot_transactions_overlay(data, param_name, 'pending', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'success', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'failure', results_dir, sweep_type)
    
    # Plot CAT transaction overlays
    plot_transactions_overlay(data, param_name, 'cat_pending', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'cat_success', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'cat_failure', results_dir, sweep_type)
    
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
    
    # Plot individual TPS and system resource plots for each simulation
    plot_individual_sweep_tps(data, param_name, results_dir, sweep_type)
    
    # Plot system memory usage over time
    plot_system_memory(data, param_name, results_dir, sweep_type)
    
    # Plot system total RAM usage over time
    plot_system_total_ram(data, param_name, results_dir, sweep_type)
    
    # Plot system CPU usage over time
    plot_system_cpu(data, param_name, results_dir, sweep_type)
    
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
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
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
        ax1.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        
        # Set up bottom panel (pending transactions breakdown)
        ax2.set_xlabel('Block Height')
        ax2.set_ylabel('Number of Pending Transactions')
        ax2.grid(True, alpha=0.3)
        ax2.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        
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
        ax1.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        
        ax2.set_title(f'Transactions per Second (Chain 1) - {create_sweep_title(param_name, sweep_type)}')
        ax2.set_xlabel('Block Height')
        ax2.set_ylabel('TPS')
        ax2.grid(True, alpha=0.3)
        ax2.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        
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

def plot_individual_sweep_tps(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot individual TPS and system resource plots for each simulation in the sweep, showing separate curves for each run.
    
    # Arguments
    * `data` - The sweep data containing individual results
    * `param_name` - Name of the parameter being swept
    * `results_dir` - Directory name of the sweep (e.g., 'sim_sweep_cat_rate')
    * `sweep_type` - Type of sweep simulation
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping individual TPS plots")
            return
        
        # Extract just the directory name from the full path
        results_dir_name = results_dir.replace('simulator/results/', '')
        
        # Get block interval from simulation stats
        try:
            stats_file = f'simulator/results/{results_dir_name}/data/sim_0/run_average/simulation_stats.json'
            with open(stats_file, 'r') as f:
                stats_data = json.load(f)
            block_interval = stats_data['parameters']['block_interval']
        except (FileNotFoundError, KeyError) as e:
            print(f"Warning: Could not determine block_interval for individual TPS plots: {e}")
            return
        
        # Create individual plots for each simulation
        for sim_index, result in enumerate(individual_results):
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
            
            # Create subdirectory for this simulation
            sim_figs_dir = f'{results_dir}/figs/sim_{sim_index}'
            os.makedirs(sim_figs_dir, exist_ok=True)
            
            # Load individual run data for this simulation
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            
            # Check if the simulation directory exists
            if not os.path.exists(sim_data_dir):
                continue
            
            # Get all run directories (exclude run_average)
            run_dirs = [d for d in os.listdir(sim_data_dir) 
                       if d.startswith('run_') and d != 'run_average' and os.path.isdir(os.path.join(sim_data_dir, d))]
            # Sort numerically by run number
            run_dirs.sort(key=lambda x: int(x.split('_')[1]) if '_' in x else 0)
            
            if not run_dirs:
                continue
            
            # Create figure with subplots
            fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 10), sharex=True)
            
            # Create color gradient for runs
            colors = create_color_gradient(len(run_dirs))
            
            # Plot each run's data
            plotted_runs = 0
            for run_idx, run_dir in enumerate(run_dirs):
                try:
                    # Load transactions per block data for this run
                    tx_per_block_file = os.path.join(sim_data_dir, run_dir, 'data', 'tx_per_block_chain_1.json')
                    if not os.path.exists(tx_per_block_file):
                        print(f"Warning: {tx_per_block_file} not found")
                        continue
                    
                    with open(tx_per_block_file, 'r') as f:
                        run_data = json.load(f)
                    
                    # Extract data
                    blocks = [entry['height'] for entry in run_data['chain_1_tx_per_block']]
                    tx_per_block = [entry['count'] for entry in run_data['chain_1_tx_per_block']]
                    
                    # Calculate TPS
                    tps = [tx_count / block_interval for tx_count in tx_per_block]
                    
                    # Apply 20-block running average to both transactions per block and TPS
                    tx_per_block_smoothed = calculate_running_average(tx_per_block, 20)
                    tps_smoothed = calculate_running_average(tps, 20)
                    
                    # Plot with color based on run
                    label = f'Run {run_idx + 1}'
                    ax1.plot(blocks, tx_per_block_smoothed, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    ax2.plot(blocks, tps_smoothed, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    plotted_runs += 1
                    
                except Exception as e:
                    print(f"Warning: Error processing run {run_dir} for simulation {sim_index}: {e}")
                    continue
            
            # Create titles
            param_label = create_parameter_label(param_name, param_value)
            title = f'Transactions per Block (Chain 1) - {param_label} (20-block running average)'
            ax1.set_title(title)
            ax1.set_ylabel('Number of Transactions')
            ax1.grid(True, alpha=0.3)
            ax1.legend()
            
            ax2.set_title(f'Transactions per Second (Chain 1) - {param_label} (Block Interval: {block_interval}s, 20-block running average)')
            ax2.set_xlabel('Block Height')
            ax2.set_ylabel('TPS')
            ax2.grid(True, alpha=0.3)
            ax2.legend()
            
            plt.tight_layout()
            
            # Save the plot
            plt.savefig(f'{sim_figs_dir}/tps.png', dpi=300, bbox_inches='tight')
            plt.close()
            
            # Create system memory usage plot for this simulation
            fig, ax = plt.subplots(figsize=(12, 8))
            
            # Plot each run's system memory usage data
            plotted_runs = 0
            for run_idx, run_dir in enumerate(run_dirs):
                try:
                    # Load system memory usage data for this run
                    memory_file = os.path.join(sim_data_dir, run_dir, 'data', 'system_memory.json')
                    if not os.path.exists(memory_file):
                        print(f"Warning: {memory_file} not found")
                        continue
                    
                    with open(memory_file, 'r') as f:
                        run_data = json.load(f)
                    
                    # Extract system memory usage data
                    if 'system_memory' in run_data:
                        memory_entries = run_data['system_memory']
                        if memory_entries:
                            # Extract block heights and memory usage values
                            heights = [entry['height'] for entry in memory_entries]
                            memory_values = [entry['bytes'] / (1024 * 1024) for entry in memory_entries]  # Convert to MB
                            
                            # Plot with color based on run
                            label = f'Run {run_idx + 1}'
                            ax.plot(heights, memory_values, color=colors[run_idx], alpha=0.7, 
                                    label=label, linewidth=1.5)
                            plotted_runs += 1
                    
                except Exception as e:
                    print(f"Warning: Error processing system memory for run {run_dir} in simulation {sim_index}: {e}")
                    continue
            
            # Create title
            param_label = create_parameter_label(param_name, param_value)
            ax.set_title(f'System Memory Usage Over Time - {param_label}')
            ax.set_xlabel('Block Height')
            ax.set_ylabel('System Memory Usage (MB)')
            ax.grid(True, alpha=0.3)
            ax.legend()
            
            plt.tight_layout()
            
            # Save the system memory usage plot
            plt.savefig(f'{sim_figs_dir}/system_memory.png', dpi=300, bbox_inches='tight')
            plt.close()
            
            # Create system total RAM usage plot for this simulation
            fig, ax = plt.subplots(figsize=(12, 8))
            
            # Plot each run's system total RAM usage data
            plotted_runs = 0
            for run_idx, run_dir in enumerate(run_dirs):
                try:
                    # Load system total RAM usage data for this run
                    ram_file = os.path.join(sim_data_dir, run_dir, 'data', 'system_total_ram.json')
                    if not os.path.exists(ram_file):
                        print(f"Warning: {ram_file} not found")
                        continue
                    
                    with open(ram_file, 'r') as f:
                        run_data = json.load(f)
                    
                    # Extract system total RAM usage data
                    if 'system_total_ram' in run_data:
                        ram_entries = run_data['system_total_ram']
                        if ram_entries:
                            # Extract block heights and RAM usage values
                            heights = [entry['height'] for entry in ram_entries]
                            ram_values = [entry['bytes'] / (1024 * 1024 * 1024) for entry in ram_entries]  # Convert to GB
                            
                            # Plot with color based on run
                            label = f'Run {run_idx + 1}'
                            ax.plot(heights, ram_values, color=colors[run_idx], alpha=0.7, 
                                    label=label, linewidth=1.5)
                            plotted_runs += 1
                    
                except Exception as e:
                    print(f"Warning: Error processing system total RAM for run {run_dir} in simulation {sim_index}: {e}")
                    continue
            
            # Create title
            param_label = create_parameter_label(param_name, param_value)
            ax.set_title(f'System Total RAM Usage Over Time - {param_label}')
            ax.set_xlabel('Block Height')
            ax.set_ylabel('System Total RAM Usage (GB)')
            ax.grid(True, alpha=0.3)
            ax.legend()
            
            plt.tight_layout()
            
            # Save the system total RAM usage plot
            plt.savefig(f'{sim_figs_dir}/system_total_ram.png', dpi=300, bbox_inches='tight')
            plt.close()
            
            # Create system CPU usage plot for this simulation
            fig, ax = plt.subplots(figsize=(12, 8))
            
            # Plot each run's system CPU usage data
            plotted_runs = 0
            for run_idx, run_dir in enumerate(run_dirs):
                try:
                    # Load system CPU usage data for this run
                    cpu_file = os.path.join(sim_data_dir, run_dir, 'data', 'system_cpu.json')
                    if not os.path.exists(cpu_file):
                        print(f"Warning: {cpu_file} not found")
                        continue
                    
                    with open(cpu_file, 'r') as f:
                        run_data = json.load(f)
                    
                    # Extract system CPU usage data
                    if 'system_cpu' in run_data:
                        cpu_entries = run_data['system_cpu']
                        if cpu_entries:
                            # Extract block heights and CPU usage values
                            heights = [entry['height'] for entry in cpu_entries]
                            cpu_values = [entry['percent'] for entry in cpu_entries]  # Already in percent
                            
                            # Plot with color based on run
                            label = f'Run {run_idx + 1}'
                            ax.plot(heights, cpu_values, color=colors[run_idx], alpha=0.7, 
                                    label=label, linewidth=1.5)
                            plotted_runs += 1
                    
                except Exception as e:
                    print(f"Warning: Error processing system CPU for run {run_dir} in simulation {sim_index}: {e}")
                    continue
            
            # Create title
            param_label = create_parameter_label(param_name, param_value)
            ax.set_title(f'System CPU Usage Over Time - {param_label}')
            ax.set_xlabel('Block Height')
            ax.set_ylabel('System CPU Usage (%)')
            ax.grid(True, alpha=0.3)
            ax.legend()
            
            plt.tight_layout()
            
            # Save the system CPU usage plot
            plt.savefig(f'{sim_figs_dir}/system_cpu.png', dpi=300, bbox_inches='tight')
            plt.close()
            
    except Exception as e:
        print(f"Warning: Error processing individual TPS plots for sweep: {e}")
        return

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
                print(f"Warning: System memory file not found: {memory_file}")
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('System Memory Usage (MB)')
        ax.set_title(f'System Memory Usage Over Time by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/system_memory.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting system memory data: {e}")
        import traceback
        traceback.print_exc()

def plot_system_total_ram(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
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
        for sim_index, (result, color) in enumerate(zip(individual_results, colors)):
            param_value = result[param_name]
            label = create_parameter_label(param_name, param_value)
            
            # Load system total RAM usage data for this simulation
            # Extract just the directory name from the full path
            results_dir_name = results_dir.replace('simulator/results/', '')
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            
            # Use averaged system total RAM usage data
            system_total_ram_file = f'{sim_data_dir}/run_average/system_total_ram.json'
            if os.path.exists(system_total_ram_file):
                with open(system_total_ram_file, 'r') as f:
                    system_total_ram_data = json.load(f)
                
                # Extract system total RAM usage data
                if 'system_total_ram' in system_total_ram_data:
                    system_total_ram_entries = system_total_ram_data['system_total_ram']
                    if system_total_ram_entries:
                        # Extract block heights and system total RAM usage values
                        heights = [entry['height'] for entry in system_total_ram_entries]
                        system_total_ram_values = [entry['bytes'] / (1024 * 1024 * 1024) for entry in system_total_ram_entries]  # Convert to GB
                        
                        # Ensure heights and system_total_ram_values have the same length
                        if len(heights) != len(system_total_ram_values):
                            print(f"Warning: Heights ({len(heights)}) and system total RAM values ({len(system_total_ram_values)}) have different lengths for simulation {sim_index}")
                            # Use the shorter length
                            min_length = min(len(heights), len(system_total_ram_values))
                            heights = heights[:min_length]
                            system_total_ram_values = system_total_ram_values[:min_length]
                        
                        # Plot the averaged data directly (no additional smoothing needed)
                        ax.plot(heights, system_total_ram_values, color=color, alpha=0.7, linewidth=2)
                    else:
                        print(f"Warning: No system total RAM entries found for simulation {sim_index}")
                else:
                    print(f"Warning: No system_total_ram key found in {system_total_ram_file}")
            else:
                print(f"Warning: System total RAM file not found: {system_total_ram_file}")
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('System Total RAM Usage (GB)')
        ax.set_title(f'System Total RAM Usage Over Time by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/system_total_ram.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting system total RAM data: {e}")
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
                print(f"Warning: System CPU file not found: {cpu_file}")
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('System CPU Usage (%)')
        ax.set_title(f'System CPU Usage Over Time by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/system_cpu.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting system CPU data: {e}")
        import traceback
        traceback.print_exc()

def run_sweep_plots(sweep_name: str, param_name: str, sweep_type: str) -> None:
    """
    Generic function to run plots for any sweep simulation.
    
    Args:
        sweep_name: The name of the sweep directory (e.g., 'sim_sweep_cat_rate')
        param_name: The parameter name being swept (e.g., 'cat_rate')
        sweep_type: The display name for the sweep (e.g., 'CAT Rate')
    """
    results_path = f'simulator/results/{sweep_name}/data/sweep_results.json'
    results_dir = f'simulator/results/{sweep_name}'
    
    # Generate all plots using the generic utility
    generate_all_plots(results_path, param_name, results_dir, sweep_type)

 