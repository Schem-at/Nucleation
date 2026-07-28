#!/usr/bin/env bash
# Build the nanobind extension (same recipe as examples/bridge_smoke/python)
# and run tests/python_mc_tick_sim_test.py against it.
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
SMOKE="$ROOT/examples/bridge_smoke/python"

NANOBIND_SPEC="nanobind>=2.12,<3"
VENV="$SMOKE/.venv"
if [ ! -d "$VENV" ]; then
    python3 -m venv "$VENV"
fi
"$VENV/bin/pip" install -q --upgrade pip "$NANOBIND_SPEC"
PY="$VENV/bin/python3"

cargo build --release --lib --features bridge-full

PY_INCLUDE=$("$PY" -c "import sysconfig; print(sysconfig.get_paths()['include'])")
NB_DIR=$("$PY" -c "import nanobind, os; print(os.path.dirname(nanobind.__file__))")
EXT_SUFFIX=$("$PY" -c "import sysconfig; print(sysconfig.get_config_var('EXT_SUFFIX'))")

BINDINGS="$ROOT/bindings/python/src"
BUILD_DIR="$(mktemp -d)"
trap 'rm -rf "$BUILD_DIR"' EXIT
OUT="$BUILD_DIR/nucleation${EXT_SUFFIX}"

EXTRA_LDFLAGS=""
if [[ "$(uname)" == "Linux" ]]; then
    EXTRA_LDFLAGS="-fuse-ld=lld"
fi

clang++ -std=c++20 -shared -fPIC -undefined dynamic_lookup -fvisibility=hidden \
    $EXTRA_LDFLAGS \
    -I "$PY_INCLUDE" \
    -I "$NB_DIR/include" \
    -I "$NB_DIR/ext/robin_map/include" \
    -I "$BINDINGS/include" \
    -I "$ROOT/bindings/python/custom" \
    "$NB_DIR/src/nb_combined.cpp" \
    "$BINDINGS/nucleation_ext.cpp" \
    "$BINDINGS"/sub_modules/nucleation/*.cpp \
    -L "$ROOT/target/release" -lnucleation \
    -o "$OUT"

if [[ "$(uname)" == "Darwin" ]]; then
    PYTHONPATH="$BUILD_DIR" DYLD_LIBRARY_PATH="$ROOT/target/release" "$PY" tests/python_mc_tick_sim_test.py
else
    PYTHONPATH="$BUILD_DIR" LD_LIBRARY_PATH="$ROOT/target/release" "$PY" tests/python_mc_tick_sim_test.py
fi
