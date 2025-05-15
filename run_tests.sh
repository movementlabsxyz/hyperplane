#!/bin/bash

# Running confirmation layer tests

TESTS=(
    # - - - CL tests - - -
    confirmation_layer::concurrent_setup::concurrent_setup_v11_to_v12::test_concurrent_setup_v12
    confirmation_layer::concurrent_setup::concurrent_setup_v13::test_concurrent_setup_v13
    confirmation_layer::basic::test_basic_confirmation_layer
    confirmation_layer::basic::test_block_interval
    confirmation_layer::basic::test_normal_transactions
    confirmation_layer::basic::test_register_chain
    confirmation_layer::basic::test_get_current_block
    confirmation_layer::basic::test_get_subblock
    confirmation_layer::basic::test_submit_transaction
    confirmation_layer::basic::test_get_subblock
    
    # - - - HIG tests - - -
    hyper_ig::basic::test_normal_transaction_success
    hyper_ig::basic::test_normal_transaction_pending
    hyper_ig::basic::test_cat_success_proposal
    hyper_ig::basic::test_cat_failure_proposal
    hyper_ig::basic::test_cat_success_update
    hyper_ig::basic::test_execute_transactions
    hyper_ig::basic::test_get_transaction_status
    hyper_ig::basic::test_get_pending_transactions

    # - - - HS tests - - -
    hyper_scheduler::basic::test_receive_success_proposal_first_message
    hyper_scheduler::basic::test_receive_failure_proposal_first_message
    hyper_scheduler::basic::test_duplicate_rejection

    # - - - CL to HIG tests - - -
    integration::cl_to_hig::channels::test_process_subblock
    integration::cl_to_hig::channels::test_process_cat_subblock
    integration::cl_to_hig::channels::test_process_multiple_subblocks_new_transactions

    ## - - - HIG to HS tests - - -
    integration::hig_to_hs::channels::test_single_cat_status_storage
    integration::hig_to_hs::channels::test_multiple_cat_status_storage
    integration::hig_to_hs::channels::test_status_proposal_failure
    integration::hig_to_hs::channels::test_send_cat_status_proposal
    integration::hig_to_hs::channels::test_process_cat_transaction
    integration::hig_to_hs::channels::test_process_status_update
    integration::hig_to_hs::channels::test_hig_to_hs_status_proposal
    integration::hig_to_hs::channels::test_hig_to_hs_status_proposal_failure
    integration::hig_to_hs::channels::test_hig_to_hs_multiple_status_proposals
    integration::hig_to_hs::channels::test_cat_status_storage_with_transaction_id
)

# for test in "${TESTS[@]}"; do
#     echo -e "\nRunning $test..."
#     cargo test --test main $test -- --test-threads=1 #--nocapture
# done


TESTS2=(

)

for test in "${TESTS2[@]}"; do
    echo -e "\nRunning $test..."
    cargo test --test main $test -- --test-threads=1 #--nocapture
done
