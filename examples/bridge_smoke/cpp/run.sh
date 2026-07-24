#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
ROOT="../../.."

cargo build --release --lib --features bridge-full --manifest-path "$ROOT/Cargo.toml"
BIN="$(mktemp -d)/bridge_smoke_cpp"
clang++ -std=c++20 -I "$ROOT/bindings/cpp" main.cpp \
    -L "$ROOT/target/release" -lnucleation -o "$BIN"
if [[ "$(uname)" == "Darwin" ]]; then
    DYLD_LIBRARY_PATH="$ROOT/target/release" "$BIN"
else
    LD_LIBRARY_PATH="$ROOT/target/release" "$BIN"
fi
