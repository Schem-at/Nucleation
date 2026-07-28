/** The evaluator — port of ga.py `_evaluate`, reformulated on the engine's
 * scalar queries so the hot loop does zero JSON.
 *
 * Fitness = center-of-mass displacement along +x of all non-air blocks
 * (moving_piston / piston_head count), minus 0.3 per estimated "left behind"
 * block. ga.py counts blocks still below start_min_x + disp/2 from a JSON
 * snapshot; with scalars only, we estimate the same quantity from the
 * center-of-mass shortfall: if the leading edge travelled t = end_max_x -
 * start_max_x but the center only moved disp, then about n * (1 - disp / t)
 * blocks stayed behind (k stationary blocks drag the mean by exactly that
 * factor). The debris gate is the same: only when disp > 0.5, and only when
 * the trailing edge (non_air_min_x) actually lags start_min_x + disp / 2.
 */

import { EXTRA_STATES } from "../ga/alphabet";
import { blockCount, type BBox, type Genome } from "../ga/genome";
import { genomeToSnbt, kickPos } from "../ga/snbt";
import type { EngineModule } from "./engine";

export function evaluate(
  eng: EngineModule,
  genome: Genome,
  bbox: BBox,
  evalTicks: number,
  seed: number,
): number {
  try {
    return evaluateInner(eng, genome, bbox, evalTicks, seed);
  } catch {
    return 0.0;
  }
}

function evaluateInner(
  eng: EngineModule,
  genome: Genome,
  bbox: BBox,
  evalTicks: number,
  seed: number,
): number {
  if (blockCount(genome) === 0) return 0.0;
  const kick = kickPos(genome, bbox);
  if (kick === null) return 0.0; // no piston, nothing to kick

  const sim = eng.TickSimulation.fromSnbt(
    genomeToSnbt(genome, bbox),
    eng.TickSettleMode.Quiet,
    0,
    0,
    0,
    EXTRA_STATES,
  );
  sim.setRngSeed(BigInt(seed));

  const n0 = sim.nonAirCount();
  if (n0 === 0) return 0.0;
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
  if (evalTicks > 4) sim.run(evalTicks - 4);

  const n1 = sim.nonAirCount();
  if (n1 === 0) return 0.0;
  const disp = sim.nonAirCenterX() - startComX;

  let penalty = 0.0;
  if (disp > 0.5 && sim.nonAirMinX() < startMinX + disp / 2) {
    const travel = sim.nonAirMaxX() - startMaxX;
    const leftBehind =
      travel > 0.5 ? Math.round(Math.max(0, n1 * (1 - disp / travel))) : 0;
    penalty = 0.3 * leftBehind;
  }
  return Math.max(0.0, disp - penalty);
}
