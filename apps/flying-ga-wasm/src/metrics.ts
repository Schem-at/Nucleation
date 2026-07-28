/** Multi-objective machinery: the machine metrics vector, objective
 * definitions in natural units, hard constraints, scalarised fitness,
 * Pareto dominance / non-dominated sorting (NSGA-II lite), and config
 * defaults for records stored before objectives existed.
 *
 * Speed is blocks/second at real game rate: the sim is not realtime, so
 * speed = displacement / (eval_ticks / 20). Engine B (1 block / 10 ticks)
 * is exactly 2.0 blk/s. */

export const GAME_TPS = 20;

/* ----------------------------------------------------------- constraints */

export interface Constraints {
  /** Hard cap on non-air blocks (also the GA's physical cap). */
  maxBlocks: number;
  /** Hard floor on non-air blocks; genomes below it are invalid (scored 0,
   * mirroring how the max cap is enforced). */
  minBlocks: number;
  /** Max pistons (sticky + normal), or null for no budget. */
  pistonBudget: number | null;
  /** Center of mass must have moved by this tick, or fitness 0. */
  mustMoveByTick: number | null;
  /** Must keep flying: second-half displacement ≥ 1 block AND ≥ 25 % of the
   * total, or the machine scores 0. Filters lurch-once machines out of
   * "slowest flier" runs; on by default whenever speed is minimized. */
  requireSustained: boolean;
  /** No stragglers: the left-behind estimate must be exactly 0 — the whole
   * machine arrives. Debris-shedders become invalid, not penalized. */
  noStragglers: boolean;
  /** Detected flight period must be within ±1 tick of targetPeriod, or the
   * machine (once it launches) scores 0. Needs a targetPeriod on the run. */
  periodBand: boolean;
  /** Block kinds that zero a genome when present. */
  banned: string[];
  /** Block kinds that zero a genome when absent. */
  required: string[];
}

/** Sustained-flight thresholds (see Constraints.requireSustained). */
export const SUSTAIN_MIN_LATE_DISP = 1.0;
export const SUSTAIN_MIN_LATE_FRACTION = 0.25;

/** True when a flight keeps propelling itself through the second half of
 * the eval window instead of lurching once and stalling. */
export function isSustained(lateDisp: number, disp: number): boolean {
  return (
    lateDisp >= SUSTAIN_MIN_LATE_DISP &&
    lateDisp >= SUSTAIN_MIN_LATE_FRACTION * Math.max(0, disp)
  );
}

export const DEFAULT_CONSTRAINTS: Constraints = {
  maxBlocks: 14,
  minBlocks: 1,
  pistonBudget: null,
  mustMoveByTick: null,
  requireSustained: false,
  noStragglers: false,
  periodBand: false,
  banned: [],
  required: [],
};

/* -------------------------------------------------------------- metrics */

/** Everything the evaluator measures about one genome, in natural units. */
export interface EvalMetrics {
  /** Constraint that zeroed this genome, or null. */
  violation: string | null;
  /** Legacy fitness: displacement minus debris penalty (blocks). */
  fit: number;
  /** Raw center-of-mass displacement (blocks). */
  disp: number;
  /** blocks/second at 20 ticks/s = disp / (evalTicks / 20). */
  speed: number;
  /** Displacement during the second half of the eval window (blocks). */
  lateDisp: number;
  /** Kept propelling itself: lateDisp ≥ 1 block and ≥ 25 % of disp. */
  sustained: boolean;
  /** Non-air block count. */
  blocks: number;
  /** Tight bounding-box volume of the build (cells). */
  volume: number;
  /** Inert payload blocks (note block / concrete) carried on a clean flight. */
  cargo: number;
  /** Flew (fit > 0.5) with no debris left behind. */
  flies: boolean;
  /** Fraction of distinct kick positions that fly; -1 = not measured. */
  robustness: number;
  /** Detected flight period in ticks (min-x rise-gap method on a sampled
   * window); 0 = none found or not measured. Measured only when a run asks
   * for it (target-period objective / period band constraint). */
  period: number;
  /** |detected period − target| in ticks; PERIOD_ERR_NONE when no period was
   * found, no target set, or the detector never ran. */
  periodErr: number;
}

/** Sentinel "worst possible" period error (no period / unmeasured). */
export const PERIOD_ERR_NONE = 999;

export function zeroMetrics(violation: string | null = null): EvalMetrics {
  return {
    violation,
    fit: 0,
    disp: 0,
    speed: 0,
    lateDisp: 0,
    sustained: false,
    blocks: 0,
    volume: 0,
    cargo: 0,
    flies: false,
    robustness: -1,
    period: 0,
    periodErr: PERIOD_ERR_NONE,
  };
}

/* ------------------------------------------------------------ objectives */

export type ObjectiveKey =
  | "speed"
  | "size"
  | "efficiency"
  | "compactness"
  | "cargo"
  | "robustness"
  | "period";

/** Per-objective optimization direction. Every objective has a natural
 * direction (speed: max, size: min); a run may flip it — "slowest engine
 * that still flies" is speed minimized. */
export type ObjectiveDir = "min" | "max";

export interface ObjectiveDef {
  key: ObjectiveKey;
  label: string;
  /** Natural unit shown on axes and stats. */
  unit: string;
  hint: string;
  /** Natural-unit value (axes, tooltips, hall of fame). */
  value(m: EvalMetrics): number;
  /** True when a larger natural value is better (size is the exception). */
  higherBetter: boolean;
  /** Normalized ~[0,1] maximization score in the NATURAL direction, for the
   * scalarised fitness (kept unclamped where the legacy behavior was). */
  score(m: EvalMetrics, maxBlocks: number): number;
  /** Ascending normalized value ~[0,1] (bigger natural value → bigger norm),
   * clamped before use — the basis for direction-inverted scores. */
  norm(m: EvalMetrics, maxBlocks: number): number;
  /** Gate for direction-inverted scores: without it, "minimize speed" would
   * hand a perfect score to machines that never fly at all. */
  eligible(m: EvalMetrics): boolean;
  fmt(v: number): string;
}

const f2 = (v: number) => v.toFixed(2);
const f0 = (v: number) => String(Math.round(v));

export const OBJECTIVES: Record<ObjectiveKey, ObjectiveDef> = {
  speed: {
    key: "speed",
    label: "speed",
    unit: "blk/s",
    hint: "center-of-mass velocity at 20 ticks/s",
    value: (m) => m.speed,
    higherBetter: true,
    score: (m) => m.speed / 2.5,
    norm: (m) => m.speed / 2.5,
    // Inverted (minimize) demands a CLEAN flight: a machine that sheds
    // debris dilutes its center-of-mass speed and would fake "slow".
    eligible: (m) => m.flies,
    fmt: f2,
  },
  size: {
    key: "size",
    label: "size",
    unit: "blocks",
    hint: "fewer blocks is better (fliers only)",
    value: (m) => m.blocks,
    higherBetter: false,
    score: (m, maxBlocks) =>
      m.flies ? (maxBlocks - m.blocks) / Math.max(1, maxBlocks) : 0,
    norm: (m, maxBlocks) => m.blocks / Math.max(1, maxBlocks),
    eligible: (m) => m.flies,
    fmt: f0,
  },
  efficiency: {
    key: "efficiency",
    label: "efficiency",
    unit: "blk/s per block",
    hint: "speed bought per block spent",
    value: (m) => (m.blocks > 0 ? m.speed / m.blocks : 0),
    higherBetter: true,
    score: (m) => (m.flies && m.blocks > 0 ? m.speed / m.blocks / 0.4 : 0),
    norm: (m) => (m.blocks > 0 ? m.speed / m.blocks / 0.4 : 0),
    eligible: (m) => m.flies && m.blocks > 0,
    fmt: (v) => v.toFixed(3),
  },
  compactness: {
    key: "compactness",
    label: "compactness",
    unit: "fill ratio",
    hint: "blocks / bounding volume — dense beats sprawling",
    value: (m) => (m.volume > 0 ? m.blocks / m.volume : 0),
    higherBetter: true,
    score: (m) => (m.flies && m.volume > 0 ? m.blocks / m.volume : 0),
    norm: (m) => (m.volume > 0 ? m.blocks / m.volume : 0),
    eligible: (m) => m.flies && m.volume > 0,
    fmt: f2,
  },
  cargo: {
    key: "cargo",
    label: "cargo",
    unit: "blocks",
    hint: "inert payload (note block / concrete) carried on a clean flight",
    value: (m) => m.cargo,
    higherBetter: true,
    score: (m) => (m.flies ? m.cargo / 6 : 0),
    norm: (m) => m.cargo / 6,
    eligible: (m) => m.flies,
    fmt: f0,
  },
  robustness: {
    key: "robustness",
    label: "robustness",
    unit: "kick tolerance",
    hint: "fraction of distinct kick positions it still flies from",
    value: (m) => Math.max(0, m.robustness),
    higherBetter: true,
    score: (m) => (m.flies ? Math.max(0, m.robustness) : 0),
    norm: (m) => Math.max(0, m.robustness),
    eligible: (m) => m.flies,
    fmt: f2,
  },
  period: {
    key: "period",
    label: "target period",
    unit: "ticks off",
    hint: "|detected gait period − target| — min-x rise-gap detector on a sampled window",
    value: (m) => m.periodErr ?? PERIOD_ERR_NONE,
    higherBetter: false,
    score: (m) =>
      m.flies && (m.period ?? 0) > 0
        ? Math.max(0, 1 - (m.periodErr ?? PERIOD_ERR_NONE) / 10)
        : 0,
    norm: (m) => Math.min(1, (m.periodErr ?? PERIOD_ERR_NONE) / 10),
    eligible: (m) => m.flies && (m.period ?? 0) > 0,
    fmt: f0,
  },
};

export const OBJECTIVE_ORDER: ObjectiveKey[] = [
  "speed",
  "size",
  "efficiency",
  "compactness",
  "cargo",
  "robustness",
  "period",
];

/** Objectives whose value is only meaningful on a machine that actually
 * FLIES (clean + sustained): cargo, robustness, target-period always;
 * speed when minimized ("slowest flier"). A run selecting any of these
 * implies the must-keep-flying + clean-flight gates — non-fliers become
 * invalid (culled), instead of scoring "great at cargo" while parked. */
export function impliedGateBy(objectives: ObjectiveChoice[]): string[] {
  const out: string[] = [];
  for (const c of objectives) {
    if (c.key === "cargo" || c.key === "robustness" || c.key === "period")
      out.push(OBJECTIVES[c.key].label);
    else if (c.key === "speed" && dirOf(c) === "min") out.push("min speed");
  }
  return out;
}

/** Constraints as actually enforced: the user's toggles plus the gates
 * implied by flight-dependent objectives. */
export function effectiveConstraints(
  constraints: Constraints,
  objectives: ObjectiveChoice[],
): Constraints {
  const implied = impliedGateBy(objectives).length > 0;
  return implied && !constraints.requireSustained
    ? { ...constraints, requireSustained: true }
    : constraints;
}

export interface ObjectiveChoice {
  key: ObjectiveKey;
  weight: number;
  /** Optimization direction; absent on records stored before directions
   * existed — falls back to the objective's natural direction. */
  dir?: ObjectiveDir;
}

/** The objective's built-in direction (speed: max, size: min, …). */
export function naturalDir(key: ObjectiveKey): ObjectiveDir {
  return OBJECTIVES[key].higherBetter ? "max" : "min";
}

/** Effective direction of a chosen objective. */
export function dirOf(c: ObjectiveChoice): ObjectiveDir {
  return c.dir ?? naturalDir(c.key);
}

export const DEFAULT_OBJECTIVES: ObjectiveChoice[] = [
  { key: "speed", weight: 1, dir: "max" },
];

export type OptimizeMode = "scalar" | "pareto";

/* ------------------------------------------------------- scalar fitness */

const clamp01 = (v: number) => Math.max(0, Math.min(1, v));

/** Direction-aware normalized ~[0,1] score for one chosen objective.
 * Natural direction keeps the legacy score untouched; an inverted
 * direction normalizes the value, clamps, then flips (1 − norm), gated so
 * "minimize speed" cannot reward machines that never fly. */
export function objectiveScore(
  c: ObjectiveChoice,
  m: EvalMetrics,
  maxBlocks: number,
): number {
  const def = OBJECTIVES[c.key];
  if (dirOf(c) === naturalDir(c.key)) return def.score(m, maxBlocks);
  if (!def.eligible(m)) return 0;
  const n = clamp01(def.norm(m, maxBlocks));
  return dirOf(c) === "max" ? n : 1 - n;
}

/** Weighted normalized sum, 0..~10. Violations are 0; non-fliers keep a
 * tiny displacement gradient so evolution can find its first flier. */
export function scalarFitness(
  m: EvalMetrics,
  objectives: ObjectiveChoice[],
  maxBlocks: number,
): number {
  if (m.violation) return 0;
  if (!m.flies && m.fit <= 0.5) return Math.max(0, m.disp) * 0.01;
  let sum = 0;
  let wTotal = 0;
  for (const c of objectives) {
    if (c.weight <= 0) continue;
    sum += c.weight * objectiveScore(c, m, maxBlocks);
    wTotal += c.weight;
  }
  return wTotal > 0 ? (10 * sum) / wTotal : 0;
}

/* ----------------------------------------------------------- Pareto core */

/** Directed value: larger always means better, per the CHOSEN direction. */
const directed = (m: EvalMetrics, c: ObjectiveChoice): number => {
  const v = OBJECTIVES[c.key].value(m);
  return dirOf(c) === "max" ? v : -v;
};

/** a dominates b on the chosen objectives: no worse everywhere, strictly
 * better somewhere — "better" per each objective's chosen direction. */
export function dominates(
  a: EvalMetrics,
  b: EvalMetrics,
  objectives: ObjectiveChoice[],
): boolean {
  let strict = false;
  for (const c of objectives) {
    const va = directed(a, c);
    const vb = directed(b, c);
    if (va < vb - 1e-9) return false;
    if (va > vb + 1e-9) strict = true;
  }
  return strict;
}

/** Indices of the non-dominated subset of `ms`. */
export function paretoFrontIdx(
  ms: EvalMetrics[],
  objectives: ObjectiveChoice[],
): number[] {
  const out: number[] = [];
  for (let i = 0; i < ms.length; i++) {
    let dominated = false;
    for (let j = 0; j < ms.length && !dominated; j++) {
      if (i !== j && dominates(ms[j], ms[i], objectives)) dominated = true;
    }
    if (!dominated) out.push(i);
  }
  return out;
}

/** NSGA-II-lite selection fitness: -(front rank) + crowding tie-break in
 * [0, 0.5]. Violations / grounded machines sit far below every front, with
 * a small displacement gradient. Higher is better. */
export function nsgaSelectionFitness(
  ms: EvalMetrics[],
  objectives: ObjectiveChoice[],
): number[] {
  const sel = new Array<number>(ms.length).fill(0);
  const eligible: number[] = [];
  ms.forEach((m, i) => {
    if (m.violation || (!m.flies && m.fit <= 0.5)) {
      sel[i] = -1000 + Math.max(0, m.disp) * 0.01;
    } else {
      eligible.push(i);
    }
  });

  // Non-dominated sort of the eligible set.
  let remaining = eligible.slice();
  let rank = 0;
  while (remaining.length > 0) {
    const subset = remaining.map((i) => ms[i]);
    const frontLocal = paretoFrontIdx(subset, objectives);
    const front = frontLocal.map((li) => remaining[li]);
    const inFront = new Set(front);

    // Crowding distance per objective, normalized to [0, 0.5] overall.
    const crowd = new Map<number, number>(front.map((i) => [i, 0]));
    for (const c of objectives) {
      const sorted = [...front].sort(
        (a, b) => directed(ms[a], c) - directed(ms[b], c),
      );
      const lo = directed(ms[sorted[0]], c);
      const hi = directed(ms[sorted[sorted.length - 1]], c);
      const span = hi - lo;
      crowd.set(sorted[0], Infinity);
      crowd.set(sorted[sorted.length - 1], Infinity);
      if (span > 1e-9) {
        for (let s = 1; s < sorted.length - 1; s++) {
          const gap =
            (directed(ms[sorted[s + 1]], c) - directed(ms[sorted[s - 1]], c)) /
            span;
          if (crowd.get(sorted[s]) !== Infinity)
            crowd.set(sorted[s], (crowd.get(sorted[s]) ?? 0) + gap);
        }
      }
    }
    for (const i of front) {
      const cd = crowd.get(i) ?? 0;
      const tie =
        cd === Infinity
          ? 0.5
          : Math.min(0.45, (cd / objectives.length) * 0.45);
      sel[i] = -rank + tie;
    }
    remaining = remaining.filter((i) => !inFront.has(i));
    rank++;
  }
  return sel;
}

/* ------------------------------------------------------ config defaults */

/** Fill objective/constraint fields absent from configs stored by older
 * versions of this app. */
export function withConfigDefaults<
  T extends {
    mode?: OptimizeMode;
    objectives?: ObjectiveChoice[];
    constraints?: Partial<Constraints>;
  },
>(cfg: T): T & {
  mode: OptimizeMode;
  objectives: ObjectiveChoice[];
  constraints: Constraints;
} {
  return {
    ...cfg,
    mode: cfg.mode ?? "scalar",
    objectives:
      cfg.objectives && cfg.objectives.length > 0
        ? cfg.objectives
        : DEFAULT_OBJECTIVES,
    constraints: { ...DEFAULT_CONSTRAINTS, ...(cfg.constraints ?? {}) },
  };
}

/** blk/s from a displacement in blocks over an eval window in ticks. */
export function speedOf(dispBlocks: number, evalTicks: number): number {
  return evalTicks > 0 ? dispBlocks / (evalTicks / GAME_TPS) : 0;
}
