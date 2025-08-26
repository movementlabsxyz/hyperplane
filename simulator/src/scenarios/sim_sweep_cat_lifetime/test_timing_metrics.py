#!/usr/bin/env python3
"""
Test script for timing metrics plotting

This script tests the timing metrics plotting functionality by creating
sample data and generating plots.
"""

import sys
import os
import json
import tempfile
import shutil

# Add the scripts directory to the Python path
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from plot_timing_metrics import plot_timing_metrics, load_sweep_data_from_run_average

def create_test_data():
    """Create test data structure for timing metrics."""
    # Create sample data structure
    test_data = {
        'individual_results': [
            {
                'cat_lifetime': 5,
                'chain_1_regular_tx_avg_latency': [(1, 10.5), (2, 12.3), (3, 15.7)],
                'chain_2_regular_tx_avg_latency': [(1, 8.2), (2, 9.1), (3, 11.4)],
                'chain_1_regular_tx_max_latency': [(1, 25.0), (2, 30.5), (3, 35.2)],
                'chain_2_regular_tx_max_latency': [(1, 20.1), (2, 22.8), (3, 28.9)],
                'chain_1_regular_tx_finalized_count': [(1, 5), (2, 12), (3, 18)],
                'chain_2_regular_tx_finalized_count': [(1, 4), (2, 10), (3, 15)]
            },
            {
                'cat_lifetime': 10,
                'chain_1_regular_tx_avg_latency': [(1, 8.1), (2, 9.5), (3, 12.8)],
                'chain_2_regular_tx_avg_latency': [(1, 6.3), (2, 7.8), (3, 9.2)],
                'chain_1_regular_tx_max_latency': [(1, 18.5), (2, 22.1), (3, 26.7)],
                'chain_2_regular_tx_max_latency': [(1, 15.2), (2, 18.9), (3, 21.4)],
                'chain_1_regular_tx_finalized_count': [(1, 8), (2, 16), (3, 24)],
                'chain_2_regular_tx_finalized_count': [(1, 6), (2, 14), (3, 20)]
            }
        ]
    }
    
    return test_data

def test_plotting():
    """Test the plotting functionality."""
    print("Testing timing metrics plotting...")
    
    # Create test data
    test_data = create_test_data()
    
    # Create temporary directory for test
    with tempfile.TemporaryDirectory() as temp_dir:
        # Create test results directory structure
        test_results_dir = os.path.join(temp_dir, 'test_results')
        os.makedirs(test_results_dir, exist_ok=True)
        
        # Test plotting
        try:
            plot_timing_metrics(
                test_data, 
                'cat_lifetime', 
                test_results_dir, 
                'Test CAT Lifetime'
            )
            print("✅ Plotting test passed!")
            
            # Check if plots were created
            figs_dir = os.path.join(test_results_dir, 'figs')
            if os.path.exists(figs_dir):
                plot_files = os.listdir(figs_dir)
                print(f"📊 Generated plots: {plot_files}")
            else:
                print("❌ Figures directory not created")
                
        except Exception as e:
            print(f"❌ Plotting test failed: {e}")
            return False
    
    return True

def test_data_loading():
    """Test the data loading functionality."""
    print("\nTesting data loading functionality...")
    
    # Create test data structure
    test_base_dir = tempfile.mkdtemp()
    try:
        # Create test directory structure
        test_dir = os.path.join(test_base_dir, 'test_sweep')
        os.makedirs(os.path.join(test_dir, 'data'), exist_ok=True)
        
        # Create metadata
        metadata = {
            'parameter_name': 'cat_lifetime',
            'parameter_values': [5, 10],
            'num_simulations': 2
        }
        
        with open(os.path.join(test_dir, 'data', 'metadata.json'), 'w') as f:
            json.dump(metadata, f)
        
        # Create test simulation data
        for sim_index, param_value in enumerate([5, 10]):
            sim_dir = os.path.join(test_dir, 'data', f'sim_{sim_index}', 'run_average')
            os.makedirs(sim_dir, exist_ok=True)
            
            # Create simulation stats
            stats = {
                'results': {
                    'total_transactions': 100 + sim_index * 50,
                    'cat_transactions': 20 + sim_index * 10,
                    'regular_transactions': 80 + sim_index * 40
                }
            }
            
            with open(os.path.join(sim_dir, 'simulation_stats.json'), 'w') as f:
                json.dump(stats, f)
            
            # Create timing metrics files
            timing_data = {
                'chain_1_regular_tx_avg_latency': [
                    {'height': 1, 'latency': 10.5 + sim_index},
                    {'height': 2, 'latency': 12.3 + sim_index}
                ]
            }
            
            with open(os.path.join(sim_dir, 'regular_tx_avg_latency_chain_1.json'), 'w') as f:
                json.dump(timing_data, f)
        
        # Test data loading
        try:
            data = load_sweep_data_from_run_average('test_sweep', test_base_dir)
            print("✅ Data loading test passed!")
            print(f"📊 Loaded {len(data['individual_results'])} simulation results")
            
            # Check data structure
            for i, result in enumerate(data['individual_results']):
                print(f"  Simulation {i}: cat_lifetime={result['cat_lifetime']}, "
                      f"total_tx={result['total_transactions']}")
                
        except Exception as e:
            print(f"❌ Data loading test failed: {e}")
            return False
            
    finally:
        shutil.rmtree(test_base_dir)
    
    return True

def main():
    """Run all tests."""
    print("🧪 Running timing metrics tests...\n")
    
    # Test plotting
    plotting_success = test_plotting()
    
    # Test data loading
    loading_success = test_data_loading()
    
    # Summary
    print("\n" + "="*50)
    if plotting_success and loading_success:
        print("🎉 All tests passed!")
        print("✅ Timing metrics plotting is working correctly")
        print("✅ Data loading is working correctly")
    else:
        print("❌ Some tests failed")
        if not plotting_success:
            print("❌ Plotting test failed")
        if not loading_success:
            print("❌ Data loading test failed")
    print("="*50)

if __name__ == "__main__":
    main()
