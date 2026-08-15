#!/usr/bin/env bash
# Execute all guide sources and regenerate the shared color fixture and media.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
WORK_DIR="$(mktemp -d /tmp/nucleation-palettes-color-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
PACK_PATH="${NUCLEATION_PACK:-$REPO_ROOT/render_work/pack.zip}"
mkdir -p "$WORK_DIR/python/work" "$WORK_DIR/javascript/work/node_modules" "$WORK_DIR/rust/work" "$WORK_DIR/media"

(
  cd "$WORK_DIR/python/work"
  PALETTES_COLOR_OUT="$WORK_DIR/python/color-atlas.schem" \
    "$REPO_ROOT/.venv/bin/python" "$REPO_ROOT/examples/readme/palettes-and-color/palettes_color.py"
) >"$WORK_DIR/python.log" 2>&1

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp examples/readme/palettes-and-color/palettes_color.mjs "$WORK_DIR/javascript/work/"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/work/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript/work"
  PALETTES_COLOR_OUT="$WORK_DIR/javascript/color-atlas.schem" node palettes_color.mjs
) >"$WORK_DIR/javascript.log" 2>&1

(
  cd "$WORK_DIR/rust/work"
  PALETTES_COLOR_OUT="$WORK_DIR/rust/color-atlas.schem" \
    cargo run --quiet \
      --manifest-path "$REPO_ROOT/examples/readme/palettes-and-color/rust/Cargo.toml" \
      --target-dir "$REPO_ROOT/target/palettes-color-docs"
) >"$WORK_DIR/rust.log" 2>&1

NUCLEATION_PACK="$PACK_PATH" \
NUCLEATION_STILL_OUT="$WORK_DIR/media/color-atlas.png" \
NUCLEATION_OUT="$WORK_DIR/media/color-atlas-build.gif" \
NUCLEATION_SCHEM_OUT="$WORK_DIR/media/color-atlas.schem" \
  .venv/bin/python examples/readme/palettes-and-color/generate.py \
  >"$WORK_DIR/media.log" 2>&1

.venv/bin/python - "$WORK_DIR" <<'PY'
from pathlib import Path
import sys

from nucleation import Diff, Schematic

root = Path(sys.argv[1])
source = Schematic.load_from_file(str(root / "python/color-atlas.schem"))
assert source.block_count() == 448
size = source.tight_dimensions()
assert (size.x, size.y, size.z) == (32, 16, 1)
for relative in ("javascript/color-atlas.schem", "rust/color-atlas.schem", "media/color-atlas.schem"):
    other = Schematic.load_from_file(str(root / relative))
    assert Diff.compute(source, other, "exact").distance() == 0, relative

png = (root / "media/color-atlas.png").read_bytes()
assert png[:8] == b"\x89PNG\r\n\x1a\n"
assert (int.from_bytes(png[16:20], "big"), int.from_bytes(png[20:24], "big")) == (720, 480)
gif = (root / "media/color-atlas-build.gif").read_bytes()
assert gif[:6] in (b"GIF87a", b"GIF89a")
assert (int.from_bytes(gif[6:8], "little"), int.from_bytes(gif[8:10], "little")) == (500, 360)
PY

grep -q "Palettes and color Python example: OK" "$WORK_DIR/python.log"
grep -q "Palettes and color JavaScript example: OK" "$WORK_DIR/javascript.log"
grep -q "Palettes and color Rust example: OK" "$WORK_DIR/rust.log"
grep -Eq "rendered [4-9][0-9] frames" "$WORK_DIR/media.log"

echo "Palettes/color docs passed: three bindings, exact 448-block parity, still, and animated build"
