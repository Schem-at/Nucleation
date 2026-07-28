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

/** Modal gap between successive min-x rises: needs ≥3 gaps with ≥60 %
 * agreement. Shared with the in-eval target-period detector (evalCore). */
export function modalGap(gaps: number[]): number | null {
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
  // Prefer a later window (steady state) if several fit — but keep SEAM_PAD
  // ticks of recorded lookahead after the window when possible, so moves
  // that straddle the seam can be cross-mapped instead of clamped.
  while (t0 + 2 * period + SEAM_PAD <= ticks) t0 += period;
  if (ticks - (t0 + period) < SEAM_PAD && t0 - period >= 0) t0 -= period;
  const pad = Math.min(SEAM_PAD, Math.max(0, ticks - (t0 + period)));

  const win: LoopFrame[] = [];
  for (let t = t0; t < t0 + period; t++)
    win.push({ blocks: frames[Math.min(t, frames.length - 1)] });

  const first = win[0].blocks;
  const after = frames[Math.min(t0 + period, frames.length - 1)];
  const minAt = (blocks: Block[]) =>
    blocks.length ? Math.min(...blocks.map((b) => b.x)) : 0;
  const anchorX = minAt(first);
  const dx = minAt(after) - anchorX;

  // Member cast over the window PLUS the pad (local ticks 0..period+pad):
  // the loop's start frame is the initial world, the windowed changes
  // (rebased) the log. moving_piston placeholders resolve to their carried
  // blocks with motion. The pad-side members are then cross-mapped into a
  // genuinely cyclic cast (see cyclicCast) so the seam does not hitch.
  const startBlocks = frames[Math.min(t0, frames.length - 1)].map((b) => ({
    pos: [b.x, b.y, b.z] as [number, number, number],
    state: b.state,
  }));
  const winChanges = changes
    .filter((c) => c.tick > t0 && c.tick <= t0 + period + pad)
    .map((c) => ({ tick: c.tick - t0, pos: c.pos, from: c.from, to: c.to }));
  const raw: CastMember[] = buildCast(startBlocks, winChanges, period + pad);
  const cast: CastMember[] =
    pad > 0 ? cyclicCast(raw, startBlocks, period, dx) : raw;

  return { frames: win, dx, period, method, anchorX, cast, allFrames: frames };
}

/** Ticks of recorded lookahead kept past the loop window so seam-straddling
 * piston moves can be cross-mapped: a stroke resolves within ~2 ticks. */
const SEAM_PAD = 4;

const isBaseState = (state: string): boolean =>
  state.startsWith("minecraft:piston[") ||
  state.startsWith("minecraft:sticky_piston[");

/** Cross-map an extended-window cast (built over `period + pad` ticks) into
 * a cyclic cast for local ticks 0..period. Frame t0+period is frame t0
 * translated +dx, so a move in progress at the seam has two half-views: its
 * own tail at the end of the loop, and — one period earlier, -dx to the
 * left — the identical move already mid-flight at t = 0. The old cast
 * CLAMPED both halves (the entry side re-played the whole move from its
 * source; the exit side froze or got yanked), hitching once per loop.
 *
 *  - "Phantoms" — flights already in progress at the window start, whose
 *    true launch tick is outside the window — keep only their seated tail;
 *    the [0, landing) half of the move is owned by the wrapped twin below.
 *    (Exception: an extending piston BASE never travels, so its phantom is
 *    simply drawn static for its whole interval.)
 *  - Seam-straddling moves from the pad side are cloned into -dx twins on
 *    wrapped time (start - period), carrying the true mid-flight pose into
 *    the head of the loop. Retract-piston companions (the extended-shell
 *    member sharing the head's cell and lifetime) wrap along with them. */
function cyclicCast(
  raw: CastMember[],
  start: Array<{ pos: [number, number, number]; state: string }>,
  period: number,
  dx: number,
): CastMember[] {
  const startMoving = new Set(
    start
      .filter((b) => b.state.startsWith("minecraft:moving_piston"))
      .map((b) => key(b.pos)),
  );
  // Moves in flight across t = period, keyed by cell + birth tick so the
  // retract shell (same cell, same lifetime, no motion) wraps with its head.
  const seamKeys = new Set<string>();
  for (const m of raw)
    if (m.motion && m.start > 0 && m.start < period && m.motion.until > period)
      seamKeys.add(`${m.x},${m.y},${m.z}|${m.start}`);

  const out: CastMember[] = [];
  for (const m of raw) {
    if (m.start >= period) continue; // pad-only: its pre-seam twin is below
    if (m.motion && m.start === 0 && startMoving.has(`${m.x},${m.y},${m.z}`)) {
      // Phantom: the move was already in progress at the window start.
      if (isBaseState(m.state) && m.state.includes("extended=true")) {
        // An extending base never travels — draw it seated throughout.
        out.push({ ...m, motion: undefined });
      } else {
        // Keep the seated tail; the wrapped twin owns [0, landing).
        if (m.motion.until < m.end)
          out.push({ ...m, start: m.motion.until, motion: undefined });
      }
      continue;
    }
    if (!seamKeys.has(`${m.x},${m.y},${m.z}|${m.start}`)) {
      out.push(m);
      continue;
    }
    // A seam-straddler: the original plays the tail of the loop…
    out.push({ ...m, end: Math.min(m.end, period) });
    // …and a -dx twin on wrapped time plays the head. Flights are trimmed
    // to their landing so the seated block does not double-draw over the
    // phantom tail kept above; retract heads (headBase === own cell) and
    // shells keep their full interval — head + shell compose the closed
    // cube until the extended=false swap.
    const retractHead =
      m.headBase !== undefined &&
      m.headBase[0] === m.x &&
      m.headBase[1] === m.y &&
      m.headBase[2] === m.z;
    const twinEnd =
      m.motion && !retractHead ? Math.min(m.end, m.motion.until) : m.end;
    out.push({
      ...m,
      x: m.x - dx,
      start: m.start - period,
      end: twinEnd - period,
      motion: m.motion
        ? {
            fx: m.motion.fx - dx,
            fy: m.motion.fy,
            fz: m.motion.fz,
            until: m.motion.until - period,
          }
        : undefined,
      headBase: m.headBase
        ? ([m.headBase[0] - dx, m.headBase[1], m.headBase[2]] as [
            number,
            number,
            number,
          ])
        : undefined,
    });
  }
  return out;
}
