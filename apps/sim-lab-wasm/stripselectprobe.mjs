/** Does Task 4's selection/cycle layer on the activity strip do what the
 * brief and its hand-off decisions claim?
 *
 * Six things must hold, matching the task-4 brief's "Verify" list and the
 * hand-off's numbered decisions:
 *
 *  1. A synthesised drag from an earlier column to a later one produces
 *     exactly `{start: earlier.firstTick, end: later.lastTick}` — real ticks
 *     read off the columns' own data attributes, never column indices.
 *  2. The reverse drag over the same two columns produces the identical
 *     span (Decision 2: drag direction is normalised).
 *  3. A click without a drag clears the selection.
 *  4. A column rendered at the minimum presence height (`{changes:0}`) is
 *     still hit-testable: the probe deliberately grabs a point near the TOP
 *     of that column's slot — outside the painted bar, which is only 6%
 *     tall and bottom-aligned — to prove the hit target is the full-height
 *     slot, not the bar (Decision 1). This is the assertion most worth
 *     having: without the `.timeline-col-hit` wrapper, this drag would miss
 *     the column entirely.
 *  5. "find cycles" is disabled while recording and enabled once stopped
 *     (Decision 4); clicking it once, with BB recorded, shades at least one
 *     span, and clicking that span selects exactly that cycle's ticks.
 *  6. An absent cycle (a short run with nothing to find) renders no
 *     `.timeline-cycle` element and raises no page error — quiet, not an
 *     exception.
 *
 * Same harness shape as `stripprobe.mjs` / `verify-handover.mjs`: real page,
 * real engine, `window.simlab` for the console-equivalent calls a human
 * would otherwise make by hand. Selection state is read the same way a
 * human verifies it — off the strip's own `.timeline-readout` text — never
 * by reaching into React state, which has no console-equivalent path.
 */
import { chromium } from "playwright";

const busyFixture = process.argv[2];
// A build with a genuine, fast, deterministic recurrence, for the cycle
// tests: BB (the brief's fixture) is a perpetual flying machine whose
// bounding box keeps changing as it goes — empirically it does not produce
// a detected cycle within thousands of ticks (see task-4-report.md), so it
// is the wrong fixture for this half of the probe. A door that is opened
// and then closed returns to bit-for-bit the same world state, which is a
// fast, reliable exact cycle to assert against; opened-and-left-open is an
// equally reliable case with NO cycle, for the null-case assertion.
// Defaults to the door corpus `stripprobe.mjs` already points at.
const doorFixture =
  process.argv[3] ??
  "../../crates/mc-tick/tests/corpus/litematics/6x6_sliding_door.litematic";
if (!busyFixture) {
  console.error("usage: node stripselectprobe.mjs <busy schematic, e.g. BB> [door schematic]");
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

function parseReadout(text) {
  const m = /sel (\d+)–(\d+)/.exec(text);
  return m ? { start: Number(m[1]), end: Number(m[2]) } : null;
}

async function readSelection(p) {
  const text = await p.locator(".timeline-readout").innerText();
  return parseReadout(text);
}

/** All drawn columns, in strip order, with their exact tick range and the
 * client-space coordinates of their FULL hit slot (`.timeline-col-hit`) —
 * not the painted bar inside it. `yTop` is near the top of the slot
 * (outside a minimum-height bar); `yBottom` is near the bottom (inside
 * where any bar's visible pixels actually are). */
async function readColumns(p) {
  return p.evaluate(() => {
    return [...document.querySelectorAll(".timeline-col-hit")].map((el) => {
      const r = el.getBoundingClientRect();
      const bar = el.querySelector(".timeline-col");
      return {
        firstTick: Number(el.dataset.firstTick),
        lastTick: Number(el.dataset.lastTick),
        changes: Number(bar?.dataset.changes ?? -1),
        x: r.left + r.width / 2,
        yTop: r.top + Math.min(5, r.height / 4),
        yBottom: r.top + r.height - 2,
        height: r.height,
      };
    });
  });
}

async function drag(p, from, to) {
  await p.mouse.move(from.x, from.y);
  await p.mouse.down();
  await p.mouse.move(to.x, to.y, { steps: 12 });
  await p.mouse.up();
}

// ============================================================================
// Part A: drag-select semantics (BB, driven, recording stopped so the strip
// is stable for the duration of these assertions).
// ============================================================================
const pa = await openPage(busyFixture);

await pa.evaluate(() => {
  const w = window.simlab.world;
  w.sim.placeBlock(31, 7, 13, "minecraft:air");
  w.applyChanges(w.drainChanges());
});
await pa.click('button:has-text("record")');
await pa.waitForTimeout(50);
const ran = await pa.evaluate(() => {
  const w = window.simlab.world;
  for (let i = 0; i < 400; i++) {
    w.sim.step();
    w.applyChanges(w.drainChanges());
  }
  return Number(w.sim.tickCount());
});
console.log(`BB: drove ${ran} ticks`);

// Guarantee at least one {changes:0} column exists for assertion 4, the
// same way `stripprobe.mjs` Part C does: right-click a plain, non-air,
// non-interactive block through the app's own click path
// (`world.sim.useBlock`) until one produces zero block changes. Not every
// BB run happens to contain one on its own (this run's 400 driven ticks
// did not), and the point of assertion 4 is the hit-test fix, not chance.
const injected = await pa.evaluate(() => {
  const w = window.simlab.world;
  const [dx, dy, dz] = w.dims;
  const INTERACTIVE = /lever|button|note_block|trapdoor|_door|_gate|repeater|comparator|daylight/;
  for (let x = 0; x < dx; x++)
    for (let y = 0; y < dy; y++)
      for (let z = 0; z < dz; z++) {
        const s = w.blockAt(x, y, z);
        if (s === "minecraft:air" || INTERACTIVE.test(s)) continue;
        const ch = w.drainChanges(); // flush anything pending first
        if (ch.length) w.applyChanges(ch);
        w.sim.useBlock(x, y, z);
        const produced = w.drainChanges();
        w.applyChanges(produced);
        if (produced.length === 0) return { pos: [x, y, z], state: s };
      }
  return null;
});
console.log(`zero-change tick injected: ${JSON.stringify(injected)}`);

await pa.click('button:has-text("stop")');
await pa.waitForTimeout(300);

let cols = await readColumns(pa);
console.log(`strip drew ${cols.length} columns`);
if (cols.length < 4) {
  console.log("❌ not enough columns to run the drag assertions — widen the run");
  failures++;
} else {
  const earlier = cols[1];
  const later = cols[cols.length - 2];

  // --- 1: forward drag ------------------------------------------------------
  await drag(pa, { x: earlier.x, y: earlier.yBottom }, { x: later.x, y: later.yBottom });
  await pa.waitForTimeout(30);
  const fwd = await readSelection(pa);
  check(
    "forward drag selects {start: earlier.firstTick, end: later.lastTick}",
    !!fwd && fwd.start === earlier.firstTick && fwd.end === later.lastTick,
    `got ${JSON.stringify(fwd)}, want {start:${earlier.firstTick}, end:${later.lastTick}}`,
  );

  // --- 2: reverse drag over the same two columns -----------------------------
  await drag(pa, { x: later.x, y: later.yBottom }, { x: earlier.x, y: earlier.yBottom });
  await pa.waitForTimeout(30);
  const rev = await readSelection(pa);
  check(
    "reverse drag over the same two columns produces the identical span",
    !!rev && !!fwd && rev.start === fwd.start && rev.end === fwd.end,
    `forward ${JSON.stringify(fwd)} vs reverse ${JSON.stringify(rev)}`,
  );

  // --- 3: click without drag clears the selection ----------------------------
  await pa.mouse.move(earlier.x, earlier.yBottom);
  await pa.mouse.down();
  await pa.mouse.up();
  await pa.waitForTimeout(30);
  const cleared = await readSelection(pa);
  check("a click without a drag clears the selection", cleared === null, `got ${JSON.stringify(cleared)}`);
}

// --- 4: a {changes:0} column is hit-testable at its full slot, not just its
//        painted bar --------------------------------------------------------
cols = await readColumns(pa);
const zero = cols.find((c) => c.changes === 0);
if (!zero) {
  console.log("❌ no {changes:0} column found on this run — cannot prove the full-height hit fix");
  failures++;
} else {
  const zeroIdx = cols.indexOf(zero);
  const neighbour = cols[zeroIdx === 0 ? 1 : zeroIdx - 1];
  console.log(
    `zero-change column: ticks ${zero.firstTick}–${zero.lastTick}, slot height ${zero.height.toFixed(1)}px, ` +
      `grabbed ${(zero.yBottom - zero.yTop).toFixed(0)}px above where its painted bar sits`,
  );
  await drag(pa, { x: zero.x, y: zero.yTop }, { x: neighbour.x, y: neighbour.yBottom });
  await pa.waitForTimeout(30);
  const sel = await readSelection(pa);
  const [first, second] = zero.firstTick <= neighbour.firstTick ? [zero, neighbour] : [neighbour, zero];
  check(
    "a {changes:0} column is hit-testable from the top of its slot, not just its painted bar",
    !!sel && sel.start === first.firstTick && sel.end === second.lastTick,
    `got ${JSON.stringify(sel)}, want {start:${first.firstTick}, end:${second.lastTick}}`,
  );
}
await pa.close();

/** Find the door's lever and kick it (one full open OR close pulse),
 * stepping to quiescence. */
async function kickDoorLever(p) {
  return p.evaluate(async () => {
    const w = window.simlab.world;
    const [dx, dy, dz] = w.dims;
    let pos = null;
    for (let x = 0; x < dx && !pos; x++)
      for (let y = 0; y < dy && !pos; y++)
        for (let z = 0; z < dz && !pos; z++)
          if (/lever/.test(w.blockAt(x, y, z))) pos = [x, y, z];
    if (!pos) return { error: "no lever on this fixture" };
    w.sim.useBlock(...pos);
    w.applyChanges(w.drainChanges());
    let steps = 0;
    for (; steps < 200; steps++) {
      w.sim.step();
      w.applyChanges(w.drainChanges());
      if (w.sim.isQuiescent?.()) {
        steps++;
        break;
      }
    }
    return { pos, steps, quiescent: !!w.sim.isQuiescent?.() };
  });
}

// ============================================================================
// Part B: cycle detection. Open the door, then close it: the world returns
// to bit-for-bit its starting state, a fast and deterministic exact cycle
// (see task-4-report.md for why BB — a perpetual, ever-translating flying
// machine — was tried first and rejected: it never produced a detected
// cycle within thousands of driven ticks).
// ============================================================================
const pb = await openPage(doorFixture);
await pb.click('button:has-text("record")');
await pb.waitForTimeout(50);

const disabledWhileRecording = await pb.locator('button:has-text("find cycles")').isDisabled();
check("\"find cycles\" is disabled while recording (Decision 4)", disabledWhileRecording);

const opened = await kickDoorLever(pb);
const closed = await kickDoorLever(pb);
console.log(`door: opened in ${opened.steps} ticks, closed in ${closed.steps} ticks`);
if (!opened.quiescent || !closed.quiescent) {
  console.log("❌ door did not settle both ways — cannot run the cycle assertions");
  failures++;
}

await pb.click('button:has-text("stop")');
await pb.waitForTimeout(300);

const disabledOnceStopped = await pb.locator('button:has-text("find cycles")').isDisabled();
check("\"find cycles\" is enabled once recording has stopped", !disabledOnceStopped);

// Exactly one call: the button click itself. Nothing here polls
// timelineCyclesJson() on a timer, mirroring what the app does — see the
// comment on `findCycles` in `App.tsx` for why (cost + staleness).
await pb.click('button:has-text("find cycles")');
await pb.waitForTimeout(50);

const found = await pb.evaluate(() => ({
  count: document.querySelectorAll(".timeline-cycle").length,
  testids: [...document.querySelectorAll(".timeline-cycle")].map((e) => e.dataset.testid),
}));
console.log(`find cycles: ${found.count} shaded span(s) — ${JSON.stringify(found.testids)}`);
check(
  "find cycles shades at least one span for an opened-then-closed door",
  found.count >= 1,
  JSON.stringify(found),
);

if (found.count >= 1) {
  const cycles = await pb.evaluate(() => JSON.parse(window.simlab.world.sim.timelineCyclesJson()));
  const chosenId = found.testids[0];
  const wanted = chosenId.endsWith("exact") ? cycles.exact : cycles.translated;
  await pb.click(`.timeline-cycle[data-testid="${chosenId}"]`);
  await pb.waitForTimeout(30);
  const sel = await readSelection(pb);
  check(
    "clicking a shaded cycle selects exactly that cycle's start/end ticks",
    !!sel && !!wanted && sel.start === wanted.start && sel.end === wanted.end,
    `got ${JSON.stringify(sel)}, want ${JSON.stringify(wanted)}`,
  );
}
await pb.close();

// ============================================================================
// Part C: an absent cycle is a quiet no-op — no shade, no page error.
// The door opened and left open (no close pulse) never returns to a prior
// state, exactly like an adder that never loops — the ordinary outcome.
// ============================================================================
const pc = await openPage(doorFixture);
await pc.click('button:has-text("record")');
await pc.waitForTimeout(50);
const openedOnly = await kickDoorLever(pc);
console.log(`door (left open): settled in ${openedOnly.steps} ticks`);
await pc.click('button:has-text("stop")');
await pc.waitForTimeout(300);
await pc.click('button:has-text("find cycles")');
await pc.waitForTimeout(50);
const nullCase = await pc.evaluate(() => ({
  json: window.simlab.world.sim.timelineCyclesJson(),
  shades: document.querySelectorAll(".timeline-cycle").length,
  errorShown: !!document.querySelector(".status.error"),
}));
console.log(`null-cycle case: ${nullCase.json}`);
const parsedNull = JSON.parse(nullCase.json);
check(
  "an opened-and-left-open door reports no cycle at all (ground truth for the no-op check)",
  parsedNull.exact === null && parsedNull.translated === null,
  nullCase.json,
);
check(
  "a fully-null cycle result renders no shaded span and shows no error state",
  nullCase.shades === 0 && !nullCase.errorShown,
  `${nullCase.shades} shades, error shown: ${nullCase.errorShown}`,
);
await pc.close();

console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
console.log(
  failures === 0 && errs.length === 0 ? "\n✅ all checks passed" : `\n❌ ${failures} check(s) failed`,
);
await b.close();
process.exit(failures === 0 && errs.length === 0 ? 0 : 1);
