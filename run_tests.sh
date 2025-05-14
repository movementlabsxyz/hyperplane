#!/bin/bash

# Running confirmation layer tests

# enlist all CL tests in a vector
CL_TESTS=(
    confirmation_layer::concurrent_setup::concurrent_setup_v12::test_concurrent_setup_v12
    confirmation_layer::concurrent_setup::concurrent_setup_v13::test_concurrent_setup_v13
    confirmation_layer::node::test_basic_confirmation_layer
    confirmation_layer::node::test_block_interval
    confirmation_layer::node::test_normal_transactions
    confirmation_layer::node::test_register_chain
    confirmation_layer::node::test_get_current_block
    confirmation_layer::node::test_get_subblock
    confirmation_layer::node::test_submit_transaction
    confirmation_layer::node::test_get_subblock
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
