#!/usr/bin/env python3

import os
import sys
import json
import matplotlib.pyplot as plt
import subprocess

# Add the current directory to the Python path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from plot_account_selection import plot_account_selection
from plot_miscellaneous import (
    plot_pending_transactions,
    plot_success_transactions,
    plot_failure_transactions,
    plot_parameters,
)

# ------------------------------------------------------------------------------------------------
# Locked Keys Plotting Functions
# ------------------------------------------------------------------------------------------------

def plot_locked_keys():
    """
    Plot locked keys data from both chains.
    """
    try:
        # Load locked keys data from chain 1
        with open('simulator/results/sim_simple_repeated/data/run_average/locked_keys_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        chain_1_blocks = [entry['height'] for entry in chain_1_data['chain_1_locked_keys']]
        chain_1_locked_keys = [entry['count'] for entry in chain_1_data['chain_1_locked_keys']]
        
        # Load locked keys data from chain 2
        with open('simulator/results/sim_simple_repeated/data/run_average/locked_keys_chain_2.json', 'r') as f:
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
        plt.savefig('simulator/results/sim_simple_repeated/figs/locked_keys.png', dpi=300, bbox_inches='tight')
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
        with open('simulator/results/sim_simple_repeated/data/run_average/locked_keys_chain_1.json', 'r') as f:
            locked_keys_data = json.load(f)
        blocks = [entry['height'] for entry in locked_keys_data['chain_1_locked_keys']]
        locked_keys = [entry['count'] for entry in locked_keys_data['chain_1_locked_keys']]
        
        # Load CAT pending transactions data
        try:
            with open('simulator/results/sim_simple_repeated/data/run_average/cat_pending_transactions_chain_1.json', 'r') as f:
                cat_pending_data = json.load(f)
            cat_pending_transactions = [entry['count'] for entry in cat_pending_data['chain_1_cat_pending']]
        except (FileNotFoundError, json.JSONDecodeError, KeyError):
            cat_pending_transactions = [0] * len(blocks)
        
        # Load regular pending transactions data
        try:
            with open('simulator/results/sim_simple_repeated/data/run_average/regular_pending_transactions_chain_1.json', 'r') as f:
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
        plt.savefig('simulator/results/sim_simple_repeated/figs/locked_keys_vs_pending.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing data for comparison plot: {e}")
        return

def main():
    # Run the averaging script first
    result = subprocess.run([sys.executable, 'average_runs.py'], 
                          capture_output=True, text=True, cwd=os.path.dirname(__file__))
    
    if result.returncode != 0:
        print(f"Error: Averaging failed: {result.stderr}")
        return False
    
    os.makedirs('simulator/results/sim_simple_repeated/figs', exist_ok=True)
    
    # Plot account selection distributions
    plot_account_selection()
    # Plot pending transactions
    plot_pending_transactions()
    # Plot success transactions
    plot_success_transactions()
    # Plot failure transactions
    plot_failure_transactions()
    # Plot simulation parameters
    plot_parameters()
    # Plot locked keys data
    plot_locked_keys()
    plot_locked_keys_with_pending()

if __name__ == "__main__":
    main() 