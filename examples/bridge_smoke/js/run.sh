#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
ROOT="../../.."

cargo build --release --target wasm32-unknown-unknown --lib --features bridge --manifest-path "$ROOT/Cargo.toml"
node main.mjs
