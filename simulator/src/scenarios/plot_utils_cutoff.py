"""
Cutoff-related plotting utilities for sweep simulations.

This module contains functions for applying cutoff logic to transaction data
and generating cutoff-specific plots.
"""

import os
import matplotlib.pyplot as plt
import numpy as np
from typing import Dict, Any, List, Tuple


def apply_cutoff_to_data(data: List[Tuple[int, int]], cutoff_height: int, transaction_type: str) -> List[Tuple[int, int]]:
    """
    Apply cutoff to time-series data and conditionally subtract offset.
    
    Args:
        data: List of (height, count) tuples
        cutoff_height: Height at which to apply cutoff
        transaction_type: Type of transaction (used to determine if offset should be subtracted)
        
    Returns:
        Filtered data with offset subtracted for success/failure transactions
    """
    if not data:
        return []
    
    # Filter data to only include heights >= cutoff_height
    filtered_data = [(height, count) for height, count in data if height >= cutoff_height]
    
    if not filtered_data:
        return []
    
    # For success and failure transactions, subtract the offset value
    # For pending transactions, don't subtract anything
    if any(status in transaction_type for status in ['success', 'failure']):
        # Find the offset value (the value at cutoff_height)
        offset_value = 0
        for height, count in data:
            if height == cutoff_height:
                offset_value = count
                break
        
        # Subtract offset from all remaining data points
        adjusted_data = [(height, count - offset_value) for height, count in filtered_data]
        return adjusted_data
    else:
        # For pending transactions, return data as-is (no offset subtraction)
        return filtered_data


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


def plot_transactions_cutoff_overlay(
    data: Dict[str, Any],
    param_name: str,
    transaction_type: str,
    results_dir: str,
    sweep_type: str,
    plot_config: Dict[str, Any]
) -> None:
    """
    Plot transaction counts with cutoff applied.
    
    Args:
        data: Sweep data containing individual results
        param_name: Name of the parameter being swept
        transaction_type: Type of transaction to plot
        results_dir: Directory to save plots
        sweep_type: Type of sweep simulation
        plot_config: Plot configuration containing cutoff settings
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
            
            # Apply cutoff to the data
            cutoff_percentage = plot_config.get('cutoff', 0.5)
            max_data_height = max(tx_data, key=lambda x: x[0])[0]
            cutoff_height = int(max_data_height * cutoff_percentage)
            
            cutoff_data = apply_cutoff_to_data(tx_data, cutoff_height, transaction_type)
            
            if not cutoff_data:
                continue
            
            # Extract heights and counts
            heights = [entry[0] for entry in cutoff_data]
            counts = [entry[1] for entry in cutoff_data]
            
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
                title = f'CAT {status.title()} Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat.png'
            else:
                title = f'CAT {status.title()} Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{status}_cat.png'
        elif transaction_type.startswith('regular_'):
            # Regular transactions
            status = transaction_type.replace('regular_', '')
            title = f'Regular {status.title()} Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{status}_regular.png'
        else:
            # Fallback
            title = f'{transaction_type.title()} Transaction Counts (Cutoff Applied) - {create_sweep_title(param_name, sweep_type)}'
            filename = f'tx_{transaction_type}.png'
        
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
                    filename = f'tx_{percentage_type}_cat_percentage.png'
                elif transaction_type == 'cat_pending_resolving':
                    title = f'CAT Pending Resolving Percentage (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    filename = f'tx_pending_cat_resolving_percentage.png'
                elif transaction_type == 'cat_pending_postponed':
                    title = f'CAT Pending Postponed Percentage (of Resolving+Postponed) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    filename = f'tx_pending_cat_postponed_percentage.png'
                elif transaction_type == 'regular':
                    if percentage_type in ['success', 'failure']:
                        title = f'Regular {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    else:
                        title = f'Regular {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    filename = f'tx_{percentage_type}_regular_percentage.png'
                else:
                    if percentage_type in ['success', 'failure']:
                        title = f'{transaction_type.title()} {percentage_type.title()} Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
                    else:
                        title = f'{transaction_type.title()} {percentage_type.title()} Percentage (of Success+Pending+Failure) Over Time - {create_sweep_title(param_name, sweep_type)} (Cutoff Applied)'
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