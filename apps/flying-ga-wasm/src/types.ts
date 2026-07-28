import type { CastMember } from "./cast";
import type { BBox, Genome } from "./ga/genome";

export interface RunConfig {
  population: number;
  /** Optional target — null means run forever (until Stop). */
  generations: number | null;
  mutation_rate: number;
  bbox: BBox;
  eval_ticks: number;
  seed: number;
  workers: number;
}

export interface Block {
  x: number;
  y: number;
  z: number;
  state: string;
}

export interface LeaderboardEntry {
  id: string;
  name?: string;
  fitness: number;
  gen: number;
  genome: Genome;
  blocks: Block[];
}

export interface HistoryPoint {
  gen: number;
  best: number;
  mean: number;
}

/** One per-tick frame of a rebuilt replay (already loop-windowed). */
export interface LoopFrame {
  blocks: Block[];
}

/** A seamless one-period flight loop, ready to render. */
export interface FlightLoopData {
  frames: LoopFrame[];
  /** Blocks the machine advances in +x over one period. */
  dx: number;
  /** Ticks per period. */
  period: number;
  /** How the period was found: "min-x cadence" | "autocorrelation" | "static". */
  method: string;
  /** Anchor: nonAirMinX at the loop's first frame (for compensation). */
  anchorX: number;
  /** Member cast over the loop window (local ticks 0..period): carried
   * blocks with interpolated motion, so pistons animate instead of
   * morphing through moving_piston cubes. Absent on old stored records. */
  cast?: CastMember[] | null;
}

/** A generation champion for the filmstrip. */
export interface BestRecord {
  gen: number;
  fitness: number;
  genome: Genome;
  blocks: Block[];
  loop?: FlightLoopData | null;
}

export type RunStatus = "idle" | "starting" | "running" | "done";

export interface RunRecord {
  id: string;
  startedAt: number;
  stoppedAt?: number;
  config: RunConfig;
  history: HistoryPoint[];
  leaderboard: LeaderboardEntry[];
  bests: BestRecord[];
  generation: number;
}
