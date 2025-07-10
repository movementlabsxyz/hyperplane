#!/usr/bin/env python3
"""
Generic Sweep Plot Generator

This script can generate plots for any sweep simulation by providing
the sweep configuration. It eliminates the need for individual plot_results.py
files in each sweep directory.
"""

import sys
import os
import argparse

# Add the current directory to the Python path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from plot_utils import run_sweep_plots

# Configuration for all sweep types
SWEEP_CONFIGS = {
    'cat_rate': {
        'sweep_name': 'sim_sweep_cat_rate',
        'param_name': 'cat_rate',
        'sweep_type': 'CAT Rate'
    },
    'cat_lifetime': {
        'sweep_name': 'sim_sweep_cat_lifetime',
        'param_name': 'cat_lifetime',
        'sweep_type': 'CAT Lifetime'
    },
    'cat_pending_dependencies': {
        'sweep_name': 'sim_sweep_cat_pending_dependencies',
        'param_name': 'allow_cat_pending_dependencies',
        'sweep_type': 'CAT Pending Dependencies'
    },
    'zipf': {
        'sweep_name': 'sim_sweep_zipf',
        'param_name': 'zipf_parameter',
        'sweep_type': 'Zipf Parameter'
    },
    'block_interval_constant_time_delay': {
        'sweep_name': 'sim_sweep_block_interval_constant_time_delay',
        'param_name': 'block_interval',
        'sweep_type': 'Block Interval (Constant Time Delay)'
    },
    'block_interval_constant_block_delay': {
        'sweep_name': 'sim_sweep_block_interval_constant_block_delay',
        'param_name': 'block_interval',
        'sweep_type': 'Block Interval (Constant Block Delay)'
    },
    'chain_delay': {
        'sweep_name': 'sim_sweep_chain_delay',
        'param_name': 'chain_delay',
        'sweep_type': 'Chain Delay'
    },
    'total_block_number': {
        'sweep_name': 'sim_sweep_total_block_number',
        'param_name': 'duration',
        'sweep_type': 'Total Block Number'
    }
}

def main():
    """Main function"""
    parser = argparse.ArgumentParser(description='Generate plots for sweep simulations')
    parser.add_argument('sweep_type', choices=list(SWEEP_CONFIGS.keys()), 
                       help='Type of sweep to plot')
    
    args = parser.parse_args()
    
    config = SWEEP_CONFIGS[args.sweep_type]
    
    print(f"Generating plots for {config['sweep_type']} sweep...")
    run_sweep_plots(config['sweep_name'], config['param_name'], config['sweep_type'])
    print(f"Plots generated successfully for {config['sweep_type']} sweep!")

if __name__ == "__main__":
    main() 