#!/bin/bash
set -e

echo "Running gb-testing test suite..."

# Run integration tests
echo "Running integration tests..."
cargo test --test '*' --features integration

# Run load tests
echo "Running load tests..."
cargo test --test '*' --features load

# Run performance benchmarks
#echo "Running performance benchmarks..."
#cargo bench

# Run stress tests
#echo "Running stress tests..."
#cargo test --test '*' --features stress

# Run chaos tests
#echo "Running chaos tests..."
#cargo test --test '*' --features chaos

echo "All tests completed!"
