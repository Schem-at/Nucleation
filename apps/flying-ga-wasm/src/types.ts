import type { CastMember } from "./cast";
import type { BBox, Genome } from "./ga/genome";
import type {
  Constraints,
  EvalMetrics,
  ObjectiveChoice,
  OptimizeMode,
} from "./metrics";

/** How generation 0 is seeded: from the 2-block minimal machine (grow
 * complexity from almost nothing), from the verified engine-b pair, or
 * purely at random. */
export type SeedMode = "minimal" | "engine-b" | "random";

/** Mutation-rate schedule: how the per-generation mutation rate evolves
 * over a run (annealing). All schedules anchor at the generation they were
 * (re)configured; a manual slider move mid-run overrides the schedule and
 * pins the rate from that generation forward. */
export type MutationScheduleKind =
  | "constant"
  | "linear"
  | "exponential"
  | "adaptive";

export interface MutationSchedule {
  kind: MutationScheduleKind;
  /** linear: generations to decay to 10 % of the anchor rate. */
  spanGens?: number;
  /** exponential: half-life in generations. */
  halfLifeGens?: number;
}

export interface RunConfig {
  population: number;
  /** Optional target — null means run forever (until Stop). */
  generations: number | null;
  mutation_rate: number;
  /** Gen-0 seeding strategy. Absent on records stored by older versions
   * (those ran with engine-b seeding). */
  seeding?: SeedMode;
  bbox: BBox;
  eval_ticks: number;
  seed: number;
  workers: number;
  /** Scalarised weighted sum vs Pareto (NSGA-II lite) selection. */
  mode: OptimizeMode;
  /** Enabled objectives (weights only matter in scalar mode). */
  objectives: ObjectiveChoice[];
  /** Hard filters — violating genomes score 0 on everything. */
  constraints: Constraints;
  /** Target flight period in ticks (target-period objective / ±1 band
   * constraint). Absent/null when no period target is set. */
  targetPeriod?: number | null;
  /** Mutation-rate schedule; absent on records stored by older versions
   * (those ran a constant rate). */
  mutationSchedule?: MutationSchedule;
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
  /** Legacy displacement fitness (blocks, debris-penalised). */
  fitness: number;
  gen: number;
  genome: Genome;
  blocks: Block[];
  /** blk/s at 20 ticks/s. Absent on records stored by older versions. */
  speed?: number;
  /** Scalarised score (0..~10) used for ranking in scalar mode. */
  score?: number;
  /** Full metrics vector. Absent on records stored by older versions. */
  metrics?: EvalMetrics;
}

export interface HistoryPoint {
  gen: number;
  /** Best displacement fitness (blocks) this generation. */
  best: number;
  /** Mean displacement fitness (blocks) this generation. */
  mean: number;
}

/** One line in the live evolution events feed. */
export interface EvolutionEvent {
  gen: number;
  kind:
    | "run"
    | "champion"
    | "pareto"
    | "slowpoke"
    | "config"
    | "retired"
    | "pause"
    | "mutation";
  text: string;
  at: number;
}

/** One member of the CURRENT population, for the inspector grid. Live-only
 * state — never persisted (a whole population per generation is too big). */
export interface PopulationMember {
  /** Unique per cell this generation (duplicated genomes stay distinct). */
  id: string;
  /** Structural fingerprint — thumbnail cache key; shared by identical genomes. */
  key: string;
  genome: Genome;
  blocks: Block[];
  /** Displacement fitness (blocks). */
  fitness: number;
  speed: number;
  speciesId: number;
  /** Constraint that invalidated this genome, or null when it qualified. */
  violation: string | null;
  metrics: EvalMetrics;
}

/** An archive entry pushed off the front by a mid-run reconfiguration:
 * invalid under the NEW constraints, shelved instead of vanishing. */
export interface RetiredEntry {
  entry: LeaderboardEntry;
  /** Why the new regime rejected it. */
  reason: string;
  /** Generation at which it was retired. */
  atGen: number;
}

/** One species' bookkeeping for the lineage view (structural fingerprint
 * clustering — see ga/species.ts). */
export interface SpeciesInfo {
  id: number;
  /** Short display label, e.g. "S7". */
  label: string;
  /** Machine dimensions (x·y·z) of the founding member. */
  dims: [number, number, number];
  /** Block-kind summary of the founder, e.g. "2×sticky_piston 3×slime". */
  summary: string;
  firstGen: number;
  lastGen: number;
  /** Species this one budded from (parent at first reproduction), if known. */
  parent: number | null;
  /** Representative machines across the species' lifetime (capped). */
  exemplars: Array<{ gen: number; blocks: Block[] }>;
  /** Peak share of the population in any one generation (0..1). */
  peakShare: number;
}

/** Per-generation species census: population count per species id. */
export interface SpeciesPoint {
  gen: number;
  counts: Array<[number, number]>;
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

export type RunStatus = "idle" | "starting" | "running" | "paused" | "done";

/** One point of the mutation-rate trajectory (the rate that bred the NEXT
 * generation after `gen`). */
export interface RatePoint {
  gen: number;
  rate: number;
}

export interface RunRecord {
  id: string;
  startedAt: number;
  stoppedAt?: number;
  config: RunConfig;
  history: HistoryPoint[];
  leaderboard: LeaderboardEntry[];
  bests: BestRecord[];
  generation: number;
  /** Pareto archive: non-dominated machines across all generations. */
  archive?: LeaderboardEntry[];
  /** Evolution events feed (newest last, capped). */
  events?: EvolutionEvent[];
  /** Archive entries shelved by mid-run reconfigurations. */
  retired?: RetiredEntry[];
  /** Lineage tracking: species registry + per-generation census. */
  species?: { info: SpeciesInfo[]; points: SpeciesPoint[] };
  /** Mutation-rate trajectory over the run (for the schedule sparkline). */
  rates?: RatePoint[];
}
