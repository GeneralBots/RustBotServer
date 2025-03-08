#!/bin/bash
set -e

echo "Running tests for all components..."

# Core tests
echo "Testing gb-core..."
cd gb-core && cargo test

# API tests
echo "Testing gb-server..."
cd ../gb-server && cargo test

# VM tests
echo "Testing gb-vm..."
cd ../gb-vm && cargo test

# Document processing tests
echo "Testing gb-document..."
cd ../gb-document && cargo test

# Image processing tests
echo "Testing gb-image..."
cd ../gb-image && cargo test

# NLP tests
echo "Testing gb-nlp..."
cd ../gb-nlp && cargo test

# Utils tests
echo "Testing gb-utils..."
cd ../gb-utils && cargo test

# Messaging tests
echo "Testing gb-messaging..."
cd ../gb-messaging && cargo test

# Monitoring tests
echo "Testing gb-monitoring..."
cd ../gb-monitoring && cargo test

echo "All tests completed successfully!"
