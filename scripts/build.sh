#!/usr/bin/env bash
# Build the Squads Ledger app using the official Ledger Docker image.
# Usage: ./scripts/build.sh [target]
# Targets: nanosplus (default), nanox, stax, flex, apex_p

set -euo pipefail

TARGET="${1:-nanosplus}"
IMAGE="ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:latest"

echo "Building app-squads for $TARGET..."

docker run --rm \
    -v "$(pwd):/app" \
    -w /app \
    "$IMAGE" \
    cargo ledger build "$TARGET"

echo "Build complete for $TARGET."
echo "Binary at: target/$TARGET/release/app-squads"
