/** Flight-loop assembly: rebuild per-tick frames from the recorded change
 * stream, detect the flight period, and cut a seamless one-period window.
 *
 * Period detection (documented method):
 * 1. Primary — min-x cadence: collect the ticks at which non_air_min_x rises
 *    (the machine's trailing edge stepping +1). The gaps between successive
 *    rises are the stride timing; the period is the modal gap, accepted when
 *    at least 3 gaps exist and >= 60% of them agree.
 * 2. Fallback — autocorrelation: correlate the per-tick changed-block-count
 *    signature against itself for lags 2..80 and take the lag with the best
 *    normalized score (> 0.5). Catches fliers whose min-x moves in bursts.
 * 3. Otherwise the machine is treated as static (no loop, still frame).
 *
 * The loop window starts at the last min-x rise that leaves a whole period of
 * recorded ticks after it (steady state, kick transient long gone). Because
 * frame t0+period is frame t0 translated +dx blocks, rendering block x
 * minus (anchorX + phase * dx) makes the wrap pixel-identical: seamless. */

import { buildCast, type CastMember } from "../cast";
import type { ChangeRec, SnapshotBlock } from "../workers/replayWorker";
import type { Block, FlightLoopData, LoopFrame } from "../types";

const key = (p: [number, number, number]) => `${p[0]},${p[1]},${p[2]}`;

function isVisible(state: string): boolean {
  return !state.startsWith("minecraft:air");
}

/** Rebuild the world at every tick 0..ticks from start + change stream. */
export function rebuildFrames(
  start: SnapshotBlock[],
  changes: ChangeRec[],
  ticks: number,
): Block[][] {
  const world = new Map<string, Block>();
  for (const b of start) {
    if (isVisible(b.state))
      world.set(key(b.pos), {
        x: b.pos[0],
        y: b.pos[1],
        z: b.pos[2],
        state: b.state,
      });
  }
  const byTick = new Map<number, ChangeRec[]>();
  for (const c of changes) {
    const arr = byTick.get(c.tick);
    if (arr) arr.push(c);
    else byTick.set(c.tick, [c]);
  }
  const frames: Block[][] = [];
  const snap = () =>
    [...world.values()].sort((a, b) => a.x + a.z - (b.x + b.z) || a.y - b.y);
  frames.push(snap()); // tick 0 = settled start
  for (let t = 1; t <= ticks; t++) {
    for (const c of byTick.get(t) ?? []) {
      if (isVisible(c.to))
        world.set(key(c.pos), {
          x: c.pos[0],
          y: c.pos[1],
          z: c.pos[2],
          state: c.to,
        });
      else world.delete(key(c.pos));
    }
    frames.push(snap());
  }
  return frames;
}

function modalGap(gaps: number[]): number | null {
  if (gaps.length < 3) return null;
  const counts = new Map<number, number>();
  for (const g of gaps) counts.set(g, (counts.get(g) ?? 0) + 1);
  let best = 0;
  let bestGap: number | null = null;
  for (const [g, n] of counts)
    if (n > best || (n === best && bestGap !== null && g < bestGap)) {
      best = n;
      bestGap = g;
    }
  if (bestGap === null || best / gaps.length < 0.6) return null;
  return bestGap;
}

function autocorrPeriod(changesPerTick: number[]): number | null {
  const n = changesPerTick.length;
  if (n < 24) return null;
  const mean = changesPerTick.reduce((a, b) => a + b, 0) / n;
  const centered = changesPerTick.map((v) => v - mean);
  const denom = centered.reduce((a, v) => a + v * v, 0);
  if (denom < 1e-9) return null;
  let bestLag: number | null = null;
  let bestScore = 0.5; // threshold
  for (let lag = 2; lag <= Math.min(80, Math.floor(n / 2)); lag++) {
    let s = 0;
    for (let i = 0; i + lag < n; i++) s += centered[i] * centered[i + lag];
    const score = s / denom;
    if (score > bestScore) {
      bestScore = score;
      bestLag = lag;
    }
  }
  return bestLag;
}

export interface AssembledLoop extends FlightLoopData {
  /** All rebuilt frames (for scrubbing/debug), not persisted. */
  allFrames?: Block[][];
}

export function assembleLoop(
  start: SnapshotBlock[],
  changes: ChangeRec[],
  minXs: number[],
  ticks: number,
): AssembledLoop {
  const frames = rebuildFrames(start, changes, ticks);

  // --- period detection -------------------------------------------------
  const rises: number[] = [];
  for (let t = 1; t < minXs.length; t++)
    if (minXs[t] > minXs[t - 1]) rises.push(t + 1); // minXs[i] is after tick i+1
  const gaps = rises.slice(1).map((t, i) => t - rises[i]);

  let period = modalGap(gaps);
  let method = "min-x cadence (modal gap between +1 shifts of non_air_min_x)";
  if (period === null) {
    const cpt: number[] = new Array(ticks + 1).fill(0);
    for (const c of changes) if (c.tick <= ticks) cpt[c.tick]++;
    period = autocorrPeriod(cpt.slice(5)); // skip the kick transient
    method = "autocorrelation of per-tick change counts";
  }

  if (period === null || period < 1) {
    const f = frames[frames.length - 1] ?? [];
    return {
      frames: [{ blocks: f }],
      dx: 0,
      period: 0,
      method: "static (no period found)",
      anchorX: f.length ? Math.min(...f.map((b) => b.x)) : 0,
    };
  }

  // --- loop window ------------------------------------------------------
  // Last rise tick that leaves a full period of frames after it.
  let t0 = rises.length ? rises[rises.length - 1] : ticks - period;
  while (t0 + period > ticks && t0 > 0) t0 -= period;
  if (t0 < 0) t0 = 0;
  // Prefer a later window (steady state) if several fit.
  while (t0 + 2 * period <= ticks) t0 += period;

  const win: LoopFrame[] = [];
  for (let t = t0; t < t0 + period; t++)
    win.push({ blocks: frames[Math.min(t, frames.length - 1)] });

  // Member cast over the window (local ticks 0..period): the loop's start
  // frame is the initial world, the windowed changes (rebased) the log.
  // moving_piston placeholders resolve to their carried blocks with motion.
  const startBlocks = frames[Math.min(t0, frames.length - 1)].map((b) => ({
    pos: [b.x, b.y, b.z] as [number, number, number],
    state: b.state,
  }));
  const winChanges = changes
    .filter((c) => c.tick > t0 && c.tick <= t0 + period)
    .map((c) => ({ tick: c.tick - t0, pos: c.pos, from: c.from, to: c.to }));
  const cast: CastMember[] = buildCast(startBlocks, winChanges, period);

  const first = win[0].blocks;
  const after = frames[Math.min(t0 + period, frames.length - 1)];
  const minAt = (blocks: Block[]) =>
    blocks.length ? Math.min(...blocks.map((b) => b.x)) : 0;
  const anchorX = minAt(first);
  const dx = minAt(after) - anchorX;

  return { frames: win, dx, period, method, anchorX, cast, allFrames: frames };
}
