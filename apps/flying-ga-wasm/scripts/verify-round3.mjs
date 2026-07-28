/** Round-3 verification against vite preview on :8444.
 *
 *  A. camera: drag the machine viewer, then 3+ poll cycles — camera matrix
 *     unchanged (via window.__fgaCameras debug registry); follow chip shows.
 *     Also asserts the built UI defaults to the "minimal" seed.
 *  B. speed+cargo Pareto ~60s — NO front member below 0.5 blk/s (implied
 *     must-fly gate); gated-chip screenshots; mid-run minBlocks bump → the
 *     retired shelf appears (screenshot).
 *  C. scalar speed-max, uncapped; at ~gen 50 flip speed → min: events entry,
 *     leaderboard flips, run continues. Then weighted-mode check: adding
 *     size w=8 re-ranks the board. Then the Lineage tab: Muller chart with
 *     ≥1 extinction (screenshots).
 *  D. engine-b-seeded target-period run (target 10) — champion period == 10.
 *
 * Run: node scripts/verify-round3.mjs
 */

import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });
const URL = "http://localhost:8444/";

const failures = [];
const results = {};
const ok = (name, cond, detail) => {
  results[name] = { pass: !!cond, detail };
  if (!cond) failures.push(`${name}: ${detail}`);
  console.log(`${cond ? "PASS" : "FAIL"} ${name} — ${detail}`);
};

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1480, height: 1050 } });
const pageErrors = [];
page.on("pageerror", (e) => pageErrors.push(String(e)));

const gen = async () =>
  parseInt((await page.getByTestId("stat-generation").innerText()).split("\n")[0], 10) || 0;
const waitGen = async (target, timeoutMs) => {
  const t0 = Date.now();
  while (Date.now() - t0 < timeoutMs) {
    const g = await gen();
    if (g >= target) return g;
    await page.waitForTimeout(2000);
  }
  return await gen();
};
const latestRecord = () =>
  page.evaluate(() => {
    const idx = JSON.parse(localStorage.getItem("fgaw:runs") ?? "[]");
    if (idx.length === 0) return null;
    return JSON.parse(localStorage.getItem(`fgaw:run:${idx[0].id}`) ?? "null");
  });
const stopRun = async () => {
  await page.getByTestId("stop-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "done" })
    .waitFor({ timeout: 120_000 });
  await page.waitForTimeout(600);
};
const toggleTheme = () =>
  page.getByRole("button", { name: "Toggle color theme" }).click();

// ---------------------------------------------------------------- phase A
console.log("== phase A: camera + seeding default ==");
await page.goto(URL, { waitUntil: "networkidle" });

const seedDefault = await page.getByTestId("cfg-seeding").inputValue();
ok("A.seeding-default", seedDefault === "minimal", `built UI default seed = "${seedDefault}"`);

await page.locator("#cfg-pop").fill("64");
await page.locator("#cfg-gens").fill("");
await page.getByTestId("start-run").click();
await page
  .getByTestId("stat-status")
  .filter({ hasText: "running" })
  .waitFor({ timeout: 90_000 });
// Wait for a machine in the viewer with a live GL canvas.
await page
  .locator('[data-testid="gl-blocks"][data-ready="1"]')
  .waitFor({ timeout: 120_000 });
await page.waitForTimeout(1000);

const camMatrix = () =>
  page.evaluate(() => {
    const c = window.__fgaCameras?.viewer;
    return c ? Array.from(c.matrixWorld.elements) : null;
  });

const m0 = await camMatrix();
ok("A.camera-registered", !!m0, "viewer camera exposed for introspection");

// Scripted drag on the viewer canvas.
const canvas = page.locator('[data-testid="gl-blocks"] canvas');
const bb = await canvas.boundingBox();
await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2);
await page.mouse.down();
for (let i = 1; i <= 8; i++)
  await page.mouse.move(bb.x + bb.width / 2 + i * 10, bb.y + bb.height / 2 + i * 4, { steps: 2 });
await page.mouse.up();
// Orbit damping decays geometrically (×0.9/frame); wait until two samples
// 400ms apart agree to 1e-10 — i.e. the controls are genuinely at rest —
// so the poll-stability check below only measures poll-driven movement.
{
  const t0 = Date.now();
  let prev = await camMatrix();
  while (Date.now() - t0 < 10_000) {
    await page.waitForTimeout(400);
    const cur = await camMatrix();
    const d = prev.map((v, i) => Math.abs(v - cur[i])).reduce((a, b) => Math.max(a, b), 0);
    prev = cur;
    if (d < 1e-10) break;
  }
}

const m1 = await camMatrix();
const moved = m0 && m1 && m0.some((v, i) => Math.abs(v - m1[i]) > 1e-6);
ok("A.drag-moved-camera", moved, "drag actually orbited the camera");
ok(
  "A.follow-chip",
  await page.getByTestId("viewer-follow-chip").isVisible(),
  "'following leader ⏸' chip appeared after the grab",
);

// Three poll cycles = three generations of live updates.
const gA = await gen();
const gA2 = await waitGen(gA + 3, 90_000);
const m2 = await camMatrix();
const drift = m1.map((v, i) => Math.abs(v - m2[i])).reduce((a, b) => Math.max(a, b), 0);
ok(
  "A.camera-stable-across-polls",
  gA2 >= gA + 3 && drift < 1e-8,
  `gen ${gA}→${gA2}, max matrix drift ${drift} (a poll-driven re-fit would move elements by ~1e0)`,
);
await stopRun();

// ---------------------------------------------------------------- phase B
console.log("== phase B: gated Pareto speed+cargo + retired shelf ==");
await page.goto(URL, { waitUntil: "networkidle" });
await page.getByTestId("mode-pareto").click();
await page.getByTestId("obj-cargo").check();
ok(
  "B.gate-chip",
  await page.getByTestId("gate-chip").isVisible(),
  "implied must-keep-flying lock chip visible with cargo selected",
);
await page
  .locator("section.panel", { hasText: "Run config" })
  .first()
  .screenshot({ path: join(SHOTS, "gated-chip-round3-light.png") });
await toggleTheme();
await page.waitForTimeout(300);
await page
  .locator("section.panel", { hasText: "Run config" })
  .first()
  .screenshot({ path: join(SHOTS, "gated-chip-round3-dark.png") });
await toggleTheme();
await page.waitForTimeout(300);

// Engine-b seed: the gate test needs REAL fliers on the front within ~60s
// (from the minimal seed no clean sustained flier evolves that fast — the
// gate correctly leaves the front empty, which asserts nothing).
await page.getByTestId("cfg-seeding").selectOption("engine-b");
await page.locator("#cfg-pop").fill("96");
await page.locator("#cfg-gens").fill("");
await page.getByTestId("start-run").click();
await page
  .getByTestId("stat-status")
  .filter({ hasText: "running" })
  .waitFor({ timeout: 90_000 });

let tB = Date.now();
while (Date.now() - tB < 60_000) {
  await page.waitForTimeout(5000);
  const g = await gen();
  const frontN = await page.locator('[data-testid="pareto-point"]').count();
  console.log(`  B t+${Math.round((Date.now() - tB) / 1000)}s gen=${g} front=${frontN}`);
}
// Gate assertion from the live persisted record (saved every ≤4s), BEFORE
// the retirement exercise mutates the archive.
await page.waitForTimeout(4500);
const recB = await latestRecord();
const front = (recB?.archive ?? []).map((e) => ({
  id: e.name ?? e.id,
  speed: e.metrics?.speed ?? e.speed ?? 0,
  cargo: e.metrics?.cargo,
  blocks: e.blocks.length,
}));
const slowest = front.reduce((a, e) => Math.min(a, e.speed), Infinity);
ok(
  "B.no-parked-cargo",
  front.length > 0 && front.every((e) => e.speed >= 0.5),
  `${front.length} front members, slowest ${slowest === Infinity ? "n/a" : slowest.toFixed(2)} blk/s (gate: ≥0.5)`,
);
console.log("  front:", JSON.stringify(front));

// Retirement: ban slime mid-run — every flier carries it, so the whole
// front is invalid under the new constraints and moves to the shelf.
await page
  .getByRole("group", { name: "Banned block kinds" })
  .getByRole("button", { name: "slime", exact: true })
  .click();
let shelfShot = false;
try {
  await page.getByTestId("retired-shelf").waitFor({ timeout: 30_000 });
  await page
    .getByTestId("pareto-panel")
    .screenshot({ path: join(SHOTS, "retired-shelf-round3-light.png") });
  await toggleTheme();
  await page.waitForTimeout(300);
  await page
    .getByTestId("pareto-panel")
    .screenshot({ path: join(SHOTS, "retired-shelf-round3-dark.png") });
  await toggleTheme();
  shelfShot = true;
} catch {
  /* asserted below */
}
const retiredCount = await page.locator('[data-testid="retired-entry"]').count();
ok(
  "B.retired-shelf",
  shelfShot && retiredCount > 0,
  `${retiredCount} archive entries retired (not vanished) after banning slime mid-run`,
);
await stopRun();

// ---------------------------------------------------------------- phase C
console.log("== phase C: mid-run speed-max → speed-min + weights + lineage ==");
await page.goto(URL, { waitUntil: "networkidle" });
await page.locator("#cfg-pop").fill("48");
await page.locator("#cfg-gens").fill("");
await page.getByTestId("start-run").click();
await page
  .getByTestId("stat-status")
  .filter({ hasText: "running" })
  .waitFor({ timeout: 90_000 });

const gFlipAt = await waitGen(50, 150_000);
const topSpeedBefore = parseFloat(
  await page.locator('[data-testid="lb-speed"]').first().innerText(),
);
await page.getByTestId("dir-speed").click(); // max → min (implies sustained)
console.log(`  flipped speed direction at gen ${gFlipAt}, top speed was ${topSpeedBefore}`);

// The runner applies the patch at the next generation boundary.
await page.waitForTimeout(1000);
const gAfterFlip = await waitGen(gFlipAt + 2, 90_000);
const feed = await page.getByTestId("events-feed").innerText();
ok(
  "C.config-event",
  /objectives changed @ gen \d+/.test(feed) && feed.includes("min speed"),
  "events feed logged the regime change",
);
await page.screenshot({
  path: join(SHOTS, "mid-run-switch-round3-light.png"),
  fullPage: true,
});
await toggleTheme();
await page.waitForTimeout(300);
await page.screenshot({
  path: join(SHOTS, "mid-run-switch-round3-dark.png"),
  fullPage: true,
});
await toggleTheme();

await waitGen(gAfterFlip + 2, 60_000);
const topSpeedAfter = parseFloat(
  await page.locator('[data-testid="lb-speed"]').first().innerText(),
);
ok(
  "C.leaderboard-flipped",
  topSpeedAfter < topSpeedBefore - 1e-6,
  `top blk/s ${topSpeedBefore} → ${topSpeedAfter} after min-speed re-rank`,
);
const gCont = await waitGen((await gen()) + 2, 60_000);
ok(
  "C.run-continues",
  gCont > gAfterFlip && pageErrors.length === 0,
  `still evolving (gen ${gCont}), ${pageErrors.length} page errors`,
);

// Weighted mode: weights actually change ranking (mid-run re-score).
const orderBefore = await page.$$eval(".lb-table tbody tr td.name", (tds) =>
  tds.map((t) => t.textContent.trim()),
);
await page.getByTestId("obj-size").check();
const wInput = page.getByTestId("weight-size");
await wInput.fill(""); // the old numeric state used to jam here
await wInput.fill("8");
await page.waitForTimeout(1000);
await waitGen((await gen()) + 2, 60_000);
const orderAfter = await page.$$eval(".lb-table tbody tr td.name", (tds) =>
  tds.map((t) => t.textContent.trim()),
);
ok(
  "C.weights-change-ranking",
  JSON.stringify(orderBefore) !== JSON.stringify(orderAfter),
  `board order changed after size w=8 (${orderBefore[0]} → ${orderAfter[0]} on top)`,
);

// Lineage view: Muller chart with at least one visible extinction.
await page.getByTestId("tab-lineage").click();
await page.getByTestId("muller-chart").waitFor({ timeout: 15_000 });
const extincts = await page.locator('[data-testid="mark-extinct"]').count();
const births = await page.locator('[data-testid="mark-birth"]').count();
ok(
  "C.lineage-extinction",
  extincts >= 1,
  `${births} birth marks, ${extincts} extinction marks on the Muller chart`,
);
// Open a species dossier for the screenshot when possible.
const bands = page.locator('[data-testid^="species-band-"]');
if ((await bands.count()) > 1) await bands.nth(0).click({ force: true });
await page.waitForTimeout(400);
await page.screenshot({
  path: join(SHOTS, "lineage-round3-light.png"),
  fullPage: true,
});
await toggleTheme();
await page.waitForTimeout(300);
await page.screenshot({
  path: join(SHOTS, "lineage-round3-dark.png"),
  fullPage: true,
});
await toggleTheme();
await page.getByRole("button", { name: "Lab" }).click();
await stopRun();

// ---------------------------------------------------------------- phase D
console.log("== phase D: target-period run (target 10) ==");
await page.goto(URL, { waitUntil: "networkidle" });
await page.getByTestId("obj-period").check();
await page.getByTestId("cfg-target-period").fill("10");
await page.getByTestId("cfg-seeding").selectOption("engine-b");
await page.locator("#cfg-pop").fill("64");
await page.locator("#cfg-gens").fill("");
await page.getByTestId("start-run").click();
await page
  .getByTestId("stat-status")
  .filter({ hasText: "running" })
  .waitFor({ timeout: 90_000 });
let tD = Date.now();
while (Date.now() - tD < 50_000) {
  await page.waitForTimeout(5000);
  console.log(`  D t+${Math.round((Date.now() - tD) / 1000)}s gen=${await gen()}`);
}
await stopRun();
const recD = await latestRecord();
const champ = recD?.leaderboard?.[0];
ok(
  "D.champion-period-10",
  champ?.metrics?.period === 10,
  `top machine "${champ?.name ?? champ?.id}" detected period = ${champ?.metrics?.period} ticks (err ${champ?.metrics?.periodErr})`,
);

console.log(JSON.stringify({ results, pageErrors }, null, 2));
await browser.close();
if (failures.length > 0) {
  console.error(`VERIFY FAILED (${failures.length}):\n- ` + failures.join("\n- "));
  process.exit(1);
}
console.log("VERIFY OK — all round-3 checks passed");
