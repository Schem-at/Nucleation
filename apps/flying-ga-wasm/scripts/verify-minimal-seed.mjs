/** Verification for minimal seeding + mutation locality: runs the DEFAULT
 * config (minimal seed — one sticky piston + one slime block — speed
 * maximized, pop 96, uncapped) for ~60 s, asserts evolution bootstraps
 * fliers from the 2-block seed and that champion complexity GROWS over
 * generations, then captures the dashboard.
 *
 * Run: node scripts/verify-minimal-seed.mjs [runSeconds]
 */

import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const RUN_SECONDS = Number(process.argv[2] ?? 60);
const URL = "http://localhost:8444/";

const out = { console: [], errors: [], results: {} };
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1480, height: 1050 } });
page.on("console", (m) => {
  if (m.type() === "error" || m.type() === "warning")
    out.console.push(`${m.type()}: ${m.text()}`);
});
page.on("pageerror", (e) => out.errors.push(String(e)));

await page.goto(URL, { waitUntil: "networkidle" });

// Defaults already: minimal seeding, speed max. Just size the run.
out.results.seedingDefault = await page
  .getByTestId("cfg-seeding")
  .inputValue();
await page.locator("#cfg-pop").fill("96");
await page.locator("#cfg-gens").fill("");
await page.getByTestId("start-run").click();

await page.getByTestId("stat-status").filter({ hasText: "running" }).waitFor({
  timeout: 90_000,
});
console.log("minimal-seed run started, evolving for", RUN_SECONDS, "s …");
const t0 = Date.now();
while (Date.now() - t0 < RUN_SECONDS * 1000) {
  await page.waitForTimeout(5000);
  const gen = await page.getByTestId("stat-generation").innerText();
  const best = await page.getByTestId("stat-best").innerText();
  console.log(
    `  t+${Math.round((Date.now() - t0) / 1000)}s gen=${gen.split("\n")[0]} best=${best.split("\n")[0]}`,
  );
}
await page.getByTestId("stop-run").click();
await page.getByTestId("stat-status").filter({ hasText: "done" }).waitFor({
  timeout: 120_000,
});
await page.waitForTimeout(1200);

// Champion complexity per generation (the filmstrip record).
out.results.champions = await page.evaluate(() => {
  const idx = JSON.parse(localStorage.getItem("fgaw:runs") ?? "[]");
  if (idx.length === 0) return [];
  const rec = JSON.parse(localStorage.getItem(`fgaw:run:${idx[0].id}`) ?? "null");
  return (rec?.bests ?? []).map((b) => ({
    gen: b.gen,
    fitness: b.fitness,
    blocks: b.blocks.length,
  }));
});
out.results.seeding = await page.evaluate(() => {
  const idx = JSON.parse(localStorage.getItem("fgaw:runs") ?? "[]");
  const rec = JSON.parse(localStorage.getItem(`fgaw:run:${idx[0]?.id}`) ?? "null");
  return rec?.config?.seeding ?? null;
});

await page.screenshot({
  path: join(SHOTS, "dashboard-minimal-seed-light.png"),
  fullPage: true,
});

console.log(JSON.stringify(out, null, 2));
await browser.close();

let fail = 0;
const assert = (ok, msg) => {
  console.log(`${ok ? "PASS" : "FAIL"}: ${msg}`);
  if (!ok) fail = 1;
};
assert(out.results.seedingDefault === "minimal", "minimal seeding is the default");
assert(out.results.seeding === "minimal", "run recorded minimal seeding");
const ch = out.results.champions;
assert(ch.length > 0, `evolution found ${ch.length} champion(s) from the 2-block seed`);
if (ch.length > 0) {
  const first = ch[0];
  const last = ch[ch.length - 1];
  assert(
    last.blocks >= first.blocks && last.fitness > first.fitness,
    `complexity/fitness grew: gen ${first.gen} (${first.blocks} blk, ${first.fitness}) -> gen ${last.gen} (${last.blocks} blk, ${last.fitness})`,
  );
}
process.exit(fail);
