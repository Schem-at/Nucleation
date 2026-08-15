#!/usr/bin/env bash
# Run the exact sources embedded in docs/features/basics.md.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"

WORK_DIR="$(mktemp -d /tmp/nucleation-basics-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
mkdir -p "$WORK_DIR/python" "$WORK_DIR/javascript/node_modules" "$WORK_DIR/rust"

(
  cd "$WORK_DIR/python"
  "$REPO_ROOT/.venv/bin/python" "$REPO_ROOT/examples/readme/basics/basics.py"
) >"${WORK_DIR}/python.log" 2>&1 < /dev/null

(
  cd "$WORK_DIR/rust"
  cargo run --quiet \
    --manifest-path "$REPO_ROOT/examples/readme/basics/rust/Cargo.toml" \
    --target-dir "$REPO_ROOT/target/basics-docs"
) >"${WORK_DIR}/rust.log" 2>&1 < /dev/null

if ! ./tools/package-npm.sh dist/npm >"${WORK_DIR}/package.log" 2>&1; then
  cat "${WORK_DIR}/package.log"
  exit 1
fi
cp "$REPO_ROOT/examples/readme/basics/basics.mjs" "$WORK_DIR/javascript/basics.mjs"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript"
  node basics.mjs >javascript.log 2>&1
)

grep -q "Basics Python examples: OK" "${WORK_DIR}/python.log"
grep -q "Basics JavaScript examples: OK" "${WORK_DIR}/javascript/javascript.log"
grep -q "Basics Rust examples: OK" "${WORK_DIR}/rust.log"

echo "Basics docs examples passed: Python, JavaScript, Rust"
