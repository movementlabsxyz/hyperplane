#!/usr/bin/env python3
"""
Paper-specific plotting script for Cat Ratio Constant Cats Per Block Sweep Simulation

This script generates plots specifically designed for paper publication,
including CAT success percentage violin plots.
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

# Check if debug mode is enabled
DEBUG_MODE = os.environ.get('DEBUG_MODE', '0') == '1'

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






def plot_cat_success_percentage_violin(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, plot_config: Dict[str, Any]) -> None:
    """
    Plot CAT success percentage violin plot for paper publication.
    
    This creates a violin plot showing the distribution of CAT success percentages
    for each target TPB value, using the final values from each run.
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
        
        print(f"Saved violin plot data to: {violin_data_file}")
        
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
        plt.xlabel('Target TPB')
        plt.ylabel('CAT Success Percentage (%)')
        plt.title(f'CAT Success Percentage Distribution by Target TPB - {create_sweep_title(param_name, sweep_type)}')
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        
        # Create the paper directory and plot
        paper_dir = f'{results_dir}/figs/paper'
        os.makedirs(paper_dir, exist_ok=True)
        plt.savefig(f'{paper_dir}/cat_success_percentage_violin.png',
                    dpi=300, bbox_inches='tight')
        plt.close()
        
        print(f"Generated violin plot: cat_success_percentage_violin.png")
        
    except Exception as e:
        print(f"Error generating violin plot: {e}")
        import traceback
        traceback.print_exc()


def main():
    """Main function to generate paper-specific plots for cat ratio constant cats per block sweep simulation."""
    # Configuration for this specific sweep
    param_name = 'target_tpb'
    results_dir = '../../../results/sim_sweep_tpb_constant_cats_per_block'
    sweep_type = 'CAT Ratio Constant Cats Per Block'
    
    # Load sweep data directly from run_average folders
    try:
        # Import the data loading function from plot_utils
        from plot_utils import load_sweep_data_from_run_average
        
        # Load data directly from run_average folders
        results_dir_name = results_dir.split('/')[-1]  # Extract 'sim_sweep_tpb_constant_cats_per_block'
        data = load_sweep_data_from_run_average(results_dir_name, '../../../results')
        
        # Check if we have any data to plot
        if not data.get('individual_results'):
            print(f"No data found for {sweep_type} simulation. Skipping paper plot generation.")
            return
        
        # Load plot configuration for cutoff settings
        from plot_utils import load_plot_config
        plot_config = load_plot_config(results_dir)
        
        # Generate paper-specific plots
        plot_cat_success_percentage_violin(data, param_name, results_dir, sweep_type, plot_config)
        
    except Exception as e:
        print(f"Error in main: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main() 