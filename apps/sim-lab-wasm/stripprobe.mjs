/** Does the activity strip draw what the brief claims it draws?
 *
 * Four things must hold:
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
 *  4. **An input-only, change-free tick is still visible.** A right-click on
 *     a block whose `on_used` writes nothing (`crates/mc-tick/src/sim.rs`'s
 *     `use_block` logs the `InputAction` *before* checking whether the block
 *     has a behaviour at all, let alone one that changes anything) yields
 *     `{changes: 0, inputs: 1}` — a real entry in `ticks`, which per
 *     `TickSimulation::timeline_activity_json` means *something happened*.
 *     It must render at a nonzero, selectable height with its input mark,
 *     not `height: 0` (invisible and — for Task 4's selection — unclickable).
 *     Proven reachable through the app's own click handler: `App.tsx`'s
 *     `act()` calls `world.sim.useBlock(...hit.pos)` on *any* solid block the
 *     crosshair hits, unconditionally — the `INTERACTIVE` regex only picks
 *     the outline colour, it does not gate the click. Right-clicking a plain
 *     block (a diamond_block on BB) hits this path for real.
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

// ============================================================================
// Part C (BB again, fresh): an input-only tick (no block changes) is still
// a visible, marked column — not `height: 0`.
// ============================================================================
const pc = await openPage(busyFixture);
await pc.click('button:has-text("record")');
await pc.waitForTimeout(50);

const inputOnly = await pc.evaluate(() => {
  const w = window.simlab.world;
  const [dx, dy, dz] = w.dims;
  const INTERACTIVE = /lever|button|note_block|trapdoor|_door|_gate|repeater|comparator|daylight/;
  // Find a plain, non-interactive solid block and right-click it through
  // exactly the call the app's own click handler makes (`App.tsx`'s `act()`:
  // `world.sim.useBlock(...hit.pos)` then drain/apply) — not a mock of the
  // input path, the same one.
  for (let x = 0; x < dx; x++)
    for (let y = 0; y < dy; y++)
      for (let z = 0; z < dz; z++) {
        const s = w.blockAt(x, y, z);
        if (s === "minecraft:air" || INTERACTIVE.test(s)) continue;
        const before = w.sim.changesCount?.() ?? 0;
        w.sim.useBlock(x, y, z);
        const ch = w.drainChanges();
        w.applyChanges(ch);
        if (ch.length === 0 && w.sim.changesCount?.() === before) {
          return { pos: [x, y, z], state: s };
        }
        // This block's use did change something — not our case; the block
        // itself is now used, but that's still fine, keep scanning others.
      }
  return null;
});

if (!inputOnly) {
  console.log("❌ could not find a plain block whose use produces zero changes on this fixture");
  failures++;
} else {
  console.log(`input-only tick: used ${inputOnly.state} @ ${inputOnly.pos.join(",")}, 0 block changes`);
  await pc.waitForTimeout(300); // let the poll timer pick this up

  const col = await pc.evaluate(() => {
    const w = window.simlab.world;
    const activity = JSON.parse(w.sim.timelineActivityJson());
    const zeroChangeInput = activity.ticks.find((t) => t.inputs > 0 && t.changes === 0);
    const el = [...document.querySelectorAll(".timeline-col")].find(
      (e) => Number(e.dataset.inputs) > 0 && Number(e.dataset.changes) === 0,
    );
    return {
      groundTruth: zeroChangeInput ?? null,
      found: !!el,
      hasInputClass: el?.classList.contains("has-input") ?? false,
      heightPct: el ? parseFloat(el.style.height) : null,
    };
  });
  check(
    "engine actually recorded a zero-change input tick (ground truth)",
    !!col.groundTruth,
    JSON.stringify(col.groundTruth),
  );
  check("a column exists for that tick", col.found);
  check("it carries the input mark", col.hasInputClass);
  check(
    "it renders at a nonzero, selectable height (not height:0)",
    col.heightPct !== null && col.heightPct > 0,
    `height: ${col.heightPct}%`,
  );
}
await pc.close();

console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
console.log(
  failures === 0 && errs.length === 0 ? "\n✅ all checks passed" : `\n❌ ${failures} check(s) failed`,
);
await b.close();
process.exit(failures === 0 && errs.length === 0 ? 0 : 1);
