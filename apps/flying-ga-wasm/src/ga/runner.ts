/** Main-thread GA orchestrator: owns the worker pool, runs the generation
 * loop (elitism + tournament / uniform crossover / mutation — same constants
 * as the backend), and reports progress via callbacks.
 *
 * Generations are uncapped: with `generations: null` the loop runs until
 * stop() is called. */

import type { EvalJob, EvalWorkerIn, EvalWorkerOut } from "../workers/evalWorker";
import type {
  BestRecord,
  HistoryPoint,
  LeaderboardEntry,
  RunConfig,
} from "../types";
import { ALPHABET } from "./alphabet";
import {
  crossover,
  engineBGenome,
  genomeBlocks,
  genomeKey,
  mutate,
  randomGenome,
  tournament,
  type Genome,
} from "./genome";
import { Rng } from "./rng";

const ELITISM = 2;
const TOURNAMENT_K = 3;
const LEADERBOARD_SIZE = 10;

export interface GenerationUpdate {
  gen: number;
  point: HistoryPoint;
  evalsPerSec: number;
  leaderboard: LeaderboardEntry[];
  newBest: BestRecord | null;
}

export interface RunnerCallbacks {
  onGeneration(u: GenerationUpdate): void;
  onDone(): void;
  onError(message: string): void;
}

interface PoolWorker {
  w: Worker;
  busy: boolean;
}

export class GaRunner {
  private pool: PoolWorker[] = [];
  private stopFlag = false;
  private running = false;

  get isRunning(): boolean {
    return this.running;
  }

  stop(): void {
    this.stopFlag = true;
  }

  dispose(): void {
    this.stopFlag = true;
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
  ): Promise<number[]> {
    const fits = new Array<number>(pop.length).fill(0);
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
          if (pending === 0) resolve(fits);
          return;
        }
        p.busy = true;
        pending++;
        p.w.onmessage = ({ data }: MessageEvent<EvalWorkerOut>) => {
          pending--;
          p.busy = false;
          if (data.type === "results") {
            for (const r of data.results) fits[r.i] = r.fit;
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
        } satisfies EvalWorkerIn);
      };
      for (const p of this.pool) pump(p);
    });
  }

  async start(cfg: RunConfig, cb: RunnerCallbacks): Promise<void> {
    if (this.running) return;
    this.running = true;
    this.stopFlag = false;
    try {
      await this.initPool(cfg.workers);

      const rng = new Rng(cfg.seed);
      const names = new Map<string, string>();
      let pop: Genome[] = [];
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
      while (pop.length < cfg.population) pop.push(randomGenome(cfg.bbox, rng));
      pop = pop.slice(0, cfg.population);

      const board = new Map<string, LeaderboardEntry>();
      let bestEver = -Infinity;
      let emaEps = 0;

      for (let gen = 0; !this.stopFlag; gen++) {
        if (cfg.generations !== null && gen >= cfg.generations) break;

        const t0 = performance.now();
        const fits = await this.evaluateAll(pop, cfg);
        const dt = Math.max((performance.now() - t0) / 1000, 1e-9);
        const eps = pop.length / dt;
        emaEps = emaEps === 0 ? eps : emaEps * 0.6 + eps * 0.4;

        const best = Math.max(...fits);
        const mean = fits.reduce((a, b) => a + b, 0) / fits.length;

        // Leaderboard: best fitness seen per distinct genome.
        pop.forEach((g, i) => {
          const f = fits[i];
          if (f <= 0) return;
          const key = genomeKey(g);
          const prev = board.get(key);
          if (!prev || f > prev.fitness) {
            board.set(key, {
              id: `m${(hash32(key) % 1e8).toString().padStart(8, "0")}`,
              name: names.get(key),
              fitness: Math.round(f * 100) / 100,
              gen: prev ? prev.gen : gen,
              genome: g,
              blocks: genomeBlocks(g, cfg.bbox).map((c) => ({
                x: c.x,
                y: c.y,
                z: c.z,
                state: ALPHABET[c.s],
              })),
            });
          }
        });
        const top = [...board.values()]
          .sort((a, b) => b.fitness - a.fitness)
          .slice(0, LEADERBOARD_SIZE);

        // New champion?
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
        }

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
        });

        // Next generation: elitism + tournament/uniform-crossover/mutation.
        const order = fits
          .map((_, i) => i)
          .sort((a, b) => fits[b] - fits[a]);
        const nxt: Genome[] = order.slice(0, ELITISM).map((i) => pop[i]);
        while (nxt.length < cfg.population) {
          const a = tournament(pop, fits, rng, TOURNAMENT_K);
          const b = tournament(pop, fits, rng, TOURNAMENT_K);
          nxt.push(mutate(crossover(a, b, rng), cfg.mutation_rate, rng));
        }
        pop = nxt;
      }
      cb.onDone();
    } catch (e) {
      cb.onError(e instanceof Error ? e.message : String(e));
    } finally {
      this.running = false;
    }
  }
}

function hash32(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}
