#!/usr/bin/env python3
"""
Plotting script for CAT Ratio with Constant CATs per Block Sweep Simulation

This script generates plots for the CAT ratio with constant CATs per block sweep using the generic
plotting utilities to eliminate code duplication.
"""

import sys
import os

# Add the scripts directory to the Python path to import plot_utils
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from plot_utils import generate_all_plots

def main():
    """Main function to generate plots for CAT ratio with constant CATs per block sweep simulation."""
    import sys
    import os
    
    # Check if debug mode is enabled
    debug_mode = os.environ.get('DEBUG_MODE', '0') == '1'
    
    if debug_mode:
        print("DEBUG: CAT ratio constant CATs per block plot_results.py main() called", flush=True)
        sys.stderr.write("STDERR DEBUG: CAT ratio constant CATs per block plot_results.py main() called\n")
        sys.stderr.flush()
    
    # Configuration for this specific sweep
    param_name = 'target_tpb'
    results_dir = 'simulator/results/sim_sweep_cat_ratio_constant_cats_per_block'
    sweep_type = 'CAT Ratio with Constant CATs per Block'
    
    if debug_mode:
        print(f"DEBUG: About to call generate_all_plots with results_dir={results_dir}, param_name={param_name}, sweep_type={sweep_type}", flush=True)
        sys.stderr.write(f"STDERR DEBUG: About to call generate_all_plots with results_dir={results_dir}, param_name={param_name}, sweep_type={sweep_type}\n")
        sys.stderr.flush()
    
    # Generate all plots using the generic utility
    # Data flow: run_average folders -> sweep_results_averaged.json -> plots
    generate_all_plots(results_dir, param_name, sweep_type)
    
    if debug_mode:
        print("DEBUG: generate_all_plots call completed", flush=True)
        sys.stderr.write("STDERR DEBUG: generate_all_plots call completed\n")
        sys.stderr.flush()

if __name__ == "__main__":
    main() 