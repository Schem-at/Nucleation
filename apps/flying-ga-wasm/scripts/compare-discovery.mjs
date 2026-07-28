/** Discovery-reliability quick-check, the GECCO'23 claim in miniature:
 * does keeping stepping stones alive find fliers more RELIABLY than fitness
 * pressure? Five seeds per mode, 60 s each, minimal seeding, identical
 * genome space. A seed "succeeds" when it produces a machine above
 * TARGET_SPEED blk/s inside the window.
 *
 * This is a quick-check, not a benchmark: n=5 and a 60 s wall-clock window
 * are far too small for a significance claim. Report the counts as counts. */

import { chromium } from "playwright";
import { join } from "node:path";
import { writeFileSync } from "node:fs";
import { SHOTS, URL, withPreview } from "./round4-lib.mjs";

const SEEDS = (process.env.SEEDS ?? "1,2,3,4,5").split(",").map(Number);
const WINDOW_MS = Number(process.env.WINDOW_MS ?? 60_000);
const TARGET_SPEED = 1.5;

const statSpeed = async (page) => {
  const t = await page.getByTestId("stat-speed").innerText();
  const v = parseFloat(t.split("\n")[0]);
  return Number.isFinite(v) ? v : 0;
};

async function runOne(page, mode, seed) {
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.evaluate(() => localStorage.clear());
  await page.reload({ waitUntil: "networkidle" });

  const openGroup = async (id) => {
    const d = page.getByTestId(`group-${id}`);
    if (!(await d.evaluate((el) => el.open))) await d.locator("summary").click();
    await page.waitForTimeout(120);
  };

  await openGroup("genome");
  await page.getByTestId("cfg-seeding").selectOption("minimal");
  await openGroup("advanced");
  await page.locator("#cfg-seed").fill(String(seed));
  await openGroup("objectives");
  if (mode === "map-elites") {
    await page.getByTestId("mode-map-elites").click();
  } else {
    await page.getByTestId("mode-scalar").click();
    // Scalar speed: the single-objective control arm.
    for (const k of ["size", "efficiency", "compactness", "cargo", "robustness", "period"]) {
      const cb = page.getByTestId(`obj-${k}`);
      if (await cb.isChecked()) await cb.click();
    }
    const sp = page.getByTestId("obj-speed");
    if (!(await sp.isChecked())) await sp.click();
  }
  await page.waitForTimeout(200);
  await page.getByTestId("start-run").click();

  const t0 = Date.now();
  let best = 0;
  let hitAt = null;
  while (Date.now() - t0 < WINDOW_MS) {
    await page.waitForTimeout(3000);
    const s = await statSpeed(page);
    if (s > best) best = s;
    if (hitAt === null && best >= TARGET_SPEED)
      hitAt = Math.round((Date.now() - t0) / 1000);
  }
  const gen = parseInt(
    (await page.getByTestId("stat-generation").innerText()).split("\n")[0],
    10,
  );
  const extra =
    mode === "map-elites"
      ? await page.evaluate(() => {
          const q = window.__fgaQd ?? [];
          const l = q[q.length - 1];
          return l ? { filled: l.filled, qd: l.qd, fill: l.fill } : null;
        })
      : null;
  await page.getByTestId("stop-run").click();
  await page.waitForTimeout(800);
  return { mode, seed, best: Math.round(best * 1000) / 1000, hitAt, gen, extra };
}

await withPreview(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1400, height: 1000 } });
  const results = [];
  for (const mode of ["map-elites", "scalar"]) {
    for (const seed of SEEDS) {
      const r = await runOne(page, mode, seed);
      results.push(r);
      console.log(
        `  ${mode.padEnd(10)} seed ${seed}: best ${r.best.toFixed(2)} blk/s @ gen ${r.gen}` +
          (r.hitAt !== null ? ` (crossed ${TARGET_SPEED} at t+${r.hitAt}s)` : " — no flier"),
      );
    }
  }
  await browser.close();

  const tally = (m) => results.filter((r) => r.mode === m && r.best >= TARGET_SPEED).length;
  const summary = {
    targetSpeed: TARGET_SPEED,
    windowMs: WINDOW_MS,
    seeds: SEEDS,
    mapElitesHits: tally("map-elites"),
    scalarHits: tally("scalar"),
    results,
  };
  console.log(
    `\nSeeds finding a >=${TARGET_SPEED} blk/s flier in ${WINDOW_MS / 1000}s:` +
      `\n  map-elites  ${summary.mapElitesHits}/${SEEDS.length}` +
      `\n  scalar speed ${summary.scalarHits}/${SEEDS.length}`,
  );
  writeFileSync(
    join(SHOTS, "..", "compare-discovery-out.json"),
    JSON.stringify(summary, null, 2),
  );
});
