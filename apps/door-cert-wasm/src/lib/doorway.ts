/** The derived doorway, as something you can LOOK at.
 *
 * `aperture()` already knows the doorway cell by cell — `passage` is every
 * walkable cell across the wall's depth, `closed` is the subset of those that
 * hold a door block when the door is shut. The certificate reports those two
 * sets as numbers ("36-cell opening, 3 deep"). This module is the other half:
 * the palette and the counts the replay overlay draws them with, so a reader
 * can check the classifier's work instead of taking the number on trust.
 *
 * Both the overlay legend and the certificate's aperture line are read off the
 * SAME `ApertureGeometry`, so they cannot disagree — if the extractor is wrong,
 * the picture is wrong in exactly the same way, which is the point.
 *
 * ------------------------------------------------------------------ colour --
 *
 * The overlay has to share the stage with the x-ray, whose flare palette is
 * already at its ceiling: `lib/xray.ts` documents that only THREE hues clear
 * the all-pairs gate on the x-ray surface, and it spends all three
 * (blue #3987e5, orange #d95926, aqua #199e70) plus a hueless wireframe cage
 * for `boundary`. There is no fourth flare hue available — this was re-checked
 * rather than assumed, with `validate_palette.js --pairs all` on #141312:
 *
 *   xray three + violet  #9085e9 → CVD ΔE 1.9 vs blue,   normal 9.8  FAIL
 *   xray three + magenta #d55181 → CVD ΔE 1.6 vs aqua,   normal 11.6 FAIL
 *   xray three + yellow  #c98500 → CVD ΔE 4.8 vs orange, normal 10.6 FAIL
 *
 * A normal-vision ΔE below 15 is a hard fail that no secondary encoding
 * excuses, so the doorway cannot join the flare palette as a fourth colour.
 * It escapes the same way `boundary` does — by MARK. Neither doorway mark is a
 * flare: the passage is drawn as thin box EDGES, the door blocks as a HATCHED
 * translucent pane. Both read as annotation over the build rather than as
 * another category of update, at any hue.
 *
 * That leaves the two doorway marks to separate from each other, which they do
 * comfortably (all-pairs, both marks on screen together):
 *
 *   dark  #9085e9 ↔ #d55181 on #141312  CVD ΔE 16.0, normal 19.7, contrast ≥3:1  PASS
 *   dark  #9085e9 ↔ #d55181 on #262626  CVD ΔE 16.0, normal 19.7, contrast ≥3:1  PASS
 *   light #4a3aa7 ↔ #d55181 on #ffffff  CVD ΔE 16.6, normal 28.4, contrast ≥3:1  PASS
 *
 * Dark mode is selected, not flipped: magenta clears 3:1 on all three surfaces
 * and so holds one step in both, while the violet steps down to #4a3aa7 for
 * the white page (#9085e9 is 2.6:1 there — it would wash out).
 *
 * Hue is never the only cue anyway. Passage and door blocks differ by mark
 * (edges vs hatched pane) and by size (full cell vs inset), and the legend
 * names both with their counts. */

import type { ApertureGeometry, Vec3 } from "./types";

/** How a doorway channel is drawn. Deliberately disjoint from `XrayStyle`'s
 * `glow | cage`: no doorway mark is ever a flare. */
export type DoorwayStyle = {
  hex: number;
  label: string;
  /** `edges` — a thin wireframe box on the cell boundary.
   *  `hatch` — a 45°-hatched translucent pane, inset inside that box. */
  mark: "edges" | "hatch";
};

/** The walkable hole. Violet steps for the light page and for the two dark
 * surfaces (dark page + x-ray darkroom). */
const PASSAGE_LIGHT = 0x4a3aa7;
const PASSAGE_DARK = 0x9085e9;
/** The blocks that fill it. One step, valid on all three surfaces. */
const CLOSED_BOTH = 0xd55181;

export function doorwayStyles(dark: boolean): [DoorwayStyle, DoorwayStyle] {
  return [
    {
      hex: dark ? PASSAGE_DARK : PASSAGE_LIGHT,
      label: "walkable passage",
      mark: "edges",
    },
    { hex: CLOSED_BOTH, label: "door blocks (shut)", mark: "hatch" },
  ];
}

/** What the overlay says about itself. Every number here is counted off the
 * geometry the timing used — none of it is re-derived from the certificate's
 * summary, so the legend is a second, independent reading of the same cells. */
export type DoorwayFacts = {
  /** Walkable cells across the full wall depth. */
  passageCells: number;
  /** Cells of that passage holding a door block when shut. */
  closedCells: number;
  /** The opening's own two spans, and the wall thickness it cuts through. */
  w: number;
  h: number;
  depth: number;
  /** Flat opening area — `w × h`, the number the certificate calls
   *  `aperture.cells`. */
  opening: number;
  /** True when the passage exactly fills its bounding box: a clean rectangular
   *  hole. False means the extractor found an L, a ring, or a ragged edge —
   *  worth seeing rather than smoothing over. */
  rectangular: boolean;
  /** Inclusive world-space bounds of the passage, for the camera and for
   *  tooling that wants to check the overlay landed where the door is. */
  min: Vec3;
  max: Vec3;
};

/** Bounds + spans of a cell set. */
function bounds(cells: Vec3[]): { min: Vec3; max: Vec3; span: Vec3 } {
  const min: Vec3 = [Infinity, Infinity, Infinity];
  const max: Vec3 = [-Infinity, -Infinity, -Infinity];
  for (const p of cells)
    for (let a = 0; a < 3; a++) {
      if (p[a] < min[a]) min[a] = p[a];
      if (p[a] > max[a]) max[a] = p[a];
    }
  return {
    min,
    max,
    span: [max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1],
  };
}

export function doorwayFacts(geo: ApertureGeometry): DoorwayFacts | null {
  if (geo.passage.length === 0) return null;
  const { min, max, span } = bounds(geo.passage);

  // Which axis is the wall's thickness? The one the opening is shallowest
  // along. A door and a skydoor differ only in which axis that turns out to
  // be, so nothing here assumes the doorway is vertical. Ties break x → z → y,
  // so a cube-shaped opening still reports a horizontal depth rather than
  // calling its own height the wall thickness.
  const order = [0, 2, 1];
  let depthAxis = order[0];
  for (const a of order) if (span[a] < span[depthAxis]) depthAxis = a;

  const rest = [0, 1, 2].filter((a) => a !== depthAxis);
  // Height is the vertical span when the doorway is one you walk through; a
  // ceiling skydoor has no vertical span left, so its two spans are x and z.
  const hAxis = rest.includes(1) ? 1 : rest[1];
  const wAxis = rest.find((a) => a !== hAxis) as number;

  const w = span[wAxis];
  const h = span[hAxis];
  const depth = span[depthAxis];

  return {
    passageCells: geo.passage.length,
    closedCells: geo.closed.length,
    w,
    h,
    depth,
    opening: w * h,
    rectangular: geo.passage.length === w * h * depth,
    min,
    max,
  };
}

/** The legend's one-line summary. Reads as the doorway rather than as a stat
 * row: the shape first, then what fills it, then how thick the wall is. */
export function doorwaySummary(f: DoorwayFacts): string {
  const n = (v: number) => v.toLocaleString("en-US");
  return (
    `${f.w} × ${f.h} opening · ${n(f.passageCells)} passage cells · ` +
    `${n(f.closedCells)} door blocks · ${f.depth} deep`
  );
}
