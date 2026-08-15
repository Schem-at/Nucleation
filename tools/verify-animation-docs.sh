#!/usr/bin/env bash
# Run the sources embedded in docs/features/animation.md and regenerate its hero in /tmp.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"

WORK_DIR="$(mktemp -d /tmp/nucleation-animation-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
mkdir -p "$WORK_DIR/python" "$WORK_DIR/javascript/node_modules" "$WORK_DIR/workshop"

(
  cd "$WORK_DIR/python"
  "$REPO_ROOT/.venv/bin/python" "$REPO_ROOT/examples/readme/animation/engine.py"
) >"$WORK_DIR/python.log" 2>&1

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp "$REPO_ROOT/examples/readme/animation/engine.mjs" "$WORK_DIR/javascript/engine.mjs"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript"
  node engine.mjs
) >"$WORK_DIR/javascript.log" 2>&1

cargo test --quiet --test animation_docs_examples \
  >"$WORK_DIR/rust.log" 2>&1

NUCLEATION_OUT="$WORK_DIR/workshop/workshop.gif" \
NUCLEATION_SCHEM_OUT="$WORK_DIR/workshop/workshop.schem" \
  .venv/bin/python examples/readme/animation/workshop.py \
  >"$WORK_DIR/workshop.log" 2>&1

"$REPO_ROOT/.venv/bin/python" - \
  "$WORK_DIR/workshop/workshop.gif" \
  "$WORK_DIR/workshop/workshop.schem" \
  "$REPO_ROOT/docs/downloads/readme/animation/workshop.schem" <<'PY'
from pathlib import Path
import sys

from nucleation import Diff, Schematic

data = Path(sys.argv[1]).read_bytes()
assert data[:6] in (b"GIF87a", b"GIF89a")
width = int.from_bytes(data[6:8], "little")
height = int.from_bytes(data[8:10], "little")
assert (width, height) == (420, 420), (width, height)

generated = Schematic.load_from_file(sys.argv[2])
published = Schematic.load_from_file(sys.argv[3])
assert Diff.compute(generated, published, "exact").distance() == 0
assert generated.get_entities_json() == published.get_entities_json()
PY

grep -q "Animation engine Python example: OK" "$WORK_DIR/python.log"
grep -q "Animation engine JavaScript example: OK" "$WORK_DIR/javascript.log"
grep -q "rendered 73 frames" "$WORK_DIR/workshop.log"

echo "Animation docs passed: Python, JavaScript, Rust, and 73 rendered GIF frames"
