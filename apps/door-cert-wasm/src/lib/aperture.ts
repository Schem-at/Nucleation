// The doorway, measured from the passage that opens — not from the set of
// cells that changed.
//
// The changed-set is the whole machine. Every retracting piston vacates a
// cell, so solid->air is mechanism as much as doorway; and a cell that is air
// in both states (the hole a sissy bar leaves, the gap a two-layer door leaves
// on any single layer) is invisible to it entirely. Reading the aperture off
// that set gets a shape that is at once too big and full of holes.
//
// So the doorway is defined as what you can walk through: in the world with
// the door OPEN, the cells of the wall plane that are air at every layer of
// the wall's depth, flood-filled out from the cells the door blocks vacated,
// and stopped by anything still solid — the frame and the parked mechanism.
// The pattern is then read back off the CLOSED world over that footprint, so
// door blocks that never appear on the seed layer are recovered rather than
// left as holes.
import type {
  Aperture,
  Classification,
  PatternCell,
  ReplayBlock,
  Vec3,
} from "./types";
import { classify } from "./classify";

const baseName = (state: string) => state.split("[", 1)[0];
const isAir = (state: string) => baseName(state).endsWith("air");
const posKey = (p: Vec3) => `${p[0]},${p[1]},${p[2]}`;

/** The standard tops out at 32 x 32; a fill wider than that has escaped. */
const MAX_SPAN = 32;

export function aperture(
  closed: ReplayBlock[],
  open: ReplayBlock[],
): { aperture: Aperture; classification: Classification | null } | null {
  // A snapshot may omit air cells entirely, so absence means air and the
  // union of both key sets is the only safe domain to compare over.
  const closedMap = new Map(closed.map((b) => [posKey(b.pos), b]));
  const openMap = new Map(open.map((b) => [posKey(b.pos), b]));
  const forward: Vec3[] = [];
  const backward: Vec3[] = [];
  for (const key of new Set([...closedMap.keys(), ...openMap.keys()])) {
    const cb = closedMap.get(key);
    const ob = openMap.get(key);
    const cSolid = cb !== undefined && !isAir(cb.state);
    const oSolid = ob !== undefined && !isAir(ob.state);
    const pos = (cb ?? ob)!.pos;
    if (cSolid && !oSolid) forward.push(pos);
    else if (!cSolid && oSolid) backward.push(pos);
  }
  const shut = forward.length >= backward.length;
  const cells = shut ? forward : backward;
  if (cells.length === 0) return null;
  // Whichever snapshot holds the door blocks is the closed one; the other is
  // the door standing open. A file saved open measures its own closing.
  const filled = shut ? closedMap : openMap;
  const vacant = shut ? openMap : closedMap;
  const solidIn = (m: Map<string, ReplayBlock>, p: Vec3) => {
    const b = m.get(posKey(p));
    return b !== undefined && !isAir(b.state);
  };

  // 1. The plane holding the most vacated cells — the seed for everything
  //    below. Only its axis has to be right; its coordinate and extent are
  //    both re-derived once the passage is known.
  let best = { axis: 0, coord: 0, n: 0 };
  for (let axis = 0; axis < 3; axis++) {
    const byCoord = new Map<number, number>();
    for (const p of cells) byCoord.set(p[axis], (byCoord.get(p[axis]) ?? 0) + 1);
    for (const [coord, n] of byCoord) if (n > best.n) best = { axis, coord, n };
  }
  // Matrix axes, in the standard's terms: a wall reads rows down the y-axis,
  // a floor hatch reads rows along z. Which of the two the door is decides
  // the orientation (Definition 2.4).
  const vertical = best.axis !== 1;
  const rowAxis = vertical ? 1 : 2;
  const colAxis = best.axis === 0 ? 2 : 0;
  const flat = (p: Vec3) => `${p[colAxis]},${p[rowAxis]}`;
  const at = (col: number, row: number, layer: number): Vec3 => {
    const p: Vec3 = [0, 0, 0];
    p[colAxis] = col;
    p[rowAxis] = row;
    p[best.axis] = layer;
    return p;
  };
  const parse = (key: string) => key.split(",").map(Number) as [number, number];

  // The build's own extent, in flat coordinates and along the plane axis. A
  // fill that reaches the edge of it has left the building.
  let domC0 = Infinity, domC1 = -Infinity, domR0 = Infinity, domR1 = -Infinity;
  let axis0 = Infinity, axis1 = -Infinity;
  for (const m of [closedMap, openMap])
    for (const b of m.values()) {
      const c = b.pos[colAxis], r = b.pos[rowAxis], k = b.pos[best.axis];
      if (c < domC0) domC0 = c;
      if (c > domC1) domC1 = c;
      if (r < domR0) domR0 = r;
      if (r > domR1) domR1 = r;
      if (k < axis0) axis0 = k;
      if (k > axis1) axis1 = k;
    }

  // 2. The largest 4-connected patch of vacated cells inside the seed plane.
  //    These are the seeds the passage is flooded from: wherever a door block
  //    came from, a person can walk.
  const plane = new Set(cells.filter((p) => p[best.axis] === best.coord).map(flat));
  const seen = new Set<string>();
  let patch: string[] = [];
  for (const start of plane) {
    if (seen.has(start)) continue;
    const comp: string[] = [];
    const queue = [start];
    seen.add(start);
    while (queue.length) {
      const key = queue.pop()!;
      comp.push(key);
      const [a, b] = parse(key);
      for (const [da, db] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
        const nk = `${a + da},${b + db}`;
        if (plane.has(nk) && !seen.has(nk)) {
          seen.add(nk);
          queue.push(nk);
        }
      }
    }
    if (comp.length > patch.length) patch = comp;
  }
  const patchSet = new Set(patch);

  // 3. A first guess at the wall's depth: parallel planes that repeat most of
  //    the same patch. Only a starting point — a door whose layers hold
  //    different parts of the pattern (a vault's outer ring and inner panel
  //    sit on different layers) repeats nothing, and reads one layer thick.
  const byPlane = new Map<number, Set<string>>();
  for (const p of cells) {
    const c = p[best.axis];
    if (!byPlane.has(c)) byPlane.set(c, new Set());
    byPlane.get(c)!.add(flat(p));
  }
  const repeats = (c: number) => {
    const s = byPlane.get(c);
    if (!s) return false;
    let hit = 0;
    for (const key of patchSet) if (s.has(key)) hit++;
    return hit >= patch.length * 0.5;
  };
  let lo = best.coord;
  let hi = best.coord;
  while (repeats(lo - 1)) lo--;
  while (repeats(hi + 1)) hi++;

  // 4. The passage. A flat cell is passage if it is air at EVERY layer of the
  //    wall — that is what "you could walk through it" means, and it is why
  //    the middle of a two-layer door is passage even though no single layer
  //    is clear. Flood out from the seeds; anything still solid when open
  //    (frame, parked panels, machinery) is a wall.
  type Fill = { set: Set<string>; parts: number };
  const floodPassage = (l0: number, l1: number): Fill | null => {
    const clear = (col: number, row: number) => {
      for (let layer = l0; layer <= l1; layer++)
        if (solidIn(vacant, at(col, row, layer))) return false;
      return true;
    };
    const visited = new Set<string>();
    let biggest: Set<string> | null = null;
    let parts = 0;
    for (const seed of patch) {
      if (visited.has(seed)) continue;
      const [sc, sr] = parse(seed);
      if (!clear(sc, sr)) continue;
      const comp = new Set<string>([seed]);
      visited.add(seed);
      const queue = [seed];
      while (queue.length) {
        const [a, b] = parse(queue.pop()!);
        // Reaching the outside of the build means the seed was wrong or the
        // doorway is not enclosed; either way the fill is meaningless.
        if (a <= domC0 || a >= domC1 || b <= domR0 || b >= domR1) return null;
        for (const [da, db] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
          const na = a + da, nb = b + db;
          const nk = `${na},${nb}`;
          if (comp.has(nk) || !clear(na, nb)) continue;
          comp.add(nk);
          visited.add(nk);
          queue.push(nk);
        }
      }
      parts++;
      if (biggest === null || comp.size > biggest.size) biggest = comp;
    }
    if (biggest === null) return null;
    let c0 = Infinity, c1 = -Infinity, r0 = Infinity, r1 = -Infinity;
    for (const key of biggest) {
      const [a, b] = parse(key);
      if (a < c0) c0 = a;
      if (a > c1) c1 = a;
      if (b < r0) r0 = b;
      if (b > r1) r1 = b;
    }
    if (c1 - c0 + 1 > MAX_SPAN || r1 - r0 + 1 > MAX_SPAN) return null;
    return { set: biggest, parts };
  };

  // The layers the door blocks actually span, read over the passage. This is
  // what fixes the depth guess above: a layer counts if a passage cell is
  // filled there when shut and clear when open. The moving test matters —
  // a lever sits on the frame inside the doorway on both states, and counting
  // it would drag the wall's depth onto the lever's own layer and then wall
  // the lever's cell out of its own doorway.
  const doorLayers = (passage: Set<string>) => {
    let l0 = Infinity, l1 = -Infinity;
    for (const key of passage) {
      const [a, b] = parse(key);
      for (let layer = axis0; layer <= axis1; layer++) {
        const p = at(a, b, layer);
        if (!solidIn(filled, p) || solidIn(vacant, p)) continue;
        if (layer < l0) l0 = layer;
        if (layer > l1) l1 = layer;
      }
    }
    return l1 < l0 ? null : { l0, l1 };
  };

  // Passage and depth define each other, so run them to a fixed point. One
  // round settles every door tested; the cap is there for the door that
  // oscillates, and lo/hi is only advanced together with the fill it produced
  // so the two never disagree.
  let passage = floodPassage(lo, hi);
  for (let iter = 0; iter < 3 && passage; iter++) {
    const span = doorLayers(passage.set);
    if (!span || (span.l0 === lo && span.l1 === hi)) break;
    const next = floodPassage(span.l0, span.l1);
    if (!next) break;
    lo = span.l0;
    hi = span.l1;
    passage = next;
  }
  const depth = hi - lo + 1;

  let loC: number, hiC: number, loR: number, hiR: number;
  let clearCells: number;
  let note: string | null = null;
  if (passage) {
    loC = Infinity; hiC = -Infinity; loR = Infinity; hiR = -Infinity;
    for (const key of passage.set) {
      const [a, b] = parse(key);
      if (a < loC) loC = a;
      if (a > hiC) hiC = a;
      if (b < loR) loR = b;
      if (b > hiR) hiR = b;
    }
    clearCells = passage.set.size;
    if (passage.parts > 1)
      note = `${passage.parts} separate openings — the largest is reported`;
  } else {
    // No passage: the fill leaked past the wall or nothing was clear through
    // it. Fall back to the footprint of the door blocks themselves, which is
    // at least a real part of the machine.
    const foot = new Set<string>();
    for (const p of cells)
      if (p[best.axis] >= lo && p[best.axis] <= hi) foot.add(flat(p));
    const reach = new Set(patch);
    const stack = [...patch];
    while (stack.length) {
      const [a, b] = parse(stack.pop()!);
      for (let da = -1; da <= 1; da++)
        for (let db = -1; db <= 1; db++) {
          const nk = `${a + da},${b + db}`;
          if (foot.has(nk) && !reach.has(nk)) {
            reach.add(nk);
            stack.push(nk);
          }
        }
    }
    loC = Infinity; hiC = -Infinity; loR = Infinity; hiR = -Infinity;
    for (const key of reach) {
      const [a, b] = parse(key);
      if (a < loC) loC = a;
      if (a > hiC) hiC = a;
      if (b < loR) loR = b;
      if (b > hiR) hiR = b;
    }
    clearCells = patch.length;
    note = "no clear passage found — measured from the door blocks instead";
  }

  const spanC = hiC - loC + 1;
  const spanR = hiR - loR + 1;
  // y is height whenever it lies in the plane; a floor hatch has no height,
  // so its two horizontal spans are reported widest-first.
  const h = vertical ? spanR : Math.min(spanC, spanR);
  const w = vertical ? spanC : Math.max(spanC, spanR);
  const doorway: Aperture = { cells: clearCells, w, h, depth, note };

  // 5. Read the pattern back off the closed world over the passage bbox: a
  //    cell is door wherever it is solid when shut at any layer of the wall.
  //    Taken from the world rather than from the changed-set, so a panel that
  //    only ever occupies a back layer still counts, and nothing outside the
  //    doorway can contribute. Rows run down the matrix, so a wall's rows
  //    count down from its top y.
  const rowOf = (row: number) => (vertical ? hiR - row : row - loR);
  const cellList: PatternCell[] = [];
  for (let col = loC; col <= hiC; col++)
    for (let row = loR; row <= hiR; row++)
      for (let layer = lo; layer <= hi; layer++) {
        const p = at(col, row, layer);
        const block = filled.get(posKey(p));
        if (!block || isAir(block.state)) continue;
        cellList.push({
          r: rowOf(row),
          c: col - loC,
          k: layer - lo,
          id: baseName(block.state),
        });
      }
  const m = spanC;
  const n = spanR;

  // 6. Orientation (Definition 2.4). A wall is a Door; a hatch is a skydoor,
  //    and which kind depends on where the machinery lives — the front side
  //    is the one you can see the pattern from.
  let orientation: Classification["orientation"] = "Door";
  if (!vertical) {
    let below = 0;
    let above = 0;
    for (const b of filled.values()) {
      if (isAir(b.state)) continue;
      if (b.pos[1] < lo) below++;
      else if (b.pos[1] > hi) above++;
    }
    orientation = below >= above ? "Skydoor" : "Ceiling Skydoor";
  }

  // 7. Flush / Deluxe / Trapdoor (Definitions 2.6-2.8): where the outermost
  //    door layer sits relative to the frame surface.
  //
  //    The frame surface is a layer whose ring around the doorway is a
  //    complete, static rectangle — a wall face. Behind it the same ring is
  //    part solid and part machinery, so a loose threshold would read the
  //    mechanism as more wall and push the frame back through the build.
  const ring: [number, number][] = [];
  for (let a = loC - 1; a <= hiC + 1; a++) {
    ring.push([a, loR - 1]);
    ring.push([a, hiR + 1]);
  }
  for (let b = loR; b <= hiR; b++) {
    ring.push([loC - 1, b]);
    ring.push([hiC + 1, b]);
  }
  const faces: number[] = [];
  for (let layer = lo - 6; layer <= hi + 6; layer++) {
    let n2 = 0;
    for (const [a, b] of ring) {
      const p = at(a, b, layer);
      if (solidIn(filled, p) && solidIn(vacant, p)) n2++;
    }
    if (n2 >= ring.length * 0.85) faces.push(layer);
  }
  // How far the door stands from the nearest frame face, and on which side of
  // it. A face is an exposed wall surface; the machinery is always behind the
  // wall, so if the space past a face holds most of the build the door is
  // standing outside that face — deluxe. If it is empty, the door is sunk
  // into the wall instead.
  let totalSolid = 0;
  for (const b of filled.values()) if (!isAir(b.state)) totalSolid++;
  const massBeyond = (face: number, below: boolean) => {
    let n2 = 0;
    for (const b of filled.values()) {
      if (isAir(b.state)) continue;
      const c = b.pos[best.axis];
      if (below ? c < face : c > face) n2++;
    }
    return n2;
  };
  const qualifiers: string[] = [];
  let frameNote: string | null = null;
  let nearest: { gap: number; proud: boolean } | null = null;
  for (const face of faces) {
    if (face >= lo && face <= hi) {
      nearest = { gap: 0, proud: false };
      break;
    }
    const below = face < lo;
    const gap = below ? lo - face : face - hi;
    const proud = massBeyond(face, below) >= totalSolid * 0.2;
    if (nearest === null || gap < nearest.gap) nearest = { gap, proud };
  }
  if (nearest) {
    const { gap, proud } = nearest;
    if (gap === 0) {
      qualifiers.push("Flush");
      frameNote = "outermost layer level with the frame face";
    } else if (proud) {
      if (gap === 1 && vertical) qualifiers.push("Deluxe");
      frameNote = `outermost layer ${gap} block${gap === 1 ? "" : "s"} proud of the frame face`;
    } else {
      frameNote = `outermost layer ${gap} block${gap === 1 ? "" : "s"} inside the frame face`;
    }
    // A skydoor within one block of the surface either way is a trapdoor.
    if (!vertical && gap <= 1) qualifiers.push("Trapdoor");
  }

  // 8. What surrounds the pattern, for the Section 5 compositions.
  const collect = (
    pick: (col: number, row: number) => boolean,
    src: Map<string, ReplayBlock>,
    layers: number[],
  ) => {
    const out: string[] = [];
    for (let a = loC - 2; a <= hiC + 2; a++)
      for (let b = loR - 2; b <= hiR + 2; b++) {
        if (!pick(a, b)) continue;
        for (const layer of layers) {
          const blk = src.get(posKey(at(a, b, layer)));
          if (blk && !isAir(blk.state)) out.push(baseName(blk.state));
        }
      }
    return out;
  };
  const layers: number[] = [];
  for (let c = lo; c <= hi; c++) layers.push(c);
  const inBox = (a: number, b: number) => a >= loC && a <= hiC && b >= loR && b <= hiR;
  const inPad = (a: number, b: number, d: number) =>
    a >= loC - d && a <= hiC + d && b >= loR - d && b <= hiR + d;
  const frameIds = collect((a, b) => !inBox(a, b) && inPad(a, b, 1), filled, layers);
  const outerIds = collect((a, b) => !inPad(a, b, 1) && inPad(a, b, 2), filled, layers);
  const sillRow = vertical ? loR : hiR;
  const sillIds = collect((a, b) => b === sillRow && a >= loC && a <= hiC, vacant, layers);

  const classification =
    cellList.length > 0 && m >= 1 && n >= 1 && m <= MAX_SPAN && n <= MAX_SPAN
      ? classify({
          cells: cellList,
          m,
          n,
          layers: depth,
          orientation,
          qualifiers,
          frameNote,
          around: { frameIds, outerIds, sillIds },
        })
      : null;

  return { aperture: doorway, classification };
}
