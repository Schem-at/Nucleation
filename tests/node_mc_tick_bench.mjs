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

// ── The GA fast path: no SNBT, and one call per batch ────────────────────
//
// Engine B expressed as the GA does it: a flat palette-index array over a
// 5x1x2 bbox (cells ((y*bz)+z)*bx+x), palette = the run's alphabet.
const PAL = [
  "minecraft:air",
  "minecraft:observer[facing=west,powered=false]",
  "minecraft:slime_block",
  "minecraft:sticky_piston[extended=false,facing=west]",
  "minecraft:sticky_piston[extended=false,facing=east]",
  "minecraft:observer[facing=east,powered=false]",
].join(";");
const BX = 5, BY = 1, BZ = 2, TRAVEL = 26, X_OFF = 1;
const ENGINE_B_CELLS = (() => {
  // Genome space: the corpus snbt's coords MINUS the x_off the builder adds.
  const g = new Array(BX * BY * BZ).fill(0);
  const at = (x, z) => z * BX + x;
  g[at(0, 0)] = 1; // observer west
  g[at(1, 0)] = 2; // slime
  g[at(2, 0)] = 3; // sticky west
  g[at(1, 1)] = 4; // sticky east
  g[at(2, 1)] = 2; // slime
  g[at(3, 1)] = 5; // observer east
  return g;
})();
const KICK = [2, 1, 1]; // structure space: above the east-facing sticky piston

const hasFastPath = typeof TickSimulation.fromBlocks === "function";
if (hasFastPath) {
  {
    // from_blocks single evals (same protocol as above, no SNBT).
    const warmP = { construct: 0, step: 0, query: 0 };
    const phases = { construct: 0, step: 0, query: 0 };
    const one = (ph) => {
      let t = performance.now();
      const sim = TickSimulation.fromBlocks(
        BX, BY, BZ, TRAVEL, X_OFF, PAL, ENGINE_B_CELLS, 0,
        TickSettleMode.Quiet, 0, 0, 0,
      );
      ph.construct += performance.now() - t;
      t = performance.now();
      for (let tick = 0; tick < 80; tick++) {
        if (tick === 2) sim.placeBlock(2, 1, 1, RB);
        if (tick === 4) sim.placeBlock(2, 1, 1, "minecraft:air");
        sim.step();
      }
      ph.step += performance.now() - t;
      t = performance.now();
      const minX = sim.nonAirMinX();
      ph.query += performance.now() - t;
      return minX - 1;
    };
    for (let i = 0; i < 10; i++) one(warmP);
    let displacement = 0;
    const t0 = performance.now();
    for (let i = 0; i < EVALS; i++) displacement = one(phases);
    const total = performance.now() - t0;
    const per = total / EVALS;
    console.log("from_blocks eval:");
    console.log(`  displacement check: +${displacement} blocks (expect >= 6)`);
    console.log(`  ${(1000 / per).toFixed(1)} evals/sec  (${per.toFixed(2)} ms/eval over ${EVALS})`);
    for (const [k, v] of Object.entries(phases))
      console.log(`    ${k}: ${(v / EVALS).toFixed(3)} ms/eval  (${((100 * v) / total).toFixed(1)}%)`);
  }
  {
    // One evalFlightBatch call for the whole batch.
    const cells = [];
    const kicks = [];
    for (let i = 0; i < EVALS; i++) {
      cells.push(...ENGINE_B_CELLS);
      kicks.push(...KICK);
    }
    // Warmup.
    TickSimulation.evalFlightBatch(
      BX, BY, BZ, TRAVEL, X_OFF, PAL, ENGINE_B_CELLS, 0, KICK,
      80, 0n, -1, false, true,
    );
    const t0 = performance.now();
    const rows = JSON.parse(
      TickSimulation.evalFlightBatch(
        BX, BY, BZ, TRAVEL, X_OFF, PAL, cells, 0, kicks,
        80, 0n, -1, false, true,
      ),
    );
    const total = performance.now() - t0;
    const disp = rows[0][9] - rows[0][2]; // endMinX - startMinX
    console.log("batch eval (one wasm call, 80t):");
    console.log(`  displacement check: +${disp} blocks (expect >= 6)`);
    console.log(
      `  ${((1000 * EVALS) / total).toFixed(1)} evals/sec  (${(total / EVALS).toFixed(2)} ms/eval over ${EVALS})`,
    );
  }
  {
    // Reuse safety: the batch path restores a pristine checkpoint between
    // genomes — a machine's row must be identical whether it flies first,
    // after a dud, or alone in the batch.
    const dud = new Array(BX * BY * BZ).fill(0);
    dud[2] = 2;
    const mixedCells = [...ENGINE_B_CELLS, ...dud, ...ENGINE_B_CELLS];
    const mixedKicks = [...KICK, 3, 0, 0, ...KICK];
    const mixed = JSON.parse(
      TickSimulation.evalFlightBatch(
        BX, BY, BZ, TRAVEL, X_OFF, PAL, mixedCells, 0, mixedKicks,
        80, 0n, -1, false, true,
      ),
    );
    const solo = JSON.parse(
      TickSimulation.evalFlightBatch(
        BX, BY, BZ, TRAVEL, X_OFF, PAL, ENGINE_B_CELLS, 0, KICK,
        80, 0n, -1, false, true,
      ),
    );
    const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
    if (!same(mixed[0], mixed[2]) || !same(mixed[0], solo[0])) {
      console.error("REUSE LEAK: batch rows differ by position", {
        first: mixed[0],
        afterDud: mixed[2],
        solo: solo[0],
      });
      process.exit(1);
    }
    console.log("reuse safety: PASS (identical rows first / after-dud / solo)");
  }
  {
    // Early-exit factor on a dud (a lone slime cube — kick does nothing):
    // 300-tick evals with and without the tick-40 frozen shortcut.
    const dud = new Array(BX * BY * BZ).fill(0);
    dud[2] = 2; // one slime block at (2,0,0)
    const cells = [];
    const kicks = [];
    for (let i = 0; i < EVALS; i++) {
      cells.push(...dud);
      kicks.push(3, 0, 0);
    }
    for (const early of [false, true]) {
      const t0 = performance.now();
      JSON.parse(
        TickSimulation.evalFlightBatch(
          BX, BY, BZ, TRAVEL, X_OFF, PAL, cells, 0, kicks,
          300, 0n, -1, false, early,
        ),
      );
      const total = performance.now() - t0;
      console.log(
        `dud batch 300t early_exit=${early}: ${((1000 * EVALS) / total).toFixed(1)} evals/sec`,
      );
    }
  }
} else {
  console.log("fast-path benches: skipped (fromBlocks not in bindings yet)");
}
