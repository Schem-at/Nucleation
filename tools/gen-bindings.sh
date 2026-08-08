#!/usr/bin/env bash
# Regenerate every language binding from the single annotated-Rust template
# (src/bridge/). Output is committed under bindings/; CI regenerates and fails on
# any diff, so the committed bindings can never go stale.
#
# diplomat-tool comes from our fork (adds the PHP backend):
#   cargo install --git https://github.com/Nano112/diplomat --branch nanobind-public-api diplomat-tool
# or a local checkout's binary via DIPLOMAT_TOOL=/path/to/diplomat-tool.
set -euo pipefail
cd "$(dirname "$0")/.."

DT="${DIPLOMAT_TOOL:-diplomat-tool}"
ENTRY="src/bridge/mod.rs"

command -v "$DT" >/dev/null || {
    echo "diplomat-tool not found; install with:" >&2
    echo "  cargo install --git https://github.com/Nano112/diplomat --branch nanobind-public-api diplomat-tool" >&2
    exit 1
}

# Probe for the PHP backend BEFORE wiping anything: upstream diplomat-tool
# lacks it and would otherwise die mid-run with bindings/php already deleted.
"$DT" php --help >/dev/null 2>&1 || {
    echo "installed diplomat-tool has no 'php' target (upstream build?); reinstall the fork:" >&2
    echo "  cargo install --git https://github.com/Nano112/diplomat --branch nanobind-public-api diplomat-tool --force" >&2
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

# A `bool` return must reach JS as a real boolean, not the raw wasm i32. A
# tool built before Nano112/diplomat@8fec8fc emits `return result;`, so every
# bool-returning method silently hands back 0/1 while its .d.ts still says
# `boolean` — and a caller comparing against `false` (the natural way to read
# "did this succeed") takes the wrong branch. It compiles and type-checks, so
# nothing else catches it. Unlike the php probe above this runs after the
# wipe, which is safe: bindings/ is committed, so `git checkout bindings/`
# undoes a failed run.
grep -rq 'result === 1' bindings/js || {
    echo "generated JS has no bool coercion; diplomat-tool predates the fix. Reinstall:" >&2
    echo "  cargo install --git https://github.com/Nano112/diplomat --branch nanobind-public-api diplomat-tool --force" >&2
    echo "then re-run this script (bindings/ is committed: 'git checkout bindings/' to reset)." >&2
    exit 1
}

# Diplomat deliberately emits the same low-level surface for every language.
# Reapply narrow target-specific compatibility layers after generation.
python3 tools/patch-js-bindings.py
python3 tools/patch-kotlin-bindings.py
python3 tools/patch-python-bindings.py

echo "bindings regenerated from $ENTRY"
