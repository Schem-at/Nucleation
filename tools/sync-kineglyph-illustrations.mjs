#!/usr/bin/env node
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const repository = process.argv[2];
if (repository === undefined) {
  throw new Error("usage: node tools/sync-kineglyph-illustrations.mjs /path/to/kineglyph");
}

const root = resolve(import.meta.dirname, "..");
const source = resolve(repository);
const media = resolve(root, "docs/media/kineglyph");
const vendor = resolve(root, "docs/javascripts/vendor");

await Promise.all([mkdir(media, { recursive: true }), mkdir(vendor, { recursive: true })]);

const rendered = spawnSync(
  "npm",
  ["run", "render:nucleation-docs", "--", media, "960"],
  { cwd: source, stdio: "inherit" },
);
if (rendered.status !== 0) {
  throw new Error(`Kineglyph illustration export failed with status ${String(rendered.status)}`);
}

const bundleSource = resolve(source, "packages/web/dist/kineglyph-web.js");
const bundleTarget = resolve(vendor, "kineglyph-web.js");
await copyFile(bundleSource, bundleTarget);

const manifestPath = resolve(media, "manifest.json");
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
const bundle = await readFile(bundleTarget);
const commit = execFileSync("git", ["rev-parse", "HEAD"], { cwd: source, encoding: "utf8" }).trim();

await writeFile(
  manifestPath,
  `${JSON.stringify(
    {
      ...manifest,
      kineglyphCommit: commit,
      bundle: {
        path: "javascripts/vendor/kineglyph-web.js",
        sha256: createHash("sha256").update(bundle).digest("hex"),
      },
    },
    null,
    2,
  )}\n`,
  "utf8",
);

process.stdout.write(`Synced Kineglyph ${commit.slice(0, 7)} into the Nucleation docs.\n`);
