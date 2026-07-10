#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
ROOT="../../.."

cargo build --release --lib --features bridge-full --manifest-path "$ROOT/Cargo.toml"
php -d ffi.enable=1 main.php
