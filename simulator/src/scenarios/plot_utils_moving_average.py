"""
Moving average plotting utilities for sweep simulations.

This module contains functions for applying moving average smoothing to transaction data
and generating moving average plots.
"""

import os
import matplotlib.pyplot as plt
import numpy as np
from typing import Dict, Any, List, Tuple


def apply_moving_average(data: List[Tuple[int, int]], window_size: int) -> List[Tuple[int, float]]:
    """
    Apply moving average smoothing to time-series data.
    
    Args:
        data: List of (height, count) tuples
        window_size: Size of the moving average window
        
    Returns:
        Smoothed data with moving average applied (float values for precision)
    """
    if not data or len(data) < window_size:
        return data
    
    smoothed_data = []
    
    for i in range(len(data)):
        # Calculate the start and end indices for the window
        start_idx = max(0, i - window_size // 2)
        end_idx = min(len(data), i + window_size // 2 + 1)
        
        # Extract the window of data
        window_data = data[start_idx:end_idx]
        
        # Calculate the average count for this window
        total_count = sum(count for _, count in window_data)
        avg_count = total_count / len(window_data)
        
        # Use the original height and the averaged count (as float for precision)
        height = data[i][0]
        smoothed_data.append((height, avg_count))
    
    return smoothed_data


def plot_transactions_overlay_with_moving_average(
    data: Dict[str, Any],
    param_name: str,
    transaction_type: str,
    results_dir: str,
    sweep_type: str,
    plot_config: Dict[str, Any]
) -> None:
    """
    Plot transaction overlay with moving average applied.
    
    Args:
        data: Sweep data containing individual results
        param_name: Name of the parameter being swept
        transaction_type: Type of transaction to plot
        results_dir: Directory to save plots
        sweep_type: Type of sweep simulation
        plot_config: Plot configuration containing moving average settings
    """
    try:
        # Extract parameter values and results
        param_values = []
        results = []
        
        for result in data['individual_results']:
            param_value = extract_parameter_value(result, param_name)
            param_values.append(param_value)
            results.append(result)
        
        # Create color gradient
        colors = create_color_gradient(len(param_values))
        
        # Create the plot
        plt.figure(figsize=(10, 6))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Get moving average window size from config
        window_size = plot_config.get('range_moving_average', 10)
        
        # Plot each parameter value
        for i, (param_value, result) in enumerate(zip(param_values, results)):
            # Extract transaction data
            if transaction_type.startswith('cat_'):
                # CAT transactions
                status = transaction_type.replace('cat_', '')
                tx_data = result.get(f'chain_1_cat_{status}', [])
            elif transaction_type.startswith('regular_'):
                # Regular transactions
                status = transaction_type.replace('regular_', '')
                tx_data = result.get(f'chain_1_regular_{status}', [])
            else:
                # Fallback
                tx_data = result.get(f'chain_1_{transaction_type}', [])
            
            if not tx_data:
                continue
            
            # Apply moving average to the data
            smoothed_data = apply_moving_average(tx_data, window_size)
            
            if not smoothed_data:
                continue
            
            # Extract heights and counts
            heights = [entry[0] for entry in smoothed_data]
            counts = [entry[1] for entry in smoothed_data]
            
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
        if transaction_type.startswith('cat_'):
            # CAT transactions
            status = transaction_type.replace('cat_', '')
            if status in ['success', 'failure']:
                title = f'CAT {status.title()} Transaction Counts (Moving Average) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat_moving_average.png'
            else:
                title = f'CAT {status.title()} Transaction Counts (Moving Average) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat_moving_average.png'
        elif transaction_type.startswith('regular_'):
            # Regular transactions
            status = transaction_type.replace('regular_', '')
            title = f'Regular {status.title()} Transaction Counts (Moving Average) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{status}_regular_moving_average.png'
        else:
            # Fallback
            title = f'{transaction_type.title()} Transaction Counts (Moving Average) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}_moving_average.png'
        
        plt.title(title, fontsize=14)
        plt.xlabel('Block Height', fontsize=12)
        plt.ylabel('Transaction Count (Smoothed)', fontsize=12)
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right", fontsize=10)
        plt.tight_layout()
        
        # Create tx_count/moving_average directory and save plot
        tx_count_ma_dir = f'{results_dir}/figs/tx_count/moving_average'
        os.makedirs(tx_count_ma_dir, exist_ok=True)
        plt.savefig(f'{tx_count_ma_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
        print(f"Generated moving average plot: {filename}")
        
    except Exception as e:
        print(f"Error generating moving average plot for {transaction_type}: {e}")
        import traceback
        traceback.print_exc()


def plot_transactions_delta_overlay_with_moving_average(
    data: Dict[str, Any],
    param_name: str,
    transaction_type: str,
    results_dir: str,
    sweep_type: str,
    plot_config: Dict[str, Any]
) -> None:
    """
    Plot transaction delta overlay with moving average applied.
    
    Args:
        data: Sweep data containing individual results
        param_name: Name of the parameter being swept
        transaction_type: Type of transaction to plot
        results_dir: Directory to save plots
        sweep_type: Type of sweep simulation
        plot_config: Plot configuration containing moving average settings
    """
    try:
        # Extract parameter values and results
        param_values = []
        results = []
        
        for result in data['individual_results']:
            param_value = extract_parameter_value(result, param_name)
            param_values.append(param_value)
            results.append(result)
        
        # Create color gradient
        colors = create_color_gradient(len(param_values))
        
        # Create the plot
        plt.figure(figsize=(10, 6))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Get moving average window size from config
        window_size = plot_config.get('range_moving_average', 10)
        
        # Plot each parameter value
        for i, (param_value, result) in enumerate(zip(param_values, results)):
            # Extract transaction data
            if transaction_type.startswith('cat_'):
                # CAT transactions
                status = transaction_type.replace('cat_', '')
                tx_data = result.get(f'chain_1_cat_{status}', [])
            elif transaction_type.startswith('regular_'):
                # Regular transactions
                status = transaction_type.replace('regular_', '')
                tx_data = result.get(f'chain_1_regular_{status}', [])
            else:
                # Fallback
                tx_data = result.get(f'chain_1_{transaction_type}', [])
            
            if not tx_data:
                continue
            
            # Calculate delta from counts
            delta_data = calculate_delta_from_counts(tx_data)
            
            if not delta_data:
                continue
            
            # Apply moving average to the delta data
            smoothed_data = apply_moving_average(delta_data, window_size)
            
            if not smoothed_data:
                continue
            
            # Extract heights and deltas
            heights = [entry[0] for entry in smoothed_data]
            deltas = [entry[1] for entry in smoothed_data]
            
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
        if transaction_type.startswith('cat_'):
            # CAT transactions
            status = transaction_type.replace('cat_', '')
            if status in ['success', 'failure']:
                title = f'CAT {status.title()} Transaction Delta (Moving Average) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat_delta_moving_average.png'
            else:
                title = f'CAT {status.title()} Transaction Delta (Moving Average) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat_delta_moving_average.png'
        elif transaction_type.startswith('regular_'):
            # Regular transactions
            status = transaction_type.replace('regular_', '')
            title = f'Regular {status.title()} Transaction Delta (Moving Average) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{status}_regular_delta_moving_average.png'
        else:
            # Fallback
            title = f'{transaction_type.title()} Transaction Delta (Moving Average) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}_delta_moving_average.png'
        
        plt.title(title, fontsize=14)
        plt.xlabel('Block Height', fontsize=12)
        plt.ylabel('Transaction Delta (Smoothed)', fontsize=12)
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right", fontsize=10)
        plt.tight_layout()
        
        # Create tx_count_delta/moving_average directory and save plot
        tx_count_delta_ma_dir = f'{results_dir}/figs/tx_count_delta/moving_average'
        os.makedirs(tx_count_delta_ma_dir, exist_ok=True)
        plt.savefig(f'{tx_count_delta_ma_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
        print(f"Generated delta moving average plot: {filename}")
        
    except Exception as e:
        print(f"Error generating delta moving average plot for {transaction_type}: {e}")
        import traceback
        traceback.print_exc()


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


def calculate_delta_from_counts(count_data: List[Tuple[int, int]]) -> List[Tuple[int, int]]:
    """Calculate delta from count data."""
    from plot_utils_delta import calculate_delta_from_counts as _calculate_delta_from_counts
    return _calculate_delta_from_counts(count_data) 