#!/bin/bash

crates=("bevy_gpu_compute_macro" "bevy_gpu_compute_core" "bevy_gpu_compute")

for crate in "${crates[@]}"; do
    echo "Processing $crate..."
    cd "$crate" || exit 1
    
    echo "Running cargo fmt..."
    cargo fmt
    
    echo "Running cargo clippy..."
    cargo clippy -- -D warnings
    
    echo "Running cargo test..."
    cargo test
    
    
    
    cd ..
done

cd bevy_gpu_compute
echo "Running example ..."
cargo run --example collision_detection_demonstration
cd ..

echo "All checks completed!"