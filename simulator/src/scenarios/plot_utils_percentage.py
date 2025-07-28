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

def create_color_gradient(num_simulations: int) -> np.ndarray:
    """Create a color gradient from blue to red with better visibility"""
    return plt.cm.viridis(np.linspace(0, 1, num_simulations))

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
    param_display = param_name.replace('_', ' ').title()
    # Remove units in parentheses for cleaner titles
    param_display = param_display.split(' (')[0]
    return f'{param_display} Sweep'

def plot_transaction_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, transaction_type: str, percentage_type: str) -> None:
    """
    Plot transaction percentage over time for each simulation.
    
    This function calculates the percentage of a specific transaction state (success, pending, or failure)
    compared to total transactions as block number increases.
    
    Args:
        data: The sweep data containing individual results
        param_name: Name of the parameter being swept
        results_dir: Directory to save the plot
        sweep_type: Type of sweep simulation
        transaction_type: One of 'cat', 'regular', or 'sumtypes'
        percentage_type: One of 'success', 'pending', or 'failure'
    """
    try:
        individual_results = data.get('individual_results', [])
        if not individual_results:
            print(f"Warning: No individual results found for {transaction_type} {percentage_type} percentage plot")
            return
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        plt.figure(figsize=(12, 6))
        
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            if transaction_type == 'sumtypes':
                # For sumTypes, we need both CAT and regular data
                cat_success_data = result.get('chain_1_cat_success', [])
                cat_pending_data = result.get('chain_1_cat_pending', [])
                cat_failure_data = result.get('chain_1_cat_failure', [])
                regular_success_data = result.get('chain_1_regular_success', [])
                regular_pending_data = result.get('chain_1_regular_pending', [])
                regular_failure_data = result.get('chain_1_regular_failure', [])
                
                if not cat_success_data or not cat_pending_data or not cat_failure_data or not regular_success_data or not regular_pending_data or not regular_failure_data:
                    continue
                
                # Create dictionaries to map height to count for each state
                cat_success_by_height = {entry[0]: entry[1] for entry in cat_success_data}
                cat_pending_by_height = {entry[0]: entry[1] for entry in cat_pending_data}
                cat_failure_by_height = {entry[0]: entry[1] for entry in cat_failure_data}
                regular_success_by_height = {entry[0]: entry[1] for entry in regular_success_data}
                regular_pending_by_height = {entry[0]: entry[1] for entry in regular_pending_data}
                regular_failure_by_height = {entry[0]: entry[1] for entry in regular_failure_data}
                
                # Get all unique heights
                all_heights = set(cat_success_by_height.keys()) | set(cat_pending_by_height.keys()) | set(cat_failure_by_height.keys()) | \
                             set(regular_success_by_height.keys()) | set(regular_pending_by_height.keys()) | set(regular_failure_by_height.keys())
                heights = sorted(all_heights)
                
                # Calculate cumulative totals and percentages
                percentages = []
                
                for height in heights:
                    # Calculate cumulative totals up to this height (inclusive) for CAT transactions
                    cumulative_cat_success = sum(cat_success_by_height.get(h, 0) for h in range(min(heights), height + 1))
                    cumulative_cat_pending = sum(cat_pending_by_height.get(h, 0) for h in range(min(heights), height + 1))
                    cumulative_cat_failure = sum(cat_failure_by_height.get(h, 0) for h in range(min(heights), height + 1))
                    
                    # Calculate cumulative totals up to this height (inclusive) for regular transactions
                    cumulative_regular_success = sum(regular_success_by_height.get(h, 0) for h in range(min(heights), height + 1))
                    cumulative_regular_pending = sum(regular_pending_by_height.get(h, 0) for h in range(min(heights), height + 1))
                    cumulative_regular_failure = sum(regular_failure_by_height.get(h, 0) for h in range(min(heights), height + 1))
                    
                    # Calculate total transactions so far (CAT + regular)
                    total_success = cumulative_cat_success + cumulative_regular_success
                    total_pending = cumulative_cat_pending + cumulative_regular_pending
                    total_failure = cumulative_cat_failure + cumulative_regular_failure
                    total_transactions = total_success + total_pending + total_failure
                    
                    # Calculate percentage based on type (avoid division by zero)
                    if total_transactions > 0:
                        if percentage_type == 'success':
                            percentage = (total_success / total_transactions) * 100
                        elif percentage_type == 'pending':
                            percentage = (total_pending / total_transactions) * 100
                        elif percentage_type == 'failure':
                            percentage = (total_failure / total_transactions) * 100
                        else:
                            percentage = 0
                    else:
                        percentage = 0
                    
                    percentages.append(percentage)
                
                # Create title and filename
                title = f'All Transactions {percentage_type.title()} Percentage Over Time - {create_sweep_title(param_name, sweep_type)}'
                filename = f'tx_{percentage_type}_sumtypes_percentage.png'
                
            else:
                # For CAT or regular transactions
                success_data = result.get(f'chain_1_{transaction_type}_success', [])
                pending_data = result.get(f'chain_1_{transaction_type}_pending', [])
                failure_data = result.get(f'chain_1_{transaction_type}_failure', [])
                
                if not success_data or not pending_data or not failure_data:
                    continue
                
                # Create dictionaries to map height to count for each state
                success_by_height = {entry[0]: entry[1] for entry in success_data}
                pending_by_height = {entry[0]: entry[1] for entry in pending_data}
                failure_by_height = {entry[0]: entry[1] for entry in failure_data}
                
                # Get all unique heights
                all_heights = set(success_by_height.keys()) | set(pending_by_height.keys()) | set(failure_by_height.keys())
                heights = sorted(all_heights)
                
                # Calculate cumulative totals and percentages
                percentages = []
                
                for height in heights:
                    # Calculate cumulative totals up to this height (inclusive)
                    cumulative_success = sum(success_by_height.get(h, 0) for h in range(min(heights), height + 1))
                    cumulative_pending = sum(pending_by_height.get(h, 0) for h in range(min(heights), height + 1))
                    cumulative_failure = sum(failure_by_height.get(h, 0) for h in range(min(heights), height + 1))
                    
                    # Calculate total transactions so far
                    total_transactions = cumulative_success + cumulative_pending + cumulative_failure
                    
                    # Calculate percentage based on type (avoid division by zero)
                    if total_transactions > 0:
                        if percentage_type == 'success':
                            percentage = (cumulative_success / total_transactions) * 100
                        elif percentage_type == 'pending':
                            percentage = (cumulative_pending / total_transactions) * 100
                        elif percentage_type == 'failure':
                            percentage = (cumulative_failure / total_transactions) * 100
                        else:
                            percentage = 0
                    else:
                        percentage = 0
                    
                    percentages.append(percentage)
                
                # Create title and filename based on transaction type
                if transaction_type == 'cat':
                    title = f'CAT {percentage_type.title()} Percentage Over Time - {create_sweep_title(param_name, sweep_type)}'
                    filename = f'tx_{percentage_type}_cat_percentage.png'
                elif transaction_type == 'regular':
                    title = f'Regular {percentage_type.title()} Percentage Over Time - {create_sweep_title(param_name, sweep_type)}'
                    filename = f'tx_{percentage_type}_regular_percentage.png'
                else:
                    title = f'{transaction_type.title()} {percentage_type.title()} Percentage Over Time - {create_sweep_title(param_name, sweep_type)}'
                    filename = f'tx_{percentage_type}_{transaction_type}_percentage.png'
            
            # Trim the last 10% of data to avoid edge effects
            if len(heights) > 10:
                trim_index = int(len(heights) * 0.9)
                heights = heights[:trim_index]
                percentages = percentages[:trim_index]
            
            if not heights:
                continue
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, percentages, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        plt.title(title)
        plt.xlabel('Block Height')
        plt.ylabel(f'{percentage_type.title()} Percentage (%)')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/{filename}', 
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