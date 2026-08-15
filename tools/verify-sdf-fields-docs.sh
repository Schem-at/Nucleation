#!/usr/bin/env bash
# Execute all guide sources and regenerate the shared SDF/field fixture and media.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
WORK_DIR="$(mktemp -d /tmp/nucleation-sdf-fields-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
PACK_PATH="${NUCLEATION_PACK:-$REPO_ROOT/render_work/pack.zip}"
mkdir -p "$WORK_DIR/python/work" "$WORK_DIR/javascript/work/node_modules" "$WORK_DIR/rust/work" "$WORK_DIR/media"

(
  cd "$WORK_DIR/python/work"
  SDF_FIELDS_OUT="$WORK_DIR/python/field-observatory.schem" \
    "$REPO_ROOT/.venv/bin/python" "$REPO_ROOT/examples/readme/sdf-and-fields/sdf_fields.py"
) >"$WORK_DIR/python.log" 2>&1

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp examples/readme/sdf-and-fields/sdf_fields.mjs "$WORK_DIR/javascript/work/"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/work/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript/work"
  SDF_FIELDS_OUT="$WORK_DIR/javascript/field-observatory.schem" node sdf_fields.mjs
) >"$WORK_DIR/javascript.log" 2>&1

(
  cd "$WORK_DIR/rust/work"
  SDF_FIELDS_OUT="$WORK_DIR/rust/field-observatory.schem" \
    cargo run --quiet \
      --manifest-path "$REPO_ROOT/examples/readme/sdf-and-fields/rust/Cargo.toml" \
      --target-dir "$REPO_ROOT/target/sdf-fields-docs"
) >"$WORK_DIR/rust.log" 2>&1

NUCLEATION_PACK="$PACK_PATH" \
NUCLEATION_STILL_OUT="$WORK_DIR/media/field-observatory.png" \
NUCLEATION_OUT="$WORK_DIR/media/field-observatory-build.gif" \
NUCLEATION_SCHEM_OUT="$WORK_DIR/media/field-observatory.schem" \
  .venv/bin/python examples/readme/sdf-and-fields/generate.py \
  >"$WORK_DIR/media.log" 2>&1

.venv/bin/python - "$WORK_DIR" <<'PY'
from pathlib import Path
import sys

from nucleation import Diff, Schematic

root = Path(sys.argv[1])
source = Schematic.load_from_file(str(root / "python/field-observatory.schem"))
assert source.block_count() == 3_175
size = source.tight_dimensions()
assert (size.x, size.y, size.z) == (22, 14, 24)
for relative in (
    "javascript/field-observatory.schem",
    "rust/field-observatory.schem",
    "media/field-observatory.schem",
):
    other = Schematic.load_from_file(str(root / relative))
    assert Diff.compute(source, other, "exact").distance() == 0, relative

png = (root / "media/field-observatory.png").read_bytes()
assert png[:8] == b"\x89PNG\r\n\x1a\n"
assert (int.from_bytes(png[16:20], "big"), int.from_bytes(png[20:24], "big")) == (720, 520)
gif = (root / "media/field-observatory-build.gif").read_bytes()
assert gif[:6] in (b"GIF87a", b"GIF89a")
assert (int.from_bytes(gif[6:8], "little"), int.from_bytes(gif[8:10], "little")) == (500, 420)
PY

grep -q "SDFs and fields Python example: OK" "$WORK_DIR/python.log"
grep -q "SDFs and fields JavaScript example: OK" "$WORK_DIR/javascript.log"
grep -q "SDFs and fields Rust example: OK" "$WORK_DIR/rust.log"
grep -Eq "rendered [4-9][0-9] frames" "$WORK_DIR/media.log"

echo "SDF/field docs passed: three bindings, exact 3,175-block parity, still, and animated build"
