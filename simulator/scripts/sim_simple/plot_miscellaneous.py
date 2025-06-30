#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.stats import zipf
import os

def load_simulation_data():
    with open('simulator/results/sim_simple/data/simulation_stats.json', 'r') as f:
        return json.load(f)

def plot_pending_transactions():
    try:
        # Load pending transactions data for both chains
        with open('simulator/results/sim_simple/data/pending_transactions_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        with open('simulator/results/sim_simple/data/pending_transactions_chain_2.json', 'r') as f:
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
        
        # Create combined plot
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_pending, 'b-', label='Chain 1')
        plt.plot(chain_2_blocks, chain_2_pending, 'r--', label='Chain 2')
        plt.title('Pending Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Pending Transactions')
        plt.xlim(left=0)  # Start x-axis at 0
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_pending.png')
        plt.close()
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing pending transactions data: {e}")
        return


def plot_success_transactions():
    try:
        # Load success transactions data for both chains
        with open('simulator/results/sim_simple/data/success_transactions_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        with open('simulator/results/sim_simple/data/success_transactions_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        
        # Extract data for chain 1
        chain_1_blocks = [entry['height'] for entry in chain_1_data['chain_1_success']]
        chain_1_success = [entry['count'] for entry in chain_1_data['chain_1_success']]
        
        # Extract data for chain 2
        chain_2_blocks = [entry['height'] for entry in chain_2_data['chain_2_success']]
        chain_2_success = [entry['count'] for entry in chain_2_data['chain_2_success']]
        
        # Check for valid values
        if not chain_1_blocks or not chain_1_success or not chain_2_blocks or not chain_2_success:
            print("Warning: Empty success transaction data found, skipping success transactions plot")
            return
        
        # Create combined plot
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_success, 'b-', label='Chain 1')
        plt.plot(chain_2_blocks, chain_2_success, 'r--', label='Chain 2')
        plt.title('Success Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Success Transactions')
        plt.xlim(left=0)  # Start x-axis at 0
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_success.png')
        plt.close()
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing success transactions data: {e}")
        return


def plot_failure_transactions():
    try:
        # Load failure transactions data for both chains
        with open('simulator/results/sim_simple/data/failure_transactions_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        with open('simulator/results/sim_simple/data/failure_transactions_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        
        # Extract data for chain 1
        chain_1_blocks = [entry['height'] for entry in chain_1_data['chain_1_failure']]
        chain_1_failure = [entry['count'] for entry in chain_1_data['chain_1_failure']]
        
        # Extract data for chain 2
        chain_2_blocks = [entry['height'] for entry in chain_2_data['chain_2_failure']]
        chain_2_failure = [entry['count'] for entry in chain_2_data['chain_2_failure']]
        
        # Check for valid values
        if not chain_1_blocks or not chain_1_failure or not chain_2_blocks or not chain_2_failure:
            print("Warning: Empty failure transaction data found, skipping failure transactions plot")
            return
        
        # Create combined plot
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_failure, 'b-', label='Chain 1')
        plt.plot(chain_2_blocks, chain_2_failure, 'r--', label='Chain 2')
        plt.title('Failure Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Failure Transactions')
        plt.xlim(left=0)  # Start x-axis at 0
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_failure.png')
        plt.close()
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing failure transactions data: {e}")
        return


def plot_parameters():
    data = load_simulation_data()
    params = data['parameters']
    
    # Create a text file with parameters
    with open('simulator/results/sim_simple/figs/parameters.txt', 'w') as f:
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
