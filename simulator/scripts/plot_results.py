#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.stats import zipf
import os

def load_simulation_data():
    with open('simulator/results/simulation_stats.json', 'r') as f:
        return json.load(f)

def plot_account_selection_distribution():
    data = load_simulation_data()
    distribution = data['results']['account_selection_distribution']
    params = data['parameters']
    
    # Extract the distribution data
    keys = [entry['account'] for entry in distribution]
    counts = [entry['transactions'] for entry in distribution]
    
    # Calculate theoretical Zipf distribution
    zipf_param = params['zipf_parameter']
    num_accounts = max(keys)
    x = np.arange(1, num_accounts + 1)  # Start from 1
    
    # Calculate theoretical distribution
    # Zipf PMF is proportional to 1/k^s where s is the parameter
    theoretical = np.array([1.0 / (k ** zipf_param) for k in x])
    # Normalize to match total transactions
    theoretical = theoretical * data['results']['total_transactions'] / theoretical.sum()
    
    # Create the plot
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
    plt.text(0.02, 0.98, f'Total Transactions: {data["results"]["total_transactions"]}', 
             transform=plt.gca().transAxes, verticalalignment='top')
    
    # Save the plot
    plt.savefig('simulator/results/figs/account_distribution.png', dpi=300, bbox_inches='tight')
    plt.close()

def plot_transaction_types():
    data = load_simulation_data()
    results = data['results']
    
    # Get transaction counts
    cat_count = results['cat_transactions']['count']
    regular_count = results['regular_transactions']['count']
    
    # Create pie chart
    plt.figure(figsize=(8, 8))
    plt.pie([cat_count, regular_count], 
            labels=['CAT', 'Regular'],
            autopct='%1.1f%%',
            colors=['#ff9999','#66b3ff'])
    plt.title('Transaction Types Distribution')
    plt.savefig('simulator/results/figs/transaction_types.png')
    plt.close()

def plot_success_rate():
    data = load_simulation_data()
    results = data['results']
    
    success_rate = results['success_rate']
    
    plt.figure(figsize=(8, 6))
    plt.bar(['Success Rate'], [success_rate], color='green')
    plt.title('Transaction Success Rate')
    plt.ylabel('Percentage')
    plt.ylim(0, 100)
    plt.grid(True, alpha=0.3)
    plt.savefig('simulator/results/figs/success_rate.png')
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

def main():
    # Create figs directory if it doesn't exist
    os.makedirs('simulator/results/figs', exist_ok=True)
    
    # Generate all plots
    plot_account_selection_distribution()
    plot_transaction_types()
    plot_success_rate()
    plot_parameters()
    
    print("Plots generated in simulator/results/figs/")

if __name__ == "__main__":
    main() 