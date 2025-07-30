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

def load_sweep_data_from_run_average(results_dir_name: str, base_path: str = 'simulator/results') -> Dict[str, Any]:
    """Load sweep data structure directly from run_average directories."""
    base_dir = f'{base_path}/{results_dir_name}/data'
    
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
    
    # Return the complete data structure directly (no file creation)
    return {
        'sweep_summary': sweep_summary,
        'individual_results': individual_results
    }



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
        
        # Create tx_count directory and save plot
        tx_count_dir = f'{results_dir}/figs/tx_count'
        os.makedirs(tx_count_dir, exist_ok=True)
        plt.savefig(f'{tx_count_dir}/{filename}', 
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
    2. Loads data directly from run_average folders for plotting
    3. Generates all plots from the run_average data
    
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
    
    # Load sweep data directly from run_average folders
    data = load_sweep_data_from_run_average(results_dir_name)
    
    # Check if we have any data to plot
    if not data.get('individual_results'):
        print(f"No data found for {sweep_type} simulation. Skipping plot generation.")
        return
    
    # Load plot configuration
    try:
        plot_config = load_plot_config(results_dir)
    except (FileNotFoundError, ValueError) as e:
        print(f"Error loading plot configuration: {e}")
        print("Plotting aborted. Please ensure all config files have the required [plot_config] section.")
        return
    
    # Create results directory only if we have data
    os.makedirs(f'{results_dir}/figs', exist_ok=True)
    
    # Generate paper plots first (fastest)
    print("DEBUG: Starting paper plot generation...")
    try:
        # Import and run the paper plot script if it exists
        plot_paper_path = os.path.join(os.path.dirname(__file__), results_dir_name, 'plot_paper.py')
        print(f"Looking for paper plot script at: {plot_paper_path}")
        print(f"Script exists: {os.path.exists(plot_paper_path)}")
        if os.path.exists(plot_paper_path):
            print("Generating paper plots...")
            print(f"DEBUG: About to run subprocess with cwd={os.path.dirname(plot_paper_path)}")
            import subprocess
            result = subprocess.run([sys.executable, plot_paper_path], 
                                   cwd=os.path.dirname(plot_paper_path), 
                                   capture_output=True, text=True)
            print(f"Paper plot script stdout: {result.stdout}")
            print(f"Paper plot script stderr: {result.stderr}")
            print(f"Paper plot script return code: {result.returncode}")
            if result.returncode == 0:
                print("Paper plots generated successfully!")
                
                # PANIC CHECK: Verify paper directory and plot were actually created
                paper_dir = f'{results_dir}/figs/paper'
                if not os.path.exists(paper_dir):
                    raise RuntimeError(f"PANIC: Paper directory was not created at {paper_dir}")
                
                # Check for any PNG files in the paper directory
                paper_files = [f for f in os.listdir(paper_dir) if f.endswith('.png')]
                if not paper_files:
                    raise RuntimeError(f"PANIC: No PNG files found in paper directory {paper_dir}")
                
                print(f"✅ Paper plots verified: {len(paper_files)} files created in {paper_dir}")
            else:
                raise RuntimeError(f"PANIC: Paper plot generation failed with return code {result.returncode}. STDOUT: {result.stdout}. STDERR: {result.stderr}")
        else:
            print(f"PANIC: No plot_paper.py found at {plot_paper_path} - skipping paper plots")
            raise RuntimeError(f"PANIC: Paper plot script not found at {plot_paper_path}")
    except Exception as e:
        print(f"PANIC: Error generating paper plots: {e}")
        import traceback
        traceback.print_exc()
        raise  # Re-raise the exception to stop execution
    
    # Use the plot manager to generate organized plots
    from plot_manager import generate_organized_plots
    generate_organized_plots(data, param_name, results_dir, sweep_type, plot_config)

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
            
            # Get CAT pending resolving data
            cat_pending_data = result.get('chain_1_cat_pending_resolving', [])
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
            
            # Plot locked keys vs CAT pending resolving (top panel)
            ax1.plot(heights, locked_keys, color=colors[i], alpha=0.7, 
                    label=f'Locked Keys - {label}', linewidth=1.5)
            ax1.plot(heights, cat_pending, color=colors[i], alpha=0.7, 
                    linestyle='--', label=f'CAT Pending Resolving - {label}', linewidth=1.5)
            
            # Plot pending transactions breakdown (bottom panel)
            ax2.plot(heights, cat_pending, color=colors[i], alpha=0.7, 
                    label=f'CAT Pending Resolving - {label}', linewidth=1.5)
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

def calculate_delta_from_counts(count_data: List[Tuple[int, int]]) -> List[Tuple[int, int]]:
    """
    Calculate delta (change) between consecutive transaction counts.
    
    For each index i, subtract the value at index i-1 from the value at index i.
    The first entry (i=0) will have delta = 0.
    
    Args:
        count_data: List of tuples (height, count) representing transaction counts over time
        
    Returns:
        List of tuples (height, delta) representing the change in counts between consecutive time steps
    """
    if not count_data:
        return []
    
    delta_data = []
    
    # First entry has delta = 0
    first_height, first_count = count_data[0]
    delta_data.append((first_height, 0))
    
    # Calculate deltas for subsequent entries
    for i in range(1, len(count_data)):
        current_height, current_count = count_data[i]
        previous_count = count_data[i-1][1]
        delta = current_count - previous_count
        delta_data.append((current_height, delta))
    
    return delta_data

def plot_transactions_delta_overlay(
    data: Dict[str, Any],
    param_name: str,
    transaction_type: str,
    results_dir: str,
    sweep_type: str
) -> None:
    """Plot transaction delta overlay for a specific transaction type"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping {transaction_type} transactions delta plot")
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
            
            # Calculate deltas from the count data
            delta_data = calculate_delta_from_counts(chain_data)
            
            if not delta_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            delta_data = trim_time_series_data(delta_data, 0.1)
            
            if not delta_data:
                continue
                
            # Extract data - delta_data is a list of tuples (height, delta)
            heights = [entry[0] for entry in delta_data]
            deltas = [entry[1] for entry in delta_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, deltas, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        # Create title and filename based on transaction type
        if transaction_type in ['pending', 'success', 'failure']:
            # Combined totals
            title = f'SumTypes {transaction_type.title()} Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}_sumTypes.png'
        elif transaction_type.startswith('cat_'):
            # CAT transactions
            status = transaction_type.replace('cat_', '')
            if status == 'pending_resolving':
                title = f'CAT Pending Resolving Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_resolving.png'
            elif status == 'pending_postponed':
                title = f'CAT Pending Postponed Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_postponed.png'
            else:
                title = f'CAT {status.title()} Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat.png'
        elif transaction_type.startswith('regular_'):
            # Regular transactions
            status = transaction_type.replace('regular_', '')
            title = f'Regular {status.title()} Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{status}_regular.png'
        else:
            # Fallback
            title = f'{transaction_type.title()} Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}.png'
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel(f'Delta in {transaction_type.title()} Transactions')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Create tx_count_delta directory and save plot
        tx_count_delta_dir = f'{results_dir}/figs/tx_count_delta'
        os.makedirs(tx_count_delta_dir, exist_ok=True)
        plt.savefig(f'{tx_count_delta_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing {transaction_type} transactions delta data: {e}")
        return

def plot_cat_pending_resolving_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending resolving transaction deltas overlay"""
    plot_transactions_delta_overlay(data, param_name, 'cat_pending_resolving', results_dir, sweep_type)

def plot_cat_pending_postponed_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending postponed transaction deltas overlay"""
    plot_transactions_delta_overlay(data, param_name, 'cat_pending_postponed', results_dir, sweep_type)

def run_sweep_plots(sweep_name: str, param_name: str, sweep_type: str) -> None:
    """
    Generic function to run plots for any sweep simulation.
    
    This function:
    1. Runs the averaging script to create run_average folders from individual runs
    2. Loads data directly from run_average folders for plotting
    3. Generates all plots from the run_average data
    
    Args:
        sweep_name: The name of the sweep directory (e.g., 'sim_sweep_cat_rate')
        param_name: The parameter name being swept (e.g., 'cat_rate')
        sweep_type: The display name for the sweep (e.g., 'CAT Rate')
    """
    results_dir = f'simulator/results/{sweep_name}'
    
    # Generate all plots using the generic utility
    # Data flow: run_average folders -> plots
    generate_all_plots(results_dir, param_name, sweep_type)

def load_plot_config(results_dir: str) -> Dict[str, Any]:
    """
    Load plot configuration from the sweep config file.
    
    Args:
        results_dir: The results directory path
        
    Returns:
        Dictionary containing plot configuration settings
        
    Raises:
        FileNotFoundError: If the config file doesn't exist
        Exception: If there's an error parsing the TOML or missing required parameters
    """
    # Extract sweep name from results_dir (e.g., 'sim_sweep_cat_rate' from 'simulator/results/sim_sweep_cat_rate')
    sweep_name = results_dir.split('/')[-1]
    config_path = f'simulator/src/scenarios/{sweep_name}/config.toml'
    
    if not os.path.exists(config_path):
        raise FileNotFoundError(f"Config file {config_path} not found")
    
    import tomllib
    with open(config_path, 'rb') as f:
        config = tomllib.load(f)
    
    plot_config = config.get('plot_config', {})
    
    # Check if required parameters exist
    if 'plot_moving_average' not in plot_config:
        raise ValueError(f"Missing required parameter 'plot_moving_average' in {config_path}")
    if 'range_moving_average' not in plot_config:
        raise ValueError(f"Missing required parameter 'range_moving_average' in {config_path}")
    if 'cutoff' not in plot_config:
        raise ValueError(f"Missing required parameter 'cutoff' in {config_path}")
    
    return {
        'plot_moving_average': plot_config['plot_moving_average'],
        'range_moving_average': plot_config['range_moving_average'],
        'cutoff': plot_config['cutoff']
    }

def apply_moving_average(data: List[Tuple[int, int]], window_size: int) -> List[Tuple[int, int]]:
    """
    Apply moving average to time series data.
    
    Args:
        data: List of tuples (height, value) representing time series data
        window_size: Number of consecutive points to average
        
    Returns:
        List of tuples (height, averaged_value) with moving average applied
    """
    if len(data) < window_size:
        return data
    
    averaged_data = []
    
    for i in range(len(data)):
        # Calculate start and end indices for the window
        start_idx = max(0, i - window_size + 1)
        end_idx = i + 1
        
        # Extract values in the window
        window_values = [data[j][1] for j in range(start_idx, end_idx)]
        
        # Calculate average
        avg_value = sum(window_values) / len(window_values)
        
        # Keep the original height, use averaged value
        averaged_data.append((data[i][0], avg_value))
    
    return averaged_data

def plot_transactions_overlay_with_moving_average(
    data: Dict[str, Any],
    param_name: str,
    transaction_type: str,
    results_dir: str,
    sweep_type: str,
    plot_config: Dict[str, Any]
) -> None:
    """Plot transaction overlay with optional moving average"""
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
            
            # Apply moving average if enabled
            if plot_config.get('plot_moving_average', False):
                window_size = plot_config.get('range_moving_average', 10)
                chain_data = apply_moving_average(chain_data, window_size)
            
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
        
        # Add moving average indicator to title if enabled
        if plot_config.get('plot_moving_average', False):
            window_size = plot_config.get('range_moving_average', 10)
            title += f" (Moving Average, Window={window_size})"
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel(f'Number of {transaction_type.title()} Transactions')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Determine directory based on whether moving average is applied
        if plot_config.get('plot_moving_average', False):
            base_dir = f'{results_dir}/figs/tx_count/moving_average'
        else:
            base_dir = f'{results_dir}/figs/tx_count'
        
        # Create directory and save plot
        os.makedirs(base_dir, exist_ok=True)
        plt.savefig(f'{base_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing {transaction_type} transactions data: {e}")
        return

def plot_transactions_delta_overlay_with_moving_average(
    data: Dict[str, Any],
    param_name: str,
    transaction_type: str,
    results_dir: str,
    sweep_type: str,
    plot_config: Dict[str, Any]
) -> None:
    """Plot transaction delta overlay with optional moving average"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping {transaction_type} transactions delta plot")
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
            
            # Calculate deltas from the count data
            delta_data = calculate_delta_from_counts(chain_data)
            
            if not delta_data:
                continue
            
            # Apply moving average if enabled
            if plot_config.get('plot_moving_average', False):
                window_size = plot_config.get('range_moving_average', 10)
                delta_data = apply_moving_average(delta_data, window_size)
            
            # Trim the last 10% of data to avoid edge effects
            delta_data = trim_time_series_data(delta_data, 0.1)
            
            if not delta_data:
                continue
                
            # Extract data - delta_data is a list of tuples (height, delta)
            heights = [entry[0] for entry in delta_data]
            deltas = [entry[1] for entry in delta_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, deltas, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        # Create title and filename based on transaction type
        if transaction_type in ['pending', 'success', 'failure']:
            # Combined totals
            title = f'SumTypes {transaction_type.title()} Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}_sumTypes.png'
        elif transaction_type.startswith('cat_'):
            # CAT transactions
            status = transaction_type.replace('cat_', '')
            if status == 'pending_resolving':
                title = f'CAT Pending Resolving Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_resolving.png'
            elif status == 'pending_postponed':
                title = f'CAT Pending Postponed Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_postponed.png'
            else:
                title = f'CAT {status.title()} Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat.png'
        elif transaction_type.startswith('regular_'):
            # Regular transactions
            status = transaction_type.replace('regular_', '')
            title = f'Regular {status.title()} Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{status}_regular.png'
        else:
            # Fallback
            title = f'{transaction_type.title()} Transaction Deltas by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}.png'
        
        # Add moving average indicator to title if enabled
        if plot_config.get('plot_moving_average', False):
            window_size = plot_config.get('range_moving_average', 10)
            title += f" (Moving Average, Window={window_size})"
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel(f'Delta in {transaction_type.title()} Transactions')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Determine directory based on whether moving average is applied
        if plot_config.get('plot_moving_average', False):
            base_dir = f'{results_dir}/figs/tx_count_delta/moving_average'
        else:
            base_dir = f'{results_dir}/figs/tx_count_delta'
        
        # Create directory and save plot
        os.makedirs(base_dir, exist_ok=True)
        plt.savefig(f'{base_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing {transaction_type} transactions delta data: {e}")
        return

def apply_cutoff_to_data(data: List[Tuple[int, int]], cutoff_height: int, transaction_type: str) -> List[Tuple[int, int]]:
    """
    Apply cutoff to time series data by removing data before cutoff_height and subtracting offset values.
    
    Args:
        data: List of tuples (height, value) representing time series data
        cutoff_height: Height at which to apply the cutoff
        transaction_type: Type of transaction (success, failure, pending, etc.)
        
    Returns:
        List of tuples (height, adjusted_value) with cutoff applied
    """
    if not data:
        return []
    
    # Find the offset value at the cutoff height
    offset_value = 0
    for height, value in data:
        if height >= cutoff_height:
            offset_value = value
            break
    
    # Apply cutoff: remove data before cutoff_height
    cutoff_data = []
    for height, value in data:
        if height >= cutoff_height:
            # Only subtract offset for success and failure transactions (including cat_ and regular_ prefixes)
            if 'success' in transaction_type or 'failure' in transaction_type:
                adjusted_value = value - offset_value
            else:
                # For pending and other types, just use the original value
                adjusted_value = value
            cutoff_data.append((height, adjusted_value))
    
    return cutoff_data

def plot_transactions_cutoff_overlay(
    data: Dict[str, Any],
    param_name: str,
    transaction_type: str,
    results_dir: str,
    sweep_type: str,
    plot_config: Dict[str, Any]
) -> None:
    """
    Plot transaction overlay with cutoff applied.
    
    This function:
    1. Removes all data before cutoff_height (sim_total_block_number * cutoff)
    2. For success, failure, and pending postponed transactions, records offset values at cutoff
    3. Subtracts these offset values from the remainder of the data vector
    4. Everything else stays the same
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping {transaction_type} cutoff plot")
            return
        
        # Get cutoff configuration
        cutoff_percentage = plot_config.get('cutoff', 0.5)
        
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
            
            # Calculate cutoff height based on maximum height in data
            max_data_height = max(chain_data, key=lambda x: x[0])[0]
            cutoff_height = int(max_data_height * cutoff_percentage)
            
            # Apply cutoff to the data
            cutoff_data = apply_cutoff_to_data(chain_data, cutoff_height, transaction_type)
            
            if not cutoff_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            cutoff_data = trim_time_series_data(cutoff_data, 0.1)
            
            if not cutoff_data:
                continue
                
            # Extract data - cutoff_data is a list of tuples (height, adjusted_value)
            heights = [entry[0] for entry in cutoff_data]
            values = [entry[1] for entry in cutoff_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, values, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        # Create title and filename based on transaction type
        if transaction_type in ['pending', 'success', 'failure']:
            # Combined totals
            title = f'SumTypes {transaction_type.title()} Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}_sumTypes_cutoff.png'
        elif transaction_type.startswith('cat_'):
            # CAT transactions
            status = transaction_type.replace('cat_', '')
            if status == 'pending_resolving':
                title = f'CAT Pending Resolving Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_resolving_cutoff.png'
            elif status == 'pending_postponed':
                title = f'CAT Pending Postponed Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_postponed_cutoff.png'
            else:
                title = f'CAT {status.title()} Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat_cutoff.png'
        elif transaction_type.startswith('regular_'):
            # Regular transactions
            status = transaction_type.replace('regular_', '')
            title = f'Regular {status.title()} Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{status}_regular_cutoff.png'
        else:
            # Fallback
            title = f'{transaction_type.title()} Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}_cutoff.png'
        
        plt.title(title, fontsize=14)
        plt.xlabel('Block Height', fontsize=12)
        plt.ylabel('Transaction Count (Adjusted)', fontsize=12)
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right", fontsize=10)
        plt.tight_layout()
        
        # Create tx_count_cutoff directory and save plot
        tx_count_cutoff_dir = f'{results_dir}/figs/tx_count_cutoff'
        os.makedirs(tx_count_cutoff_dir, exist_ok=True)
        plt.savefig(f'{tx_count_cutoff_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
        print(f"Generated cutoff plot: {filename}")
        
    except Exception as e:
        print(f"Error generating cutoff plot for {transaction_type}: {e}")
        import traceback
        traceback.print_exc()



def apply_cutoff_to_percentage_data(data: Dict[str, Any], plot_config: Dict[str, Any]) -> Dict[str, Any]:
    """
    Apply cutoff to all transaction data in the results before percentage calculation.
    
    Args:
        data: The original sweep data
        plot_config: Plot configuration containing cutoff settings
        
    Returns:
        Modified data with cutoff applied to all transaction data
    """
    cutoff_percentage = plot_config.get('cutoff', 0.5)
    modified_data = {'individual_results': []}
    
    for result in data['individual_results']:
        modified_result = result.copy()
        
        # Apply cutoff to all transaction data
        transaction_keys = [
            'chain_1_cat_success', 'chain_1_cat_failure', 'chain_1_cat_pending',
            'chain_1_regular_success', 'chain_1_regular_failure', 'chain_1_regular_pending',
            'chain_1_cat_pending_resolving', 'chain_1_cat_pending_postponed'
        ]
        
        for key in transaction_keys:
            if key in modified_result:
                tx_data = modified_result[key]
                if tx_data:
                    # Calculate cutoff height based on maximum height in data
                    max_data_height = max(tx_data, key=lambda x: x[0])[0]
                    cutoff_height = int(max_data_height * cutoff_percentage)
                    
                    # Apply cutoff to the data
                    cutoff_data = apply_cutoff_to_data(tx_data, cutoff_height, key)
                    modified_result[key] = cutoff_data
        
        modified_data['individual_results'].append(modified_result)
    
    return modified_data

def plot_transaction_percentage_cutoff(
    data: Dict[str, Any], 
    param_name: str, 
    results_dir: str, 
    sweep_type: str, 
    transaction_type: str, 
    percentage_type: str,
    plot_config: Dict[str, Any]
) -> None:
    """
    Plot transaction percentage with cutoff applied.
    
    This function applies cutoff to the data and creates percentage plots
    saved to the tx_count_cutoff directory.
    """
    # Apply cutoff to the data
    cutoff_data = apply_cutoff_to_percentage_data(data, plot_config)
    
    # Import the percentage plotting function
    from plot_utils_percentage import plot_transaction_percentage
    
    # Create a modified version that saves to the correct directory
    def plot_transaction_percentage_cutoff_save(cutoff_data, param_name, results_dir, sweep_type, transaction_type, percentage_type):
        """Modified version of plot_transaction_percentage that saves to tx_count_cutoff directory"""
        try:
            # Extract parameter values and results
            param_values = []
            results = []
            
            for result in cutoff_data['individual_results']:
                param_value = extract_parameter_value(result, param_name)
                param_values.append(param_value)
                results.append(result)
            
            # Create color gradient
            from plot_utils_percentage import create_color_gradient
            colors = create_color_gradient(len(param_values))
            
            # Create the plot
            plt.figure(figsize=(10, 6))
            
            # Track maximum height for xlim
            max_height = 0
            
            # Plot each parameter value
            for i, (param_value, result) in enumerate(zip(param_values, results)):
                # Extract transaction data using the same structure as regular plots
                if transaction_type == 'cat':
                    # For CAT transactions, get the specific status data
                    if percentage_type == 'success':
                        tx_data = result.get('chain_1_cat_success', [])
                    elif percentage_type == 'failure':
                        tx_data = result.get('chain_1_cat_failure', [])
                    else:  # pending
                        tx_data = result.get('chain_1_cat_pending', [])
                elif transaction_type == 'regular':
                    # For regular transactions, get the specific status data
                    if percentage_type == 'success':
                        tx_data = result.get('chain_1_regular_success', [])
                    elif percentage_type == 'failure':
                        tx_data = result.get('chain_1_regular_failure', [])
                    else:  # pending
                        tx_data = result.get('chain_1_regular_pending', [])
                else:  # sumtypes
                    # For sumtypes, combine CAT and regular data
                    if percentage_type == 'success':
                        cat_data = result.get('chain_1_cat_success', [])
                        regular_data = result.get('chain_1_regular_success', [])
                    elif percentage_type == 'failure':
                        cat_data = result.get('chain_1_cat_failure', [])
                        regular_data = result.get('chain_1_regular_failure', [])
                    else:  # pending
                        cat_data = result.get('chain_1_cat_pending', [])
                        regular_data = result.get('chain_1_regular_pending', [])
                    
                    # Create a combined dataset by summing CAT and regular at each height
                    combined_data = {}
                    
                    # Add CAT data
                    for height, count in cat_data:
                        combined_data[height] = combined_data.get(height, 0) + count
                    
                    # Add regular data
                    for height, count in regular_data:
                        combined_data[height] = combined_data.get(height, 0) + count
                    
                    # Convert back to sorted list of tuples
                    tx_data = sorted(combined_data.items())
                
                if not tx_data:
                    continue
                
                # Extract heights and transaction counts
                heights = []
                percentages = []
                
                # Convert tx_data to list of tuples if it's not already
                if isinstance(tx_data, list) and tx_data and isinstance(tx_data[0], dict):
                    # If it's a list of dictionaries, convert to list of tuples
                    tx_data = [(entry.get('height', 0), entry.get('count', 0)) for entry in tx_data]
                
                # Process each block - tx_data is now a list of tuples (height, count)
                for height, count in tx_data:
                    heights.append(height)
                    
                    # Calculate percentage using counts at this specific height (not cumulative)
                    if transaction_type == 'cat':
                        # For CAT transactions, get all status data
                        cat_success_data = result.get('chain_1_cat_success', [])
                        cat_failure_data = result.get('chain_1_cat_failure', [])
                        cat_pending_data = result.get('chain_1_cat_pending', [])
                        
                        # Convert to height->count mapping
                        cat_success_by_height = {entry[0]: entry[1] for entry in cat_success_data}
                        cat_failure_by_height = {entry[0]: entry[1] for entry in cat_failure_data}
                        cat_pending_by_height = {entry[0]: entry[1] for entry in cat_pending_data}
                        
                        # Get counts at this specific height
                        success_at_height = cat_success_by_height.get(height, 0)
                        failure_at_height = cat_failure_by_height.get(height, 0)
                        pending_at_height = cat_pending_by_height.get(height, 0)
                        
                    elif transaction_type == 'regular':
                        # For regular transactions, get all status data
                        regular_success_data = result.get('chain_1_regular_success', [])
                        regular_failure_data = result.get('chain_1_regular_failure', [])
                        regular_pending_data = result.get('chain_1_regular_pending', [])
                        
                        # Convert to height->count mapping
                        regular_success_by_height = {entry[0]: entry[1] for entry in regular_success_data}
                        regular_failure_by_height = {entry[0]: entry[1] for entry in regular_failure_data}
                        regular_pending_by_height = {entry[0]: entry[1] for entry in regular_pending_data}
                        
                        # Get counts at this specific height
                        success_at_height = regular_success_by_height.get(height, 0)
                        failure_at_height = regular_failure_by_height.get(height, 0)
                        pending_at_height = regular_pending_by_height.get(height, 0)
                        
                    elif transaction_type in ['cat_pending_resolving', 'cat_pending_postponed']:
                        # For CAT pending resolving/postponed, use (resolving + postponed) as denominator
                        cat_pending_resolving_data = result.get('chain_1_cat_pending_resolving', [])
                        cat_pending_postponed_data = result.get('chain_1_cat_pending_postponed', [])
                        
                        # Convert to height->count mapping
                        cat_pending_resolving_by_height = {entry[0]: entry[1] for entry in cat_pending_resolving_data}
                        cat_pending_postponed_by_height = {entry[0]: entry[1] for entry in cat_pending_postponed_data}
                        
                        # Get counts at this specific height
                        resolving_at_height = cat_pending_resolving_by_height.get(height, 0)
                        postponed_at_height = cat_pending_postponed_by_height.get(height, 0)
                        
                        # Calculate percentage of resolving/postponed vs total pending (resolving + postponed)
                        total_pending = resolving_at_height + postponed_at_height
                        if total_pending > 0:
                            if transaction_type == 'cat_pending_resolving':
                                percentage = (resolving_at_height / total_pending) * 100
                            else:  # cat_pending_postponed
                                percentage = (postponed_at_height / total_pending) * 100
                        else:
                            percentage = 0
                        
                    else:  # sumtypes
                        # For sumtypes, combine CAT and regular data
                        cat_success_data = result.get('chain_1_cat_success', [])
                        cat_failure_data = result.get('chain_1_cat_failure', [])
                        cat_pending_data = result.get('chain_1_cat_pending', [])
                        regular_success_data = result.get('chain_1_regular_success', [])
                        regular_failure_data = result.get('chain_1_regular_failure', [])
                        regular_pending_data = result.get('chain_1_regular_pending', [])
                        
                        # Convert to height->count mapping
                        cat_success_by_height = {entry[0]: entry[1] for entry in cat_success_data}
                        cat_failure_by_height = {entry[0]: entry[1] for entry in cat_failure_data}
                        cat_pending_by_height = {entry[0]: entry[1] for entry in cat_pending_data}
                        regular_success_by_height = {entry[0]: entry[1] for entry in regular_success_data}
                        regular_failure_by_height = {entry[0]: entry[1] for entry in regular_failure_data}
                        regular_pending_by_height = {entry[0]: entry[1] for entry in regular_pending_data}
                        
                        # Get combined counts at this specific height (CAT + regular)
                        success_at_height = cat_success_by_height.get(height, 0) + regular_success_by_height.get(height, 0)
                        failure_at_height = cat_failure_by_height.get(height, 0) + regular_failure_by_height.get(height, 0)
                        pending_at_height = cat_pending_by_height.get(height, 0) + regular_pending_by_height.get(height, 0)
                    
                    # Calculate percentage based on type
                    if percentage_type in ['success', 'failure']:
                        # For success and failure, use only (success + failure) as denominator
                        success_failure_total = success_at_height + failure_at_height
                        if success_failure_total > 0:
                            if percentage_type == 'success':
                                percentage = (success_at_height / success_failure_total) * 100
                            elif percentage_type == 'failure':
                                percentage = (failure_at_height / success_failure_total) * 100
                            else:
                                percentage = 0
                        else:
                            percentage = 0
                    else:
                        # For pending, use (success + pending + failure) as denominator
                        total = success_at_height + pending_at_height + failure_at_height
                        if total > 0:
                            percentage = (pending_at_height / total) * 100
                        else:
                            percentage = 0
                    
                    percentages.append(percentage)
                
                # Create title and filename based on transaction type
                if transaction_type == 'cat':
                    if percentage_type in ['success', 'failure']:
                        title = f'CAT {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    else:
                        title = f'CAT {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    filename = f'tx_{percentage_type}_cat_percentage_cutoff.png'
                elif transaction_type == 'cat_pending_resolving':
                    title = f'CAT Pending Resolving Percentage (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    filename = f'tx_pending_cat_resolving_percentage_cutoff.png'
                elif transaction_type == 'cat_pending_postponed':
                    title = f'CAT Pending Postponed Percentage (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    filename = f'tx_pending_cat_postponed_percentage_cutoff.png'
                elif transaction_type == 'regular':
                    if percentage_type in ['success', 'failure']:
                        title = f'Regular {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    else:
                        title = f'Regular {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    filename = f'tx_{percentage_type}_regular_percentage_cutoff.png'
                else:
                    if percentage_type in ['success', 'failure']:
                        title = f'{transaction_type.title()} {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    else:
                        title = f'{transaction_type.title()} {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    filename = f'tx_{percentage_type}_{transaction_type}_percentage_cutoff.png'
                
                # Trim the last 10% of data to avoid edge effects
                if len(heights) > 10:
                    trim_index = int(len(heights) * 0.9)
                    heights = heights[:trim_index]
                    percentages = percentages[:trim_index]
                
                if not heights:
                    continue
                
                # Update maximum height
                if heights:
                    max_height = max(max_height, max(heights))
                
                # Plot with color based on parameter
                from plot_utils_percentage import create_parameter_label
                label = create_parameter_label(param_name, param_value)
                plt.plot(heights, percentages, color=colors[i], alpha=0.7, 
                        label=label, linewidth=1.5)
            
            # Set x-axis limits before finalizing the plot
            plt.xlim(left=0, right=max_height)
            
            plt.title(title)
            plt.xlabel('Block Height')
            plt.ylabel(f'{percentage_type.title()} Percentage (%)')
            plt.grid(True, alpha=0.3)
            plt.legend(loc="upper right")
            plt.tight_layout()
            
            # Create tx_count_cutoff directory and save plot
            tx_count_cutoff_dir = f'{results_dir}/figs/tx_count_cutoff'
            os.makedirs(tx_count_cutoff_dir, exist_ok=True)
            plt.savefig(f'{tx_count_cutoff_dir}/{filename}', 
                       dpi=300, bbox_inches='tight')
            plt.close()
            
        except Exception as e:
            print(f"Warning: Error creating {transaction_type} {percentage_type} percentage cutoff plot: {e}")
            return
    
    # Call the modified function
    plot_transaction_percentage_cutoff_save(cutoff_data, param_name, results_dir, sweep_type, transaction_type, percentage_type)



 