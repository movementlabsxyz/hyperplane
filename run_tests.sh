#!/bin/bash

# Running confirmation layer tests

# enlist all CL tests in a vector
CL_TESTS=(
    # confirmation_layer::concurrent_setup::concurrent_setup_v12::test_concurrent_setup_v12
    confirmation_layer::concurrent_setup::concurrent_setup_v13::test_concurrent_setup_v13
    confirmation_layer::basic::test_basic_confirmation_layer
    # confirmation_layer::basic::test_block_interval
    # confirmation_layer::basic::test_normal_transactions
    # confirmation_layer::basic::test_register_chain
    # confirmation_layer::basic::test_get_current_block
    # confirmation_layer::basic::test_get_subblock
    # confirmation_layer::basic::test_submit_transaction
    # confirmation_layer::basic::test_get_subblock
)

for test in "${CL_TESTS[@]}"; do
    echo -e "\nRunning $test..."
    cargo test --test main $test -- --test-threads=1 #--nocapture
done

# Running hyper ig tests

HIG_TESTS=(
    hyper_ig::basic::test_normal_transaction_success
    hyper_ig::basic::test_normal_transaction_pending
    hyper_ig::basic::test_cat_success_proposal
    hyper_ig::basic::test_cat_failure_proposal
    hyper_ig::basic::test_cat_success_update
    hyper_ig::basic::test_execute_transactions
    hyper_ig::basic::test_get_transaction_status
    hyper_ig::basic::test_get_pending_transactions
)

for test in "${HIG_TESTS[@]}"; do
    echo -e "\nRunning $test..."
    cargo test --test main $test -- --test-threads=1 #--nocapture
done

# Running hyper scheduler tests

HS_TESTS=(
    hyper_scheduler::basic::test_receive_success_proposal
    hyper_scheduler::basic::test_receive_failure_proposal
    hyper_scheduler::basic::test_receive_proposal_errors
    hyper_scheduler::basic::test_send_success_update
    hyper_scheduler::basic::test_send_failure_update
    hyper_scheduler::basic::test_send_update_errors
    hyper_scheduler::basic::test_process_single_chain_cat
    hyper_scheduler::basic::test_process_two_chain_cat
    hyper_scheduler::basic::test_process_conflicting_statuses
    hyper_scheduler::basic::test_process_cat_timeout
)

# for test in "${HS_TESTS[@]}"; do
#     echo -e "\nRunning $test..."
#     cargo test --test main $test -- --test-threads=1 --nocapture
# done

# Running CL to HIG integration tests

CL_TO_HIG_TESTS=(
    integration::cl_to_hig::channels::test_process_subblock
    integration::cl_to_hig::channels::test_process_cat_subblock
    integration::cl_to_hig::channels::test_process_multiple_subblocks_new_transactions
)

for test in "${CL_TO_HIG_TESTS[@]}"; do
    echo -e "\nRunning $test..."
    cargo test --test main $test -- --test-threads=1 --nocapture
done
