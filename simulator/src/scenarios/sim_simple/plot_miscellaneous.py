#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.stats import zipf
import os

def load_simulation_data():
    with open('simulator/results/sim_simple/data/run_average/simulation_stats.json', 'r') as f:
        return json.load(f)

def plot_pending_transactions():
    try:
        with open('simulator/results/sim_simple/data/run_average/pending_transactions_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        with open('simulator/results/sim_simple/data/run_average/pending_transactions_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        chain_1_blocks = [entry['height'] for entry in chain_1_data['chain_1_pending']]
        chain_1_pending = [entry['count'] for entry in chain_1_data['chain_1_pending']]
        chain_2_blocks = [entry['height'] for entry in chain_2_data['chain_2_pending']]
        chain_2_pending = [entry['count'] for entry in chain_2_data['chain_2_pending']]
        
        # Load CAT and regular pending data for chain 1
        try:
            with open('simulator/results/sim_simple/data/run_average/cat_pending_transactions_chain_1.json', 'r') as f:
                chain_1_cat_data = json.load(f)
            chain_1_cat_pending = [entry['count'] for entry in chain_1_cat_data['chain_1_cat_pending']]
        except:
            chain_1_cat_pending = [0] * len(chain_1_blocks)
        
        try:
            with open('simulator/results/sim_simple/data/run_average/regular_pending_transactions_chain_1.json', 'r') as f:
                chain_1_regular_data = json.load(f)
            chain_1_regular_pending = [entry['count'] for entry in chain_1_regular_data['chain_1_regular_pending']]
        except:
            chain_1_regular_pending = [0] * len(chain_1_blocks)
        
        # Load CAT and regular pending data for chain 2
        try:
            with open('simulator/results/sim_simple/data/run_average/cat_pending_transactions_chain_2.json', 'r') as f:
                chain_2_cat_data = json.load(f)
            chain_2_cat_pending = [entry['count'] for entry in chain_2_cat_data['chain_2_cat_pending']]
        except:
            chain_2_cat_pending = [0] * len(chain_2_blocks)
        
        try:
            with open('simulator/results/sim_simple/data/run_average/regular_pending_transactions_chain_2.json', 'r') as f:
                chain_2_regular_data = json.load(f)
            chain_2_regular_pending = [entry['count'] for entry in chain_2_regular_data['chain_2_regular_pending']]
        except:
            chain_2_regular_pending = [0] * len(chain_2_blocks)
        
        # Plot chain 1 only with CAT and regular breakdown
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_pending, 'b-', label='Total', linewidth=2)
        plt.plot(chain_1_blocks, chain_1_cat_pending, 'r-', label='CAT', linewidth=1.5)
        plt.plot(chain_1_blocks, chain_1_regular_pending, 'g-', label='Regular', linewidth=1.5)
        plt.title('Pending Transactions by Height (Chain 1)')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Pending Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_pending_chain1.png')
        plt.close()
        # Plot chain 2 only with CAT and regular breakdown
        plt.figure(figsize=(12, 6))
        plt.plot(chain_2_blocks, chain_2_pending, 'b-', label='Total', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_cat_pending, 'r-', label='CAT', linewidth=1.5)
        plt.plot(chain_2_blocks, chain_2_regular_pending, 'g-', label='Regular', linewidth=1.5)
        plt.title('Pending Transactions by Height (Chain 2)')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Pending Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_pending_chain2.png')
        plt.close()
        # Plot both chains together (original combined plot)
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_pending, 'b-', label='Chain 1')
        plt.plot(chain_2_blocks, chain_2_pending, 'r--', label='Chain 2')
        plt.title('Pending Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Pending Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_pending_all.png')
        plt.close()
        
        # Plot CAT pending transactions only (combined from both chains)
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_cat_pending, 'b-', label='Chain 1 CAT', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_cat_pending, 'r--', label='Chain 2 CAT', linewidth=2)
        plt.title('CAT Pending Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of CAT Pending Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_pending_cat.png')
        plt.close()
        
        # Plot regular pending transactions only (combined from both chains)
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_regular_pending, 'b-', label='Chain 1 Regular', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_regular_pending, 'r--', label='Chain 2 Regular', linewidth=2)
        plt.title('Regular Pending Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Regular Pending Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_pending_regular.png')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing pending transactions data: {e}")
        return

def plot_success_transactions():
    try:
        with open('simulator/results/sim_simple/data/run_average/success_transactions_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        with open('simulator/results/sim_simple/data/run_average/success_transactions_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        chain_1_blocks = [entry['height'] for entry in chain_1_data['chain_1_success']]
        chain_1_success = [entry['count'] for entry in chain_1_data['chain_1_success']]
        chain_2_blocks = [entry['height'] for entry in chain_2_data['chain_2_success']]
        chain_2_success = [entry['count'] for entry in chain_2_data['chain_2_success']]
        
        # Load CAT and regular success data for chain 1
        try:
            with open('simulator/results/sim_simple/data/run_average/cat_success_transactions_chain_1.json', 'r') as f:
                chain_1_cat_data = json.load(f)
            chain_1_cat_success = [entry['count'] for entry in chain_1_cat_data['chain_1_cat_success']]
        except:
            chain_1_cat_success = [0] * len(chain_1_blocks)
        
        try:
            with open('simulator/results/sim_simple/data/run_average/regular_success_transactions_chain_1.json', 'r') as f:
                chain_1_regular_data = json.load(f)
            chain_1_regular_success = [entry['count'] for entry in chain_1_regular_data['chain_1_regular_success']]
        except:
            chain_1_regular_success = [0] * len(chain_1_blocks)
        
        # Load CAT and regular success data for chain 2
        try:
            with open('simulator/results/sim_simple/data/run_average/cat_success_transactions_chain_2.json', 'r') as f:
                chain_2_cat_data = json.load(f)
            chain_2_cat_success = [entry['count'] for entry in chain_2_cat_data['chain_2_cat_success']]
        except:
            chain_2_cat_success = [0] * len(chain_2_blocks)
        
        try:
            with open('simulator/results/sim_simple/data/run_average/regular_success_transactions_chain_2.json', 'r') as f:
                chain_2_regular_data = json.load(f)
            chain_2_regular_success = [entry['count'] for entry in chain_2_regular_data['chain_2_regular_success']]
        except:
            chain_2_regular_success = [0] * len(chain_2_blocks)
        
        # Plot chain 1 only with CAT and regular breakdown
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_success, 'b-', label='Total', linewidth=2)
        plt.plot(chain_1_blocks, chain_1_cat_success, 'r-', label='CAT', linewidth=1.5)
        plt.plot(chain_1_blocks, chain_1_regular_success, 'g-', label='Regular', linewidth=1.5)
        plt.title('Success Transactions by Height (Chain 1)')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Success Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_success_chain1.png')
        plt.close()
        # Plot chain 2 only with CAT and regular breakdown
        plt.figure(figsize=(12, 6))
        plt.plot(chain_2_blocks, chain_2_success, 'b-', label='Total', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_cat_success, 'r-', label='CAT', linewidth=1.5)
        plt.plot(chain_2_blocks, chain_2_regular_success, 'g-', label='Regular', linewidth=1.5)
        plt.title('Success Transactions by Height (Chain 2)')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Success Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_success_chain2.png')
        plt.close()
        # Plot both chains together (original combined plot)
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_success, 'b-', label='Chain 1')
        plt.plot(chain_2_blocks, chain_2_success, 'r--', label='Chain 2')
        plt.title('Success Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Success Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_success_all.png')
        plt.close()
        
        # Plot CAT success transactions only (combined from both chains)
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_cat_success, 'b-', label='Chain 1 CAT', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_cat_success, 'r--', label='Chain 2 CAT', linewidth=2)
        plt.title('CAT Success Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of CAT Success Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_success_cat.png')
        plt.close()
        
        # Plot regular success transactions only (combined from both chains)
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_regular_success, 'b-', label='Chain 1 Regular', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_regular_success, 'r--', label='Chain 2 Regular', linewidth=2)
        plt.title('Regular Success Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Regular Success Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_success_regular.png')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing success transactions data: {e}")
        return

def plot_failure_transactions():
    try:
        with open('simulator/results/sim_simple/data/run_average/failure_transactions_chain_1.json', 'r') as f:
            chain_1_data = json.load(f)
        with open('simulator/results/sim_simple/data/run_average/failure_transactions_chain_2.json', 'r') as f:
            chain_2_data = json.load(f)
        chain_1_blocks = [entry['height'] for entry in chain_1_data['chain_1_failure']]
        chain_1_failure = [entry['count'] for entry in chain_1_data['chain_1_failure']]
        chain_2_blocks = [entry['height'] for entry in chain_2_data['chain_2_failure']]
        chain_2_failure = [entry['count'] for entry in chain_2_data['chain_2_failure']]
        
        # Load CAT and regular failure data for chain 1
        try:
            with open('simulator/results/sim_simple/data/run_average/cat_failure_transactions_chain_1.json', 'r') as f:
                chain_1_cat_data = json.load(f)
            chain_1_cat_failure = [entry['count'] for entry in chain_1_cat_data['chain_1_cat_failure']]
        except:
            chain_1_cat_failure = [0] * len(chain_1_blocks)
        
        try:
            with open('simulator/results/sim_simple/data/run_average/regular_failure_transactions_chain_1.json', 'r') as f:
                chain_1_regular_data = json.load(f)
            chain_1_regular_failure = [entry['count'] for entry in chain_1_regular_data['chain_1_regular_failure']]
        except:
            chain_1_regular_failure = [0] * len(chain_1_blocks)
        
        # Load CAT and regular failure data for chain 2
        try:
            with open('simulator/results/sim_simple/data/run_average/cat_failure_transactions_chain_2.json', 'r') as f:
                chain_2_cat_data = json.load(f)
            chain_2_cat_failure = [entry['count'] for entry in chain_2_cat_data['chain_2_cat_failure']]
        except:
            chain_2_cat_failure = [0] * len(chain_2_blocks)
        
        try:
            with open('simulator/results/sim_simple/data/run_average/regular_failure_transactions_chain_2.json', 'r') as f:
                chain_2_regular_data = json.load(f)
            chain_2_regular_failure = [entry['count'] for entry in chain_2_regular_data['chain_2_regular_failure']]
        except:
            chain_2_regular_failure = [0] * len(chain_2_blocks)
        
        # Plot chain 1 only with CAT and regular breakdown
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_failure, 'b-', label='Total', linewidth=2)
        plt.plot(chain_1_blocks, chain_1_cat_failure, 'r-', label='CAT', linewidth=1.5)
        plt.plot(chain_1_blocks, chain_1_regular_failure, 'g-', label='Regular', linewidth=1.5)
        plt.title('Failure Transactions by Height (Chain 1)')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Failure Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_failure_chain1.png')
        plt.close()
        # Plot chain 2 only with CAT and regular breakdown
        plt.figure(figsize=(12, 6))
        plt.plot(chain_2_blocks, chain_2_failure, 'b-', label='Total', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_cat_failure, 'r-', label='CAT', linewidth=1.5)
        plt.plot(chain_2_blocks, chain_2_regular_failure, 'g-', label='Regular', linewidth=1.5)
        plt.title('Failure Transactions by Height (Chain 2)')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Failure Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_failure_chain2.png')
        plt.close()
        # Plot both chains together (original combined plot)
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_failure, 'b-', label='Chain 1')
        plt.plot(chain_2_blocks, chain_2_failure, 'r--', label='Chain 2')
        plt.title('Failure Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Failure Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_failure_all.png')
        plt.close()
        
        # Plot CAT failure transactions only (combined from both chains)
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_cat_failure, 'b-', label='Chain 1 CAT', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_cat_failure, 'r--', label='Chain 2 CAT', linewidth=2)
        plt.title('CAT Failure Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of CAT Failure Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_failure_cat.png')
        plt.close()
        
        # Plot regular failure transactions only (combined from both chains)
        plt.figure(figsize=(12, 6))
        plt.plot(chain_1_blocks, chain_1_regular_failure, 'b-', label='Chain 1 Regular', linewidth=2)
        plt.plot(chain_2_blocks, chain_2_regular_failure, 'r--', label='Chain 2 Regular', linewidth=2)
        plt.title('Regular Failure Transactions by Height')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Regular Failure Transactions')
        plt.xlim(left=0)
        plt.grid(True)
        plt.legend()
        plt.savefig('simulator/results/sim_simple/figs/tx_count_failure_regular.png')
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
        # Handle both old and new parameter names
        if 'duration_seconds' in params:
            f.write(f"Duration (seconds): {params['duration_seconds']}\n")
        if 'sim_total_block_number' in params:
            f.write(f"Total Blocks: {params['sim_total_block_number']}\n")
        f.write(f"Zipf Parameter: {params['zipf_parameter']}\n")
        f.write(f"CAT Ratio: {params['ratio_cats']}\n")
        f.write(f"Block Interval: {params['block_interval']}\n")
        f.write(f"Chain Delays: {params['chain_delays']}\n")
