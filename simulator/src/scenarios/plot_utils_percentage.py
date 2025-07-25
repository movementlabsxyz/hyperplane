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
    """Create a color gradient from red (0) to blue (max)"""
    return plt.cm.RdYlBu_r(np.linspace(0, 1, num_simulations))

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

def plot_cat_success_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot CAT success percentage over time for each simulation.
    
    This function calculates the percentage of successful CATs compared to total CATs
    (success + pending + failure) as block number increases.
    """
    try:
        individual_results = data.get('individual_results', [])
        if not individual_results:
            print("Warning: No individual results found for CAT success percentage plot")
            return
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        plt.figure(figsize=(12, 6))
        
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get CAT transaction data for all states
            cat_success_data = result.get('chain_1_cat_success', [])
            cat_pending_data = result.get('chain_1_cat_pending', [])
            cat_failure_data = result.get('chain_1_cat_failure', [])
            
            if not cat_success_data or not cat_pending_data or not cat_failure_data:
                continue
            
            # Create dictionaries to map height to count for each state
            success_by_height = {entry[0]: entry[1] for entry in cat_success_data}
            pending_by_height = {entry[0]: entry[1] for entry in cat_pending_data}
            failure_by_height = {entry[0]: entry[1] for entry in cat_failure_data}
            
            # Get all unique heights
            all_heights = set(success_by_height.keys()) | set(pending_by_height.keys()) | set(failure_by_height.keys())
            heights = sorted(all_heights)
            
            # Calculate cumulative totals and percentages
            success_percentages = []
            
            for height in heights:
                # Calculate cumulative totals up to this height (inclusive)
                cumulative_success = sum(success_by_height.get(h, 0) for h in range(min(heights), height + 1))
                cumulative_pending = sum(pending_by_height.get(h, 0) for h in range(min(heights), height + 1))
                cumulative_failure = sum(failure_by_height.get(h, 0) for h in range(min(heights), height + 1))
                
                # Calculate total CATs so far
                total_cats = cumulative_success + cumulative_pending + cumulative_failure
                
                # Calculate success percentage (avoid division by zero)
                if total_cats > 0:
                    success_percentage = (cumulative_success / total_cats) * 100
                else:
                    success_percentage = 0
                
                success_percentages.append(success_percentage)
            
            # Trim the last 10% of data to avoid edge effects
            if len(heights) > 10:
                trim_index = int(len(heights) * 0.9)
                heights = heights[:trim_index]
                success_percentages = success_percentages[:trim_index]
            
            if not heights:
                continue
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, success_percentages, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        plt.title(f'CAT Success Percentage Over Time - {create_sweep_title(param_name, sweep_type)}')
        plt.xlabel('Block Height')
        plt.ylabel('CAT Success Percentage (%)')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/tx_success_cat_percentage.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error creating CAT success percentage plot: {e}")
        return 

def plot_cat_failure_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot CAT failure percentage over time for each simulation.
    
    This function calculates the percentage of failed CATs compared to total CATs
    (success + pending + failure) as block number increases.
    """
    try:
        individual_results = data.get('individual_results', [])
        if not individual_results:
            print("Warning: No individual results found for CAT failure percentage plot")
            return
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        plt.figure(figsize=(12, 6))
        
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get CAT transaction data for all states
            cat_success_data = result.get('chain_1_cat_success', [])
            cat_pending_data = result.get('chain_1_cat_pending', [])
            cat_failure_data = result.get('chain_1_cat_failure', [])
            
            if not cat_success_data or not cat_pending_data or not cat_failure_data:
                continue
            
            # Create dictionaries to map height to count for each state
            success_by_height = {entry[0]: entry[1] for entry in cat_success_data}
            pending_by_height = {entry[0]: entry[1] for entry in cat_pending_data}
            failure_by_height = {entry[0]: entry[1] for entry in cat_failure_data}
            
            # Get all unique heights
            all_heights = set(success_by_height.keys()) | set(pending_by_height.keys()) | set(failure_by_height.keys())
            heights = sorted(all_heights)
            
            # Calculate cumulative totals and percentages
            failure_percentages = []
            
            for height in heights:
                # Calculate cumulative totals up to this height (inclusive)
                cumulative_success = sum(success_by_height.get(h, 0) for h in range(min(heights), height + 1))
                cumulative_pending = sum(pending_by_height.get(h, 0) for h in range(min(heights), height + 1))
                cumulative_failure = sum(failure_by_height.get(h, 0) for h in range(min(heights), height + 1))
                
                # Calculate total CATs so far
                total_cats = cumulative_success + cumulative_pending + cumulative_failure
                
                # Calculate failure percentage (avoid division by zero)
                if total_cats > 0:
                    failure_percentage = (cumulative_failure / total_cats) * 100
                else:
                    failure_percentage = 0
                
                failure_percentages.append(failure_percentage)
            
            # Trim the last 10% of data to avoid edge effects
            if len(heights) > 10:
                trim_index = int(len(heights) * 0.9)
                heights = heights[:trim_index]
                failure_percentages = failure_percentages[:trim_index]
            
            if not heights:
                continue
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, failure_percentages, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        plt.title(f'CAT Failure Percentage Over Time - {create_sweep_title(param_name, sweep_type)}')
        plt.xlabel('Block Height')
        plt.ylabel('CAT Failure Percentage (%)')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/tx_failure_cat_percentage.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error creating CAT failure percentage plot: {e}")
        return

def plot_cat_pending_percentage(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot CAT pending percentage over time for each simulation.
    
    This function calculates the percentage of pending CATs compared to total CATs
    (success + pending + failure) as block number increases.
    """
    try:
        individual_results = data.get('individual_results', [])
        if not individual_results:
            print("Warning: No individual results found for CAT pending percentage plot")
            return
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        plt.figure(figsize=(12, 6))
        
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get CAT transaction data for all states
            cat_success_data = result.get('chain_1_cat_success', [])
            cat_pending_data = result.get('chain_1_cat_pending', [])
            cat_failure_data = result.get('chain_1_cat_failure', [])
            
            if not cat_success_data or not cat_pending_data or not cat_failure_data:
                continue
            
            # Create dictionaries to map height to count for each state
            success_by_height = {entry[0]: entry[1] for entry in cat_success_data}
            pending_by_height = {entry[0]: entry[1] for entry in cat_pending_data}
            failure_by_height = {entry[0]: entry[1] for entry in cat_failure_data}
            
            # Get all unique heights
            all_heights = set(success_by_height.keys()) | set(pending_by_height.keys()) | set(failure_by_height.keys())
            heights = sorted(all_heights)
            
            # Calculate cumulative totals and percentages
            pending_percentages = []
            
            for height in heights:
                # Calculate cumulative totals up to this height (inclusive)
                cumulative_success = sum(success_by_height.get(h, 0) for h in range(min(heights), height + 1))
                cumulative_pending = sum(pending_by_height.get(h, 0) for h in range(min(heights), height + 1))
                cumulative_failure = sum(failure_by_height.get(h, 0) for h in range(min(heights), height + 1))
                
                # Calculate total CATs so far
                total_cats = cumulative_success + cumulative_pending + cumulative_failure
                
                # Calculate pending percentage (avoid division by zero)
                if total_cats > 0:
                    pending_percentage = (cumulative_pending / total_cats) * 100
                else:
                    pending_percentage = 0
                
                pending_percentages.append(pending_percentage)
            
            # Trim the last 10% of data to avoid edge effects
            if len(heights) > 10:
                trim_index = int(len(heights) * 0.9)
                heights = heights[:trim_index]
                pending_percentages = pending_percentages[:trim_index]
            
            if not heights:
                continue
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, pending_percentages, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        plt.title(f'CAT Pending Percentage Over Time - {create_sweep_title(param_name, sweep_type)}')
        plt.xlabel('Block Height')
        plt.ylabel('CAT Pending Percentage (%)')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/tx_pending_cat_percentage.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error creating CAT pending percentage plot: {e}")
        return 