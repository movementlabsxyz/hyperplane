#!/bin/bash

# run_tests.sh - Test runner for Hyperplane
#
# This script runs tests in two different modes:
#
# 1. Test Set 1 (./run_tests.sh 1 <logging>):
#    - Runs all unit tests and integration tests
#    - Includes error checking to ensure tests are actually run
#    - Automatically selects the correct test target (--lib or --test main)
#
# 2. Test Set 2 (./run_tests.sh 2 <logging>):
#    - Runs a single integration test
#    - Used for debugging with full log output
#
# Logging argument:
#   - 0: Disable logging
#   - 1: Enable logging
#
# Examples:
#   ./run_tests.sh 1 0    # Run all tests without logging
#   ./run_tests.sh 1 1    # Run all tests with logging
#   ./run_tests.sh 2 1    # Run single test with logging

set -e


# All tests
TESTS=(
    # Mock VM tests
    mock_vm::tests::test_credit_transaction
    mock_vm::tests::test_send_transaction

    # Setup/Concurrency tests
    setup_with_mpsc::v1_to_v7::test_v1
    setup_with_mpsc::v1_to_v7::test_v2
    setup_with_mpsc::v1_to_v7::test_v3
    setup_with_mpsc::v1_to_v7::test_v4
    setup_with_mpsc::v1_to_v7::test_v5
    setup_with_mpsc::v1_to_v7::test_v6
    setup_with_mpsc::v1_to_v7::test_v7
    setup_with_mpsc::v8_to_v10::test_v8
    setup_with_mpsc::v8_to_v10::test_v9
    setup_with_mpsc::v8_to_v10::test_v10
    setup_with_mpsc::v11_to_v12::test_v11
    setup_with_mpsc::v11_to_v12::test_v12
    setup_with_mpsc::v13::test_v13

    # Confirmation Layer tests
    confirmation_layer::tests::basic::test_block_interval
    confirmation_layer::tests::basic::test_transaction_submission
    confirmation_layer::tests::basic::test_chain_registration
    confirmation_layer::tests::basic::test_get_current_block
    confirmation_layer::tests::basic::test_get_subblock
    confirmation_layer::tests::basic::test_chain_not_found
    confirmation_layer::tests::basic::test_get_registered_chains
    confirmation_layer::tests::basic::test_get_block_interval
    confirmation_layer::tests::basic::test_submit_transaction_chain_not_registered
    confirmation_layer::tests::basic::test_submit_cl_transaction_for_two_chains
    confirmation_layer::tests::basic::test_dynamic_channel_registration
    confirmation_layer::tests::shutdown::test_cl_node_shutdown
    confirmation_layer::tests::shutdown::test_cl_node_shutdown_multiple_times
    confirmation_layer::tests::shutdown::test_cl_node_restart_after_shutdown

    # Hyper IG tests
    hyper_ig::tests::basic::test_regular_transaction_success
    hyper_ig::tests::basic::test_regular_transaction_failure
    hyper_ig::tests::basic::test_regular_transaction_pending
    hyper_ig::tests::basic::test_cat_process_and_send_success
    hyper_ig::tests::basic::test_cat_process_and_send_failure
    hyper_ig::tests::basic::test_get_pending_transactions
    hyper_ig::tests::basic::test_wrong_chain_subblock
    hyper_ig::tests::basic::test_cat_pattern
    hyper_ig::tests::basic::test_send_after_credit
    hyper_ig::tests::basic::test_cat_send_no_funds
    hyper_ig::tests::basic::test_cat_credit_pending
    hyper_ig::tests::basic::test_cat_send_after_credit
    hyper_ig::tests::basic::test_get_chain_state_empty
    hyper_ig::tests::basic::test_get_chain_state_after_transaction
    hyper_ig::tests::basic::test_duplicate_transaction_id
    hyper_ig::tests::basic::test_hs_message_delay
    hyper_ig::tests::basic::test_cat_pending_dependency_restriction
    hyper_ig::tests::basic::test_cat_pending_dependency_flag_runtime_change
    hyper_ig::tests::basic::test_proposal_queue_independent_delays
    hyper_ig::tests::dependencies::test_failed_dependency
    hyper_ig::tests::dependencies::test_multiple_transactions_same_key_success
    hyper_ig::tests::dependencies::test_multiple_transactions_same_key_fail
    hyper_ig::tests::dependencies::test_single_dependency
    hyper_ig::tests::dependencies::test_success_dependency
    hyper_ig::tests::dependencies::test_cat_lock_release_on_success
    hyper_ig::tests::timeouts::test_cat_timeout
    hyper_ig::tests::timeouts::test_cat_timeout_not
    hyper_ig::tests::timeouts::test_cat_timeout_irreversible
    hyper_ig::tests::timeouts::test_cat_success_should_not_timeout
    hyper_ig::tests::timeouts::test_status_update_before_timeout_should_process
    hyper_ig::tests::timeouts::test_status_update_at_timeout_boundary_should_process
    hyper_ig::tests::preloaded_accounts::test_preloaded_accounts
    hyper_ig::tests::preloaded_accounts::test_transactions_with_preloaded_accounts
    hyper_ig::tests::preloaded_accounts::test_no_preloaded_accounts
    hyper_ig::tests::preloaded_accounts::test_credit_with_preloaded_accounts
    hyper_ig::tests::preloaded_accounts::test_simulation_with_preloaded_accounts
    hyper_ig::tests::shutdown::test_hig_node_shutdown
    hyper_ig::tests::shutdown::test_hig_node_shutdown_multiple_times
    hyper_ig::tests::shutdown::test_hig_node_restart_after_shutdown

    # Hyper Scheduler tests
    hyper_scheduler::tests::basic::test_receive_cat_for_unregistered_chain
    hyper_scheduler::tests::basic::test_receive_success_proposal_first_message
    hyper_scheduler::tests::basic::test_receive_failure_proposal_first_message
    hyper_scheduler::tests::basic::test_duplicate_rejection
    hyper_scheduler::tests::basic::test_process_proposals_for_two_chain_cat
    hyper_scheduler::tests::basic::test_cannot_set_success_if_constituent_chains_dont_match
    hyper_scheduler::tests::shutdown::test_hs_node_shutdown
    hyper_scheduler::tests::shutdown::test_hs_node_shutdown_multiple_times
    hyper_scheduler::tests::shutdown::test_hs_node_restart_after_shutdown

    # Integration tests: CL to HIG
    integration::cl_to_hig::test_process_subblock_with_regular_transaction_success
    integration::cl_to_hig::test_process_subblock_with_regular_transaction_failure

    # Integration tests: HIG to CL
    integration::hig_to_cl::test_hs_waits_for_all_proposals

    # Integration tests: HS to CL
    integration::hs_to_cl::test_single_chain_cat_status_update
    integration::hs_to_cl::test_several_single_chain_cat_status_updates

    # Integration tests: CL to HS
    integration::cl_to_hs::test_cat_one_cat_success
    integration::cl_to_hs::test_cat_one_cat_failure
    integration::cl_to_hs::test_hig_delays


    # Integration tests: CL to CL
    integration::cl_to_cl::test_two_chain_cat_success
    integration::cl_to_cl::test_two_chain_cat_failure

    # Integration tests: e2e
    integration::e2e::test_two_chain_cat_success
    integration::e2e::test_two_chain_cat_failure
    integration::e2e::test_cat_with_only_chain_1_credit
    integration::e2e::test_cat_with_both_chains_credit
    integration::e2e::test_cat_credit_then_send
    integration::e2e::test_hig_delays
)

# Test specific tests
TESTS2=(

    # Shutdown tests
    confirmation_layer::tests::shutdown::test_cl_node_shutdown
    confirmation_layer::tests::shutdown::test_cl_node_shutdown_multiple_times
    confirmation_layer::tests::shutdown::test_cl_node_restart_after_shutdown
    hyper_ig::tests::shutdown::test_hig_node_shutdown
    hyper_ig::tests::shutdown::test_hig_node_shutdown_multiple_times
    hyper_ig::tests::shutdown::test_hig_node_restart_after_shutdown
    hyper_scheduler::tests::shutdown::test_hs_node_shutdown
    hyper_scheduler::tests::shutdown::test_hs_node_shutdown_multiple_times
    hyper_scheduler::tests::shutdown::test_hs_node_restart_after_shutdown

)

# Check if arguments are provided
if [ $# -ne 2 ]; then
    echo "Usage: $0 <test_set> <logging>"
    echo "  test_set: 1 for first set of tests, 2 for second set"
    echo "  logging:  0 to disable logging, 1 to enable logging"
    exit 1
fi

# Validate logging argument
if [ "$2" != "0" ] && [ "$2" != "1" ]; then
    echo "Error: logging must be 0 or 1"
    exit 1
fi

# Set logging environment variables
if [ "$2" = "1" ]; then
    export HYPERPLANE_LOGGING=true           # Enable logging
    export HYPERPLANE_LOG_TO_FILE=false      # Log to terminal for tests
else
    export HYPERPLANE_LOGGING=false          # Disable logging
    export HYPERPLANE_LOG_TO_FILE=true       # Default to file when logging is enabled
fi

# Function to determine test target based on test path
get_test_target() {
    local test=$1
    if [[ $test == integration::* ]] || [[ $test == setup_with_mpsc::* ]]; then
        echo "--test main"
    else
        echo "--lib"
    fi
}

# Function to run a test and check if it actually ran
run_test() {
    local test=$1
    local test_target=$2
    
    echo -e "\nRunning $test..."
    # Run the test and capture both stdout and stderr
    if ! cargo test $test_target $test --features test -- --test-threads=1 --nocapture --exact; then
        echo "Error: Test $test failed"
        exit 1
    fi
}

# Run the appropriate test set based on the input
if [ "$1" = "1" ]; then
    for test in "${TESTS[@]}"; do
        test_target=$(get_test_target "$test")
        run_test "$test" "$test_target"
    done
elif [ "$1" = "2" ]; then
    for test in "${TESTS2[@]}"; do
        test_target=$(get_test_target "$test")
        run_test "$test" "$test_target"
    done
else
    echo "Invalid test set. Use 1 or 2."
    exit 1
fi

 