/** SNBT builder + kick finder — faithful port of apps/flying-ga/backend/ga.py. */

import {
  ALPHABET,
  PISTON_EAST,
  PISTON_IDXS,
  STICKY_PISTON_IDXS,
} from "./alphabet";
import { genomeBlocks, type BBox, type Genome } from "./genome";

/** Machine sits at x-offset 1 in the corridor. */
export const X_OFF = 1;

function palEntry(state: string): string {
  if (!state.includes("[")) return `{Name: "${state}"}`;
  const [name, propsRaw] = state.slice(0, -1).split("[", 2);
  const kv = propsRaw
    .split(",")
    .map((p) => {
      const [k, v] = p.split("=", 2);
      return `${k}: "${v}"`;
    })
    .join(", ");
  return `{Name: "${name}", Properties: {${kv}}}`;
}

/** Structure SNBT shaped like flying_machine_east.snbt: machine at x-offset 1,
 * corridor of air sized bbox + [26, 2, 2] for travel room. */
export function genomeToSnbt(genome: Genome, bbox: BBox): string {
  const [bx, by, bz] = bbox;
  const size = [bx + 26, by + 2, bz + 2];
  const palette: string[] = [];
  const palOf = new Map<number, number>();
  const blocks: string[] = [];
  for (const { x, y, z, s } of genomeBlocks(genome, bbox)) {
    if (!palOf.has(s)) {
      palOf.set(s, palette.length);
      palette.push(ALPHABET[s]);
    }
    blocks.push(`{pos: [${x + X_OFF}, ${y}, ${z}], state: ${palOf.get(s)}}`);
  }
  return (
    "{\n  DataVersion: 4903,\n" +
    `  size: [${size[0]}, ${size[1]}, ${size[2]}],\n` +
    "  palette: [" +
    palette.map(palEntry).join(", ") +
    "],\n" +
    "  blocks: [" +
    blocks.join(", ") +
    "],\n" +
    "  entities: []\n}"
  );
}

/** Structure-space cell to drop the redstone block into: beside the first
 * east-facing sticky piston, else beside any sticky piston, else any piston
 * (normal pistons joined the alphabet — a push responds to a kick just as
 * well). Must be air. */
export function kickPos(
  genome: Genome,
  bbox: BBox,
): [number, number, number] | null {
  const cells = genomeBlocks(genome, bbox);
  const occupied = new Set(cells.map((c) => `${c.x + X_OFF},${c.y},${c.z}`));
  const pistons =
    cells.filter((c) => c.s === PISTON_EAST).length > 0
      ? cells.filter((c) => c.s === PISTON_EAST)
      : cells.filter((c) => STICKY_PISTON_IDXS.has(c.s)).length > 0
        ? cells.filter((c) => STICKY_PISTON_IDXS.has(c.s))
        : cells.filter((c) => PISTON_IDXS.has(c.s));
  for (const { x, y, z } of pistons) {
    const sx = x + X_OFF;
    // Above first (the reference kick), then sideways in z, then behind.
    const cands: Array<[number, number, number]> = [
      [sx, y + 1, z],
      [sx, y, z + 1],
      [sx, y, z - 1],
      [sx - 1, y, z],
    ];
    for (const cand of cands) {
      if (
        cand[1] >= 0 &&
        cand[2] >= 0 &&
        !occupied.has(`${cand[0]},${cand[1]},${cand[2]}`)
      )
        return cand;
    }
  }
  return null;
}
