#!/bin/bash
set -e

echo "Deploying General Bots platform..."

# Create DB.

cargo run -p gb-migrations --bin migrations

echo "Deployment completed successfully!"
echo "Please wait for all pods to be ready..."

