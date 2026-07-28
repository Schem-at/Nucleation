/// <reference lib="webworker" />
/** Replay worker: re-simulates one genome with full change recording so the
 * main thread can rebuild per-tick frames and cut a seamless flight loop.
 * JSON is fine here — this runs once per new champion, not in the eval loop. */

import { EXTRA_STATES } from "../ga/alphabet";
import type { BBox, Genome } from "../ga/genome";
import { genomeToSnbt, kickPos } from "../ga/snbt";
import { loadEngine } from "./engine";

export interface ReplayRequest {
  type: "replay";
  token: number;
  genome: Genome;
  bbox: BBox;
  evalTicks: number;
  seed: number;
  /** Cap on recorded ticks (period detection needs far fewer than a full eval). */
  maxTicks: number;
}

export interface SnapshotBlock {
  pos: [number, number, number];
  state: string;
}

export interface ChangeRec {
  tick: number;
  pos: [number, number, number];
  from: string;
  to: string;
}

export type ReplayWorkerOut =
  | { type: "ready" }
  | { type: "error"; token?: number; message: string }
  | {
      type: "replay";
      token: number;
      start: SnapshotBlock[];
      changes: ChangeRec[];
      minXs: number[]; // non_air_min_x after each tick (1-based ticks)
      ticks: number;
    };

self.onmessage = async ({ data }: MessageEvent<ReplayRequest | { type: "init" }>) => {
  try {
    const eng = await loadEngine();
    if (data.type === "init") {
      (self as unknown as Worker).postMessage({ type: "ready" } satisfies ReplayWorkerOut);
      return;
    }
    const { token, genome, bbox, evalTicks, seed, maxTicks } = data;
    const kick = kickPos(genome, bbox);
    const sim = eng.TickSimulation.fromSnbt(
      genomeToSnbt(genome, bbox),
      eng.TickSettleMode.Quiet,
      0,
      0,
      0,
      EXTRA_STATES,
    );
    sim.setRngSeed(BigInt(seed));

    const start = JSON.parse(sim.worldSnapshotJson()) as SnapshotBlock[];

    const ticks = Math.min(evalTicks, maxTicks);
    const minXs: number[] = [];
    // Same protocol as the evaluator, single-stepped: after 2 ticks place the
    // redstone block beside the piston, after 4 ticks remove it.
    for (let t = 1; t <= ticks; t++) {
      sim.step();
      if (kick) {
        if (t === 2) sim.placeBlock(kick[0], kick[1], kick[2], "minecraft:redstone_block");
        if (t === 4) sim.placeBlock(kick[0], kick[1], kick[2], "minecraft:air");
      }
      minXs.push(sim.nonAirMinX());
    }

    const changes = JSON.parse(sim.changesJson()) as ChangeRec[];
    (self as unknown as Worker).postMessage({
      type: "replay",
      token,
      start,
      changes,
      minXs,
      ticks,
    } satisfies ReplayWorkerOut);
  } catch (e) {
    (self as unknown as Worker).postMessage({
      type: "error",
      token: (data as ReplayRequest).token,
      message: String(e),
    } satisfies ReplayWorkerOut);
  }
};
