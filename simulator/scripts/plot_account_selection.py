import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.stats import zipf

def plot_account_selection():
    # Load simulation parameters
    with open('simulator/results/data/simulation_stats.json', 'r') as f:
        sim_stats = json.load(f)
    
    # Get parameters
    zipf_param = sim_stats['parameters']['zipf_parameter']
    num_accounts = sim_stats['parameters']['num_accounts']
    
    # Load sender selection data
    with open('simulator/results/data/account_sender_selection.json', 'r') as f:
        sender_data = json.load(f)
    
    # Load receiver selection data
    with open('simulator/results/data/account_receiver_selection.json', 'r') as f:
        receiver_data = json.load(f)
    
    # Extract data
    sender_accounts = [entry['account'] for entry in sender_data['sender_selection']]
    sender_counts = [entry['transactions'] for entry in sender_data['sender_selection']]
    
    receiver_accounts = [entry['account'] for entry in receiver_data['receiver_selection']]
    receiver_counts = [entry['transactions'] for entry in receiver_data['receiver_selection']]
    
    # Plot sender selection (should be uniform)
    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(10, 15))
    
    # Linear-linear scale
    ax1.scatter(sender_accounts, sender_counts, alpha=0.6, label='Actual Distribution')
    ax1.set_title('Sender Account Selection Distribution (Linear-Linear)')
    ax1.set_xlabel('Account ID')
    ax1.set_ylabel('Number of Transactions')
    ax1.legend()
    ax1.grid(True)
    
    # Log-linear scale
    ax2.scatter(sender_accounts, sender_counts, alpha=0.6, label='Actual Distribution')
    ax2.set_title('Sender Account Selection Distribution (Log-Linear)')
    ax2.set_xlabel('Account ID')
    ax2.set_ylabel('Number of Transactions')
    ax2.set_yscale('log')
    ax2.legend()
    ax2.grid(True)
    
    # Log-log scale
    ax3.scatter(sender_accounts, sender_counts, alpha=0.6, label='Actual Distribution')
    ax3.set_title('Sender Account Selection Distribution (Log-Log)')
    ax3.set_xlabel('Account ID')
    ax3.set_ylabel('Number of Transactions')
    ax3.set_xscale('log')
    ax3.set_yscale('log')
    ax3.legend()
    ax3.grid(True)
    
    plt.tight_layout()
    plt.savefig('simulator/results/figs/account_sender_selection.png')
    plt.close()
    
    # Plot receiver selection (should follow Zipf)
    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(10, 15))
    
    # Calculate theoretical Zipf distribution for all accounts
    theoretical_accounts = list(range(1, num_accounts + 1))
    zipf_weights = [1.0 / (i ** zipf_param) for i in range(1, num_accounts + 1)]
    total_weight = sum(zipf_weights)
    total_transactions = sum(receiver_counts)
    theoretical_counts = [weight * total_transactions / total_weight for weight in zipf_weights]
    
    # Linear-linear scale
    ax1.scatter(receiver_accounts, receiver_counts, alpha=0.6, label='Actual Distribution')
    ax1.plot(theoretical_accounts, theoretical_counts, 'r--', label='Theoretical Zipf')
    ax1.set_title('Receiver Account Selection Distribution (Linear-Linear)')
    ax1.set_xlabel('Account ID')
    ax1.set_ylabel('Number of Transactions')
    ax1.legend()
    ax1.grid(True)
    
    # Log-linear scale
    ax2.scatter(receiver_accounts, receiver_counts, alpha=0.6, label='Actual Distribution')
    ax2.plot(theoretical_accounts, theoretical_counts, 'r--', label='Theoretical Zipf')
    ax2.set_title('Receiver Account Selection Distribution (Log-Linear)')
    ax2.set_xlabel('Account ID')
    ax2.set_ylabel('Number of Transactions')
    ax2.set_yscale('log')
    ax2.legend()
    ax2.grid(True)
    
    # Log-log scale
    ax3.scatter(receiver_accounts, receiver_counts, alpha=0.6, label='Actual Distribution')
    ax3.plot(theoretical_accounts, theoretical_counts, 'r--', label='Theoretical Zipf')
    ax3.set_title('Receiver Account Selection Distribution (Log-Log)')
    ax3.set_xlabel('Account ID')
    ax3.set_ylabel('Number of Transactions')
    ax3.set_xscale('log')
    ax3.set_yscale('log')
    ax3.legend()
    ax3.grid(True)
    
    plt.tight_layout()
    plt.savefig('simulator/results/figs/account_receiver_selection.png')
    plt.close()

if __name__ == '__main__':
    plot_account_selection() 