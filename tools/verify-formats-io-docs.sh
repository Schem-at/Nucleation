#!/usr/bin/env bash
# Execute every source in docs/features/formats-and-io.md and regenerate its artifacts.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
WORK_DIR="$(mktemp -d /tmp/nucleation-formats-io-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
mkdir -p "$WORK_DIR/python/work" "$WORK_DIR/javascript/work/node_modules" "$WORK_DIR/rust/work" "$WORK_DIR/media"

(
  cd "$WORK_DIR/python/work"
  FORMATS_IO_OUT_DIR="$WORK_DIR/python/out" \
    "$REPO_ROOT/.venv/bin/python" "$REPO_ROOT/examples/readme/formats-and-io/formats_io.py"
) >"$WORK_DIR/python.log" 2>&1

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp examples/readme/formats-and-io/formats_io.mjs "$WORK_DIR/javascript/work/"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/work/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript/work"
  FORMATS_IO_OUT_DIR="$WORK_DIR/javascript/out" node formats_io.mjs
) >"$WORK_DIR/javascript.log" 2>&1

(
  cd "$WORK_DIR/rust/work"
  FORMATS_IO_OUT_DIR="$WORK_DIR/rust/out" \
    cargo run --quiet \
      --manifest-path "$REPO_ROOT/examples/readme/formats-and-io/rust/Cargo.toml" \
      --target-dir "$REPO_ROOT/target/formats-io-docs"
) >"$WORK_DIR/rust.log" 2>&1

NUCLEATION_OUT="$WORK_DIR/media/round-trip-build.gif" \
NUCLEATION_STILL_OUT="$WORK_DIR/media/format-fixture.png" \
NUCLEATION_DOWNLOAD_DIR="$WORK_DIR/media/downloads" \
  .venv/bin/python examples/readme/formats-and-io/generate.py \
  >"$WORK_DIR/media.log" 2>&1

.venv/bin/python - "$WORK_DIR" <<'PY'
from pathlib import Path
import sys

from nucleation import Diff, Schematic

root = Path(sys.argv[1])
extensions = (".litematic", ".schem", ".snbt", ".nusn", ".mcstructure")
source = Schematic.load_from_file(str(root / "python/out/round-trip.litematic"))
assert source.block_count() == 19

for binding in ("python", "javascript", "rust"):
    binding_root = root / binding / "out"
    same_binding = Schematic.load_from_file(str(binding_root / "round-trip.litematic"))
    assert Diff.compute(source, same_binding, "exact").distance() == 0, binding
    for extension in extensions:
        artifact = binding_root / f"round-trip{extension}"
        loaded = Schematic.load_from_file(str(artifact))
        assert loaded.block_count() == 19, (binding, extension)

for extension in extensions:
    generated = Schematic.load_from_file(str(root / "media/downloads" / f"round-trip{extension}"))
    expected = {".schem": 1, ".mcstructure": 3}.get(extension, 0)
    assert Diff.compute(source, generated, "exact").distance() == expected, extension

gif = (root / "media/round-trip-build.gif").read_bytes()
assert gif[:6] in (b"GIF87a", b"GIF89a")
assert (int.from_bytes(gif[6:8], "little"), int.from_bytes(gif[8:10], "little")) == (460, 340)
png = (root / "media/format-fixture.png").read_bytes()
assert png[:8] == b"\x89PNG\r\n\x1a\n"
assert (int.from_bytes(png[16:20], "big"), int.from_bytes(png[20:24], "big")) == (600, 380)
PY

grep -q "Formats and I/O Python example: OK" "$WORK_DIR/python.log"
grep -q "Formats and I/O JavaScript example: OK" "$WORK_DIR/javascript.log"
grep -q "Formats and I/O Rust example: OK" "$WORK_DIR/rust.log"
grep -q "rendered 53 frames" "$WORK_DIR/media.log"

echo "Formats-and-I/O docs passed: three bindings, five formats each, exact source parity, PNG, and 53 GIF frames"
