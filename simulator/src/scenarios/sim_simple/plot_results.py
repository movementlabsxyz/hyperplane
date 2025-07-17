#!/usr/bin/env python3

import os
import sys
import json
import matplotlib.pyplot as plt
import subprocess
import numpy as np

# Add the current directory to the Python path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from plot_account_selection import plot_account_selection
from plot_miscellaneous import (
    plot_tx_pending,
    plot_tx_success,
    plot_tx_failure,
    plot_parameters,
    plot_tx_allStatus_cat,
    plot_tx_allStatus_regular,
    plot_tx_allStatus_all,
    plot_comprehensive_comparison,
)

# Global variables for paths
BASE_DATA_PATH = 'simulator/results/sim_simple/data/sim_0/run_average'
FIGS_PATH = 'simulator/results/sim_simple/figs'

def calculate_running_average(data: list, window_size: int = 10) -> list:
    """
    Calculate running average of a list of values.
    
    Args:
        data: List of numeric values
        window_size: Size of the averaging window
    
    Returns:
        List of running averages
    """
    if len(data) < window_size:
        return data
    
    result = []
    for i in range(len(data)):
        start = max(0, i - window_size + 1)
        end = i + 1
        window = data[start:end]
        result.append(sum(window) / len(window))
    
    return result

def create_run_label(run_idx: int, total_runs: int) -> str:
    """
    Create a label for a run, showing first 5, then "...", then last 5 if more than 10 runs.
    
    Args:
        run_idx: Index of the current run (0-based)
        total_runs: Total number of runs
    
    Returns:
        Label string or None if run should not appear in legend
    """
    if total_runs > 10:
        if run_idx < 5:
            return f'Run {run_idx + 1}'
        elif run_idx == 5:
            return "..."
        elif run_idx >= total_runs - 5:
            return f'Run {run_idx + 1}'
        else:
            return None  # Don't show in legend but still plot
    else:
        return f'Run {run_idx + 1}'

# ------------------------------------------------------------------------------------------------
# Per-Run Plotting Functions (for sim_0 directory)
# ------------------------------------------------------------------------------------------------

def create_per_run_plots():
    """
    Create per-run plots in the sim_0 directory, similar to sweep simulations.
    """
    sim_figs_dir = f'{FIGS_PATH}/sim_0'
    os.makedirs(sim_figs_dir, exist_ok=True)
    
    # Load individual run data for this simulation
    sim_data_dir = f'simulator/results/sim_simple/data/sim_0'
    
    # Check if the simulation directory exists
    if not os.path.exists(sim_data_dir):
        print(f"Warning: {sim_data_dir} not found")
        return
    
    # Get all run directories (exclude run_average)
    run_dirs = [d for d in os.listdir(sim_data_dir) 
               if d.startswith('run_') and d != 'run_average' and os.path.isdir(os.path.join(sim_data_dir, d))]
    # Sort numerically by run number
    run_dirs.sort(key=lambda x: int(x.split('_')[1]) if '_' in x else 0)
    
    if not run_dirs:
        print(f"Warning: No run directories found in {sim_data_dir}")
        return
    
    # Load block interval from simulation stats to calculate TPS
    try:
        with open(f'{BASE_DATA_PATH}/simulation_stats.json', 'r') as f:
            stats_data = json.load(f)
        block_interval = stats_data['parameters']['block_interval']  # in seconds
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Could not load block interval: {e}")
        return
    
    # Create figure with subplots for TPS
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 10), sharex=True)
    
    # Create color gradient for runs
    colors = plt.cm.viridis(np.linspace(0, 1, len(run_dirs)))
    
    # Plot each run's data for TPS
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        # Create label - show first 5, then "...", then last 5 if more than 10 runs
        label = create_run_label(run_idx, len(run_dirs))
        try:
            # Load transactions per block data for this run
            tx_per_block_file = os.path.join(sim_data_dir, run_dir, 'data', 'tx_per_block_chain_1.json')
            if not os.path.exists(tx_per_block_file):
                print(f"Warning: {tx_per_block_file} not found")
                continue
            
            with open(tx_per_block_file, 'r') as f:
                run_data = json.load(f)
            
            # Extract data
            blocks = [entry['height'] for entry in run_data['chain_1_tx_per_block']]
            tx_per_block = [entry['count'] for entry in run_data['chain_1_tx_per_block']]
            
            # Calculate TPS
            tps = [tx_count / block_interval for tx_count in tx_per_block]
            
            # Apply 20-block running average to both transactions per block and TPS
            tx_per_block_smoothed = calculate_running_average(tx_per_block, 20)
            tps_smoothed = calculate_running_average(tps, 20)
            
            # Plot with color based on run
            ax1.plot(blocks, tx_per_block_smoothed, color=colors[run_idx], alpha=0.7, 
                    label=label, linewidth=1.5)
            ax2.plot(blocks, tps_smoothed, color=colors[run_idx], alpha=0.7, 
                    label=label, linewidth=1.5)
            plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing run {run_dir} for TPS plot: {e}")
            continue
    
    # Create titles for TPS plot
    title = f'Transactions per Block (Chain 1) - Simple Simulation (20-block running average)'
    ax1.set_title(title)
    ax1.set_ylabel('Number of Transactions')
    ax1.grid(True, alpha=0.3)
    ax1.legend()
    
    ax2.set_title(f'Transactions per Second (Chain 1) - Simple Simulation (Block Interval: {block_interval}s, 20-block running average)')
    ax2.set_xlabel('Block Height')
    ax2.set_ylabel('TPS')
    ax2.grid(True, alpha=0.3)
    ax2.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/tps.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # Create system memory usage plot
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's system memory usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        # Create label - show first 5, then "...", then last 5 if more than 10 runs
        label = create_run_label(run_idx, len(run_dirs))
        try:
            # Load system memory usage data for this run
            memory_file = os.path.join(sim_data_dir, run_dir, 'data', 'system_memory.json')
            if not os.path.exists(memory_file):
                print(f"Warning: {memory_file} not found")
                continue
            
            with open(memory_file, 'r') as f:
                run_data = json.load(f)
            
            # Extract system memory usage data
            if 'system_memory' in run_data:
                memory_entries = run_data['system_memory']
                if memory_entries:
                    # Extract block heights and memory usage values
                    heights = [entry['height'] for entry in memory_entries]
                    memory_values = [entry['bytes'] / (1024 * 1024) for entry in memory_entries]  # Convert to MB
                    
                    # Plot with color based on run
                    ax.plot(heights, memory_values, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing system memory for run {run_dir}: {e}")
            continue
    
    # Create title for system memory plot
    ax.set_title('System Memory Usage Over Time - Simple Simulation')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System Memory Usage (MB)')
    ax.grid(True, alpha=0.3)
    ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_memory.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # Create system total memory usage plot
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's system total memory usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        # Create label - show first 5, then "...", then last 5 if more than 10 runs
        label = create_run_label(run_idx, len(run_dirs))
        try:
            # Load system total memory usage data for this run
            memory_file = os.path.join(sim_data_dir, run_dir, 'data', 'system_total_memory.json')
            if not os.path.exists(memory_file):
                print(f"Warning: {memory_file} not found")
                continue
            
            with open(memory_file, 'r') as f:
                run_data = json.load(f)
            
            # Extract system total memory usage data
            if 'system_total_memory' in run_data:
                memory_entries = run_data['system_total_memory']
                if memory_entries:
                    # Extract block heights and memory usage values
                    heights = [entry['height'] for entry in memory_entries]
                    memory_values = [entry['bytes'] / (1024 * 1024 * 1024) for entry in memory_entries]  # Convert to GB
                    
                    # Plot with color based on run
                    ax.plot(heights, memory_values, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing system total memory for run {run_dir}: {e}")
            continue
    
    # Create title for system total memory plot
    ax.set_title('System Total Memory Usage Over Time - Simple Simulation')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System Total Memory Usage (GB)')
    ax.grid(True, alpha=0.3)
    ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_total_memory.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # Create transaction plots for this simulation
    # Define transaction types to plot with their file names and data keys
    transaction_types = [
        # CAT transaction plots
        ('cat_pending_transactions_chain_1', 'pending_cat__chain1'),
        ('cat_success_transactions_chain_1', 'success_cat__chain1'),
        ('cat_failure_transactions_chain_1', 'failure_cat__chain1'),
        ('cat_pending_transactions_chain_2', 'pending_cat__chain2'),
        ('cat_success_transactions_chain_2', 'success_cat__chain2'),
        ('cat_failure_transactions_chain_2', 'failure_cat__chain2'),
        # Regular transaction plots
        ('regular_pending_transactions_chain_1', 'pending_regular__chain1'),
        ('regular_success_transactions_chain_1', 'success_regular__chain1'),
        ('regular_failure_transactions_chain_1', 'failure_regular__chain1'),
        ('regular_pending_transactions_chain_2', 'pending_regular__chain2'),
        ('regular_success_transactions_chain_2', 'success_regular__chain2'),
        ('regular_failure_transactions_chain_2', 'failure_regular__chain2')
    ]
    
    # Create plots for each transaction type
    for file_name, tx_type in transaction_types:
        fig, ax = plt.subplots(figsize=(12, 8))
        
        # Plot each run's transaction data
        plotted_runs = 0
        for run_idx, run_dir in enumerate(run_dirs):
            # Create label - show first 5, then "...", then last 5 if more than 10 runs
            label = create_run_label(run_idx, len(run_dirs))
            try:
                # Load transaction data for this run
                tx_file = os.path.join(sim_data_dir, run_dir, 'data', f'{file_name}.json')
                if not os.path.exists(tx_file):
                    continue
                
                with open(tx_file, 'r') as f:
                    run_data = json.load(f)
                
                # Extract transaction data - the data is stored as a list of objects with height and count fields
                # Handle different naming patterns
                if '__' in tx_type:
                    # For plots with chain specification (e.g., 'failure_cat__chain1')
                    parts = tx_type.split('__')
                    chain_num = parts[1]
                    base_parts = parts[0].split('_')
                    
                    if len(base_parts) == 2 and base_parts[1] == 'cat':
                        # CAT transaction plots (e.g., 'failure_cat__chain1' -> 'chain_1_cat_failure')
                        base_type = base_parts[0]  # 'failure'
                        data_key = f'chain_1_cat_{base_type}' if chain_num == 'chain1' else f'chain_2_cat_{base_type}'
                    elif len(base_parts) == 2 and base_parts[1] == 'regular':
                        # Regular transaction plots (e.g., 'failure_regular__chain1' -> 'chain_1_regular_failure')
                        base_type = base_parts[0]  # 'failure'
                        data_key = f'chain_1_regular_{base_type}' if chain_num == 'chain1' else f'chain_2_regular_{base_type}'
                    else:
                        # Combined transaction plots (e.g., 'failure__chain1' -> 'chain_1_failure')
                        base_type = base_parts[0]
                        data_key = f'chain_1_{base_type}' if chain_num == 'chain1' else f'chain_2_{base_type}'
                else:
                    # fallback (should not happen)
                    data_key = None
                
                if data_key in run_data:
                    tx_entries = run_data[data_key]
                    if tx_entries:
                        # Extract block heights and transaction counts
                        heights = [entry['height'] for entry in tx_entries]
                        tx_counts = [entry['count'] for entry in tx_entries]
                        
                        # Plot with color based on run (even if all values are zero)
                        ax.plot(heights, tx_counts, color=colors[run_idx], alpha=0.7, 
                                label=label, linewidth=1.5)
                        plotted_runs += 1
                
            except Exception as e:
                print(f"Warning: Error processing {tx_type} for run {run_dir}: {e}")
                continue
        
        # Create title
        tx_type_display = tx_type.replace('_', ' ').title()
        ax.set_title(f'{tx_type_display} Transactions Over Time - Simple Simulation')
        ax.set_xlabel('Block Height')
        ax.set_ylabel(f'Number of {tx_type_display} Transactions')
        ax.grid(True, alpha=0.3)
        if plotted_runs > 0:
            ax.legend()
            # Set y-axis to start from 0 and show some range
            ax.set_ylim(bottom=0)
        else:
            # If no data was plotted, set a default range
            ax.set_ylim(0, 10)
            ax.text(0.5, 0.5, 'No data available', ha='center', va='center', transform=ax.transAxes)
        
        plt.tight_layout()
        
        # Save the transaction plot
        plt.savefig(f'{sim_figs_dir}/tx_{tx_type}.png', dpi=300, bbox_inches='tight')
        plt.close()
    
    # Create combined transaction plots (sumTypes) that combine CAT and regular transactions
    combined_types = [
        ('pending', 'pending_sumTypes__chain1'),
        ('success', 'success_sumTypes__chain1'),
        ('failure', 'failure_sumTypes__chain1'),
        ('pending', 'pending_sumTypes__chain2'),
        ('success', 'success_sumTypes__chain2'),
        ('failure', 'failure_sumTypes__chain2')
    ]
    
    for base_type, combined_name in combined_types:
        # Determine which chain this is for
        chain_num = combined_name.split('__')[1]
        chain_id = 'chain_1' if chain_num == 'chain1' else 'chain_2'
        
        fig, ax = plt.subplots(figsize=(12, 8))
        
        # Plot each run's combined transaction data
        plotted_runs = 0
        for run_idx, run_dir in enumerate(run_dirs):
            # Create label - show first 5, then "...", then last 5 if more than 10 runs
            label = create_run_label(run_idx, len(run_dirs))
            try:
                # Load CAT and regular transaction data for this run using the same logic as individual plots
                cat_file = os.path.join(sim_data_dir, run_dir, 'data', f'cat_{base_type}_transactions_{chain_id}.json')
                regular_file = os.path.join(sim_data_dir, run_dir, 'data', f'regular_{base_type}_transactions_{chain_id}.json')
                
                if not os.path.exists(cat_file) or not os.path.exists(regular_file):
                    continue
                
                # Load CAT data
                with open(cat_file, 'r') as f:
                    cat_data = json.load(f)
                
                # Load regular data
                with open(regular_file, 'r') as f:
                    regular_data = json.load(f)
                
                # Get the data keys using the same logic as individual plots
                cat_key = f'{chain_id}_cat_{base_type}'
                regular_key = f'{chain_id}_regular_{base_type}'
                
                # Extract data using the same approach as individual plots
                cat_heights = []
                cat_counts = []
                if cat_key in cat_data:
                    cat_entries = cat_data[cat_key]
                    if cat_entries:
                        cat_heights = [entry['height'] for entry in cat_entries]
                        cat_counts = [entry['count'] for entry in cat_entries]
                
                regular_heights = []
                regular_counts = []
                if regular_key in regular_data:
                    regular_entries = regular_data[regular_key]
                    if regular_entries:
                        regular_heights = [entry['height'] for entry in regular_entries]
                        regular_counts = [entry['count'] for entry in regular_entries]
                
                # Combine the data by adding counts at each height
                combined_data = {}
                
                # Add CAT data
                for height, count in zip(cat_heights, cat_counts):
                    combined_data[height] = combined_data.get(height, 0) + count
                
                # Add regular data
                for height, count in zip(regular_heights, regular_counts):
                    combined_data[height] = combined_data.get(height, 0) + count
                
                # Convert back to sorted list
                combined_entries = sorted(combined_data.items())
                
                # Extract block heights and transaction counts
                heights = [entry[0] for entry in combined_entries]
                tx_counts = [entry[1] for entry in combined_entries]
                
                # Plot with color based on run
                ax.plot(heights, tx_counts, color=colors[run_idx], alpha=0.7, 
                        label=label, linewidth=1.5)
                plotted_runs += 1
                
            except Exception as e:
                print(f"Warning: Error processing combined {base_type} for run {run_dir}: {e}")
                continue
        
        # Create title
        base_type_display = base_type.replace('_', ' ').title()
        chain_display = 'Chain 1' if chain_num == 'chain1' else 'Chain 2'
        ax.set_title(f'{base_type_display} Transactions (CAT + Regular) Over Time - {chain_display} - Simple Simulation')
        ax.set_xlabel('Block Height')
        ax.set_ylabel(f'Number of {base_type_display} Transactions')
        ax.grid(True, alpha=0.3)
        if plotted_runs > 0:
            ax.legend()
            # Set y-axis to start from 0 and show some range
            ax.set_ylim(bottom=0)
        else:
            # If no data was plotted, set a default range
            ax.set_ylim(0, 10)
            ax.text(0.5, 0.5, 'No data available', ha='center', va='center', transform=ax.transAxes)
        
        plt.tight_layout()
        
        # Save the combined transaction plot
        plt.savefig(f'{sim_figs_dir}/tx_{combined_name}.png', dpi=300, bbox_inches='tight')
        plt.close()
    
    # Create system CPU usage plot
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's system CPU usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        try:
            # Load system CPU usage data for this run
            cpu_file = os.path.join(sim_data_dir, run_dir, 'data', 'system_cpu.json')
            if not os.path.exists(cpu_file):
                print(f"Warning: {cpu_file} not found")
                continue
            
            with open(cpu_file, 'r') as f:
                run_data = json.load(f)
            
            # Extract system CPU usage data
            if 'system_cpu' in run_data:
                cpu_entries = run_data['system_cpu']
                if cpu_entries:
                    # Extract block heights and CPU usage values
                    heights = [entry['height'] for entry in cpu_entries]
                    cpu_values = [entry['percent'] for entry in cpu_entries]  # Already in percent
                    
                    # Plot with color based on run
                    ax.plot(heights, cpu_values, color=colors[run_idx], alpha=0.7, 
                            label=create_run_label(run_idx, len(run_dirs)), linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing system CPU for run {run_dir}: {e}")
            continue
    
    # Create title for system CPU plot
    ax.set_title('System CPU Usage Over Time - Simple Simulation')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System CPU Usage (%)')
    ax.grid(True, alpha=0.3)
    ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_cpu.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # Create filtered system CPU usage plot (removes spikes above 30%)
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's filtered system CPU usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        try:
            # Load system CPU usage data for this run
            cpu_file = os.path.join(sim_data_dir, run_dir, 'data', 'system_cpu.json')
            if not os.path.exists(cpu_file):
                print(f"Warning: {cpu_file} not found")
                continue
            
            with open(cpu_file, 'r') as f:
                run_data = json.load(f)
            
            # Extract system CPU usage data
            if 'system_cpu' in run_data:
                cpu_entries = run_data['system_cpu']
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
                    
                    # Plot filtered data with color based on run
                    if filtered_heights and filtered_cpu_values:
                        ax.plot(filtered_heights, filtered_cpu_values, color=colors[run_idx], alpha=0.7, 
                                label=create_run_label(run_idx, len(run_dirs)), linewidth=1.5)
                        plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing filtered system CPU for run {run_dir}: {e}")
            continue
    
    # Create title for filtered system CPU plot
    ax.set_title('System CPU Usage Over Time (Filtered ≤30%) - Simple Simulation')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System CPU Usage (%)')

    ax.grid(True, alpha=0.3)
    ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_cpu_filtered.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # Create system total CPU usage plot
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's system total CPU usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        try:
            # Load system total CPU usage data for this run
            cpu_file = os.path.join(sim_data_dir, run_dir, 'data', 'system_total_cpu.json')
            if not os.path.exists(cpu_file):
                print(f"Warning: {cpu_file} not found")
                continue
            
            with open(cpu_file, 'r') as f:
                run_data = json.load(f)
            
            # Extract system total CPU usage data
            if 'system_total_cpu' in run_data:
                cpu_entries = run_data['system_total_cpu']
                if cpu_entries:
                    # Extract block heights and CPU usage values
                    heights = [entry['height'] for entry in cpu_entries]
                    cpu_values = [entry['percent'] for entry in cpu_entries]  # Already in percent
                    
                    # Plot with color based on run
                    ax.plot(heights, cpu_values, color=colors[run_idx], alpha=0.7, 
                            label=create_run_label(run_idx, len(run_dirs)), linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing system total CPU for run {run_dir}: {e}")
            continue
    
    # Create title for system total CPU plot
    ax.set_title('System Total CPU Usage Over Time - Simple Simulation')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System Total CPU Usage (%)')
    ax.grid(True, alpha=0.3)
    ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_total_cpu.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # Create loop steps without transaction issuance plot
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's loop steps data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        try:
            # Load loop steps data for this run
            loop_steps_file = os.path.join(sim_data_dir, run_dir, 'data', 'loop_steps_without_tx_issuance.json')
            if not os.path.exists(loop_steps_file):
                print(f"Warning: {loop_steps_file} not found")
                continue
            
            with open(loop_steps_file, 'r') as f:
                run_data = json.load(f)
            
            # Extract loop steps data
            if 'loop_steps_without_tx_issuance' in run_data:
                loop_steps_entries = run_data['loop_steps_without_tx_issuance']
                if loop_steps_entries:
                    # Extract block heights and loop steps values
                    heights = [entry['height'] for entry in loop_steps_entries]
                    loop_steps_values = [entry['count'] for entry in loop_steps_entries]
                    
                    # Plot with color based on run
                    ax.plot(heights, loop_steps_values, color=colors[run_idx], alpha=0.7, 
                            label=create_run_label(run_idx, len(run_dirs)), linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing loop steps for run {run_dir}: {e}")
            continue
    
    # Create title for loop steps plot
    ax.set_title('Loop Steps Without Transaction Issuance Over Time - Simple Simulation')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('Loop Steps Count')
    ax.grid(True, alpha=0.3)
    ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/loop_steps_without_tx_issuance.png', dpi=300, bbox_inches='tight')
    plt.close()

# ------------------------------------------------------------------------------------------------
# Locked Keys Plotting Functions
# ------------------------------------------------------------------------------------------------

def plot_locked_keys():
    """
    Plot locked keys data from both chains.
    """
    try:
        # Load locked keys data from chain 1
        with open(f'{BASE_DATA_PATH}/locked_keys_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        chain_1_blocks = [entry['height'] for entry in chain_1_data['chain_1_locked_keys']]
        chain_1_locked_keys = [entry['count'] for entry in chain_1_data['chain_1_locked_keys']]
        
        # Load locked keys data from chain 2
        with open(f'{BASE_DATA_PATH}/locked_keys_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        chain_2_blocks = [entry['height'] for entry in chain_2_data['chain_2_locked_keys']]
        chain_2_locked_keys = [entry['count'] for entry in chain_2_data['chain_2_locked_keys']]
        
        # Create the plot
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_locked_keys, 'b-', label='Chain 1', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_locked_keys, 'r--', label='Chain 2', linewidth=2)
        plt.title('Locked Keys by Block Height (Averaged)')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Locked Keys')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend()
        
        # Save the plot
        plt.savefig(f'{FIGS_PATH}/locked_keys.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing locked keys data: {e}")
        return

def plot_locked_keys_with_pending():
    """
    Plot locked keys data alongside pending transactions for comparison.
    """
    try:
        # Load locked keys data
        with open(f'{BASE_DATA_PATH}/locked_keys_chain_1.json', 'r') as f:
            locked_keys_data = json.load(f)
        blocks = [entry['height'] for entry in locked_keys_data['chain_1_locked_keys']]
        locked_keys = [entry['count'] for entry in locked_keys_data['chain_1_locked_keys']]
        
        # Load CAT pending transactions data
        try:
            with open(f'{BASE_DATA_PATH}/cat_pending_transactions_chain_1.json', 'r') as f:
                cat_pending_data = json.load(f)
            cat_pending_transactions = [entry['count'] for entry in cat_pending_data['chain_1_cat_pending']]
        except (FileNotFoundError, json.JSONDecodeError, KeyError):
            cat_pending_transactions = [0] * len(blocks)
        
        # Load regular pending transactions data
        try:
            with open(f'{BASE_DATA_PATH}/regular_pending_transactions_chain_1.json', 'r') as f:
                regular_pending_data = json.load(f)
            regular_pending_transactions = [entry['count'] for entry in regular_pending_data['chain_1_regular_pending']]
        except (FileNotFoundError, json.JSONDecodeError, KeyError):
            regular_pending_transactions = [0] * len(blocks)
        
        # Create the plot
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8), sharex=True)
        
        # Plot locked keys and CAT pending
        ax1.plot(blocks, locked_keys, 'b-', linewidth=2, label='Locked Keys')
        ax1.plot(blocks, cat_pending_transactions, 'orange', linewidth=2, label='CAT Pending')
        ax1.set_ylabel('Count')
        ax1.set_title('Locked Keys vs Pending Transactions (Chain 1) - Averaged')
        ax1.grid(True, alpha=0.3)
        ax1.legend()
        
        # Plot pending transactions (CAT and regular)
        ax2.plot(blocks, cat_pending_transactions, 'orange', linewidth=2, label='CAT Pending')
        ax2.plot(blocks, regular_pending_transactions, 'green', linewidth=2, label='Regular Pending')
        ax2.set_xlabel('Block Height')
        ax2.set_ylabel('Number of Pending Transactions')
        ax2.grid(True, alpha=0.3)
        ax2.legend()
        
        plt.tight_layout()
        
        # Save the plot
        plt.savefig(f'{FIGS_PATH}/locked_keys_and_tx_pending.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing data for comparison plot: {e}")
        return

def plot_transactions_per_block():
    """
    Plot transactions per block and TPS for both chains.
    """
    try:
        # Load transactions per block data from chain 1
        with open(f'{BASE_DATA_PATH}/tx_per_block_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        chain_1_blocks = [entry['height'] for entry in chain_1_data['chain_1_tx_per_block']]
        chain_1_tx_per_block = [entry['count'] for entry in chain_1_data['chain_1_tx_per_block']]
        
        # Load transactions per block data from chain 2
        with open(f'{BASE_DATA_PATH}/tx_per_block_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        chain_2_blocks = [entry['height'] for entry in chain_2_data['chain_2_tx_per_block']]
        chain_2_tx_per_block = [entry['count'] for entry in chain_2_data['chain_2_tx_per_block']]
        
        # Load block interval from simulation stats to calculate TPS
        with open(f'{BASE_DATA_PATH}/simulation_stats.json', 'r') as f:
            stats_data = json.load(f)
        block_interval = stats_data['parameters']['block_interval']  # in seconds
        
        # Calculate TPS (transactions per second)
        chain_1_tps = [tx_count / block_interval for tx_count in chain_1_tx_per_block]
        chain_2_tps = [tx_count / block_interval for tx_count in chain_2_tx_per_block]
        
        # Create subplots
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 10), sharex=True)
        
        # Plot 1: Transactions per Block
        ax1.plot(chain_1_blocks, chain_1_tx_per_block, 'b-', label='Chain 1', linewidth=2)
        ax1.plot(chain_2_blocks, chain_2_tx_per_block, 'r--', label='Chain 2', linewidth=2)
        ax1.set_title('Transactions per Block (Averaged)')
        ax1.set_ylabel('Number of Transactions')
        ax1.grid(True, alpha=0.3)
        ax1.legend()
        
        # Plot 2: TPS
        ax2.plot(chain_1_blocks, chain_1_tps, 'b-', label='Chain 1', linewidth=2)
        ax2.plot(chain_2_blocks, chain_2_tps, 'r--', label='Chain 2', linewidth=2)
        ax2.set_title(f'Transactions per Second (Block Interval: {block_interval}s)')
        ax2.set_xlabel('Block Height')
        ax2.set_ylabel('TPS')
        ax2.grid(True, alpha=0.3)
        ax2.legend()
        
        plt.tight_layout()
        
        # Save the plot
        plt.savefig(f'{FIGS_PATH}/tps.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing transactions per block data: {e}")
        return

def plot_system_memory():
    """
    Plot system memory usage over time.
    """
    try:
        # Load system memory usage data
        with open(f'{BASE_DATA_PATH}/system_memory.json', 'r') as f:
            memory_data = json.load(f)
        
        # Extract system memory usage data
        if 'system_memory' in memory_data:
            memory_entries = memory_data['system_memory']
            if memory_entries:
                # Extract block heights and memory usage values
                heights = [entry['height'] for entry in memory_entries]
                memory_values = [entry['bytes'] / (1024 * 1024) for entry in memory_entries]  # Convert to MB
                
                # Create the plot
                plt.figure(figsize=(12, 6))
                plt.plot(heights, memory_values, 'g-', linewidth=2)
                plt.title('System Memory Usage Over Time (Averaged)')
                plt.xlabel('Block Height')
                plt.ylabel('System Memory Usage (MB)')
                plt.grid(True, alpha=0.3)
                
                # Save the plot
                plt.savefig(f'{FIGS_PATH}/system_memory.png', dpi=300, bbox_inches='tight')
                plt.close()
            else:
                print("Warning: No system memory data found")
        else:
            print("Warning: System memory data not found in expected format")
            
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing system memory data: {e}")
        return

def plot_system_total_memory():
    """
    Plot system total RAM usage over time.
    """
    try:
        # Load system total memory usage data
        with open(f'{BASE_DATA_PATH}/system_total_memory.json', 'r') as f:
            system_total_memory_data = json.load(f)
        
        # Extract system total memory usage data
        if 'system_total_memory' in system_total_memory_data:
            system_total_memory_entries = system_total_memory_data['system_total_memory']
            if system_total_memory_entries:
                # Extract block heights and system total memory usage values
                heights = [entry['height'] for entry in system_total_memory_entries]
                system_total_memory_values = [entry['bytes'] / (1024 * 1024 * 1024) for entry in system_total_memory_entries]  # Convert to GB
                
                # Create the plot
                plt.figure(figsize=(12, 6))
                plt.plot(heights, system_total_memory_values, 'm-', linewidth=2)
                plt.title('System Total Memory Usage Over Time (Averaged)')
                plt.xlabel('Block Height')
                plt.ylabel('System Total Memory Usage (GB)')
                plt.grid(True, alpha=0.3)
                
                # Save the plot
                plt.savefig(f'{FIGS_PATH}/system_total_memory.png', dpi=300, bbox_inches='tight')
                plt.close()
            else:
                print("Warning: No system total memory data found")
        else:
            print("Warning: System total memory data not found in expected format")
            
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing system total memory data: {e}")
        return

def plot_system_cpu():
    """
    Plot system CPU usage over time.
    """
    try:
        # Load system CPU usage data
        with open(f'{BASE_DATA_PATH}/system_cpu.json', 'r') as f:
            cpu_data = json.load(f)
        
        # Extract system CPU usage data
        if 'system_cpu' in cpu_data:
            cpu_entries = cpu_data['system_cpu']
            if cpu_entries:
                # Extract block heights and CPU usage values
                heights = [entry['height'] for entry in cpu_entries]
                cpu_values = [entry['percent'] for entry in cpu_entries]  # Already in percent
                
                # Create the plot
                plt.figure(figsize=(12, 6))
                plt.plot(heights, cpu_values, 'r-', linewidth=2)
                plt.title('System CPU Usage Over Time (Averaged)')
                plt.xlabel('Block Height')
                plt.ylabel('System CPU Usage (%)')
                plt.grid(True, alpha=0.3)
                
                # Save the plot
                plt.savefig(f'{FIGS_PATH}/system_cpu.png', dpi=300, bbox_inches='tight')
                plt.close()
            else:
                print("Warning: No system CPU data found")
        else:
            print("Warning: System CPU data not found in expected format")
            
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing system CPU data: {e}")
        return

def plot_system_cpu_filtered():
    """
    Plot system CPU usage over time with spikes above 30% filtered out.
    """
    try:
        # Load system CPU usage data
        with open(f'{BASE_DATA_PATH}/system_cpu.json', 'r') as f:
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
                
                # Create the plot
                plt.figure(figsize=(12, 6))
                plt.plot(filtered_heights, filtered_cpu_values, 'r-', linewidth=2)
                plt.title('System CPU Usage Over Time (Filtered ≤30%, Averaged)')
                plt.xlabel('Block Height')
                plt.ylabel('System CPU Usage (%)')

                plt.grid(True, alpha=0.3)
                
                # Save the plot
                plt.savefig(f'{FIGS_PATH}/system_cpu_filtered.png', dpi=300, bbox_inches='tight')
                plt.close()
            else:
                print("Warning: No system CPU data found")
        else:
            print("Warning: System CPU data not found in expected format")
            
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing filtered system CPU data: {e}")
        return

def plot_system_total_cpu():
    """
    Plot system total CPU usage over time.
    """
    try:
        # Load system total CPU usage data
        with open(f'{BASE_DATA_PATH}/system_total_cpu.json', 'r') as f:
            cpu_data = json.load(f)
        
        # Extract system total CPU usage data
        if 'system_total_cpu' in cpu_data:
            cpu_entries = cpu_data['system_total_cpu']
            if cpu_entries:
                # Extract block heights and CPU usage values
                heights = [entry['height'] for entry in cpu_entries]
                cpu_values = [entry['percent'] for entry in cpu_entries]  # Already in percent
                
                # Create the plot
                plt.figure(figsize=(12, 6))
                plt.plot(heights, cpu_values, 'orange', linewidth=2)
                plt.title('System Total CPU Usage Over Time (Averaged)')
                plt.xlabel('Block Height')
                plt.ylabel('System Total CPU Usage (%)')
                plt.grid(True, alpha=0.3)
                
                # Save the plot
                plt.savefig(f'{FIGS_PATH}/system_total_cpu.png', dpi=300, bbox_inches='tight')
                plt.close()
            else:
                print("Warning: No system total CPU data found")
        else:
            print("Warning: System total CPU data not found in expected format")
            
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing system total CPU data: {e}")
        return

def plot_loop_steps_without_tx_issuance():
    """
    Plot loop steps without transaction issuance over time.
    """
    try:
        # Load loop steps data
        with open(f'{BASE_DATA_PATH}/loop_steps_without_tx_issuance.json', 'r') as f:
            loop_steps_data = json.load(f)
        
        # Extract loop steps data
        if 'loop_steps_without_tx_issuance' in loop_steps_data:
            loop_steps_entries = loop_steps_data['loop_steps_without_tx_issuance']
            if loop_steps_entries:
                # Extract block heights and loop steps values
                heights = [entry['height'] for entry in loop_steps_entries]
                loop_steps_values = [entry['count'] for entry in loop_steps_entries]
                
                # Create the plot
                plt.figure(figsize=(12, 6))
                plt.plot(heights, loop_steps_values, 'purple', linewidth=2)
                plt.title('Loop Steps Without Transaction Issuance Over Time (Averaged)')
                plt.xlabel('Block Height')
                plt.ylabel('Loop Steps Count')
                plt.grid(True, alpha=0.3)
                
                # Save the plot
                plt.savefig(f'{FIGS_PATH}/loop_steps_without_tx_issuance.png', dpi=300, bbox_inches='tight')
                plt.close()
            else:
                print("Warning: No loop steps data found")
        else:
            print("Warning: Loop steps data not found in expected format")
            
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing loop steps data: {e}")
        return

def main():
    """Main function to run all plotting functions for the simple simulation."""
    # Check if simple simulation data exists (try multiple possible paths)
    metadata_paths = [
        '../../../results/sim_simple/data/metadata.json',  # From sim_simple directory
        'simulator/results/sim_simple/data/metadata.json',  # From simulator root
        'results/sim_simple/data/metadata.json'  # From simulator root alternative
    ]
    
    metadata_exists = any(os.path.exists(path) for path in metadata_paths)
    if not metadata_exists:
        print("No simple simulation data found. Skipping plots.")
        print("Please run the simple simulation first (option 1).")
        return True
    
    # Determine the correct results directory path
    results_dir = None
    for path in metadata_paths:
        if os.path.exists(path):
            # Get the absolute path to the results directory
            # From sim_simple directory, go up to simulator root, then to results/sim_simple
            results_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', '..', 'results', 'sim_simple'))
            break
    
    if not results_dir:
        print("Could not determine results directory path.")
        return False
    
    # Run the averaging script first
    result = subprocess.run([sys.executable, '../../average_runs.py', results_dir], 
                          capture_output=True, text=True, cwd=os.path.dirname(__file__))
    
    # Always show the output from the averaging script
    if result.stdout:
        print("Averaging script output:")
        print(result.stdout)
    if result.stderr:
        print("Averaging script errors:")
        print(result.stderr)
    
    if result.returncode != 0:
        print(f"Error: Averaging failed with return code {result.returncode}")
        return False
    
    os.makedirs(FIGS_PATH, exist_ok=True)
    
    # Plot account selection distributions
    plot_account_selection()
    # Plot pending transactions
    plot_tx_pending()
    # Plot success transactions
    plot_tx_success()
    # Plot failure transactions
    plot_tx_failure()
    # Plot simulation parameters
    plot_parameters()
    # Plot locked keys data
    plot_locked_keys()
    plot_locked_keys_with_pending()
    
    # Plot transactions per block
    plot_transactions_per_block()
    
    # Plot system memory usage
    plot_system_memory()
    
    # Plot system total memory usage
    plot_system_total_memory()
    
    # Plot system CPU usage
    plot_system_cpu()
    plot_system_cpu_filtered() # Added this line
    
    # Plot system total CPU usage
    plot_system_total_cpu()
    
    # Plot loop steps without transaction issuance
    plot_loop_steps_without_tx_issuance()
    
    # Create per-run plots in sim_0 directory
    create_per_run_plots()
    
    # Plot comparison charts
    plot_tx_allStatus_cat()
    plot_tx_allStatus_regular()
    plot_tx_allStatus_all()
    plot_comprehensive_comparison()

if __name__ == "__main__":
    main() 