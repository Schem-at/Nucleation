#!/usr/bin/env bash
# Run the sources embedded in docs/features/fast-generation.md and regenerate its media.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
WORK_DIR="$(mktemp -d /tmp/nucleation-fast-generation-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
mkdir -p "$WORK_DIR/python" "$WORK_DIR/javascript/node_modules" "$WORK_DIR/rust" "$WORK_DIR/media"

FAST_GENERATION_OUT="$WORK_DIR/python/campus.schem" \
  .venv/bin/python examples/readme/fast-generation/fast_generation.py \
  >"$WORK_DIR/python.log" 2>&1

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp examples/readme/fast-generation/fast_generation.mjs "$WORK_DIR/javascript/"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript"
  FAST_GENERATION_OUT="$WORK_DIR/javascript/campus.schem" node fast_generation.mjs
) >"$WORK_DIR/javascript.log" 2>&1

FAST_GENERATION_OUT="$WORK_DIR/rust/campus.schem" \
  cargo run --quiet \
    --manifest-path examples/readme/fast-generation/rust/Cargo.toml \
    --target-dir target/fast-generation-docs \
  >"$WORK_DIR/rust.log" 2>&1

NUCLEATION_OUT="$WORK_DIR/media/bulk-campus.gif" \
NUCLEATION_SCHEM_OUT="$WORK_DIR/media/bulk-campus.schem" \
  .venv/bin/python examples/readme/fast-generation/generate.py \
  >"$WORK_DIR/media.log" 2>&1

.venv/bin/python - "$WORK_DIR" <<'PY'
from pathlib import Path
import sys

from nucleation import Diff, Schematic

root = Path(sys.argv[1])
python = Schematic.load_from_file(str(root / "python/campus.schem"))
for relative in ("javascript/campus.schem", "rust/campus.schem", "media/bulk-campus.schem"):
    other = Schematic.load_from_file(str(root / relative))
    assert Diff.compute(python, other, "exact").distance() == 0, relative
    assert other.block_count() == 6_926, relative

gif = (root / "media/bulk-campus.gif").read_bytes()
assert gif[:6] in (b"GIF87a", b"GIF89a")
assert int.from_bytes(gif[6:8], "little") == 460
assert int.from_bytes(gif[8:10], "little") == 380
PY

grep -q "Fast generation Python example: OK" "$WORK_DIR/python.log"
grep -q "Fast generation JavaScript example: OK" "$WORK_DIR/javascript.log"
grep -q "Fast generation Rust example: OK" "$WORK_DIR/rust.log"
grep -q "rendered 54 frames" "$WORK_DIR/media.log"

echo "Fast-generation docs passed: three bindings, exact schematic parity, 54 GIF frames"
