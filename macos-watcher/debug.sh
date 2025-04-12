#!/bin/bash
set -e
source .env

# Build debug version for testing
echo "Building debug version..."
cargo build

codesign --force --sign "$APPLE_ID" --options runtime target/debug/macos-watcher

# Run directly without bundling
echo "Running binary directly..."
./target/debug/macos-watcher

echo "Check ~/macos_watcher.log for output" 