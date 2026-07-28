/** Genome ops — faithful port of apps/flying-ga/backend/ga.py.
 * Genome: one state index per cell of the configured bbox (default 5x3x3),
 * flattened as ((y * bz) + z) * bx + x. */

import { AIR, ALPHABET, MAX_BLOCKS } from "./alphabet";
import type { Rng } from "./rng";

export type Genome = number[];
export type BBox = [number, number, number];

export interface Cell {
  x: number;
  y: number;
  z: number;
  s: number;
}

export function cellIndex(x: number, y: number, z: number, bbox: BBox): number {
  const [bx, , bz] = bbox;
  return (y * bz + z) * bx + x;
}

/** Non-air cells in ga.py's iteration order (y, z, x). */
export function genomeBlocks(genome: Genome, bbox: BBox): Cell[] {
  const [bx, by, bz] = bbox;
  const out: Cell[] = [];
  for (let y = 0; y < by; y++)
    for (let z = 0; z < bz; z++)
      for (let x = 0; x < bx; x++) {
        const s = genome[cellIndex(x, y, z, bbox)];
        if (s !== AIR) out.push({ x, y, z, s });
      }
  return out;
}

export function blockCount(genome: Genome): number {
  let n = 0;
  for (const s of genome) if (s !== AIR) n++;
  return n;
}

/** Turn random non-air cells to air until <= cap remain (in place).
 * The cap is the run's max_blocks constraint — 14 (MAX_BLOCKS) is only the
 * DEFAULT, not a physical ceiling: pistons push at most 12 blocks per push
 * LINE, but multi-engine machines push per row and can be much bigger. The
 * sim itself culls genuinely unpushable designs (they simply don't fly). */
export function capBlocks(genome: Genome, rng: Rng, cap: number = MAX_BLOCKS): Genome {
  const filled: number[] = [];
  genome.forEach((s, i) => {
    if (s !== AIR) filled.push(i);
  });
  while (filled.length > cap) {
    const j = rng.randrange(filled.length);
    const i = filled.splice(j, 1)[0];
    genome[i] = AIR;
  }
  return genome;
}

const idx = (state: string): number => {
  const i = ALPHABET.indexOf(state);
  if (i < 0) throw new Error(`state not in alphabet: ${state}`);
  return i;
};

/** Engine B (verified): travels +x at 1 block / 10 ticks. */
const ENGINE_B: Array<[number, number, number, number]> = [
  [1, 0, 0, idx("minecraft:observer[facing=west,powered=false]")],
  [2, 0, 0, idx("minecraft:slime_block")],
  [3, 0, 0, idx("minecraft:sticky_piston[extended=false,facing=west]")],
  [2, 0, 1, idx("minecraft:sticky_piston[extended=false,facing=east]")],
  [3, 0, 1, idx("minecraft:slime_block")],
  [4, 0, 1, idx("minecraft:observer[facing=east,powered=false]")],
];

export function engineBGenome(bbox: BBox, mirrorZ: boolean): Genome | null {
  const [bx, by, bz] = bbox;
  if (bx < 5 || bz < 2) return null;
  const g: Genome = new Array(bx * by * bz).fill(AIR);
  for (const [x, y, z, s] of ENGINE_B) {
    const zz = mirrorZ ? 1 - z : z;
    g[cellIndex(x, y, zz, bbox)] = s;
  }
  return g;
}

/** The minimal seed: one sticky piston with one slime block stuck to its
 * face — the smallest thing the kick finder can act on (redstone block goes
 * above the piston). Gen 0 grows every machine from this. */
export function minimalSeedGenome(bbox: BBox): Genome | null {
  const [bx, by, bz] = bbox;
  if (bx < 2) return null;
  const g: Genome = new Array(bx * by * bz).fill(AIR);
  g[cellIndex(0, 0, 0, bbox)] = idx(
    "minecraft:sticky_piston[extended=false,facing=east]",
  );
  g[cellIndex(1, 0, 0, bbox)] = idx("minecraft:slime_block");
  return g;
}

export function randomGenome(
  bbox: BBox,
  rng: Rng,
  cap: number = MAX_BLOCKS,
): Genome {
  const [bx, by, bz] = bbox;
  const n = bx * by * bz;
  const density = Math.min(0.9, Math.min(8.0, cap * 0.6) / n); // ~8 blocks expected
  const g: Genome = [];
  for (let i = 0; i < n; i++)
    g.push(rng.next() < density ? rng.randrangeAB(1, ALPHABET.length) : AIR);
  return capBlocks(g, rng, cap);
}

/** Cells inside the 26-neighborhood (Moore) of any non-air cell — the only
 * places an ADDITION mutation may land, so machines grow limb by limb
 * instead of scattering into disconnected clouds. Null when the genome is
 * all air (bootstrap: anything goes). */
function mooreMask(genome: Genome, bbox: BBox): boolean[] | null {
  const [bx, by, bz] = bbox;
  const mask = new Array<boolean>(genome.length).fill(false);
  let any = false;
  for (let y = 0; y < by; y++)
    for (let z = 0; z < bz; z++)
      for (let x = 0; x < bx; x++) {
        if (genome[cellIndex(x, y, z, bbox)] === AIR) continue;
        any = true;
        for (let dy = -1; dy <= 1; dy++)
          for (let dz = -1; dz <= 1; dz++)
            for (let dx = -1; dx <= 1; dx++) {
              const nx = x + dx;
              const ny = y + dy;
              const nz = z + dz;
              if (nx >= 0 && nx < bx && ny >= 0 && ny < by && nz >= 0 && nz < bz)
                mask[cellIndex(nx, ny, nz, bbox)] = true;
            }
      }
  return any ? mask : null;
}

/** Point mutation with locality: state changes and removals are
 * unrestricted; additions (air → block) only land in the Moore
 * neighborhood of the pre-mutation structure. */
export function mutate(
  genome: Genome,
  rate: number,
  rng: Rng,
  bbox: BBox,
  cap: number = MAX_BLOCKS,
): Genome {
  const g = genome.slice();
  const near = mooreMask(genome, bbox);
  for (let i = 0; i < g.length; i++)
    if (rng.next() < rate) {
      // 40% of mutations clear the cell; the rest pick any state.
      if (rng.next() < 0.4) {
        g[i] = AIR;
        continue;
      }
      const s = rng.randrangeAB(1, ALPHABET.length);
      if (g[i] === AIR && near !== null && !near[i]) continue; // no remote seeds
      g[i] = s;
    }
  return capBlocks(g, rng, cap);
}

/** Uniform crossover. It can still splice two parents into a disconnected
 * result; we deliberately accept that rather than repair it — the debris
 * penalty, the must-fly gates, and (when on) the sustained-flight
 * constraint cull detached machines within a generation, and mutation
 * locality stops the disconnection from compounding. */
export function crossover(
  a: Genome,
  b: Genome,
  rng: Rng,
  cap: number = MAX_BLOCKS,
): Genome {
  const g = a.map((ai, i) => (rng.next() < 0.5 ? ai : b[i]));
  return capBlocks(g, rng, cap);
}

export function tournament(
  pop: Genome[],
  fits: number[],
  rng: Rng,
  k = 3,
): Genome {
  let best = -1;
  for (const i of rng.sample(pop.length, k))
    if (best < 0 || fits[i] > fits[best]) best = i;
  return pop[best];
}

export const genomeKey = (g: Genome): string => g.join(",");
