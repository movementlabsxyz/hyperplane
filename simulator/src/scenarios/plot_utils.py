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
    param_display_names = {
        'zipf_parameter': 'Zipf Parameter',
        'block_interval': 'Block Interval',
        'cat_rate': 'CAT Rate',
        'chain_delay': 'Chain Delay',
        'duration': 'Duration',
        'cat_lifetime': 'CAT Lifetime'
    }
    
    param_display = param_display_names.get(param_name, param_name.replace('_', ' ').title())
    return f'{param_display} Sweep'

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
            chain_data = result[f'chain_1_{transaction_type}']
            
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
            title = f'All {transaction_type.title()} Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'{transaction_type}_all_transactions_overlay.png'
        elif transaction_type.startswith('cat_'):
            # CAT transactions
            status = transaction_type.replace('cat_', '')
            title = f'CAT {status.title()} Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'{status}_cat_transactions_overlay.png'
        elif transaction_type.startswith('regular_'):
            # Regular transactions
            status = transaction_type.replace('regular_', '')
            title = f'Regular {status.title()} Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'{status}_regular_transactions_overlay.png'
        else:
            # Fallback
            title = f'{transaction_type.title()} Transactions by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'{transaction_type}_transactions_overlay.png'
        
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
            
            # Calculate total success, failure, and pending from chain_1 data
            success_total = sum(count for _, count in result['chain_1_success'])
            failure_total = sum(count for _, count in result['chain_1_failure'])
            pending_total = sum(count for _, count in result['chain_1_pending'])
            
            success_counts.append(success_total)
            failure_counts.append(failure_total)
            pending_counts.append(pending_total)
        
        # Create the line chart
        ax.plot(param_values, success_counts, 'go-', linewidth=2, markersize=6, label='Success')
        ax.plot(param_values, failure_counts, 'ro-', linewidth=2, markersize=6, label='Failed')
        ax.plot(param_values, pending_counts, 'yo-', linewidth=2, markersize=6, label='Pending')
        
        param_display_names = {
            'zipf_parameter': 'Zipf Parameter',
            'block_interval': 'Block Interval (seconds)',
            'cat_rate': 'CAT Rate',
            'chain_delay': 'Chain Delay (blocks)',
            'duration': 'Duration (blocks)',
            'cat_lifetime': 'CAT Lifetime (blocks)'
        }
        
        xlabel = param_display_names.get(param_name, param_name.replace('_', ' ').title())
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
            
            # Calculate total CAT and regular failures from chain_1 data
            cat_failure_total = sum(count for _, count in result.get('chain_1_cat_failure', []))
            regular_failure_total = sum(count for _, count in result.get('chain_1_regular_failure', []))
            
            cat_failure_counts.append(cat_failure_total)
            regular_failure_counts.append(regular_failure_total)
        
        # Create the line chart
        ax.plot(param_values, cat_failure_counts, 'ro-', linewidth=2, markersize=6, label='CAT Failures')
        ax.plot(param_values, regular_failure_counts, 'mo-', linewidth=2, markersize=6, label='Regular Failures')
        
        param_display_names = {
            'zipf_parameter': 'Zipf Parameter',
            'block_interval': 'Block Interval (seconds)',
            'cat_rate': 'CAT Rate',
            'chain_delay': 'Chain Delay (seconds)',
            'duration': 'Duration (blocks)',
            'cat_lifetime': 'CAT Lifetime (blocks)',
            'allow_cat_pending_dependencies': 'Allow CAT Pending Dependencies'
        }
        
        xlabel = param_display_names.get(param_name, param_name.replace('_', ' ').title())
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
            
            # Calculate totals from chain_1 data for the specified transaction type
            success_total = sum(count for _, count in result.get(f'chain_1_{transaction_type}_success', []))
            failure_total = sum(count for _, count in result.get(f'chain_1_{transaction_type}_failure', []))
            pending_total = sum(count for _, count in result.get(f'chain_1_{transaction_type}_pending', []))
            
            success_counts.append(success_total)
            failure_counts.append(failure_total)
            pending_counts.append(pending_total)
        
        # Create the line chart
        ax.plot(param_values, success_counts, 'go-', linewidth=2, markersize=6, label='Success')
        ax.plot(param_values, failure_counts, 'ro-', linewidth=2, markersize=6, label='Failed')
        ax.plot(param_values, pending_counts, 'yo-', linewidth=2, markersize=6, label='Pending')
        
        param_display_names = {
            'zipf_parameter': 'Zipf Parameter',
            'block_interval': 'Block Interval (seconds)',
            'cat_rate': 'CAT Rate',
            'chain_delay': 'Chain Delay (seconds)',
            'duration': 'Duration (blocks)',
            'cat_lifetime': 'CAT Lifetime (blocks)',
            'allow_cat_pending_dependencies': 'Allow CAT Pending Dependencies'
        }
        
        xlabel = param_display_names.get(param_name, param_name.replace('_', ' ').title())
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
        
        param_display_names = {
            'zipf_parameter': 'Zipf Parameter',
            'block_interval': 'Block Interval (seconds)',
            'cat_rate': 'CAT Rate',
            'chain_delay': 'Chain Delay (seconds)',
            'duration': 'Duration (blocks)',
            'cat_lifetime': 'CAT Lifetime (blocks)'
        }
        
        xlabel = param_display_names.get(param_name, param_name.replace('_', ' ').title())
        
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
    plot_sweep_locked_keys(results_dir)
    
    print(f"{sweep_type} simulation plots generated successfully!") 

# ------------------------------------------------------------------------------------------------
# Locked Keys Plotting
# ------------------------------------------------------------------------------------------------

def plot_sweep_locked_keys(results_dir: str) -> None:
    """
    Plot locked keys overlay for sweep simulations.
    
    # Arguments
    * `results_dir` - Directory name of the sweep (e.g., 'sim_sweep_cat_rate')
    """
    try:
        # Load sweep results
        with open(f'{results_dir}/data/sweep_results.json', 'r') as f:
            sweep_data = json.load(f)
        
        individual_results = sweep_data['individual_results']
        
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
                
            # Extract data - chain_data is a list of tuples (height, count)
            heights = [entry[0] for entry in chain_data]
            counts = [entry[1] for entry in chain_data]
            
            # Plot with color based on parameter
            label = create_parameter_label(key, param_value)
            plt.plot(heights, counts, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Get parameter name from results directory
        param_name = results_dir.replace('simulator/results/', '').replace('sim_sweep_', '').replace('_', ' ').title()
        plt.title(f'Locked Keys by Height (Chain 1) - {param_name} Sweep')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Locked Keys')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/locked_keys_overlay.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing sweep locked keys data for {results_dir}: {e}")
        return 

 