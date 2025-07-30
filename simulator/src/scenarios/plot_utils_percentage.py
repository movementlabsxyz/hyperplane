#!/usr/bin/env python3
"""
CAT percentage plotting utilities for Hyperplane simulator sweep results.
"""

import os
import sys
import json
import matplotlib.pyplot as plt
import numpy as np
from typing import Dict, List, Tuple, Any

# Global colormap setting - easily switch between different colormaps
# Options: 'viridis', 'RdYlBu_r', 'plasma', 'inferno', 'magma', 'cividis'
COLORMAP = 'viridis'  # Change this to switch colormaps globally

def create_color_gradient(num_simulations: int) -> np.ndarray:
    """Create a color gradient using the global COLORMAP setting"""
    return plt.cm.get_cmap(COLORMAP)(np.linspace(0, 1, num_simulations))

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
    param_display = param_name.replace('_', ' ').title()
    # Remove units in parentheses for cleaner titles
    param_display = param_display.split(' (')[0]
    return f'{param_display} Sweep'

def plot_transaction_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, transaction_type: str, percentage_type: str) -> None:
    """
    Plot transaction percentage over time for each simulation.
    
    This function calculates the percentage of a specific transaction state (success, pending, or failure)
    as block number increases.
    
    For success and failure percentages: uses (success + failure) as denominator
    For pending percentage: uses (success + pending + failure) as denominator
    
    Args:
        data: The sweep data containing individual results
        param_name: Name of the parameter being swept
        results_dir: Directory to save the plot
        sweep_type: Type of sweep simulation
        transaction_type: One of 'cat', 'regular', or 'sumtypes'
        percentage_type: One of 'success', 'pending', or 'failure'
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
                    title = f'CAT {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
                else:
                    title = f'CAT {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{percentage_type}_cat_percentage.png'
            elif transaction_type == 'cat_pending_resolving':
                title = f'CAT Pending Resolving Percentage (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_resolving_percentage.png'
            elif transaction_type == 'cat_pending_postponed':
                title = f'CAT Pending Postponed Percentage (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_pending_cat_postponed_percentage.png'
            elif transaction_type == 'regular':
                if percentage_type in ['success', 'failure']:
                    title = f'Regular {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
                else:
                    title = f'Regular {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{percentage_type}_regular_percentage.png'
            else:
                if percentage_type in ['success', 'failure']:
                    title = f'{transaction_type.title()} {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
                else:
                    title = f'{transaction_type.title()} {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{percentage_type}_{transaction_type}_percentage.png'
        
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
        
        # Create tx_count directory and save plot
        tx_count_dir = f'{results_dir}/figs/tx_count'
        os.makedirs(tx_count_dir, exist_ok=True)
        plt.savefig(f'{tx_count_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error creating {transaction_type} {percentage_type} percentage plot: {e}")
        return

# Wrapper functions for backward compatibility
def plot_cat_success_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT success percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat', 'success')

def plot_cat_failure_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT failure percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat', 'failure')

def plot_cat_pending_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat', 'pending')

def plot_regular_success_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot regular success percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'regular', 'success')

def plot_regular_failure_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot regular failure percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'regular', 'failure')

def plot_regular_pending_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot regular pending percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'regular', 'pending')

def plot_sumtypes_success_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot sumTypes success percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'sumtypes', 'success')

def plot_sumtypes_failure_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot sumTypes failure percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'sumtypes', 'failure')

def plot_sumtypes_pending_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot sumTypes pending percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'sumtypes', 'pending')

def plot_cat_pending_resolving_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending resolving percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat_pending_resolving', 'pending')

def plot_cat_pending_postponed_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending postponed percentage over time."""
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat_pending_postponed', 'pending')

def plot_transaction_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, transaction_type: str, percentage_type: str) -> None:
    """
    Plot transaction percentage delta over time for each simulation.
    
    This function calculates the delta of percentage of a specific transaction state (success, pending, or failure)
    as block number increases.
    
    For success and failure percentages: uses (success + failure) as denominator
    For pending percentage: uses (success + pending + failure) as denominator
    
    Args:
        data: The sweep data containing individual results
        param_name: Name of the parameter being swept
        results_dir: Directory to save the plot
        sweep_type: Type of sweep simulation
        transaction_type: One of 'cat', 'regular', or 'sumtypes'
        percentage_type: One of 'success', 'pending', or 'failure'
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
            elif transaction_type == 'cat_pending_resolving':
                # For CAT pending resolving transactions
                tx_data = result.get('chain_1_cat_pending_resolving', [])
            elif transaction_type == 'cat_pending_postponed':
                # For CAT pending postponed transactions
                tx_data = result.get('chain_1_cat_pending_postponed', [])
            elif transaction_type == 'regular':
                # For regular transactions, get the specific status data
                if percentage_type == 'success':
                    tx_data = result.get('chain_1_regular_success', [])
                elif percentage_type == 'failure':
                    tx_data = result.get('chain_1_regular_failure', [])
                else:  # pending
                    tx_data = result.get('chain_1_regular_pending', [])
            elif transaction_type == 'sumtypes':
                # For sumTypes (combined CAT + regular), get the specific status data
                if percentage_type == 'success':
                    cat_success = result.get('chain_1_cat_success', [])
                    regular_success = result.get('chain_1_regular_success', [])
                    # Combine CAT and regular success data
                    combined_data = {}
                    for height, count in cat_success:
                        combined_data[height] = combined_data.get(height, 0) + count
                    for height, count in regular_success:
                        combined_data[height] = combined_data.get(height, 0) + count
                    tx_data = sorted(combined_data.items())
                elif percentage_type == 'failure':
                    cat_failure = result.get('chain_1_cat_failure', [])
                    regular_failure = result.get('chain_1_regular_failure', [])
                    # Combine CAT and regular failure data
                    combined_data = {}
                    for height, count in cat_failure:
                        combined_data[height] = combined_data.get(height, 0) + count
                    for height, count in regular_failure:
                        combined_data[height] = combined_data.get(height, 0) + count
                    tx_data = sorted(combined_data.items())
                else:  # pending
                    cat_pending = result.get('chain_1_cat_pending', [])
                    regular_pending = result.get('chain_1_regular_pending', [])
                    # Combine CAT and regular pending data
                    combined_data = {}
                    for height, count in cat_pending:
                        combined_data[height] = combined_data.get(height, 0) + count
                    for height, count in regular_pending:
                        combined_data[height] = combined_data.get(height, 0) + count
                    tx_data = sorted(combined_data.items())
            else:
                # For other transaction types, use the data directly
                tx_data = result.get(f'chain_1_{transaction_type}', [])
            
            if not tx_data:
                continue
            
            # Calculate percentage over time
            heights = []
            percentages = []
            
            for height, count in tx_data:
                # Calculate total for denominator based on percentage type and transaction type
                if transaction_type in ['cat_pending_resolving', 'cat_pending_postponed']:
                    # For CAT pending resolving/postponed, use (resolving + postponed) as denominator
                    resolving_data = result.get('chain_1_cat_pending_resolving', [])
                    postponed_data = result.get('chain_1_cat_pending_postponed', [])
                    
                    # Find total at this height (resolving + postponed)
                    total_at_height = 0
                    for h, c in resolving_data:
                        if h == height:
                            total_at_height += c
                    for h, c in postponed_data:
                        if h == height:
                            total_at_height += c
                            
                elif percentage_type in ['success', 'failure']:
                    # For success/failure percentages, use (success + failure) as denominator
                    if transaction_type == 'cat':
                        success_data = result.get('chain_1_cat_success', [])
                        failure_data = result.get('chain_1_cat_failure', [])
                    elif transaction_type == 'regular':
                        success_data = result.get('chain_1_regular_success', [])
                        failure_data = result.get('chain_1_regular_failure', [])
                    elif transaction_type == 'sumtypes':
                        # For sumTypes, combine CAT and regular
                        cat_success = result.get('chain_1_cat_success', [])
                        cat_failure = result.get('chain_1_cat_failure', [])
                        regular_success = result.get('chain_1_regular_success', [])
                        regular_failure = result.get('chain_1_regular_failure', [])
                        success_data = cat_success + regular_success
                        failure_data = cat_failure + regular_failure
                    else:
                        success_data = result.get(f'chain_1_{transaction_type}_success', [])
                        failure_data = result.get(f'chain_1_{transaction_type}_failure', [])
                    
                    # Find total at this height
                    total_at_height = 0
                    for h, c in success_data:
                        if h == height:
                            total_at_height += c
                    for h, c in failure_data:
                        if h == height:
                            total_at_height += c
                else:
                    # For pending percentage, use (success + pending + failure) as denominator
                    if transaction_type == 'cat':
                        success_data = result.get('chain_1_cat_success', [])
                        pending_data = result.get('chain_1_cat_pending', [])
                        failure_data = result.get('chain_1_cat_failure', [])
                    elif transaction_type == 'regular':
                        success_data = result.get('chain_1_regular_success', [])
                        pending_data = result.get('chain_1_regular_pending', [])
                        failure_data = result.get('chain_1_regular_failure', [])
                    elif transaction_type == 'sumtypes':
                        # For sumTypes, combine CAT and regular
                        cat_success = result.get('chain_1_cat_success', [])
                        cat_pending = result.get('chain_1_cat_pending', [])
                        cat_failure = result.get('chain_1_cat_failure', [])
                        regular_success = result.get('chain_1_regular_success', [])
                        regular_pending = result.get('chain_1_regular_pending', [])
                        regular_failure = result.get('chain_1_regular_failure', [])
                        success_data = cat_success + regular_success
                        pending_data = cat_pending + regular_pending
                        failure_data = cat_failure + regular_failure
                    else:
                        success_data = result.get(f'chain_1_{transaction_type}_success', [])
                        pending_data = result.get(f'chain_1_{transaction_type}_pending', [])
                        failure_data = result.get(f'chain_1_{transaction_type}_failure', [])
                    
                    # Find total at this height
                    total_at_height = 0
                    for h, c in success_data:
                        if h == height:
                            total_at_height += c
                    for h, c in pending_data:
                        if h == height:
                            total_at_height += c
                    for h, c in failure_data:
                        if h == height:
                            total_at_height += c
                
                if total_at_height > 0:
                    percentage = (count / total_at_height) * 100
                    heights.append(height)
                    percentages.append(percentage)
            
            if not heights:
                continue
            
            # Calculate deltas from percentage data
            from plot_utils import calculate_delta_from_counts
            percentage_data = list(zip(heights, percentages))
            delta_data = calculate_delta_from_counts(percentage_data)
            
            if not delta_data:
                continue
            
            # Extract delta data
            delta_heights = [entry[0] for entry in delta_data]
            delta_percentages = [entry[1] for entry in delta_data]
            
            # Trim the last 10% of data to avoid edge effects
            if len(delta_heights) > 10:
                trim_index = int(len(delta_heights) * 0.9)
                delta_heights = delta_heights[:trim_index]
                delta_percentages = delta_percentages[:trim_index]
            
            if not delta_heights:
                continue
            
            # Update maximum height
            if delta_heights:
                max_height = max(max_height, max(delta_heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(delta_heights, delta_percentages, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        # Create title and filename based on transaction type and percentage type
        if transaction_type == 'cat':
            title = f'CAT {percentage_type.title()} Percentage Deltas Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_cat_percentage.png'
        elif transaction_type == 'cat_pending_resolving':
            title = f'CAT Pending Resolving Percentage Deltas (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_pending_cat_resolving_percentage.png'
        elif transaction_type == 'cat_pending_postponed':
            title = f'CAT Pending Postponed Percentage Deltas (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_pending_cat_postponed_percentage.png'
        elif transaction_type == 'regular':
            if percentage_type in ['success', 'failure']:
                title = f'Regular {percentage_type.title()} Percentage Deltas (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            else:
                title = f'Regular {percentage_type.title()} Percentage Deltas (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_regular_percentage.png'
        elif transaction_type == 'sumtypes':
            if percentage_type in ['success', 'failure']:
                title = f'SumTypes {percentage_type.title()} Percentage Deltas (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            else:
                title = f'SumTypes {percentage_type.title()} Percentage Deltas (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_sumtypes_percentage.png'
        else:
            if percentage_type in ['success', 'failure']:
                title = f'{transaction_type.title()} {percentage_type.title()} Percentage Deltas (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            else:
                title = f'{transaction_type.title()} {percentage_type.title()} Percentage Deltas (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_{transaction_type}_percentage.png'
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel(f'{percentage_type.title()} Percentage Delta (%)')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Create tx_count_delta directory and save plot
        tx_count_delta_dir = f'{results_dir}/figs/tx_count_delta'
        os.makedirs(tx_count_delta_dir, exist_ok=True)
        plt.savefig(f'{tx_count_delta_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error creating {transaction_type} {percentage_type} percentage delta plot: {e}")
        return

# Wrapper functions for delta percentage plots
def plot_cat_success_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT success percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat', 'success')

def plot_cat_failure_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT failure percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat', 'failure')

def plot_cat_pending_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat', 'pending')

def plot_regular_success_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot regular success percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'regular', 'success')

def plot_regular_failure_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot regular failure percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'regular', 'failure')

def plot_regular_pending_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot regular pending percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'regular', 'pending')

def plot_sumtypes_success_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot sumTypes success percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'sumtypes', 'success')

def plot_sumtypes_failure_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot sumTypes failure percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'sumtypes', 'failure')

def plot_sumtypes_pending_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot sumTypes pending percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'sumtypes', 'pending')

def plot_cat_pending_resolving_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending resolving percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat_pending_resolving', 'pending')

def plot_cat_pending_postponed_percentage_delta(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot CAT pending postponed percentage deltas over time."""
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat_pending_postponed', 'pending') 

def plot_transaction_percentage_with_moving_average(
    data: Dict[str, Any],
    param_name: str,
    results_dir: str,
    sweep_type: str,
    transaction_type: str,
    percentage_type: str,
    plot_config: Dict[str, Any]
) -> None:
    """
    Plot transaction percentage over time for each simulation with optional moving average.
    
    This function calculates the percentage of a specific transaction state (success, pending, or failure)
    as block number increases.
    
    For success and failure percentages: uses (success + failure) as denominator
    For pending percentage: uses (success + pending + failure) as denominator
    
    Args:
        data: The sweep data containing individual results
        param_name: Name of the parameter being swept
        results_dir: Directory to save the plot
        sweep_type: Type of sweep simulation
        transaction_type: One of 'cat', 'regular', or 'sumtypes'
        percentage_type: One of 'success', 'pending', or 'failure'
        plot_config: Plot configuration including moving average settings
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
            elif transaction_type == 'cat_pending_resolving':
                # For CAT pending resolving transactions
                tx_data = result.get('chain_1_cat_pending_resolving', [])
            elif transaction_type == 'cat_pending_postponed':
                # For CAT pending postponed transactions
                tx_data = result.get('chain_1_cat_pending_postponed', [])
            elif transaction_type == 'regular':
                # For regular transactions, get the specific status data
                if percentage_type == 'success':
                    tx_data = result.get('chain_1_regular_success', [])
                elif percentage_type == 'failure':
                    tx_data = result.get('chain_1_regular_failure', [])
                else:  # pending
                    tx_data = result.get('chain_1_regular_pending', [])
            elif transaction_type == 'sumtypes':
                # For sumTypes (combined CAT + regular), get the specific status data
                if percentage_type == 'success':
                    cat_success = result.get('chain_1_cat_success', [])
                    regular_success = result.get('chain_1_regular_success', [])
                    # Combine CAT and regular success data
                    combined_data = {}
                    for height, count in cat_success:
                        combined_data[height] = combined_data.get(height, 0) + count
                    for height, count in regular_success:
                        combined_data[height] = combined_data.get(height, 0) + count
                    tx_data = sorted(combined_data.items())
                elif percentage_type == 'failure':
                    cat_failure = result.get('chain_1_cat_failure', [])
                    regular_failure = result.get('chain_1_regular_failure', [])
                    # Combine CAT and regular failure data
                    combined_data = {}
                    for height, count in cat_failure:
                        combined_data[height] = combined_data.get(height, 0) + count
                    for height, count in regular_failure:
                        combined_data[height] = combined_data.get(height, 0) + count
                    tx_data = sorted(combined_data.items())
                else:  # pending
                    cat_pending = result.get('chain_1_cat_pending', [])
                    regular_pending = result.get('chain_1_regular_pending', [])
                    # Combine CAT and regular pending data
                    combined_data = {}
                    for height, count in cat_pending:
                        combined_data[height] = combined_data.get(height, 0) + count
                    for height, count in regular_pending:
                        combined_data[height] = combined_data.get(height, 0) + count
                    tx_data = sorted(combined_data.items())
            else:
                # For other transaction types, use the data directly
                tx_data = result.get(f'chain_1_{transaction_type}', [])
            
            if not tx_data:
                continue
            
            # Calculate percentage over time using the same point-in-time logic as regular function
            heights = []
            percentages = []
            
            # Convert tx_data to list of tuples if it's not already
            if isinstance(tx_data, list) and tx_data and isinstance(tx_data[0], dict):
                # If it's a list of dictionaries, convert to list of tuples
                tx_data = [(entry.get('height', 0), entry.get('count', 0)) for entry in tx_data]
            
            # Process each block - tx_data is now a list of tuples (height, count)
            for height, count in tx_data:
                heights.append(height)
                
                # Calculate percentage using counts at this specific height (same logic as regular function)
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
                    
                elif transaction_type == 'cat_pending_resolving':
                    # For CAT pending resolving transactions, calculate percentage of total CAT pending
                    cat_pending_resolving_data = result.get('chain_1_cat_pending_resolving', [])
                    cat_pending_postponed_data = result.get('chain_1_cat_pending_postponed', [])
                    
                    # Convert to height->count mapping
                    cat_pending_resolving_by_height = {entry[0]: entry[1] for entry in cat_pending_resolving_data}
                    cat_pending_postponed_by_height = {entry[0]: entry[1] for entry in cat_pending_postponed_data}
                    
                    # Get counts at this specific height
                    resolving_at_height = cat_pending_resolving_by_height.get(height, 0)
                    postponed_at_height = cat_pending_postponed_by_height.get(height, 0)
                    
                    # Calculate percentage of resolving vs total pending (resolving + postponed)
                    total_pending = resolving_at_height + postponed_at_height
                    if total_pending > 0:
                        percentage = (resolving_at_height / total_pending) * 100
                    else:
                        percentage = 0
                    
                elif transaction_type == 'cat_pending_postponed':
                    # For CAT pending postponed transactions, calculate percentage of total CAT pending
                    cat_pending_resolving_data = result.get('chain_1_cat_pending_resolving', [])
                    cat_pending_postponed_data = result.get('chain_1_cat_pending_postponed', [])
                    
                    # Convert to height->count mapping
                    cat_pending_resolving_by_height = {entry[0]: entry[1] for entry in cat_pending_resolving_data}
                    cat_pending_postponed_by_height = {entry[0]: entry[1] for entry in cat_pending_postponed_data}
                    
                    # Get counts at this specific height
                    resolving_at_height = cat_pending_resolving_by_height.get(height, 0)
                    postponed_at_height = cat_pending_postponed_by_height.get(height, 0)
                    
                    # Calculate percentage of postponed vs total pending (resolving + postponed)
                    total_pending = resolving_at_height + postponed_at_height
                    if total_pending > 0:
                        percentage = (postponed_at_height / total_pending) * 100
                    else:
                        percentage = 0
                    
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
                
                # Calculate percentage based on type (same logic as regular function)
                if transaction_type in ['cat_pending_resolving', 'cat_pending_postponed']:
                    # Percentage already calculated above for these special cases
                    pass
                elif percentage_type in ['success', 'failure']:
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
            
            if not heights:
                continue
            
            # Apply moving average if enabled
            if plot_config.get('plot_moving_average', False):
                from plot_utils import apply_moving_average
                percentage_data = list(zip(heights, percentages))
                window_size = plot_config.get('range_moving_average', 10)
                percentage_data = apply_moving_average(percentage_data, window_size)
                heights = [entry[0] for entry in percentage_data]
                percentages = [entry[1] for entry in percentage_data]
            
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
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, percentages, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        # Create title and filename based on transaction type and percentage type
        if transaction_type == 'cat':
            title = f'CAT {percentage_type.title()} Percentage Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_cat_percentage.png'
        elif transaction_type == 'cat_pending_resolving':
            title = f'CAT Pending Resolving Percentage (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_pending_cat_resolving_percentage.png'
        elif transaction_type == 'cat_pending_postponed':
            title = f'CAT Pending Postponed Percentage (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_pending_cat_postponed_percentage.png'
        elif transaction_type == 'regular':
            if percentage_type in ['success', 'failure']:
                title = f'Regular {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            else:
                title = f'Regular {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_regular_percentage.png'
        elif transaction_type == 'sumtypes':
            if percentage_type in ['success', 'failure']:
                title = f'SumTypes {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            else:
                title = f'SumTypes {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_sumtypes_percentage.png'
        else:
            if percentage_type in ['success', 'failure']:
                title = f'{transaction_type.title()} {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            else:
                title = f'{transaction_type.title()} {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_{transaction_type}_percentage.png'
        
        # Add moving average indicator to title if enabled
        if plot_config.get('plot_moving_average', False):
            window_size = plot_config.get('range_moving_average', 10)
            title += f" (Moving Average, Window={window_size})"
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel(f'{percentage_type.title()} Percentage (%)')
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
        
    except Exception as e:
        print(f"Warning: Error creating {transaction_type} {percentage_type} percentage plot: {e}")
        return 

def plot_transaction_percentage_delta_with_moving_average(
    data: Dict[str, Any],
    param_name: str,
    results_dir: str,
    sweep_type: str,
    transaction_type: str,
    percentage_type: str,
    plot_config: Dict[str, Any]
) -> None:
    """
    Plot transaction percentage delta over time for each simulation with optional moving average.
    
    This function calculates the delta of percentage of a specific transaction state (success, pending, or failure)
    as block number increases.
    
    For success and failure percentages: uses (success + failure) as denominator
    For pending percentage: uses (success + pending + failure) as denominator
    
    Args:
        data: The sweep data containing individual results
        param_name: Name of the parameter being swept
        results_dir: Directory to save the plot
        sweep_type: Type of sweep simulation
        transaction_type: One of 'cat', 'regular', or 'sumtypes'
        percentage_type: One of 'success', 'pending', or 'failure'
        plot_config: Plot configuration including moving average settings
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
            elif transaction_type == 'cat_pending_resolving':
                # For CAT pending resolving transactions
                tx_data = result.get('chain_1_cat_pending_resolving', [])
            elif transaction_type == 'cat_pending_postponed':
                # For CAT pending postponed transactions
                tx_data = result.get('chain_1_cat_pending_postponed', [])
            elif transaction_type == 'regular':
                # For regular transactions, get the specific status data
                if percentage_type == 'success':
                    tx_data = result.get('chain_1_regular_success', [])
                elif percentage_type == 'failure':
                    tx_data = result.get('chain_1_regular_failure', [])
                else:  # pending
                    tx_data = result.get('chain_1_regular_pending', [])
            elif transaction_type == 'sumtypes':
                # For sumTypes (combined CAT + regular), get the specific status data
                if percentage_type == 'success':
                    cat_success = result.get('chain_1_cat_success', [])
                    regular_success = result.get('chain_1_regular_success', [])
                    # Combine CAT and regular success data
                    combined_data = {}
                    for height, count in cat_success:
                        combined_data[height] = combined_data.get(height, 0) + count
                    for height, count in regular_success:
                        combined_data[height] = combined_data.get(height, 0) + count
                    tx_data = sorted(combined_data.items())
                elif percentage_type == 'failure':
                    cat_failure = result.get('chain_1_cat_failure', [])
                    regular_failure = result.get('chain_1_regular_failure', [])
                    # Combine CAT and regular failure data
                    combined_data = {}
                    for height, count in cat_failure:
                        combined_data[height] = combined_data.get(height, 0) + count
                    for height, count in regular_failure:
                        combined_data[height] = combined_data.get(height, 0) + count
                    tx_data = sorted(combined_data.items())
                else:  # pending
                    cat_pending = result.get('chain_1_cat_pending', [])
                    regular_pending = result.get('chain_1_regular_pending', [])
                    # Combine CAT and regular pending data
                    combined_data = {}
                    for height, count in cat_pending:
                        combined_data[height] = combined_data.get(height, 0) + count
                    for height, count in regular_pending:
                        combined_data[height] = combined_data.get(height, 0) + count
                    tx_data = sorted(combined_data.items())
            else:
                # For other transaction types, use the data directly
                tx_data = result.get(f'chain_1_{transaction_type}', [])
            
            if not tx_data:
                continue
            
            # Calculate percentage over time
            heights = []
            percentages = []
            
            for height, count in tx_data:
                # Calculate total for denominator based on percentage type
                if percentage_type in ['success', 'failure']:
                    # For success/failure percentages, use (success + failure) as denominator
                    if transaction_type == 'cat':
                        success_data = result.get('chain_1_cat_success', [])
                        failure_data = result.get('chain_1_cat_failure', [])
                    elif transaction_type == 'regular':
                        success_data = result.get('chain_1_regular_success', [])
                        failure_data = result.get('chain_1_regular_failure', [])
                    elif transaction_type == 'sumtypes':
                        # For sumTypes, combine CAT and regular
                        cat_success = result.get('chain_1_cat_success', [])
                        cat_failure = result.get('chain_1_cat_failure', [])
                        regular_success = result.get('chain_1_regular_success', [])
                        regular_failure = result.get('chain_1_regular_failure', [])
                        success_data = cat_success + regular_success
                        failure_data = cat_failure + regular_failure
                    else:
                        success_data = result.get(f'chain_1_{transaction_type}_success', [])
                        failure_data = result.get(f'chain_1_{transaction_type}_failure', [])
                    
                    # Find total at this height
                    total_at_height = 0
                    for h, c in success_data:
                        if h == height:
                            total_at_height += c
                    for h, c in failure_data:
                        if h == height:
                            total_at_height += c
                else:
                    # For pending percentage, use (success + pending + failure) as denominator
                    if transaction_type == 'cat':
                        success_data = result.get('chain_1_cat_success', [])
                        pending_data = result.get('chain_1_cat_pending', [])
                        failure_data = result.get('chain_1_cat_failure', [])
                    elif transaction_type == 'regular':
                        success_data = result.get('chain_1_regular_success', [])
                        pending_data = result.get('chain_1_regular_pending', [])
                        failure_data = result.get('chain_1_regular_failure', [])
                    elif transaction_type == 'sumtypes':
                        # For sumTypes, combine CAT and regular
                        cat_success = result.get('chain_1_cat_success', [])
                        cat_pending = result.get('chain_1_cat_pending', [])
                        cat_failure = result.get('chain_1_cat_failure', [])
                        regular_success = result.get('chain_1_regular_success', [])
                        regular_pending = result.get('chain_1_regular_pending', [])
                        regular_failure = result.get('chain_1_regular_failure', [])
                        success_data = cat_success + regular_success
                        pending_data = cat_pending + regular_pending
                        failure_data = cat_failure + regular_failure
                    else:
                        success_data = result.get(f'chain_1_{transaction_type}_success', [])
                        pending_data = result.get(f'chain_1_{transaction_type}_pending', [])
                        failure_data = result.get(f'chain_1_{transaction_type}_failure', [])
                    
                    # Find total at this height
                    total_at_height = 0
                    for h, c in success_data:
                        if h == height:
                            total_at_height += c
                    for h, c in pending_data:
                        if h == height:
                            total_at_height += c
                    for h, c in failure_data:
                        if h == height:
                            total_at_height += c
                
                if total_at_height > 0:
                    percentage = (count / total_at_height) * 100
                    heights.append(height)
                    percentages.append(percentage)
            
            if not heights:
                continue
            
            # Calculate deltas from percentage data
            from plot_utils import calculate_delta_from_counts
            percentage_data = list(zip(heights, percentages))
            delta_data = calculate_delta_from_counts(percentage_data)
            
            if not delta_data:
                continue
            
            # Extract delta data
            delta_heights = [entry[0] for entry in delta_data]
            delta_percentages = [entry[1] for entry in delta_data]
            
            # Apply moving average if enabled
            if plot_config.get('plot_moving_average', False):
                from plot_utils import apply_moving_average
                window_size = plot_config.get('range_moving_average', 10)
                delta_data = apply_moving_average(delta_data, window_size)
                delta_heights = [entry[0] for entry in delta_data]
                delta_percentages = [entry[1] for entry in delta_data]
            
            # Trim the last 10% of data to avoid edge effects
            if len(delta_heights) > 10:
                trim_index = int(len(delta_heights) * 0.9)
                delta_heights = delta_heights[:trim_index]
                delta_percentages = delta_percentages[:trim_index]
            
            if not delta_heights:
                continue
            
            # Update maximum height
            if delta_heights:
                max_height = max(max_height, max(delta_heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(delta_heights, delta_percentages, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        # Create title and filename based on transaction type and percentage type
        if transaction_type == 'cat':
            title = f'CAT {percentage_type.title()} Percentage Deltas Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_cat_percentage.png'
        elif transaction_type == 'cat_pending_resolving':
            title = f'CAT Pending Resolving Percentage Deltas (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_pending_cat_resolving_percentage.png'
        elif transaction_type == 'cat_pending_postponed':
            title = f'CAT Pending Postponed Percentage Deltas (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_pending_cat_postponed_percentage.png'
        elif transaction_type == 'regular':
            if percentage_type in ['success', 'failure']:
                title = f'Regular {percentage_type.title()} Percentage Deltas (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            else:
                title = f'Regular {percentage_type.title()} Percentage Deltas (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_regular_percentage.png'
        elif transaction_type == 'sumtypes':
            if percentage_type in ['success', 'failure']:
                title = f'SumTypes {percentage_type.title()} Percentage Deltas (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            else:
                title = f'SumTypes {percentage_type.title()} Percentage Deltas (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_sumtypes_percentage.png'
        else:
            if percentage_type in ['success', 'failure']:
                title = f'{transaction_type.title()} {percentage_type.title()} Percentage Deltas (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            else:
                title = f'{transaction_type.title()} {percentage_type.title()} Percentage Deltas (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{percentage_type}_{transaction_type}_percentage.png'
        
        # Add moving average indicator to title if enabled
        if plot_config.get('plot_moving_average', False):
            window_size = plot_config.get('range_moving_average', 10)
            title += f" (Moving Average, Window={window_size})"
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel(f'{percentage_type.title()} Percentage Delta (%)')
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
        
    except Exception as e:
        print(f"Warning: Error creating {transaction_type} {percentage_type} percentage delta plot: {e}")
        return 