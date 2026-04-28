#!/usr/bin/env bash
# Run Speculos tests using the official Ledger Docker image.
# Usage: ./scripts/test.sh [device]
# Devices: nanosp (default), nanox, stax, flex

set -euo pipefail

DEVICE="${1:-nanosp}"
IMAGE="ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:latest"

echo "Running tests for $DEVICE..."

docker run --rm \
    -v "$(pwd):/app" \
    -w /app \
    "$IMAGE" \
    bash -c "cargo ledger build nanosplus && pytest tests/standalone/ --device $DEVICE -v"
