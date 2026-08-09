/** STALE-ENGINE GUARD.
 *
 *  The bug this exists to make impossible:
 *
 *    `npm run build` used to run `tsc && vite build` WITHOUT re-syncing the
 *    engine, so `dist/engine/` kept whatever `public/engine/` happened to hold
 *    at the last build. The harness serves `dist/`, so `npm run verify` could
 *    exercise an engine several builds behind the one on disk — and it did:
 *    three gate checks went red with a router reason string ("cannot ramp
 *    between levels") that no longer exists in the current engine at all. A
 *    phantom failure attributed to the wrong lane costs more than the check is
 *    worth.
 *
 *  Two modes, because there are two ways to serve the wrong engine:
 *
 *    --write   after a sync: record the copied wasm's hash in
 *              `public/engine/BUILD.json`, which ships with it. That stamp is
 *              what the PAGE checks itself against at boot, so a browser cache
 *              serving an older wasm than the server has is caught too.
 *    --check   before a build or a verify: the three copies that must agree —
 *              `dist/npm-eda` (source of truth), `public/engine` (what a build
 *              will copy) and `dist/engine` (what the harness will serve) —
 *              compared by hash, and a loud non-zero exit naming the fix.
 */
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, existsSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, "..");
const WASM = "nucleation.wasm";
const SOURCE = path.join(root, "..", "..", "dist", "npm-eda");
const PUBLIC = path.join(root, "public", "engine");
const SERVED = path.join(root, "dist", "engine");

/** A file's identity: the first 16 hex of its sha256, plus its byte count.
 *  Both, because the byte count is the one a `HEAD` can check for free. */
function stamp(dir) {
  const file = path.join(dir, WASM);
  if (!existsSync(file)) return null;
  const bytes = readFileSync(file);
  return {
    sha256: createHash("sha256").update(bytes).digest("hex").slice(0, 16),
    bytes: bytes.length,
    mtime: statSync(file).mtime.toISOString(),
  };
}

const REBUILD =
  "  rebuild the engine (repo root):\n" +
  "    NUCLEATION_WASM_FEATURES=bridge,simulation,mc-tick,routing,hdl,meshing \\\n" +
  "      ./tools/package-npm.sh dist/npm-eda\n" +
  "  then re-sync and rebuild the app:\n" +
  "    cd apps/eda-studio && npm run sync-engine && npm run build\n";

const mode = process.argv[2] ?? "--check";

if (mode === "--write") {
  const s = stamp(PUBLIC);
  if (!s) {
    console.error(`engine-stamp: no ${WASM} in ${PUBLIC} — run \`npm run sync-engine\``);
    process.exit(1);
  }
  writeFileSync(path.join(PUBLIC, "BUILD.json"),
    `${JSON.stringify({ ...s, stampedAt: new Date().toISOString() }, null, 2)}\n`);
  console.log(`engine-stamp: public/engine is ${s.sha256} (${s.bytes} bytes, built ${s.mtime})`);
  process.exit(0);
}

const source = stamp(SOURCE);
if (!source) {
  console.error(`engine-stamp: NO ENGINE at ${SOURCE}/${WASM}.\n${REBUILD}`);
  process.exit(1);
}
const copies = [["public/engine", stamp(PUBLIC)], ["dist/engine", stamp(SERVED)]];
const stale = copies.filter(([, s]) => s && s.sha256 !== source.sha256);
const missing = copies.filter(([, s]) => !s).map(([n]) => n);

console.log(`engine-stamp: dist/npm-eda is ${source.sha256} (${source.bytes} bytes, ${source.mtime})`);
for (const [name, s] of copies) {
  console.log(`  ${s && s.sha256 === source.sha256 ? "OK  " : s ? "STALE" : "-   "} ${name}` +
    (s ? ` ${s.sha256} (${s.bytes} bytes, ${s.mtime})` : " absent"));
}
// `dist/engine` absent is fine — nothing has been built yet. `public/engine`
// absent is not: `sync-engine` has never run.
if (missing.includes("public/engine")) {
  console.error(`\nengine-stamp: public/engine is MISSING.\n${REBUILD}`);
  process.exit(1);
}
if (stale.length) {
  console.error(`\nengine-stamp: ${stale.map(([n]) => n).join(" and ")} ` +
    `${stale.length > 1 ? "are" : "is"} a DIFFERENT engine from dist/npm-eda.\n` +
    `Whatever you measure against it is measuring an engine that no longer exists.\n${REBUILD}`);
  process.exit(1);
}
console.log("engine-stamp: every copy is the engine on disk.");
