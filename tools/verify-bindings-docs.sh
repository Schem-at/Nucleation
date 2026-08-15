#!/usr/bin/env bash
# Execute the guide's three primary bindings and regenerate its media.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
WORK_DIR="$(mktemp -d /tmp/nucleation-bindings-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
mkdir -p "$WORK_DIR/python" "$WORK_DIR/javascript/node_modules" "$WORK_DIR/rust" "$WORK_DIR/media"

BINDINGS_OUT="$WORK_DIR/python/stack.schem" \
  .venv/bin/python examples/readme/bindings-and-languages/bindings.py \
  >"$WORK_DIR/python.log" 2>&1

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp examples/readme/bindings-and-languages/bindings.mjs "$WORK_DIR/javascript/"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript"
  BINDINGS_OUT="$WORK_DIR/javascript/stack.schem" node bindings.mjs
) >"$WORK_DIR/javascript.log" 2>&1

BINDINGS_OUT="$WORK_DIR/rust/stack.schem" \
  cargo run --quiet \
    --manifest-path examples/readme/bindings-and-languages/rust/Cargo.toml \
    --target-dir target/bindings-docs \
  >"$WORK_DIR/rust.log" 2>&1

NUCLEATION_OUT="$WORK_DIR/media/binding-stack.gif" \
NUCLEATION_STILL_OUT="$WORK_DIR/media/binding-stack.png" \
NUCLEATION_SCHEM_OUT="$WORK_DIR/media/binding-stack.schem" \
  .venv/bin/python examples/readme/bindings-and-languages/generate.py \
  >"$WORK_DIR/media.log" 2>&1

.venv/bin/python - "$WORK_DIR" <<'PY'
from pathlib import Path
import sys

from nucleation import Diff, Schematic

root = Path(sys.argv[1])
python = Schematic.load_from_file(str(root / "python/stack.schem"))
for relative in ("javascript/stack.schem", "rust/stack.schem", "media/binding-stack.schem"):
    other = Schematic.load_from_file(str(root / relative))
    assert Diff.compute(python, other, "exact").distance() == 0, relative
    assert other.block_count() == 84, relative
    size = other.tight_dimensions()
    assert (size.x, size.y, size.z) == (7, 4, 7), relative

gif = (root / "media/binding-stack.gif").read_bytes()
assert gif[:6] in (b"GIF87a", b"GIF89a")
assert (int.from_bytes(gif[6:8], "little"), int.from_bytes(gif[8:10], "little")) == (460, 380)
png = (root / "media/binding-stack.png").read_bytes()
assert png[:8] == b"\x89PNG\r\n\x1a\n"
assert (int.from_bytes(png[16:20], "big"), int.from_bytes(png[20:24], "big")) == (560, 420)
PY

grep -q "Bindings Python example: OK" "$WORK_DIR/python.log"
grep -q "Bindings JavaScript example: OK" "$WORK_DIR/javascript.log"
grep -q "Bindings Rust example: OK" "$WORK_DIR/rust.log"
grep -q "rendered 57 frames" "$WORK_DIR/media.log"

echo "Bindings docs passed: Python, JavaScript, Rust, exact schematic parity, PNG, and 57 GIF frames"
