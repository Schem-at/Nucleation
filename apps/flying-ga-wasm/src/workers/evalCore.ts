/** The evaluator — port of ga.py `_evaluate`, reformulated on the engine's
 * batch flight API so the hot loop crosses the wasm boundary once per
 * generation chunk instead of a dozen times per machine, and builds zero
 * SNBT text (genomes go over as flat palette-index arrays).
 *
 * The engine's `evalFlightBatch` replicates the flight protocol exactly
 * (quiet settle, redstone kick at t2 removed at t4, the same probe schedule
 * and gait detector) and returns raw scalars; every DECISION — violations,
 * fitness, penalties, constraint gates — stays here, bit-identical to the
 * previous per-call implementation.
 *
 * Legacy fitness = center-of-mass displacement along +x of all non-air
 * blocks, minus 0.3 per estimated "left behind" block: if the leading edge
 * travelled t but the center only moved disp, about n * (1 - disp / t)
 * blocks stayed behind. Debris gate: only when disp > 0.5 and the trailing
 * edge actually lags start_min_x + disp / 2.
 *
 * Hard constraints (violating genomes score 0 on everything): max/min
 * blocks, piston budget, banned / required kinds (pre-sim), and
 * must-move-by-tick-N (in-sim). Robustness re-flies the machine from up to
 * two extra kick positions, only when a run asks for it.
 *
 * The engine also early-exits machines that are provably frozen (quiescent
 * and unmoved at tick 40): every later scalar equals the tick-40 scalar, so
 * results are identical and only wall time changes. */

import { AIR, ALPHABET, INERT_KINDS, KIND_OF_INDEX } from "../ga/alphabet";
import { genomeBlocks, type BBox, type Cell, type Genome } from "../ga/genome";
import { kickPositions, travelRoom, X_OFF } from "../ga/snbt";
import {
  isSustained,
  speedOf,
  zeroMetrics,
  PERIOD_ERR_NONE,
  type Constraints,
  type EvalMetrics,
} from "../metrics";
import type { EngineModule } from "./engine";

const ROBUSTNESS_KICKS = 3;
/** The full alphabet, pre-interned engine-side per call — the same binding
 * surface the SNBT path provided via EXTRA_STATES. */
const PALETTE = ALPHABET.join(";");

/** Raw per-flight scalars from the engine, in evalFlightBatch row order. */
type FlightRow = [
  number, // n0
  number, // start com x
  number, // start min x
  number, // start max x
  number | null, // com at the must-move deadline (null = no deadline)
  number, // com at mid-window
  number, // detected period (0 = none / not measured)
  number, // n1
  number | null, // end com x (null = world emptied)
  number, // end min x
  number, // end max x
];

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

/** The post-sim flight math, identical to the old per-call `fly` tail. */
function flightFromRow(row: FlightRow, mustMoveByTick: number | null): Flight {
  const [n0, startComX, startMinX, startMaxX, comMove, , period, n1, endComX, endMinX, endMaxX] =
    row;
  const comMid = row[5];
  if (n0 === 0) return NO_FLIGHT("empty");
  if (
    mustMoveByTick !== null &&
    comMove !== null &&
    comMove - startComX < 0.25
  )
    return NO_FLIGHT(`did not move by tick ${mustMoveByTick}`);
  if (n1 === 0 || endComX === null) return NO_FLIGHT(null);

  const disp = endComX - startComX;
  const lateDisp = endComX - comMid;
  let penalty = 0.0;
  let leftBehind = 0;
  if (disp > 0.5 && endMinX < startMinX + disp / 2) {
    const travel = endMaxX - startMaxX;
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

/** One engine call flying `genomes[i]` from `kicks[i]`. */
function flyBatch(
  eng: EngineModule,
  genomes: Genome[],
  kicks: Array<[number, number, number]>,
  bbox: BBox,
  evalTicks: number,
  seed: number,
  mustMoveByTick: number | null,
  needPeriod: boolean,
): FlightRow[] {
  const [bx, by, bz] = bbox;
  const cells: number[] = [];
  for (const g of genomes) for (const s of g) cells.push(s);
  const flatKicks: number[] = [];
  for (const [x, y, z] of kicks) flatKicks.push(x, y, z);
  const json = eng.TickSimulation.evalFlightBatch(
    bx,
    by,
    bz,
    travelRoom(evalTicks),
    X_OFF,
    PALETTE,
    cells,
    AIR,
    flatKicks,
    evalTicks,
    BigInt(seed),
    mustMoveByTick ?? -1,
    needPeriod,
    true,
  );
  return JSON.parse(json) as FlightRow[];
}

interface Precheck {
  violation: string | null;
  kicks: Array<[number, number, number]>;
  n: number;
  volume: number;
  inert: number;
}

/** Pre-sim constraint gates + geometry facts, straight from the genome. */
function precheck(
  genome: Genome,
  bbox: BBox,
  constraints: Constraints,
  needRobustness: boolean,
): Precheck {
  const fail = (violation: string): Precheck => ({
    violation,
    kicks: [],
    n: 0,
    volume: 0,
    inert: 0,
  });
  const cells: Cell[] = genomeBlocks(genome, bbox);
  const n = cells.length;
  if (n === 0) return fail("empty genome");
  if (n > constraints.maxBlocks) return fail(`over ${constraints.maxBlocks} blocks`);
  if (n < constraints.minBlocks) return fail(`under ${constraints.minBlocks} blocks`);

  const kinds = cells.map((c) => KIND_OF_INDEX[c.s]);
  if (constraints.pistonBudget !== null) {
    const pistons = kinds.filter(
      (k) => k === "sticky_piston" || k === "piston",
    ).length;
    if (pistons > constraints.pistonBudget)
      return fail(`over piston budget (${constraints.pistonBudget})`);
  }
  for (const b of constraints.banned)
    if (kinds.some((k) => k === b)) return fail(`banned kind: ${b}`);
  for (const r of constraints.required)
    if (!kinds.some((k) => k === r)) return fail(`missing kind: ${r}`);

  const kicks = kickPositions(genome, bbox, needRobustness ? ROBUSTNESS_KICKS : 1);
  if (kicks.length === 0) return fail("no piston to kick");

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
  return { violation: null, kicks, n, volume, inert };
}

/** The post-flight constraint gates + metrics assembly (unchanged logic). */
function metricsFromFlight(
  first: Flight,
  pre: Precheck,
  evalTicks: number,
  constraints: Constraints,
  targetPeriod: number | null,
  robustness: number,
): EvalMetrics {
  if (first.violation) return zeroMetrics(first.violation);

  const flies = first.fit > 0.5 && first.clean;
  const sustained = isSustained(first.lateDisp, first.disp);
  const periodErr =
    targetPeriod !== null && first.period > 0
      ? Math.abs(first.period - targetPeriod)
      : PERIOD_ERR_NONE;
  if (constraints.noStragglers && first.leftBehind > 0)
    return zeroMetrics(
      `left ${first.leftBehind} block${first.leftBehind === 1 ? "" : "s"} behind — must arrive whole`,
    );
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
  if (constraints.requireSustained && first.fit > 0.5 && !(sustained && first.clean))
    return zeroMetrics(
      first.clean
        ? `stalled — ${first.lateDisp.toFixed(1)} blocks in the back half`
        : "shed debris — the whole machine must keep flying",
    );

  return {
    violation: null,
    fit: first.fit,
    disp: first.disp,
    speed: speedOf(first.disp, evalTicks),
    lateDisp: first.lateDisp,
    sustained,
    blocks: pre.n,
    volume: pre.volume,
    cargo: flies ? pre.inert : 0,
    flies,
    robustness,
    period: first.period,
    periodErr,
  };
}

/** Batch evaluation — the pool's fast path (no robustness re-flights). */
export function evaluateBatch(
  eng: EngineModule,
  genomes: Genome[],
  bbox: BBox,
  evalTicks: number,
  seed: number,
  constraints: Constraints,
  needPeriod: boolean,
  targetPeriod: number | null,
): EvalMetrics[] {
  const out: EvalMetrics[] = new Array(genomes.length);
  const survivors: number[] = [];
  const pres: Precheck[] = new Array(genomes.length);
  for (let i = 0; i < genomes.length; i++) {
    const pre = precheck(genomes[i], bbox, constraints, false);
    pres[i] = pre;
    if (pre.violation) out[i] = zeroMetrics(pre.violation);
    else survivors.push(i);
  }
  if (survivors.length === 0) return out;
  try {
    const rows = flyBatch(
      eng,
      survivors.map((i) => genomes[i]),
      survivors.map((i) => pres[i].kicks[0]),
      bbox,
      evalTicks,
      seed,
      constraints.mustMoveByTick,
      needPeriod,
    );
    survivors.forEach((i, k) => {
      const first = flightFromRow(rows[k], constraints.mustMoveByTick);
      out[i] = metricsFromFlight(
        first,
        pres[i],
        evalTicks,
        constraints,
        targetPeriod,
        -1,
      );
    });
  } catch {
    for (const i of survivors) out[i] = zeroMetrics("engine error");
  }
  return out;
}

/** Full single-genome evaluation, including robustness re-flights. */
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
    const pre = precheck(genome, bbox, constraints, needRobustness);
    if (pre.violation) return zeroMetrics(pre.violation);
    const [row] = flyBatch(
      eng,
      [genome],
      [pre.kicks[0]],
      bbox,
      evalTicks,
      seed,
      constraints.mustMoveByTick,
      needPeriod,
    );
    const first = flightFromRow(row, constraints.mustMoveByTick);
    let robustness = -1;
    if (needRobustness && !first.violation) {
      let flew = first.fit > 0.5 ? 1 : 0;
      const alts = pre.kicks.slice(1);
      if (alts.length > 0) {
        const rows = flyBatch(
          eng,
          alts.map(() => genome),
          alts,
          bbox,
          evalTicks,
          seed,
          null,
          false,
        );
        for (const r of rows) if (flightFromRow(r, null).fit > 0.5) flew++;
      }
      robustness = flew / pre.kicks.length;
    }
    return metricsFromFlight(
      first,
      pre,
      evalTicks,
      constraints,
      targetPeriod,
      robustness,
    );
  } catch {
    return zeroMetrics("engine error");
  }
}
