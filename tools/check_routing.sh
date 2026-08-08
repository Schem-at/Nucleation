#!/usr/bin/env bash
# CI-ish checks for the redstone-EDA crates (pnr-core + nucleation-routing)
# and their integration into the main crate.
#
# WASM compatibility is a REQUIREMENT: both crates must build for
# wasm32-unknown-unknown (no fs, no threads, no unseeded randomness in core
# paths), and everything must be deterministic from seeds.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== unit + integration tests =="
cargo test -p pnr-core -p nucleation-routing

echo "== wasm32-unknown-unknown =="
rustup target list --installed | grep -q wasm32-unknown-unknown \
  || rustup target add wasm32-unknown-unknown
cargo check -p pnr-core --target wasm32-unknown-unknown
cargo check -p nucleation-routing --target wasm32-unknown-unknown

echo "== main-crate integration (routing feature) =="
cargo check --features routing
cargo test --features routing --lib routing::

echo "== bridge surface compiles (codegen not required) =="
cargo check --features bridge-full,routing

echo "ALL ROUTING CHECKS PASSED"
