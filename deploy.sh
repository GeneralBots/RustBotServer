#!/bin/bash
set -e

echo "Deploying General Bots platform..."

# Create namespace
kubectl apply -f k8s/base/namespace.yaml

# Deploy infrastructure components
kubectl apply -f k8s/base/postgres.yaml
kubectl apply -f k8s/base/redis.yaml
kubectl apply -f k8s/base/kafka.yaml
kubectl apply -f k8s/base/monitoring.yaml

# Deploy application components
kubectl apply -f k8s/base/api.yaml
kubectl apply -f k8s/base/webrtc.yaml
kubectl apply -f k8s/base/image.yaml
kubectl apply -f k8s/base/document.yaml

# Deploy ingress rules
kubectl apply -f k8s/base/ingress.yaml

# Create DB.

#cargo run -p gb-migrations --bin migrations

echo "Deployment completed successfully!"
echo "Please wait for all pods to be ready..."
kubectl -n general-bots get pods -w

