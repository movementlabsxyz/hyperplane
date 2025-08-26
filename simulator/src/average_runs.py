#!/usr/bin/env python3

import os
import sys
import json
import glob
from collections import defaultdict
import numpy as np
import shutil

def load_metadata(results_dir):
    """Load metadata to get number of runs and parameters."""
    try:
        metadata_path = os.path.join(results_dir, 'data', 'metadata.json')
        with open(metadata_path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"Error: metadata.json not found in {results_dir}/data/. Cannot determine number of runs.")
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
                    # Handle different value field names
                    if 'count' in entry:
                        value = entry['count']
                    elif 'bytes' in entry:
                        value = entry['bytes']
                    elif 'percent' in entry:
                        value = entry['percent']
                    elif 'latency' in entry:
                        value = entry['latency']
                    else:
                        # Skip entries without expected value field
                        continue
                    height_data[height].append(value)
                break  # Found the file, no need to check others
    
    # Calculate averages and sort by block height
    averaged_data = []
    for height in sorted(height_data.keys()):
        values = height_data[height]
        avg_value = np.mean(values)
        
        # Use the same field name as the original data
        if key_name == 'system_memory':
            averaged_data.append({
                'height': height,
                'bytes': avg_value
            })
        elif key_name == 'system_cpu':
            averaged_data.append({
                'height': height,
                'percent': avg_value
            })
        elif key_name == 'system_total_cpu':
            averaged_data.append({
                'height': height,
                'percent': avg_value
            })
        elif key_name == 'system_total_memory':
            averaged_data.append({
                'height': height,
                'bytes': avg_value
            })
        elif 'latency' in key_name:
            averaged_data.append({
                'height': height,
                'latency': avg_value
            })
        else:
            averaged_data.append({
                'height': height,
                'count': avg_value
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

def create_averaged_data(results_dir):
    """Create averaged data from all individual runs for all simulations."""
    metadata = load_metadata(results_dir)
    if not metadata:
        return False
    
    num_runs = metadata['num_runs']
    num_simulations = metadata.get('num_simulations', 1)  # Default to 1 for simple simulations
    base_dir = os.path.join(results_dir, 'data')
    
    # Process each simulation (for simple: only sim_0, for sweep: sim_0, sim_1, etc.)
    for sim_index in range(num_simulations):
        sim_dir = os.path.join(base_dir, f'sim_{sim_index}')
        
        # Load data from all runs for this simulation
        all_runs_data = []
        for run_num in range(num_runs):
            run_dir = os.path.join(sim_dir, f'run_{run_num}')
            if os.path.exists(run_dir):
                run_data = load_run_data(run_dir)
                if run_data:
                    all_runs_data.append(run_data)
                else:
                    print(f"[Averaging] Warning: No data loaded from {run_dir}")
            else:
                print(f"[Averaging] Missing run directory: {run_dir}")
        
        if not all_runs_data:
            print(f"[Averaging] Error: No run data found to average for simulation {sim_index}.")
            return False

        # If only one run, just copy its data to run_average
        if len(all_runs_data) == 1:
            avg_dir = os.path.join(sim_dir, 'run_average')
            os.makedirs(avg_dir, exist_ok=True)
            single_run_dir = os.path.join(sim_dir, 'run_0', 'data')
            for filename in os.listdir(single_run_dir):
                src = os.path.join(single_run_dir, filename)
                dst = os.path.join(avg_dir, filename)
                shutil.copy2(src, dst)
            continue

        # Create run_average directory for this simulation
        avg_dir = os.path.join(sim_dir, 'run_average')
        os.makedirs(avg_dir, exist_ok=True)
        
        # Average simulation statistics
        avg_stats = {
            'simulation_index': sim_index,
            'averaging_info': {
                'num_runs': len(all_runs_data),
                'note': 'Results are averaged across multiple simulation runs'
            },
            'parameters': all_runs_data[0]['simulation_stats.json']['parameters'],  # Copy parameters from first run
            'results': {
                'total_transactions': average_scalar_values(all_runs_data, ['results', 'total_transactions']),
                'cat_transactions': average_scalar_values(all_runs_data, ['results', 'cat_transactions']),
                'regular_transactions': average_scalar_values(all_runs_data, ['results', 'regular_transactions'])
            }
        }
        
        stats_path = os.path.join(avg_dir, 'simulation_stats.json')
        with open(stats_path, 'w') as f:
            json.dump(avg_stats, f, indent=2)
        
        # Average time series data
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
            ('cat_pending_resolving_transactions_chain_1.json', 'chain_1_cat_pending_resolving'),
            ('cat_pending_resolving_transactions_chain_2.json', 'chain_2_cat_pending_resolving'),
            ('cat_pending_postponed_transactions_chain_1.json', 'chain_1_cat_pending_postponed'),
            ('cat_pending_postponed_transactions_chain_2.json', 'chain_2_cat_pending_postponed'),
            ('regular_pending_transactions_chain_1.json', 'chain_1_regular_pending'),
            ('regular_pending_transactions_chain_2.json', 'chain_2_regular_pending'),
            ('regular_success_transactions_chain_1.json', 'chain_1_regular_success'),
            ('regular_success_transactions_chain_2.json', 'chain_2_regular_success'),
            ('regular_failure_transactions_chain_1.json', 'chain_1_regular_failure'),
            ('regular_failure_transactions_chain_2.json', 'chain_2_regular_failure'),
            ('locked_keys_chain_1.json', 'chain_1_locked_keys'),
            ('locked_keys_chain_2.json', 'chain_2_locked_keys'),
            ('tx_per_block_chain_1.json', 'chain_1_tx_per_block'),
            ('tx_per_block_chain_2.json', 'chain_2_tx_per_block'),
            ('regular_tx_avg_latency_chain_1.json', 'chain_1_regular_tx_avg_latency'),
            ('regular_tx_avg_latency_chain_2.json', 'chain_2_regular_tx_avg_latency'),
            ('regular_tx_max_latency_chain_1.json', 'chain_1_regular_tx_max_latency'),
            ('regular_tx_max_latency_chain_2.json', 'chain_2_regular_tx_max_latency'),
            ('regular_tx_finalized_count_chain_1.json', 'chain_1_regular_tx_finalized_count'),
            ('regular_tx_finalized_count_chain_2.json', 'chain_2_regular_tx_finalized_count'),
            ('system_memory.json', 'system_memory'),
            ('system_total_memory.json', 'system_total_memory'),
            ('system_cpu.json', 'system_cpu'),
            ('system_total_cpu.json', 'system_total_cpu'),
            ('loop_steps_without_tx_issuance.json', 'loop_steps_without_tx_issuance'),
        ]
        
        for filename, key_name in time_series_files:
            averaged_data = average_time_series_data(all_runs_data, key_name)
            if averaged_data:
                output_data = {key_name: averaged_data}
                output_file = os.path.join(avg_dir, filename)
                with open(output_file, 'w') as f:
                    json.dump(output_data, f, indent=2)
        
        # Average account selection data
        avg_sender, avg_receiver = average_account_selection_data(all_runs_data)
        
        sender_path = os.path.join(avg_dir, 'account_sender_selection.json')
        receiver_path = os.path.join(avg_dir, 'account_receiver_selection.json')
        with open(sender_path, 'w') as f:
            json.dump(avg_sender, f, indent=2)
        with open(receiver_path, 'w') as f:
            json.dump(avg_receiver, f, indent=2)
        
        # No verbose output for completion
    return True

def main():
    """Main function to run the averaging process."""
    
    if len(sys.argv) != 2:
        print("Usage: python3 average_runs.py <results_dir>")
        print("Example: python3 average_runs.py ../../../results/sim_sweep_cat_ratio")
        return 1
    
    results_dir = sys.argv[1]
    
    try:
        success = create_averaged_data(results_dir)
        if not success:
            print("Averaging failed!")
            return 1
        return 0
    except Exception as e:
        print(f"[Averaging] Exception during averaging: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    exit(main()) 