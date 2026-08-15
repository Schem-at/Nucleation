#!/usr/bin/env bash
# Run every source embedded in docs/features/shapes-and-brushes.md and regenerate its media.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
WORK_DIR="$(mktemp -d /tmp/nucleation-shapes-brushes-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
mkdir -p "$WORK_DIR/python" "$WORK_DIR/javascript/node_modules" "$WORK_DIR/rust" "$WORK_DIR/media"

SHAPES_BRUSHES_OUT="$WORK_DIR/python/garden.schem" \
  .venv/bin/python examples/readme/shapes-brushes/shapes_brushes.py \
  >"$WORK_DIR/python.log" 2>&1

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp examples/readme/shapes-brushes/shapes_brushes.mjs "$WORK_DIR/javascript/"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript"
  SHAPES_BRUSHES_OUT="$WORK_DIR/javascript/garden.schem" node shapes_brushes.mjs
) >"$WORK_DIR/javascript.log" 2>&1

SHAPES_BRUSHES_OUT="$WORK_DIR/rust/garden.schem" \
  cargo run --quiet \
    --manifest-path examples/readme/shapes-brushes/rust/Cargo.toml \
    --target-dir target/shapes-brushes-docs \
  >"$WORK_DIR/rust.log" 2>&1

NUCLEATION_OUT="$WORK_DIR/media/torus-sweep.gif" \
NUCLEATION_STILL_OUT="$WORK_DIR/media/orbital-garden.png" \
NUCLEATION_SCHEM_OUT="$WORK_DIR/media/orbital-garden.schem" \
  .venv/bin/python examples/readme/shapes-brushes/generate.py \
  >"$WORK_DIR/media.log" 2>&1

.venv/bin/python - "$WORK_DIR" <<'PY'
from pathlib import Path
import sys

from nucleation import Diff, Schematic

root = Path(sys.argv[1])
python = Schematic.load_from_file(str(root / "python/garden.schem"))
for relative in ("javascript/garden.schem", "rust/garden.schem", "media/orbital-garden.schem"):
    other = Schematic.load_from_file(str(root / relative))
    assert Diff.compute(python, other, "exact").distance() == 0, relative
    assert other.block_count() == 6_627, relative

gif = (root / "media/torus-sweep.gif").read_bytes()
assert gif[:6] in (b"GIF87a", b"GIF89a")
assert (int.from_bytes(gif[6:8], "little"), int.from_bytes(gif[8:10], "little")) == (460, 380)

png = (root / "media/orbital-garden.png").read_bytes()
assert png[:8] == b"\x89PNG\r\n\x1a\n"
assert (int.from_bytes(png[16:20], "big"), int.from_bytes(png[20:24], "big")) == (700, 500)
PY

grep -q "Shapes and brushes Python example: OK" "$WORK_DIR/python.log"
grep -q "Shapes and brushes JavaScript example: OK" "$WORK_DIR/javascript.log"
grep -q "Shapes and brushes Rust example: OK" "$WORK_DIR/rust.log"
grep -q "rendered 56 frames" "$WORK_DIR/media.log"

echo "Shapes-and-brushes docs passed: three bindings, exact schematic parity, generated PNG and 56 GIF frames"
