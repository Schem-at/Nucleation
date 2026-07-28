/** Hall of Fame — the strangest and finest fliers ever seen in this
 * browser, across every run. One inductee per category, persisted to
 * localStorage and challenged live as runs evolve. */

import type { BBox, Genome } from "./ga/genome";
import type { EvalMetrics } from "./metrics";
import type { Block, LeaderboardEntry } from "./types";

const HOF_KEY = "fgaw:hof";

export type HofCategory =
  | "fastest"
  | "slowpoke"
  | "featherweight"
  | "juggernaut"
  | "efficient"
  | "hauler"
  | "sprawl";

export interface HofEntry {
  category: HofCategory;
  runId: string;
  gen: number;
  when: number;
  id: string;
  name?: string;
  metrics: EvalMetrics;
  blocks: Block[];
  genome: Genome;
  /** Sim params so the inductee can be re-flown from any later session. */
  bbox: BBox;
  evalTicks: number;
  seed: number;
}

export type Hof = Partial<Record<HofCategory, HofEntry>>;

interface HofMeta {
  title: string;
  blurb: string;
  hero(m: EvalMetrics): string;
  /** True when a beats b for this category. Candidates must fly. */
  beats(a: EvalMetrics, b: EvalMetrics): boolean;
  /** Extra admission gate beyond "it flies". */
  admit?(m: EvalMetrics): boolean;
}

const bySpeed = (a: EvalMetrics, b: EvalMetrics) => a.speed > b.speed + 1e-9;

export const HOF_META: Record<HofCategory, HofMeta> = {
  fastest: {
    title: "Fastest",
    blurb: "highest blocks/second ever clocked",
    hero: (m) => `${m.speed.toFixed(2)} blk/s`,
    beats: bySpeed,
  },
  slowpoke: {
    title: "Slowpoke",
    blurb: "slowest engine that still flies — sustained, never stalling",
    hero: (m) => `${m.speed.toFixed(2)} blk/s`,
    beats: (a, b) => a.speed < b.speed - 1e-9,
    // Sustained self-propulsion only: a machine that lurches once and
    // coasts does not get to call itself slow. Old records have no
    // `sustained` field and never qualify.
    admit: (m) => m.flies && m.sustained === true && m.speed > 1e-6,
  },
  featherweight: {
    title: "Featherweight",
    blurb: "fewest blocks that still fly",
    hero: (m) => `${m.blocks} blocks`,
    beats: (a, b) =>
      a.blocks < b.blocks || (a.blocks === b.blocks && bySpeed(a, b)),
  },
  juggernaut: {
    title: "Juggernaut",
    blurb: "heaviest machine that still flies",
    hero: (m) => `${m.blocks} blocks`,
    beats: (a, b) =>
      a.blocks > b.blocks || (a.blocks === b.blocks && bySpeed(a, b)),
  },
  efficient: {
    title: "Most efficient",
    blurb: "most speed per block spent",
    hero: (m) => `${(m.speed / Math.max(1, m.blocks)).toFixed(3)} blk/s·blk`,
    beats: (a, b) =>
      a.speed / Math.max(1, a.blocks) > b.speed / Math.max(1, b.blocks) + 1e-9,
  },
  hauler: {
    title: "Pack mule",
    blurb: "most inert cargo carried on a clean flight",
    hero: (m) => `${m.cargo} cargo`,
    beats: (a, b) => a.cargo > b.cargo || (a.cargo === b.cargo && bySpeed(a, b)),
    admit: (m) => m.cargo > 0,
  },
  sprawl: {
    title: "Wing span",
    blurb: "most sprawling flier — biggest volume per block",
    hero: (m) => `${(m.volume / Math.max(1, m.blocks)).toFixed(1)}× spread`,
    beats: (a, b) =>
      a.volume / Math.max(1, a.blocks) > b.volume / Math.max(1, b.blocks) + 1e-9,
    admit: (m) => m.volume > m.blocks,
  },
};

export const HOF_ORDER: HofCategory[] = [
  "fastest",
  "slowpoke",
  "featherweight",
  "juggernaut",
  "efficient",
  "hauler",
  "sprawl",
];

export function loadHof(): Hof {
  try {
    const raw = localStorage.getItem(HOF_KEY);
    return raw ? (JSON.parse(raw) as Hof) : {};
  } catch {
    return {};
  }
}

function saveHof(hof: Hof): void {
  try {
    localStorage.setItem(HOF_KEY, JSON.stringify(hof));
  } catch {
    /* quota — the hall stays in memory for this session */
  }
}

/** Challenge every category with a batch of candidates. Returns the (maybe
 * updated) hall and whether anything changed. */
export function considerForHof(
  hof: Hof,
  candidates: LeaderboardEntry[],
  runId: string,
  sim: { bbox: BBox; evalTicks: number; seed: number },
): { hof: Hof; changed: boolean } {
  let changed = false;
  let next = hof;
  for (const c of candidates) {
    const m = c.metrics;
    if (!m || m.violation || !(m.fit > 0.5)) continue;
    for (const cat of HOF_ORDER) {
      const meta = HOF_META[cat];
      if (meta.admit && !meta.admit(m)) continue;
      const cur = next[cat];
      if (cur && !meta.beats(m, cur.metrics)) continue;
      next = {
        ...next,
        [cat]: {
          category: cat,
          runId,
          gen: c.gen,
          when: Date.now(),
          id: c.id,
          name: c.name,
          metrics: m,
          blocks: c.blocks,
          genome: c.genome,
          bbox: sim.bbox,
          evalTicks: sim.evalTicks,
          seed: sim.seed,
        },
      };
      changed = true;
    }
  }
  if (changed) saveHof(next);
  return { hof: next, changed };
}

export function clearHof(): void {
  try {
    localStorage.removeItem(HOF_KEY);
  } catch {
    /* ignore */
  }
}
