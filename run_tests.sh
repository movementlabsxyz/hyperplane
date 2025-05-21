#!/bin/bash

# Check if an argument is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <test_set>"
    echo "  test_set: 1 for first set of tests, 2 for second set"
    exit 1
fi

# Running confirmation layer tests

TESTS=(
    # - - - Concurrency setup tests - - -
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

    # - - - CL tests - - -
    confirmation_layer::basic::test_regular_transactions
    confirmation_layer::basic::test_basic_confirmation_layer
    confirmation_layer::basic::test_block_interval
    confirmation_layer::basic::test_register_chain
    confirmation_layer::basic::test_get_current_block
    confirmation_layer::basic::test_get_subblock
    confirmation_layer::basic::test_submit_transaction
    confirmation_layer::basic::test_set_block_interval
    confirmation_layer::basic::test_invalid_block_interval
    confirmation_layer::basic::test_chain_not_found
    confirmation_layer::basic::test_chain_already_registered
    confirmation_layer::basic::test_chain_registration
    confirmation_layer::basic::test_block_interval_validation
    confirmation_layer::basic::test_subblock_not_found
    confirmation_layer::basic::test_get_registered_chains
    confirmation_layer::basic::test_get_block_interval
    confirmation_layer::basic::test_submit_transaction_chain_not_registered
    confirmation_layer::basic::test_submit_cl_transaction_for_multiple_chains
    
    # - - - HIG tests - - -
    hyper_ig::basic::test_regular_transaction_success
    hyper_ig::basic::test_regular_transaction_failure
    hyper_ig::basic::test_regular_transaction_pending
    hyper_ig::basic::test_cat_success_proposal
    hyper_ig::basic::test_cat_failure_proposal
    hyper_ig::basic::test_get_pending_transactions

    # - - - HS tests - - -
    hyper_scheduler::basic::test_receive_success_proposal_first_message
    hyper_scheduler::basic::test_receive_failure_proposal_first_message
    hyper_scheduler::basic::test_duplicate_rejection
    hyper_scheduler::basic::test_process_proposals_for_two_chain_cat
    
    # - - - CL to HIG tests - - -
    integration::cl_to_hig::channels::test_process_subblock_with_regular_transaction_success
    integration::cl_to_hig::channels::test_process_subblock_with_regular_transaction_failure
    integration::cl_to_hig::channels::test_process_subblock_with_cat_transaction

    # - - - HS to CL tests - - -
    integration::hs_to_cl::channels::test_single_chain_cat_status_update
    integration::hs_to_cl::channels::test_several_single_chain_cat_status_updates

    ## - - - CL to HS tests - - -
    integration::cl_to_hs::channels::test_single_chain_cat_success
    integration::cl_to_hs::channels::test_single_chain_cat_failure

    # - - - CL to CL tests - - -
    integration::cl_to_cl::channels::test_single_chain_cat_success
    integration::cl_to_cl::channels::test_single_chain_cat_failure

    # - - - e2e tests - - -
    integration::e2e::channels::test_single_chain_cat_success
    integration::e2e::channels::test_single_chain_cat_failure
)



TESTS2=(

integration::cl_to_hs::channels::test_single_chain_cat_success
    # hyper_scheduler::basic::test_process_proposals_for_two_chain_cat
)

# Run the appropriate test set based on the input
if [ "$1" = "1" ]; then
    for test in "${TESTS[@]}"; do
        echo -e "\nRunning $test..."
        cargo test --test main $test -- --test-threads=1 --nocapture | grep "FAILED"
    done
elif [ "$1" = "2" ]; then
    for test in "${TESTS2[@]}"; do
        echo -e "\nRunning $test..."
        cargo test --test main $test -- --test-threads=1 --nocapture #| grep "FAILED"
    done
else
    echo "Invalid test set. Use 1 or 2."
    exit 1
fi
 