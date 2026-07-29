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
   *  `hatch` — a 45°-hatched translucent pane, inset inside that box.
   *  `core`  — a small plain cube at the centre of the cell.
   *  `dot`   — a smaller, ACHROMATIC cube at the centre of the cell. */
  mark: "edges" | "hatch" | "core" | "dot";
};

/** The walkable hole. Violet steps for the light page and for the two dark
 * surfaces (dark page + x-ray darkroom). */
const PASSAGE_LIGHT = 0x4a3aa7;
const PASSAGE_DARK = 0x9085e9;
/** The blocks that fill it. One step, valid on all three surfaces. */
const CLOSED_BOTH = 0xd55181;

/** ------------------------------------------------------- three channels --
 *
 * The overlay now separates the door blocks you can SEE from the ones hidden
 * behind them — the pattern from the carriers. A vault's centre panel is the
 * pattern; the slime blocks directly behind it, which push it, are not, and
 * being able to point at each is the whole reason the surface is extracted.
 *
 * That third channel takes no new hue, and could not: the note at the top of
 * this file records that no fourth colour clears the all-pairs gate beside the
 * x-ray's three, and that the doorway escapes by MARK instead. So the carriers
 * stay on the same magenta as the pattern — they ARE door blocks, and a
 * different hue would deny it — and separate by mark and by size:
 *
 *   pattern  hatched pane, 0.70 of the cell, translucent — an airy panel
 *   carrier  plain cube,   0.40 of the cell, denser      — a small solid core
 *
 * The separation is SIZE and DENSITY rather than brightness. A first pass made
 * the carrier a dim ghost of the pane and it disappeared: a carrier sits
 * directly behind the block that hides it — that is the definition — so it is
 * always seen through a pane, and dimming it further left the third mark
 * invisible on exactly the doors it exists for. A small dense core reads
 * against a large airy panel at any depth, and stays obviously secondary
 * because it covers a third of the area. The legend names both with their
 * counts, so identity is never carried by appearance alone. */
const CARRIER_ALPHA = 0.86;
const CARRIER_ALPHA_XRAY = 0.6;
export { CARRIER_ALPHA, CARRIER_ALPHA_XRAY };

export function doorwayStyles(
  dark: boolean,
): [DoorwayStyle, DoorwayStyle, DoorwayStyle] {
  return [
    {
      hex: dark ? PASSAGE_DARK : PASSAGE_LIGHT,
      label: "walkable passage",
      mark: "edges",
    },
    { hex: CLOSED_BOTH, label: "pattern blocks (visible)", mark: "hatch" },
    { hex: CLOSED_BOTH, label: "carriers (hidden behind)", mark: "core" },
  ];
}

/** ------------------------------------------------------------ dead weight --
 *
 * The blocks that neither moved nor received a single update all cycle. A
 * fourth mark on the same stage, and the colour budget documented above is
 * already spent — so this one does not spend any: it is ACHROMATIC.
 *
 * That is not a dodge, it is the right encoding. Every other mark on this
 * stage names something the machine DID; dead weight names the absence of it,
 * and a hueless mark reads as exactly that beside three coloured ones. Being
 * achromatic it enters no hue pair, so it needs no all-pairs check against the
 * x-ray's three flares or the doorway's violet and magenta — the same escape
 * `boundary` takes, and for the same reason.
 *
 * It still has to separate from `boundary`, which is also hueless. It does, by
 * MARK and by SIZE: boundary is a full-cell wireframe CAGE, dead weight is a
 * small solid DOT at the cell centre — a third of the cell, no edges, no
 * lattice. The two are never confusable at any zoom, and the legend names both.
 *
 * One step per plate, chosen for contrast against the build rather than
 * against a flat surface: the dot is drawn over textured Minecraft blocks in
 * light mode and over the darkroom in x-ray, so it goes near-black on the
 * white page and near-white on both dark plates. */
const IDLE_LIGHT = 0x1f1d1b;
const IDLE_DARK = 0xe8e6df;
/** Fraction of a cell the dot covers. Deliberately smaller than the carrier
 *  core (0.40) so the two never read as the same mark. */
export const IDLE_DOT = 0.3;

export function idleStyle(dark: boolean): DoorwayStyle {
  return {
    hex: dark ? IDLE_DARK : IDLE_LIGHT,
    label: "did nothing this cycle",
    mark: "dot",
  };
}

/** What the overlay says about itself. Every number here is counted off the
 * geometry the timing used — none of it is re-derived from the certificate's
 * summary, so the legend is a second, independent reading of the same cells. */
export type DoorwayFacts = {
  /** Walkable cells across the full wall depth. */
  passageCells: number;
  /** Cells of that passage holding a door block when shut. */
  closedCells: number;
  /** …of which these are the ones a viewer can see from either face — the
   *  pattern. */
  visibleCells: number;
  /** …and these are hidden behind them: carriers, not pattern. */
  carrierCells: number;
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
    visibleCells: geo.visible.length,
    carrierCells: geo.carriers.length,
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
 * row: the shape first, then what fills it, then how thick the wall is. The
 * door-block count is split only when there is something to split — a door
 * with no carriers should not have to say "0 carriers". */
export function doorwaySummary(f: DoorwayFacts): string {
  const n = (v: number) => v.toLocaleString("en-US");
  const blocks =
    f.carrierCells > 0
      ? `${n(f.visibleCells)} pattern + ${n(f.carrierCells)} carrier blocks`
      : `${n(f.closedCells)} door blocks`;
  return (
    `${f.w} × ${f.h} opening · ${n(f.passageCells)} passage cells · ` +
    `${blocks} · ${f.depth} deep`
  );
}
