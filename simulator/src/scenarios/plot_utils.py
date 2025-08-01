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
from plot_utils_moving_average import apply_moving_average

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
    'cat_ratio': 'CAT Ratio',
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
    elif param_name == 'cat_ratio':
        return f'CAT Ratio: {param_value:.3f}'
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


def plot_total_cat_transactions(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot total CAT transactions (success + failure + pending) over time."""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping total CAT transactions plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's total CAT transactions
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get CAT success, failure, and pending data
            cat_success_data = result.get('chain_1_cat_success', [])
            cat_failure_data = result.get('chain_1_cat_failure', [])
            cat_pending_data = result.get('chain_1_cat_pending', [])
            
            # Create a combined dataset by summing all CAT transactions at each height
            combined_data = {}
            
            # Add CAT success data
            for height, count in cat_success_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add CAT failure data
            for height, count in cat_failure_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add CAT pending data
            for height, count in cat_pending_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Convert back to sorted list of tuples
            chain_data = sorted(combined_data.items())
            
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
        
        # Create title and filename
        title = f'Total CAT Transactions (Success + Failure + Pending) by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
        filename = 'tx_sumStates_cat.png'
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel('Total CAT Transactions')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper left")
        plt.tight_layout()
        
        # Create tx_count directory and save plot
        tx_count_dir = f'{results_dir}/figs/tx_count'
        os.makedirs(tx_count_dir, exist_ok=True)
        plt.savefig(f'{tx_count_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error generating total CAT transactions plot: {e}")
        import traceback
        traceback.print_exc()


def plot_total_regular_transactions(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot total regular transactions (success + failure + pending) over time."""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping total regular transactions plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's total regular transactions
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get regular success, failure, and pending data
            regular_success_data = result.get('chain_1_regular_success', [])
            regular_failure_data = result.get('chain_1_regular_failure', [])
            regular_pending_data = result.get('chain_1_regular_pending', [])
            
            # Create a combined dataset by summing all regular transactions at each height
            combined_data = {}
            
            # Add regular success data
            for height, count in regular_success_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add regular failure data
            for height, count in regular_failure_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add regular pending data
            for height, count in regular_pending_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Convert back to sorted list of tuples
            chain_data = sorted(combined_data.items())
            
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
        
        # Create title and filename
        title = f'Total Regular Transactions (Success + Failure + Pending) by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
        filename = 'tx_sumStates_regular.png'
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel('Total Regular Transactions')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper left")
        plt.tight_layout()
        
        # Create tx_count directory and save plot
        tx_count_dir = f'{results_dir}/figs/tx_count'
        os.makedirs(tx_count_dir, exist_ok=True)
        plt.savefig(f'{tx_count_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error generating total regular transactions plot: {e}")
        import traceback
        traceback.print_exc()


def plot_total_sumtypes_transactions(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot total sumtypes transactions (CAT + regular) over time."""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping total sumtypes transactions plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's total sumtypes transactions
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get CAT success, failure, and pending data
            cat_success_data = result.get('chain_1_cat_success', [])
            cat_failure_data = result.get('chain_1_cat_failure', [])
            cat_pending_data = result.get('chain_1_cat_pending', [])
            
            # Get regular success, failure, and pending data
            regular_success_data = result.get('chain_1_regular_success', [])
            regular_failure_data = result.get('chain_1_regular_failure', [])
            regular_pending_data = result.get('chain_1_regular_pending', [])
            
            # Create a combined dataset by summing all transactions at each height
            combined_data = {}
            
            # Add CAT success data
            for height, count in cat_success_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add CAT failure data
            for height, count in cat_failure_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add CAT pending data
            for height, count in cat_pending_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add regular success data
            for height, count in regular_success_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add regular failure data
            for height, count in regular_failure_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add regular pending data
            for height, count in regular_pending_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Convert back to sorted list of tuples
            chain_data = sorted(combined_data.items())
            
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
        
        # Create title and filename
        title = f'Total SumTypes Transactions (CAT + Regular) by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
        filename = 'tx_sumStates_sumTypes.png'
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel('Total SumTypes Transactions')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper left")
        plt.tight_layout()
        
        # Create tx_count directory and save plot
        tx_count_dir = f'{results_dir}/figs/tx_count'
        os.makedirs(tx_count_dir, exist_ok=True)
        plt.savefig(f'{tx_count_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error generating total sumtypes transactions plot: {e}")
        import traceback
        traceback.print_exc()


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
        results_dir: The full path to the results directory (e.g., 'simulator/results/sim_sweep_cat_ratio')
        param_name: The parameter name being swept (e.g., 'cat_ratio')
        sweep_type: The display name for the sweep (e.g., 'CAT Ratio')
    """
    print(f"DEBUG: generate_all_plots called with results_dir={results_dir}, param_name={param_name}, sweep_type={sweep_type}")
    
    # Extract the results directory name from the full path
    results_dir_name = results_dir.replace('simulator/results/', '')
    
    # First, run the averaging script to create averaged data from run_average folders
    print("Running averaging script...")
    try:
        # Import and call the averaging function directly
        import sys
        sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
        from average_runs import create_averaged_data
        
        # Use relative path from simulator directory (where the script runs from)
        results_path = f'simulator/results/{results_dir_name}'
        
        # Debug: Print current working directory and the path being passed
        print(f"DEBUG: Current working directory: {os.getcwd()}")
        print(f"DEBUG: Results path being passed to averaging: {results_path}")
        print(f"DEBUG: Full path to metadata: {os.path.join(os.getcwd(), results_path, 'data', 'metadata.json')}")
        print(f"DEBUG: Full path exists: {os.path.exists(os.path.join(os.getcwd(), results_path, 'data', 'metadata.json'))}")
        
        # Call the averaging function directly
        success = create_averaged_data(results_path)
        
        if success:
            print("Averaging completed successfully!")
        else:
            print("Averaging failed!")
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
    
    # Import and run the paper plot script if it exists
    plot_paper_path = os.path.join(os.path.dirname(__file__), results_dir_name, 'plot_paper.py')
    print(f"DEBUG: Looking for paper plot script at: {plot_paper_path}")
    print(f"DEBUG: Paper plot script exists: {os.path.exists(plot_paper_path)}")
    
    if os.path.exists(plot_paper_path):
        print("Generating paper plots...")
        
        try:
            # Import the paper plot module and call its main function
            sys.path.insert(0, os.path.dirname(plot_paper_path))
            
            # Import the paper plot module
            module_name = os.path.basename(results_dir_name)
            print(f"DEBUG: Importing module: {module_name}.plot_paper")
            paper_module = __import__(f"{module_name}.plot_paper", fromlist=['plot_cat_success_percentage_with_overlay', 'plot_cat_success_percentage_violin_paper', 'plot_cat_success_percentage_violin', 'plot_tx_pending_cat_postponed_violin', 'plot_tx_pending_cat_resolving_violin', 'plot_tx_pending_regular_violin'])
            
            # Call the individual paper plot functions directly
            print(f"DEBUG: Calling paper plot functions with data keys: {list(data.keys())}")
            paper_module.plot_cat_success_percentage_with_overlay(data, param_name, results_dir, sweep_type)
            paper_module.plot_cat_success_percentage_violin_paper(data, param_name, results_dir, sweep_type, plot_config)
            paper_module.plot_cat_success_percentage_violin(data, param_name, results_dir, sweep_type, plot_config)
            print("DEBUG: About to run postponed violin plot...")
            paper_module.plot_tx_pending_cat_postponed_violin(data, param_name, results_dir, sweep_type, plot_config)
            print("DEBUG: About to run resolving violin plot...")
            paper_module.plot_tx_pending_cat_resolving_violin(data, param_name, results_dir, sweep_type, plot_config)
            paper_module.plot_tx_pending_regular_violin(data, param_name, results_dir, sweep_type, plot_config)
            
            print("Paper plots generated successfully!")
        except Exception as e:
            print(f"Error generating paper plots: {e}")
            import traceback
            traceback.print_exc()
    else:
        print(f"No plot_paper.py found at {plot_paper_path} - skipping paper plots")
    
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
    * `results_dir` - Directory name of the sweep (e.g., 'sim_sweep_cat_ratio')
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
    * `results_dir` - Directory name of the sweep (e.g., 'sim_sweep_cat_ratio')
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
    Plot transactions per block (TPB) for sweep simulations.
    
    # Arguments
    * `data` - The sweep data containing individual results
    * `param_name` - Name of the parameter being swept
    * `results_dir` - Directory name of the sweep (e.g., 'sim_sweep_cat_ratio')
    * `sweep_type` - Type of sweep simulation
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping transactions per block overlay plot")
            return
        
        # Create single figure for TPB
        fig, ax = plt.subplots(1, 1, figsize=(12, 6))
        
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
            
            # Get target TPB from simulation stats
            try:
                # Extract just the directory name from the full path
                results_dir_name = results_dir.replace('simulator/results/', '')
                # Use simulation_stats.json from the first simulation's run_average directory
                stats_file = f'simulator/results/{results_dir_name}/data/sim_0/run_average/simulation_stats.json'
                with open(stats_file, 'r') as f:
                    stats_data = json.load(f)
                target_tpb = stats_data['parameters']['target_tpb']
            except (FileNotFoundError, KeyError) as e:
                print(f"Warning: Could not determine target_tpb from simulation stats: {e}")
                target_tpb = None
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            ax.plot(heights, tx_per_block, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Create title and add target TPB line if available
        title = f'Transactions per Block (TPB) - {create_sweep_title(param_name, sweep_type)}'
        ax.set_title(title)
        ax.set_xlabel('Block Height')
        ax.set_ylabel('Number of Transactions')
        ax.grid(True, alpha=0.3)
        ax.legend(loc="upper right")
        
        # Add target TPB line if available
        if target_tpb is not None:
            ax.axhline(y=target_tpb, color='g', linestyle=':', label=f'Target TPB: {target_tpb}', linewidth=2)
            ax.legend(loc="upper right")
        
        plt.tight_layout()
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/tpb.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error processing transactions per block data for sweep: {e}")
        return

def plot_sweep_tpb_moving_average(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, plot_config: Dict[str, Any]) -> None:
    """Plot TPB (Transactions Per Block) with moving average for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping TPB moving average plot")
            return
        
        # Set moving average window size
        window_size = 100
        
        # Create figure
        fig, ax = plt.subplots(figsize=(10, 6))
        
        # Get parameter values for coloring
        param_values = [result[param_name] for result in individual_results]
        colors = create_color_gradient(len(param_values))
        
        # Plot each simulation's TPB data with moving average
        missing_files = []
        for sim_index, (result, color) in enumerate(zip(individual_results, colors)):
            param_value = result[param_name]
            label = create_parameter_label(param_name, param_value)
            
            # Load TPB data for this simulation
            # Extract just the directory name from the full path
            results_dir_name = results_dir.replace('simulator/results/', '')
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{sim_index}'
            
            # Use averaged TPB data
            tx_per_block_file = f'{sim_data_dir}/run_average/tx_per_block_chain_1.json'
            if os.path.exists(tx_per_block_file):
                with open(tx_per_block_file, 'r') as f:
                    tx_per_block_data = json.load(f)
                
                # Extract TPB data
                if 'chain_1_tx_per_block' in tx_per_block_data:
                    tx_per_block_entries = tx_per_block_data['chain_1_tx_per_block']
                    if tx_per_block_entries:
                        # Extract block heights and transaction counts
                        heights = [entry['height'] for entry in tx_per_block_entries]
                        tx_counts = [entry['count'] for entry in tx_per_block_entries]
                        
                        # Ensure heights and tx_counts have the same length
                        if len(heights) != len(tx_counts):
                            print(f"Warning: Heights ({len(heights)}) and transaction counts ({len(tx_counts)}) have different lengths for simulation {sim_index}")
                            # Use the shorter length
                            min_length = min(len(heights), len(tx_counts))
                            heights = heights[:min_length]
                            tx_counts = tx_counts[:min_length]
                        
                        # Use transaction counts directly as TPB (no calculation needed)
                        tpb_values = tx_counts
                        
                        # Apply moving average
                        if len(heights) >= window_size:
                            # Convert to list of tuples for moving average function
                            data_points = list(zip(heights, tpb_values))
                            smoothed_data = apply_moving_average(data_points, window_size)
                            
                            # Extract smoothed heights and values
                            smoothed_heights = [point[0] for point in smoothed_data]
                            smoothed_values = [point[1] for point in smoothed_data]
                            
                            # Plot the smoothed data
                            ax.plot(smoothed_heights, smoothed_values, color=color, alpha=0.7, linewidth=2)
                        else:
                            print(f"Warning: Not enough data points for moving average (window={window_size}) for simulation {sim_index}")
                            # Plot original data if not enough points
                            ax.plot(heights, tpb_values, color=color, alpha=0.7, linewidth=2)
                    else:
                        print(f"Warning: No transaction per block entries found for simulation {sim_index}")
                else:
                    print(f"Warning: No chain_1_tx_per_block key found in {tx_per_block_file}")
            else:
                missing_files.append(tx_per_block_file)
            
            # Add legend entry for this parameter value
            ax.plot([], [], color=color, label=label, linewidth=2)
        
        # Print summary warning for missing files
        if missing_files:
            print(f"Warning: {len(missing_files)} tx_per_block_chain_1.json files not found across all simulations")
        
        # Customize plot
        ax.set_xlabel('Block Height')
        ax.set_ylabel('TPB (Moving Average)')
        ax.set_title(f'Transactions Per Block Over Time (Moving Average, Window={window_size}) by {create_sweep_title(param_name, sweep_type)}')
        ax.legend(loc="upper right")
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/tpb_moving_average.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting TPB moving average data: {e}")
        import traceback
        traceback.print_exc()

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
            # print(f"Generating individual curves plots for simulation {sim_index}...")
            
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
            
        # print(f"Individual curves plots generated for all simulations in {sweep_type} sweep!")
        
    except Exception as e:
        print(f"Error generating individual curves plots: {e}")
        import traceback
        traceback.print_exc()



def run_sweep_plots(sweep_name: str, param_name: str, sweep_type: str) -> None:
    """
    Generic function to run plots for any sweep simulation.
    
    This function:
    1. Runs the averaging script to create run_average folders from individual runs
    2. Loads data directly from run_average folders for plotting
    3. Generates all plots from the run_average data
    
    Args:
        sweep_name: The name of the sweep directory (e.g., 'sim_sweep_cat_ratio')
        param_name: The parameter name being swept (e.g., 'cat_ratio')
        sweep_type: The display name for the sweep (e.g., 'CAT Ratio')
    """
    print(f"DEBUG: run_sweep_plots called with sweep_name={sweep_name}, param_name={param_name}, sweep_type={sweep_type}")
    results_dir = f'simulator/results/{sweep_name}'
    print(f"DEBUG: results_dir = {results_dir}")
    
    # Generate all plots using the generic utility
    # Data flow: run_average folders -> plots
    print(f"DEBUG: About to call generate_all_plots...")
    generate_all_plots(results_dir, param_name, sweep_type)
    print(f"DEBUG: generate_all_plots completed")

def load_plot_config(results_dir: str) -> Dict[str, Any]:
    """
    Load plot configuration from the results data directory or fallback to source config.
    
    Args:
        results_dir: The results directory path
        
    Returns:
        Dictionary containing plot configuration settings
        
    Raises:
        FileNotFoundError: If the config file doesn't exist
        Exception: If there's an error parsing the TOML or missing required parameters
    """
    # Extract sweep name from results_dir (e.g., 'sim_sweep_cat_ratio' from 'simulator/results/sim_sweep_cat_ratio')
    sweep_name = results_dir.split('/')[-1]
    
    # Try different possible paths for the config file, prioritizing results data directory
    possible_paths = [
        f'{results_dir}/data/config.toml',  # Primary: results data directory (actual simulation config)
        'config.toml',  # Fallback: when called from paper plot script (working directory is scenario directory)
        f'simulator/src/scenarios/{sweep_name}/config.toml',  # Fallback: when called from main plotting functions
    ]
    
    config_path = None
    for path in possible_paths:
        if os.path.exists(path):
            config_path = path
            break
    
    if config_path is None:
        raise FileNotFoundError(f"Config file not found. Tried: {possible_paths}")
    
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







 