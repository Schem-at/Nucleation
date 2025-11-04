#!/bin/bash

echo "🧪 Running WASM chunk iterator tests..."

# First, ensure we have the right target installed
echo "📦 Installing WASM target if not present..."
rustup target add wasm32-unknown-unknown

# Make sure we have wasm-pack installed
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Run the WASM tests with wasm-pack (this is the preferred method)
echo "🚀 Running WASM bindgen tests with wasm-pack..."
wasm-pack test --node --features wasm,simulation

# Run Node.js integration test if the build exists
if [ -d "pkg" ]; then
    echo "🧪 Running Node.js integration tests..."
    cd tests && npm test
    cd ..
else
    echo "⚠️  No pkg directory found. Run ./build-wasm.sh first for Node.js tests."
fi

echo "✅ WASM tests completed!"
