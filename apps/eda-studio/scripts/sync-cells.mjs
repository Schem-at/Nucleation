/** Copy the enhanced community cells into public/cells/ with a manifest.
 *
 *  These `.schem` files each carry an embedded `CellContract`
 *  (computational_schematics/enhanced/REPORT.md), which is what makes them
 *  usable as library cells: placing one immediately exposes typed ports. They
 *  are copies, so public/cells/ is gitignored like public/engine/.
 */
import { mkdirSync, readdirSync, copyFileSync, writeFileSync, rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const app = path.join(here, "..");
const src = path.join(app, "..", "..", "computational_schematics", "enhanced");
const out = path.join(app, "public", "cells");

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

let names = [];
try {
  names = readdirSync(src).filter((f) => f.endsWith(".schem"));
} catch (err) {
  console.warn(`sync-cells: ${src} unreadable (${err.message}); library will be empty`);
}
// Smallest first: the library panel reads better with the simple cells on top,
// and the multi-megabyte ones (REGISTERFILE, MULTIPY) load last.
const { statSync } = await import("node:fs");
names.sort((a, b) => statSync(path.join(src, a)).size - statSync(path.join(src, b)).size);
for (const f of names) copyFileSync(path.join(src, f), path.join(out, f));
writeFileSync(path.join(out, "manifest.json"), JSON.stringify(names, null, 2));
console.log(`sync-cells: ${names.length} cells -> public/cells/`);
