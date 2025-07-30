#!/usr/bin/env python3
"""
System-related plotting utilities for Hyperplane simulator sweep results.

This module provides system monitoring plotting functions that can be used by all
sweep simulation plotting scripts to eliminate code duplication.
"""

import os
import sys
import json
import matplotlib.pyplot as plt
import numpy as np
from typing import Dict, List, Tuple, Any, Optional

# Import utility functions from plot_utils
from plot_utils import (
    create_color_gradient,
    extract_parameter_value,
    create_parameter_label,
    create_sweep_title,
    trim_time_series_data,
    PARAM_DISPLAY_NAMES
)

# ------------------------------------------------------------------------------------------------
# System Memory Plotting
# ------------------------------------------------------------------------------------------------

def plot_system_memory(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot system memory usage over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping system memory plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's memory usage
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get memory data
            memory_data = result.get('system_memory', [])
            
            if not memory_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            memory_data = trim_time_series_data(memory_data, 0.1)
            
            if not memory_data:
                continue
                
            # Extract data - memory_data is a list of tuples (height, memory_mb)
            heights = [entry[0] for entry in memory_data]
            memory_mb = [entry[1] for entry in memory_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, memory_mb, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        plt.title(f'System Memory Usage Over Time - {create_sweep_title(param_name, sweep_type)}')
        plt.xlabel('Block Height')
        plt.ylabel('Memory Usage (MB)')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/system_memory.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing system memory data: {e}")
        return


def plot_system_total_memory(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot total system memory usage over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping total system memory plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's total memory usage
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get total memory data
            total_memory_data = result.get('system_total_memory', [])
            
            if not total_memory_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            total_memory_data = trim_time_series_data(total_memory_data, 0.1)
            
            if not total_memory_data:
                continue
                
            # Extract data - total_memory_data is a list of tuples (height, memory_mb)
            heights = [entry[0] for entry in total_memory_data]
            memory_mb = [entry[1] for entry in total_memory_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, memory_mb, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        plt.title(f'Total System Memory Usage Over Time - {create_sweep_title(param_name, sweep_type)}')
        plt.xlabel('Block Height')
        plt.ylabel('Total Memory Usage (MB)')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/system_total_memory.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing total system memory data: {e}")
        return


# ------------------------------------------------------------------------------------------------
# System CPU Plotting
# ------------------------------------------------------------------------------------------------

def plot_system_cpu(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot system CPU usage over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping system CPU plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's CPU usage
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get CPU data
            cpu_data = result.get('system_cpu', [])
            
            if not cpu_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            cpu_data = trim_time_series_data(cpu_data, 0.1)
            
            if not cpu_data:
                continue
                
            # Extract data - cpu_data is a list of tuples (height, cpu_percent)
            heights = [entry[0] for entry in cpu_data]
            cpu_percent = [entry[1] for entry in cpu_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, cpu_percent, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        plt.title(f'System CPU Usage Over Time - {create_sweep_title(param_name, sweep_type)}')
        plt.xlabel('Block Height')
        plt.ylabel('CPU Usage (%)')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/system_cpu.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing system CPU data: {e}")
        return


def plot_system_cpu_filtered(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot filtered system CPU usage over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping filtered system CPU plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's filtered CPU usage
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get filtered CPU data
            cpu_filtered_data = result.get('system_cpu_filtered', [])
            
            if not cpu_filtered_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            cpu_filtered_data = trim_time_series_data(cpu_filtered_data, 0.1)
            
            if not cpu_filtered_data:
                continue
                
            # Extract data - cpu_filtered_data is a list of tuples (height, cpu_percent)
            heights = [entry[0] for entry in cpu_filtered_data]
            cpu_percent = [entry[1] for entry in cpu_filtered_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, cpu_percent, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        plt.title(f'Filtered System CPU Usage Over Time - {create_sweep_title(param_name, sweep_type)}')
        plt.xlabel('Block Height')
        plt.ylabel('Filtered CPU Usage (%)')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/system_cpu_filtered.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing filtered system CPU data: {e}")
        return


def plot_system_total_cpu(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot total system CPU usage over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping total system CPU plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's total CPU usage
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get total CPU data
            total_cpu_data = result.get('system_total_cpu', [])
            
            if not total_cpu_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            total_cpu_data = trim_time_series_data(total_cpu_data, 0.1)
            
            if not total_cpu_data:
                continue
                
            # Extract data - total_cpu_data is a list of tuples (height, cpu_percent)
            heights = [entry[0] for entry in total_cpu_data]
            cpu_percent = [entry[1] for entry in total_cpu_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, cpu_percent, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        plt.title(f'Total System CPU Usage Over Time - {create_sweep_title(param_name, sweep_type)}')
        plt.xlabel('Block Height')
        plt.ylabel('Total CPU Usage (%)')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/system_total_cpu.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing total system CPU data: {e}")
        return


# ------------------------------------------------------------------------------------------------
# Loop Steps Plotting
# ------------------------------------------------------------------------------------------------

def plot_loop_steps_without_tx_issuance(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot loop steps without transaction issuance over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping loop steps without tx issuance plot")
            return
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each simulation's loop steps without tx issuance
        for i, result in enumerate(individual_results):
            param_value = extract_parameter_value(result, param_name)
            
            # Get loop steps data
            loop_steps_data = result.get('loop_steps_without_tx_issuance', [])
            
            if not loop_steps_data:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            loop_steps_data = trim_time_series_data(loop_steps_data, 0.1)
            
            if not loop_steps_data:
                continue
                
            # Extract data - loop_steps_data is a list of tuples (height, steps)
            heights = [entry[0] for entry in loop_steps_data]
            steps = [entry[1] for entry in loop_steps_data]
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, steps, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        plt.title(f'Loop Steps Without Transaction Issuance Over Time - {create_sweep_title(param_name, sweep_type)}')
        plt.xlabel('Block Height')
        plt.ylabel('Loop Steps Without TX Issuance')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Save plot
        plt.savefig(f'{results_dir}/figs/loop_steps_without_tx_issuance.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing loop steps without tx issuance data: {e}")
        return 