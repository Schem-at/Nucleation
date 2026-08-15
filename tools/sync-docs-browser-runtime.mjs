#!/usr/bin/env node
import { createHash } from "node:crypto";
import { copyFile, mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { basename, resolve } from "node:path";

const packageRoot = process.argv[2];
if (packageRoot === undefined) {
  throw new Error("usage: node tools/sync-docs-browser-runtime.mjs /path/to/assembled/npm-package");
}

const source = resolve(packageRoot);
const target = resolve(import.meta.dirname, "../docs/javascripts/vendor/nucleation");
const entries = [
  "Brush.mjs",
  "BuildingTool.mjs",
  "Field3.mjs",
  "InterpolationSpace.mjs",
  "MeshConfig.mjs",
  "MeshResult.mjs",
  "Palette.mjs",
  "ResourcePack.mjs",
  "Schematic.mjs",
  "Sdf.mjs",
];
const modules = new Set();

async function collect(name) {
  if (modules.has(name)) return;
  modules.add(name);
  const sourceText = await readFile(resolve(source, name), "utf8");
  for (const match of sourceText.matchAll(/from\s+["'](\.\/[^"']+\.mjs)["']/g)) {
    await collect(basename(match[1]));
  }
}

for (const entry of entries) await collect(entry);

await rm(target, { recursive: true, force: true });
await mkdir(target, { recursive: true });
for (const name of [...modules].sort()) {
  await copyFile(resolve(source, name), resolve(target, name));
}
await copyFile(resolve(source, "nucleation.wasm"), resolve(target, "nucleation.wasm"));

const wasm = await readFile(resolve(target, "nucleation.wasm"));
await writeFile(
  resolve(target, "manifest.json"),
  `${JSON.stringify(
    {
      features: ["bridge", "meshing"],
      wasmSha256: createHash("sha256").update(wasm).digest("hex"),
      modules: [...modules].sort(),
    },
    null,
    2,
  )}\n`,
);

const written = await readdir(target);
process.stdout.write(
  `Synced ${String(modules.size)} browser modules and Nucleation WASM (${String(written.length)} files).\n`,
);
