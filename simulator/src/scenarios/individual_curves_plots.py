import os
import json
import numpy as np
import matplotlib.pyplot as plt
from typing import List, Dict, Any, Tuple

# Global colormap setting - easily switch between different colormaps
# Options: 'viridis', 'RdYlBu_r', 'plasma', 'inferno', 'magma', 'cividis'
COLORMAP = 'viridis'  # Change this to switch colormaps globally


def calculate_running_average(data: List[float], window_size: int = 10) -> List[float]:
    """
    Calculate running average of data with specified window size.
    """
    if len(data) < window_size:
        return data
    
    running_avg = []
    for i in range(len(data)):
        start = max(0, i - window_size + 1)
        window_data = data[start:i + 1]
        running_avg.append(sum(window_data) / len(window_data))
    
    return running_avg


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


def create_per_run_plots(sim_data_dir: str, sim_figs_dir: str, block_interval: float = None):
    """
    Create per-run plots showing individual runs with different colors.
    
    Args:
        sim_data_dir: Directory containing run data
        sim_figs_dir: Directory to save plot figures
        block_interval: Block interval in seconds (for TPS calculation)
    """
    os.makedirs(sim_figs_dir, exist_ok=True)
    
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
    
    # Create color gradient for runs
    colors = plt.cm.get_cmap(COLORMAP)(np.linspace(0, 1, len(run_dirs)))
    
    # Plot TPS if block_interval is provided
    if block_interval is not None:
        create_tps_plot(run_dirs, sim_data_dir, sim_figs_dir, colors, block_interval)
    
    # Create system memory usage plot
    create_system_memory_plot(run_dirs, sim_data_dir, sim_figs_dir, colors)
    
    # Create system total memory usage plot
    create_system_total_memory_plot(run_dirs, sim_data_dir, sim_figs_dir, colors)
    
    # Create system CPU usage plot
    create_system_cpu_plot(run_dirs, sim_data_dir, sim_figs_dir, colors)
    
    # Create system CPU filtered plot
    create_system_cpu_filtered_plot(run_dirs, sim_data_dir, sim_figs_dir, colors)
    
    # Create system total CPU usage plot
    create_system_total_cpu_plot(run_dirs, sim_data_dir, sim_figs_dir, colors)
    
    # Create loop steps plot
    create_loop_steps_plot(run_dirs, sim_data_dir, sim_figs_dir, colors)
    
    # Create transaction plots
    create_transaction_plots(run_dirs, sim_data_dir, sim_figs_dir, colors)


def create_tps_plot(run_dirs: List[str], sim_data_dir: str, sim_figs_dir: str, colors: np.ndarray, block_interval: float):
    """Create TPS (Transactions Per Second) plot."""
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 10), sharex=True)
    
    # Plot each run's data for TPS
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
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
            
            # Plot with color based on run (plot all runs, only add label if it should appear in legend)
            ax1.plot(blocks, tx_per_block_smoothed, color=colors[run_idx], alpha=0.7, 
                    label=label, linewidth=1.5)
            ax2.plot(blocks, tps_smoothed, color=colors[run_idx], alpha=0.7, 
                    label=label, linewidth=1.5)
            plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing run {run_dir} for TPS plot: {e}")
            continue
    
    # Create titles for TPS plot
    title = f'Transactions per Block (Chain 1) - Per Run Analysis (20-block running average)'
    ax1.set_title(title)
    ax1.set_ylabel('Number of Transactions')
    ax1.grid(True, alpha=0.3)
    if plotted_runs > 0:
        ax1.legend()
    
    ax2.set_title(f'Transactions per Second (Chain 1) - Per Run Analysis (Block Interval: {block_interval}s, 20-block running average)')
    ax2.set_xlabel('Block Height')
    ax2.set_ylabel('TPS')
    ax2.grid(True, alpha=0.3)
    if plotted_runs > 0:
        ax2.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/tps_individual_runs.png', dpi=300, bbox_inches='tight')
    plt.close()


def create_system_memory_plot(run_dirs: List[str], sim_data_dir: str, sim_figs_dir: str, colors: np.ndarray):
    """Create system memory usage plot."""
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's system memory usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
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
                    
                    # Plot with color based on run (plot all runs, only add label if it should appear in legend)
                    ax.plot(heights, memory_values, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing system memory for run {run_dir}: {e}")
            continue
    
    # Create title for system memory plot
    ax.set_title('System Memory Usage Over Time - Per Run Analysis')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System Memory Usage (MB)')
    ax.grid(True, alpha=0.3)
    if plotted_runs > 0:
        ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_memory_individual_runs.png', dpi=300, bbox_inches='tight')
    plt.close()


def create_system_total_memory_plot(run_dirs: List[str], sim_data_dir: str, sim_figs_dir: str, colors: np.ndarray):
    """Create system total memory usage plot."""
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's system total memory usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
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
                    
                    # Plot with color based on run (plot all runs, only add label if it should appear in legend)
                    ax.plot(heights, memory_values, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing system total memory for run {run_dir}: {e}")
            continue
    
    # Create title for system total memory plot
    ax.set_title('System Total Memory Usage Over Time - Per Run Analysis')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System Total Memory Usage (GB)')
    ax.grid(True, alpha=0.3)
    if plotted_runs > 0:
        ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_total_memory_individual_runs.png', dpi=300, bbox_inches='tight')
    plt.close()


def create_system_cpu_plot(run_dirs: List[str], sim_data_dir: str, sim_figs_dir: str, colors: np.ndarray):
    """Create system CPU usage plot."""
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's system CPU usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        label = create_run_label(run_idx, len(run_dirs))
        
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
                    
                    # Plot with color based on run (plot all runs, only add label if it should appear in legend)
                    ax.plot(heights, cpu_values, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing system CPU for run {run_dir}: {e}")
            continue
    
    # Create title for system CPU plot
    ax.set_title('System CPU Usage Over Time - Per Run Analysis')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System CPU Usage (%)')
    ax.grid(True, alpha=0.3)
    if plotted_runs > 0:
        ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_cpu_individual_runs.png', dpi=300, bbox_inches='tight')
    plt.close()


def create_system_cpu_filtered_plot(run_dirs: List[str], sim_data_dir: str, sim_figs_dir: str, colors: np.ndarray):
    """Create filtered system CPU usage plot (removes spikes above 30%)."""
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's filtered system CPU usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        label = create_run_label(run_idx, len(run_dirs))
        
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
                    
                    # Plot filtered data with color based on run (plot all runs, only add label if it should appear in legend)
                    if filtered_heights and filtered_cpu_values:
                        ax.plot(filtered_heights, filtered_cpu_values, color=colors[run_idx], alpha=0.7, 
                                label=label, linewidth=1.5)
                        plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing filtered system CPU for run {run_dir}: {e}")
            continue
    
    # Create title for filtered system CPU plot
    ax.set_title('System CPU Usage Over Time (Filtered ≤30%) - Per Run Analysis')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System CPU Usage (%)')
    ax.grid(True, alpha=0.3)
    if plotted_runs > 0:
        ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_cpu_filtered_individual_runs.png', dpi=300, bbox_inches='tight')
    plt.close()


def create_system_total_cpu_plot(run_dirs: List[str], sim_data_dir: str, sim_figs_dir: str, colors: np.ndarray):
    """Create system total CPU usage plot."""
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's system total CPU usage data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        label = create_run_label(run_idx, len(run_dirs))
        
        try:
            # Load system total CPU usage data for this run
            total_cpu_file = os.path.join(sim_data_dir, run_dir, 'data', 'system_total_cpu.json')
            if not os.path.exists(total_cpu_file):
                print(f"Warning: {total_cpu_file} not found")
                continue
            
            with open(total_cpu_file, 'r') as f:
                run_data = json.load(f)
            
            # Extract system total CPU usage data
            if 'system_total_cpu' in run_data:
                total_cpu_entries = run_data['system_total_cpu']
                if total_cpu_entries:
                    # Extract block heights and total CPU usage values
                    heights = [entry['height'] for entry in total_cpu_entries]
                    total_cpu_values = [entry['percent'] for entry in total_cpu_entries]  # Already in percent
                    
                    # Plot with color based on run (plot all runs, only add label if it should appear in legend)
                    ax.plot(heights, total_cpu_values, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing system total CPU for run {run_dir}: {e}")
            continue
    
    # Create title for system total CPU plot
    ax.set_title('System Total CPU Usage Over Time - Per Run Analysis')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('System Total CPU Usage (%)')
    ax.grid(True, alpha=0.3)
    if plotted_runs > 0:
        ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/system_total_cpu_individual_runs.png', dpi=300, bbox_inches='tight')
    plt.close()


def create_loop_steps_plot(run_dirs: List[str], sim_data_dir: str, sim_figs_dir: str, colors: np.ndarray):
    """Create loop steps without transaction issuance plot."""
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Plot each run's loop steps data
    plotted_runs = 0
    for run_idx, run_dir in enumerate(run_dirs):
        label = create_run_label(run_idx, len(run_dirs))
        
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
                    
                    # Plot with color based on run (plot all runs, only add label if it should appear in legend)
                    ax.plot(heights, loop_steps_values, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    plotted_runs += 1
            
        except Exception as e:
            print(f"Warning: Error processing loop steps for run {run_dir}: {e}")
            continue
    
    # Create title for loop steps plot
    ax.set_title('Loop Steps Without Transaction Issuance Over Time - Per Run Analysis')
    ax.set_xlabel('Block Height')
    ax.set_ylabel('Loop Steps Count')
    ax.grid(True, alpha=0.3)
    if plotted_runs > 0:
        ax.legend()
    
    plt.tight_layout()
    plt.savefig(f'{sim_figs_dir}/loop_steps_without_tx_issuance_individual_runs.png', dpi=300, bbox_inches='tight')
    plt.close()


def create_transaction_plots(run_dirs: List[str], sim_data_dir: str, sim_figs_dir: str, colors: np.ndarray):
    """Create transaction plots for different transaction types."""
    # Define transaction types to plot with their file names and data keys
    transaction_types = [
        # Chain 1 transaction plots
        ('cat_pending_transactions_chain_1', 'pending_cat__chain1'),
        ('cat_success_transactions_chain_1', 'success_cat__chain1'),
        ('cat_failure_transactions_chain_1', 'failure_cat__chain1'),
        ('regular_pending_transactions_chain_1', 'pending_regular__chain1'),
        ('regular_success_transactions_chain_1', 'success_regular__chain1'),
        ('regular_failure_transactions_chain_1', 'failure_regular__chain1'),
        # Chain 2 transaction plots
        ('cat_pending_transactions_chain_2', 'pending_cat__chain2'),
        ('cat_success_transactions_chain_2', 'success_cat__chain2'),
        ('cat_failure_transactions_chain_2', 'failure_cat__chain2'),
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
                    print(f"Warning: {tx_file} not found")
                    continue
                
                with open(tx_file, 'r') as f:
                    run_data = json.load(f)
                
                # Extract transaction data - the data is stored as a list of objects with height and count fields
                # Handle different naming patterns
                if '__' in tx_type:
                    # For chain_1 and chain_2 plots
                    chain_num = tx_type.split('__')[1]
                    base_type = tx_type.split('__')[0]
                    # Extract the transaction type (cat/regular) and status (pending/success/failure)
                    if base_type.startswith('pending_'):
                        tx_type_name = base_type.split('_')[1]  # 'cat' or 'regular'
                        status = 'pending'
                    elif base_type.startswith('success_'):
                        tx_type_name = base_type.split('_')[1]  # 'cat' or 'regular'
                        status = 'success'
                    elif base_type.startswith('failure_'):
                        tx_type_name = base_type.split('_')[1]  # 'cat' or 'regular'
                        status = 'failure'
                    else:
                        # Fallback for old format
                        tx_type_name = 'cat' if 'cat' in base_type else 'regular'
                        status = base_type
                    
                    chain_id = 'chain_1' if chain_num == 'chain1' else 'chain_2'
                    data_key = f'{chain_id}_{tx_type_name}_{status}'
                else:
                    # Fallback for other formats
                    data_key = f'chain_1_{tx_type}'
                
                if data_key in run_data:
                    tx_entries = run_data[data_key]
                    if tx_entries:
                        # Extract block heights and transaction counts
                        heights = [entry['height'] for entry in tx_entries]
                        tx_counts = [entry['count'] for entry in tx_entries]
                        
                        # Plot with color based on run (plot all runs, not just legend runs)
                        ax.plot(heights, tx_counts, color=colors[run_idx], alpha=0.7, 
                                label=label, linewidth=1.5)
                        plotted_runs += 1
                
            except Exception as e:
                print(f"Warning: Error processing {tx_type} for run {run_dir}: {e}")
                continue
        
        # Create title
        tx_type_display = tx_type.replace('_', ' ').title()
        ax.set_title(f'{tx_type_display} Transactions Over Time - Per Run Analysis')
        ax.set_xlabel('Block Height')
        ax.set_ylabel(f'Number of {tx_type_display} Transactions')
        ax.grid(True, alpha=0.3)
        if plotted_runs > 0:
            ax.legend()
        
        plt.tight_layout()
        
        # Save the transaction plot with proper categorization
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
                # Load CAT and regular transaction data for this run
                cat_file = os.path.join(sim_data_dir, run_dir, 'data', f'cat_{base_type}_transactions_{chain_id}.json')
                regular_file = os.path.join(sim_data_dir, run_dir, 'data', f'regular_{base_type}_transactions_{chain_id}.json')
                
                if not os.path.exists(cat_file) or not os.path.exists(regular_file):
                    print(f"Warning: {cat_file} or {regular_file} not found")
                    continue
                
                with open(cat_file, 'r') as f:
                    cat_data = json.load(f)
                with open(regular_file, 'r') as f:
                    regular_data = json.load(f)
                
                # Get the data keys
                cat_key = f'{chain_id}_cat_{base_type}'
                regular_key = f'{chain_id}_regular_{base_type}'
                
                if cat_key in cat_data and regular_key in regular_data:
                    cat_entries = cat_data[cat_key]
                    regular_entries = regular_data[regular_key]
                    
                    # Create a dictionary to store combined data by height
                    combined_data = {}
                    
                    # Add CAT transactions
                    for entry in cat_entries:
                        height = entry['height']
                        if height not in combined_data:
                            combined_data[height] = 0
                        combined_data[height] += entry['count']
                    
                    # Add regular transactions
                    for entry in regular_entries:
                        height = entry['height']
                        if height not in combined_data:
                            combined_data[height] = 0
                        combined_data[height] += entry['count']
                    
                    # Convert to sorted lists
                    heights = sorted(combined_data.keys())
                    tx_counts = [combined_data[height] for height in heights]
                    
                    # Plot with color based on run (plot all runs, not just legend runs)
                    ax.plot(heights, tx_counts, color=colors[run_idx], alpha=0.7, 
                            label=label, linewidth=1.5)
                    plotted_runs += 1
            
            except Exception as e:
                print(f"Warning: Error processing combined {base_type} transactions for run {run_dir}: {e}")
                continue
        
        # Create title
        ax.set_title(f'Combined {base_type.title()} Transactions (CAT + Regular) Over Time - Per Run Analysis')
        ax.set_xlabel('Block Height')
        ax.set_ylabel(f'Number of Combined {base_type.title()} Transactions')
        ax.grid(True, alpha=0.3)
        if plotted_runs > 0:
            ax.legend()
        
        plt.tight_layout()
        
        # Save the combined transaction plot with the old naming convention
        plt.savefig(f'{sim_figs_dir}/tx_{combined_name}.png', dpi=300, bbox_inches='tight')
        plt.close() 