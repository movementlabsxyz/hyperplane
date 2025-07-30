#!/usr/bin/env python3
"""
Paper-specific plotting script for CAT Rate Sweep Simulation

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

def plot_cat_success_percentage_paper(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str) -> None:
    """
    Plot CAT success percentage for paper publication.
    
    This creates a clean plot showing CAT success percentage vs CAT rate parameter,
    designed for paper publication without a title.
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
        
        # Create color gradient
        colors = create_color_gradient(len(param_values))
        
        # Create the plot
        plt.figure(figsize=(10, 6))
        
        # Track maximum height for xlim
        max_height = 0
        
        # Plot each parameter value
        for i, (param_value, result) in enumerate(zip(param_values, results)):
            # Get CAT success and failure data
            cat_success_data = result.get('chain_1_cat_success', [])
            cat_failure_data = result.get('chain_1_cat_failure', [])
            
            if not cat_success_data and not cat_failure_data:
                continue
            
            # Calculate percentage over time using point-in-time calculations
            heights = []
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
                    heights.append(height)
                    percentages.append(percentage)
            
            if not heights:
                continue
            
            # Trim the last 10% of data to avoid edge effects
            if len(heights) > 10:
                trim_index = int(len(heights) * 0.9)
                heights = heights[:trim_index]
                percentages = percentages[:trim_index]
            
            if not heights:
                continue
            
            # Update maximum height
            if heights:
                max_height = max(max_height, max(heights))
            
            # Plot with color based on parameter
            label = create_parameter_label(param_name, param_value)
            plt.plot(heights, percentages, color=colors[i], alpha=0.7, 
                    label=label, linewidth=1.5)
        
        # Set x-axis limits before finalizing the plot
        plt.xlim(left=0, right=max_height)
        
        # No title as requested
        plt.xlabel('Block Height')
        plt.ylabel('CAT Success Percentage (%)')
        plt.grid(True, alpha=0.3)
        plt.legend(loc="upper right")
        plt.tight_layout()
        
        # Create paper directory within figs and save plot
        paper_dir = f'{results_dir}/figs/paper'
        os.makedirs(paper_dir, exist_ok=True)
        plt.savefig(f'{paper_dir}/tx_success_cat_percentage.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Error creating CAT success percentage paper plot: {e}")
        import traceback
        traceback.print_exc()
        raise  # Re-raise to trigger panic

def main():
    """Main function for paper-specific plots for cat rate sweep simulation."""
    try:
        # Configuration for this specific sweep
        param_name = 'cat_ratio'
        results_dir = '../../../results/sim_sweep_cat_rate'
        sweep_type = 'CAT Rate'
        
        # Load sweep data directly from run_average folders (same as other plots)
        results_dir_name = results_dir.split('/')[-1]  # Extract 'sim_sweep_cat_rate'
        from plot_utils import load_sweep_data_from_run_average
        data = load_sweep_data_from_run_average(results_dir_name, '../../../results')
        
        # Check if we have any data to plot
        if not data.get('individual_results'):
            print(f"No data found for {sweep_type} simulation. Skipping paper plot generation.")
            return
        
        # Generate paper plots
        print("Generating paper plots...")
        plot_cat_success_percentage_paper(data, param_name, results_dir, sweep_type)
        
        print("Paper plots generated successfully!")
        
    except Exception as e:
        print(f"Error generating paper plots: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main() 