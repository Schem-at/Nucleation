// Node test for the update recorder — the data behind a sub-tick propagation
// view (apps/door-cert-wasm/ROADMAP.md §6).
//
// Run with: node tests/node_mc_tick_updates_test.mjs
// Prereq: cargo build --release --target wasm32-unknown-unknown --lib \
//           --features bridge,mc-tick
//
// Asserts the three properties a scrubber depends on: recording is off until
// asked for, deliveries come out in (tick, seq) order, and `state` is the block
// as it stood *at dispatch time* rather than at the tick boundary — which is
// the whole reason the log exists.

import fs from "node:fs";
import { TickSimulation, TickSettleMode } from "../bindings/js/index.mjs";

let pass = 0;
let fail = 0;
function expect(cond, what) {
  if (cond) {
    pass++;
    console.log("  ok  " + what);
  } else {
    fail++;
    console.log("  FAIL " + what);
  }
}

const door = fs.readFileSync(
  "crates/mc-tick/tests/corpus/structures/door_6x6_inworld.snbt",
  "utf8",
);

// --- off by default ---------------------------------------------------------
const quiet = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
quiet.useBlock(10, 4, 1);
quiet.run(20);
expect(quiet.updatesCount() === 0, "no updates recorded until asked for");
expect(JSON.parse(quiet.updatesJson()).length === 0, "updatesJson is empty when off");

// --- recording on -----------------------------------------------------------
const sim = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
// Block-change recording is already on: the bridge enables it at construction.
sim.recordUpdates(true);
const clickTick = sim.tickCount();
sim.useBlock(10, 4, 1);
sim.run(40);

const updates = JSON.parse(sim.updatesJson());
expect(updates.length > 0, `updates are recorded (${updates.length})`);
expect(updates.length === sim.updatesCount(), "updatesCount agrees with the log");

// Ordered by (tick, seq), with seq restarting at 0 each tick.
let ordered = true;
let seqRestarts = true;
for (let i = 1; i < updates.length; i++) {
  const a = updates[i - 1];
  const b = updates[i];
  if (b.tick < a.tick || (b.tick === a.tick && b.seq !== a.seq + 1)) ordered = false;
  if (b.tick > a.tick && b.seq !== 0) seqRestarts = false;
}
expect(ordered, "deliveries are in (tick, seq) order");
expect(seqRestarts, "seq restarts at 0 on each new tick");

// Every update carries a phase and a kind we understand.
const kinds = new Set(updates.map((u) => u.kind));
const phases = new Set(updates.map((u) => u.phase));
expect(
  [...kinds].every((k) => k === "neighbor" || k === "shape"),
  `kinds are neighbor/shape (saw ${[...kinds].join(", ")})`,
);
expect(phases.size > 0, `phases are labelled (${[...phases].join(", ")})`);

// --- paging -----------------------------------------------------------------
const firstTick = updates[0].tick;
const page = JSON.parse(sim.updatesJsonBetween(firstTick, firstTick + 1));
expect(
  page.length > 0 && page.every((u) => u.tick === firstTick),
  `updatesJsonBetween pages one tick (${page.length} in tick ${firstTick})`,
);
expect(
  page.length === updates.filter((u) => u.tick === firstTick).length,
  "the page holds exactly that tick's updates",
);

// --- dispatch-time state ----------------------------------------------------
// The claim under test: `state` is what stood at the position when the update
// landed, NOT the block's value at the tick boundary. Take a cell the change
// log shows changing mid-run, and check the updates delivered to it after that
// change report the new block while ones before report the old.
const changes = JSON.parse(sim.changesJson());
const moved = changes.find(
  (c) => c.tick > clickTick && c.from !== c.to && !c.to.includes("moving_piston"),
);
expect(moved !== undefined, "the door produced a block change to check against");

if (moved) {
  const key = moved.pos.join(",");
  const at = updates.filter((u) => u.pos.join(",") === key);
  const before = at.filter((u) => u.tick < moved.tick);
  const after = at.filter((u) => u.tick > moved.tick);
  expect(
    before.length === 0 || before.every((u) => u.state === moved.from),
    `updates before tick ${moved.tick} at ${key} see the old block`,
  );
  expect(
    after.length === 0 || after.every((u) => u.state === moved.to),
    `updates after tick ${moved.tick} at ${key} see the new block`,
  );
}

// --- heat agrees with raw ---------------------------------------------------
const lastTick = updates[updates.length - 1].tick;
const heat = JSON.parse(sim.updatesHeatJson(firstTick, lastTick + 1));
expect(
  Array.isArray(heat.phases) && heat.phases[0] === "boundary",
  "heat carries a phase legend",
);

const rawPerCell = new Map(); // "tick|x,y,z" -> {n, nb, sh, ph}
for (const u of updates) {
  const key = `${u.tick}|${u.pos.join(",")}`;
  let e = rawPerCell.get(key);
  if (!e) rawPerCell.set(key, (e = { n: 0, nb: 0, sh: 0, ph: new Map() }));
  e.n++;
  if (u.kind === "neighbor") e.nb++;
  else e.sh++;
  e.ph.set(u.phase, (e.ph.get(u.phase) ?? 0) + 1);
}

let heatCells = 0;
let countsMatch = true;
let splitsMatch = true;
let phasesMatch = true;
let totalsMatch = true;
for (const t of heat.ticks) {
  let tickSum = 0;
  for (const c of t.cells) {
    heatCells++;
    tickSum += c.n;
    const raw = rawPerCell.get(`${t.tick}|${c.p.join(",")}`);
    if (!raw) {
      countsMatch = false;
      continue;
    }
    if (raw.n !== c.n) countsMatch = false;
    if (raw.nb !== c.nb || raw.sh !== c.sh || c.nb + c.sh !== c.n) splitsMatch = false;
    for (const [phaseName, n] of raw.ph) {
      if (c.ph[heat.phases.indexOf(phaseName)] !== n) phasesMatch = false;
    }
  }
  if (tickSum !== t.total) totalsMatch = false;
}
expect(heatCells === rawPerCell.size, `heat has one row per (tick, cell) — ${heatCells}`);
expect(countsMatch, "heat cell counts match the raw log");
expect(splitsMatch, "heat neighbour/shape split matches and sums to the total");
expect(phasesMatch, "heat phase breakdown matches the raw log");
expect(totalsMatch, "each heat tick's total is the sum of its cells");

// --- wave expands back to raw ----------------------------------------------
const busiest = heat.ticks.reduce((a, b) => (b.total > a.total ? b : a));
const wave = JSON.parse(sim.updatesWaveJson(busiest.tick));
const rawTick = updates.filter((u) => u.tick === busiest.tick);
expect(wave.n === rawTick.length, `wave holds the whole tick (${wave.n})`);
expect(
  wave.pos.length === wave.n * 3 &&
    wave.kind.length === wave.n &&
    wave.phase.length === wave.n &&
    wave.from.length === wave.n &&
    wave.state.length === wave.n,
  "wave arrays are parallel and correctly sized",
);

// seq is the array index, so index i must be raw delivery i of that tick.
let waveMatches = true;
for (let i = 0; i < wave.n; i++) {
  const raw = rawTick[i];
  if (
    wave.pos[i * 3] !== raw.pos[0] ||
    wave.pos[i * 3 + 1] !== raw.pos[1] ||
    wave.pos[i * 3 + 2] !== raw.pos[2] ||
    wave.kinds[wave.kind[i]] !== raw.kind ||
    wave.phases[wave.phase[i]] !== raw.phase ||
    wave.dirs[wave.from[i]] !== raw.from ||
    wave.states[wave.state[i]] !== raw.state
  ) {
    waveMatches = false;
    break;
  }
}
expect(waveMatches, "wave expands cell-for-cell back to the raw log");
expect(
  wave.states.length < wave.n / 10,
  `wave dedupes states hard (${wave.states.length} distinct across ${wave.n})`,
);

// --- turning it back off ----------------------------------------------------
sim.recordUpdates(false);
expect(sim.updatesCount() === 0, "recordUpdates(false) drops the log");

console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail === 0 ? 0 : 1);
