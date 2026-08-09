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

# ---------------------------------------------------------------- input guard
# A release wasm build is minutes, and this script gets called from `npm run
# dev` (via sync-engine) and from the dev-loop tiers, where the inputs usually
# have not changed at all. Stamp the inputs and skip the whole assembly when
# they match.
#
# The stamp covers everything that can change the emitted package: the Rust
# sources, the manifests, the generated JS glue, the npm veneers, and the
# feature set (which selects a different wasm binary entirely). Paths+sizes
# +mtimes rather than content hashes, so the guard itself stays milliseconds.
#
# Force a rebuild with NUCLEATION_FORCE_REBUILD=1.
# BSD stat (macOS) and GNU stat (CI) take different flags, and a silently empty
# stamp would make the guard skip rebuilds it should perform — so pick the right
# one up front and fail loudly if neither works.
if stat -f '%N %z %m' Cargo.toml >/dev/null 2>&1; then
  STAT_FLAG=-f; STAT_FMT='%N %z %m'          # BSD / macOS
elif stat -c '%n %s %Y' Cargo.toml >/dev/null 2>&1; then
  STAT_FLAG=-c; STAT_FMT='%n %s %Y'          # GNU / Linux
else
  echo "package-npm.sh: cannot determine stat(1) flavour; refusing to guess" >&2
  exit 1
fi

stamp_inputs() {
  printf 'features=%s\n' "$FEATURES"
  find src crates bindings/js bindings/npm build.rs Cargo.toml Cargo.lock \
    -type f \( -name '*.rs' -o -name '*.toml' -o -name '*.lock' -o -name '*.mjs' \
               -o -name '*.ts' -o -name '*.mts' -o -name '*.json' -o -name '*.md' \) \
    -exec stat "$STAT_FLAG" "$STAT_FMT" {} + 2>/dev/null | sort
}

STAMP_FILE="$OUT/.build-stamp"
STAMP="$(stamp_inputs | shasum -a 256 | cut -d' ' -f1)"

if [[ -z "${NUCLEATION_FORCE_REBUILD:-}" \
      && -f "$STAMP_FILE" && -f "$OUT/nucleation.wasm" \
      && "$(cat "$STAMP_FILE" 2>/dev/null)" == "$STAMP" ]]; then
  echo "npm package in $OUT is up to date (inputs unchanged) — skipping."
  echo "  force with NUCLEATION_FORCE_REBUILD=1"
  exit 0
fi

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

printf '%s' "$STAMP" > "$STAMP_FILE"

echo "npm package assembled in $OUT (set version + npm publish from there)"
