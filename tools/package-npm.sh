#!/usr/bin/env bash
# Assemble the publishable npm package from the generated JS glue + the wasm binary.
# Usage: tools/package-npm.sh [out_dir]   (default: dist/npm)
set -euo pipefail
cd "$(dirname "$0")/.."

OUT="${1:-dist/npm}"

cargo build --release --target wasm32-unknown-unknown --lib --features bridge

rm -rf "$OUT"
mkdir -p "$OUT"
cp bindings/js/*.mjs bindings/js/*.d.ts "$OUT/"
cp target/wasm32-unknown-unknown/release/nucleation.wasm "$OUT/"
cp bindings/npm/package.json "$OUT/"

# Package-local wasm path (the committed bindings/diplomat.config.mjs points at
# target/ for the in-repo smoke tests instead).
cat > "$OUT/diplomat.config.mjs" <<'EOF'
import { fileURLToPath } from "node:url";
import path from "node:path";

export default {
  wasm_path: path.join(path.dirname(fileURLToPath(import.meta.url)), "nucleation.wasm"),
};
EOF

# The generated glue imports ../diplomat.config.mjs (it expects to sit one level below
# the config); rewrite to package-local.
sed -i.bak "s#'../diplomat.config.mjs'#'./diplomat.config.mjs'#" "$OUT/diplomat-wasm.mjs" && rm "$OUT/diplomat-wasm.mjs.bak"

echo "npm package assembled in $OUT (set version + npm publish from there)"
