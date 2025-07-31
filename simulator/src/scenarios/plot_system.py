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

# Import moving average function from plot_utils_moving_average
from plot_utils_moving_average import apply_moving_average

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
            
            # Load system memory usage data for this simulation
            # Extract just the directory name from the full path
            results_dir_name = results_dir.replace('simulator/results/', '')
            sim_data_dir = f'simulator/results/{results_dir_name}/data/sim_{i}'
            
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
                            print(f"Warning: Heights ({len(heights)}) and memory values ({len(memory_values)}) have different lengths for simulation {i}")
                            # Use the shorter length
                            min_length = min(len(heights), len(memory_values))
                            heights = heights[:min_length]
                            memory_values = memory_values[:min_length]
                        
                        # Update maximum height
                        if heights:
                            max_height = max(max_height, max(heights))
                        
                        # Plot with color based on parameter
                        label = create_parameter_label(param_name, param_value)
                        plt.plot(heights, memory_values, color=colors[i], alpha=0.7, 
                                label=label, linewidth=1.5)
                    else:
                        print(f"Warning: No system memory entries found for simulation {i}")
                else:
                    print(f"Warning: No system_memory key found in {memory_file}")
            else:
                print(f"Warning: system_memory.json file not found for simulation {i}")
        
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
        
    except Exception as e:
        print(f"Error plotting system memory data: {e}")
        import traceback
        traceback.print_exc()


def plot_system_memory_total(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot total system memory usage over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping total system memory plot")
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
        plt.savefig(f'{results_dir}/figs/system_memory_total.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting system total memory data: {e}")
        import traceback
        traceback.print_exc()


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
    """Plot filtered system CPU usage over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping filtered system CPU plot")
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


def plot_system_cpu_total(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot total system CPU usage over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping system total CPU plot")
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
        plt.savefig(f'{results_dir}/figs/system_cpu_total.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting system total CPU data: {e}")
        import traceback
        traceback.print_exc()


# ------------------------------------------------------------------------------------------------
# Loop Steps Plotting
# ------------------------------------------------------------------------------------------------

def plot_loop_steps_without_tx_issuance(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """Plot loop steps without transaction issuance over time for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping loop steps plot")
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


def plot_loop_steps_without_tx_issuance_moving_average(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, plot_config: Dict[str, Any]) -> None:
    """Plot loop steps without transaction issuance over time with moving average for sweep simulations"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping loop steps moving average plot")
            return
        
        # Set moving average window size directly for system plots
        window_size = 100
        
        # Create figure
        fig, ax = plt.subplots(figsize=(12, 8))
        
        # Get parameter values for coloring
        param_values = [result[param_name] for result in individual_results]
        colors = create_color_gradient(len(param_values))
        
        # Plot each simulation's loop steps data with moving average
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
                        
                        # Apply moving average
                        if len(heights) >= window_size:
                            # Convert to list of tuples for moving average function
                            data_points = list(zip(heights, loop_steps_values))
                            smoothed_data = apply_moving_average(data_points, window_size)
                            
                            # Extract smoothed heights and values
                            smoothed_heights = [point[0] for point in smoothed_data]
                            smoothed_values = [point[1] for point in smoothed_data]
                            
                            # Plot the smoothed data
                            ax.plot(smoothed_heights, smoothed_values, color=color, alpha=0.7, linewidth=2)
                        else:
                            print(f"Warning: Not enough data points for moving average (window={window_size}) for simulation {sim_index}")
                            # Plot original data if not enough points
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
        ax.set_ylabel('Loop Steps Count (Moving Average)')
        ax.set_title(f'Loop Steps Without Transaction Issuance Over Time (Moving Average, Window={window_size}) by {PARAM_DISPLAY_NAMES.get(param_name, param_name.replace("_", " ").title())}')
        ax.legend(loc="upper right")
        ax.grid(True, alpha=0.3)
        
        # Save the plot
        plt.savefig(f'{results_dir}/figs/loop_steps_without_tx_issuance_moving_average.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error plotting loop steps moving average data: {e}")
        import traceback
        traceback.print_exc()


 