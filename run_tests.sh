#!/bin/bash

# Running confirmation layer tests

# echo -e "\nRunning mutex v12 test..."
# cargo test test_concurrent_setup_v12 -- --test-threads=1 --nocapture 

echo -e "\nRunning mutex v13 test..."
cargo test test_concurrent_setup_v13 -- --test-threads=1 --nocapture

echo "Running basic confirmation layer test..."
cargo test test_basic_confirmation_layer -- --test-threads=1 --nocapture

echo "Running block interval test..."
cargo test test_block_interval -- --test-threads=1 --nocapture

echo "Running normal transactions test..."
cargo test test_normal_transactions -- --test-threads=1 --nocapture

echo "Running chain registration test..."
cargo test test_register_chain -- --test-threads=1 --nocapture

echo "Running get current block test..."
cargo test test_get_current_block -- --test-threads=1 --nocapture

echo "Running get subblock test..."
cargo test test_get_subblock -- --test-threads=1 --nocapture

echo "Running submit transaction test..."
cargo test test_submit_transaction -- --test-threads=1 --nocapture

echo "Running get subblock test..."
cargo test test_get_subblock -- --test-threads=1 --nocapture

echo "Running submit transaction test..."
cargo test test_submit_transaction -- --test-threads=1 --nocapture

echo "Running get subblock test..."
cargo test test_get_subblock -- --test-threads=1 --nocapture

echo "Running submit transaction test..."
cargo test test_submit_transaction -- --test-threads=1 --nocapture


# Running consensus layer tests

# echo "Running basic consensus layer test..."
# cargo test test_basic_consensus_layer -- --test-threads=1 --nocapture


