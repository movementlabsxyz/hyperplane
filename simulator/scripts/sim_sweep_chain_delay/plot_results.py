#!/usr/bin/env python3
"""
Plot results for chain delay sweep simulation.
"""

import json
import matplotlib.pyplot as plt
import numpy as np
import os
from pathlib import Path

def load_sweep_data():
    """Load the combined results from the JSON file."""
    results_path = Path("simulator/results/sim_sweep_chain_delay/data/sweep_results.json")
    if not results_path.exists():
        print(f"Results file not found: {results_path}")
        return None
    
    with open(results_path, 'r') as f:
        return json.load(f)

def create_color_gradient(num_simulations):
    """Create a color gradient from red (0) to blue (max)"""
    colors = plt.cm.RdYlBu_r(np.linspace(0, 1, num_simulations))
    return colors

def plot_pending_transactions_overlay():
    """Plot pending transactions for all simulations with color gradient"""
    try:
        data = load_sweep_data()
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping pending transactions plot")
            return
        
        # Create figure
        plt.figure(figsize=(12, 8))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Plot each simulation's chain 1 pending transactions
        for i, result in enumerate(individual_results):
            chain_delay = result['chain_delay']
            chain_1_pending = result['chain_1_pending']
            
            if not chain_1_pending:
                continue
                
            # Extract data - chain_1_pending is a list of tuples (height, count)
            heights = [entry[0] for entry in chain_1_pending]
            counts = [entry[1] for entry in chain_1_pending]
            
            # Plot with color based on chain delay
            plt.plot(heights, counts, color=colors[i], alpha=0.7, 
                    label=f'Chain delay: {chain_delay:.1f}s', linewidth=1.5)
        
        plt.title('Pending Transactions by Height (Chain 1) - Chain Delay Sweep')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Pending Transactions')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        plt.savefig('simulator/results/sim_sweep_chain_delay/figs/pending_transactions_overlay.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing pending transactions data: {e}")
        return

def plot_success_transactions_overlay():
    """Plot success transactions for all simulations with color gradient"""
    try:
        data = load_sweep_data()
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping success transactions plot")
            return
        
        # Create figure
        plt.figure(figsize=(12, 8))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Plot each simulation's chain 1 success transactions
        for i, result in enumerate(individual_results):
            chain_delay = result['chain_delay']
            chain_1_success = result['chain_1_success']
            
            if not chain_1_success:
                continue
                
            # Extract data - chain_1_success is a list of tuples (height, count)
            heights = [entry[0] for entry in chain_1_success]
            counts = [entry[1] for entry in chain_1_success]
            
            # Plot with color based on chain delay
            plt.plot(heights, counts, color=colors[i], alpha=0.7, 
                    label=f'Chain delay: {chain_delay:.1f}s', linewidth=1.5)
        
        plt.title('Success Transactions by Height (Chain 1) - Chain Delay Sweep')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Success Transactions')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        plt.savefig('simulator/results/sim_sweep_chain_delay/figs/success_transactions_overlay.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing success transactions data: {e}")
        return

def plot_failure_transactions_overlay():
    """Plot failure transactions for all simulations with color gradient"""
    try:
        data = load_sweep_data()
        individual_results = data['individual_results']
        
        if not individual_results:
            print("Warning: No individual results found, skipping failure transactions plot")
            return
        
        # Create figure
        plt.figure(figsize=(12, 8))
        
        # Create color gradient
        colors = create_color_gradient(len(individual_results))
        
        # Plot each simulation's chain 1 failure transactions
        for i, result in enumerate(individual_results):
            chain_delay = result['chain_delay']
            chain_1_failure = result['chain_1_failure']
            
            if not chain_1_failure:
                continue
                
            # Extract data - chain_1_failure is a list of tuples (height, count)
            heights = [entry[0] for entry in chain_1_failure]
            counts = [entry[1] for entry in chain_1_failure]
            
            # Plot with color based on chain delay
            plt.plot(heights, counts, color=colors[i], alpha=0.7, 
                    label=f'Chain delay: {chain_delay:.1f}s', linewidth=1.5)
        
        plt.title('Failure Transactions by Height (Chain 1) - Chain Delay Sweep')
        plt.xlabel('Block Height')
        plt.ylabel('Number of Failure Transactions')
        plt.xlim(left=0)
        plt.grid(True, alpha=0.3)
        plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        plt.savefig('simulator/results/sim_sweep_chain_delay/figs/failure_transactions_overlay.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing failure transactions data: {e}")
        return

def plot_transaction_status_chart(ax, data):
    """Create a line chart showing failed/success/pending data vs chain delay"""
    try:
        individual_results = data['individual_results']
        
        if not individual_results:
            return
        
        # Extract data for the chart
        chain_delays = []
        success_counts = []
        failure_counts = []
        pending_counts = []
        
        for result in individual_results:
            chain_delays.append(result['chain_delay'])
            
            # Calculate total success, failure, and pending from chain_1 data
            success_total = sum(count for _, count in result['chain_1_success'])
            failure_total = sum(count for _, count in result['chain_1_failure'])
            pending_total = sum(count for _, count in result['chain_1_pending'])
            
            success_counts.append(success_total)
            failure_counts.append(failure_total)
            pending_counts.append(pending_total)
        
        # Create the line chart
        ax.plot(chain_delays, success_counts, 'go-', linewidth=2, markersize=6, label='Success')
        ax.plot(chain_delays, failure_counts, 'ro-', linewidth=2, markersize=6, label='Failed')
        ax.plot(chain_delays, pending_counts, 'yo-', linewidth=2, markersize=6, label='Pending')
        
        ax.set_title('Transaction Status vs Chain Delay')
        ax.set_xlabel('Chain-2 Delay (seconds)')
        ax.set_ylabel('Number of Transactions')
        ax.legend()
        ax.grid(True, alpha=0.3)
        ax.set_ylim(bottom=0)
        
    except (KeyError, IndexError) as e:
        print(f"Warning: Error creating transaction status chart: {e}")
        ax.text(0.5, 0.5, 'Error creating chart', ha='center', va='center', transform=ax.transAxes)
        ax.axis('off')

def plot_sweep_summary():
    """Plot summary statistics across all chain delays"""
    try:
        data = load_sweep_data()
        sweep_summary = data['sweep_summary']
        
        if not sweep_summary:
            print("Warning: No sweep summary found, skipping summary plot")
            return
        
        chain_delays = sweep_summary['chain_delays']
        total_transactions = sweep_summary['total_transactions']
        cat_transactions = sweep_summary['cat_transactions']
        regular_transactions = sweep_summary['regular_transactions']
        
        # Create subplots
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 10))
        
        # Plot 1: Total transactions
        ax1.plot(chain_delays, total_transactions, 'bo-', linewidth=2, markersize=6)
        ax1.set_title('Total Transactions vs Chain Delay')
        ax1.set_xlabel('Chain-2 Delay (seconds)')
        ax1.set_ylabel('Total Transactions')
        ax1.grid(True, alpha=0.3)
        
        # Plot 2: CAT transactions
        ax2.plot(chain_delays, cat_transactions, 'ro-', linewidth=2, markersize=6)
        ax2.set_title('CAT Transactions vs Chain Delay')
        ax2.set_xlabel('Chain-2 Delay (seconds)')
        ax2.set_ylabel('CAT Transactions')
        ax2.grid(True, alpha=0.3)
        ax2.set_ylim(bottom=0)
        
        # Plot 3: Transaction status chart
        plot_transaction_status_chart(ax3, data)
        
        # Plot 4: Transaction type distribution (line chart)
        ax4.plot(chain_delays, cat_transactions, 'ro-', linewidth=2, markersize=6, label='CAT Transactions')
        ax4.plot(chain_delays, regular_transactions, 'go-', linewidth=2, markersize=6, label='Regular Transactions')
        ax4.set_title('Transaction Distribution by Chain Delay')
        ax4.set_xlabel('Chain-2 Delay (seconds)')
        ax4.set_ylabel('Number of Transactions')
        ax4.legend()
        ax4.grid(True, alpha=0.3)
        ax4.set_ylim(bottom=0)
        
        plt.tight_layout()
        plt.savefig('simulator/results/sim_sweep_chain_delay/figs/sweep_summary.png', 
                   dpi=300, bbox_inches='tight')
        plt.close()
        
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Error processing sweep summary data: {e}")
        return

def main():
    # Create results directory if it doesn't exist
    os.makedirs('simulator/results/sim_sweep_chain_delay/figs', exist_ok=True)
    
    print("Generating sweep chain delay simulation plots...")
    
    # Plot transaction overlays
    plot_pending_transactions_overlay()
    plot_success_transactions_overlay()
    plot_failure_transactions_overlay()
    
    # Plot sweep summary
    plot_sweep_summary()
    
    print("Sweep chain delay simulation plots generated successfully!")

if __name__ == "__main__":
    main() 