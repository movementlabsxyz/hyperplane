#!/bin/bash

# echo -e "\nRunning mutex v12 test..."
# cargo test test_concurrent_setup_v12 -- --test-threads=1 --nocapture 

echo -e "\nRunning mutex v13 test..."
cargo test test_concurrent_setup_v13 -- --test-threads=1 --nocapture

echo "Running basic confirmation layer test..."
cargo test test_basic_confirmation_layer -- --test-threads=1 --nocapture
