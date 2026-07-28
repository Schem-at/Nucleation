/** MAP-Elites verification: a minimal-seed quality-diversity run, watched
 * for ~2.5 minutes.
 *
 * Asserts: fill rate grows, QD-score is monotone non-decreasing, the archive
 * lights at least 15 distinct cells, and clicking a cell stages that exact
 * machine. Shoots the illuminated grid + the QD curve in both themes. */

import { chromium } from "playwright";
import { join } from "node:path";
import { writeFileSync } from "node:fs";
import { failures, helpers, ok, SHOTS, URL, withPreview } from "./round4-lib.mjs";

const RUN_MS = Number(process.env.RUN_MS ?? 150_000);
const SAMPLE_MS = 6_000;

await withPreview(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1560, height: 1200 } });
  page.on("pageerror", (e) => failures.push(`pageerror: ${e.message}`));
  await page.goto(URL, { waitUntil: "networkidle" });
  const h = helpers(page);
  // The config rail is a drawer over the main column: it must be open to
  // touch a control and shut to click anything on the grid.
  const drawerOpen = () =>
    page.locator("aside.side-col").evaluate((el) => el.classList.contains("open"));
  const openDrawer = async () => {
    if (!(await drawerOpen())) await page.getByTestId("config-drawer-toggle").click();
    await page.waitForTimeout(250);
  };
  const closeDrawer = async () => {
    if (await drawerOpen()) await page.keyboard.press("Escape");
    await page.waitForTimeout(350);
  };

  // Config: minimal seed (the default) + MAP-Elites archive mode.
  await h.openGroup("genome");
  await page.getByTestId("cfg-seeding").selectOption("minimal");
  await h.openGroup("objectives");
  await page.getByTestId("mode-map-elites").click();
  await page.waitForTimeout(150);
  ok(
    "map-elites config panel",
    (await page.getByTestId("me-config").count()) === 1,
    "behaviour-space controls appear when the mode is selected",
  );
  const dims = {
    x: await page.getByTestId("cfg-be-x").inputValue(),
    y: await page.getByTestId("cfg-be-y").inputValue(),
    binsX: await page.getByTestId("cfg-bins-x").inputValue(),
    binsY: await page.getByTestId("cfg-bins-y").inputValue(),
    quality: await page.getByTestId("cfg-quality").inputValue(),
  };
  ok(
    "defaults",
    dims.x === "speed" &&
      dims.y === "size" &&
      dims.binsX === "20" &&
      dims.binsY === "12" &&
      dims.quality === "displacement",
    `speed x size, ${dims.binsX}x${dims.binsY}, quality=${dims.quality}`,
  );

  await page.getByTestId("start-run").click();
  await page.waitForTimeout(2500);
  await closeDrawer();

  // ---- watch the archive fill -------------------------------------------
  const samples = [];
  const t0 = Date.now();
  let shotDone = false;
  while (Date.now() - t0 < RUN_MS) {
    await page.waitForTimeout(SAMPLE_MS);
    const s = await page.evaluate(() => {
      const q = window.__fgaQd ?? [];
      const last = q[q.length - 1] ?? null;
      return {
        gen: last?.gen ?? 0,
        qd: last?.qd ?? 0,
        fill: last?.fill ?? 0,
        filled: last?.filled ?? 0,
        cells: document.querySelectorAll('[data-testid="me-cell"]').length,
      };
    });
    samples.push({ t: Math.round((Date.now() - t0) / 1000), ...s });
    console.log(
      `  t+${samples[samples.length - 1].t}s gen ${s.gen} · ${s.filled} cells (${(s.fill * 100).toFixed(1)} %) · QD ${s.qd.toFixed(1)}`,
    );
    // Mid-run screenshots once the grid is actually illuminated (or at the
    // tail of the window, whatever the run managed).
    if (
      !shotDone &&
      (s.filled >= 25 || Date.now() - t0 > RUN_MS - SAMPLE_MS * 2)
    ) {
      shotDone = true;
      await shoot(page, "light");
      await h.toggleTheme();
      await page.waitForTimeout(400);
      await shoot(page, "dark");
      await h.toggleTheme();
      await page.waitForTimeout(400);
    }
  }

  // Snapshot the trajectory BEFORE any reconfiguration: a mid-run grid
  // rebuild is allowed to move the QD-score, so monotonicity is asserted on
  // the untouched stretch.
  const trace = await page.evaluate(() => window.__fgaQd ?? []);

  // ---- assertions --------------------------------------------------------
  const first = trace[0] ?? { fill: 0, qd: 0, filled: 0 };
  const last = trace[trace.length - 1] ?? first;

  ok(
    "fill rate grows",
    last.fill > first.fill && last.fill > 0,
    `${(first.fill * 100).toFixed(1)} % @ gen ${first.gen ?? 0} -> ${(last.fill * 100).toFixed(1)} % @ gen ${last.gen ?? 0}`,
  );

  let qdDrops = 0;
  let worstDrop = 0;
  for (let i = 1; i < trace.length; i++) {
    const d = trace[i].qd - trace[i - 1].qd;
    if (d < -1e-6) {
      qdDrops++;
      worstDrop = Math.min(worstDrop, d);
    }
  }
  ok(
    "QD-score monotone non-decreasing",
    qdDrops === 0,
    `${trace.length} generations, ${qdDrops} decreases (worst ${worstDrop.toFixed(4)}), final QD ${last.qd.toFixed(2)}`,
  );

  // Pause first: cells go on lighting up between two separate locator
  // counts on a live run, which would make lit+empty land off 240.
  await openDrawer();
  await page.getByTestId("pause-run").click();
  await page.getByTestId("stat-status").filter({ hasText: "paused" }).waitFor({
    timeout: 30_000,
  });
  await closeDrawer();
  const count = () =>
    page.evaluate(() => ({
      lit: document.querySelectorAll('[data-testid="me-cell"]').length,
      empty: document.querySelectorAll('[data-testid="me-cell-empty"]').length,
    }));
  const { lit: cellCount, empty: emptyCount } = await count();
  ok(
    "at least 15 distinct filled cells",
    cellCount >= 15,
    `${cellCount} lit cells on a 20x12 grid (archive reported ${last.filled} at the last sample)`,
  );
  ok(
    "empty cells rendered distinctly",
    emptyCount > 0 && emptyCount + cellCount === 240,
    `${cellCount} lit + ${emptyCount} empty = ${cellCount + emptyCount} of a 20x12 grid`,
  );

  // ---- click-to-stage ----------------------------------------------------
  const cells = page.locator('[data-testid="me-cell"]');
  const target = cells.nth(Math.floor((await cells.count()) / 2));
  const mid = await target.getAttribute("data-mid");
  await target.click();
  await page.waitForTimeout(900);
  const staged = await page.getByTestId("viewer-machine").innerText();
  ok(
    "clicking a cell stages that machine",
    staged.trim() === mid,
    `clicked cell holding ${mid}; viewer shows "${staged.trim()}"`,
  );
  const stageBlocks = await page.evaluate(
    () => document.querySelectorAll('[data-testid="flight-stage"] canvas').length,
  );
  ok(
    "flight stage rendering",
    stageBlocks > 0,
    `${stageBlocks} canvas on the flight stage after the pick`,
  );

  // ---- mid-run reconfigure: re-bin, don't discard ------------------------
  // Still paused: change a behaviour dimension, resume for one boundary so
  // the patch lands, pause again, count. Nothing else can move the archive.
  const beforeFilled = cellCount;
  await openDrawer();
  await h.openGroup("objectives");
  await page.getByTestId("cfg-be-y").selectOption("compactness");
  await page.waitForTimeout(700);
  await page.getByTestId("resume-run").click();
  await page.waitForTimeout(2500);
  await page.getByTestId("pause-run").click();
  await page.getByTestId("stat-status").filter({ hasText: "paused" }).waitFor({
    timeout: 30_000,
  });
  const feed = await h.eventsText();
  await closeDrawer();
  const { lit: afterFilled, empty: afterEmpty } = await count();
  ok(
    "grid rebuild re-bins instead of discarding",
    afterFilled > 0 && afterEmpty + afterFilled === 240,
    `speed x size -> speed x compactness: ${beforeFilled} lit cells before, ${afterFilled} after (grid still ${afterFilled + afterEmpty} cells)`,
  );
  ok(
    "rebuild is narrated",
    /behaviour grid rebuilt/.test(feed),
    feed.split("\n").find((l) => /behaviour grid rebuilt/.test(l)) ??
      "no rebuild line in the events feed",
  );

  await openDrawer();
  await page.getByTestId("stop-run").click();
  await page.waitForTimeout(1500);

  writeFileSync(
    join(SHOTS, "..", "verify-mapelites-out.json"),
    JSON.stringify(
      { samples, trace, cellCount, emptyCount, afterFilled, mid, staged },
      null,
      2,
    ),
  );

  await browser.close();
});

async function shoot(page, theme) {
  const grid = page.getByTestId("mapelites-grid");
  await grid.scrollIntoViewIfNeeded();
  await page.waitForTimeout(250);
  await grid.screenshot({ path: join(SHOTS, `archive-grid-${theme}-mapelites.png`) });
  const qd = page.getByTestId("qd-chart");
  await qd.scrollIntoViewIfNeeded();
  await page.waitForTimeout(250);
  await qd.screenshot({ path: join(SHOTS, `qd-curve-${theme}-mapelites.png`) });
  await page
    .getByTestId("mapelites-panel")
    .screenshot({ path: join(SHOTS, `archive-panel-${theme}-mapelites.png`) });
}
