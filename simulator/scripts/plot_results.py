#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.stats import zipf

def plot_zipf_distribution():
    # Read the simulation results
    with open('simulator/results/simulation_stats.json', 'r') as f:
        data = json.load(f)
    
    # Extract the distribution data
    distribution = data['key_selection_distribution']
    keys = [entry['key'] for entry in distribution]
    counts = [entry['count'] for entry in distribution]
    
    # Calculate theoretical Zipf distribution
    zipf_param = data['zipf_parameter']
    num_accounts = max(keys)
    x = np.arange(1, num_accounts + 1)  # Start from 1
    
    # Calculate theoretical distribution
    # Zipf PMF is proportional to 1/k^s where s is the parameter
    theoretical = np.array([1.0 / (k ** zipf_param) for k in x])
    # Normalize to match total transactions
    theoretical = theoretical * data['total_transactions'] / theoretical.sum()
    
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
    plt.text(0.02, 0.98, f'Total Transactions: {data["total_transactions"]}', 
             transform=plt.gca().transAxes, verticalalignment='top')
    
    # Save the plot
    plt.savefig('simulator/results/zipf_distribution.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # Save distribution data with metadata
    distribution_data = {
        "zipf_parameter": zipf_param,
        "num_accounts": num_accounts,
        "total_transactions": data["total_transactions"],
        "distribution": distribution
    }
    
    with open('simulator/results/key_selection_distribution.json', 'w') as f:
        json.dump(distribution_data, f, indent=2)

if __name__ == '__main__':
    plot_zipf_distribution() 