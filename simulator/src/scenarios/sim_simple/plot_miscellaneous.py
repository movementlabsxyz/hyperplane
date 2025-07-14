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
    """Load simulation statistics from the averaged results."""
    with open(f'{BASE_DATA_PATH}/simulation_stats.json', 'r') as f:
        return json.load(f)

def load_transaction_data(transaction_type):
    """
    Load transaction data for a given type (pending, success, failure).
    Returns a dictionary with chain data and breakdown data.
    """
    try:
        # Load main transaction data to get block heights
        with open(f'{BASE_DATA_PATH}/{transaction_type}_transactions_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        with open(f'{BASE_DATA_PATH}/{transaction_type}_transactions_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        
        chain_1_blocks = [entry['height'] for entry in chain_1_data[f'chain_1_{transaction_type}']]
        chain_2_blocks = [entry['height'] for entry in chain_2_data[f'chain_2_{transaction_type}']]
        
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
            'chain_1_cat': chain_1_cat,
            'chain_1_regular': chain_1_regular,
            'chain_2_blocks': chain_2_blocks,
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
    
    # Calculate total as sum of CAT and regular transactions
    def calculate_total(cat_data, regular_data):
        """Helper function to sum CAT and regular transactions."""
        return [cat + reg for cat, reg in zip(cat_data, regular_data)]
    
    chain_1_total = calculate_total(data['chain_1_cat'], data['chain_1_regular'])
    chain_2_total = calculate_total(data['chain_2_cat'], data['chain_2_regular'])
    
    # Plot chain 1 with breakdown
    create_plot(
        data['chain_1_blocks'],
        [
            {'blocks': data['chain_1_blocks'], 'data': chain_1_total, 'style': 'b-', 'label': 'Total', 'linewidth': 2},
            {'blocks': data['chain_1_blocks'], 'data': data['chain_1_cat'], 'style': 'r-', 'label': 'CAT'},
            {'blocks': data['chain_1_blocks'], 'data': data['chain_1_regular'], 'style': 'g-', 'label': 'Regular'}
        ],
        f'{tx_type_capitalized} Transactions by Height (Chain 1)',
        f'Number of {tx_type_capitalized} Transactions',
        f'tx_{transaction_type}__chain1.png'
    )
    
    # Plot chain 2 with breakdown
    create_plot(
        data['chain_2_blocks'],
        [
            {'blocks': data['chain_2_blocks'], 'data': chain_2_total, 'style': 'b-', 'label': 'Total', 'linewidth': 2},
            {'blocks': data['chain_2_blocks'], 'data': data['chain_2_cat'], 'style': 'r-', 'label': 'CAT'},
            {'blocks': data['chain_2_blocks'], 'data': data['chain_2_regular'], 'style': 'g-', 'label': 'Regular'}
        ],
        f'{tx_type_capitalized} Transactions by Height (Chain 2)',
        f'Number of {tx_type_capitalized} Transactions',
        f'tx_{transaction_type}__chain2.png'
    )
    
    # Plot both chains together
    create_plot(
        data['chain_1_blocks'],
        [
            {'blocks': data['chain_1_blocks'], 'data': chain_1_total, 'style': 'b-', 'label': 'Chain 1'},
            {'blocks': data['chain_2_blocks'], 'data': chain_2_total, 'style': 'r--', 'label': 'Chain 2'}
        ],
        f'{tx_type_capitalized} Transactions by Height',
        f'Number of {tx_type_capitalized} Transactions',
        f'tx_{transaction_type}_sumTypes.png'
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
        f'tx_{transaction_type}_cat.png'
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
        f'tx_{transaction_type}_regular.png'
    )

def plot_tx_pending():
    """Plot pending transactions using the generic function."""
    plot_transaction_type('pending')

def plot_tx_success():
    """Plot success transactions using the generic function."""
    plot_transaction_type('success')

def plot_tx_failure():
    """Plot failure transactions using the generic function."""
    plot_transaction_type('failure')

def plot_parameters():
    """Create a text file with simulation parameters for reference."""
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

def plot_tx_allStatus_cat():
    """
    Plot CAT transactions: pending, success, and failure rates in the same figure.
    """
    try:
        # Load CAT transaction data for all states
        pending_data = load_transaction_data('pending')
        success_data = load_transaction_data('success')
        failure_data = load_transaction_data('failure')
        
        if not all([pending_data, success_data, failure_data]):
            print("Warning: Could not load all CAT transaction data for comparison plot")
            return
        
        # Create the plot
        plt.figure(figsize=(12, 6))
        
        # Plot CAT transactions for chain 1
        plt.plot(pending_data['chain_1_blocks'], pending_data['chain_1_cat'], 
                'orange', linewidth=2, label='CAT Pending (Chain 1)')
        plt.plot(success_data['chain_1_blocks'], success_data['chain_1_cat'], 
                'green', linewidth=2, label='CAT Success (Chain 1)')
        plt.plot(failure_data['chain_1_blocks'], failure_data['chain_1_cat'], 
                'red', linewidth=2, label='CAT Failure (Chain 1)')
        
        # Plot CAT transactions for chain 2
        plt.plot(pending_data['chain_2_blocks'], pending_data['chain_2_cat'], 
                'orange', linestyle='--', linewidth=2, label='CAT Pending (Chain 2)')
        plt.plot(success_data['chain_2_blocks'], success_data['chain_2_cat'], 
                'green', linestyle='--', linewidth=2, label='CAT Success (Chain 2)')
        plt.plot(failure_data['chain_2_blocks'], failure_data['chain_2_cat'], 
                'red', linestyle='--', linewidth=2, label='CAT Failure (Chain 2)')
        
        plt.title('CAT Transactions: Pending, Success, and Failure Rates')
        plt.xlabel('Block Height')
        plt.ylabel('Number of CAT Transactions')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend()
        
        # Save the plot
        plt.savefig(f'{FIGS_PATH}/tx_allStatus_cat.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error creating CAT comparison plot: {e}")

def plot_tx_allStatus_regular():
    """
    Plot regular transactions: pending, success, and failure rates in the same figure.
    """
    try:
        # Load regular transaction data for all states
        pending_data = load_transaction_data('pending')
        success_data = load_transaction_data('success')
        failure_data = load_transaction_data('failure')
        
        if not all([pending_data, success_data, failure_data]):
            print("Warning: Could not load all regular transaction data for comparison plot")
            return
        
        # Create the plot
        plt.figure(figsize=(12, 6))
        
        # Plot regular transactions for chain 1
        plt.plot(pending_data['chain_1_blocks'], pending_data['chain_1_regular'], 
                'orange', linewidth=2, label='Regular Pending (Chain 1)')
        plt.plot(success_data['chain_1_blocks'], success_data['chain_1_regular'], 
                'green', linewidth=2, label='Regular Success (Chain 1)')
        plt.plot(failure_data['chain_1_blocks'], failure_data['chain_1_regular'], 
                'red', linewidth=2, label='Regular Failure (Chain 1)')
        
        # Plot regular transactions for chain 2
        plt.plot(pending_data['chain_2_blocks'], pending_data['chain_2_regular'], 
                'orange', linestyle='--', linewidth=2, label='Regular Pending (Chain 2)')
        plt.plot(success_data['chain_2_blocks'], success_data['chain_2_regular'], 
                'green', linestyle='--', linewidth=2, label='Regular Success (Chain 2)')
        plt.plot(failure_data['chain_2_blocks'], failure_data['chain_2_regular'], 
                'red', linestyle='--', linewidth=2, label='Regular Failure (Chain 2)')
        
        plt.title('Regular Transactions: Pending, Success, and Failure Rates')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Regular Transactions')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend()
        
        # Save the plot
        plt.savefig(f'{FIGS_PATH}/tx_allStatus_regular.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error creating regular comparison plot: {e}")

def plot_tx_allStatus_all():
    """
    Plot all transactions: pending, success, and failure rates in the same figure.
    All transactions are calculated as CAT + regular transactions.
    """
    try:
        # Load transaction data for all states
        pending_data = load_transaction_data('pending')
        success_data = load_transaction_data('success')
        failure_data = load_transaction_data('failure')
        
        if not all([pending_data, success_data, failure_data]):
            print("Warning: Could not load all transaction data for comparison plot")
            return
        
        # Calculate all transactions as sum of CAT and regular for each chain and state
        def calculate_all_transactions(cat_data, regular_data):
            """Helper function to sum CAT and regular transactions."""
            return [cat + reg for cat, reg in zip(cat_data, regular_data)]
        
        # Calculate all transactions for chain 1
        chain_1_all_pending = calculate_all_transactions(pending_data['chain_1_cat'], pending_data['chain_1_regular'])
        chain_1_all_success = calculate_all_transactions(success_data['chain_1_cat'], success_data['chain_1_regular'])
        chain_1_all_failure = calculate_all_transactions(failure_data['chain_1_cat'], failure_data['chain_1_regular'])
        
        # Calculate all transactions for chain 2
        chain_2_all_pending = calculate_all_transactions(pending_data['chain_2_cat'], pending_data['chain_2_regular'])
        chain_2_all_success = calculate_all_transactions(success_data['chain_2_cat'], success_data['chain_2_regular'])
        chain_2_all_failure = calculate_all_transactions(failure_data['chain_2_cat'], failure_data['chain_2_regular'])
        
        # Create the plot
        plt.figure(figsize=(12, 6))
        
        # Plot all transactions for chain 1
        plt.plot(pending_data['chain_1_blocks'], chain_1_all_pending, 
                'orange', linewidth=2, label='All Pending (Chain 1)')
        plt.plot(success_data['chain_1_blocks'], chain_1_all_success, 
                'green', linewidth=2, label='All Success (Chain 1)')
        plt.plot(failure_data['chain_1_blocks'], chain_1_all_failure, 
                'red', linewidth=2, label='All Failure (Chain 1)')
        
        # Plot all transactions for chain 2
        plt.plot(pending_data['chain_2_blocks'], chain_2_all_pending, 
                'orange', linestyle='--', linewidth=2, label='All Pending (Chain 2)')
        plt.plot(success_data['chain_2_blocks'], chain_2_all_success, 
                'green', linestyle='--', linewidth=2, label='All Success (Chain 2)')
        plt.plot(failure_data['chain_2_blocks'], chain_2_all_failure, 
                'red', linestyle='--', linewidth=2, label='All Failure (Chain 2)')
        
        plt.title('All Transactions: Pending, Success, and Failure Rates (CAT + Regular)')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Transactions')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend()
        
        # Save the plot
        plt.savefig(f'{FIGS_PATH}/tx_allStatus_all.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error creating all transactions comparison plot: {e}")

def plot_comprehensive_comparison():
    """
    Create a comprehensive comparison plot with subplots showing all transaction types.
    """
    try:
        # Load transaction data for all states
        pending_data = load_transaction_data('pending')
        success_data = load_transaction_data('success')
        failure_data = load_transaction_data('failure')
        
        if not all([pending_data, success_data, failure_data]):
            print("Warning: Could not load all transaction data for comprehensive comparison plot")
            return
        
        # Calculate all transactions as sum of CAT and regular for each chain and state
        def calculate_all_transactions(cat_data, regular_data):
            """Helper function to sum CAT and regular transactions."""
            return [cat + reg for cat, reg in zip(cat_data, regular_data)]
        
        # Calculate all transactions for chain 1
        chain_1_all_pending = calculate_all_transactions(pending_data['chain_1_cat'], pending_data['chain_1_regular'])
        chain_1_all_success = calculate_all_transactions(success_data['chain_1_cat'], success_data['chain_1_regular'])
        chain_1_all_failure = calculate_all_transactions(failure_data['chain_1_cat'], failure_data['chain_1_regular'])
        
        # Calculate all transactions for chain 2
        chain_2_all_pending = calculate_all_transactions(pending_data['chain_2_cat'], pending_data['chain_2_regular'])
        chain_2_all_success = calculate_all_transactions(success_data['chain_2_cat'], success_data['chain_2_regular'])
        chain_2_all_failure = calculate_all_transactions(failure_data['chain_2_cat'], failure_data['chain_2_regular'])
        
        # Create subplots
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(16, 12))
        
        # Plot 1: All transactions
        ax1.plot(pending_data['chain_1_blocks'], chain_1_all_pending, 
                'orange', linewidth=2, label='Pending (Chain 1)')
        ax1.plot(success_data['chain_1_blocks'], chain_1_all_success, 
                'green', linewidth=2, label='Success (Chain 1)')
        ax1.plot(failure_data['chain_1_blocks'], chain_1_all_failure, 
                'red', linewidth=2, label='Failure (Chain 1)')
        ax1.plot(pending_data['chain_2_blocks'], chain_2_all_pending, 
                'orange', linestyle='--', linewidth=2, label='Pending (Chain 2)')
        ax1.plot(success_data['chain_2_blocks'], chain_2_all_success, 
                'green', linestyle='--', linewidth=2, label='Success (Chain 2)')
        ax1.plot(failure_data['chain_2_blocks'], chain_2_all_failure, 
                'red', linestyle='--', linewidth=2, label='Failure (Chain 2)')
        ax1.set_title('All Transactions: Pending, Success, and Failure (CAT + Regular)')
        ax1.set_xlabel('Block Height')
        ax1.set_ylabel('Number of Transactions')
        ax1.grid(True, alpha=0.3)
        ax1.legend()
        
        # Plot 2: CAT transactions
        ax2.plot(pending_data['chain_1_blocks'], pending_data['chain_1_cat'], 
                'orange', linewidth=2, label='Pending (Chain 1)')
        ax2.plot(success_data['chain_1_blocks'], success_data['chain_1_cat'], 
                'green', linewidth=2, label='Success (Chain 1)')
        ax2.plot(failure_data['chain_1_blocks'], failure_data['chain_1_cat'], 
                'red', linewidth=2, label='Failure (Chain 1)')
        ax2.plot(pending_data['chain_2_blocks'], pending_data['chain_2_cat'], 
                'orange', linestyle='--', linewidth=2, label='Pending (Chain 2)')
        ax2.plot(success_data['chain_2_blocks'], success_data['chain_2_cat'], 
                'green', linestyle='--', linewidth=2, label='Success (Chain 2)')
        ax2.plot(failure_data['chain_2_blocks'], failure_data['chain_2_cat'], 
                'red', linestyle='--', linewidth=2, label='Failure (Chain 2)')
        ax2.set_title('CAT Transactions: Pending, Success, and Failure')
        ax2.set_xlabel('Block Height')
        ax2.set_ylabel('Number of CAT Transactions')
        ax2.grid(True, alpha=0.3)
        ax2.legend()
        
        # Plot 3: Regular transactions
        ax3.plot(pending_data['chain_1_blocks'], pending_data['chain_1_regular'], 
                'orange', linewidth=2, label='Pending (Chain 1)')
        ax3.plot(success_data['chain_1_blocks'], success_data['chain_1_regular'], 
                'green', linewidth=2, label='Success (Chain 1)')
        ax3.plot(failure_data['chain_1_blocks'], failure_data['chain_1_regular'], 
                'red', linewidth=2, label='Failure (Chain 1)')
        ax3.plot(pending_data['chain_2_blocks'], pending_data['chain_2_regular'], 
                'orange', linestyle='--', linewidth=2, label='Pending (Chain 2)')
        ax3.plot(success_data['chain_2_blocks'], success_data['chain_2_regular'], 
                'green', linestyle='--', linewidth=2, label='Success (Chain 2)')
        ax3.plot(failure_data['chain_2_blocks'], failure_data['chain_2_regular'], 
                'red', linestyle='--', linewidth=2, label='Failure (Chain 2)')
        ax3.set_title('Regular Transactions: Pending, Success, and Failure')
        ax3.set_xlabel('Block Height')
        ax3.set_ylabel('Number of Regular Transactions')
        ax3.grid(True, alpha=0.3)
        ax3.legend()
        
        # Plot 4: Chain comparison (success only for clarity)
        ax4.plot(success_data['chain_1_blocks'], chain_1_all_success, 
                'blue', linewidth=2, label='All Success (Chain 1)')
        ax4.plot(success_data['chain_2_blocks'], chain_2_all_success, 
                'red', linewidth=2, label='All Success (Chain 2)')
        ax4.plot(success_data['chain_1_blocks'], success_data['chain_1_cat'], 
                'blue', linestyle='--', linewidth=2, label='CAT Success (Chain 1)')
        ax4.plot(success_data['chain_2_blocks'], success_data['chain_2_cat'], 
                'red', linestyle='--', linewidth=2, label='CAT Success (Chain 2)')
        ax4.set_title('Chain Comparison: Success Transactions')
        ax4.set_xlabel('Block Height')
        ax4.set_ylabel('Number of Successful Transactions')
        ax4.grid(True, alpha=0.3)
        ax4.legend()
        
        plt.tight_layout()
        
        # Save the plot
        plt.savefig(f'{FIGS_PATH}/comprehensive_comparison.png', dpi=300, bbox_inches='tight')
        plt.close()
        
    except Exception as e:
        print(f"Warning: Error creating comprehensive comparison plot: {e}")
