/** The evaluator — port of ga.py `_evaluate`, reformulated on the engine's
 * scalar queries so the hot loop does zero JSON, extended with hard
 * constraints and the full metrics vector for multi-objective runs.
 *
 * Legacy fitness = center-of-mass displacement along +x of all non-air
 * blocks, minus 0.3 per estimated "left behind" block. ga.py counts blocks
 * still below start_min_x + disp/2 from a JSON snapshot; with scalars only,
 * we estimate the same quantity from the center-of-mass shortfall: if the
 * leading edge travelled t = end_max_x - start_max_x but the center only
 * moved disp, then about n * (1 - disp / t) blocks stayed behind (k
 * stationary blocks drag the mean by exactly that factor). The debris gate
 * is the same: only when disp > 0.5, and only when the trailing edge
 * (non_air_min_x) actually lags start_min_x + disp / 2.
 *
 * Hard constraints (violating genomes score 0 on everything): max blocks,
 * piston budget, banned / required block kinds (pre-sim), and
 * must-move-by-tick-N (mid-sim). The robustness objective re-flies the
 * machine from up to two extra kick positions, so it is only measured when
 * a run asks for it. */

import { EXTRA_STATES, INERT_KINDS, KIND_OF_INDEX } from "../ga/alphabet";
import { genomeBlocks, type BBox, type Genome } from "../ga/genome";
import { genomeToSnbt, kickPositions, travelRoom } from "../ga/snbt";
import {
  isSustained,
  speedOf,
  zeroMetrics,
  PERIOD_ERR_NONE,
  type Constraints,
  type EvalMetrics,
} from "../metrics";
import { modalGap } from "../replay/period";
import type { EngineModule } from "./engine";

const ROBUSTNESS_KICKS = 3;
/** Sampled window (ticks) for the in-eval period detector. Cost: when the
 * target-period objective / period-band constraint is on, the LAST ≤120
 * ticks of the main flight are stepped one tick at a time with a nonAirMinX
 * scalar probe each tick (~120 extra wasm calls per eval, no JSON) instead
 * of one bulk run() — everything else, including the detector itself, is
 * arithmetic on those 120 ints. Detection is skipped entirely otherwise. */
const PERIOD_WINDOW = 120;

export function evaluate(
  eng: EngineModule,
  genome: Genome,
  bbox: BBox,
  evalTicks: number,
  seed: number,
  constraints: Constraints,
  needRobustness: boolean,
  needPeriod = false,
  targetPeriod: number | null = null,
): EvalMetrics {
  try {
    return evaluateInner(
      eng,
      genome,
      bbox,
      evalTicks,
      seed,
      constraints,
      needRobustness,
      needPeriod,
      targetPeriod,
    );
  } catch {
    return zeroMetrics("engine error");
  }
}

interface Flight {
  disp: number;
  /** Displacement during the second half of the eval window. */
  lateDisp: number;
  fit: number;
  /** No debris penalty applied — the whole machine kept up. */
  clean: boolean;
  /** Estimated blocks left behind (0 when clean). */
  leftBehind: number;
  /** Detected gait period in ticks (0 = none / not measured). */
  period: number;
  violation: string | null;
}

const NO_FLIGHT = (violation: string | null): Flight => ({
  disp: 0,
  lateDisp: 0,
  fit: 0,
  clean: false,
  leftBehind: 0,
  period: 0,
  violation,
});

/** One simulated flight from a given kick position. */
function fly(
  eng: EngineModule,
  snbt: string,
  kick: [number, number, number],
  evalTicks: number,
  seed: number,
  mustMoveByTick: number | null,
  needPeriod = false,
): Flight {
  const sim = eng.TickSimulation.fromSnbt(
    snbt,
    eng.TickSettleMode.Quiet,
    0,
    0,
    0,
    EXTRA_STATES,
  );
  sim.setRngSeed(BigInt(seed));

  const n0 = sim.nonAirCount();
  if (n0 === 0) return NO_FLIGHT("empty");
  const startComX = sim.nonAirCenterX();
  const startMinX = sim.nonAirMinX();
  const startMaxX = sim.nonAirMaxX();

  // Kick protocol (pinned by crates/mc-tick flying_machine test): quiet settle
  // happened at construction; redstone block beside the piston at tick 2,
  // removed at tick 4.
  const [kx, ky, kz] = kick;
  sim.run(2);
  sim.placeBlock(kx, ky, kz, "minecraft:redstone_block");
  sim.run(2);
  sim.placeBlock(kx, ky, kz, "minecraft:air");
  let elapsed = 4;

  // Scalar probes on the way to evalTicks, in tick order: the optional
  // must-move deadline, and always a mid-window center-of-mass sample so
  // the sustained-flight test can compare halves.
  const midTick = Math.min(evalTicks, Math.max(elapsed, Math.floor(evalTicks / 2)));
  const moveCheck =
    mustMoveByTick !== null
      ? Math.min(Math.max(mustMoveByTick, elapsed), evalTicks)
      : null;
  let comMid = startComX;
  const probes = [...new Set(moveCheck !== null ? [moveCheck, midTick] : [midTick])]
    .sort((a, b) => a - b);
  for (const t of probes) {
    if (t > elapsed) {
      sim.run(t - elapsed);
      elapsed = t;
    }
    if (t === moveCheck && sim.nonAirCenterX() - startComX < 0.25)
      return NO_FLIGHT(`did not move by tick ${mustMoveByTick}`);
    if (t === midTick) comMid = sim.nonAirCenterX();
  }

  // Target-period detection (only when the run asks): step the LAST
  // PERIOD_WINDOW ticks one at a time, sampling nonAirMinX each tick, then
  // read the modal gap between successive +1 rises of the trailing edge —
  // the same min-x rise-gap method the replay viewer uses.
  let period = 0;
  if (needPeriod) {
    const winStart = Math.max(elapsed, evalTicks - PERIOD_WINDOW);
    if (winStart > elapsed) {
      sim.run(winStart - elapsed);
      elapsed = winStart;
    }
    const gaps: number[] = [];
    let prevMinX = sim.nonAirMinX();
    let lastRise = -1;
    while (elapsed < evalTicks) {
      sim.run(1);
      elapsed++;
      const mx = sim.nonAirMinX();
      if (mx > prevMinX) {
        if (lastRise >= 0) gaps.push(elapsed - lastRise);
        lastRise = elapsed;
      }
      prevMinX = mx;
    }
    period = modalGap(gaps) ?? 0;
  }
  if (evalTicks > elapsed) sim.run(evalTicks - elapsed);

  const n1 = sim.nonAirCount();
  if (n1 === 0) return NO_FLIGHT(null);
  const endComX = sim.nonAirCenterX();
  const disp = endComX - startComX;
  const lateDisp = endComX - comMid;

  let penalty = 0.0;
  let leftBehind = 0;
  if (disp > 0.5 && sim.nonAirMinX() < startMinX + disp / 2) {
    const travel = sim.nonAirMaxX() - startMaxX;
    leftBehind =
      travel > 0.5 ? Math.round(Math.max(0, n1 * (1 - disp / travel))) : 0;
    penalty = 0.3 * leftBehind;
  }
  return {
    disp,
    lateDisp,
    fit: Math.max(0.0, disp - penalty),
    clean: penalty === 0,
    leftBehind,
    period,
    violation: null,
  };
}

function evaluateInner(
  eng: EngineModule,
  genome: Genome,
  bbox: BBox,
  evalTicks: number,
  seed: number,
  constraints: Constraints,
  needRobustness: boolean,
  needPeriod: boolean,
  targetPeriod: number | null,
): EvalMetrics {
  const cells = genomeBlocks(genome, bbox);
  const n = cells.length;
  if (n === 0) return zeroMetrics("empty genome");
  if (n > constraints.maxBlocks)
    return zeroMetrics(`over ${constraints.maxBlocks} blocks`);
  if (n < constraints.minBlocks)
    return zeroMetrics(`under ${constraints.minBlocks} blocks`);

  const kinds = cells.map((c) => KIND_OF_INDEX[c.s]);
  if (constraints.pistonBudget !== null) {
    const pistons = kinds.filter(
      (k) => k === "sticky_piston" || k === "piston",
    ).length;
    if (pistons > constraints.pistonBudget)
      return zeroMetrics(`over piston budget (${constraints.pistonBudget})`);
  }
  for (const b of constraints.banned)
    if (kinds.some((k) => k === b)) return zeroMetrics(`banned kind: ${b}`);
  for (const r of constraints.required)
    if (!kinds.some((k) => k === r)) return zeroMetrics(`missing kind: ${r}`);

  const kicks = kickPositions(
    genome,
    bbox,
    needRobustness ? ROBUSTNESS_KICKS : 1,
  );
  if (kicks.length === 0) return zeroMetrics("no piston to kick");

  // Geometry facts (pre-sim, from the genome itself).
  let minX = Infinity,
    maxX = -Infinity,
    minY = Infinity,
    maxY = -Infinity,
    minZ = Infinity,
    maxZ = -Infinity;
  for (const c of cells) {
    minX = Math.min(minX, c.x);
    maxX = Math.max(maxX, c.x);
    minY = Math.min(minY, c.y);
    maxY = Math.max(maxY, c.y);
    minZ = Math.min(minZ, c.z);
    maxZ = Math.max(maxZ, c.z);
  }
  const volume = (maxX - minX + 1) * (maxY - minY + 1) * (maxZ - minZ + 1);
  const inert = kinds.filter((k) => INERT_KINDS.has(k)).length;

  const snbt = genomeToSnbt(genome, bbox, travelRoom(evalTicks));
  const first = fly(
    eng,
    snbt,
    kicks[0],
    evalTicks,
    seed,
    constraints.mustMoveByTick,
    needPeriod,
  );
  if (first.violation) return zeroMetrics(first.violation);

  const flies = first.fit > 0.5 && first.clean;
  const sustained = isSustained(first.lateDisp, first.disp);
  const periodErr =
    targetPeriod !== null && first.period > 0
      ? Math.abs(first.period - targetPeriod)
      : PERIOD_ERR_NONE;
  // "No stragglers": the machine must arrive whole. Any left-behind estimate
  // makes it invalid (culled), not merely penalized.
  if (constraints.noStragglers && first.leftBehind > 0)
    return zeroMetrics(
      `left ${first.leftBehind} block${first.leftBehind === 1 ? "" : "s"} behind — must arrive whole`,
    );
  // Period band: once a machine launches, its gait must sit within ±1 tick
  // of the target (non-launchers keep their bootstrap gradient).
  if (
    constraints.periodBand &&
    targetPeriod !== null &&
    first.fit > 0.5 &&
    !(first.period > 0 && Math.abs(first.period - targetPeriod) <= 1)
  )
    return zeroMetrics(
      first.period > 0
        ? `period ${first.period}t outside ${targetPeriod}±1t`
        : `no stable period (target ${targetPeriod}±1t)`,
    );
  // "Must keep flying": the WHOLE machine must keep propelling itself.
  // Lurch-once-and-stall machines score 0 instead of qualifying as slow
  // fliers, and so do debris-shedders (a dropped tail dilutes the
  // center-of-mass speed into a fake "slow"). Machines that never launched
  // (fit ≤ 0.5) keep their tiny displacement gradient so evolution can
  // still bootstrap.
  if (constraints.requireSustained && first.fit > 0.5 && !(sustained && first.clean))
    return zeroMetrics(
      first.clean
        ? `stalled — ${first.lateDisp.toFixed(1)} blocks in the back half`
        : "shed debris — the whole machine must keep flying",
    );
  let robustness = -1;
  if (needRobustness) {
    let flew = first.fit > 0.5 ? 1 : 0;
    for (const kick of kicks.slice(1)) {
      const alt = fly(eng, snbt, kick, evalTicks, seed, null);
      if (alt.fit > 0.5) flew++;
    }
    robustness = flew / kicks.length;
  }

  return {
    violation: null,
    fit: first.fit,
    disp: first.disp,
    speed: speedOf(first.disp, evalTicks),
    lateDisp: first.lateDisp,
    sustained,
    blocks: n,
    volume,
    cargo: flies ? inert : 0,
    flies,
    robustness,
    period: first.period,
    periodErr,
  };
}
