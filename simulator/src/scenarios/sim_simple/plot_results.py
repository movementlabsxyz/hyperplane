#!/usr/bin/env python3

import os
import sys
import json
import matplotlib.pyplot as plt
import subprocess
import numpy as np

# Add the current directory to the Python path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Import the reusable run plotting module
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from run_plots import create_per_run_plots as create_per_run_plots_reusable

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
    Create per-run plots in the sim_0 directory using the reusable module.
    """
    sim_figs_dir = f'{FIGS_PATH}/sim_0'
    sim_data_dir = f'simulator/results/sim_simple/data/sim_0'
    
    # Load block interval from simulation stats to calculate TPS
    try:
        with open(f'{BASE_DATA_PATH}/simulation_stats.json', 'r') as f:
            stats_data = json.load(f)
        block_interval = stats_data['parameters']['block_interval']  # in seconds
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Could not load block interval: {e}")
        block_interval = None
    
    # Use the reusable module to create per-run plots
    create_per_run_plots_reusable(sim_data_dir, sim_figs_dir, block_interval)

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