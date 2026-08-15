#!/usr/bin/env bash
# Execute all guide sources, mesh the shared fixture, and regenerate media.
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
WORK_DIR="$(mktemp -d /tmp/nucleation-meshing-rendering-docs.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
PACK_PATH="${NUCLEATION_PACK:-$REPO_ROOT/render_work/pack.zip}"
mkdir -p "$WORK_DIR/python/work" "$WORK_DIR/javascript/work/node_modules" "$WORK_DIR/rust/work" "$WORK_DIR/media"

(
  cd "$WORK_DIR/python/work"
  NUCLEATION_PACK="$PACK_PATH" \
  MESH_RENDER_GLB_OUT="$WORK_DIR/python/render-lab.glb" \
  MESH_RENDER_SCHEM_OUT="$WORK_DIR/python/render-lab.schem" \
  MESH_RENDER_PNG_OUT="$WORK_DIR/python/render-lab.png" \
    "$REPO_ROOT/.venv/bin/python" "$REPO_ROOT/examples/readme/meshing-and-rendering/mesh_render.py"
) >"$WORK_DIR/python.log" 2>&1

if ! ./tools/package-npm.sh dist/npm >"$WORK_DIR/package.log" 2>&1; then
  cat "$WORK_DIR/package.log"
  exit 1
fi
cp examples/readme/meshing-and-rendering/mesh_render.mjs "$WORK_DIR/javascript/work/"
ln -s "$REPO_ROOT/dist/npm" "$WORK_DIR/javascript/work/node_modules/nucleation"
(
  cd "$WORK_DIR/javascript/work"
  NUCLEATION_PACK="$PACK_PATH" \
  MESH_RENDER_GLB_OUT="$WORK_DIR/javascript/render-lab.glb" \
  MESH_RENDER_SCHEM_OUT="$WORK_DIR/javascript/render-lab.schem" \
    node mesh_render.mjs
) >"$WORK_DIR/javascript.log" 2>&1

(
  cd "$WORK_DIR/rust/work"
  NUCLEATION_PACK="$PACK_PATH" \
  MESH_RENDER_GLB_OUT="$WORK_DIR/rust/render-lab.glb" \
  MESH_RENDER_SCHEM_OUT="$WORK_DIR/rust/render-lab.schem" \
  MESH_RENDER_PNG_OUT="$WORK_DIR/rust/render-lab.png" \
    cargo run --quiet \
      --manifest-path "$REPO_ROOT/examples/readme/meshing-and-rendering/rust/Cargo.toml" \
      --target-dir "$REPO_ROOT/target/meshing-rendering-docs"
) >"$WORK_DIR/rust.log" 2>&1

NUCLEATION_PACK="$PACK_PATH" \
NUCLEATION_STILL_OUT="$WORK_DIR/media/render-lab.png" \
NUCLEATION_OUT="$WORK_DIR/media/render-lab-turntable.gif" \
NUCLEATION_SCHEM_OUT="$WORK_DIR/media/render-lab.schem" \
NUCLEATION_GLB_OUT="$WORK_DIR/media/render-lab.glb" \
  .venv/bin/python examples/readme/meshing-and-rendering/generate.py \
  >"$WORK_DIR/media.log" 2>&1

.venv/bin/python - "$WORK_DIR" <<'PY'
from pathlib import Path
import sys

from nucleation import Diff, Schematic

root = Path(sys.argv[1])
source = Schematic.load_from_file(str(root / "python/render-lab.schem"))
assert source.block_count() == 308
size = source.tight_dimensions()
assert (size.x, size.y, size.z) == (11, 5, 9)
for relative in ("javascript/render-lab.schem", "rust/render-lab.schem", "media/render-lab.schem"):
    other = Schematic.load_from_file(str(root / relative))
    assert Diff.compute(source, other, "exact").distance() == 0, relative

for binding in ("python", "javascript", "rust", "media"):
    glb = (root / binding / "render-lab.glb").read_bytes()
    assert glb[:4] == b"glTF", binding
    assert len(glb) > 50_000, binding

for binding in ("python", "rust"):
    png = (root / binding / "render-lab.png").read_bytes()
    assert png[:8] == b"\x89PNG\r\n\x1a\n"
    assert (int.from_bytes(png[16:20], "big"), int.from_bytes(png[20:24], "big")) == (640, 440)

still = (root / "media/render-lab.png").read_bytes()
assert (int.from_bytes(still[16:20], "big"), int.from_bytes(still[20:24], "big")) == (720, 480)
gif = (root / "media/render-lab-turntable.gif").read_bytes()
assert gif[:6] in (b"GIF87a", b"GIF89a")
assert (int.from_bytes(gif[6:8], "little"), int.from_bytes(gif[8:10], "little")) == (460, 380)
PY

grep -q "Meshing Python example: OK (2320 vertices, 1160 triangles)" "$WORK_DIR/python.log"
grep -q "Meshing JavaScript example: OK (2320 vertices, 1160 triangles)" "$WORK_DIR/javascript.log"
grep -q "Meshing Rust example: OK (2320 vertices, 1160 triangles)" "$WORK_DIR/rust.log"
grep -q "rendered 48 frames" "$WORK_DIR/media.log"

echo "Meshing/rendering docs passed: three meshes, exact fixture parity, GLBs, two native renders, and 48 GIF frames"
