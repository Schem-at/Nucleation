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

console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail === 0 ? 0 : 1);
