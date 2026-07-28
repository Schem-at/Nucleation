// Flying-machine evaluation benchmark for the wasm TickSimulation bindings.
//
// Run with: node tests/node_mc_tick_bench.mjs [evals]
// Prereq: cargo build --release --target wasm32-unknown-unknown --lib --features bridge,mc-tick
//
// One eval = the GA's unit of work: construct from SNBT (quiet settle),
// redstone-block kick at t2 (removed t4), run 80 ticks, read displacement.
// Phases are timed separately so optimization goes where the time is.

import fs from "node:fs";
import { TickSimulation, TickSettleMode } from "../bindings/js/index.mjs";

const EVALS = Number(process.argv[2] ?? 200);
const snbt = fs.readFileSync(
  "crates/mc-tick/tests/corpus/structures/flying_machine_east.snbt",
  "utf8",
);
const RB = "minecraft:redstone_block";

const hasScalars = typeof TickSimulation.prototype.nonAirMinX === "function";

function evalOnce(phases, useScalars) {
  let t = performance.now();
  const sim = TickSimulation.fromSnbt(snbt, TickSettleMode.Quiet, 0, 0, 0, "");
  phases.construct += performance.now() - t;

  t = performance.now();
  for (let tick = 0; tick < 80; tick++) {
    if (tick === 2) sim.placeBlock(2, 1, 1, RB);
    if (tick === 4) sim.placeBlock(2, 1, 1, "minecraft:air");
    sim.step();
  }
  phases.step += performance.now() - t;

  t = performance.now();
  let minX;
  if (useScalars) {
    minX = sim.nonAirMinX();
  } else {
    const snap = JSON.parse(sim.worldSnapshotJson());
    minX = Math.min(...snap.map((b) => b.pos[0]));
  }
  phases.query += performance.now() - t;
  return minX - 1; // machine starts at x=1
}

function bench(label, useScalars) {
  // Warmup.
  const warm = { construct: 0, step: 0, query: 0 };
  for (let i = 0; i < 10; i++) evalOnce(warm, useScalars);

  const phases = { construct: 0, step: 0, query: 0 };
  let displacement = 0;
  const t0 = performance.now();
  for (let i = 0; i < EVALS; i++) displacement = evalOnce(phases, useScalars);
  const total = performance.now() - t0;

  const per = total / EVALS;
  console.log(`${label}:`);
  console.log(`  displacement check: +${displacement} blocks (expect >= 6)`);
  console.log(`  ${(1000 / per).toFixed(1)} evals/sec  (${per.toFixed(2)} ms/eval over ${EVALS})`);
  for (const [k, v] of Object.entries(phases)) {
    console.log(`    ${k}: ${(v / EVALS).toFixed(3)} ms/eval  (${((100 * v) / total).toFixed(1)}%)`);
  }
  return 1000 / per;
}

bench("json-query eval", false);
if (hasScalars) {
  bench("scalar-query eval", true);
} else {
  console.log("scalar-query eval: skipped (nonAirMinX not in bindings yet)");
}
