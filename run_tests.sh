#!/bin/bash

set -e

# Check if an argument is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <test_set>"
    echo "  test_set: 1 for first set of tests, 2 for second set"
    exit 1
fi

# Running confirmation layer tests

TESTS=(
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

    # Hyper IG tests
    hyper_ig::tests::basic::test_regular_transaction_success
    hyper_ig::tests::basic::test_regular_transaction_failure
    hyper_ig::tests::basic::test_regular_transaction_pending
    hyper_ig::tests::basic::test_cat_process_and_send_success
    hyper_ig::tests::basic::test_cat_process_and_send_failure
    hyper_ig::tests::basic::test_get_pending_transactions
    hyper_ig::tests::basic::test_wrong_chain_subblock

    # Hyper Scheduler tests
    hyper_scheduler::tests::basic::test_receive_success_proposal_first_message
    hyper_scheduler::tests::basic::test_receive_failure_proposal_first_message
    hyper_scheduler::tests::basic::test_duplicate_rejection
    hyper_scheduler::tests::basic::test_process_proposals_for_two_chain_cat

    # Integration tests: CL to HIG
    integration::cl_to_hig::channels::test_process_subblock_with_regular_transaction_success
    integration::cl_to_hig::channels::test_process_subblock_with_regular_transaction_failure
    integration::cl_to_hig::channels::test_process_subblock_with_cat_transaction

    # Integration tests: HS to CL
    integration::hs_to_cl::channels::test_single_chain_cat_status_update
    integration::hs_to_cl::channels::test_several_single_chain_cat_status_updates

    # Integration tests: CL to HS
    integration::cl_to_hs::channels::test_cat_one_cat_success
    integration::cl_to_hs::channels::test_cat_one_cat_failure

    # Integration tests: CL to CL
    integration::cl_to_cl::channels::test_two_chain_cat_success
    integration::cl_to_cl::channels::test_two_chain_cat_failure

    # Integration tests: e2e
    integration::e2e::channels::test_single_chain_cat_success
    integration::e2e::channels::test_single_chain_cat_failure
)

TESTS2=(
)

# Run the appropriate test set based on the input
if [ "$1" = "1" ]; then
    for test in "${TESTS[@]}"; do
        echo -e "\nRunning $test..."
        cargo test $test -- --test-threads=1 --nocapture | grep "FAILED"
    done
elif [ "$1" = "2" ]; then
    for test in "${TESTS2[@]}"; do
        echo -e "\nRunning $test..."
        cargo test $test -- --test-threads=1 --nocapture #| grep "FAILED"
        # cargo test $test -- --test-threads=1 --nocapture --exact #| grep "FAILED"
    done
else
    echo "Invalid test set. Use 1 or 2."
    exit 1
fi
 