#!/usr/bin/env bash
# Regenerate every language binding from the single annotated-Rust template
# (src/bridge/). Output is committed under bindings/; CI regenerates and fails on
# any diff, so the committed bindings can never go stale.
#
# diplomat-tool comes from our fork (adds the PHP backend):
#   cargo install --git https://github.com/Nano112/diplomat --branch php-backend diplomat-tool
# or a local checkout's binary via DIPLOMAT_TOOL=/path/to/diplomat-tool.
set -euo pipefail
cd "$(dirname "$0")/.."

DT="${DIPLOMAT_TOOL:-diplomat-tool}"
ENTRY="src/bridge/mod.rs"

command -v "$DT" >/dev/null || {
    echo "diplomat-tool not found; install with:" >&2
    echo "  cargo install --git https://github.com/Nano112/diplomat --branch php-backend diplomat-tool" >&2
    exit 1
}

# python/ and kotlin/ keep hand-maintained packaging at their roots (pyproject/CMake,
# gradle); only their generated subtrees are wiped.
rm -rf bindings/c bindings/cpp bindings/js bindings/kotlin/src bindings/python/src bindings/php

"$DT" c       bindings/c       -e "$ENTRY" -s
"$DT" cpp     bindings/cpp     -e "$ENTRY" -s
"$DT" js      bindings/js      -e "$ENTRY" -s
"$DT" kotlin  bindings/kotlin  -e "$ENTRY" -s --config-file tools/bindgen/kotlin.toml
"$DT" nanobind bindings/python/src -e "$ENTRY" -s --config-file tools/bindgen/nanobind.toml
"$DT" php     bindings/php     -e "$ENTRY" -s --config-file tools/bindgen/php.toml

echo "bindings regenerated from $ENTRY"
