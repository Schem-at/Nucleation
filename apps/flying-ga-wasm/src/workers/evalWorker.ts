/// <reference lib="webworker" />
/** Evaluation worker: instantiates the ~11 MB wasm engine once, then
 * evaluates genome batches for the rest of its life. Returns the full
 * metrics vector per genome — the runner decides what "fitness" means. */

import type { BBox, Genome } from "../ga/genome";
import type { Constraints, EvalMetrics } from "../metrics";
import { loadEngine, type EngineModule } from "./engine";
import { evaluate, evaluateBatch } from "./evalCore";

export interface EvalJob {
  i: number;
  genome: Genome;
}

export type EvalWorkerIn =
  | { type: "init" }
  | {
      type: "eval";
      jobs: EvalJob[];
      bbox: BBox;
      evalTicks: number;
      seed: number;
      constraints: Constraints;
      needRobustness: boolean;
      needPeriod: boolean;
      targetPeriod: number | null;
    };

export type EvalWorkerOut =
  | { type: "ready" }
  | { type: "error"; message: string }
  | { type: "results"; results: Array<{ i: number; m: EvalMetrics }> };

let engine: EngineModule | null = null;

self.onmessage = async ({ data }: MessageEvent<EvalWorkerIn>) => {
  try {
    if (data.type === "init") {
      engine = await loadEngine();
      (self as unknown as Worker).postMessage({ type: "ready" } satisfies EvalWorkerOut);
      return;
    }
    if (data.type === "eval") {
      if (!engine) engine = await loadEngine();
      const {
        jobs,
        bbox,
        evalTicks,
        seed,
        constraints,
        needRobustness,
        needPeriod,
        targetPeriod,
      } = data;
      // Fast path: one engine call for the whole chunk. Robustness needs
      // per-genome alternate kicks, so it keeps the single-genome path.
      const results = needRobustness
        ? jobs.map((j) => ({
            i: j.i,
            m: evaluate(
              engine!,
              j.genome,
              bbox,
              evalTicks,
              seed,
              constraints,
              true,
              needPeriod,
              targetPeriod,
            ),
          }))
        : evaluateBatch(
            engine!,
            jobs.map((j) => j.genome),
            bbox,
            evalTicks,
            seed,
            constraints,
            needPeriod,
            targetPeriod,
          ).map((m, k) => ({ i: jobs[k].i, m }));
      (self as unknown as Worker).postMessage({
        type: "results",
        results,
      } satisfies EvalWorkerOut);
    }
  } catch (e) {
    (self as unknown as Worker).postMessage({
      type: "error",
      message: String(e),
    } satisfies EvalWorkerOut);
  }
};
