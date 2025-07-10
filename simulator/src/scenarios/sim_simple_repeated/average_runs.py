#!/usr/bin/env python3

import os
import json
import glob
from collections import defaultdict
import numpy as np

def load_metadata():
    """Load metadata to get number of runs and parameters."""
    try:
        with open('../../../results/sim_simple_repeated/data/metadata.json', 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print("Error: metadata.json not found. Cannot determine number of runs.")
        return None

def load_run_data(run_dir):
    """Load all data files from a single run directory."""
    run_data = {}
    
    # The actual data files are in run_X/data/
    data_dir = os.path.join(run_dir, 'data')
    if not os.path.exists(data_dir):
        print(f"Warning: No data directory found in {run_dir}")
        return run_data
    
    # Load all JSON files in the data directory
    for filename in os.listdir(data_dir):
        if filename.endswith('.json'):
            filepath = os.path.join(data_dir, filename)
            try:
                with open(filepath, 'r') as f:
                    data = json.load(f)
                    run_data[filename] = data
            except Exception as e:
                print(f"Warning: Could not load {filepath}: {e}")
    
    return run_data

def average_time_series_data(all_runs_data, key_name):
    """Average time series data across all runs."""
    if not all_runs_data:
        return []
    
    # Collect all data points by block height
    height_data = defaultdict(list)
    
    for run_data in all_runs_data:
        # Find the file that contains this key_name
        for filename, file_data in run_data.items():
            if key_name in file_data:
                for entry in file_data[key_name]:
                    height = entry['height']
                    count = entry['count']
                    height_data[height].append(count)
                break  # Found the file, no need to check others
    
    # Calculate averages and sort by block height
    averaged_data = []
    for height in sorted(height_data.keys()):
        counts = height_data[height]
        avg_count = np.mean(counts)
        averaged_data.append({
            'height': height,
            'count': avg_count
        })
    
    return averaged_data


def average_scalar_values(all_runs_data, key_path):
    """Average scalar values across all runs."""
    if not all_runs_data:
        return 0.0
    
    values = []
    for run_data in all_runs_data:
        if 'simulation_stats.json' in run_data:
            stats = run_data['simulation_stats.json']
            # Navigate the nested structure
            current = stats
            for key in key_path:
                if key in current:
                    current = current[key]
                else:
                    current = 0
                    break
            values.append(float(current))
    
    return np.mean(values) if values else 0.0


def average_account_selection_data(all_runs_data):
    """Average account selection statistics across all runs."""
    if not all_runs_data:
        return {}, {}
    
    sender_stats = defaultdict(list)
    receiver_stats = defaultdict(list)
    
    for run_data in all_runs_data:
        # Average sender selection data
        if 'account_sender_selection.json' in run_data:
            sender_data = run_data['account_sender_selection.json']
            # Handle old format with sender_selection array
            if 'sender_selection' in sender_data:
                for entry in sender_data['sender_selection']:
                    account_id = entry['account']
                    count = entry['transactions']
                    sender_stats[account_id].append(count)
            else:
                # Handle new format with direct key-value pairs
                for account_id, count in sender_data.items():
                    sender_stats[account_id].append(count)
        
        # Average receiver selection data
        if 'account_receiver_selection.json' in run_data:
            receiver_data = run_data['account_receiver_selection.json']
            # Handle old format with receiver_selection array
            if 'receiver_selection' in receiver_data:
                for entry in receiver_data['receiver_selection']:
                    account_id = entry['account']
                    count = entry['transactions']
                    receiver_stats[account_id].append(count)
            else:
                # Handle new format with direct key-value pairs
                for account_id, count in receiver_data.items():
                    receiver_stats[account_id].append(count)
    
    # Calculate averages
    avg_sender = {account_id: np.mean(counts) for account_id, counts in sender_stats.items()}
    avg_receiver = {account_id: np.mean(counts) for account_id, counts in receiver_stats.items()}
    
    return avg_sender, avg_receiver

def create_averaged_data():
    """Create averaged data from all individual runs."""
    metadata = load_metadata()
    if not metadata:
        return False
    
    num_runs = metadata['num_runs']
    base_dir = '../../../results/sim_simple_repeated/data'
    
    print(f"Averaging data from {num_runs} runs...")
    
    # Load data from all runs
    all_runs_data = []
    for run_num in range(num_runs):
        run_dir = os.path.join(base_dir, f'run_{run_num}')
        if os.path.exists(run_dir):
            run_data = load_run_data(run_dir)
            if run_data:
                all_runs_data.append(run_data)
                print(f"  Loaded data from run_{run_num}")
        else:
            print(f"  Warning: run_{run_num} directory not found")
    
    if not all_runs_data:
        print("Error: No run data found to average.")
        return False
    
    # Create run_average directory
    avg_dir = os.path.join(base_dir, 'run_average')
    os.makedirs(avg_dir, exist_ok=True)
    
    # Average simulation statistics
    print("Averaging simulation statistics...")
    avg_stats = {
        'parameters': metadata['parameters'],
        'averaging_info': {
            'num_runs': len(all_runs_data),
            'note': 'Results are averaged across multiple simulation runs'
        },
        'results': {
            'total_transactions': average_scalar_values(all_runs_data, ['results', 'total_transactions']),
            'cat_transactions': average_scalar_values(all_runs_data, ['results', 'cat_transactions']),
            'regular_transactions': average_scalar_values(all_runs_data, ['results', 'regular_transactions'])
        }
    }
    
    with open(os.path.join(avg_dir, 'simulation_stats.json'), 'w') as f:
        json.dump(avg_stats, f, indent=2)
    
    # Average time series data
    print("Averaging time series data...")
    time_series_files = [
        ('pending_transactions_chain_1.json', 'chain_1_pending'),
        ('pending_transactions_chain_2.json', 'chain_2_pending'),
        ('success_transactions_chain_1.json', 'chain_1_success'),
        ('success_transactions_chain_2.json', 'chain_2_success'),
        ('failure_transactions_chain_1.json', 'chain_1_failure'),
        ('failure_transactions_chain_2.json', 'chain_2_failure'),
        ('cat_pending_transactions_chain_1.json', 'chain_1_cat_pending'),
        ('cat_pending_transactions_chain_2.json', 'chain_2_cat_pending'),
        ('cat_success_transactions_chain_1.json', 'chain_1_cat_success'),
        ('cat_success_transactions_chain_2.json', 'chain_2_cat_success'),
        ('cat_failure_transactions_chain_1.json', 'chain_1_cat_failure'),
        ('cat_failure_transactions_chain_2.json', 'chain_2_cat_failure'),
        ('regular_pending_transactions_chain_1.json', 'chain_1_regular_pending'),
        ('regular_pending_transactions_chain_2.json', 'chain_2_regular_pending'),
        ('regular_success_transactions_chain_1.json', 'chain_1_regular_success'),
        ('regular_success_transactions_chain_2.json', 'chain_2_regular_success'),
        ('regular_failure_transactions_chain_1.json', 'chain_1_regular_failure'),
        ('regular_failure_transactions_chain_2.json', 'chain_2_regular_failure'),
        ('locked_keys_chain_1.json', 'chain_1_locked_keys'),
        ('locked_keys_chain_2.json', 'chain_2_locked_keys')
    ]
    
    for filename, key_name in time_series_files:
        averaged_data = average_time_series_data(all_runs_data, key_name)
        if averaged_data:
            output_data = {key_name: averaged_data}
            with open(os.path.join(avg_dir, filename), 'w') as f:
                json.dump(output_data, f, indent=2)
    
    # Average account selection data
    print("Averaging account selection data...")
    avg_sender, avg_receiver = average_account_selection_data(all_runs_data)
    
    with open(os.path.join(avg_dir, 'account_sender_selection.json'), 'w') as f:
        json.dump(avg_sender, f, indent=2)
    
    with open(os.path.join(avg_dir, 'account_receiver_selection.json'), 'w') as f:
        json.dump(avg_receiver, f, indent=2)
    
    print(f"Averaging complete! Averaged data saved to {avg_dir}/")
    print(f"Averaged {len(all_runs_data)} runs with float precision.")
    
    return True

def main():
    """Main function to run the averaging process."""
    if not os.path.exists('../../../results/sim_simple_repeated/data/metadata.json'):
        print("Error: No repeated simulation data found.")
        print("Please run the simple repeated simulation first.")
        return
    
    success = create_averaged_data()
    if success:
        print("\nNext steps:")
        print("1. The existing plotting scripts will now work with the averaged data")
        print("2. Run the plotting script to generate figures from the averaged data")
        print("3. Individual run data is preserved in run_0/, run_1/, etc. for detailed analysis")

if __name__ == "__main__":
    main() 