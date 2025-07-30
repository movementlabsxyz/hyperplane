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

def plot_cat_success_percentage_with_overlay(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot CAT success percentage with both average and individual curves overlaid.
    
    This creates a plot showing:
    1. Individual curves for each parameter value (lighter colors)
    2. Average curve for each parameter value (darker colors, same color family)
    3. Legend showing the parameter values
    """
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            print(f"Warning: No individual results found, skipping CAT success percentage plot")
            return
        
        # Create figure
        plt.figure(figsize=(12, 8))
        
        # Create color gradient for parameter values
        colors = create_color_gradient(len(individual_results))
        
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
                
                # Calculate success percentage at each height
                heights = []
                percentages = []
                
                for height in sorted(combined_data.keys()):
                    total_cat = combined_data[height]
                    success_count = next((count for h, count in cat_success_data if h == height), 0)
                    
                    if total_cat > 0:
                        percentage = (success_count / total_cat) * 100
                        heights.append(height)
                        percentages.append(percentage)
                
                if heights:
                    # Trim the last 10% of data to avoid edge effects
                    trim_idx = int(len(heights) * 0.9)
                    heights = heights[:trim_idx]
                    percentages = percentages[:trim_idx]
                    
                    # Plot individual curve (lighter)
                    plt.plot(heights, percentages, color=light_color, alpha=0.5, linewidth=0.8)
                    
                    # Track all heights and percentages for averaging
                    all_heights.update(heights)
                    all_percentages.append((heights, percentages))
                    
                    # Update maximum height
                    if heights:
                        max_height = max(max_height, max(heights))
            
            # Calculate and plot average curve (darker)
            if all_percentages:
                # Find common height range
                min_height = min(all_heights)
                max_height_common = max(all_heights)
                
                # Create height bins for averaging
                height_bins = list(range(min_height, max_height_common + 1))
                avg_percentages = []
                
                for height in height_bins:
                    percentages_at_height = []
                    for heights, percentages in all_percentages:
                        if height in heights:
                            idx = heights.index(height)
                            percentages_at_height.append(percentages[idx])
                    
                    if percentages_at_height:
                        avg_percentages.append(np.mean(percentages_at_height))
                    else:
                        avg_percentages.append(0)
                
                # Plot average curve (darker, same color family)
                plt.plot(height_bins, avg_percentages, color=base_color, alpha=0.9, 
                        linewidth=2.5, label=create_parameter_label(param_name, param_value))
        
        # Set x-axis limits
        plt.xlim(left=0, right=max_height)
        
        # Create title and filename
        title = f'CAT Success Percentage by Height (Chain 1) - {create_sweep_title(param_name, sweep_type)}'
        filename = 'cat_success_percentage_paper.png'
        
        plt.title(title, fontsize=14)
        plt.xlabel('Block Height', fontsize=12)
        plt.ylabel('CAT Success Percentage (%)', fontsize=12)
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right", fontsize=10)
        plt.tight_layout()
        
        # Save plot
        os.makedirs(f'{results_dir}/figs', exist_ok=True)
        plt.savefig(f'{results_dir}/figs/{filename}', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
        print(f"Generated paper plot: {filename}")
        
    except Exception as e:
        print(f"Error generating paper plot: {e}")
        return

def main():
    """Main function to generate paper-specific plots for chain delay sweep simulation."""
    # Configuration for this specific sweep
    param_name = 'chain_delay'
    results_dir = 'simulator/results/sim_sweep_chain_delay'
    sweep_type = 'Chain Delay'
    
    # Load sweep data
    try:
        # Load the sweep data from the averaged results
        data_file = f'{results_dir}/data/sweep_results_averaged.json'
        if os.path.exists(data_file):
            with open(data_file, 'r') as f:
                data = json.load(f)
        else:
            print(f"Warning: {data_file} not found. Skipping paper plot generation.")
            return
        
        # Generate paper-specific plots
        plot_cat_success_percentage_with_overlay(data, param_name, results_dir, sweep_type)
        
    except Exception as e:
        print(f"Error in main: {e}")

if __name__ == "__main__":
    main() 