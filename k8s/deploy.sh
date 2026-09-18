#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NAMESPACE="yscraper"

echo "Deploying to context: $(kubectl config current-context)"

kubectl apply -k "$SCRIPT_DIR/overlays/prod"

kubectl rollout restart deployment/web-server deployment/webapp -n "$NAMESPACE"

kubectl rollout status deployment/web-server -n "$NAMESPACE"
kubectl rollout status deployment/webapp -n "$NAMESPACE"
