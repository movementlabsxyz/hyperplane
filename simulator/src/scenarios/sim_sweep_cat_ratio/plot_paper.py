#!/usr/bin/env python3
"""
Paper-specific plotting script for CAT Ratio Sweep Simulation

This script generates plots specifically designed for paper publication,
including CAT success percentage with average and individual curves overlaid.
"""

import sys
import os
import json
import matplotlib.pyplot as plt
import numpy as np
from typing import Dict, List, Tuple, Any

# Add the scripts directory to the Python path to import plot_utils
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from plot_utils import create_color_gradient, extract_parameter_value, create_parameter_label, create_sweep_title, trim_time_series_data
from plot_utils_percentage import plot_transaction_percentage

def load_individual_run_data(results_dir: str, param_name: str) -> List[Dict[str, Any]]:
    """
    Load individual run data from data/sim_x/run_y/data/ directories.
    
    Returns a list of run data dictionaries, each containing:
    - param_name: parameter value
    - sim_index: simulation index
    - run_index: run index
    - chain_1_cat_success: success data
    - chain_1_cat_failure: failure data
    """
    individual_runs = []
    
    # Load metadata to get parameter values
    metadata_path = f'{results_dir}/data/metadata.json'
    if not os.path.exists(metadata_path):
        print(f"Warning: No metadata found at {metadata_path}")
        return individual_runs
    
    with open(metadata_path, 'r') as f:
        metadata = json.load(f)
    
    param_values = metadata['parameter_values']
    num_simulations = len(param_values)
    
    # For each simulation
    for sim_index in range(num_simulations):
        param_value = param_values[sim_index]
        sim_dir = f'{results_dir}/data/sim_{sim_index}'
        
        # Find all run directories
        if not os.path.exists(sim_dir):
            continue
            
        run_dirs = [d for d in os.listdir(sim_dir) if d.startswith('run_') and d != 'run_average']
        
        # For each run
        for run_dir in run_dirs:
            run_index = int(run_dir.split('_')[1])
            run_data_dir = f'{sim_dir}/{run_dir}/data'
            
            if not os.path.exists(run_data_dir):
                continue
            
            # Load CAT success and failure data
            cat_success_file = f'{run_data_dir}/cat_success_transactions_chain_1.json'
            cat_failure_file = f'{run_data_dir}/cat_failure_transactions_chain_1.json'
            
            run_data = {
                param_name: param_value,
                'sim_index': sim_index,
                'run_index': run_index
            }
            
            # Load success data
            if os.path.exists(cat_success_file):
                with open(cat_success_file, 'r') as f:
                    success_data = json.load(f)
                    if 'chain_1_cat_success' in success_data:
                        # Convert to list of tuples for plotting
                        time_series_data = []
                        for entry in success_data['chain_1_cat_success']:
                            time_series_data.append((entry['height'], entry['count']))
                        run_data['chain_1_cat_success'] = time_series_data
            
            # Load failure data
            if os.path.exists(cat_failure_file):
                with open(cat_failure_file, 'r') as f:
                    failure_data = json.load(f)
                    if 'chain_1_cat_failure' in failure_data:
                        # Convert to list of tuples for plotting
                        time_series_data = []
                        for entry in failure_data['chain_1_cat_failure']:
                            time_series_data.append((entry['height'], entry['count']))
                        run_data['chain_1_cat_failure'] = time_series_data
            
            individual_runs.append(run_data)
    
    return individual_runs

def plot_cat_success_percentage_with_overlay(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot CAT success percentage with both average and individual curves overlaid.
    
    This creates a plot showing:
    1. Individual curves for each parameter value (lighter colors)
    2. Average curve for each parameter value (darker colors, same color family)
    3. Individual run curves (black, thin lines)
    4. Legend showing the parameter values
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping CAT success percentage plot")
            return
        
        # Load individual run data
        individual_runs = load_individual_run_data(results_dir, param_name)
        print(f"Loaded {len(individual_runs)} individual runs")
        
        # Create figure
        plt.figure(figsize=(10, 6))
        
        # Create color gradient using coolwarm colormap
        colors = plt.colormaps['coolwarm'](np.linspace(0, 1, len(individual_results)))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Group results by parameter value to calculate averages
        param_groups = {}
        
        # First pass: group results by parameter value
        for result in individual_results:
            param_value = extract_parameter_value(result, param_name)
            if param_value not in param_groups:
                param_groups[param_value] = []
            param_groups[param_value].append(result)
        
        # Plot individual curves (lighter) and calculate averages
        for param_value, group_results in param_groups.items():
            # Find the color index for this parameter value
            param_values = sorted([extract_parameter_value(r, param_name) for r in individual_results])
            color_idx = param_values.index(param_value)
            base_color = colors[color_idx]
            
            # Create lighter color for individual curves
            light_color = (*base_color[:3], 0.3)  # 30% opacity
            
            # Plot individual curves for this parameter value
            all_heights = set()
            all_percentages = []
            
            for result in group_results:
                # Get CAT success and failure data
                cat_success_data = result.get('chain_1_cat_success', [])
                cat_failure_data = result.get('chain_1_cat_failure', [])
                
                if not cat_success_data and not cat_failure_data:
                    continue
                
                # Calculate percentage over time
                combined_data = {}
                
                # Add success data
                for height, count in cat_success_data:
                    combined_data[height] = combined_data.get(height, 0) + count
                
                # Add failure data
                for height, count in cat_failure_data:
                    combined_data[height] = combined_data.get(height, 0) + count
                
                # Calculate success percentage at each height (using only success + failure as denominator)
                heights = []
                percentages = []
                
                for height in sorted(combined_data.keys()):
                    success_count = next((count for h, count in cat_success_data if h == height), 0)
                    failure_count = next((count for h, count in cat_failure_data if h == height), 0)
                    
                    # Use only (success + failure) as denominator (same as regular plot)
                    success_failure_total = success_count + failure_count
                    if success_failure_total > 0:
                        percentage = (success_count / success_failure_total) * 100
                        heights.append(height)
                        percentages.append(percentage)
                
                if heights:
                    # Trim the last 10% of data to avoid edge effects
                    trim_idx = int(len(heights) * 0.9)
                    heights = heights[:trim_idx]
                    percentages = percentages[:trim_idx]
                    
                    # Plot with thicker lines for paper (will be overlaid with lighter lines later)
                    label = create_parameter_label(param_name, param_value)
                    plt.plot(heights, percentages, color=base_color, alpha=0.7, 
                            label=label, linewidth=3.0, linestyle='--')
                    
                    # Update maximum height
                    if heights:
                        max_height = max(max_height, max(heights))
        
        # Plot individual run curves (thin lines with same color as corresponding thick line)
        for run_data in individual_runs:
            cat_success_data = run_data.get('chain_1_cat_success', [])
            cat_failure_data = run_data.get('chain_1_cat_failure', [])
            
            if not cat_success_data and not cat_failure_data:
                continue
            
            # Get parameter value for this run
            param_value = run_data[param_name]
            
            # Find the color for this parameter value
            param_values = sorted([extract_parameter_value(r, param_name) for r in individual_results])
            color_idx = param_values.index(param_value)
            base_color = colors[color_idx]
            
            # Calculate percentage over time (same logic as above)
            combined_data = {}
            
            # Add success data
            for height, count in cat_success_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Add failure data
            for height, count in cat_failure_data:
                combined_data[height] = combined_data.get(height, 0) + count
            
            # Calculate success percentage at each height
            heights = []
            percentages = []
            
            for height in sorted(combined_data.keys()):
                success_count = next((count for h, count in cat_success_data if h == height), 0)
                failure_count = next((count for h, count in cat_failure_data if h == height), 0)
                
                success_failure_total = success_count + failure_count
                if success_failure_total > 0:
                    percentage = (success_count / success_failure_total) * 100
                    heights.append(height)
                    percentages.append(percentage)
            
            if heights:
                # Trim the last 10% of data to avoid edge effects
                trim_idx = int(len(heights) * 0.9)
                heights = heights[:trim_idx]
                percentages = percentages[:trim_idx]
                
                # Plot individual run curve (thin, same color as thick line)
                plt.plot(heights, percentages, color=base_color, alpha=0.3, 
                        linewidth=1.5, linestyle='-')
                
                # Update maximum height
                if heights:
                    max_height = max(max_height, max(heights))
        
        # Set x-axis limits
        plt.xlim(left=0, right=max_height)
        
        # Create title and filename (same as regular plot)
        title = f'CAT Success Percentage (of Success+Failure) Over Time - {create_sweep_title(param_name, sweep_type)}'
        filename = 'tx_success_cat_percentage.png'
        
        plt.title(title, fontsize=14)
        plt.xlabel('Block Height', fontsize=12)
        plt.ylabel('CAT Success Percentage (%)', fontsize=12)
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right", fontsize=10)
        plt.tight_layout()
        
        # Create the paper directory and plot
        paper_dir = f'{results_dir}/figs/paper'
        os.makedirs(paper_dir, exist_ok=True)
        plt.savefig(f'{paper_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
        # print(f"Generated paper plot: {filename}")
        
    except Exception as e:
        print(f"Error generating paper plot: {e}")
        import traceback
        traceback.print_exc()
        raise  # Re-raise to trigger panic



def plot_cat_success_percentage_violin_paper(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, plot_config: Dict[str, Any]) -> None:
    """
    Plot CAT success percentage violin plot for paper publication.
    
    This creates a violin plot showing the distribution of CAT success percentages
    for each CAT ratio, using cutoff data to avoid edge effects.
    """
    try:
        individual_results = data['individual_results']
        
        # Extract parameter values and results
        param_values = []
        results = []
        
        for result in individual_results:
            param_value = extract_parameter_value(result, param_name)
            param_values.append(param_value)
            results.append(result)
        
        # Apply cutoff to the data to match tx_count_cutoff processing
        from plot_utils_cutoff import apply_cutoff_to_percentage_data
        cutoff_data = apply_cutoff_to_percentage_data(data, plot_config)
        
        # Collect percentage data for each simulation
        violin_data = []
        labels = []
        
        for i, (param_value, result) in enumerate(zip(param_values, cutoff_data['individual_results'])):
            # Get CAT success and failure data with cutoff applied
            cat_success_data = result.get('chain_1_cat_success', [])
            cat_failure_data = result.get('chain_1_cat_failure', [])
            
            if not cat_success_data and not cat_failure_data:
                continue
            
            # Calculate percentage over time using point-in-time calculations
            percentages = []
            
            # Convert to height->count mapping
            cat_success_by_height = {entry[0]: entry[1] for entry in cat_success_data}
            cat_failure_by_height = {entry[0]: entry[1] for entry in cat_failure_data}
            
            # Get all unique heights
            all_heights = set()
            for height, _ in cat_success_data:
                all_heights.add(height)
            for height, _ in cat_failure_data:
                all_heights.add(height)
            
            # Calculate percentage at each height
            for height in sorted(all_heights):
                success_at_height = cat_success_by_height.get(height, 0)
                failure_at_height = cat_failure_by_height.get(height, 0)
                
                # Calculate percentage of success vs total (success + failure)
                total = success_at_height + failure_at_height
                if total > 0:
                    percentage = (success_at_height / total) * 100
                    percentages.append(percentage)
            
            # Discard the first 20% of data points for violin plot
            if percentages:
                num_points = len(percentages)
                discard_count = int(num_points * 0.2)  # 20% of data points
                filtered_percentages = percentages[discard_count:]
                
                if filtered_percentages:  # Only add if we have data after filtering
                    violin_data.append(filtered_percentages)
                    labels.append(f'{param_value:.3f}')
        
        if not violin_data:
            print("Warning: No data available for violin plot")
            return
        
        # Save violin plot data to data/paper/ folder
        paper_data_dir = f'{results_dir}/data/paper'
        os.makedirs(paper_data_dir, exist_ok=True)
        
        # Create data structure for saving
        violin_plot_data = {
            'parameter_name': param_name,
            'sweep_type': sweep_type,
            'num_simulations': len(violin_data),
            'num_runs': len(violin_data[0]) if violin_data else 0,
            'data': []
        }
        
        for i, (percentages, label) in enumerate(zip(violin_data, labels)):
            violin_plot_data['data'].append({
                'simulation_index': i,
                'parameter_value': float(label),
                'final_percentages': percentages,
                'mean_percentage': np.mean(percentages),
                'std_percentage': np.std(percentages),
                'min_percentage': np.min(percentages),
                'max_percentage': np.max(percentages)
            })
        
        # Save the data
        violin_data_file = f'{paper_data_dir}/cat_success_percentage_violin_paper.json'
        with open(violin_data_file, 'w') as f:
            json.dump(violin_plot_data, f, indent=2)
        
        # print(f"Saved violin plot data to: {violin_data_file}")
        
        # Create violin plot
        plt.figure(figsize=(10, 6))
        
        # Create violin plot
        violin_parts = plt.violinplot(violin_data, positions=range(len(violin_data)), showmeans=True)
        
        # Customize violin plot appearance
        violin_parts['cmeans'].set_color('red')
        violin_parts['cmeans'].set_linewidth(2)
        violin_parts['cbars'].set_color('black')
        violin_parts['cmins'].set_color('black')
        violin_parts['cmaxes'].set_color('black')
        
        # Set x-axis labels
        plt.xticks(range(len(violin_data)), labels)
        
        # Customize plot
        plt.xlabel('CAT Ratio')
        plt.ylabel('CAT Success Percentage (%)')
        plt.title(f'CAT Success Percentage Distribution by CAT Ratio - {create_sweep_title(param_name, sweep_type)}')
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        
        # Create the paper directory and plot
        paper_dir = f'{results_dir}/figs/paper'
        os.makedirs(paper_dir, exist_ok=True)
        plt.savefig(f'{paper_dir}/cat_success_percentage_violin_paper.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
        # print(f"Generated violin plot: cat_success_percentage_violin_paper.png")
        
    except Exception as e:
        print(f"Error generating violin plot: {e}")
        import traceback
        traceback.print_exc()


def plot_cat_success_percentage_violin(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, plot_config: Dict[str, Any]) -> None:
    """
    Plot CAT success percentage violin plot for paper publication.
    
    This creates a violin plot showing the distribution of CAT success percentages
    for each CAT ratio, using the final values from each run.
    """
    try:
        individual_results = data['individual_results']
        
        # Extract parameter values and results
        param_values = []
        results = []
        
        for result in individual_results:
            param_value = extract_parameter_value(result, param_name)
            param_values.append(param_value)
            results.append(result)
        
        # Load metadata to get number of runs
        metadata_path = f'{results_dir}/data/metadata.json'
        if not os.path.exists(metadata_path):
            print(f"Warning: No metadata found at {metadata_path}")
            return
        
        with open(metadata_path, 'r') as f:
            metadata = json.load(f)
        
        num_runs = metadata['num_runs']
        num_simulations = len(param_values)
        
        # Collect percentage data for each simulation
        violin_data = []
        labels = []
        
        for sim_index in range(num_simulations):
            param_value = param_values[sim_index]
            sim_dir = f'{results_dir}/data/sim_{sim_index}'
            
            # Find all run directories for this simulation
            if not os.path.exists(sim_dir):
                continue
                
            run_dirs = [d for d in os.listdir(sim_dir) if d.startswith('run_') and d != 'run_average']
            
            # Calculate final CAT success percentage for each run
            final_percentages = []
            
            for run_dir in run_dirs:
                run_data_dir = f'{sim_dir}/{run_dir}/data'
                
                if not os.path.exists(run_data_dir):
                    continue
                
                # Load CAT success and failure data
                cat_success_file = f'{run_data_dir}/cat_success_transactions_chain_1.json'
                cat_failure_file = f'{run_data_dir}/cat_failure_transactions_chain_1.json'
                
                # Load success data
                cat_success_data = []
                if os.path.exists(cat_success_file):
                    with open(cat_success_file, 'r') as f:
                        success_data = json.load(f)
                        if 'chain_1_cat_success' in success_data:
                            cat_success_data = [(entry['height'], entry['count']) for entry in success_data['chain_1_cat_success']]
                
                # Load failure data
                cat_failure_data = []
                if os.path.exists(cat_failure_file):
                    with open(cat_failure_file, 'r') as f:
                        failure_data = json.load(f)
                        if 'chain_1_cat_failure' in failure_data:
                            cat_failure_data = [(entry['height'], entry['count']) for entry in failure_data['chain_1_cat_failure']]
                
                if not cat_success_data and not cat_failure_data:
                    continue
                
                # Calculate percentage over time using point-in-time calculations
                percentages = []
                
                # Convert to height->count mapping
                cat_success_by_height = {entry[0]: entry[1] for entry in cat_success_data}
                cat_failure_by_height = {entry[0]: entry[1] for entry in cat_failure_data}
                
                # Get all unique heights
                all_heights = set()
                for height, _ in cat_success_data:
                    all_heights.add(height)
                for height, _ in cat_failure_data:
                    all_heights.add(height)
                
                # Calculate percentage at each height
                for height in sorted(all_heights):
                    success_at_height = cat_success_by_height.get(height, 0)
                    failure_at_height = cat_failure_by_height.get(height, 0)
                    
                    # Calculate percentage of success vs total (success + failure)
                    total = success_at_height + failure_at_height
                    if total > 0:
                        percentage = (success_at_height / total) * 100
                        percentages.append(percentage)
                
                # Get the final percentage (last value in the vector)
                if percentages:
                    final_percentage = percentages[-1]
                    final_percentages.append(final_percentage)
            
            # Add the final percentages for this simulation to violin data
            if final_percentages:
                violin_data.append(final_percentages)
                labels.append(f'{param_value:.3f}')
    
        if not violin_data:
            print("Warning: No data available for violin plot")
            return
        
        # Save violin plot data to data/paper/ folder
        paper_data_dir = f'{results_dir}/data/paper'
        os.makedirs(paper_data_dir, exist_ok=True)
        
        # Create data structure for saving
        violin_plot_data = {
            'parameter_name': param_name,
            'sweep_type': sweep_type,
            'num_simulations': len(violin_data),
            'num_runs': num_runs,
            'data': []
        }
        
        for i, (percentages, label) in enumerate(zip(violin_data, labels)):
            violin_plot_data['data'].append({
                'simulation_index': i,
                'parameter_value': float(label),
                'final_percentages': percentages,
                'mean_percentage': np.mean(percentages),
                'std_percentage': np.std(percentages),
                'min_percentage': np.min(percentages),
                'max_percentage': np.max(percentages)
            })
        
        # Save the data
        violin_data_file = f'{paper_data_dir}/cat_success_percentage_violin.json'
        with open(violin_data_file, 'w') as f:
            json.dump(violin_plot_data, f, indent=2)
        
        # print(f"Saved violin plot data to: {violin_data_file}")
        
        # Create violin plot
        violin_parts = plt.violinplot(violin_data, positions=range(len(violin_data)), showmeans=True)
        
        # Customize violin plot appearance
        violin_parts['cmeans'].set_color('red')
        violin_parts['cmeans'].set_linewidth(2)
        violin_parts['cbars'].set_color('black')
        violin_parts['cmins'].set_color('black')
        violin_parts['cmaxes'].set_color('black')
        
        # Set x-axis labels
        plt.xticks(range(len(violin_data)), labels)
        
        # Customize plot
        plt.xlabel('CAT Ratio')
        plt.ylabel('CAT Success Percentage (%)')
        plt.title(f'CAT Success Percentage Distribution by CAT Ratio - {create_sweep_title(param_name, sweep_type)}')
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        
        # Create the paper directory and plot
        paper_dir = f'{results_dir}/figs/paper'
        os.makedirs(paper_dir, exist_ok=True)
        plt.savefig(f'{paper_dir}/cat_success_percentage_violin.png',
                    dpi=300, bbox_inches='tight')
        plt.close()
        
        # print(f"Generated violin plot: cat_success_percentage_violin.png")
        
    except Exception as e:
        print(f"Error generating violin plot: {e}")
        import traceback
        traceback.print_exc()


def plot_tx_pending_cat_postponed_violin(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, plot_config: Dict[str, Any]) -> None:
    """
    Plot CAT pending postponed transactions violin plot for paper publication.
    
    This function creates a violin plot showing the distribution of CAT pending postponed
    transaction counts across different CAT ratios, using data from individual runs.
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping CAT pending postponed violin plot")
            return
        
        # Load data from the sweep results (same as overlay plots)
        violin_data = []
        labels = []
        
        for sim_index, result in enumerate(individual_results):
            param_value = result[param_name]
            labels.append(f'{param_value:.3f}')
            
            # Get the time series data from the sweep results (same as overlay plots)
            cat_pending_postponed_data = result.get('chain_1_cat_pending_postponed', [])
            
            print(f"DEBUG: Simulation {sim_index} (CAT Ratio {param_value:.3f}):")
            print(f"  - Found {len(cat_pending_postponed_data)} data points")
            if cat_pending_postponed_data:
                print(f"  - First 5 data points: {cat_pending_postponed_data[:5]}")
                print(f"  - Last 5 data points: {cat_pending_postponed_data[-5:]}")
                
                # Extract ALL count values from the time series
                count_values = [entry[1] for entry in cat_pending_postponed_data]  # (height, count) tuples
                print(f"  - Extracted {len(count_values)} count values")
                print(f"  - Count values range: min={min(count_values)}, max={max(count_values)}, mean={np.mean(count_values):.2f}")
                
                # Cut off the first 30% to avoid initialization effects
                cutoff_index = int(len(count_values) * 0.3)
                trimmed_values = count_values[cutoff_index:]
                print(f"  - After trimming first 30%: {len(trimmed_values)} values remain")
                print(f"  - Trimmed values range: min={min(trimmed_values)}, max={max(trimmed_values)}, mean={np.mean(trimmed_values):.2f}")
                
                violin_data.append(trimmed_values)  # Trimmed values for this simulation
            else:
                print(f"  - No data available")
                violin_data.append([0])  # No data available
        
        if not violin_data or all(len(values) == 0 for values in violin_data):
            print("Warning: No CAT pending postponed data found, skipping violin plot")
            return
        
        # Debug: Print the data before creating violin plot
        print("=== CAT Pending Postponed Violin Data ===")
        for i, (values, label) in enumerate(zip(violin_data, labels)):
            print(f"Simulation {i} (CAT Ratio {label}): {len(values)} values, min={min(values)}, max={max(values)}, mean={np.mean(values):.2f}")
            if len(values) > 10:
                print(f"  - Sample values: {values[:5]} ... {values[-5:]}")
            else:
                print(f"  - All values: {values}")
        print("========================================")
        
        # Create violin plot
        plt.figure(figsize=(10, 6))
        
        # Create data structure for saving
        violin_plot_data = {
            'parameter_name': param_name,
            'sweep_type': sweep_type,
            'num_simulations': len(violin_data),
            'data': []
        }
        
        for i, (values, label) in enumerate(zip(violin_data, labels)):
            violin_plot_data['data'].append({
                'simulation_index': i,
                'parameter_value': float(label),
                'final_values': values,
                'mean_value': np.mean(values),
                'std_value': np.std(values),
                'min_value': np.min(values),
                'max_value': np.max(values)
            })
        
        # Save the data
        paper_data_dir = f'{results_dir}/data/paper'
        os.makedirs(paper_data_dir, exist_ok=True)
        violin_data_file = f'{paper_data_dir}/tx_pending_cat_postponed_violin.json'
        with open(violin_data_file, 'w') as f:
            json.dump(violin_plot_data, f, indent=2)
        
        # Create violin plot
        violin_parts = plt.violinplot(violin_data, positions=range(len(violin_data)), showmeans=True)
        
        # Customize violin plot appearance
        violin_parts['cmeans'].set_color('red')
        violin_parts['cmeans'].set_linewidth(2)
        violin_parts['cbars'].set_color('black')
        violin_parts['cmins'].set_color('black')
        violin_parts['cmaxes'].set_color('black')
        
        # Set x-axis labels
        plt.xticks(range(len(violin_data)), labels)
        
        # Customize plot
        plt.xlabel('CAT Ratio')
        plt.ylabel('CAT Pending Postponed Transactions')
        plt.title(f'CAT Pending Postponed Transactions Distribution by CAT Ratio - {create_sweep_title(param_name, sweep_type)}')
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        
        # Create the paper directory and plot
        paper_dir = f'{results_dir}/figs/paper'
        os.makedirs(paper_dir, exist_ok=True)
        plt.savefig(f'{paper_dir}/tx_pending_cat_postponed_violin.png',
                    dpi=300, bbox_inches='tight')
        plt.close()
        
        # print(f"Generated violin plot: tx_pending_cat_postponed_violin.png")
        
    except Exception as e:
        print(f"Error generating CAT pending postponed violin plot: {e}")
        import traceback
        traceback.print_exc()


def plot_tx_pending_cat_resolving_violin(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, plot_config: Dict[str, Any]) -> None:
    """
    Plot CAT pending resolving transactions violin plot for paper publication.
    
    This function creates a violin plot showing the distribution of CAT pending resolving
    transaction counts across different CAT ratios, using data from individual runs.
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping CAT pending resolving violin plot")
            return
        
        # Load data from the sweep results (same as overlay plots)
        violin_data = []
        labels = []
        
        for sim_index, result in enumerate(individual_results):
            param_value = result[param_name]
            labels.append(f'{param_value:.3f}')
            
            # Get the time series data from the sweep results (same as overlay plots)
            cat_pending_resolving_data = result.get('chain_1_cat_pending_resolving', [])
            
            print(f"DEBUG: Simulation {sim_index} (CAT Ratio {param_value:.3f}):")
            print(f"  - Found {len(cat_pending_resolving_data)} data points")
            if cat_pending_resolving_data:
                print(f"  - First 5 data points: {cat_pending_resolving_data[:5]}")
                print(f"  - Last 5 data points: {cat_pending_resolving_data[-5:]}")
                
                # Extract ALL count values from the time series
                count_values = [entry[1] for entry in cat_pending_resolving_data]  # (height, count) tuples
                print(f"  - Extracted {len(count_values)} count values")
                print(f"  - Count values range: min={min(count_values)}, max={max(count_values)}, mean={np.mean(count_values):.2f}")
                
                # Cut off the first 30% to avoid initialization effects
                cutoff_index = int(len(count_values) * 0.3)
                trimmed_values = count_values[cutoff_index:]
                print(f"  - After trimming first 30%: {len(trimmed_values)} values remain")
                print(f"  - Trimmed values range: min={min(trimmed_values)}, max={max(trimmed_values)}, mean={np.mean(trimmed_values):.2f}")
                
                violin_data.append(trimmed_values)  # Trimmed values for this simulation
            else:
                print(f"  - No data available")
                violin_data.append([0])  # No data available
        
        if not violin_data or all(len(values) == 0 for values in violin_data):
            print("Warning: No CAT pending resolving data found, skipping violin plot")
            return
        
        # Debug: Print the data before creating violin plot
        print("=== CAT Pending Resolving Violin Data ===")
        for i, (values, label) in enumerate(zip(violin_data, labels)):
            print(f"Simulation {i} (CAT Ratio {label}): {len(values)} values, min={min(values)}, max={max(values)}, mean={np.mean(values):.2f}")
            if len(values) > 10:
                print(f"  - Sample values: {values[:5]} ... {values[-5:]}")
            else:
                print(f"  - All values: {values}")
        print("========================================")
        
        # Create violin plot
        plt.figure(figsize=(10, 6))
        
        # Create data structure for saving
        violin_plot_data = {
            'parameter_name': param_name,
            'sweep_type': sweep_type,
            'num_simulations': len(violin_data),
            'data': []
        }
        
        for i, (values, label) in enumerate(zip(violin_data, labels)):
            violin_plot_data['data'].append({
                'simulation_index': i,
                'parameter_value': float(label),
                'final_values': values,
                'mean_value': np.mean(values),
                'std_value': np.std(values),
                'min_value': np.min(values),
                'max_value': np.max(values)
            })
        
        # Save the data
        paper_data_dir = f'{results_dir}/data/paper'
        os.makedirs(paper_data_dir, exist_ok=True)
        violin_data_file = f'{paper_data_dir}/tx_pending_cat_resolving_violin.json'
        with open(violin_data_file, 'w') as f:
            json.dump(violin_plot_data, f, indent=2)
        
        # Create violin plot
        violin_parts = plt.violinplot(violin_data, positions=range(len(violin_data)), showmeans=True)
        
        # Customize violin plot appearance
        violin_parts['cmeans'].set_color('red')
        violin_parts['cmeans'].set_linewidth(2)
        violin_parts['cbars'].set_color('black')
        violin_parts['cmins'].set_color('black')
        violin_parts['cmaxes'].set_color('black')
        
        # Set x-axis labels
        plt.xticks(range(len(violin_data)), labels)
        
        # Customize plot
        plt.xlabel('CAT Ratio')
        plt.ylabel('CAT Pending Resolving Transactions')
        plt.title(f'CAT Pending Resolving Transactions Distribution by CAT Ratio - {create_sweep_title(param_name, sweep_type)}')
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        
        # Create the paper directory and plot
        paper_dir = f'{results_dir}/figs/paper'
        os.makedirs(paper_dir, exist_ok=True)
        plt.savefig(f'{paper_dir}/tx_pending_cat_resolving_violin.png',
                    dpi=300, bbox_inches='tight')
        plt.close()
        
        # print(f"Generated violin plot: tx_pending_cat_resolving_violin.png")
        
    except Exception as e:
        print(f"Error generating CAT pending resolving violin plot: {e}")
        import traceback
        traceback.print_exc()


def plot_tx_pending_regular_violin(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, plot_config: Dict[str, Any]) -> None:
    """
    Plot regular pending transactions violin plot for paper publication.
    
    This function creates a violin plot showing the distribution of regular pending
    transaction counts across different CAT ratios, using data from individual runs.
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping regular pending violin plot")
            return
        
        # Load individual run data for each simulation
        violin_data = []
        labels = []
        
        for sim_index, result in enumerate(individual_results):
            param_value = result[param_name]
            labels.append(f'{param_value:.3f}')
            
            # Load individual run data for this simulation
            results_dir_name = results_dir.split('/')[-1]  # Extract 'sim_sweep_cat_ratio'
            sim_data_dir = f'../../../results/{results_dir_name}/data/sim_{sim_index}'
            
            # Get all run directories for this simulation
            run_dirs = []
            for item in os.listdir(sim_data_dir):
                if item.startswith('run_') and os.path.isdir(os.path.join(sim_data_dir, item)):
                    run_dirs.append(item)
            run_dirs.sort()  # Ensure consistent ordering
            
            # Collect regular pending data from each run
            regular_pending_values = []
            
            for run_dir in run_dirs:
                run_data_file = f'{sim_data_dir}/{run_dir}/data/regular_pending_transactions_chain_1.json'
                
                if os.path.exists(run_data_file):
                    with open(run_data_file, 'r') as f:
                        run_data = json.load(f)
                    
                    if 'chain_1_regular_pending' in run_data:
                        # Get the final value (last entry) from the time series
                        time_series = run_data['chain_1_regular_pending']
                        if time_series:
                            # Get the last value (final count)
                            final_value = time_series[-1]['count']
                            regular_pending_values.append(final_value)
            
            violin_data.append(regular_pending_values)
        
        if not violin_data or all(len(values) == 0 for values in violin_data):
            print("Warning: No regular pending data found, skipping violin plot")
            return
        
        # Create violin plot
        plt.figure(figsize=(10, 6))
        
        # Create data structure for saving
        violin_plot_data = {
            'parameter_name': param_name,
            'sweep_type': sweep_type,
            'num_simulations': len(violin_data),
            'data': []
        }
        
        for i, (values, label) in enumerate(zip(violin_data, labels)):
            violin_plot_data['data'].append({
                'simulation_index': i,
                'parameter_value': float(label),
                'final_values': values,
                'mean_value': np.mean(values),
                'std_value': np.std(values),
                'min_value': np.min(values),
                'max_value': np.max(values)
            })
        
        # Save the data
        paper_data_dir = f'{results_dir}/data/paper'
        os.makedirs(paper_data_dir, exist_ok=True)
        violin_data_file = f'{paper_data_dir}/tx_pending_regular_violin.json'
        with open(violin_data_file, 'w') as f:
            json.dump(violin_plot_data, f, indent=2)
        
        # Create violin plot
        violin_parts = plt.violinplot(violin_data, positions=range(len(violin_data)), showmeans=True)
        
        # Customize violin plot appearance
        violin_parts['cmeans'].set_color('red')
        violin_parts['cmeans'].set_linewidth(2)
        violin_parts['cbars'].set_color('black')
        violin_parts['cmins'].set_color('black')
        violin_parts['cmaxes'].set_color('black')
        
        # Set x-axis labels
        plt.xticks(range(len(violin_data)), labels)
        
        # Customize plot
        plt.xlabel('CAT Ratio')
        plt.ylabel('Regular Pending Transactions')
        plt.title(f'Regular Pending Transactions Distribution by CAT Ratio - {create_sweep_title(param_name, sweep_type)}')
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        
        # Create the paper directory and plot
        paper_dir = f'{results_dir}/figs/paper'
        os.makedirs(paper_dir, exist_ok=True)
        plt.savefig(f'{paper_dir}/tx_pending_regular_violin.png',
                    dpi=300, bbox_inches='tight')
        plt.close()
        
        # print(f"Generated violin plot: tx_pending_regular_violin.png")
        
    except Exception as e:
        print(f"Error generating regular pending violin plot: {e}")
        import traceback
        traceback.print_exc()


def main():
    """Main function to generate paper-specific plots for CAT ratio sweep simulation."""
    # Configuration for this specific sweep
    param_name = 'cat_ratio'
    results_dir = '../../../results/sim_sweep_cat_ratio'
    sweep_type = 'CAT Ratio'
    
    # Load sweep data directly from run_average folders
    try:
        # Import the data loading function from plot_utils (same as cat_ratio)
        from plot_utils import load_sweep_data_from_run_average
        
        # Load data directly from run_average folders
        results_dir_name = results_dir.split('/')[-1]  # Extract 'sim_sweep_cat_ratio'
        data = load_sweep_data_from_run_average(results_dir_name, '../../../results')
        
        # Check if we have any data to plot
        if not data.get('individual_results'):
            print(f"No data found for {sweep_type} simulation. Skipping paper plot generation.")
            return
        

        # Load plot configuration for cutoff settings
        from plot_utils import load_plot_config
        plot_config = load_plot_config(results_dir)
        
        # Generate paper-specific plots
        plot_cat_success_percentage_with_overlay(data, param_name, results_dir, sweep_type)
        plot_cat_success_percentage_violin_paper(data, param_name, results_dir, sweep_type, plot_config)
        plot_cat_success_percentage_violin(data, param_name, results_dir, sweep_type, plot_config)
        print("DEBUG: About to run postponed violin plot...")
        plot_tx_pending_cat_postponed_violin(data, param_name, results_dir, sweep_type, plot_config)
        print("DEBUG: About to run resolving violin plot...")
        plot_tx_pending_cat_resolving_violin(data, param_name, results_dir, sweep_type, plot_config)
        plot_tx_pending_regular_violin(data, param_name, results_dir, sweep_type, plot_config)
        
    except Exception as e:
        print(f"Error in main: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main() 