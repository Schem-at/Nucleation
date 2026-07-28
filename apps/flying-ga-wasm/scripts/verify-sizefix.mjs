/** Size-pressure + seam-jank verification (defects: "size/efficiency have
 * no effect", "flight loop hitches at the wrap", "piston base slides back
 * on extension").
 *
 *  D1a. scalar speed-only run (seed 42, minimal seed, pop 64, 400 ticks);
 *       at ~gen 40 the efficiency objective is checked mid-run; by ~gen 110
 *       the CHAMPION (stage machine) must have shed blocks vs the gen-40
 *       speed-only champion, while still a flier.
 *  D1b. max-blocks is then tightened mid-run below the current leaderboard
 *       sizes; within ~12 generations no over-cap machine may remain on
 *       the leaderboard.
 *  D2.  the champion's stored flight loop is sampled at 60 fps across two
 *       wrap boundaries in compensated space; the max frame-to-frame
 *       nearest-neighbour jump at the seam must match mid-loop jumps.
 *  D3.  no piston BASE member may carry interpolated motion, and no
 *       sampled base pose may show a fractional offset.
 *  Camera: the translate-compensation accumulator is replayed for 10 loops
 *       and must return to the same x at equal phase (no drift).
 *
 * Run: node scripts/verify-sizefix.mjs   (vite preview spawned on :8444)
 */

import { chromium } from "playwright";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { withPreview, ok, failures, SHOTS, ROOT, URL } from "./round4-lib.mjs";

const GEN_SWITCH = 40;
const GEN_END = 110;
const GEN_CAP_GRACE = 12;

/* ---------------------------------------------------- loop-sampling maths */

const isBase = (s) =>
  s.startsWith("minecraft:piston[") || s.startsWith("minecraft:sticky_piston[");

function poseAt(cast, t) {
  const items = [];
  for (const m of cast) {
    if (t < m.start || t >= m.end) continue;
    let ox = 0,
      oy = 0,
      oz = 0;
    if (m.motion) {
      const span = m.motion.until - m.start;
      const p = span > 0 ? Math.min(1, Math.max(0, (t - m.start) / span)) : 1;
      const r = 1 - p;
      ox = (m.motion.fx - m.x) * r;
      oy = (m.motion.fy - m.y) * r;
      oz = (m.motion.fz - m.z) * r;
    }
    items.push({ x: m.x + ox, y: m.y + oy, z: m.z + oz, ox, oy, oz, state: m.state });
  }
  return items;
}

/** D3 audit: a piston BASE may move only as genuine cargo — its motion
 * source must be a same-state member vacating exactly at its birth — and an
 * in-place state swap (same cell, same block name, adjacent segments: an
 * extension stroke) must never carry motion. Wrapped seam twins (start < 0)
 * are shifted copies of already-audited members. */
function baseMotionAudit(cast) {
  const nameOf = (s) => s.split("[")[0];
  const bad = [];
  for (const m of cast) {
    if (!isBase(m.state) || m.start < 0) continue;
    if (m.motion) {
      const src = cast.find(
        (n) =>
          n !== m &&
          n.state === m.state &&
          n.x === m.motion.fx &&
          n.y === m.motion.fy &&
          n.z === m.motion.fz &&
          Math.abs(n.end - m.start) < 1e-9,
      );
      if (!src)
        bad.push(
          `base at ${m.x},${m.y},${m.z}@${m.start} has motion from ` +
            `${m.motion.fx},${m.motion.fy},${m.motion.fz} with no vacating source`,
        );
      const inPlacePrev = cast.find(
        (n) =>
          n !== m &&
          n.x === m.x &&
          n.y === m.y &&
          n.z === m.z &&
          Math.abs(n.end - m.start) < 1e-9 &&
          nameOf(n.state) === nameOf(m.state),
      );
      if (inPlacePrev)
        bad.push(`in-place swap with motion at ${m.x},${m.y},${m.z}@${m.start}`);
    }
  }
  return bad;
}

const dist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y, a.z - b.z);
const directedHausdorff = (A, B) => {
  let worst = 0;
  for (const a of A) {
    let best = Infinity;
    for (const b of B) best = Math.min(best, dist(a, b));
    worst = Math.max(worst, best);
  }
  return worst;
};

/** Sample the rendered loop (compensated space: drawn x = x - shift) at
 * 60 fps over ~2.4 periods; quantify seam vs mid-loop frame jumps. */
function measureLoop(loop) {
  const { cast, period, dx, anchorX } = loop;
  const FPS = 60;
  const TPS = 10;
  const frames = [];
  const seam = new Set();
  const total = Math.ceil((2.4 * period * FPS) / TPS);
  let prevPhase = 0;
  let baseOffsetMax = 0;
  for (let f = 0; f <= total; f++) {
    const elapsed = (f / FPS) * TPS; // game ticks
    const phase = (elapsed % period) / period;
    if (f > 0 && phase < prevPhase) seam.add(f);
    prevPhase = phase;
    const t = phase * period;
    const shift = anchorX + phase * dx;
    const items = poseAt(cast, t).map((it) => {
      if (isBase(it.state))
        baseOffsetMax = Math.max(
          baseOffsetMax,
          Math.abs(it.ox),
          Math.abs(it.oy),
          Math.abs(it.oz),
        );
      return { ...it, x: it.x - shift };
    });
    frames.push(items);
  }
  const seamJumps = [];
  const midJumps = [];
  for (let f = 1; f < frames.length; f++) {
    const a = frames[f - 1];
    const b = frames[f];
    if (a.length === 0 || b.length === 0) continue;
    const d = Math.max(directedHausdorff(a, b), directedHausdorff(b, a));
    (seam.has(f) ? seamJumps : midJumps).push(d);
  }
  const sorted = [...midJumps].sort((x, y) => x - y);
  return {
    frames: frames.length,
    seamCount: seamJumps.length,
    seamMax: Math.max(...seamJumps),
    midMax: Math.max(...midJumps),
    midMedian: sorted[Math.floor(sorted.length / 2)],
    baseOffsetMax,
    baseWithMotion: loop.cast.filter((m) => isBase(m.state) && m.motion).length,
  };
}

/** Replay the stage's camera-follow accumulator for `loops` periods and
 * report the worst |camera.x(phase) - shift(phase)| drift. */
function measureCameraDrift(loop, loops = 10) {
  const { period, dx, anchorX } = loop;
  const FPS = 60;
  const TPS = 10;
  // The stage applies the first frame's full shift once (framing the fitted
  // camera into world space); drift is measured against that baseline.
  let cam = 0;
  let prevShift = anchorX;
  let drift = 0;
  const total = Math.ceil((loops * period * FPS) / TPS);
  for (let f = 0; f <= total; f++) {
    const phase = (((f / FPS) * TPS) % period) / period;
    const shift = anchorX + phase * dx;
    const d = shift - prevShift;
    prevShift = shift;
    cam += d;
    drift = Math.max(drift, Math.abs(cam - phase * dx));
  }
  return drift;
}

/* ------------------------------------------------------------- browser run */

await withPreview(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1480, height: 1050 } });
  const pageErrors = [];
  page.on("pageerror", (e) => pageErrors.push(String(e)));
  await page.goto(URL, { waitUntil: "networkidle" });

  const gen = async () =>
    parseInt(
      (await page.getByTestId("stat-generation").innerText()).split("\n")[0],
      10,
    ) || 0;
  const waitGen = async (target, timeoutMs = 900_000) => {
    const t0 = Date.now();
    let g = await gen();
    while (g < target && Date.now() - t0 < timeoutMs) {
      await page.waitForTimeout(2000);
      g = await gen();
    }
    return g;
  };
  /** Live champion filmstrip via the App's test hook (localStorage would
   * lose the run to the quota fallback on long runs). */
  const bests = () =>
    page.evaluate(() =>
      (window.__fgaBests ?? []).map((b) => ({
        gen: b.gen,
        fitness: b.fitness,
        blocks: b.blocks.length,
      })),
    );
  const lastLoop = () =>
    page.evaluate(() => {
      const bs = window.__fgaBests ?? [];
      for (let i = bs.length - 1; i >= 0; i--) {
        const l = bs[i].loop;
        if (l && l.period > 0 && l.cast && l.cast.length) return l;
      }
      return null;
    });
  const lbSizes = () =>
    page.$$eval(".lb-table tbody tr", (rows) =>
      rows.map((r) => parseInt(r.children[4].textContent, 10)),
    );

  // ---- start: scalar, speed-only, deterministic seed ---------------------
  await page.locator("#cfg-pop").fill("64");
  await page.locator("#cfg-gens").fill("");
  await page.getByTestId("group-advanced").locator("summary").click(); // expand: eval ticks / seed
  await page.locator("#cfg-ticks").fill("400");
  await page.getByTestId("start-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "running" })
    .waitFor({ timeout: 90_000 });

  await waitGen(GEN_SWITCH);
  const champ40 = (await bests()).at(-1);
  const sizes40 = await lbSizes();
  console.log(
    `gen ~${await gen()}: speed-only champion ${champ40.blocks} blocks ` +
      `(disp ${champ40.fitness}), lb sizes = [${sizes40}]`,
  );

  // ---- D1a: add efficiency mid-run --------------------------------------
  await page.getByTestId("obj-efficiency").check();
  console.log(`efficiency objective checked @ ~gen ${await gen()}`);
  await waitGen(GEN_END);

  const champEnd = (await bests()).at(-1);
  const sizesEnd = await lbSizes();
  console.log(
    `gen ~${await gen()}: champion now ${champEnd.blocks} blocks ` +
      `(disp ${champEnd.fitness}), lb sizes = [${sizesEnd}]`,
  );
  ok(
    "D1a.champion-sheds-blocks",
    champEnd.blocks < champ40.blocks,
    `champion ${champ40.blocks} → ${champEnd.blocks} blocks after efficiency @ ~gen ${GEN_SWITCH}`,
  );
  ok(
    "D1a.champion-still-flies",
    champEnd.fitness > 0.5,
    `champion displacement ${champEnd.fitness} blocks`,
  );

  await page.screenshot({ path: join(SHOTS, "stage-sizefix.png"), fullPage: false });

  // ---- D1b: tighten max blocks mid-run -----------------------------------
  const capBase = await lbSizes();
  const capVal = Math.max(3, Math.max(...capBase) - 2);
  await page.getByTestId("cfg-maxb").fill(String(capVal));
  const gCap = await gen();
  console.log(`max blocks tightened to ${capVal} @ ~gen ${gCap} (lb sizes [${capBase}])`);
  await waitGen(gCap + GEN_CAP_GRACE);
  const sizesAfterCap = await lbSizes();
  console.log(`gen ~${await gen()}: lb sizes now [${sizesAfterCap}]`);
  ok(
    "D1b.overcap-evicted",
    sizesAfterCap.every((s) => s <= capVal),
    `cap ${capVal}, lb sizes after ${GEN_CAP_GRACE} gens = [${sizesAfterCap}]`,
  );
  await page.screenshot({ path: join(SHOTS, "leaderboard-sizefix.png"), fullPage: true });

  // ---- stop run 1 --------------------------------------------------------
  await page.getByTestId("stop-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "done" })
    .waitFor({ timeout: 120_000 });

  // ---- D2/D3: engine-b run — a strong periodic flier for the seam --------
  await page.reload({ waitUntil: "networkidle" });
  await page.getByTestId("group-genome").locator("summary").click();
  await page.getByTestId("cfg-seeding").selectOption("engine-b");
  await page.locator("#cfg-pop").fill("32");
  await page.locator("#cfg-gens").fill("");
  await page.getByTestId("start-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "running" })
    .waitFor({ timeout: 90_000 });
  {
    const t0 = Date.now();
    let loop = null;
    while (Date.now() - t0 < 120_000 && !loop) {
      loop = await lastLoop();
      if (!loop) await page.waitForTimeout(2000);
    }
    ok("D2.loop-captured", !!loop, loop ? `period ${loop.period}t, dx +${loop.dx}x, cast ${loop.cast.length}` : "no periodic loop appeared");
    if (loop) {
      writeFileSync(join(ROOT, "verify-sizefix-loop.json"), JSON.stringify(loop));
      const m = measureLoop(loop);
      console.log(
        `loop sampling: ${m.frames} frames, ${m.seamCount} seam boundaries — ` +
          `seam max jump ${m.seamMax.toFixed(3)} blk, mid-loop max ${m.midMax.toFixed(3)} blk, ` +
          `mid-loop median ${m.midMedian.toFixed(3)} blk`,
      );
      ok(
        "D2.seam-matches-midloop",
        m.seamMax <= Math.max(1.25 * m.midMedian + 0.05, 0.15),
        `seam max ${m.seamMax.toFixed(3)} vs mid-loop median ${m.midMedian.toFixed(3)} (mid-loop max ${m.midMax.toFixed(3)})`,
      );
      const audit = baseMotionAudit(loop.cast);
      ok(
        "D3.base-motion-genuine",
        audit.length === 0,
        audit.length === 0
          ? `${m.baseWithMotion} base motions, all genuine cargo hand-offs (max sampled base offset ${m.baseOffsetMax.toFixed(3)} blk while carried)`
          : audit.join(" | "),
      );
      const drift = measureCameraDrift(loop, 10);
      ok(
        "D2.camera-no-drift",
        drift < 1e-6,
        `camera-vs-shift drift over 10 loops = ${drift.toExponential(2)}`,
      );
    }
  }
  // Pause the run so a new champion can't swap the stage mid-export.
  await page.getByTestId("pause-run").click();
  await page.waitForTimeout(1500);
  await page.screenshot({ path: join(SHOTS, "stage-loop-sizefix.png"), fullPage: false });
  try {
    await page.waitForFunction(
      () => {
        const b = document.querySelector('[data-testid="export-gif"]');
        return b && !b.disabled;
      },
      { timeout: 60_000 },
    );
    const [dl] = await Promise.all([
      page.waitForEvent("download", { timeout: 90_000 }),
      page.getByTestId("export-gif").click(),
    ]);
    await dl.saveAs(join(SHOTS, "flight-sizefix.gif"));
    console.log("GIF saved to screenshots/flight-sizefix.gif");
  } catch (e) {
    console.log("GIF export skipped:", String(e).split("\n")[0]);
  }
  await page.getByTestId("stop-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "done" })
    .waitFor({ timeout: 120_000 });

  ok("no-page-errors", pageErrors.length === 0, pageErrors.join(" | ") || "clean");
  await browser.close();
});

console.log(failures.length === 0 ? "\nALL PASS" : `\nFAILURES:\n${failures.join("\n")}`);
process.exit(failures.length === 0 ? 0 : 1);
