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

// --- turning it back off ----------------------------------------------------
sim.recordUpdates(false);
expect(sim.updatesCount() === 0, "recordUpdates(false) drops the log");

console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail === 0 ? 0 : 1);
