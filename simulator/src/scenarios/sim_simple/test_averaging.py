#!/usr/bin/env python3
"""
Test script for Simple Repeated Simulation Averaging Workflow

This script validates that all components of the averaging workflow are properly set up:
- Checks if metadata.json exists (indicates simulation was run)
- Verifies individual run directories exist (run_0/, run_1/, etc.)
- Confirms averaging and plotting scripts are present
- Validates that averaged data files exist (if averaging was run)

Use this script to diagnose issues with the averaging workflow or to verify
that everything is set up correctly before running plots.

Usage: python3 test_averaging.py
"""

import os
import sys

def test_averaging_workflow():
    """Test the complete averaging workflow."""
    print("Testing Simple Repeated Simulation Averaging Workflow")
    print("=" * 60)
    
    # Check if metadata exists
    metadata_path = 'simulator/results/sim_simple/data/metadata.json'
    if not os.path.exists(metadata_path):
        print("❌ No metadata.json found. Please run the simple repeated simulation first.")
        return False
    
    print("✅ Found metadata.json")
    
    # Check if individual run directories exist
    base_dir = 'simulator/results/sim_simple/data'
    run_dirs = [d for d in os.listdir(base_dir) if d.startswith('run_') and os.path.isdir(os.path.join(base_dir, d))]
    
    if not run_dirs:
        print("❌ No individual run directories found. Please run the simple repeated simulation first.")
        return False
    
    print(f"✅ Found {len(run_dirs)} individual run directories: {', '.join(run_dirs)}")
    
    # Check if averaging script exists
    if not os.path.exists('average_runs.py'):
        print("❌ average_runs.py not found.")
        return False
    
    print("✅ Found average_runs.py")
    
    # Check if plotting scripts exist
    required_scripts = ['plot_results.py', 'plot_miscellaneous.py', 'plot_account_selection.py']
    for script in required_scripts:
        if not os.path.exists(script):
            print(f"❌ {script} not found.")
            return False
    
    print("✅ All plotting scripts found")
    
    # Check if run_average directory exists (after running averaging)
    avg_dir = os.path.join(base_dir, 'run_average')
    if os.path.exists(avg_dir):
        print("✅ run_average directory exists")
        
        # Check if averaged data files exist
        required_files = [
            'simulation_stats.json',
            'pending_transactions_chain_1.json',
            'success_transactions_chain_1.json',
            'failure_transactions_chain_1.json'
        ]
        
        missing_files = []
        for file in required_files:
            if not os.path.exists(os.path.join(avg_dir, file)):
                missing_files.append(file)
        
        if missing_files:
            print(f"❌ Missing averaged data files: {', '.join(missing_files)}")
            return False
        
        print("✅ All required averaged data files exist")
    else:
        print("ℹ️  run_average directory not found. Run 'python3 average_runs.py' to create it.")
    
    print("\n📋 Workflow Summary:")
    print("1. ✅ Run simple repeated simulation (creates run_0/, run_1/, etc.)")
    print("2. ℹ️  Run 'python3 average_runs.py' (creates run_average/ with averaged data)")
    print("3. ℹ️  Run 'python3 plot_results.py' (generates plots from averaged data)")
    
    return True

if __name__ == "__main__":
    success = test_averaging_workflow()
    if success:
        print("\n🎉 All tests passed! The averaging workflow is ready to use.")
    else:
        print("\n❌ Some tests failed. Please check the issues above.")
        sys.exit(1) 