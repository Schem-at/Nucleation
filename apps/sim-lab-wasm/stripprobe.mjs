/** Does the activity strip draw what the brief claims it draws?
 *
 * Three things must hold:
 *
 *  1. **One column per active tick, not per tick in the span.** At a strip
 *     width wide enough that bucketing does not engage, the number of
 *     `.timeline-col` elements must equal `timelineActivityJson().ticks.length`
 *     exactly — never the tick span, which includes every idle tick between
 *     active ones.
 *  2. **A still build adds no columns.** This is the brief's central claim:
 *     letting the sim run with nothing happening must not advance the strip.
 *     Proven on a build that genuinely goes idle — a flying machine like BB
 *     never reaches `isQuiescent()` (it is a *perpetual* motion machine, by
 *     design), so this uses a door instead: kick the lever, let it settle,
 *     then step 80 more ticks and check the column count does not move.
 *  3. **Bucketing conserves everything.** Once there are more active ticks
 *     than the strip has pixels for, adjacent entries merge into one column
 *     — but each column's first/last tick must still be a real recorded
 *     tick, and the summed counts across all columns must equal the
 *     unbucketed total (nothing dropped, nothing double-counted).
 *
 * Same harness shape as `verify-handover.mjs` / `uiprobe.mjs`: real page,
 * real engine, `window.simlab` for the console-equivalent calls a human
 * would otherwise make by hand.
 */
import { chromium } from "playwright";

const busyFixture = process.argv[2];
// A build that actually goes quiet, for the idle assertion — BB never does
// (see the comment above). Defaults to the door corpus README.md already
// points at.
const settleFixture =
  process.argv[3] ??
  "../../crates/mc-tick/tests/corpus/litematics/6x6_sliding_door.litematic";
if (!busyFixture) {
  console.error("usage: node stripprobe.mjs <busy schematic, e.g. BB> [settling schematic]");
  process.exit(1);
}

const b = await chromium.launch();
const errs = [];
let failures = 0;
const check = (name, cond, detail = "") => {
  console.log(`${cond ? "✅" : "❌"} ${name}${detail ? " — " + detail : ""}`);
  if (!cond) failures++;
};

async function openPage(file) {
  const p = await b.newPage();
  p.on("pageerror", (e) => errs.push(String(e).slice(0, 200)));
  await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
  await p.setInputFiles("input[type=file]", file);
  await p.waitForSelector(".status.ready", { timeout: 300000 });
  return p;
}

// ============================================================================
// Part A (BB): unbucketed column count, and bucketing conservation.
// ============================================================================
const pa = await openPage(busyFixture);

// Kick the machine, then start recording through the real button (the same
// control a person clicks) so the whole App.tsx wiring — the 250 ms poll
// timer, the strip render — is exercised, not just the engine.
await pa.evaluate(() => {
  const w = window.simlab.world;
  w.sim.placeBlock(31, 7, 13, "minecraft:air");
  w.applyChanges(w.drainChanges());
});
await pa.click('button:has-text("record")');
await pa.waitForTimeout(50); // let the polling effect mount and fire once

// Drive ticks the same way the frame loop does (`step()` then drain/apply),
// straight from the console rather than through the run button, so this
// probe does not depend on wall-clock pacing to build up activity.
const ran = await pa.evaluate(() => {
  const w = window.simlab.world;
  for (let i = 0; i < 400; i++) {
    w.sim.step();
    w.applyChanges(w.drainChanges());
  }
  return Number(w.sim.tickCount());
});
console.log(`BB: drove 400 ticks to tick ${ran}`);

await pa.waitForTimeout(300); // let the 250ms poll timer catch up

// --- 1: columns == active ticks, not the span ------------------------------
const a1 = await pa.evaluate(() => {
  const w = window.simlab.world;
  const activity = JSON.parse(w.sim.timelineActivityJson());
  return {
    activeTicks: activity.ticks.length,
    span: activity.end - activity.start + 1,
    cols: document.querySelectorAll(".timeline-col").length,
    trackWidth: document.querySelector(".timeline-columns")?.clientWidth ?? 0,
  };
});
console.log(
  `activity: ${a1.activeTicks} active ticks over a span of ${a1.span} ticks, ` +
    `strip (${a1.trackWidth}px) drew ${a1.cols} columns`,
);
check(
  "unbucketed: column count equals active-tick count",
  a1.cols === a1.activeTicks,
  `${a1.cols} cols vs ${a1.activeTicks} active ticks`,
);
check(
  "column count is not the tick span (idle ticks are skipped)",
  a1.cols < a1.span,
  `${a1.cols} cols < ${a1.span} span ticks`,
);

// --- 3: bucketing conserves ticks and counts --------------------------------
// Shrink the strip well below the pixel budget the active-tick count needs,
// forcing bucketing to engage regardless of exactly how much BB did this run.
const narrowWidth = Math.max(60, Math.round(a1.activeTicks / 5));
await pa.setViewportSize({ width: narrowWidth + 200, height: 500 });
await pa.waitForTimeout(250); // ResizeObserver callback + a render
const bucketed = await pa.evaluate(() => {
  const w = window.simlab.world;
  const raw = JSON.parse(w.sim.timelineActivityJson()).ticks;
  const total = (key) => raw.reduce((s, t) => s + t[key], 0);
  const rawTicks = new Set(raw.map((t) => t.tick));
  const cols = [...document.querySelectorAll(".timeline-col")].map((el) => ({
    first: Number(el.dataset.firstTick),
    last: Number(el.dataset.lastTick),
    changes: Number(el.dataset.changes),
    inputs: Number(el.dataset.inputs),
    pistons: Number(el.dataset.pistons),
  }));
  return {
    rawCount: raw.length,
    colCount: cols.length,
    trackWidth: document.querySelector(".timeline-columns")?.clientWidth ?? 0,
    resolvable: cols.every(
      (c) => rawTicks.has(c.first) && rawTicks.has(c.last) && c.first <= c.last,
    ),
    totals: { changes: total("changes"), inputs: total("inputs"), pistons: total("pistons") },
    sums: {
      changes: cols.reduce((s, c) => s + c.changes, 0),
      inputs: cols.reduce((s, c) => s + c.inputs, 0),
      pistons: cols.reduce((s, c) => s + c.pistons, 0),
    },
  };
});
console.log(
  `bucketing: strip narrowed to ${bucketed.trackWidth}px, ${bucketed.rawCount} active ticks → ${bucketed.colCount} columns`,
);
check("bucketing engaged (fewer columns than active ticks)", bucketed.colCount < bucketed.rawCount);
check(
  "every bucketed column's first/last tick resolves to a real recorded tick",
  bucketed.resolvable,
);
check(
  "summed changes across buckets equal the unbucketed total",
  bucketed.sums.changes === bucketed.totals.changes,
  `${bucketed.sums.changes} vs ${bucketed.totals.changes}`,
);
check(
  "summed inputs across buckets equal the unbucketed total",
  bucketed.sums.inputs === bucketed.totals.inputs,
  `${bucketed.sums.inputs} vs ${bucketed.totals.inputs}`,
);
check(
  "summed pistons across buckets equal the unbucketed total",
  bucketed.sums.pistons === bucketed.totals.pistons,
  `${bucketed.sums.pistons} vs ${bucketed.totals.pistons}`,
);
await pa.close();

// ============================================================================
// Part B (a settling door): idle ticks add no columns.
// ============================================================================
const pb = await openPage(settleFixture);
await pb.click('button:has-text("record")');
await pb.waitForTimeout(50);

const kick = await pb.evaluate(() => {
  const w = window.simlab.world;
  const [dx, dy, dz] = w.dims;
  for (let x = 0; x < dx; x++)
    for (let y = 0; y < dy; y++)
      for (let z = 0; z < dz; z++) {
        if (/lever/.test(w.blockAt(x, y, z))) {
          w.sim.useBlock(x, y, z);
          w.applyChanges(w.drainChanges());
          return [x, y, z];
        }
      }
  return null;
});
if (!kick) {
  console.log("❌ settling fixture has no lever to kick — cannot run the idle assertion");
  failures++;
} else {
  console.log(`door: kicked lever @ ${kick.join(",")}`);
  const settle = await pb.evaluate(() => {
    const w = window.simlab.world;
    let steps = 0;
    for (; steps < 200; steps++) {
      w.sim.step();
      w.applyChanges(w.drainChanges());
      if (w.sim.isQuiescent?.()) {
        steps++;
        break;
      }
    }
    return { steps, quiescent: !!w.sim.isQuiescent?.() };
  });
  check("settling fixture actually reaches quiescence", settle.quiescent, `after ${settle.steps} ticks`);

  await pb.waitForTimeout(300); // let the strip catch up to the settled state
  const before = await pb.evaluate(() => document.querySelectorAll(".timeline-col").length);

  // Now step well past settling — every one of these ticks is genuinely
  // idle: the engine agrees (`isQuiescent()` stays true throughout).
  const idleRun = await pb.evaluate(() => {
    const w = window.simlab.world;
    let stillQuiescent = true;
    for (let i = 0; i < 80; i++) {
      w.sim.step();
      w.applyChanges(w.drainChanges());
      stillQuiescent = stillQuiescent && !!w.sim.isQuiescent?.();
    }
    return { stillQuiescent };
  });
  check("the 80 ticks driven for the idle check were actually idle", idleRun.stillQuiescent);

  await pb.waitForTimeout(300); // at least one more poll tick
  const after = await pb.evaluate(() => document.querySelectorAll(".timeline-col").length);
  check(
    "idle ticks add no new columns — the strip does not advance when nothing happens",
    after === before,
    `${before} → ${after} columns across 80 idle ticks`,
  );
}
await pb.close();

console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
console.log(
  failures === 0 && errs.length === 0 ? "\n✅ all checks passed" : `\n❌ ${failures} check(s) failed`,
);
await b.close();
process.exit(failures === 0 && errs.length === 0 ? 0 : 1);
