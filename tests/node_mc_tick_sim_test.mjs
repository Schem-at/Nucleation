// Node test for the generated mc-tick bindings (TickSimulation).
//
// Run with: node tests/node_mc_tick_sim_test.mjs
// Prereq: cargo build --release --target wasm32-unknown-unknown --lib --features bridge,mc-tick
//
// Replays the shulker-pipeline scenario the Rust case
// (crates/mc-tick/tests/cases/shulker_pipeline.test.json) pins, and opens the
// 6x6 in-world door — asserting the same end states through the bindings.

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

// --- shulker pipeline, seeded ---
const snbt = fs.readFileSync(
  "crates/mc-tick/tests/corpus/structures/shulker_pipeline.snbt",
  "utf8",
);
const sim = TickSimulation.fromSnbt(snbt, TickSettleMode.Placement, 0, 0, 0, "");
sim.setRngSeed(12345n);

const RB = "minecraft:redstone_block";
const actions = [
  [5, -1, 2, 1, RB],
  [30, 2, 0, 1, RB],
  [34, 2, 0, 1, "minecraft:air"],
  [38, 2, 0, 1, RB],
  [42, 2, 0, 1, "minecraft:air"],
  [46, 1, 1, 2, RB],
  [50, 1, 4, 1, RB],
  [54, 1, 4, 1, "minecraft:air"],
  [60, 1, 1, 2, "minecraft:air"],
  [82, 2, 0, 1, RB],
  [86, 2, 0, 1, "minecraft:air"],
];
for (let t = 0; t <= 110; t++) {
  if (t === 12) {
    expect(
      sim.getBlock(1, 2, 1).startsWith("minecraft:white_shulker_box"),
      "shulker placed by the dispenser at tick 12",
    );
  }
  if (t === 64) {
    expect(sim.getBlock(1, 2, 1) === "minecraft:air", "shulker broken by tick 64");
  }
  for (const [at, x, y, z, state] of actions) {
    if (at === t) sim.placeBlock(x, y, z, state);
  }
  if (t < 110) sim.step();
}
expect(sim.tickCount() === 110, "tickCount reaches 110");

const entities = JSON.parse(sim.itemEntitiesJson());
const east = (e) => e.pos[0] >= 3 && e.pos[0] < 6;
const diamonds = entities.items
  .filter((e) => e.item === "minecraft:diamond" && east(e))
  .reduce((n, e) => n + e.count, 0);
expect(diamonds === 2, `two diamonds land east of the dropper (got ${diamonds})`);
const shulkers = entities.items.filter(
  (e) => e.item === "minecraft:white_shulker_box" && east(e),
);
expect(shulkers.length === 1, "one shulker item lands east");
expect(
  shulkers.length === 1 && shulkers[0].contents.length === 0,
  "the shipped shulker is empty",
);

const summary = JSON.parse(sim.eventsSummaryJson());
expect(summary.length > 10, "events summary has per-tick rows");
expect(
  summary.some((r) => r.piston > 0),
  "piston activity shows in the summary",
);
const changes = JSON.parse(sim.changesJson());
expect(changes.length > 20, "block changes recorded");
const snapshot = JSON.parse(sim.worldSnapshotJson());
expect(snapshot.length > 20, "world snapshot lists blocks");

// --- checkpoint/restore ---
const cp = sim.checkpoint();
sim.placeBlock(0, 3, 0, RB);
sim.step();
sim.restore(cp);
expect(sim.getBlock(0, 3, 0) === "minecraft:air", "restore rewinds a write");

// --- the 6x6 door opens on its lever ---
const door = fs.readFileSync(
  "crates/mc-tick/tests/corpus/structures/door_6x6_inworld.snbt",
  "utf8",
);
const doorSim = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
doorSim.useBlock(10, 4, 1);
doorSim.run(40);
const doorChanges = JSON.parse(doorSim.changesJson());
expect(
  doorChanges.some((c) => c.to.includes("moving_piston")),
  "the 6x6 door's pistons move after the lever click",
);

// --- what the engine currently has in flight ---
//
// The renderer's answer to "which block is sliding, from where, arriving
// when". Reconstructing it from `changesJson` is a reimplementation of piston
// mechanics in the host, which is how an animation ends up on a different
// clock from the simulation that decides the landing.
const flightSim = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
flightSim.useBlock(10, 4, 1);
let air = [];
for (let t = 0; t < 40 && air.length === 0; t++) {
  flightSim.step();
  air = JSON.parse(flightSim.movingBlocksJson());
}
expect(air.length > 0, `the door reports blocks in flight (got ${air.length})`);
const m = air[0] ?? {};
expect(
  typeof m.state === "string" && !m.state.startsWith("minecraft:moving_piston"),
  `a flight names the block being carried, not the placeholder (${m.state})`,
);
const step = { east: [1, 0, 0], west: [-1, 0, 0], up: [0, 1, 0], down: [0, -1, 0], south: [0, 0, 1], north: [0, 0, -1] };
const d = step[m.dir] ?? [0, 0, 0];
expect(
  m.from?.every?.((v, i) => v + d[i] === m.to[i]),
  `it travels one cell ${m.dir}: ${JSON.stringify(m.from)} -> ${JSON.stringify(m.to)}`,
);
expect(
  air.every((f) => f.started === m.started && f.lands === m.lands),
  "blocks dispatched together share one flight window",
);
expect(
  m.lands > m.started && m.started === flightSim.tickCount() - 1,
  `the window is the tick it set off to the tick it lands (${m.started}..${m.lands} at tick ${flightSim.tickCount()})`,
);

// A retracting piston is the one move where what lands is not what travels:
// vanilla's PistonHeadRenderer draws a `piston_head` on the interpolated slot
// and the body, EXTENDED=true, parked outside it. Run the door long enough for
// its pistons to come home and check the split holds everywhere.
let retractions = 0;
let split = true;
for (let t = 0; t < 120; t++) {
  flightSim.step();
  for (const f of JSON.parse(flightSim.movingBlocksJson())) {
    const retracting = f.source_piston && !f.extending;
    if (retracting) {
      retractions++;
      if (
        !f.carried.startsWith("minecraft:piston_head") ||
        !(f.remains ?? "").includes("extended=true") ||
        !f.state.includes("extended=false")
      ) split = false;
    } else if (f.carried !== f.state || f.remains !== null) split = false;
    // A moving arm comes in two lengths — the game shortens it while the head
    // is beside its body, or the shaft passes through the back of the piston.
    if (f.carried.startsWith("minecraft:piston_head")) {
      if (
        !f.carried_short?.includes("short=true") ||
        f.carried_short.replace("short=true", "short=false") !== f.carried
      ) split = false;
    } else if (f.carried_short !== null) split = false;
  }
}
expect(retractions > 0, `the door's pistons retract (${retractions} flight-ticks seen)`);
expect(
  split,
  "a retraction carries a piston_head and parks an extended body; every other move carries itself and parks nothing; " +
    "every moving arm offers both lengths",
);

// --- the event timeline ---
const tl = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
tl.recordTimeline();
tl.useBlock(10, 4, 1);
tl.run(40);
tl.stopTimeline();
// A stopped span stays readable — that is what makes it exportable.
const activity = JSON.parse(tl.timelineActivityJson());
expect(activity.ticks.length > 0, `the door records activity (${activity.ticks.length} active ticks)`);
expect(
  activity.ticks.every((t) => t.changes > 0 || t.inputs > 0 || t.pistons > 0),
  "every listed tick did something — idle ticks are absent, so a still build does not advance the strip",
);
expect(
  activity.ticks.every((t, i, a) => i === 0 || t.tick > a[i - 1].tick),
  "ticks are strictly ascending",
);
expect(
  activity.ticks.length < activity.end - activity.start + 1,
  `the door has idle ticks that were skipped (${activity.ticks.length} of ${activity.end - activity.start + 1})`,
);

console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail === 0 ? 0 : 1);
