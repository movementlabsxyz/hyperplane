#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.stats import zipf
import os

def load_simulation_data():
    with open('simulator/results/data/simulation_stats.json', 'r') as f:
        return json.load(f)

def plot_pending_transactions():
    try:
        # Load pending transactions data for both chains
        with open('simulator/results/data/pending_transactions_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        with open('simulator/results/data/pending_transactions_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        
        # Extract data for chain 1
        chain_1_blocks = [entry['height'] for entry in chain_1_data['chain_1_pending']]
        chain_1_pending = [entry['count'] for entry in chain_1_data['chain_1_pending']]
        
        # Extract data for chain 2
        chain_2_blocks = [entry['height'] for entry in chain_2_data['chain_2_pending']]
        chain_2_pending = [entry['count'] for entry in chain_2_data['chain_2_pending']]
        
        # Check for valid values
        if not chain_1_blocks or not chain_1_pending or not chain_2_blocks or not chain_2_pending:
            print("Warning: Empty pending transaction data found, skipping pending transactions plot")
            return
        
        # Create plot for chain 1
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_pending, 'b-', label='Chain 1 Pending Transactions')
        plt.title('Chain 1 Pending Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Pending Transactions')
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/figs/pending_transactions_chain_1.png')
        plt.close()
        
        # Create plot for chain 2
        plt.figure(figsize=(12, 6))
        plt.plot(chain_2_blocks, chain_2_pending, 'r-', label='Chain 2 Pending Transactions')
        plt.title('Chain 2 Pending Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Pending Transactions')
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/figs/pending_transactions_chain_2.png')
        plt.close()
        
        # Create combined plot
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_pending, 'b-', label='Chain 1')
        plt.plot(chain_2_blocks, chain_2_pending, 'r--', label='Chain 2')
        plt.title('Pending Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Pending Transactions')
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/figs/pending_transactions_combined.png')
        plt.close()
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing pending transactions data: {e}")
        return


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
