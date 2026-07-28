/** MAP-Elites archive — the quality-diversity alternative to fitness
 * pressure (Mouret & Clune 2015; the flying-machine result this mode is
 * modelled on is GECCO'23, arXiv 2302.00782).
 *
 * The archive, not the population, is the run's memory. A 2-D behaviour
 * grid is carved into bins over two readouts (speed × size by default);
 * each cell keeps exactly one machine — the best it has ever seen by the
 * chosen quality measure. Offspring are bred from parents drawn UNIFORMLY
 * from filled cells, so a mediocre-but-novel design keeps reproducing
 * instead of being out-competed by the current champion. That is the
 * stepping-stone effect: the slow, weird, half-working machines are the
 * ancestors of the fast ones, and fitness pressure alone throws them away.
 *
 * Flight gating: a machine that does not fly cleanly has no meaningful
 * speed, so it is pinned to the speed bin 0 column — it competes with the
 * other grounded designs of its size, never with real fliers. */

import {
  BEHAVIOURS,
  QUALITIES,
  type BehaviourKey,
  type EvalMetrics,
  type MapElitesSpec,
} from "../metrics";
import type { LeaderboardEntry } from "../types";
import type { Rng } from "./rng";

export interface ArchiveCell {
  ix: number;
  iy: number;
  /** Quality of the incumbent under the archive's quality measure. */
  quality: number;
  entry: LeaderboardEntry;
}

/** One cell, flattened for the UI and for persistence. The machine itself
 * is not duplicated here — `id` indexes the run's elite list. */
export interface GridCell {
  ix: number;
  iy: number;
  q: number;
  id: string;
  gen: number;
  speed: number;
  blocks: number;
  disp: number;
}

export interface GridSnapshot {
  spec: MapElitesSpec;
  xRange: [number, number];
  yRange: [number, number];
  cells: GridCell[];
  filled: number;
  total: number;
  /** filled / total, 0..1. */
  fillRate: number;
  /** Sum of cell qualities — the standard QD-score. */
  qdScore: number;
  /** Highest single-cell quality in the archive. */
  bestQuality: number;
}

/** Outcome of one placement attempt. */
export type PlaceResult = "new" | "improved" | null;

export class MapElitesArchive {
  spec: MapElitesSpec;
  private maxBlocks: number;
  private cells = new Map<number, ArchiveCell>();

  constructor(spec: MapElitesSpec, maxBlocks: number) {
    this.spec = spec;
    this.maxBlocks = maxBlocks;
  }

  get size(): number {
    return this.cells.size;
  }

  get total(): number {
    return this.spec.binsX * this.spec.binsY;
  }

  rangeOf(key: BehaviourKey): [number, number] {
    return BEHAVIOURS[key].range(this.maxBlocks);
  }

  private binOn(key: BehaviourKey, bins: number, m: EvalMetrics): number {
    // Flight gate: a machine that never left the ground has no meaningful
    // speed, so it only ever competes in the speed-0 column.
    //
    // The threshold is LAUNCH (fit > 0.5 — the app's own "this moved" bar),
    // NOT `m.flies`, which additionally demands a debris-free flight. Gating
    // on `flies` was measured to break the mode outright: nearly every early
    // flier sheds a block, so the whole population was pinned to bin 0, the
    // archive collapsed to `binsY` reachable cells, and 40 000 evaluations
    // produced a best of 0.05 blk/s. Cleanliness is a QUALITY question (and
    // a constraint, when the run turns "no stragglers" on) — not a reason to
    // pretend a moving machine isn't moving.
    if (key === "speed" && !(m.fit > 0.5)) return 0;
    const [lo, hi] = this.rangeOf(key);
    const t = (BEHAVIOURS[key].value(m) - lo) / Math.max(1e-9, hi - lo);
    return Math.max(0, Math.min(bins - 1, Math.floor(t * bins)));
  }

  /** Behaviour cell for a metrics vector, or null when it is not eligible
   * for the archive at all (hard constraint violation). */
  binOf(m: EvalMetrics): [number, number] | null {
    if (m.violation) return null;
    return [
      this.binOn(this.spec.x, this.spec.binsX, m),
      this.binOn(this.spec.y, this.spec.binsY, m),
    ];
  }

  qualityOf(m: EvalMetrics): number {
    return QUALITIES[this.spec.quality].value(m);
  }

  private key(ix: number, iy: number): number {
    return iy * this.spec.binsX + ix;
  }

  /** Cheap pre-test: would this metrics vector win its cell? Lets callers
   * skip materializing a full entry (block list included) for the ~99 % of
   * offspring that lose to the incumbent. */
  wouldAccept(m: EvalMetrics): boolean {
    const bin = this.binOf(m);
    if (!bin) return false;
    const prev = this.cells.get(this.key(bin[0], bin[1]));
    return !prev || this.qualityOf(m) > prev.quality + 1e-9;
  }

  /** Offer a machine to its cell. It is kept when the cell is empty or it
   * beats the incumbent's quality. */
  place(entry: LeaderboardEntry, m: EvalMetrics): PlaceResult {
    const bin = this.binOf(m);
    if (!bin) return null;
    const [ix, iy] = bin;
    const q = this.qualityOf(m);
    const k = this.key(ix, iy);
    const prev = this.cells.get(k);
    if (!prev) {
      this.cells.set(k, { ix, iy, quality: q, entry });
      return "new";
    }
    if (q > prev.quality + 1e-9) {
      this.cells.set(k, { ix, iy, quality: q, entry });
      return "improved";
    }
    return null;
  }

  /** Mid-run geometry change: re-bin every machine the archive already
   * holds into the new grid instead of discarding the run's discoveries.
   * Colliding machines fight it out on the new quality measure. */
  rebin(spec: MapElitesSpec, maxBlocks: number): { kept: number; merged: number } {
    const held = [...this.cells.values()];
    this.spec = spec;
    this.maxBlocks = maxBlocks;
    this.cells = new Map();
    let merged = 0;
    for (const c of held) {
      const m = c.entry.metrics;
      if (!m) continue;
      const before = this.cells.size;
      const r = this.place(c.entry, m);
      if (r === null || this.cells.size === before) merged++;
    }
    return { kept: this.cells.size, merged };
  }

  /** Drop machines that a tightened constraint set has invalidated. */
  evict(reject: (e: LeaderboardEntry) => string | null): Array<{
    entry: LeaderboardEntry;
    reason: string;
  }> {
    const out: Array<{ entry: LeaderboardEntry; reason: string }> = [];
    for (const [k, c] of [...this.cells]) {
      const reason = c.entry.metrics ? reject(c.entry) : "no stored metrics";
      if (reason) {
        this.cells.delete(k);
        out.push({ entry: c.entry, reason });
      }
    }
    return out;
  }

  /** Uniform draw over FILLED cells — the whole point of the algorithm. */
  sample(rng: Rng): ArchiveCell | null {
    const n = this.cells.size;
    if (n === 0) return null;
    const i = Math.min(n - 1, Math.floor(rng.next() * n));
    let j = 0;
    for (const c of this.cells.values()) if (j++ === i) return c;
    return null;
  }

  /** Elites, best quality first — feeds the leaderboard-shaped consumers
   * (Hall of Fame candidates, persistence). */
  elites(): LeaderboardEntry[] {
    return [...this.cells.values()]
      .sort((a, b) => b.quality - a.quality)
      .map((c) => c.entry);
  }

  qdScore(): number {
    let s = 0;
    for (const c of this.cells.values()) s += Math.max(0, c.quality);
    return s;
  }

  snapshot(): GridSnapshot {
    const cells: GridCell[] = [];
    let best = 0;
    let qd = 0;
    for (const c of this.cells.values()) {
      const m = c.entry.metrics;
      const q = Math.max(0, c.quality);
      qd += q;
      if (q > best) best = q;
      cells.push({
        ix: c.ix,
        iy: c.iy,
        q: Math.round(c.quality * 1000) / 1000,
        id: c.entry.id,
        gen: c.entry.gen,
        speed: Math.round((m?.speed ?? 0) * 1000) / 1000,
        blocks: m?.blocks ?? c.entry.blocks.length,
        disp: Math.round((m?.fit ?? 0) * 100) / 100,
      });
    }
    cells.sort((a, b) => a.iy - b.iy || a.ix - b.ix);
    const total = this.total;
    return {
      spec: this.spec,
      xRange: this.rangeOf(this.spec.x),
      yRange: this.rangeOf(this.spec.y),
      cells,
      filled: cells.length,
      total,
      fillRate: total > 0 ? cells.length / total : 0,
      qdScore: Math.round(qd * 1000) / 1000,
      bestQuality: Math.round(best * 1000) / 1000,
    };
  }
}

/** Natural-unit label for a bin edge, e.g. "0.45" on the speed axis. */
export function binEdgeLabel(
  key: BehaviourKey,
  range: [number, number],
  bins: number,
  i: number,
): string {
  const [lo, hi] = range;
  return BEHAVIOURS[key].fmt(lo + ((hi - lo) * i) / bins);
}

/** Human-readable span of one bin, for tooltips. */
export function binSpanLabel(
  key: BehaviourKey,
  range: [number, number],
  bins: number,
  i: number,
): string {
  const def = BEHAVIOURS[key];
  const [lo, hi] = range;
  const w = (hi - lo) / bins;
  return `${def.fmt(lo + w * i)}–${def.fmt(lo + w * (i + 1))} ${def.unit}`;
}
