/** Species assignment + lineage bookkeeping for the Lineage view.
 *
 * A genome's structural fingerprint is its bounding dimensions plus the
 * sorted multiset of its block states (state index encodes kind AND facing).
 * Exact fingerprints hash straight to a species; a NEW fingerprint first
 * looks for an existing species within a small edit distance — at most one
 * block added, removed, or substituted (multiset distance ≤ 1) and dims off
 * by ≤ 1 on a single axis — and merges into it, so cosmetic one-block
 * variants don't fragment the chart. Per-generation cost is hashing plus a
 * scan over the (small) species registry for unseen fingerprints only.
 *
 * Lineage: when a fingerprint founds a truly new species, its parent is the
 * species of crossover parent A at reproduction time (elites are their own
 * parent), giving each band a budded-from link. */

import { KIND_OF_INDEX } from "./alphabet";
import { genomeBlocks, type BBox, type Cell, type Genome } from "./genome";
import type { Block, SpeciesInfo } from "../types";

const EXEMPLAR_CAP = 5;
/** Minimum generations between stored exemplars of one species. */
const EXEMPLAR_STRIDE = 20;

interface SpeciesRec {
  info: SpeciesInfo;
  /** state index -> count of the founding member (merge reference). */
  counts: Map<number, number>;
  total: number;
}

function multisetDistance(
  a: Map<number, number>,
  b: Map<number, number>,
): number {
  let adds = 0;
  let removes = 0;
  const keys = new Set([...a.keys(), ...b.keys()]);
  for (const k of keys) {
    const d = (b.get(k) ?? 0) - (a.get(k) ?? 0);
    if (d > 0) adds += d;
    else removes -= d;
  }
  return Math.max(adds, removes);
}

function dimsOf(cells: Cell[]): [number, number, number] {
  let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity, minZ = Infinity, maxZ = -Infinity;
  for (const c of cells) {
    minX = Math.min(minX, c.x); maxX = Math.max(maxX, c.x);
    minY = Math.min(minY, c.y); maxY = Math.max(maxY, c.y);
    minZ = Math.min(minZ, c.z); maxZ = Math.max(maxZ, c.z);
  }
  return cells.length
    ? [maxX - minX + 1, maxY - minY + 1, maxZ - minZ + 1]
    : [0, 0, 0];
}

function dimsMergeable(
  a: [number, number, number],
  b: [number, number, number],
): boolean {
  let off = 0;
  for (let i = 0; i < 3; i++) {
    const d = Math.abs(a[i] - b[i]);
    if (d > 1) return false;
    if (d === 1) off++;
  }
  return off <= 1;
}

function kindSummary(cells: Cell[]): string {
  const byKind = new Map<string, number>();
  for (const c of cells) {
    const k = KIND_OF_INDEX[c.s];
    byKind.set(k, (byKind.get(k) ?? 0) + 1);
  }
  return [...byKind.entries()]
    .sort((x, y) => y[1] - x[1] || (x[0] < y[0] ? -1 : 1))
    .map(([k, n]) => `${n}×${k}`)
    .join(" ");
}

export class SpeciesTracker {
  private species: SpeciesRec[] = [];
  private byFp = new Map<string, number>();

  /** Species id per population index for the CURRENT generation (valid
   * after census()); the runner reads it to tag reproduction parents. */
  ids: number[] = [];

  /** Assign every genome a species and return this generation's counts.
   * `parentSpecies[i]` is the species id of individual i's primary parent
   * (previous generation), or null for gen-0 members. */
  census(
    pop: Genome[],
    bbox: BBox,
    gen: number,
    parentSpecies: Array<number | null>,
    toBlocks: (g: Genome) => Block[],
  ): Array<[number, number]> {
    const counts = new Map<number, number>();
    this.ids = pop.map((g, i) => {
      const cells = genomeBlocks(g, bbox);
      const sid = this.resolve(cells, gen, parentSpecies[i] ?? null, () =>
        toBlocks(g),
      );
      counts.set(sid, (counts.get(sid) ?? 0) + 1);
      return sid;
    });
    // Update lifetime + peak-share + exemplar bookkeeping.
    for (const [sid, n] of counts) {
      const rec = this.species[sid];
      rec.info.lastGen = gen;
      rec.info.peakShare = Math.max(rec.info.peakShare, n / pop.length);
    }
    return [...counts.entries()].sort((a, b) => a[0] - b[0]);
  }

  private resolve(
    cells: Cell[],
    gen: number,
    parent: number | null,
    blocks: () => Block[],
  ): number {
    const dims = dimsOf(cells);
    const cts = new Map<number, number>();
    for (const c of cells) cts.set(c.s, (cts.get(c.s) ?? 0) + 1);
    const fp =
      dims.join("x") +
      "|" +
      [...cts.entries()]
        .sort((a, b) => a[0] - b[0])
        .map(([s, n]) => `${s}:${n}`)
        .join(",");
    const hit = this.byFp.get(fp);
    if (hit !== undefined) {
      this.maybeExemplar(hit, gen, blocks);
      return hit;
    }
    // Near-identical merge: one block off, dims off by ≤1 on one axis.
    for (const rec of this.species) {
      if (
        Math.abs(rec.total - cells.length) <= 1 &&
        dimsMergeable(rec.info.dims, dims) &&
        multisetDistance(rec.counts, cts) <= 1
      ) {
        this.byFp.set(fp, rec.info.id);
        this.maybeExemplar(rec.info.id, gen, blocks);
        return rec.info.id;
      }
    }
    // A genuinely new species is born.
    const id = this.species.length;
    const rec: SpeciesRec = {
      counts: cts,
      total: cells.length,
      info: {
        id,
        label: `S${id + 1}`,
        dims,
        summary: kindSummary(cells),
        firstGen: gen,
        lastGen: gen,
        parent,
        exemplars: [{ gen, blocks: blocks() }],
        peakShare: 0,
      },
    };
    this.species.push(rec);
    this.byFp.set(fp, id);
    return id;
  }

  private maybeExemplar(sid: number, gen: number, blocks: () => Block[]): void {
    const ex = this.species[sid].info.exemplars;
    if (ex.length >= EXEMPLAR_CAP) return;
    if (ex.length === 0 || gen - ex[ex.length - 1].gen >= EXEMPLAR_STRIDE)
      ex.push({ gen, blocks: blocks() });
  }

  /** Serializable registry snapshot (for the UI and per-run persistence). */
  infoSnapshot(): SpeciesInfo[] {
    return this.species.map((r) => ({
      ...r.info,
      dims: [...r.info.dims] as [number, number, number],
      exemplars: r.info.exemplars.map((e) => ({ gen: e.gen, blocks: e.blocks })),
    }));
  }
}
