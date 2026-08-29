#!/usr/bin/env bash
# Build-animation parity: the native fixtures under tests/fixtures/build-animation
# against the packaged WASM/JS engine. The fixtures are regenerated with
# `NUCLEATION_WRITE_FIXTURES=1 cargo test --test build_animation_parity`.
set -euo pipefail
cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
WORK_DIR="$(mktemp -d /tmp/nucleation-build-animation.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
mkdir -p "$WORK_DIR/javascript/node_modules" "$WORK_DIR/javascript/fixtures"
# The animated-GLB checks need the vanilla pack the docs generators use.
if [[ -z "${NUCLEATION_PACK:-}" && -f "$REPO_ROOT/render_work/pack.zip" ]]; then
  export NUCLEATION_PACK="$REPO_ROOT/render_work/pack.zip"
fi

cargo test --quiet --test build_animation_parity >"$WORK_DIR/rust.log" 2>&1 \
  || { cat "$WORK_DIR/rust.log"; exit 1; }

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp "$REPO_ROOT/tests/node_build_animation_test.mjs" "$WORK_DIR/javascript/"
cp -R "$REPO_ROOT/tests/fixtures/build-animation" "$WORK_DIR/javascript/fixtures/"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/node_modules/nucleation"
( cd "$WORK_DIR/javascript" && node --test node_build_animation_test.mjs ) >"$WORK_DIR/javascript.log" 2>&1 \
  || { cat "$WORK_DIR/javascript.log"; exit 1; }

"$REPO_ROOT/.venv/bin/python" "$REPO_ROOT/tests/python_build_animation_test.py" >"$WORK_DIR/python.log" 2>&1 \
  || { cat "$WORK_DIR/python.log"; exit 1; }
grep -q "Build-animation Python parity: OK" "$WORK_DIR/python.log"

echo "Build-animation parity passed: Rust fixtures, WASM/JS, Python"
