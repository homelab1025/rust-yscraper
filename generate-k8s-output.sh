#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT="${1:-${SCRIPT_DIR}/output.yaml}"

kubectl kustomize "${SCRIPT_DIR}/k8s/overlays/prod" -o "${OUTPUT}"

echo "Generated: ${OUTPUT}"
