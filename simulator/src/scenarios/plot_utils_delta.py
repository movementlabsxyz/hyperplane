"""
Delta plotting utilities for sweep simulations.

This module contains functions for calculating and plotting transaction deltas
(rate of change) in sweep simulations.
"""

import os
import json
import matplotlib.pyplot as plt
import numpy as np
from typing import Dict, Any, List, Tuple


def calculate_delta_from_counts(count_data: List[Tuple[int, int]]) -> List[Tuple[int, int]]:
    """
    Calculate delta (rate of change) from count data.
    
    Args:
        count_data: List of (height, count) tuples
        
    Returns:
        List of (height, delta) tuples representing the rate of change
    """
    if len(count_data) < 2:
        return []
    
    delta_data = []
    
    for i in range(1, len(count_data)):
        prev_height, prev_count = count_data[i-1]
        curr_height, curr_count = count_data[i]
        
        # Calculate delta as the difference in counts
        delta = curr_count - prev_count
        
        # Use the current height for the delta point
        delta_data.append((curr_height, delta))
    
    return delta_data


def plot_transactions_delta_overlay(
    data: Dict[str, Any],
    param_name: str,
    transaction_type: str,
    results_dir: str,
    sweep_type: str
) -> None:
    """
    Plot transaction delta overlay.
    
    Args:
        data: Sweep data containing individual results
        param_name: Name of the parameter being swept
        transaction_type: Type of transaction to plot
        results_dir: Directory to save plots
        sweep_type: Type of sweep simulation
    """
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
    """Plot CAT pending resolving delta."""
    plot_transactions_delta_overlay(data, param_name, 'cat_pending_resolving', results_dir, sweep_type)


def plot_cat_pending_postponed_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending postponed delta."""
    plot_transactions_delta_overlay(data, param_name, 'cat_pending_postponed', results_dir, sweep_type)


# Import helper functions from plot_utils.py
def extract_parameter_value(result: Dict[str, Any], param_name: str) -> float:
    """Extract parameter value from result dictionary."""
    from plot_utils import extract_parameter_value as _extract_parameter_value
    return _extract_parameter_value(result, param_name)


def create_parameter_label(param_name: str, param_value: float) -> str:
    """Create parameter label for plotting."""
    from plot_utils import create_parameter_label as _create_parameter_label
    return _create_parameter_label(param_name, param_value)


def create_sweep_title(param_name: str, sweep_type: str) -> str:
    """Create sweep title for plotting."""
    from plot_utils import create_sweep_title as _create_sweep_title
    return _create_sweep_title(param_name, sweep_type)


def create_color_gradient(num_simulations: int) -> np.ndarray:
    """Create color gradient for plotting."""
    from plot_utils import create_color_gradient as _create_color_gradient
    return _create_color_gradient(num_simulations)


def trim_time_series_data(time_series_data: List[Tuple[int, int]], cutoff_percentage: float = 0.1) -> List[Tuple[int, int]]:
    """Trim time series data to avoid edge effects."""
    from plot_utils import trim_time_series_data as _trim_time_series_data
    return _trim_time_series_data(time_series_data, cutoff_percentage) 