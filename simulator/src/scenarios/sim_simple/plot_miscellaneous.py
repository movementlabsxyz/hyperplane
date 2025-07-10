#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.stats import zipf
import os

# Global variable for the base data path
BASE_DATA_PATH = 'simulator/results/sim_simple/data/sim_0/run_average'
# Global variable for the output figures path
FIGS_PATH = 'simulator/results/sim_simple/figs'

def load_simulation_data():
    with open(f'{BASE_DATA_PATH}/simulation_stats.json', 'r') as f:
        return json.load(f)

def load_transaction_data(transaction_type):
    """
    Load transaction data for a given type (pending, success, failure).
    Returns a dictionary with chain data and breakdown data.
    """
    try:
        # Load main transaction data
        with open(f'{BASE_DATA_PATH}/{transaction_type}_transactions_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        with open(f'{BASE_DATA_PATH}/{transaction_type}_transactions_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        
        chain_1_blocks = [entry['height'] for entry in chain_1_data[f'chain_1_{transaction_type}']]
        chain_1_main = [entry['count'] for entry in chain_1_data[f'chain_1_{transaction_type}']]
        chain_2_blocks = [entry['height'] for entry in chain_2_data[f'chain_2_{transaction_type}']]
        chain_2_main = [entry['count'] for entry in chain_2_data[f'chain_2_{transaction_type}']]
        
        # Load CAT and regular breakdown data
        def load_breakdown_data(chain_num, tx_type):
            try:
                with open(f'{BASE_DATA_PATH}/cat_{tx_type}_transactions_chain_{chain_num}.json', 'r') as f:
                    cat_data = json.load(f)
                cat_counts = [entry['count'] for entry in cat_data[f'chain_{chain_num}_cat_{tx_type}']]
            except:
                cat_counts = [0] * len(chain_1_blocks if chain_num == 1 else chain_2_blocks)
            
            try:
                with open(f'{BASE_DATA_PATH}/regular_{tx_type}_transactions_chain_{chain_num}.json', 'r') as f:
                    regular_data = json.load(f)
                regular_counts = [entry['count'] for entry in regular_data[f'chain_{chain_num}_regular_{tx_type}']]
            except:
                regular_counts = [0] * len(chain_1_blocks if chain_num == 1 else chain_2_blocks)
            
            return cat_counts, regular_counts
        
        chain_1_cat, chain_1_regular = load_breakdown_data(1, transaction_type)
        chain_2_cat, chain_2_regular = load_breakdown_data(2, transaction_type)
        
        return {
            'chain_1_blocks': chain_1_blocks,
            'chain_1_main': chain_1_main,
            'chain_1_cat': chain_1_cat,
            'chain_1_regular': chain_1_regular,
            'chain_2_blocks': chain_2_blocks,
            'chain_2_main': chain_2_main,
            'chain_2_cat': chain_2_cat,
            'chain_2_regular': chain_2_regular
        }
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error loading {transaction_type} transaction data: {e}")
        return None

def create_plot(blocks, data_series, title, ylabel, filename, figsize=(12, 6)):
    """Helper function to create and save a plot."""
    plt.figure(figsize=figsize)
    for series in data_series:
        plt.plot(series['blocks'], series['data'], series['style'], 
                label=series['label'], linewidth=series.get('linewidth', 1.5))
    
    plt.title(title)
    plt.xlabel('Block Height')
    plt.ylabel(ylabel)
    plt.xlim(left=0)
    plt.grid(True)
    plt.legend()
    plt.savefig(f'{FIGS_PATH}/{filename}')
    plt.close()

def plot_transaction_type(transaction_type):
    """
    Generic function to plot a transaction type (pending, success, failure).
    Creates all the standard plots for that transaction type.
    """
    data = load_transaction_data(transaction_type)
    if not data:
        return
    
    tx_type_capitalized = transaction_type.capitalize()
    
    # Plot chain 1 with breakdown
    create_plot(
        data['chain_1_blocks'],
        [
            {'blocks': data['chain_1_blocks'], 'data': data['chain_1_main'], 'style': 'b-', 'label': 'Total', 'linewidth': 2},
            {'blocks': data['chain_1_blocks'], 'data': data['chain_1_cat'], 'style': 'r-', 'label': 'CAT'},
            {'blocks': data['chain_1_blocks'], 'data': data['chain_1_regular'], 'style': 'g-', 'label': 'Regular'}
        ],
        f'{tx_type_capitalized} Transactions by Height (Chain 1)',
        f'Number of {tx_type_capitalized} Transactions',
        f'tx_count_{transaction_type}_chain1.png'
    )
    
    # Plot chain 2 with breakdown
    create_plot(
        data['chain_2_blocks'],
        [
            {'blocks': data['chain_2_blocks'], 'data': data['chain_2_main'], 'style': 'b-', 'label': 'Total', 'linewidth': 2},
            {'blocks': data['chain_2_blocks'], 'data': data['chain_2_cat'], 'style': 'r-', 'label': 'CAT'},
            {'blocks': data['chain_2_blocks'], 'data': data['chain_2_regular'], 'style': 'g-', 'label': 'Regular'}
        ],
        f'{tx_type_capitalized} Transactions by Height (Chain 2)',
        f'Number of {tx_type_capitalized} Transactions',
        f'tx_count_{transaction_type}_chain2.png'
    )
    
    # Plot both chains together
    create_plot(
        data['chain_1_blocks'],
        [
            {'blocks': data['chain_1_blocks'], 'data': data['chain_1_main'], 'style': 'b-', 'label': 'Chain 1'},
            {'blocks': data['chain_2_blocks'], 'data': data['chain_2_main'], 'style': 'r--', 'label': 'Chain 2'}
        ],
        f'{tx_type_capitalized} Transactions by Height',
        f'Number of {tx_type_capitalized} Transactions',
        f'tx_count_{transaction_type}_all.png'
    )
    
    # Plot CAT transactions only
    create_plot(
        data['chain_1_blocks'],
        [
            {'blocks': data['chain_1_blocks'], 'data': data['chain_1_cat'], 'style': 'b-', 'label': 'Chain 1 CAT', 'linewidth': 2},
            {'blocks': data['chain_2_blocks'], 'data': data['chain_2_cat'], 'style': 'r--', 'label': 'Chain 2 CAT', 'linewidth': 2}
        ],
        f'CAT {tx_type_capitalized} Transactions by Height',
        f'Number of CAT {tx_type_capitalized} Transactions',
        f'tx_count_{transaction_type}_cat.png'
    )
    
    # Plot regular transactions only
    create_plot(
        data['chain_1_blocks'],
        [
            {'blocks': data['chain_1_blocks'], 'data': data['chain_1_regular'], 'style': 'b-', 'label': 'Chain 1 Regular', 'linewidth': 2},
            {'blocks': data['chain_2_blocks'], 'data': data['chain_2_regular'], 'style': 'r--', 'label': 'Chain 2 Regular', 'linewidth': 2}
        ],
        f'Regular {tx_type_capitalized} Transactions by Height',
        f'Number of Regular {tx_type_capitalized} Transactions',
        f'tx_count_{transaction_type}_regular.png'
    )

def plot_pending_transactions():
    """Plot pending transactions using the generic function."""
    plot_transaction_type('pending')

def plot_success_transactions():
    """Plot success transactions using the generic function."""
    plot_transaction_type('success')

def plot_failure_transactions():
    """Plot failure transactions using the generic function."""
    plot_transaction_type('failure')

def plot_parameters():
    data = load_simulation_data()
    params = data['parameters']
    
    # Create a text file with parameters
    with open(f'{FIGS_PATH}/parameters.txt', 'w') as f:
        f.write("Simulation Parameters:\n")
        f.write("=====================\n")
        f.write(f"Initial Balance: {params['initial_balance']}\n")
        f.write(f"Number of Accounts: {params['num_accounts']}\n")
        f.write(f"Target TPS: {params['target_tps']}\n")
        # Handle both old and new parameter names
        if 'duration_seconds' in params:
            f.write(f"Duration (seconds): {params['duration_seconds']}\n")
        if 'sim_total_block_number' in params:
            f.write(f"Total Blocks: {params['sim_total_block_number']}\n")
        f.write(f"Zipf Parameter: {params['zipf_parameter']}\n")
        f.write(f"CAT Ratio: {params['ratio_cats']}\n")
        f.write(f"Block Interval: {params['block_interval']}\n")
        f.write(f"Chain Delays: {params['chain_delays']}\n")
