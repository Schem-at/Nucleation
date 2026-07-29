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
//
// ------------------------------------------------------------- pattern --
//
// The pattern is what you can SEE from the hallway, not everything solid
// inside the passage.
//
// Reading it as "every passage cell holding a block when shut" gets the answer
// wrong on any door with carriers in its own doorway. A vault pushes its
// centre panel forward with slime blocks; those slime blocks sit in passage
// cells, so the volume reading counts them — but they are directly behind the
// panel they push and nobody standing in the hallway ever sees one. Counting
// them both misdescribes the door and makes it unclassifiable, because no
// definition in the standard has a shape for "panel plus its pusher".
//
// So the surface is cast: for each (row, column) of the cross-section, walk in
// from the face and take the FIRST solid cell. That cell, and its depth below
// the face, is the pattern there. Everything behind it is machinery.
//
// A door can be two-sided, and §7.5 works the vault as exactly that — "a 5x5
// Vault Door, which is a 5x5 Dual Funnel Door" — so both faces are cast and
// both kept. A door whose two visible surfaces are each a funnel is a vault,
// and saying so is the whole reason the surface is extracted separately.
import type {
  Aperture,
  ApertureGeometry,
  Classification,
  PatternCell,
  ReplayBlock,
  SurfaceReading,
  Vec3,
} from "./types";
import { classify, readSurface } from "./classify";

const baseName = (state: string) => state.split("[", 1)[0];
const isAir = (state: string) => baseName(state).endsWith("air");
const posKey = (p: Vec3) => `${p[0]},${p[1]},${p[2]}`;

/** The standard tops out at 32 x 32; a fill wider than that has escaped. */
const MAX_SPAN = 32;

export function aperture(
  closed: ReplayBlock[],
  open: ReplayBlock[],
): {
  aperture: Aperture;
  classification: Classification | null;
  geometry: ApertureGeometry | null;
} | null {
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

  // 1. The seed plane. For each axis, the plane of that axis holding the most
  //    vacated cells; the axes are then tried in that order and the first one
  //    that finds a walkable passage wins.
  //
  //    Taking the single densest plane and stopping there — which is what this
  //    used to do — is right for every door whose opening is wider and taller
  //    than its wall is deep, and wrong for every door that is not. A 2 x 2
  //    doorway in a 3-deep wall leaves 6 vacated cells on an x-plane and 6 on
  //    a y-plane against only 4 on the z-plane the passage actually lies in,
  //    so the seed plane gets chosen THROUGH the doorway rather than across
  //    it: the row axis becomes the wall's own depth, the fill spans it end to
  //    end on its first step, the escape guard fires, and an ordinary 2 x 2
  //    door with its own lever is reported as having no passage at all.
  //
  //    "Trying the densest axis first, first walkable wins" was the original
  //    rule, defended as safe because nothing that used to yield a passage can
  //    change. That is true and it is the wrong half of the story: the hazard
  //    is not that a solved door moves, it is that A WRONG AXIS CAN NOW ANSWER
  //    WHERE NOTHING ANSWERED BEFORE. `Fastest_3x3_Hipster` is the case —
  //    its real 3 x 3 doorway sits on an x-plane whose fill escapes, while a
  //    y-plane grows a 5-cell blob out of two stray vacated cells and, being
  //    merely first, was published as a "2 × 4 Ceiling Skydoor". A confident
  //    wrong answer is worse than a refusal, so first-wins is gone: all three
  //    axes are solved and the winner is chosen STRUCTURALLY, below.
  const axisSeeds: { axis: number; coord: number; n: number }[] = [];
  for (let axis = 0; axis < 3; axis++) {
    const byCoord = new Map<number, number>();
    for (const p of cells) byCoord.set(p[axis], (byCoord.get(p[axis]) ?? 0) + 1);
    let top = { axis, coord: 0, n: 0 };
    for (const [coord, n] of byCoord) if (n > top.n) top = { axis, coord, n };
    axisSeeds.push(top);
  }
  axisSeeds.sort((a, b) => b.n - a.n || a.axis - b.axis);

  type Solution = {
    best: { axis: number; coord: number; n: number };
    vertical: boolean;
    rowAxis: number;
    colAxis: number;
    flat: (p: Vec3) => string;
    at: (col: number, row: number, layer: number) => Vec3;
    domC0: number; domC1: number; domR0: number; domR1: number;
    axis0: number; axis1: number;
    patch: string[];
    lo: number;
    hi: number;
    passage: { set: Set<string>; parts: number } | null;
    /** The fill ran off the edge of the build rather than being stopped by a
     *  wall. Distinct from "found nothing walkable": an escape means the plane
     *  may still be the right one and only the FILL is unusable — see the
     *  candidate rules below. */
    escaped: boolean;
  };

  const parse = (key: string) => key.split(",").map(Number) as [number, number];

  const solveOnAxis = (best: { axis: number; coord: number; n: number }): Solution => {
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
    /** Could a person get through this component?
     *
     *  A wall opening is walked through, and a player is one cell wide and TWO
     *  cells tall, so one row high is a letterbox. A floor opening is dropped
     *  through, which needs no height — but a one-cell-wide slot in a floor is
     *  a gap between blocks, not a hatch, so a hatch has to be two cells in
     *  both directions to count. Between them these are what stop a machine's
     *  own voids reading as doorways: `skittles-270b-3x3hipster` BUDs a 3 × 1
     *  slot open, and it is not the 3 × 3 doorway the build is named for. */
    const walkable = (comp: Set<string>) => {
      let c0 = Infinity, c1 = -Infinity, r0 = Infinity, r1 = -Infinity;
      for (const key of comp) {
        const [a, b] = parse(key);
        if (a < c0) c0 = a;
        if (a > c1) c1 = a;
        if (b < r0) r0 = b;
        if (b > r1) r1 = b;
      }
      return r1 - r0 + 1 >= 2 && (vertical || c1 - c0 + 1 >= 2);
    };
    let escaped = false;
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
          // Reaching the outside of the build means the fill is not bounded by
          // a wall. Often the seed was wrong — but not always: a builder who
          // crops the schematic to the machine leaves the doorway touching the
          // saved region's own edge, with no wall above it to stop anything.
          // So this kills the FILL, and is recorded rather than thrown away.
          if (a <= domC0 || a >= domC1 || b <= domR0 || b >= domR1) {
            escaped = true;
            return null;
          }
          for (const [da, db] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
            const na = a + da, nb = b + db;
            const nk = `${na},${nb}`;
            if (comp.has(nk) || !clear(na, nb)) continue;
            comp.add(nk);
            visited.add(nk);
            queue.push(nk);
          }
        }
        // "A passage anyone could walk through" is meant literally, and it is
        // the only thing standing between a doorway and a gap in a machine.
        if (!walkable(comp)) continue;
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
    const doorLayers = (passageSet: Set<string>) => {
      let l0 = Infinity, l1 = -Infinity;
      for (const key of passageSet) {
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
    return {
      best, vertical, rowAxis, colAxis, flat, at,
      domC0, domC1, domR0, domR1, axis0, axis1,
      patch, lo, hi, passage, escaped: escaped && passage === null,
    };
  };

  // Every axis is solved, then one is chosen on the SHAPE of what it found.
  //
  // A doorway is a rectangle far more often than a machine's leftover void is,
  // and that is a structural property of the answer rather than a threshold
  // tuned until the corpus passed. Two candidates come off each axis:
  //
  //   * the flood, when it was stopped by a wall; and
  //   * failing that, and only when the fill ESCAPED, the vacated patch itself
  //     — but only if the door blocks left a solid rectangle at least 2 x 2.
  //     A cropped schematic has no wall to stop a fill, so the door blocks are
  //     the only evidence of the doorway left; a ragged patch is not evidence.
  //
  // Ranked: rectangles before ragged shapes, a measured fill before a bare
  // patch, and then — deliberately — the DENSEST AXIS, which is the order the
  // old rule used. Size is not the tiebreak: ranking two equally ragged fills
  // by area re-decides doors the old rule already answered, and it re-decided
  // `5x5_circel_entities` into a reading its own timing gate then threw out.
  // Preserving the old order among like candidates is what keeps this change
  // additive; the ranking only ever moves a door the old rule got WRONG.
  //
  // The patch route can only overrule a fill that actually found something if
  // it describes a STRICTLY BIGGER opening — otherwise a 2 x 2 scrap of
  // machinery could unseat a real ragged doorway like an iris, which is
  // exactly the failure this ranking exists to avoid.
  const bboxOf = (set: Set<string>) => {
    let c0 = Infinity, c1 = -Infinity, r0 = Infinity, r1 = -Infinity;
    for (const key of set) {
      const [a, b] = parse(key);
      if (a < c0) c0 = a;
      if (a > c1) c1 = a;
      if (b < r0) r0 = b;
      if (b > r1) r1 = b;
    }
    return { w: c1 - c0 + 1, h: r1 - r0 + 1 };
  };
  type Candidate = {
    sol: Solution;
    set: Set<string>;
    size: number;
    rect: boolean;
    fromFlood: boolean;
    /** Position in the densest-first axis order — the old rule's tiebreak. */
    rank: number;
  };
  const solutions = axisSeeds.map(solveOnAxis);
  const candidates: Candidate[] = [];
  solutions.forEach((s, rank) => {
    if (s.passage) {
      const bb = bboxOf(s.passage.set);
      candidates.push({
        sol: s,
        set: s.passage.set,
        size: s.passage.set.size,
        rect: s.passage.set.size === bb.w * bb.h,
        fromFlood: true,
        rank,
      });
    } else if (s.escaped && s.patch.length > 0) {
      const set = new Set(s.patch);
      const bb = bboxOf(set);
      if (bb.w >= 2 && bb.h >= 2 && set.size === bb.w * bb.h)
        candidates.push({ sol: s, set, size: set.size, rect: true, fromFlood: false, rank });
    }
  });
  let bestFlood = 0;
  for (const c of candidates) if (c.fromFlood && c.size > bestFlood) bestFlood = c.size;
  const eligible = candidates.filter((c) => c.fromFlood || c.size > bestFlood);
  eligible.sort(
    (a, b) =>
      (a.rect ? 0 : 1) - (b.rect ? 0 : 1) ||
      (a.fromFlood ? 0 : 1) - (b.fromFlood ? 0 : 1) ||
      a.rank - b.rank,
  );
  const chosen = eligible[0] ?? null;
  // Nothing anywhere. The densest plane is still the honest answer to "where is
  // the machine", and the fallback below measures from it.
  const sol = chosen ? chosen.sol : solutions[0];
  // A patch-derived doorway becomes this solution's passage: those cells are
  // exactly the ones the door blocks vacated, so they are what the pattern is
  // read over and what the timing is taken across.
  if (chosen && !chosen.fromFlood) sol.passage = { set: chosen.set, parts: 1 };
  const { best, vertical, flat, at, patch, passage, lo, hi } = sol;
  const depth = hi - lo + 1;

  let loC: number, hiC: number, loR: number, hiR: number;
  let clearCells: number;
  const notes: string[] = [];
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
      notes.push(`${passage.parts} separate openings — the largest is reported`);
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
    notes.push("no clear passage found — measured from the door blocks instead");
  }

  const spanC = hiC - loC + 1;
  const spanR = hiR - loR + 1;
  // y is height whenever it lies in the plane; a floor hatch has no height,
  // so its two horizontal spans are reported widest-first.
  const h = vertical ? spanR : Math.min(spanC, spanR);
  const w = vertical ? spanC : Math.max(spanC, spanR);
  // `w × h` is the ENVELOPE the opening sits in, not its area. A cross, an
  // iris caught part-open, a ragged extractor result: all of them clear fewer
  // cells than their own bounding box, and quoting the spans alone invites the
  // reader to multiply them and get a number nobody measured. So the shape is
  // stated whenever the two disagree, and `rectangular` carries it to the sheet
  // rather than leaving the note as the only place it is written down.
  const rectangular = clearCells === spanC * spanR;
  if (!rectangular)
    notes.push(
      `the opening is not a rectangle — ${clearCells} cells clear inside a ${w} × ${h} envelope`,
    );
  const doorway: Aperture = {
    cells: clearCells,
    w,
    h,
    depth,
    rectangular,
    note: notes.length ? notes.join("; ") : null,
  };

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

  // 5b. The visible surface, cast in from each face. `k` is counted from the
  //     face it was cast from, so each surface is a complete pattern in its
  //     own right and every Section 3/4 definition applies to it unchanged.
  const castSurface = (fromFront: boolean) => {
    const cells: PatternCell[] = [];
    const hits: Vec3[] = [];
    for (let col = loC; col <= hiC; col++)
      for (let row = loR; row <= hiR; row++)
        for (let d = 0; d < depth; d++) {
          const layer = fromFront ? lo + d : hi - d;
          const p = at(col, row, layer);
          const block = filled.get(posKey(p));
          if (!block || isAir(block.state)) continue;
          cells.push({ r: rowOf(row), c: col - loC, k: d, id: baseName(block.state) });
          hits.push(p);
          break;
        }
    return { cells, hits };
  };
  const frontCast = castSurface(true);
  // A one-block-thick door has one face worth reading: the back cast would
  // return the same cells at the same depths and "dual" would be vacuous.
  const backCast = depth > 1 ? castSurface(false) : null;
  const surfaces: SurfaceReading[] = [readSurface("front", frontCast.cells, m, n)];
  if (backCast) surfaces.push(readSurface("back", backCast.cells, m, n));

  const visibleSet = new Set(
    [...frontCast.hits, ...(backCast?.hits ?? [])].map(posKey),
  );

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

  // 9. The cells the timing is taken over, in world coordinates. `passage` is
  //    what "walkable" means — air at every layer of the wall once open — and
  //    `closedCells` is the subset of it the door blocks actually fill when
  //    shut. Timing the second set rather than the first is the only way a
  //    sissy bar or a checkerboard ever reads as closed, because most of its
  //    doorway is air in both states.
  let geometry: ApertureGeometry | null = null;
  if (passage) {
    const passageCells: Vec3[] = [];
    const closedCells: Vec3[] = [];
    // …and the split the overlay draws: the blocks a viewer can SEE, and the
    // ones hidden behind them. Both are still door blocks and both are still
    // timed; only one of them is the pattern.
    const visibleCells: Vec3[] = [];
    const carrierCells: Vec3[] = [];
    for (const key of passage.set) {
      const [a, b] = parse(key);
      for (let layer = lo; layer <= hi; layer++) {
        const p = at(a, b, layer);
        passageCells.push(p);
        if (!solidIn(filled, p)) continue;
        closedCells.push(p);
        (visibleSet.has(posKey(p)) ? visibleCells : carrierCells).push(p);
      }
    }
    geometry = {
      passage: passageCells,
      closed: closedCells,
      visible: visibleCells,
      carriers: carrierCells,
      restIsClosed: shut,
    };
  }

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
          surfaces,
        })
      : null;

  return { aperture: doorway, classification, geometry };
}
