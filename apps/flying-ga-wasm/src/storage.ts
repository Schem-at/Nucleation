/** Run history persisted to localStorage — a fresh page load can browse past
 * runs. Index under one key, each run under its own key. History is decimated
 * and filmstrip loops trimmed so a long uncapped run stays well under quota. */

import type { BestRecord, HistoryPoint, RunRecord } from "./types";

/** Elites kept when persisting a map-elites run. A 40×40 grid can hold
 * 1600 machines; storing every block list blows the localStorage quota, so
 * the archive is truncated to its highest-quality cells (the list arrives
 * sorted) and the grid drops the cells whose machine went with them. */
const ARCHIVE_PERSIST_CAP = 600;

const INDEX_KEY = "fgaw:runs";
const runKey = (id: string) => `fgaw:run:${id}`;

export interface RunSummary {
  id: string;
  startedAt: number;
  stoppedAt?: number;
  generation: number;
  best: number;
}

function safeGet<T>(key: string): T | null {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch {
    return null;
  }
}

function safeSet(key: string, value: unknown): boolean {
  try {
    localStorage.setItem(key, JSON.stringify(value));
    return true;
  } catch {
    return false;
  }
}

export function listRuns(): RunSummary[] {
  return safeGet<RunSummary[]>(INDEX_KEY) ?? [];
}

export function loadRun(id: string): RunRecord | null {
  return safeGet<RunRecord>(runKey(id));
}

/** Keep at most `cap` points, preserving first/last and the overall shape. */
export function decimateHistory(h: HistoryPoint[], cap = 2000): HistoryPoint[] {
  if (h.length <= cap) return h;
  const stride = Math.ceil(h.length / cap);
  const out: HistoryPoint[] = [];
  for (let i = 0; i < h.length; i += stride) {
    // Bucket: keep max best + avg mean, stamped at the bucket's last gen.
    const bucket = h.slice(i, i + stride);
    // QD measures are cumulative archive state, so the bucket's LAST value
    // is the correct representative (a max would be identical for a
    // monotone series but would lie after a mid-run grid rebuild).
    const tail = bucket[bucket.length - 1];
    out.push({
      gen: tail.gen,
      best: Math.max(...bucket.map((p) => p.best)),
      mean:
        Math.round(
          (bucket.reduce((a, p) => a + p.mean, 0) / bucket.length) * 100,
        ) / 100,
      ...(tail.qd !== undefined ? { qd: tail.qd } : {}),
      ...(tail.fill !== undefined ? { fill: tail.fill } : {}),
    });
  }
  if (out[out.length - 1].gen !== h[h.length - 1].gen)
    out.push(h[h.length - 1]);
  return out;
}

function slimBests(bests: BestRecord[], cap = 48): BestRecord[] {
  const slim = bests.map((b, i) => ({
    ...b,
    // Keep the loop only for the reigning champion; older filmstrip entries
    // re-simulate on demand.
    loop: i === bests.length - 1 ? b.loop : undefined,
  }));
  if (slim.length <= cap) return slim;
  return [slim[0], ...slim.slice(slim.length - (cap - 1))];
}

export function saveRun(rec: RunRecord): void {
  const slim: RunRecord = {
    ...rec,
    history: decimateHistory(rec.history),
    bests: slimBests(rec.bests),
  };
  if (rec.grid && (rec.archive?.length ?? 0) > ARCHIVE_PERSIST_CAP) {
    const keep = rec.archive!.slice(0, ARCHIVE_PERSIST_CAP);
    const ids = new Set(keep.map((e) => e.id));
    slim.archive = keep;
    slim.grid = {
      ...rec.grid,
      cells: rec.grid.cells.filter((c) => ids.has(c.id)),
    };
  }
  if (!safeSet(runKey(rec.id), slim)) {
    // Quota: drop oldest runs, retry once.
    const runs = listRuns().sort((a, b) => a.startedAt - b.startedAt);
    for (const r of runs.slice(0, Math.ceil(runs.length / 2)))
      localStorage.removeItem(runKey(r.id));
    safeSet(runKey(rec.id), slim);
  }
  const index = listRuns().filter((r) => r.id !== rec.id);
  index.push({
    id: rec.id,
    startedAt: rec.startedAt,
    stoppedAt: rec.stoppedAt,
    generation: rec.generation,
    best: rec.history.length
      ? Math.max(...rec.history.map((p) => p.best))
      : 0,
  });
  index.sort((a, b) => b.startedAt - a.startedAt);
  safeSet(INDEX_KEY, index.slice(0, 30));
}

export function deleteRun(id: string): void {
  localStorage.removeItem(runKey(id));
  safeSet(
    INDEX_KEY,
    listRuns().filter((r) => r.id !== id),
  );
}
