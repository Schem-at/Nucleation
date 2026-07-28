// Door pattern classification against the community standard:
//   MYuen222, "Door Pattern Definitions v1.1".
//
// The standard defines every pattern as a set of block coordinates, which
// makes them directly testable: generate the reference set for the measured
// pattern size and test for equality. Nothing here guesses — a pattern is
// either set-equal to a definition under the symmetry group, or it is
// reported as unclassified.
//
// Coordinate convention. The standard's Section 3 works in skydoor terms
// (x = pattern length m, z = pattern height n, y = depth); Section 6 restates
// the same patterns as matrices in door terms (x = length, y = height,
// z = depth). We use the Section 6 matrix convention throughout:
//
//     cells[r][c] = the sorted list of depth layers occupied at row r,
//                   column c, with r counted downwards from the top and
//                   c counted left to right. An empty list means air.
//
// The two conventions differ by a rotation, and every test runs over the full
// symmetry group anyway, so nothing depends on which one a door was authored
// in.

import type {
  Classification,
  PatternCell,
  PatternSurroundings,
  SurfaceReading,
} from "./types";

/** A door pattern as the standard's integrated DBPM (Definition 6.2), with
 *  the layer set kept per cell rather than collapsed — several patterns put
 *  one cell on two different layers. */
export type Grid = {
  m: number;
  n: number;
  /** [row][col] -> ascending, de-duplicated layer indices. */
  cells: number[][][];
};

const emptyGrid = (m: number, n: number): Grid => ({
  m,
  n,
  cells: Array.from({ length: n }, () => Array.from({ length: m }, () => [] as number[])),
});

function put(g: Grid, r: number, c: number, k: number) {
  if (r < 0 || r >= g.n || c < 0 || c >= g.m) return;
  const list = g.cells[r][c];
  if (!list.includes(k)) list.push(k);
}

/** Fill every cell of the grid at a layer given by f. */
function fill(m: number, n: number, f: (r: number, c: number) => number | number[] | null): Grid {
  const g = emptyGrid(m, n);
  for (let r = 0; r < n; r++)
    for (let c = 0; c < m; c++) {
      const k = f(r, c);
      if (k === null) continue;
      for (const kk of Array.isArray(k) ? k : [k]) put(g, r, c, kk);
    }
  return g;
}

/** Canonical form: layers shifted so the shallowest used layer is 0, then
 *  serialised. Two grids are set-equal iff their canonical forms match. */
function canon(g: Grid): string {
  let lo = Infinity;
  for (const row of g.cells) for (const ls of row) for (const k of ls) if (k < lo) lo = k;
  if (lo === Infinity) lo = 0;
  return g.cells
    .map((row) =>
      row
        .map((ls) => (ls.length ? [...ls].sort((a, b) => a - b).map((k) => k - lo).join("/") : "."))
        .join(","),
    )
    .join(";");
}

const layerCount = (g: Grid): number => {
  let lo = Infinity;
  let hi = -Infinity;
  for (const row of g.cells)
    for (const ls of row)
      for (const k of ls) {
        if (k < lo) lo = k;
        if (k > hi) hi = k;
      }
  return hi < lo ? 0 : hi - lo + 1;
};

const isEmpty = (g: Grid) => g.cells.every((row) => row.every((ls) => ls.length === 0));

/* ------------------------------------------------------------------ */
/* Symmetry group                                                      */
/* ------------------------------------------------------------------ */

/** Quarter turn. An n x m grid becomes m x n. */
function rot90(g: Grid): Grid {
  const out = emptyGrid(g.n, g.m);
  for (let r = 0; r < g.m; r++)
    for (let c = 0; c < g.n; c++) out.cells[r][c] = [...g.cells[g.n - 1 - c][r]];
  return out;
}

function mirror(g: Grid): Grid {
  const out = emptyGrid(g.m, g.n);
  for (let r = 0; r < g.n; r++)
    for (let c = 0; c < g.m; c++) out.cells[r][c] = [...g.cells[r][g.m - 1 - c]];
  return out;
}

/** Front/back reversal — the standard's inverted arrangement (Def 2.25). */
function flipDepth(g: Grid): Grid {
  const L = layerCount(g);
  const out = emptyGrid(g.m, g.n);
  for (let r = 0; r < g.n; r++)
    for (let c = 0; c < g.m; c++)
      out.cells[r][c] = g.cells[r][c].map((k) => L - 1 - k).sort((a, b) => a - b);
  return out;
}

const TURN = ["", "rotated 90°", "rotated 180°", "rotated 270°"];

/** Every orientation the same door could have been authored in: four
 *  rotations, an optional mirror, and an optional front/back reversal. */
function orientations(g: Grid): { grid: Grid; label: string }[] {
  const out: { grid: Grid; label: string }[] = [];
  let turned = g;
  for (let t = 0; t < 4; t++) {
    for (const mir of [false, true]) {
      const base = mir ? mirror(turned) : turned;
      for (const inv of [false, true]) {
        const parts: string[] = [];
        if (TURN[t]) parts.push(TURN[t]);
        if (mir) parts.push("mirrored");
        if (inv) parts.push("front/back reversed");
        out.push({ grid: inv ? flipDepth(base) : base, label: parts.join(", ") });
      }
    }
    turned = rot90(turned);
  }
  return out;
}

/* ------------------------------------------------------------------ */
/* Standard geometry helpers                                           */
/* ------------------------------------------------------------------ */

const Q = (a: number, b: number) => Math.floor(a / b);

/** Centre block region, Definition 2.13. Returns inclusive row/column
 *  bounds of the centre block(s) within an m x n pattern. */
export function centreRegion(m: number, n: number) {
  let cw: number;
  let ch: number;
  if (m >= n) {
    cw = m - 2 * Math.floor((n - 1) / 2);
    ch = 2 - n + 2 * Math.floor(n / 2);
  } else {
    cw = 2 - m + 2 * Math.floor(m / 2);
    ch = n - 2 * Math.floor((m - 1) / 2);
  }
  cw = Math.max(1, Math.min(m, cw));
  ch = Math.max(1, Math.min(n, ch));
  const c0 = Math.floor((m - cw) / 2);
  const r0 = Math.floor((n - ch) / 2);
  return { r0, r1: r0 + ch - 1, c0, c1: c0 + cw - 1 };
}

/** Chebyshev/Manhattan components of the distance from a cell to the centre
 *  block region — the standard's a and b indices for the centre-block
 *  families (funnel, cave, bar, vortex). */
function centreOffsets(m: number, n: number) {
  const { r0, r1, c0, c1 } = centreRegion(m, n);
  const dr = (r: number) => (r < r0 ? r0 - r : r > r1 ? r - r1 : 0);
  const dc = (c: number) => (c < c0 ? c0 - c : c > c1 ? c - c1 : 0);
  return { dr, dc, r0, r1, c0, c1 };
}

/* ------------------------------------------------------------------ */
/* Reference patterns                                                  */
/* ------------------------------------------------------------------ */

type Ref = {
  name: string;
  ref: string;
  /** Null when the definition does not apply at this pattern size. */
  build: (m: number, n: number) => Grid | null;
};

/** Patterns in the order they are tested. Ties are possible at small sizes
 *  (a 3 x 3 circle is a 3 x 3 regular), so the plainest definition wins. */
export const REFS: Ref[] = [
  {
    name: "Regular",
    ref: "§3.2",
    build: (m, n) => fill(m, n, () => 0),
  },
  {
    name: "Checkerboard",
    ref: "§3.13",
    build: (m, n) => (m < 2 || n < 2 ? null : fill(m, n, (r, c) => ((r + c) % 2 === 0 ? 0 : null))),
  },
  {
    name: "Sissy bar",
    ref: "§3.12",
    build: (m, n) => (n < 4 ? null : fill(m, n, (r) => (r === 1 || r === n - 2 ? 0 : null))),
  },
  {
    name: "Triangle",
    ref: "§4.3",
    build: (m, n) => {
      if (m !== 2 * n - 1 || n < 2) return null;
      return fill(m, n, (r, c) => {
        const a = c <= n - 1 ? c : 2 * n - 2 - c;
        // column a carries b = 0..a counted up from the bottom row
        return n - 1 - r <= a ? 0 : null;
      });
    },
  },
  {
    name: "Right triangle",
    ref: "§4.4",
    build: (m, n) => (m !== n || n < 2 ? null : fill(m, n, (r, c) => (n - 1 - r <= c ? 0 : null))),
  },
  {
    name: "Diamond",
    ref: "§4.5",
    build: (m, n) => {
      if (m !== n || n % 2 === 0 || n < 3) return null;
      const { dr, dc } = centreOffsets(m, n);
      const rad = Q(n - 1, 2);
      return fill(m, n, (r, c) => (dr(r) + dc(c) <= rad ? 0 : null));
    },
  },
  {
    name: "Semi-diamond",
    ref: "§4.5",
    build: (m, n) => {
      if (m !== n || n % 2 !== 0 || n < 4) return null;
      const { dr, dc } = centreOffsets(m, n);
      const rad = Q(n - 1, 2);
      return fill(m, n, (r, c) => (dr(r) + dc(c) <= rad ? 0 : null));
    },
  },
  {
    name: "Circle",
    ref: "§4.2",
    build: (m, n) => {
      if (m !== n || n < 3) return null;
      const { dr, dc } = centreOffsets(m, n);
      const rad = Q(m - 1, 2) + 0.5;
      return fill(m, n, (r, c) => (dc(c) ** 2 + dr(r) ** 2 <= rad * rad ? 0 : null));
    },
  },
  {
    name: "Full checkerboard",
    ref: "§3.14",
    build: (m, n) => (m < 2 || n < 2 ? null : fill(m, n, (r, c) => (r + c) % 2)),
  },
  {
    name: "Pitch",
    ref: "§3.10",
    build: (m, n) => (n < 2 ? null : fill(m, n, (r) => r)),
  },
  {
    name: "Bar",
    ref: "§3.11",
    build: (m, n) => {
      if (n < 3) return null;
      const { dr } = centreOffsets(m, n);
      return fill(m, n, (r) => dr(r));
    },
  },
  {
    name: "Corner",
    ref: "§3.6",
    build: (m, n) => (m !== n || n < 2 ? null : fill(m, n, (r, c) => Math.max(r, c))),
  },
  {
    name: "Funnel",
    ref: "§3.3",
    build: (m, n) => {
      if (n < 3 || m < n) return null;
      const { dr, dc } = centreOffsets(m, n);
      return fill(m, n, (r, c) => Math.max(dr(r), dc(c)));
    },
  },
  {
    name: "Cave",
    ref: "§3.5",
    build: (m, n) => {
      if (n < 3 || m < n) return null;
      const { dr, dc } = centreOffsets(m, n);
      return fill(m, n, (r, c) => dr(r) + dc(c));
    },
  },
  {
    name: "Dual Cave Corner",
    ref: "§3.7",
    build: (m, n) =>
      m !== n || n < 2 ? null : fill(m, n, (r, c) => Math.min(r + c, 2 * m - 2 - r - c)),
  },
  {
    name: "Gold play button",
    ref: "§3.8",
    build: (m, n) => {
      if (m !== n || m < 3 || m > 5) return null;
      const { r0, r1 } = centreRegion(m, n);
      const dr = (r: number) => (r < r0 ? r0 - r : r > r1 ? r - r1 : 0);
      // Derived from Example 7.1: a bar one column in, plus a play triangle
      // whose half-height shrinks by one per column, both one layer back.
      return fill(m, n, (r, c) => (c === 1 || (c >= 2 && dr(r) <= m - 2 - c) ? 1 : 0));
    },
  },
  { name: "Vortex", ref: "§3.9", build: buildVortex },
];

/** Vortex, Definition 3.9.1: the funnel set S with the spiral T removed and
 *  the spirals U and V added. Verified against Example 7.2 (5 x 5). */
function buildVortex(m: number, n: number): Grid | null {
  if (m !== n || n < 4) return null;
  const { r0, r1, c0, c1 } = centreRegion(m, n);
  if (r1 - r0 !== c1 - c0) return null;
  const midR = (r0 + r1) / 2;
  const midC = (c0 + c1) / 2;

  const A = range(0, Q(n - 1, 2));
  const D = range(0, Q(m, 2));
  const F = range(0, Q(m, 2) - 1);

  // (dx, dz) sign per quadrant: +z is up the matrix, so it subtracts rows.
  const QUAD = [
    { sx: 1, sz: 1 }, // I
    { sx: -1, sz: 1 }, // II
    { sx: -1, sz: -1 }, // III
    { sx: 1, sz: -1 }, // IV
  ];
  // T, U and V are chiral: the (x, z) magnitudes swap as the quadrant turns.
  const SPIN = [
    (k: number) => ({ dx: k + 1, dz: k }), // I
    (k: number) => ({ dx: -k, dz: k + 1 }), // II
    (k: number) => ({ dx: -(k + 1), dz: -k }), // III
    (k: number) => ({ dx: k, dz: -(k + 1) }), // IV
  ];
  const SPIN2 = [
    (k: number) => ({ dx: k + 2, dz: k }),
    (k: number) => ({ dx: -k, dz: k + 2 }),
    (k: number) => ({ dx: -(k + 2), dz: -k }),
    (k: number) => ({ dx: k, dz: -(k + 2) }),
  ];

  const key = (r: number, c: number, k: number) => `${r}|${c}|${k}`;
  const set = new Set<string>();
  const drop = new Set<string>();
  const inBounds = (r: number, c: number) => r >= 0 && r < n && c >= 0 && c < m;

  for (let r = r0; r <= r1; r++)
    for (let c = c0; c <= c1; c++) {
      for (let qi = 0; qi < 4; qi++) {
        const { sx, sz } = QUAD[qi];
        // Does this centre block lie in quadrant qi (Definition 2.16)?
        const inCol = sx > 0 ? c >= midC : c <= midC;
        const inRow = sz > 0 ? r <= midR : r >= midR;
        if (!inCol || !inRow) continue;

        for (const a of A)
          for (const b of A) {
            const rr = r - sz * b;
            const cc = c + sx * a;
            if (inBounds(rr, cc)) set.add(key(rr, cc, Math.max(a, b)));
          }
        for (const d of D) {
          const t = SPIN[qi](d);
          const rr = r - t.dz;
          const cc = c + t.dx;
          if (inBounds(rr, cc)) {
            drop.add(key(rr, cc, d + 1));
            set.add(key(rr, cc, d));
          }
        }
        for (const f of F) {
          const v = SPIN2[qi](f);
          const rr = r - v.dz;
          const cc = c + v.dx;
          if (inBounds(rr, cc)) set.add(key(rr, cc, f + 1));
        }
      }
    }

  const g = emptyGrid(m, n);
  const maxK = Q(n - 1, 2);
  for (const k of set) {
    if (drop.has(k)) continue;
    const [r, c, kk] = k.split("|").map(Number);
    // The worked example layers the centre deepest, so invert.
    put(g, r, c, maxK - kk);
  }
  return isEmpty(g) ? null : g;
}

const range = (a: number, b: number) => {
  const out: number[] = [];
  for (let i = a; i <= b; i++) out.push(i);
  return out;
};

/* ------------------------------------------------------------------ */
/* Composition variants of a reference pattern                         */
/* ------------------------------------------------------------------ */

/** Iris (Def 5.1.1): the pattern less its centre blocks. */
function iris(g: Grid): Grid {
  const { r0, r1, c0, c1 } = centreRegion(g.m, g.n);
  const out = emptyGrid(g.m, g.n);
  for (let r = 0; r < g.n; r++)
    for (let c = 0; c < g.m; c++)
      if (!(r >= r0 && r <= r1 && c >= c0 && c <= c1)) out.cells[r][c] = [...g.cells[r][c]];
  return out;
}

/** Onion (Def 5.1.2): only the blocks touching the pattern perimeter. */
function onion(g: Grid): Grid {
  const out = emptyGrid(g.m, g.n);
  for (let r = 0; r < g.n; r++)
    for (let c = 0; c < g.m; c++)
      if (r === 0 || c === 0 || r === g.n - 1 || c === g.m - 1)
        out.cells[r][c] = [...g.cells[r][c]];
  return out;
}

/* ------------------------------------------------------------------ */
/* Block compositions, Section 5                                       */
/* ------------------------------------------------------------------ */

const shortId = (id: string) => id.replace(/^minecraft:/, "").split("[")[0];
const isGlass = (id: string) => /(^|_)glass(_pane)?$/.test(id) || id.endsWith("_stained_glass");
const isQuartz = (id: string) => id.includes("quartz");
const isRail = (id: string) => id === "rail" || id.endsWith("_rail");

type Tag = { label: string; ref: string };

function compositions(
  cells: PatternCell[],
  m: number,
  n: number,
  around: PatternSurroundings,
): Tag[] {
  const tags: Tag[] = [];
  if (cells.length === 0) return tags;
  const ids = cells.map((c) => shortId(c.id));
  const all = (pred: (id: string) => boolean) => ids.every(pred);

  const { r0, r1, c0, c1 } = centreRegion(m, n);
  const centre = cells.filter((c) => c.r >= r0 && c.r <= r1 && c.c >= c0 && c.c <= c1);
  const centreIds = centre.map((c) => shortId(c.id));
  const centreAll = (pred: (id: string) => boolean) =>
    centreIds.length > 0 && centreIds.every(pred);

  // 5.5 / 5.7 — a single material for the whole door.
  if (all((id) => id === "redstone_lamp")) tags.push({ label: "Full lamp", ref: "§5.3.3" });
  else if (all(isGlass)) {
    const frame = around.frameIds.map(shortId);
    const outer = around.outerIds.map(shortId);
    if (frame.length > 0 && frame.every(isGlass) && outer.length > 0 && outer.every(isQuartz))
      tags.push({ label: "Stargate", ref: "§5.2" });
    else tags.push({ label: "Glass", ref: "§5.5" });
  } else if (all((id) => id === "sand" || id === "red_sand"))
    tags.push({ label: "Sand", ref: "§5.5" });
  else if (all((id) => id === "tnt")) tags.push({ label: "TNT", ref: "§5.7" });

  // 5.3 / 5.4 — the centre blocks swapped for something else.
  if (centre.length < cells.length) {
    if (centreAll((id) => id === "redstone_lamp")) tags.push({ label: "Lamp", ref: "§5.3.1" });
    else if (centreAll(isGlass)) tags.push({ label: "Glass centre", ref: "§5.4.1" });
    else if (centreAll((id) => id === "redstone_block"))
      tags.push({ label: "Redstone centre", ref: "§5.4.1" });
  }

  // 5.6 — a stripe of glass down the middle row or column.
  const glassCells = cells.filter((c) => isGlass(shortId(c.id)));
  if (glassCells.length > 0 && glassCells.length < cells.length) {
    const cStripe = new Set([Q(m - 1, 2), Q(m, 2)]);
    const rStripe = new Set([Q(n - 1, 2), Q(n, 2)]);
    const matches = (pick: (c: PatternCell) => boolean) =>
      cells.every((c) => pick(c) === isGlass(shortId(c.id)));
    if (matches((c) => cStripe.has(c.c)))
      tags.push({ label: "Glass stripe", ref: "§5.6.1" });
    else if (matches((c) => rStripe.has(c.r)))
      tags.push({ label: "Glass stripe", ref: "§5.6.4" });
  }

  // 5.8 — what the doorway is floored with once it is open.
  const sill = around.sillIds.map(shortId).filter((id) => !id.endsWith("air"));
  if (sill.length > 0 && sill.every((id) => id.endsWith("carpet")))
    tags.push({ label: "Carpet", ref: "§5.8.1" });
  else if (sill.length > 0 && sill.every(isRail)) tags.push({ label: "Rail", ref: "§5.8.2" });

  return tags;
}

/* ------------------------------------------------------------------ */
/* Matching                                                            */
/* ------------------------------------------------------------------ */

export type Match = {
  pattern: string;
  patternRef: string;
  transform: string | null;
  /** The Section 5.1 variant that matched, when it was not the plain form. */
  variant: Tag | null;
};

/** Test a measured grid against every reference definition, over the full
 *  symmetry group and the two Section 5.1 variants. Null when nothing is
 *  set-equal — never a nearest guess. */
export function matchGrid(measured: Grid): Match | null {
  for (const view of orientations(measured)) {
    const target = canon(view.grid);
    for (const def of REFS) {
      const base = def.build(view.grid.m, view.grid.n);
      if (!base) continue;
      const variants: { g: Grid; tag: Tag | null }[] = [
        { g: base, tag: null },
        { g: iris(base), tag: { label: "Iris", ref: "§5.1.1" } },
        { g: onion(base), tag: { label: "Onion", ref: "§5.1.2" } },
      ];
      for (const v of variants) {
        if (isEmpty(v.g)) continue;
        if (canon(v.g) !== target) continue;
        return {
          pattern: def.name,
          patternRef: def.ref,
          transform: view.label || null,
          variant: v.tag,
        };
      }
    }
  }
  return null;
}

/* ------------------------------------------------------------------ */
/* Visible surfaces                                                    */
/* ------------------------------------------------------------------ */

/** A depth matrix as ASCII: the depth digit where a block is visible, `.`
 *  where the column is empty all the way through. Reading a door out loud is
 *  how a reader checks the classifier, so the matrices are first-class output
 *  rather than a debug dump. */
export function asciiDepth(depth: number[][]): string {
  return depth
    .map((row) => row.map((k) => (k < 0 ? "." : k > 9 ? "+" : String(k))).join(" "))
    .join("\n");
}

/** Read one visible surface as a pattern in its own right.
 *
 * `cells` are already the first-solid hits cast in from one face, with `k`
 * counted from THAT face, so the surface is a complete pattern by itself and
 * every definition in Section 3/4 applies to it unchanged. */
export function readSurface(
  side: SurfaceReading["side"],
  cells: PatternCell[],
  m: number,
  n: number,
): SurfaceReading {
  const grid = emptyGrid(m, n);
  for (const cell of cells) put(grid, cell.r, cell.c, cell.k);
  for (const row of grid.cells) for (const ls of row) ls.sort((a, b) => a - b);

  const matrix = grid.cells.map((row) => row.map((ls) => (ls.length ? 1 : 0)));
  const depth = grid.cells.map((row) => row.map((ls) => (ls.length ? ls[0] : -1)));
  const hit = cells.length > 0 ? matchGrid(grid) : null;

  return {
    side,
    m,
    n,
    layers: layerCount(grid),
    cells: cells.length,
    pattern: hit?.pattern ?? null,
    patternRef: hit?.patternRef ?? null,
    transform: hit?.transform ?? null,
    variant: hit?.variant ?? null,
    matrix,
    depth,
    ascii: asciiDepth(depth),
  };
}

/** Patterns the standard gives a proper name to in their dual arrangement.
 *  §7.5 works the 5 × 5 vault as "a 5x5 Dual Funnel Door" — the vault IS the
 *  dual funnel, and that is the only such name the standard supplies. */
const DUAL_NAMES: Record<string, { name: string; ref: string }> = {
  Funnel: { name: "Vault", ref: "§7.5" },
};

export type DualReading = {
  /** The pattern both visible surfaces read as. */
  pattern: string;
  patternRef: string;
  /** "Vault" where the standard names the dual arrangement, else
   *  "Dual <pattern>". */
  name: string;
  /** The section the NAME comes from — §7.5 for the vault, else Def 2.24,
   *  which is what defines a dual arrangement in the first place. */
  ref: string;
  /** True when the whole door volume is symmetric about its centre depth
   *  plane, which is what Definition 2.24 actually requires. False means the
   *  two faces match but the machinery between them does not mirror, and the
   *  reading says so instead of claiming the name outright. */
  symmetric: boolean;
};

/** Definition 2.24 (Dual): the pattern is S + S′ where S′ is S reflected in a
 *  plane through the centre blocks whose normal points front-to-back.
 *
 *  Tested on the visible surfaces first — two faces showing the same pattern is
 *  the claim a reader can check by walking round the door — and then on the
 *  full door volume, which is the letter of the definition. */
export function readDual(
  front: SurfaceReading,
  back: SurfaceReading,
  volume: Grid,
): DualReading | null {
  if (!front.pattern || front.pattern !== back.pattern) return null;
  // A FLAT face is never a dual. Reflecting a flat pattern about its own plane
  // returns the same set, so S + S′ = S and Definition 2.24 adds nothing: a
  // plain slab three blocks thick is a regular door, not a "dual regular" one.
  // That is the same rule `classify` already applies to the volume — a pattern
  // extruded uniformly through the wall is still that pattern. Only a
  // depth-varying face has a reflection worth naming.
  if (front.layers < 2) return null;
  // The two faces are read from opposite directions, so a dual door's back
  // surface is its front surface mirrored — a member of the symmetry group
  // either way. Requiring the same DEPTHS as well as the same silhouette is
  // what stops "both sides happen to be funnels" from passing as a dual.
  const fg = emptyGrid(front.m, front.n);
  front.depth.forEach((row, r) => row.forEach((k, c) => k >= 0 && put(fg, r, c, k)));
  const bg = emptyGrid(back.m, back.n);
  back.depth.forEach((row, r) => row.forEach((k, c) => k >= 0 && put(bg, r, c, k)));
  const bt = canon(bg);
  if (!orientations(fg).some((v) => canon(v.grid) === bt)) return null;

  const named = DUAL_NAMES[front.pattern];
  return {
    pattern: front.pattern,
    patternRef: front.patternRef!,
    name: named?.name ?? `Dual ${front.pattern.toLowerCase()}`,
    ref: named?.ref ?? "Def 2.24",
    symmetric: canon(volume) === canon(flipDepth(volume)),
  };
}

/* ------------------------------------------------------------------ */
/* The classifier                                                      */
/* ------------------------------------------------------------------ */

export type ClassifyInput = {
  /** Door blocks in DBPM coordinates, with their material. */
  cells: PatternCell[];
  m: number;
  n: number;
  /** Number of parallel planes the door occupies. */
  layers: number;
  orientation: "Door" | "Skydoor" | "Ceiling Skydoor";
  /** Flush / Deluxe / Trapdoor, when one of the standard's terms applies. */
  qualifiers: string[];
  /** Plain-language reading of where the door sits in the frame. */
  frameNote: string | null;
  around: PatternSurroundings;
  /** What the door shows from each face — the pattern proper. See the header
   *  of `aperture.ts` for why the visible surface, not the filled volume, is
   *  the thing the standard's definitions are about. */
  surfaces: SurfaceReading[];
};

export function classify(input: ClassifyInput): Classification {
  const { cells, m, n, layers, orientation, qualifiers, frameNote, around, surfaces } =
    input;

  const measured = emptyGrid(m, n);
  for (const cell of cells) put(measured, cell.r, cell.c, cell.k);
  for (const row of measured.cells) for (const ls of row) ls.sort((a, b) => a - b);

  // A pattern extruded uniformly through the wall is still that pattern: if
  // every door block occupies the same run of layers, the run is the pattern
  // DEPTH (Def 3.1.1's w), not part of the arrangement. Only patterns whose
  // depth varies from cell to cell — pitch, corner, funnel — are 3-D.
  const shapes = new Set(measured.cells.flatMap((row) => row.filter((ls) => ls.length).map((ls) => ls.join("/"))));
  const extruded = shapes.size === 1 && [...shapes][0].includes("/");
  if (extruded)
    for (const row of measured.cells)
      for (let c = 0; c < row.length; c++) if (row[c].length) row[c] = [0];

  const matrix = measured.cells.map((row) => row.map((ls) => (ls.length ? 1 : 0)));
  const depth = measured.cells.map((row) =>
    row.map((ls) => (ls.length ? Math.min(...ls) : -1)),
  );

  const composition = compositions(cells, m, n, around);

  // The FILLED VOLUME reading, kept for reference. It is what this classifier
  // used to report as the door's pattern, and it is wrong to do so on any door
  // with carriers inside its own doorway — a vault's slime blocks sit in the
  // passage but face nobody. It stays because when a door has no carriers the
  // two readings agree, and a disagreement is itself worth seeing.
  const volumeHit = matchGrid(measured);

  // The SURFACE reading, which is the door's pattern. A one-sided door has one
  // surface and its front face is the answer; a two-sided door has two, and if
  // they agree it is that pattern DUAL (Definition 2.24).
  const front = surfaces.find((s) => s.side === "front") ?? null;
  const back = surfaces.find((s) => s.side === "back") ?? null;
  const dual = front && back ? readDual(front, back, measured) : null;

  let pattern: string | null;
  let patternRef: string | null;
  let transform: string | null = null;
  if (dual) {
    pattern = dual.name;
    patternRef = dual.ref;
  } else if (front?.pattern) {
    pattern = front.pattern;
    patternRef = front.patternRef;
    transform = front.transform;
    if (front.variant && !composition.some((t) => t.label === front.variant!.label))
      composition.unshift(front.variant);
  } else {
    // No surface matched. Fall back to the volume reading rather than throwing
    // the answer away — but only when it found something the surfaces did not.
    pattern = volumeHit?.pattern ?? null;
    patternRef = volumeHit?.patternRef ?? null;
    transform = volumeHit?.transform ?? null;
    if (volumeHit?.variant && !composition.some((t) => t.label === volumeHit.variant!.label))
      composition.unshift(volumeHit.variant);
  }

  // The formal name, pattern FIRST. `qualifiers` are frame properties (Flush,
  // Deluxe, Trapdoor) and follow the pattern rather than standing in for it —
  // "4 × 4 Vault Door, flush" rather than "4 × 4 Flush Door".
  const words = [`${m} × ${n}`];
  if (pattern) words.push(pattern);
  words.push(orientation);

  return {
    name: words.join(" "),
    m,
    n,
    layers,
    orientation,
    qualifiers,
    frameNote,
    pattern,
    patternRef,
    transform,
    dual,
    surfaces,
    volumePattern: volumeHit?.pattern ?? null,
    composition,
    matrix,
    depth,
    extruded,
    unclassified: pattern === null,
  };
}
