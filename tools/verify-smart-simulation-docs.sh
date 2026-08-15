#!/usr/bin/env bash
# Execute the guide's three sources and regenerate all media.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
WORK_DIR="$(mktemp -d /tmp/nucleation-smart-simulation-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
mkdir -p "$WORK_DIR/python" "$WORK_DIR/javascript/node_modules" "$WORK_DIR/rust" "$WORK_DIR/media"

SMART_SIMULATION_OUT="$WORK_DIR/python/circuit.schem" \
  .venv/bin/python examples/readme/smart-simulation/smart_simulation.py \
  >"$WORK_DIR/python.log" 2>&1

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp examples/readme/smart-simulation/smart_simulation.mjs "$WORK_DIR/javascript/"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript"
  SMART_SIMULATION_OUT="$WORK_DIR/javascript/circuit.schem" node smart_simulation.mjs
) >"$WORK_DIR/javascript.log" 2>&1

SMART_SIMULATION_OUT="$WORK_DIR/rust/circuit.schem" \
  cargo run --quiet \
    --manifest-path examples/readme/smart-simulation/rust/Cargo.toml \
    --target-dir target/smart-simulation-docs \
  >"$WORK_DIR/rust.log" 2>&1

SMART_SIMULATION_OUT="$WORK_DIR/media/source.schem" \
NUCLEATION_OUT="$WORK_DIR/media/smart-circuit.gif" \
NUCLEATION_IDLE_OUT="$WORK_DIR/media/circuit-idle.png" \
NUCLEATION_POWERED_OUT="$WORK_DIR/media/circuit-powered.png" \
NUCLEATION_SCHEM_OUT="$WORK_DIR/media/smart-circuit.schem" \
  .venv/bin/python examples/readme/smart-simulation/generate.py \
  >"$WORK_DIR/media.log" 2>&1

.venv/bin/python - "$WORK_DIR" <<'PY'
from pathlib import Path
import json
import sys

from nucleation import Diff, Schematic

root = Path(sys.argv[1])
python = Schematic.load_from_file(str(root / "python/circuit.schem"))
for relative in (
    "javascript/circuit.schem",
    "rust/circuit.schem",
    "media/smart-circuit.schem",
):
    other = Schematic.load_from_file(str(root / relative))
    assert Diff.compute(python, other, "exact").distance() == 0, relative
    assert other.block_count() == 36, relative

assert python.get_block_string(3, 1, 0) == (
    "minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]"
)
barrel = json.loads(python.get_block_entity_json(0, 1, 2))
assert barrel["nbt"]["Items"]["List"][0]["Compound"]["id"]["String"] == "minecraft:iron_ingot"

gif = (root / "media/smart-circuit.gif").read_bytes()
assert gif[:6] in (b"GIF87a", b"GIF89a")
assert (int.from_bytes(gif[6:8], "little"), int.from_bytes(gif[8:10], "little")) == (460, 300)
for name in ("circuit-idle.png", "circuit-powered.png"):
    png = (root / "media" / name).read_bytes()
    assert png[:8] == b"\x89PNG\r\n\x1a\n"
    assert (int.from_bytes(png[16:20], "big"), int.from_bytes(png[20:24], "big")) == (560, 300)
PY

grep -q "Smart simulation Python example: OK" "$WORK_DIR/python.log"
grep -q "Smart simulation JavaScript example: OK" "$WORK_DIR/javascript.log"
grep -q "Smart simulation Rust example: OK" "$WORK_DIR/rust.log"
grep -q "rendered 56 frames" "$WORK_DIR/media.log"

echo "Smart-simulation docs passed: three bindings, exact parity, two simulators, two PNGs, and 56 GIF frames"
