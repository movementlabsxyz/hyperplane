#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.stats import zipf
import os
from plot_account_selection import plot_account_selection

def load_simulation_data():
    with open('simulator/results/data/simulation_stats.json', 'r') as f:
        return json.load(f)

def plot_account_selection_distribution():
    # Load account distribution data
    with open('simulator/results/data/account_selection_distribution.json', 'r') as f:
        account_data = json.load(f)
    
    # Load simulation parameters
    with open('simulator/results/data/simulation_stats.json', 'r') as f:
        sim_data = json.load(f)
    
    distribution = account_data['distribution']
    params = account_data['parameters']
    
    # Extract the distribution data
    keys = [entry['account'] for entry in distribution]
    counts = [entry['transactions'] for entry in distribution]
    
    # Calculate theoretical Zipf distribution
    zipf_param = params['zipf_parameter']
    num_accounts = params['num_accounts']
    total_transactions = params['total_transactions']
    x = np.arange(1, num_accounts + 1)  # Start from 1
    
    # Calculate theoretical distribution
    # Zipf PMF is proportional to 1/k^s where s is the parameter
    theoretical = np.array([1.0 / (k ** zipf_param) for k in x])
    # Normalize to match total transactions
    theoretical = theoretical * total_transactions / theoretical.sum()
    
    # Create the log-log plot
    plt.figure(figsize=(12, 6))
    
    # Plot actual distribution as scatter points
    plt.scatter(keys, counts, alpha=0.7, label='Actual Distribution', s=50)
    
    # Plot theoretical distribution
    plt.plot(x, theoretical, 'r-', label='Theoretical Zipf Distribution')
    
    # Customize the plot
    plt.title(f'Account Selection Distribution (Zipf parameter: {zipf_param})')
    plt.xlabel('Account Index')
    plt.ylabel('Selection Count')
    plt.grid(True, alpha=0.3)
    plt.xscale('log')  # Set x-axis to log scale
    plt.yscale('log')  # Set y-axis to log scale
    
    # Set axis limits
    plt.xlim(min(keys), max(keys))
    plt.ylim(1, max(counts) * 1.1)  # Start at 1, end slightly above max count
    
    plt.legend()
    
    # Add total transactions to the plot
    plt.text(0.02, 0.98, f'Total Transactions: {total_transactions}', 
             transform=plt.gca().transAxes, verticalalignment='top')
    
    # Save the log-log plot
    plt.savefig('simulator/results/figs/account_distribution_log_log.png', dpi=300, bbox_inches='tight')
    plt.close()

    # Create the linear-log plot
    plt.figure(figsize=(12, 6))
    
    # Plot actual distribution as scatter points
    plt.scatter(keys, counts, alpha=0.7, label='Actual Distribution', s=50)
    
    # Plot theoretical distribution
    plt.plot(x, theoretical, 'r-', label='Theoretical Zipf Distribution')
    
    # Customize the plot
    plt.title(f'Account Selection Distribution (Zipf parameter: {zipf_param})')
    plt.xlabel('Account Index')
    plt.ylabel('Selection Count')
    plt.grid(True, alpha=0.3)
    plt.yscale('log')  # Only y-axis is log scale
    
    # Set axis limits
    plt.xlim(0, max(keys) + 1)  # Linear x-axis
    plt.ylim(1, max(counts) * 1.1)  # Start at 1, end slightly above max count
    
    plt.legend()
    
    # Add total transactions to the plot
    plt.text(0.02, 0.98, f'Total Transactions: {total_transactions}', 
             transform=plt.gca().transAxes, verticalalignment='top')
    
    # Save the linear-log plot
    plt.savefig('simulator/results/figs/account_distribution_lin_log.png', dpi=300, bbox_inches='tight')
    plt.close()

    # Create the linear-linear plot
    plt.figure(figsize=(12, 6))
    
    # Plot actual distribution as scatter points
    plt.scatter(keys, counts, alpha=0.7, label='Actual Distribution', s=50)
    
    # Plot theoretical distribution
    plt.plot(x, theoretical, 'r-', label='Theoretical Zipf Distribution')
    
    # Customize the plot
    plt.title(f'Account Selection Distribution (Zipf parameter: {zipf_param})')
    plt.xlabel('Account Index')
    plt.ylabel('Selection Count')
    plt.grid(True, alpha=0.3)
    
    # Set axis limits
    plt.xlim(0, max(keys) + 1)  # Linear x-axis
    plt.ylim(0, max(counts) * 1.1)  # Linear y-axis
    
    plt.legend()
    
    # Add total transactions to the plot
    plt.text(0.02, 0.98, f'Total Transactions: {total_transactions}', 
             transform=plt.gca().transAxes, verticalalignment='top')
    
    # Save the linear-linear plot
    plt.savefig('simulator/results/figs/account_distribution_lin_lin.png', dpi=300, bbox_inches='tight')
    plt.close()

def plot_transaction_success():
    # Load simulation stats
    with open('simulator/results/data/simulation_stats.json', 'r') as f:
        stats = json.load(f)
    
    # Extract data
    successful_txs = stats['results']['successful_transactions']
    failed_txs = stats['results']['failed_transactions']
    
    # Create pie chart
    plt.figure(figsize=(8, 8))
    plt.pie([successful_txs, failed_txs], labels=['Successful', 'Failed'], autopct='%1.1f%%')
    plt.title('Transaction Success Rate')
    plt.savefig('simulator/results/figs/transaction_success.png')
    plt.close()

def plot_transaction_types():
    # Load simulation stats
    with open('simulator/results/data/simulation_stats.json', 'r') as f:
        stats = json.load(f)
    
    # Extract data
    cat_txs = stats['results']['cat_transactions']
    regular_txs = stats['results']['regular_transactions']
    
    # Create pie chart
    plt.figure(figsize=(8, 8))
    plt.pie([cat_txs, regular_txs], labels=['CAT', 'Regular'], autopct='%1.1f%%')
    plt.title('Transaction Types')
    plt.savefig('simulator/results/figs/transaction_types.png')
    plt.close()

def plot_pending_transactions():
    # Load pending transactions data
    with open('simulator/results/data/pending_transactions.json', 'r') as f:
        data = json.load(f)
    
    # Extract data
    blocks = [entry['block'] for entry in data['pending_transactions_by_height']]
    pending_counts = [entry['pending_count'] for entry in data['pending_transactions_by_height']]
    
    # Create plot
    plt.figure(figsize=(12, 6))
    plt.plot(blocks, pending_counts, 'b-', label='Pending Transactions')
    plt.title('Pending Transactions by height')
    plt.xlabel('Block Number')
    plt.ylabel('Number of Pending Transactions')
    plt.grid(True)
    plt.legend()
    plt.savefig('simulator/results/figs/pending_transactions.png')
    plt.close()

def plot_cumulative_transactions():
    # Load simulation stats
    with open('simulator/results/data/simulation_stats.json', 'r') as f:
        stats = json.load(f)
    
    # Load pending transactions data
    with open('simulator/results/data/pending_transactions.json', 'r') as f:
        pending_data = json.load(f)
    
    # Extract data
    blocks = [entry['block'] for entry in pending_data['pending_transactions_by_height']]
    pending = [entry['pending_count'] for entry in pending_data['pending_transactions_by_height']]
    
    # Calculate cumulative transactions
    total_txs = stats['results']['total_transactions']
    successful_txs = stats['results']['successful_transactions']
    failed_txs = stats['results']['failed_transactions']
    
    # Create line plot
    plt.figure(figsize=(10, 6))
    plt.plot(blocks, pending, label='Pending')
    plt.axhline(y=total_txs, color='g', linestyle='--', label='Total')
    plt.axhline(y=successful_txs, color='b', linestyle='--', label='Successful')
    plt.axhline(y=failed_txs, color='r', linestyle='--', label='Failed')
    plt.title('Cumulative Transactions Over Time')
    plt.xlabel('Block Number')
    plt.ylabel('Number of Transactions')
    plt.grid(True)
    plt.legend()
    plt.savefig('simulator/results/figs/cumulative_transactions.png')
    plt.close()

def plot_parameters():
    data = load_simulation_data()
    params = data['parameters']
    
    # Create a text file with parameters
    with open('simulator/results/figs/parameters.txt', 'w') as f:
        f.write("Simulation Parameters:\n")
        f.write("=====================\n")
        f.write(f"Initial Balance: {params['initial_balance']}\n")
        f.write(f"Number of Accounts: {params['num_accounts']}\n")
        f.write(f"Target TPS: {params['target_tps']}\n")
        f.write(f"Duration (seconds): {params['duration_seconds']}\n")
        f.write(f"Zipf Parameter: {params['zipf_parameter']}\n")
        f.write(f"CAT Ratio: {params['ratio_cats']}\n")
        f.write(f"Block Interval: {params['block_interval']}\n")
        f.write(f"Chain Delays: {params['chain_delays']}\n")

def plot_simulation_results():
    # Load simulation stats
    with open('simulator/results/data/simulation_stats.json', 'r') as f:
        stats = json.load(f)
    
    # Extract data
    total_txs = stats['results']['total_transactions']
    successful_txs = stats['results']['successful_transactions']
    failed_txs = stats['results']['failed_transactions']
    cat_txs = stats['results']['cat_transactions']
    regular_txs = stats['results']['regular_transactions']
    
    # Create figure with subplots
    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 12))
    
    # Plot transaction success/failure
    ax1.pie([successful_txs, failed_txs], labels=['Successful', 'Failed'], autopct='%1.1f%%')
    ax1.set_title('Transaction Success Rate')
    
    # Plot transaction types
    ax2.pie([cat_txs, regular_txs], labels=['CAT', 'Regular'], autopct='%1.1f%%')
    ax2.set_title('Transaction Types')
    
    # Plot pending transactions over time
    blocks = [entry['block'] for entry in stats['results']['pending_transactions_by_height']]
    pending = [entry['pending_count'] for entry in stats['results']['pending_transactions_by_height']]
    ax3.plot(blocks, pending)
    ax3.set_title('Pending Transactions Over Time')
    ax3.set_xlabel('Block Number')
    ax3.set_ylabel('Pending Transactions')
    
    # Plot cumulative transactions
    ax4.plot(blocks, np.cumsum(pending))
    ax4.set_title('Cumulative Transactions')
    ax4.set_xlabel('Block Number')
    ax4.set_ylabel('Total Transactions')
    
    # Adjust layout and save
    plt.tight_layout()
    plt.savefig('simulator/results/figs/simulation_results.png')
    plt.close()

def main():
    # Create results and figs directories if they don't exist
    os.makedirs('simulator/results/figs', exist_ok=True)
    
    # Generate all plots
    plot_transaction_success()
    plot_transaction_types()
    plot_pending_transactions()
    plot_cumulative_transactions()
    plot_account_selection()
    
    print("All plots have been generated in the simulator/results/figs directory")

if __name__ == '__main__':
    main() 