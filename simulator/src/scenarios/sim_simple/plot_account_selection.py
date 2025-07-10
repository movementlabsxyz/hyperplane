#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.stats import zipf

def plot_distribution(role, zipf_param, num_accounts):
    """
    Plots the account selection distribution with theoretical curve in three scales.
    
    Args:
        role (str): Either 'sender' or 'receiver'
        zipf_param (float): Zipf parameter for theoretical distribution (0 for uniform)
        num_accounts (int): Total number of accounts in the system
    
    The function creates three plots:
    1. Linear-linear scale
    2. Log-linear scale
    3. Log-log scale
    
    Each plot shows:
    - Actual distribution as scatter points
    - Theoretical distribution as a red dashed line
    - For zipf_param = 0, this is a uniform distribution
    - For zipf_param > 0, this is a Zipf distribution
    """
    # Load and extract data
    with open(f'simulator/results/sim_simple/data/sim_0/run_average/account_{role}_selection.json', 'r') as f:
        data = json.load(f)
        
    # Handle both old format (with 'account' and 'transactions' keys) and new format (direct key-value pairs)
    if isinstance(data, dict) and f'{role}_selection' in data:
        # Old format
        accounts = [entry['account'] for entry in data[f'{role}_selection']]
        counts = [entry['transactions'] for entry in data[f'{role}_selection']]
    else:
        # New format (direct key-value pairs)
        # Convert string keys to integers and sort them
        accounts = [int(account_id) for account_id in data.keys()]
        counts = list(data.values())
        # Sort by account ID to ensure proper plotting order
        sorted_pairs = sorted(zip(accounts, counts))
        accounts, counts = zip(*sorted_pairs)
    
    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(10, 15))
        
    # Calculate theoretical distribution
    theoretical_accounts = list(range(1, num_accounts + 1))
    zipf_weights = [1.0 / (i ** zipf_param) for i in range(1, num_accounts + 1)]
    total_weight = sum(zipf_weights)
    total_transactions = sum(counts)
    theoretical_counts = [weight * total_transactions / total_weight for weight in zipf_weights]
    
    # Linear-linear scale
    ax1.scatter(accounts, counts, alpha=0.6, label='Actual Distribution')
    ax1.plot(theoretical_accounts, theoretical_counts, 'r--', label='Theoretical Distribution')
    ax1.set_title(f'{role.capitalize()} Account Selection Distribution (Linear-Linear)')
    ax1.set_xlabel('Account ID')
    ax1.set_ylabel('Number of Transactions')
    ax1.legend()
    ax1.grid(True)
        
    # Log-linear scale
    ax2.scatter(accounts, counts, alpha=0.6, label='Actual Distribution')
    ax2.plot(theoretical_accounts, theoretical_counts, 'r--', label='Theoretical Distribution')
    ax2.set_title(f'{role.capitalize()} Account Selection Distribution (Log-Linear)')
    ax2.set_xlabel('Account ID')
    ax2.set_ylabel('Number of Transactions')
    ax2.set_yscale('log')
    ax2.legend()
    ax2.grid(True)
    
    # Log-log scale
    ax3.scatter(accounts, counts, alpha=0.6, label='Actual Distribution')
    ax3.plot(theoretical_accounts, theoretical_counts, 'r--', label='Theoretical Distribution')
    ax3.set_title(f'{role.capitalize()} Account Selection Distribution (Log-Log)')
    ax3.set_xlabel('Account ID')
    ax3.set_ylabel('Number of Transactions')
    ax3.set_xscale('log')
    ax3.set_yscale('log')
    ax3.legend()
    ax3.grid(True)
        
    plt.tight_layout()
    plt.savefig(f'simulator/results/sim_simple/figs/account_{role}_selection.png')
    plt.close()
        
def plot_account_selection():
    """
    Main function to plot account selection distributions for both senders and receivers.
    
    This function:
    1. Loads simulation parameters (zipf_param, num_accounts)
    2. Plots sender distribution with uniform theoretical curve (zipf_param = 0)
    3. Plots receiver distribution with Zipf theoretical curve
    
    The plots are saved as:
    - simulator/results/sim_simple/figs/account_sender_selection.png
    - simulator/results/sim_simple/figs/account_receiver_selection.png
    """
    # Load simulation parameters
    with open('simulator/results/sim_simple/data/sim_0/run_average/simulation_stats.json', 'r') as f:
        sim_stats = json.load(f)
    
    # Get parameters
    zipf_param = sim_stats['parameters']['zipf_parameter']
    num_accounts = sim_stats['parameters']['num_accounts']
    
    # Plot sender distribution (uniform, zipf_param = 0)
    plot_distribution('sender', 0.0, num_accounts)
    
    # Plot receiver distribution (Zipf)
    plot_distribution('receiver', zipf_param, num_accounts)

if __name__ == '__main__':
    plot_account_selection() 