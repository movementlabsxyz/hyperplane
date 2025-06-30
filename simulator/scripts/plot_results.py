#!/usr/bin/env python3

import os
from plot_account_selection import plot_account_selection
from plot_miscellaneous import (
    plot_pending_transactions,
    plot_success_transactions,
    plot_failure_transactions,
    plot_parameters,
)

def main():
    # Create results directory if it doesn't exist
    os.makedirs('simulator/results/figs', exist_ok=True)
    
    # Plot account selection distributions
    plot_account_selection()
    
    # Plot pending transactions
    plot_pending_transactions()
    
    # Plot success transactions
    plot_success_transactions()
    
    # Plot failure transactions
    plot_failure_transactions()
    
    # Plot simulation parameters
    plot_parameters()

if __name__ == "__main__":
    main() 