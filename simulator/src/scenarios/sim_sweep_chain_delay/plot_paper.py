#!/usr/bin/env python3
"""
Paper-specific plotting script for Chain Delay Sweep Simulation

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
        plt.figure(figsize=(12, 8))
        
        # Create color gradient using coolwarm colormap
        colors = plt.cm.get_cmap('coolwarm')(np.linspace(0, 1, len(individual_results)))
        
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
        
        # Plot individual run curves (black, thin lines)
        for run_data in individual_runs:
            cat_success_data = run_data.get('chain_1_cat_success', [])
            cat_failure_data = run_data.get('chain_1_cat_failure', [])
            
            if not cat_success_data and not cat_failure_data:
                continue
            
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
                
                # Plot individual run curve (black, thin)
                plt.plot(heights, percentages, color='black', alpha=0.3, 
                        linewidth=0.5, linestyle='-')
                
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
        
        # Create a simple test file in figs directory
        test_file = f'{results_dir}/figs/paper__test.txt'
        print(f"DEBUG: Creating test file at {test_file}")
        with open(test_file, 'w') as f:
            f.write("Paper plot script executed successfully!")
        print(f"DEBUG: Test file created successfully")
        
        # Also try to create the paper directory and plot
        paper_dir = f'{results_dir}/figs/paper'
        print(f"DEBUG: Creating paper directory at {paper_dir}")
        os.makedirs(paper_dir, exist_ok=True)
        print(f"DEBUG: Paper directory created successfully")
        plt.savefig(f'{paper_dir}/{filename}', 
                   dpi=300, bbox_inches='tight')
        print(f"DEBUG: Plot saved successfully to {paper_dir}/{filename}")
        plt.close()
        
        print(f"Generated paper plot: {filename}")
        
    except Exception as e:
        print(f"Error generating paper plot: {e}")
        import traceback
        traceback.print_exc()
        raise  # Re-raise to trigger panic

def main():
    """Main function to generate paper-specific plots for chain delay sweep simulation."""
    # Configuration for this specific sweep
    param_name = 'chain_delay'
    results_dir = '../../../results/sim_sweep_chain_delay'
    sweep_type = 'Chain Delay'
    
    # Load sweep data directly from run_average folders
    try:
        # Import the data loading function from plot_utils (same as cat_rate)
        from plot_utils import load_sweep_data_from_run_average
        
        # Load data directly from run_average folders
        results_dir_name = results_dir.split('/')[-1]  # Extract 'sim_sweep_chain_delay'
        data = load_sweep_data_from_run_average(results_dir_name, '../../../results')
        
        # Check if we have any data to plot
        if not data.get('individual_results'):
            print(f"No data found for {sweep_type} simulation. Skipping paper plot generation.")
            return
        
        # Generate paper-specific plots
        plot_cat_success_percentage_with_overlay(data, param_name, results_dir, sweep_type)
        
    except Exception as e:
        print(f"Error in main: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main() 