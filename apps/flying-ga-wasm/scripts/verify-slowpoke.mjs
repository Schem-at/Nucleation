/** Verification for "slowest engine that still flies": runs a scalar-mode
 * session with speed MINIMIZED (solo objective, engine-b seeding, pop 96,
 * uncapped) against the vite preview on :8444, asserts the champion is
 * slower than engine B's 2.0 blk/s yet sustained (second-half displacement
 * ≥ 1 block and ≥ 25% of total), captures light+dark screenshots of the
 * dashboard and the Hall of Fame with the Slowpoke plinth filled, and
 * exports a GIF of the slowest flier.
 *
 * Run: node scripts/verify-slowpoke.mjs [runSeconds]
 */

import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const RUN_SECONDS = Number(process.argv[2] ?? 90);
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

// ---- configure: scalar mode, speed solo MINIMIZED, engine-b seed, pop 96 ----
await page.getByTestId("dir-speed").click(); // max ↑ -> min ↓
out.results.dirToggle = (await page.getByTestId("dir-speed").innerText()).trim();
out.results.sustainedAutoOn = await page.getByTestId("cfg-sustained").isChecked();
await page.getByTestId("cfg-seeding").selectOption("engine-b");
await page.locator("#cfg-pop").fill("96");
await page.locator("#cfg-gens").fill(""); // uncapped
await page.getByTestId("start-run").click();

await page.getByTestId("stat-status").filter({ hasText: "running" }).waitFor({
  timeout: 90_000,
});
console.log("run started (min speed, sustained), evolving for", RUN_SECONDS, "s …");

const t0 = Date.now();
while (Date.now() - t0 < RUN_SECONDS * 1000) {
  await page.waitForTimeout(5000);
  const gen = await page.getByTestId("stat-generation").innerText();
  const lbTop = await page
    .locator(".lb-table tbody tr td")
    .nth(2)
    .innerText()
    .catch(() => "—");
  console.log(
    `  t+${Math.round((Date.now() - t0) / 1000)}s gen=${gen.split("\n")[0]} lb-top-bps=${lbTop}`,
  );
}

await page.getByTestId("stop-run").click();
await page.getByTestId("stat-status").filter({ hasText: "done" }).waitFor({
  timeout: 120_000,
});
await page.waitForTimeout(1200);

// ---- measurements -----------------------------------------------------------
out.results.generation = (
  await page.getByTestId("stat-generation").innerText()
).split("\n")[0];

// Full leaderboard (ranked by score: slowest sustained flier first).
out.results.leaderboard = await page.$$eval(".lb-table tbody tr", (rows) =>
  rows.slice(0, 10).map((r) => {
    const c = [...r.querySelectorAll("td")].map((td) => td.textContent.trim());
    return { name: c[1], bps: c[2], dist: c[3], size: c[4], gen: c[5] };
  }),
);

// Authoritative metrics from the stored run record.
out.results.champion = await page.evaluate(() => {
  const idx = JSON.parse(localStorage.getItem("fgaw:runs") ?? "[]");
  if (idx.length === 0) return null;
  const rec = JSON.parse(localStorage.getItem(`fgaw:run:${idx[0].id}`) ?? "null");
  const top = rec?.leaderboard?.[0];
  if (!top) return null;
  return {
    name: top.name ?? top.id,
    score: top.score,
    speed: top.metrics?.speed,
    disp: top.metrics?.disp,
    lateDisp: top.metrics?.lateDisp,
    sustained: top.metrics?.sustained,
    flies: top.metrics?.flies,
    gen: top.gen,
    blockCount: top.blocks.length,
    blocks: top.blocks.map((b) => `${b.x},${b.y},${b.z} ${b.state}`),
  };
});

// Slowpoke events in the feed.
out.results.slowpokeEvents = await page.$$eval(".ev-slowpoke", (els) =>
  els.map((el) => el.textContent.trim()),
);

// ---- screenshots: light -----------------------------------------------------
await page.screenshot({
  path: join(SHOTS, "dashboard-slowpoke-light.png"),
  fullPage: true,
});
await page.getByTestId("tab-hof").click();
await page.getByTestId("hof-slowpoke").waitFor({ timeout: 15_000 });
out.results.hofSlowpoke = await page.$eval(
  '[data-testid="hof-slowpoke"]',
  (c) => ({
    hero: c.querySelector(".hof-hero")?.textContent,
    sub: c.querySelector(".hof-sub")?.textContent,
  }),
);
out.results.hofSlowpokeEntry = await page.evaluate(() => {
  const hof = JSON.parse(localStorage.getItem("fgaw:hof") ?? "{}");
  const e = hof.slowpoke;
  if (!e) return null;
  return {
    speed: e.metrics.speed,
    disp: e.metrics.disp,
    lateDisp: e.metrics.lateDisp,
    sustained: e.metrics.sustained,
    blockCount: e.blocks.length,
    gen: e.gen,
    blocks: e.blocks.map((b) => `${b.x},${b.y},${b.z} ${b.state}`),
  };
});
await page.waitForTimeout(500);
await page.screenshot({
  path: join(SHOTS, "hof-slowpoke-light.png"),
  fullPage: true,
});

// ---- dark theme -------------------------------------------------------------
await page.getByRole("button", { name: "Toggle color theme" }).click();
await page.waitForTimeout(400);
await page.screenshot({
  path: join(SHOTS, "hof-slowpoke-dark.png"),
  fullPage: true,
});
await page.getByRole("button", { name: "Lab" }).click();
await page.waitForTimeout(600);
await page.screenshot({
  path: join(SHOTS, "dashboard-slowpoke-dark.png"),
  fullPage: true,
});
await page.getByRole("button", { name: "Toggle color theme" }).click();
await page.waitForTimeout(300);

// ---- GIF of the slowest flier: stage the Slowpoke plinth machine ------------
await page.getByTestId("tab-hof").click();
await page.getByTestId("hof-slowpoke").click(); // stages it on the lab stage
await page.waitForTimeout(500);
try {
  await page
    .getByTestId("export-gif")
    .and(page.locator(":enabled"))
    .waitFor({ timeout: 90_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download", { timeout: 180_000 }),
    page.getByTestId("export-gif").click(),
  ]);
  const name = download.suggestedFilename();
  const dest = join(SHOTS, `slowpoke-${name}`);
  await download.saveAs(dest);
  out.results.gif = `screenshots/slowpoke-${name}`;
} catch (e) {
  out.errors.push(`slowpoke gif: ${e}`);
}
out.results.stagePeriod = await page
  .locator(".stage-note")
  .innerText()
  .catch(() => null);

console.log(JSON.stringify(out, null, 2));
await browser.close();

// ---- assertions -------------------------------------------------------------
const c = out.results.champion;
let fail = 0;
const assert = (ok, msg) => {
  console.log(`${ok ? "PASS" : "FAIL"}: ${msg}`);
  if (!ok) fail = 1;
};
assert(out.results.dirToggle.includes("min"), "speed direction toggled to min");
assert(out.results.sustainedAutoOn, "must-keep-flying auto-enabled");
assert(!!c, "champion exists");
if (c) {
  assert(c.speed > 0 && c.speed < 2.0 - 1e-6, `champion slower than 2.0 blk/s (${c.speed})`);
  assert(c.sustained === true, `champion sustained (lateDisp=${c.lateDisp}, disp=${c.disp})`);
}
assert(!!out.results.hofSlowpokeEntry, "Slowpoke plinth filled");
assert(out.results.slowpokeEvents.length > 0, "slowpoke event(s) in the feed");
process.exit(fail);
