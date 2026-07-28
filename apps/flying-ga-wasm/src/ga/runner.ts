/** Main-thread GA orchestrator: owns the worker pool, runs the generation
 * loop (elitism + tournament / uniform crossover / mutation — same constants
 * as the backend), and reports progress via callbacks.
 *
 * Two selection regimes:
 *  - scalar: weighted normalized objective sum (0..~10);
 *  - pareto: NSGA-II-lite (non-dominated rank + crowding tie-break) on the
 *    chosen objectives, with an uncapped cross-generation archive of
 *    non-dominated machines.
 *
 * Generations are uncapped: with `generations: null` the loop runs until
 * stop() is called. */

import type { EvalJob, EvalWorkerIn, EvalWorkerOut } from "../workers/evalWorker";
import type {
  BestRecord,
  EvolutionEvent,
  HistoryPoint,
  LeaderboardEntry,
  MutationSchedule,
  PopulationMember,
  RetiredEntry,
  RunConfig,
  SpeciesInfo,
} from "../types";
import {
  dirOf,
  effectiveConstraints,
  impliedGateBy,
  naturalDir,
  nsgaSelectionFitness,
  paretoFrontIdx,
  scalarFitness,
  zeroMetrics,
  OBJECTIVES,
  type Constraints,
  type EvalMetrics,
  type ObjectiveChoice,
} from "../metrics";
import { ALPHABET } from "./alphabet";
import {
  crossover,
  engineBGenome,
  genomeBlocks,
  genomeKey,
  minimalSeedGenome,
  mutate,
  randomGenome,
  type Genome,
} from "./genome";
import { Rng } from "./rng";
import { SpeciesTracker } from "./species";

const ELITISM = 2;
const TOURNAMENT_K = 3;
const LEADERBOARD_SIZE = 10;
const EVENTS_PER_GEN_CAP = 6;

/* ------------------------------------------- mutation-rate schedules ----
 * The effective mutation rate is recomputed at every generation boundary
 * from the active schedule, anchored at the generation the schedule was
 * (re)configured. A manual patch that changes the rate overrides the
 * schedule and pins the rate from that generation forward.
 *
 * Constants:
 * - RATE_MIN / RATE_MAX: hard clamp on the effective rate.
 * - linear: decays from the anchor rate to LINEAR_FLOOR_FRACTION × anchor
 *   over `spanGens` generations (default LINEAR_SPAN_DEFAULT), then holds.
 * - exponential: rate = anchor × 0.5^((gen − anchorGen)/halfLifeGens)
 *   (default half-life HALF_LIFE_DEFAULT gens), clamped at RATE_MIN.
 * - adaptive (1/5th-success rule): a generation "succeeds" when it sets a
 *   new run champion or adds a Pareto-archive point. Over the last
 *   ADAPT_WINDOW generations, if the success fraction is below
 *   ADAPT_TARGET (1/5) the rate is multiplied by ADAPT_UP (explore more);
 *   above it, by ADAPT_DOWN (exploit). Updated every generation once
 *   ADAPT_MIN_HISTORY generations of history exist. */
const RATE_MIN = 0.005;
const RATE_MAX = 0.5;
const LINEAR_SPAN_DEFAULT = 120;
const LINEAR_FLOOR_FRACTION = 0.1;
const HALF_LIFE_DEFAULT = 30;
const ADAPT_WINDOW = 10;
const ADAPT_TARGET = 0.2;
const ADAPT_UP = 1.15;
const ADAPT_DOWN = 0.87;
const ADAPT_MIN_HISTORY = 3;

const clampRate = (r: number): number =>
  Math.min(RATE_MAX, Math.max(RATE_MIN, r));

export function describeSchedule(s: MutationSchedule): string {
  switch (s.kind) {
    case "constant":
      return "constant";
    case "linear":
      return `linear decay over ${s.spanGens ?? LINEAR_SPAN_DEFAULT} gens`;
    case "exponential":
      return `exponential, half-life ${s.halfLifeGens ?? HALF_LIFE_DEFAULT} gens`;
    case "adaptive":
      return `adaptive (1/5th-success rule, window ${ADAPT_WINDOW})`;
  }
}

export interface GenerationUpdate {
  gen: number;
  point: HistoryPoint;
  evalsPerSec: number;
  leaderboard: LeaderboardEntry[];
  newBest: BestRecord | null;
  /** Non-dominated archive (pareto mode; empty in scalar mode). */
  archive: LeaderboardEntry[];
  /** This generation's valid, flying machines — the scatter cloud. */
  cloud: EvalMetrics[];
  /** New slowest sustained flier of the run (null when unbeaten). */
  slowpoke: LeaderboardEntry | null;
  /** Events emitted this generation (may be empty). */
  events: EvolutionEvent[];
  /** All archive entries shelved by mid-run reconfigurations so far. */
  retired: RetiredEntry[];
  /** Lineage census: this generation's per-species counts + the full
   * species registry snapshot. */
  species: { counts: Array<[number, number]>; info: SpeciesInfo[] };
  /** Effective mutation rate that breeds the NEXT generation. */
  mutationRate: number;
  /** The ENTIRE current population, for the inspector (live-only). */
  population: PopulationMember[];
}

/** The subset of a run's config that may change WHILE it is live. Fields
 * that define the genome space (bbox, alphabet, seeding, population, eval
 * window, seed) stay locked for the run's whole life. */
export interface LiveConfigPatch {
  objectives: ObjectiveChoice[];
  constraints: Constraints;
  targetPeriod: number | null;
  /** Base mutation rate; a mid-run change pins the rate (schedule override). */
  mutationRate: number;
  /** Active schedule; a mid-run change re-anchors it at the current gen. */
  mutationSchedule: MutationSchedule;
}

export interface RunnerCallbacks {
  onGeneration(u: GenerationUpdate): void;
  onDone(): void;
  onError(message: string): void;
  /** The loop parked at a generation boundary (full state retained). */
  onPause?(gen: number): void;
  /** The loop woke and will evaluate `gen` next. */
  onResume?(gen: number): void;
}

interface PoolWorker {
  w: Worker;
  busy: boolean;
}

export class GaRunner {
  private pool: PoolWorker[] = [];
  private stopFlag = false;
  private running = false;
  private pendingPatch: LiveConfigPatch | null = null;
  private pauseFlag = false;
  private wakeFns: Array<() => void> = [];

  get isRunning(): boolean {
    return this.running;
  }

  get isPaused(): boolean {
    return this.pauseFlag;
  }

  stop(): void {
    this.stopFlag = true;
    this.wake();
  }

  /** Park the loop at the next generation boundary; population, archive,
   * species and schedule state are all retained. No-op when idle. */
  pause(): void {
    if (this.running) this.pauseFlag = true;
  }

  /** Continue a paused run exactly where it stopped. */
  resume(): void {
    this.pauseFlag = false;
    this.wake();
  }

  private wake(): void {
    const fns = this.wakeFns;
    this.wakeFns = [];
    for (const f of fns) f();
  }

  /** Queue a mid-run reconfiguration; it is applied at the next generation
   * boundary (re-rank, archive re-prune, event). No-op when idle. */
  reconfigure(patch: LiveConfigPatch): void {
    if (this.running) this.pendingPatch = patch;
  }

  dispose(): void {
    this.stopFlag = true;
    this.wake();
    for (const p of this.pool) p.w.terminate();
    this.pool = [];
  }

  /** Create + warm the pool (each worker instantiates ~11 MB of wasm once). */
  private async initPool(n: number): Promise<void> {
    if (this.pool.length === n) return;
    this.dispose();
    this.stopFlag = false;
    const workers = Array.from({ length: n }, () => ({
      w: new Worker(new URL("../workers/evalWorker.ts", import.meta.url), {
        type: "module",
      }),
      busy: false,
    }));
    await Promise.all(
      workers.map(
        (p) =>
          new Promise<void>((resolve, reject) => {
            p.w.onmessage = ({ data }: MessageEvent<EvalWorkerOut>) => {
              if (data.type === "ready") resolve();
              else if (data.type === "error") reject(new Error(data.message));
            };
            p.w.postMessage({ type: "init" } satisfies EvalWorkerIn);
          }),
      ),
    );
    this.pool = workers;
  }

  /** Evaluate a whole population across the pool; small batches keep the
   * workers load-balanced when eval cost varies per genome. */
  private evaluateAll(
    pop: Genome[],
    cfg: RunConfig,
    live: LiveConfigPatch,
  ): Promise<EvalMetrics[]> {
    const needRobustness = live.objectives.some((o) => o.key === "robustness");
    const needPeriod =
      live.objectives.some((o) => o.key === "period") ||
      (live.constraints.periodBand && live.targetPeriod !== null);
    const constraints = effectiveConstraints(live.constraints, live.objectives);
    const out = new Array<EvalMetrics | null>(pop.length).fill(null);
    const batchSize = Math.max(
      1,
      Math.min(8, Math.ceil(pop.length / (this.pool.length * 3))),
    );
    const queue: EvalJob[][] = [];
    for (let i = 0; i < pop.length; i += batchSize)
      queue.push(
        pop.slice(i, i + batchSize).map((genome, j) => ({ i: i + j, genome })),
      );

    return new Promise((resolve, reject) => {
      let pending = 0;
      const pump = (p: PoolWorker) => {
        const jobs = queue.shift();
        if (!jobs) {
          if (pending === 0)
            resolve(out.map((m) => m ?? zeroMetrics("missing result")));
          return;
        }
        p.busy = true;
        pending++;
        p.w.onmessage = ({ data }: MessageEvent<EvalWorkerOut>) => {
          pending--;
          p.busy = false;
          if (data.type === "results") {
            for (const r of data.results) out[r.i] = r.m;
            pump(p);
          } else if (data.type === "error") {
            reject(new Error(data.message));
          }
        };
        p.w.postMessage({
          type: "eval",
          jobs,
          bbox: cfg.bbox,
          evalTicks: cfg.eval_ticks,
          seed: cfg.seed,
          constraints,
          needRobustness,
          needPeriod,
          targetPeriod: live.targetPeriod,
        } satisfies EvalWorkerIn);
      };
      for (const p of this.pool) pump(p);
    });
  }

  async start(cfg: RunConfig, cb: RunnerCallbacks): Promise<void> {
    if (this.running) return;
    this.running = true;
    this.stopFlag = false;
    this.pauseFlag = false;
    try {
      await this.initPool(cfg.workers);

      const rng = new Rng(cfg.seed);
      const names = new Map<string, string>();

      // Live regime: objectives / constraints / period target may be
      // reconfigured mid-run (applied at generation boundaries). Genome
      // space (bbox, alphabet, seeding, population, eval window) is fixed.
      const live: LiveConfigPatch = {
        objectives: cfg.objectives.map((o) => ({ ...o })),
        constraints: { ...cfg.constraints },
        targetPeriod: cfg.targetPeriod ?? null,
        mutationRate: cfg.mutation_rate,
        mutationSchedule: cfg.mutationSchedule ?? { kind: "constant" },
      };
      // Mutation-rate schedule state (see the constants block up top).
      const sched = {
        schedule: live.mutationSchedule,
        anchorGen: 0,
        anchorRate: clampRate(cfg.mutation_rate),
        /** Non-null once a mid-run slider move overrode the schedule. */
        manualRate: null as number | null,
        adaptiveRate: clampRate(cfg.mutation_rate),
        /** Per-gen success flags, newest last (≤ ADAPT_WINDOW kept). */
        successes: [] as boolean[],
      };
      let lastPatchRate = cfg.mutation_rate;
      let lastScheduleJson = JSON.stringify(sched.schedule);
      let mutRate = sched.anchorRate;
      const scheduledRate = (gen: number): number => {
        const s = sched.schedule;
        const dg = Math.max(0, gen - sched.anchorGen);
        switch (s.kind) {
          case "constant":
            return sched.anchorRate;
          case "linear": {
            const span = Math.max(1, s.spanGens ?? LINEAR_SPAN_DEFAULT);
            const floor = sched.anchorRate * LINEAR_FLOOR_FRACTION;
            const t = Math.min(1, dg / span);
            return sched.anchorRate + (floor - sched.anchorRate) * t;
          }
          case "exponential": {
            const hl = Math.max(1, s.halfLifeGens ?? HALF_LIFE_DEFAULT);
            return sched.anchorRate * Math.pow(0.5, dg / hl);
          }
          case "adaptive":
            return sched.adaptiveRate;
        }
      };
      const labelOf = () =>
        live.objectives
          .map(
            (c) =>
              `${dirOf(c) === naturalDir(c.key) ? "" : `${dirOf(c)} `}${c.key}`,
          )
          .join(" + ");

      let pop: Genome[] = [];
      const seeding = cfg.seeding ?? "engine-b";
      if (seeding === "engine-b") {
        for (const [mirror, name] of [
          [false, "engine-b"],
          [true, "engine-b-mirror"],
        ] as const) {
          const g = engineBGenome(cfg.bbox, mirror);
          if (g) {
            pop.push(g);
            names.set(genomeKey(g), name);
          }
        }
      } else if (seeding === "minimal") {
        const g = minimalSeedGenome(cfg.bbox);
        if (g) {
          pop.push(g);
          names.set(genomeKey(g), "seed-minimal");
        }
      }
      while (pop.length < cfg.population)
        pop.push(randomGenome(cfg.bbox, rng, live.constraints.maxBlocks));
      pop = pop.slice(0, cfg.population);

      const pareto = cfg.mode === "pareto";
      const tracker = new SpeciesTracker();
      let parentSpecies: Array<number | null> = pop.map(() => null);
      const retired: RetiredEntry[] = [];

      const mkEntry = (
        g: Genome,
        m: EvalMetrics,
        gen: number,
        score: number,
      ): LeaderboardEntry => {
        const key = genomeKey(g);
        return {
          id: `m${(hash32(key) % 1e8).toString().padStart(8, "0")}`,
          name: names.get(key),
          fitness: Math.round(m.fit * 100) / 100,
          gen,
          genome: g,
          speed: Math.round(m.speed * 1000) / 1000,
          score: Math.round(score * 100) / 100,
          metrics: m,
          blocks: genomeBlocks(g, cfg.bbox).map((c) => ({
            x: c.x,
            y: c.y,
            z: c.z,
            state: ALPHABET[c.s],
          })),
        };
      };

      const board = new Map<string, LeaderboardEntry>();
      const archive = new Map<string, LeaderboardEntry>();
      let bestEver = -Infinity;
      let slowestEver = Infinity;
      let emaEps = 0;

      for (let gen = 0; !this.stopFlag; gen++) {
        if (cfg.generations !== null && gen >= cfg.generations) break;

        // A real pause: park BETWEEN generations, full state retained.
        // Reconfigure patches queued while parked apply on wake, below.
        if (this.pauseFlag) {
          cb.onPause?.(gen - 1 < 0 ? 0 : gen - 1);
          while (this.pauseFlag && !this.stopFlag)
            await new Promise<void>((res) => this.wakeFns.push(res));
          if (this.stopFlag) break;
          cb.onResume?.(gen);
        }

        const events: EvolutionEvent[] = [];
        if (gen === 0) {
          events.push({
            gen,
            kind: "run",
            at: Date.now(),
            text: `run started — ${cfg.mode} mode on ${labelOf()}, ${seeding} seed, pop ${cfg.population}`,
          });
        }

        // Apply a queued mid-run reconfiguration at the generation boundary:
        // this generation evaluates + ranks under the NEW regime.
        if (this.pendingPatch) {
          const patch = this.pendingPatch;
          this.pendingPatch = null;
          const objChanged =
            JSON.stringify(patch.objectives) !== JSON.stringify(live.objectives) ||
            patch.targetPeriod !== live.targetPeriod;
          const conDiffs = constraintDiffs(live.constraints, patch.constraints);
          live.objectives = patch.objectives.map((o) => ({ ...o }));
          live.constraints = { ...patch.constraints };
          live.targetPeriod = patch.targetPeriod;
          if (objChanged) {
            const gates = impliedGateBy(live.objectives);
            events.push({
              gen,
              kind: "config",
              at: Date.now(),
              text: `objectives changed @ gen ${gen}: ${labelOf()}${
                gates.length ? ` (implied gate: must keep flying — ${gates.join(", ")})` : ""
              }`,
            });
          }
          if (conDiffs.length > 0)
            events.push({
              gen,
              kind: "config",
              at: Date.now(),
              text: `constraints changed @ gen ${gen}: ${conDiffs.join(", ")}`,
            });
          // Mutation regime: a rate change pins the rate (schedule
          // override); a schedule change re-anchors at this generation.
          if (Math.abs(patch.mutationRate - lastPatchRate) > 1e-12) {
            lastPatchRate = patch.mutationRate;
            sched.manualRate = clampRate(patch.mutationRate);
            events.push({
              gen,
              kind: "mutation",
              at: Date.now(),
              text:
                sched.schedule.kind === "constant"
                  ? `mutation rate set @ gen ${gen} → ${sched.manualRate.toFixed(3)}`
                  : `mutation schedule overridden @ gen ${gen} — rate pinned at ${sched.manualRate.toFixed(3)}`,
            });
          }
          const schedJson = JSON.stringify(patch.mutationSchedule);
          if (schedJson !== lastScheduleJson) {
            lastScheduleJson = schedJson;
            sched.schedule = { ...patch.mutationSchedule };
            live.mutationSchedule = sched.schedule;
            sched.anchorGen = gen;
            sched.anchorRate = sched.manualRate ?? mutRate;
            sched.adaptiveRate = sched.anchorRate;
            sched.manualRate = null;
            events.push({
              gen,
              kind: "mutation",
              at: Date.now(),
              text: `mutation schedule set @ gen ${gen}: ${describeSchedule(sched.schedule)} from ${sched.anchorRate.toFixed(3)}`,
            });
          }
          // Re-rank the leaderboard under the new regime (scalar scores are
          // recomputable from stored metrics; pareto ranks by speed).
          if (!pareto) {
            for (const e of board.values())
              if (e.metrics)
                e.score =
                  Math.round(
                    scalarFitness(
                      e.metrics,
                      live.objectives,
                      live.constraints.maxBlocks,
                    ) * 100,
                  ) / 100;
          }
          // Archive: entries invalid under the new constraints move to the
          // retired shelf (they don't vanish); the survivors are re-pruned
          // under the new dominance.
          const effC = effectiveConstraints(live.constraints, live.objectives);
          for (const [k, e] of [...archive]) {
            const reason = e.metrics
              ? storedViolation(e, effC, live.targetPeriod)
              : "no stored metrics";
            if (reason) {
              archive.delete(k);
              retired.push({ entry: e, reason, atGen: gen });
              if (events.length < EVENTS_PER_GEN_CAP)
                events.push({
                  gen,
                  kind: "retired",
                  at: Date.now(),
                  text: `retired from the front — ${e.name ?? e.id}: ${reason}`,
                });
            }
          }
          const survivors = [...archive.entries()].filter(([, e]) => e.metrics);
          const keep = new Set(
            paretoFrontIdx(
              survivors.map(([, e]) => e.metrics!),
              live.objectives,
            ),
          );
          survivors.forEach(([k], i) => {
            if (!keep.has(i)) archive.delete(k);
          });
          console.info(
            `[reconfig] gen ${gen}: ${labelOf()} — archive ${archive.size} kept, ${retired.length} retired total`,
          );
        }
        const objSel: ObjectiveChoice[] = live.objectives;

        const t0 = performance.now();
        const metrics = await this.evaluateAll(pop, cfg, live);
        const dt = Math.max((performance.now() - t0) / 1000, 1e-9);
        const eps = pop.length / dt;
        emaEps = emaEps === 0 ? eps : emaEps * 0.6 + eps * 0.4;

        // Species census (cheap: fingerprint hashing; registry scan only
        // for never-seen fingerprints).
        const speciesCounts = tracker.census(
          pop,
          cfg.bbox,
          gen,
          parentSpecies,
          (g) =>
            genomeBlocks(g, cfg.bbox).map((c) => ({
              x: c.x,
              y: c.y,
              z: c.z,
              state: ALPHABET[c.s],
            })),
        );

        // Selection fitness per regime.
        const scalars = metrics.map((m) =>
          scalarFitness(m, live.objectives, live.constraints.maxBlocks),
        );
        const selFits = pareto
          ? nsgaSelectionFitness(metrics, objSel)
          : scalars;

        // History stays in physical units: displacement fitness (blocks).
        const fits = metrics.map((m) => m.fit);
        const best = Math.max(...fits);
        const mean = fits.reduce((a, b) => a + b, 0) / fits.length;

        // Leaderboard: best score seen per distinct genome.
        pop.forEach((g, i) => {
          const m = metrics[i];
          const rankScore = pareto ? m.speed : scalars[i];
          if (m.violation || m.fit <= 0) return;
          const key = genomeKey(g);
          const prev = board.get(key);
          if (!prev || rankScore > (prev.score ?? -Infinity)) {
            const e = mkEntry(g, m, prev ? prev.gen : gen, rankScore);
            board.set(key, e);
          }
        });
        const top = [...board.values()]
          .sort((a, b) => (b.score ?? 0) - (a.score ?? 0))
          .slice(0, LEADERBOARD_SIZE);

        // Pareto archive: fold in this generation's fliers, prune dominated.
        if (pareto) {
          for (let i = 0; i < pop.length; i++) {
            const m = metrics[i];
            if (m.violation || !(m.fit > 0.5)) continue;
            const key = genomeKey(pop[i]);
            if (archive.has(key)) continue;
            const entries = [...archive.values()];
            if (entries.some((e) => e.metrics && dominatesM(e.metrics, m, objSel)))
              continue;
            // Dedupe exact objective-vector ties — one representative per
            // point keeps the front readable.
            if (
              entries.some(
                (e) =>
                  e.metrics &&
                  objSel.every(
                    (c) =>
                      Math.abs(
                        OBJECTIVES[c.key].value(e.metrics!) -
                          OBJECTIVES[c.key].value(m),
                      ) < 1e-9,
                  ),
              )
            )
              continue;
            // It joins: evict everything it dominates.
            for (const [k, e] of archive)
              if (e.metrics && dominatesM(m, e.metrics, objSel))
                archive.delete(k);
            const entry = mkEntry(pop[i], m, gen, m.speed);
            archive.set(key, entry);
            if (events.length < EVENTS_PER_GEN_CAP)
              events.push({
                gen,
                kind: "pareto",
                at: Date.now(),
                text: `new Pareto point — ${m.speed.toFixed(2)} blk/s at ${m.blocks} blocks`,
              });
          }
          // Safety net: keep the archive strictly non-dominated.
          const entries = [...archive.entries()].filter(([, e]) => e.metrics);
          const frontIdx = new Set(
            paretoFrontIdx(
              entries.map(([, e]) => e.metrics!),
              objSel,
            ),
          );
          entries.forEach(([k], i) => {
            if (!frontIdx.has(i)) archive.delete(k);
          });
        }

        // New slowpoke? Slowest sustained flier of the run — the machine
        // that still propels itself in the back half of the window at the
        // lowest blk/s. Feeds the Slowpoke plinth even when the run's
        // objectives never look at slowness.
        let slowpoke: LeaderboardEntry | null = null;
        {
          let si = -1;
          for (let i = 0; i < pop.length; i++) {
            const m = metrics[i];
            if (m.violation || !m.flies || !m.sustained) continue;
            if (m.speed > 1e-6 && m.speed < slowestEver - 1e-9) {
              slowestEver = m.speed;
              si = i;
            }
          }
          if (si >= 0) {
            slowpoke = mkEntry(pop[si], metrics[si], gen, scalars[si]);
            events.push({
              gen,
              kind: "slowpoke",
              at: Date.now(),
              text: `new slowpoke — sustained ${metrics[si].speed.toFixed(2)} blk/s on ${metrics[si].blocks} blocks (${metrics[si].lateDisp.toFixed(1)} of ${metrics[si].disp.toFixed(1)} blocks in the back half)`,
            });
          }
        }

        // New champion? (Physical displacement champion drives the stage.)
        let newBest: BestRecord | null = null;
        if (best > bestEver + 1e-9) {
          bestEver = best;
          const gi = fits.indexOf(best);
          newBest = {
            gen,
            fitness: Math.round(best * 100) / 100,
            genome: pop[gi],
            blocks: genomeBlocks(pop[gi], cfg.bbox).map((c) => ({
              x: c.x,
              y: c.y,
              z: c.z,
              state: ALPHABET[c.s],
            })),
          };
          events.push({
            gen,
            kind: "champion",
            at: Date.now(),
            text: `new champion — ${metrics[gi].speed.toFixed(2)} blk/s (${best.toFixed(1)} blocks), ${metrics[gi].blocks} block build`,
          });
        }

        // Mutation-rate schedule step. "Success" for the adaptive 1/5th
        // rule: this generation set a new run champion or grew the archive.
        {
          const grewArchive = events.some((e) => e.kind === "pareto");
          sched.successes.push(newBest !== null || grewArchive);
          if (sched.successes.length > ADAPT_WINDOW) sched.successes.shift();
          if (
            sched.schedule.kind === "adaptive" &&
            sched.manualRate === null &&
            sched.successes.length >= ADAPT_MIN_HISTORY
          ) {
            const frac =
              sched.successes.filter(Boolean).length / sched.successes.length;
            if (frac < ADAPT_TARGET) sched.adaptiveRate *= ADAPT_UP;
            else if (frac > ADAPT_TARGET) sched.adaptiveRate *= ADAPT_DOWN;
            sched.adaptiveRate = clampRate(sched.adaptiveRate);
          }
          mutRate = clampRate(sched.manualRate ?? scheduledRate(gen));
        }

        // The whole current population, for the inspector grid.
        const population: PopulationMember[] = pop.map((g, i) => {
          const m = metrics[i];
          return {
            id: `g${gen}-${i}`,
            key: genomeKey(g),
            genome: g,
            blocks: genomeBlocks(g, cfg.bbox).map((c) => ({
              x: c.x,
              y: c.y,
              z: c.z,
              state: ALPHABET[c.s],
            })),
            fitness: Math.round(m.fit * 100) / 100,
            speed: Math.round(m.speed * 1000) / 1000,
            speciesId: tracker.ids[i] ?? -1,
            violation: m.violation,
            metrics: m,
          };
        });

        cb.onGeneration({
          gen,
          point: {
            gen,
            best: Math.round(best * 100) / 100,
            mean: Math.round(mean * 100) / 100,
          },
          evalsPerSec: Math.round(emaEps),
          leaderboard: top,
          newBest,
          mutationRate: Math.round(mutRate * 1e4) / 1e4,
          population,
          archive: pareto
            ? [...archive.values()].sort((a, b) => (b.speed ?? 0) - (a.speed ?? 0))
            : [],
          cloud: metrics.filter((m) => !m.violation && m.fit > 0.5),
          slowpoke,
          events,
          retired: retired.slice(),
          species: { counts: speciesCounts, info: tracker.infoSnapshot() },
        });

        // Next generation: elitism + tournament/uniform-crossover/mutation.
        // Parent-A's species tags each child for lineage tracking.
        const cap = live.constraints.maxBlocks;
        const order = selFits
          .map((_, i) => i)
          .sort((a, b) => selFits[b] - selFits[a]);
        const nxt: Genome[] = [];
        const nextParents: Array<number | null> = [];
        for (const i of order.slice(0, ELITISM)) {
          nxt.push(pop[i]);
          nextParents.push(tracker.ids[i] ?? null);
        }
        while (nxt.length < cfg.population) {
          const ia = tourneyIdx(selFits, rng, TOURNAMENT_K);
          const ib = tourneyIdx(selFits, rng, TOURNAMENT_K);
          nxt.push(
            mutate(
              crossover(pop[ia], pop[ib], rng, cap),
              mutRate,
              rng,
              cfg.bbox,
              cap,
            ),
          );
          nextParents.push(tracker.ids[ia] ?? null);
        }
        pop = nxt;
        parentSpecies = nextParents;
      }
      cb.onDone();
    } catch (e) {
      cb.onError(e instanceof Error ? e.message : String(e));
    } finally {
      this.running = false;
    }
  }
}

/** Local alias so the archive fold reads naturally. Direction-aware: each
 * objective compares per its CHOSEN direction, not its natural one. */
function dominatesM(
  a: EvalMetrics,
  b: EvalMetrics,
  objectives: ObjectiveChoice[],
): boolean {
  let strict = false;
  for (const c of objectives) {
    const def = OBJECTIVES[c.key];
    const sign = dirOf(c) === "max" ? 1 : -1;
    const va = sign * def.value(a);
    const vb = sign * def.value(b);
    if (va < vb - 1e-9) return false;
    if (va > vb + 1e-9) strict = true;
  }
  return strict;
}

/** Human-readable diffs between two constraint sets (for the events feed). */
function constraintDiffs(prev: Constraints, next: Constraints): string[] {
  const out: string[] = [];
  if (prev.maxBlocks !== next.maxBlocks)
    out.push(`max blocks → ${next.maxBlocks}`);
  if (prev.minBlocks !== next.minBlocks)
    out.push(`min blocks → ${next.minBlocks}`);
  if (prev.pistonBudget !== next.pistonBudget)
    out.push(`piston cap → ${next.pistonBudget ?? "any"}`);
  if (prev.mustMoveByTick !== next.mustMoveByTick)
    out.push(`must move by → ${next.mustMoveByTick ?? "no deadline"}`);
  if (prev.requireSustained !== next.requireSustained)
    out.push(`must keep flying ${next.requireSustained ? "on" : "off"}`);
  if (prev.noStragglers !== next.noStragglers)
    out.push(`no stragglers ${next.noStragglers ? "on" : "off"}`);
  if (prev.periodBand !== next.periodBand)
    out.push(`period band ${next.periodBand ? "on" : "off"}`);
  if (prev.banned.join() !== next.banned.join())
    out.push(`banned: ${next.banned.join("+") || "none"}`);
  if (prev.required.join() !== next.required.join())
    out.push(`required: ${next.required.join("+") || "none"}`);
  return out;
}

/** Tournament by index (same RNG consumption as genome.ts `tournament`). */
function tourneyIdx(fits: number[], rng: Rng, k: number): number {
  let best = -1;
  for (const i of rng.sample(fits.length, k))
    if (best < 0 || fits[i] > fits[best]) best = i;
  return best;
}

/** Judge a stored archive entry against NEW constraints using only what we
 * kept: its metrics vector and its block list. Returns the violation text,
 * or null when it still qualifies. */
function storedViolation(
  e: LeaderboardEntry,
  c: Constraints,
  targetPeriod: number | null,
): string | null {
  const m = e.metrics!;
  const n = m.blocks || e.blocks.length;
  if (n > c.maxBlocks) return `over ${c.maxBlocks} blocks`;
  if (n < c.minBlocks) return `under ${c.minBlocks} blocks`;
  const kinds = e.blocks.map((b) =>
    b.state.replace(/^minecraft:/, "").split("[")[0],
  );
  if (c.pistonBudget !== null) {
    const pistons = kinds.filter(
      (k) => k === "sticky_piston" || k === "piston",
    ).length;
    if (pistons > c.pistonBudget)
      return `over piston budget (${c.pistonBudget})`;
  }
  for (const b of c.banned)
    if (kinds.includes(b)) return `banned kind: ${b}`;
  for (const r of c.required)
    if (!kinds.includes(r)) return `missing kind: ${r}`;
  if (c.noStragglers && !m.flies) return "shed debris — must arrive whole";
  if (c.requireSustained && !(m.flies && m.sustained))
    return m.flies
      ? "stalled in the back half"
      : "not a clean sustained flier";
  if (c.periodBand && targetPeriod !== null) {
    const p = m.period ?? 0;
    if (!(p > 0 && Math.abs(p - targetPeriod) <= 1))
      return p > 0
        ? `period ${p}t outside ${targetPeriod}±1t`
        : `no measured period (target ${targetPeriod}±1t)`;
  }
  return null;
}

function hash32(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}
