/** Main-thread facade over the dedicated replay worker. One worker, one
 * request at a time (champions arrive far slower than it simulates). */

import type { BBox, Genome } from "../ga/genome";
import type { FlightLoopData } from "../types";
import type { ReplayRequest, ReplayWorkerOut } from "../workers/replayWorker";
import { assembleLoop } from "./period";

const MAX_REPLAY_TICKS = 400;

export class ReplayClient {
  private worker: Worker | null = null;
  private token = 0;
  private pending = new Map<
    number,
    { resolve: (l: FlightLoopData) => void; reject: (e: Error) => void }
  >();

  private ensure(): Worker {
    if (this.worker) return this.worker;
    const w = new Worker(new URL("../workers/replayWorker.ts", import.meta.url), {
      type: "module",
    });
    w.onmessage = ({ data }: MessageEvent<ReplayWorkerOut>) => {
      if (data.type === "replay") {
        const p = this.pending.get(data.token);
        if (!p) return;
        this.pending.delete(data.token);
        const { allFrames: _drop, ...loop } = assembleLoop(
          data.start,
          data.changes,
          data.minXs,
          data.ticks,
        );
        p.resolve(loop);
      } else if (data.type === "error") {
        const p = data.token !== undefined ? this.pending.get(data.token) : null;
        if (p && data.token !== undefined) {
          this.pending.delete(data.token);
          p.reject(new Error(data.message));
        }
      }
    };
    this.worker = w;
    return w;
  }

  replay(
    genome: Genome,
    bbox: BBox,
    evalTicks: number,
    seed: number,
  ): Promise<FlightLoopData> {
    const w = this.ensure();
    const token = ++this.token;
    return new Promise((resolve, reject) => {
      this.pending.set(token, { resolve, reject });
      w.postMessage({
        type: "replay",
        token,
        genome,
        bbox,
        evalTicks,
        seed,
        maxTicks: MAX_REPLAY_TICKS,
      } satisfies ReplayRequest);
    });
  }

  dispose(): void {
    this.worker?.terminate();
    this.worker = null;
    this.pending.clear();
  }
}
