#!/usr/bin/env bash
# Assemble the publishable npm package from the generated JS glue + the wasm binary.
# Usage: tools/package-npm.sh [out_dir]   (default: dist/npm)
set -euo pipefail
cd "$(dirname "$0")/.."

OUT="${1:-dist/npm}"

# simulation+meshing ride along (schemati's flow runs redstone sim in the
# browser); rendering (wgpu needs wasm-bindgen glue) and scripting (luajit
# can't target wasm) stay out of the wasm build.
# `mc-tick` is the headless tick engine (TickSimulation); `simulation` is the
# separate MCHPRS-backed redstone world and is NOT what the browser apps use.
# Naming the wrong one produces an engine that loads, meshes, and silently has
# no TickSimulation at all — which is how a whole app came up dead.
FEATURES="${NUCLEATION_WASM_FEATURES:-bridge,mc-tick,meshing}"
cargo build --release --target wasm32-unknown-unknown --lib --features "$FEATURES"

rm -rf "$OUT"
mkdir -p "$OUT"
cp bindings/js/*.mjs bindings/js/*.d.ts bindings/js/*.d.mts "$OUT"/
cp target/wasm32-unknown-unknown/release/nucleation.wasm "$OUT/"
cp bindings/npm/package.json "$OUT/"
cp bindings/npm/README.md "$OUT/"
# Hand-written veneers overlaying the generated core (mirrors of the Python
# bindings/python/nucleation/*.py veneers; exported as e.g. `nucleation/design`).
# They live in veneer/ because the repo's macOS checkouts are case-insensitive:
# a top-level design.mjs would collide with the generated Design.mjs.
mkdir -p "$OUT/veneer"
cp bindings/npm/veneer/*.mjs bindings/npm/veneer/*.d.ts "$OUT/veneer/"

# Package-local wasm path (the committed bindings/diplomat.config.mjs points at
# target/ for the in-repo smoke tests instead).
cat > "$OUT/diplomat.config.mjs" <<'EOF'
// Isomorphic: fs.readFileSync accepts file:// URLs in Node; the browser
// branch fetches the same URL relative to the module.
export default {
  wasm_path: new URL("./nucleation.wasm", import.meta.url),
};
EOF

# The generated glue imports ../diplomat.config.mjs (it expects to sit one level below
# the config); rewrite to package-local.
sed -i.bak "s#'../diplomat.config.mjs'#'./diplomat.config.mjs'#" "$OUT/diplomat-wasm.mjs" && rm "$OUT/diplomat-wasm.mjs.bak"

echo "npm package assembled in $OUT (set version + npm publish from there)"
