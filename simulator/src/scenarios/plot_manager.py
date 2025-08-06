"""
Plot Manager for Hyperplane Simulator

This module handles the organized generation of plots for sweep simulations,
following a specific order and structure.
"""

import os
from typing import Dict, Any
from plot_utils import (
    plot_sweep_summary, plot_sweep_locked_keys, plot_sweep_locked_keys_with_pending,
    plot_sweep_transactions_per_block, generate_individual_curves_plots,
    plot_transactions_overlay, plot_sweep_tpb_moving_average,
    plot_total_cat_transactions, plot_total_regular_transactions, plot_total_sumtypes_transactions
)
from plot_system import (
    plot_system_memory, plot_system_memory_total,
    plot_system_cpu, plot_system_cpu_filtered, plot_system_cpu_total,
    plot_cl_queue_length, plot_loops_steps_without_tx_issuance_and_cl_queue,
    plot_loop_steps_without_tx_issuance, plot_loop_steps_without_tx_issuance_moving_average
)
from plot_utils_delta import (
    plot_transactions_delta_overlay
)
from plot_utils_moving_average import (
    plot_transactions_overlay_with_moving_average, plot_transactions_delta_overlay_with_moving_average
)
from plot_utils_cutoff import (
    plot_transactions_cutoff_overlay, plot_transaction_percentage_cutoff
)
from plot_utils_percentage import (
    plot_transaction_percentage, plot_transaction_percentage_delta,
    plot_transaction_percentage_with_moving_average,
    plot_transaction_percentage_delta_with_moving_average
)

def generate_paper_plots(*args, **kwargs):
    # Paper plots are handled separately in plot_paper.py
    pass

# System performance and resource usage plots (CPU, memory, TPS, etc.)
def generate_system_plots(data, param_name, results_dir, sweep_type, plot_config):
    # print("Generating system plots...")
    plot_sweep_summary(data, param_name, results_dir, sweep_type)
    plot_sweep_locked_keys(data, param_name, results_dir, sweep_type)
    plot_sweep_transactions_per_block(data, param_name, results_dir, sweep_type)
    plot_sweep_tpb_moving_average(data, param_name, results_dir, sweep_type, plot_config)
    plot_system_memory(data, param_name, results_dir, sweep_type)
    plot_system_memory_total(data, param_name, results_dir, sweep_type)
    plot_system_cpu(data, param_name, results_dir, sweep_type)
    plot_system_cpu_filtered(data, param_name, results_dir, sweep_type)
    plot_system_cpu_total(data, param_name, results_dir, sweep_type)
    plot_cl_queue_length(data, param_name, results_dir, sweep_type)
    plot_loops_steps_without_tx_issuance_and_cl_queue(data, param_name, results_dir, sweep_type)
    plot_loop_steps_without_tx_issuance(data, param_name, results_dir, sweep_type)
    plot_loop_steps_without_tx_issuance_moving_average(data, param_name, results_dir, sweep_type, plot_config)

# Transaction count plots with their percentage counterparts (regular and moving average)
def generate_tx_count_plots(data, param_name, results_dir, sweep_type, plot_config):
    # print("Generating tx_count plots and their percentage plots...")
    
    # First: Generate all base overlay plots (non-percentage)
    # tx_count overlays
    plot_transactions_overlay(data, param_name, 'pending', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'success', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'failure', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'cat_pending', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'cat_success', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'cat_failure', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'cat_pending_resolving', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'cat_pending_postponed', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'regular_pending', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'regular_success', results_dir, sweep_type)
    plot_transactions_overlay(data, param_name, 'regular_failure', results_dir, sweep_type)
    
    # Total transaction plots
    # plot_total_cat_transactions(data, param_name, results_dir, sweep_type)
    # plot_total_regular_transactions(data, param_name, results_dir, sweep_type)
    # plot_total_sumtypes_transactions(data, param_name, results_dir, sweep_type)
    
    # Moving average overlays (if enabled)
    if plot_config.get('plot_moving_average', False):
        # print("Generating tx_count/moving_average plots...")
        plot_transactions_overlay_with_moving_average(data, param_name, 'pending', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'success', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'failure', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'cat_pending', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'cat_success', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'cat_failure', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'cat_pending_resolving', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'cat_pending_postponed', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'regular_pending', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'regular_success', results_dir, sweep_type, plot_config)
        plot_transactions_overlay_with_moving_average(data, param_name, 'regular_failure', results_dir, sweep_type, plot_config)
    
    # Last: Generate all percentage plots (after base plots are created)
    # tx_count percentage
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat', 'success')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat', 'failure')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat', 'pending')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'regular', 'success')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'regular', 'failure')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'regular', 'pending')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'sumtypes', 'success')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'sumtypes', 'failure')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'sumtypes', 'pending')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat_pending_resolving', 'pending')
    plot_transaction_percentage(data, param_name, results_dir, sweep_type, 'cat_pending_postponed', 'pending')
    
    # Moving average percentage plots (if enabled)
    if plot_config.get('plot_moving_average', False):
        # print("Generating tx_count/moving_average percentage plots...")
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'cat', 'success', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'cat', 'failure', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'cat', 'pending', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'regular', 'success', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'regular', 'failure', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'regular', 'pending', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'sumtypes', 'success', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'sumtypes', 'failure', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'sumtypes', 'pending', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'cat_pending_resolving', 'pending', plot_config)
        plot_transaction_percentage_with_moving_average(data, param_name, results_dir, sweep_type, 'cat_pending_postponed', 'pending', plot_config)

# Transaction count plots with cutoff applied (data trimmed and offset subtracted)
def generate_tx_count_cutoff_plots(data, param_name, results_dir, sweep_type, plot_config):
    # print("Generating tx_count_cutoff plots and their percentage plots...")
    plot_transactions_cutoff_overlay(data, param_name, 'pending', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'success', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'failure', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'cat_pending', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'cat_success', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'cat_failure', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'cat_pending_resolving', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'cat_pending_postponed', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'regular_pending', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'regular_success', results_dir, sweep_type, plot_config)
    plot_transactions_cutoff_overlay(data, param_name, 'regular_failure', results_dir, sweep_type, plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'cat', 'success', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'cat', 'failure', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'cat', 'pending', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'regular', 'success', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'regular', 'failure', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'regular', 'pending', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'sumtypes', 'success', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'sumtypes', 'failure', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'sumtypes', 'pending', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'cat_pending_resolving', 'pending', plot_config)
    plot_transaction_percentage_cutoff(data, param_name, results_dir, sweep_type, 'cat_pending_postponed', 'pending', plot_config)
    # No moving average for cutoff as per user request

# Transaction delta plots (rate of change) with their percentage counterparts
def generate_tx_count_delta_plots(data, param_name, results_dir, sweep_type, plot_config):
    # print("Generating tx_count_delta plots and their percentage plots...")
    plot_transactions_delta_overlay(data, param_name, 'pending', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'success', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'failure', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'cat_pending', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'cat_success', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'cat_failure', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'cat_pending_resolving', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'cat_pending_postponed', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'regular_pending', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'regular_success', results_dir, sweep_type)
    plot_transactions_delta_overlay(data, param_name, 'regular_failure', results_dir, sweep_type)
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat', 'success')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat', 'failure')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat', 'pending')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'regular', 'success')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'regular', 'failure')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'regular', 'pending')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'sumtypes', 'success')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'sumtypes', 'failure')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'sumtypes', 'pending')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat_pending_resolving', 'pending')
    plot_transaction_percentage_delta(data, param_name, results_dir, sweep_type, 'cat_pending_postponed', 'pending')
    if plot_config.get('plot_moving_average', False):
        # print("Generating tx_count_delta/moving_average plots and their percentage plots...")
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'pending', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'success', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'failure', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'cat_pending', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'cat_success', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'cat_failure', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'cat_pending_resolving', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'cat_pending_postponed', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'regular_pending', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'regular_success', results_dir, sweep_type, plot_config)
        plot_transactions_delta_overlay_with_moving_average(data, param_name, 'regular_failure', results_dir, sweep_type, plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'cat', 'success', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'cat', 'failure', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'cat', 'pending', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'regular', 'success', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'regular', 'failure', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'regular', 'pending', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'sumtypes', 'success', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'sumtypes', 'failure', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'sumtypes', 'pending', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'cat_pending_resolving', 'pending', plot_config)
        plot_transaction_percentage_delta_with_moving_average(data, param_name, results_dir, sweep_type, 'cat_pending_postponed', 'pending', plot_config)

# Individual simulation plots (one folder per simulation with detailed curves)
def generate_sim_x_plots(data, param_name, results_dir, sweep_type, plot_config):
    # print("Generating individual curves plots for each simulation...")
    generate_individual_curves_plots(data, param_name, results_dir, sweep_type)

def generate_organized_plots(data: Dict[str, Any], param_name: str, results_dir: str, sweep_type: str, plot_config: Dict[str, Any]) -> None:
    generate_paper_plots()
    generate_system_plots(data, param_name, results_dir, sweep_type, plot_config)
    generate_tx_count_plots(data, param_name, results_dir, sweep_type, plot_config)
    generate_tx_count_cutoff_plots(data, param_name, results_dir, sweep_type, plot_config)
    generate_tx_count_delta_plots(data, param_name, results_dir, sweep_type, plot_config)
    generate_sim_x_plots(data, param_name, results_dir, sweep_type, plot_config)
    # print(f"{sweep_type} simulation plots generated successfully!") 