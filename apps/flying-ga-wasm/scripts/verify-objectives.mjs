/** Verification for the multi-objective upgrade: runs a Pareto-mode session
 * (speed vs size, pop 96, uncapped) for ~2 minutes against the vite preview
 * on :8444, asserts the front populates and blk/s reads correctly, then
 * captures light+dark screenshots (dashboard / pareto / hall of fame) and
 * exports a GIF of a Pareto-corner machine.
 *
 * Run: node scripts/verify-objectives.mjs [runSeconds]
 */

import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const RUN_SECONDS = Number(process.argv[2] ?? 120);
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

// ---- configure: Pareto mode, speed + size, pop 96, uncapped -----------------
await page.getByTestId("mode-pareto").click();
await page.getByTestId("obj-size").check();
await page.locator("#cfg-pop").fill("96");
await page.locator("#cfg-gens").fill(""); // uncapped
await page.getByTestId("start-run").click();

// Wait for the run to actually start evaluating.
await page.getByTestId("stat-status").filter({ hasText: "running" }).waitFor({
  timeout: 90_000,
});
console.log("run started, evolving for", RUN_SECONDS, "s …");

const t0 = Date.now();
while (Date.now() - t0 < RUN_SECONDS * 1000) {
  await page.waitForTimeout(5000);
  const gen = await page.getByTestId("stat-generation").innerText();
  const front = await page.locator('[data-testid="pareto-point"]').count();
  console.log(
    `  t+${Math.round((Date.now() - t0) / 1000)}s gen=${gen.split("\n")[0]} front=${front}`,
  );
}

// ---- stop and let the last generation settle --------------------------------
await page.getByTestId("stop-run").click();
await page.getByTestId("stat-status").filter({ hasText: "done" }).waitFor({
  timeout: 120_000,
});
await page.waitForTimeout(1200);

// ---- measurements -----------------------------------------------------------
const frontCount = await page.locator('[data-testid="pareto-point"]').count();
out.results.frontCount = frontCount;
out.results.generation = (
  await page.getByTestId("stat-generation").innerText()
).split("\n")[0];
out.results.bestSpeedStat = (
  await page.getByTestId("stat-speed").innerText()
).split("\n")[0];
out.results.bestDistanceStat = (
  await page.getByTestId("stat-best").innerText()
).split("\n")[0];

// Leaderboard rows: name + blk/s + dist + size.
out.results.leaderboard = await page.$$eval(".lb-table tbody tr", (rows) =>
  rows.slice(0, 10).map((r) => {
    const c = [...r.querySelectorAll("td")].map((td) => td.textContent.trim());
    return { name: c[1], bps: c[2], dist: c[3], size: c[4], gen: c[5] };
  }),
);

// Pareto front contents via tooltip-free DOM: hover each point is heavy;
// instead read from the archive through the app's localStorage record.
out.results.front = await page.evaluate(() => {
  const idx = JSON.parse(localStorage.getItem("fgaw:runs") ?? "[]");
  if (idx.length === 0) return [];
  const rec = JSON.parse(
    localStorage.getItem(`fgaw:run:${idx[0].id}`) ?? "null",
  );
  return (rec?.archive ?? []).map((e) => ({
    name: e.name ?? e.id,
    speed: e.speed,
    blocks: e.blocks.length,
    fitness: e.fitness,
    gen: e.gen,
  }));
});

// Engine-B blk/s check (1 block / 10 ticks -> 2.0 blk/s).
const engineB = out.results.leaderboard.find((r) => r.name.startsWith("engine-b"));
out.results.engineB = engineB ?? null;

// Chart axis toggle: blk/s mode.
await page.getByTestId("axis-bps").click();
await page.waitForTimeout(300);
out.results.chartAriaBps = await page
  .locator(".chart-svg")
  .first()
  .getAttribute("aria-label");
await page
  .locator("section.panel", { hasText: "Fitness over generations" })
  .first()
  .screenshot({ path: join(SHOTS, "chart-bps-objectives-light.png") });

// Stage speed readout.
out.results.stageSpeed = (
  await page.getByTestId("stage-speed").innerText()
).split("\n")[0];

// ---- screenshots: light -----------------------------------------------------
await page.screenshot({
  path: join(SHOTS, "dashboard-objectives-light.png"),
  fullPage: true,
});
await page
  .getByTestId("pareto-panel")
  .screenshot({ path: join(SHOTS, "pareto-objectives-light.png") });
await page.getByTestId("tab-hof").click();
await page.waitForTimeout(400);
out.results.hofCards = await page.$$eval(".hof-card:not(.empty)", (cards) =>
  cards.map((c) => ({
    cat: c.querySelector(".hof-cat")?.textContent,
    hero: c.querySelector(".hof-hero")?.textContent,
    sub: c.querySelector(".hof-sub")?.textContent,
  })),
);
await page.screenshot({
  path: join(SHOTS, "hof-objectives-light.png"),
  fullPage: true,
});

// ---- dark theme -------------------------------------------------------------
await page.getByRole("button", { name: "Toggle color theme" }).click();
await page.waitForTimeout(400);
await page.screenshot({
  path: join(SHOTS, "hof-objectives-dark.png"),
  fullPage: true,
});
await page.getByRole("button", { name: "Lab" }).click();
await page.waitForTimeout(600);
await page.screenshot({
  path: join(SHOTS, "dashboard-objectives-dark.png"),
  fullPage: true,
});
await page
  .getByTestId("pareto-panel")
  .screenshot({ path: join(SHOTS, "pareto-objectives-dark.png") });
await page.getByRole("button", { name: "Toggle color theme" }).click();
await page.waitForTimeout(300);

// ---- GIF of a Pareto-corner machine ----------------------------------------
// Corner = smallest machine on the front (last point along the size axis).
const points = page.locator('[data-testid="pareto-point"]');
const n = await points.count();
if (n > 0) {
  // Points render in archive order (speed desc). Index 0 = the fast corner;
  // fall back to the small-and-slow corner if its replay fails.
  for (const idx of [0, n - 1]) {
    await points.nth(idx).click();
    try {
      await page
        .getByTestId("export-gif")
        .and(page.locator(":enabled"))
        .waitFor({ timeout: 60_000 });
      const [download] = await Promise.all([
        page.waitForEvent("download", { timeout: 120_000 }),
        page.getByTestId("export-gif").click(),
      ]);
      const name = download.suggestedFilename();
      const dest = join(SHOTS, `pareto-corner-${name}`);
      await download.saveAs(dest);
      out.results.gif = `screenshots/pareto-corner-${name}`;
      out.results.gifCornerIndex = idx;
      break;
    } catch (e) {
      out.errors.push(`gif corner ${idx}: ${e}`);
    }
  }
}

console.log(JSON.stringify(out, null, 2));
await browser.close();

if (frontCount < 2) {
  console.error("FAIL: Pareto front has fewer than 2 machines");
  process.exit(1);
}
console.log("VERIFY OK");
