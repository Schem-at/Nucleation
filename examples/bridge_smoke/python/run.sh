#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
ROOT="../../.."

# The generated dealloc shim uses nanobind's public low-level instance API
# (diplomat fork branch nanobind-public-api), so any nanobind >= 2.12 works,
# including 2.13+ where the private nb_inst layout changed.
NANOBIND_SPEC="nanobind>=2.12,<3"

VENV=".venv"
if [ ! -d "$VENV" ]; then
    python3 -m venv "$VENV"
fi
"$VENV/bin/pip" install -q --upgrade pip "$NANOBIND_SPEC"
PY="$VENV/bin/python3"

cargo build --release --lib --features bridge-full --manifest-path "$ROOT/Cargo.toml"

PY_INCLUDE=$("$PY" -c "import sysconfig; print(sysconfig.get_paths()['include'])")
NB_DIR=$("$PY" -c "import nanobind, os; print(os.path.dirname(nanobind.__file__))")
EXT_SUFFIX=$("$PY" -c "import sysconfig; print(sysconfig.get_config_var('EXT_SUFFIX'))")

BINDINGS="$ROOT/bindings/python/src"
OUT="nucleation${EXT_SUFFIX}"

# bridge-full's static lib is big enough that GNU ld chokes on it
# ("failed to set dynamic section sizes: bad value"); use lld on Linux.
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
    DYLD_LIBRARY_PATH="$ROOT/target/release" "$PY" main.py
else
    LD_LIBRARY_PATH="$ROOT/target/release" "$PY" main.py
fi

rm -f "$OUT"
