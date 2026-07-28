// Certification pipeline, ported from apps/door-cert/backend/main.py
// `process()` — but running entirely in this module worker on the wasm
// engine served from public/engine/. Only progress + the finished record
// cross the worker boundary; the UI thread never blocks on the sim.
import type {
  Aperture,
  Census,
  CertRecord,
  Classification,
  LeverFlip,
  Material,
  PatternCell,
  ReplayBlock,
  ReplayChange,
  TickEvents,
  Vec3,
  WorkerJob,
  WorkerMessage,
} from "../lib/types";
import { classify } from "../lib/classify";

const SEED = 12345n;

const post = (m: WorkerMessage) => (self as unknown as Worker).postMessage(m);
const progress = (step: string) => post({ type: "progress", step });

const baseName = (state: string) => state.split("[", 1)[0];
const isAir = (state: string) => baseName(state).endsWith("air");
const posKey = (p: Vec3) => `${p[0]},${p[1]},${p[2]}`;

/** Cells that differ between two snapshot keys — the size of a stroke. */
function symmetricDiff(a: string, b: string): number {
  const sa = new Set(a.split("\n"));
  const sb = new Set(b.split("\n"));
  let n = 0;
  for (const k of sa) if (!sb.has(k)) n++;
  for (const k of sb) if (!sa.has(k)) n++;
  return n;
}

function snapshotKey(blocks: ReplayBlock[]): string {
  return blocks
    .map((b) => `${b.pos[0]},${b.pos[1]},${b.pos[2]}|${b.state}`)
    .sort()
    .join("\n");
}

/** First and last tick carrying a block change in [from, to), ignoring the
 * lever cell itself — clicking it is the stimulus, not a response. */
function windowSpan(
  changes: ReplayChange[],
  from: number,
  to: number,
  leverKey: string,
): { first: number | null; last: number | null } {
  let first: number | null = null;
  let last: number | null = null;
  for (const c of changes) {
    if (c.tick < from || c.tick >= to) continue;
    if (posKey(c.pos) === leverKey) continue;
    if (first === null || c.tick < first) first = c.tick;
    if (last === null || c.tick > last) last = c.tick;
  }
  return { first, last };
}

/** The doorway: cells solid in one snapshot and air in the other. Polarity is
 * whichever direction opens more cells, so a door saved open still measures
 * its own opening rather than its closing.
 *
 * Every retracting piston also vacates a cell, so the raw solid→air set is
 * the whole mechanism, not the hole you walk through. The doorway is the one
 * flat, contiguous sheet in that set: take the axis-aligned plane holding the
 * most vacated cells, then the largest connected patch within it. Depth is
 * how many parallel planes repeat that same patch — the wall's thickness. */
function aperture(
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

  // 1. The plane holding the most vacated cells.
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

  // 2. The largest 4-connected patch inside that plane.
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
      const [a, b] = key.split(",").map(Number);
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

  // 3. Depth: parallel planes that repeat most of the same patch.
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
  const depth = hi - lo + 1;

  // 4. The pattern footprint: every door cell in those layers, projected flat.
  //    Diagonal neighbours count, or a checkerboard would fall apart into
  //    single cells.
  const foot = new Set<string>();
  for (const p of cells) if (p[best.axis] >= lo && p[best.axis] <= hi) foot.add(flat(p));
  const reach = new Set(patch);
  const stack = [...patch];
  while (stack.length) {
    const [a, b] = stack.pop()!.split(",").map(Number);
    for (let da = -1; da <= 1; da++)
      for (let db = -1; db <= 1; db++) {
        const nk = `${a + da},${b + db}`;
        if (foot.has(nk) && !reach.has(nk)) {
          reach.add(nk);
          stack.push(nk);
        }
      }
  }
  let loC = Infinity, hiC = -Infinity, loR = Infinity, hiR = -Infinity;
  for (const key of reach) {
    const [a, b] = key.split(",").map(Number);
    if (a < loC) loC = a;
    if (a > hiC) hiC = a;
    if (b < loR) loR = b;
    if (b > hiR) hiR = b;
  }

  // 5. The pattern size is the hole in the wall, not the blocks filling it —
  //    a sissy bar leaves whole rows empty. Flood the non-static cells of the
  //    doorway plane out from the door blocks; if the fill escapes a margin
  //    around them it has leaked past the frame, so fall back to the blocks.
  const PAD = 3;
  const winC0 = loC - PAD, winC1 = hiC + PAD, winR0 = loR - PAD, winR1 = hiR + PAD;
  const isStatic = (col: number, row: number) => {
    const p = at(col, row, best.coord);
    return solidIn(filled, p) && solidIn(vacant, p);
  };
  const hole = new Set(reach);
  const holeQ = [...reach];
  let leaked = false;
  while (holeQ.length && !leaked) {
    const [a, b] = holeQ.pop()!.split(",").map(Number);
    for (const [da, db] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
      const na = a + da, nb = b + db;
      if (na < winC0 || na > winC1 || nb < winR0 || nb > winR1) {
        leaked = true;
        break;
      }
      const nk = `${na},${nb}`;
      if (hole.has(nk) || isStatic(na, nb)) continue;
      hole.add(nk);
      holeQ.push(nk);
    }
  }
  if (!leaked) {
    for (const key of hole) {
      const [a, b] = key.split(",").map(Number);
      if (a < loC) loC = a;
      if (a > hiC) hiC = a;
      if (b < loR) loR = b;
      if (b > hiR) hiR = b;
    }
  }

  const spanC = hiC - loC + 1;
  const spanR = hiR - loR + 1;
  // y is height whenever it lies in the plane; a floor hatch has no height,
  // so its two horizontal spans are reported widest-first.
  const h = vertical ? spanR : Math.min(spanC, spanR);
  const w = vertical ? spanC : Math.max(spanC, spanR);
  const doorway: Aperture = { cells: patch.length, w, h, depth };

  // 6. Read the pattern off the door blocks. Rows run down the matrix, so a
  //    wall's rows count down from its top y.
  const rowOf = (row: number) => (vertical ? hiR - row : row - loR);
  const cellList: PatternCell[] = [];
  let clean = true;
  for (const p of cells) {
    const layer = p[best.axis];
    if (layer < lo || layer > hi) continue;
    const col = p[colAxis], row = p[rowAxis];
    if (col < loC || col > hiC || row < loR || row > hiR) continue;
    const block = filled.get(posKey(p));
    if (!block) {
      clean = false;
      continue;
    }
    cellList.push({ r: rowOf(row), c: col - loC, k: layer - lo, id: baseName(block.state) });
  }
  const m = spanC;
  const n = spanR;

  // 7. Orientation (Definition 2.4). A wall is a Door; a hatch is a skydoor,
  //    and which kind depends on where the machinery lives — the front side
  //    is the one you can see the pattern from.
  let orientation: Classification["orientation"] = "Door";
  if (!vertical) {
    let below = 0;
    let above = 0;
    for (const b of filled.values()) {
      if (isAir(b.state)) continue;
      if (b.pos[1] < best.coord) below++;
      else if (b.pos[1] > best.coord) above++;
    }
    orientation = below >= above ? "Skydoor" : "Ceiling Skydoor";
  }

  // 8. Flush / Deluxe / Trapdoor (Definitions 2.6-2.8): where the outermost
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

  // 9. What surrounds the pattern, for the Section 5 compositions.
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
    clean && cellList.length > 0 && m >= 1 && n >= 1 && m <= 32 && n <= 32
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

/** Parts census, taken from the door at rest. */
function census(blocks: ReplayBlock[]): Census {
  const c: Census = {
    sticky_piston: 0,
    piston: 0,
    observer: 0,
    repeater: 0,
    repeater_delays: [],
    comparator: 0,
    redstone_block: 0,
    redstone_torch: 0,
    redstone_wire: 0,
    slime_block: 0,
    honey_block: 0,
  };
  const delays = new Set<number>();
  for (const b of blocks) {
    const name = baseName(b.state).replace(/^minecraft:/, "");
    switch (name) {
      case "sticky_piston": c.sticky_piston++; break;
      case "piston": c.piston++; break;
      case "observer": c.observer++; break;
      case "comparator": c.comparator++; break;
      case "redstone_block": c.redstone_block++; break;
      case "redstone_torch":
      case "redstone_wall_torch": c.redstone_torch++; break;
      case "redstone_wire": c.redstone_wire++; break;
      case "slime_block": c.slime_block++; break;
      case "honey_block": c.honey_block++; break;
      case "repeater": {
        c.repeater++;
        const m = b.state.match(/delay=(\d+)/);
        if (m) delays.add(Number(m[1]));
        break;
      }
    }
  }
  c.repeater_delays = [...delays].sort((a, b) => a - b);
  return c;
}

async function certify(job: WorkerJob): Promise<CertRecord> {
  // Dynamic import inside the handler: a failing top-level import in a
  // module worker fires neither onmessage nor onerror — this form reports.
  progress("engine");
  // Runtime URL served from public/ — a variable specifier keeps both tsc
  // and rollup from trying to resolve it at build time.
  const engineUrl = "/engine/index.mjs";
  const engine: any = await import(/* @vite-ignore */ engineUrl);
  const { TickSimulation, TickSettleMode, Schematic } = engine;

  // -- parsing ------------------------------------------------------------
  progress("parsing");
  const bytes = new Uint8Array(job.buffer);
  let sim: any;
  let w: number, h: number, l: number;
  if (job.ext === ".snbt") {
    // Vanilla gametest-style structure SNBT ("blocks:" flavor) is parsed
    // by mc-tick itself; Schematic.fromData only knows the nucleation
    // "data:" flavor, so go straight to the simulator.
    const snbt = new TextDecoder().decode(bytes);
    const m = snbt.match(/size:\s*\[([^\]]+)\]/);
    if (!m) throw new Error("structure SNBT has no size field");
    [w, h, l] = m[1].split(",").map((v) => parseInt(v.replace(/[^-\d]/g, ""), 10));
    // A schematic IS a world state: both test doors tick to quiescence in
    // zero ticks exactly as authored. So the file is simulated as it stands.
    // Vanilla's placement pass is a separate thing — it re-derives `locked`
    // and wire connections and loads block-entity NBT after the block writes,
    // which destroys derived state the author saved (a vault's repeater locks
    // go `locked=true` -> `false`). That models pasting, not the schematic.
    sim = TickSimulation.fromSnbt(snbt, TickSettleMode.InWorld, 0, 0, 0, "");
  } else {
    const schem = Schematic.fromData(Array.from(bytes));
    const dims = schem.dimensions();
    [w, h, l] = [dims.x, dims.y, dims.z];
    sim = TickSimulation.fromSchematic(schem, TickSettleMode.InWorld, 0, 0, 0, "");
  }
  sim.setRngSeed(SEED);

  // -- simulating ---------------------------------------------------------
  progress("simulating");
  const initialSnapshot: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());

  const lever = initialSnapshot.find((b) => b.state.startsWith("minecraft:lever"));
  if (!lever) throw new Error("no lever found — the door must include its lever");
  const [lx, ly, lz] = lever.pos;

  // Settle the placement cascade in-world, then measure exactly one cycle:
  // open, close. If the door lands back where it started, that was its
  // steady state and there is nothing more to prove. Only a door saved
  // mid-cycle needs the extra conditioning pass, and it says so on the
  // certificate rather than padding every replay with four lever flips.
  sim.runUntilQuiescent(200);

  const leverKey = posKey(lever.pos);
  type Cycle = {
    tOpen: number;
    tClose: number;
    openBlocks: ReplayBlock[];
    endBlocks: ReplayBlock[];
  };
  const runCycle = (): Cycle => {
    const tOpen: number = sim.tickCount();
    sim.useBlock(lx, ly, lz);
    sim.runUntilQuiescent(300);
    const openBlocks: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());
    const tClose: number = sim.tickCount();
    sim.useBlock(lx, ly, lz);
    sim.runUntilQuiescent(300);
    const endBlocks: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());
    return { tOpen, tClose, openBlocks, endBlocks };
  };

  // -- measuring ----------------------------------------------------------
  progress("measuring");
  let restBlocks: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());
  let restKey = snapshotKey(restBlocks);
  const savedRest = restBlocks;
  let tRebase: number = sim.tickCount();
  // The change log is cumulative and already holds the placement writes, so
  // the measured cycle is everything appended from here on — a count, not a
  // tick filter, because settling can complete without advancing the clock.
  let logBase: number = (JSON.parse(sim.changesJson()) as ReplayChange[]).length;
  let cycle = runCycle();
  // A door saved off its own cycle has to be run onto it before anything
  // measured means much, and one lap is not always enough: a 4x4 vault saved
  // with eight panels parked elsewhere lands 24 cells out after one cycle and
  // only becomes periodic on the second. Keep cycling to a fixed point, with
  // a cap — a machine that never repeats itself has genuinely not reset.
  const MAX_PRIMING = 4;
  let primingCycles = 0;
  while (snapshotKey(cycle.endBlocks) !== restKey && primingCycles < MAX_PRIMING) {
    primingCycles++;
    restBlocks = cycle.endBlocks;
    restKey = snapshotKey(restBlocks);
    tRebase = sim.tickCount();
    logBase = (JSON.parse(sim.changesJson()) as ReplayChange[]).length;
    cycle = runCycle();
  }
  const neededPriming = primingCycles > 0;
  // How far the saved state sits from the cycle the machine actually runs.
  // Reported, never hidden: a schematic that does not reproduce its own
  // resting state is a fact about the download, not a detail to smooth over.
  const savedStateDrift = neededPriming
    ? symmetricDiff(snapshotKey(savedRest), restKey)
    : 0;

  const allChanges: ReplayChange[] = (
    JSON.parse(sim.changesJson()) as ReplayChange[]
  ).slice(logBase);
  const endTick: number = sim.tickCount();

  const openSpan = windowSpan(allChanges, cycle.tOpen, cycle.tClose, leverKey);
  if (openSpan.last === null) throw new Error("lever click caused no block changes");
  const openTicks = openSpan.last - cycle.tOpen;
  const openLatency = openSpan.first! - cycle.tOpen;

  const closeSpan = windowSpan(allChanges, cycle.tClose, Infinity, leverKey);
  const closeTicks = closeSpan.last !== null ? closeSpan.last - cycle.tClose : 0;
  const closeLatency = closeSpan.first !== null ? closeSpan.first - cycle.tClose : 0;

  const verdict = snapshotKey(cycle.endBlocks) === restKey ? "CERTIFIED" : "DID NOT RESET";

  // How much of the build actually travels on a stroke — a door that merely
  // returns to its reference state has proven nothing (a dead machine does
  // that too). This is the number that separates a working door from a
  // crippled one.
  const movedCells = symmetricDiff(restKey, snapshotKey(cycle.openBlocks));

  // The doorway itself, and how much of the build it costs to get it.
  progress("classifying");
  const analysis = aperture(restBlocks, cycle.openBlocks);
  const doorway = analysis?.aperture ?? null;
  const parts = census(restBlocks);
  const cycleTicks = openTicks + closeTicks;
  const cyclesPerMinute = cycleTicks > 0 ? 1200 / cycleTicks : 0;


  // Everything below describes the MEASURED cycle only: the change log,
  // the activity trace and the replay are all rebased so tick 0 is the
  // moment the lever is clicked open.
  const simTicks = endTick - tRebase;
  const changes: ReplayChange[] = allChanges
    .filter((c) => c.tick >= tRebase)
    .map((c) => ({ ...c, tick: c.tick - tRebase }));
  const flips: LeverFlip[] = [
    { tick: cycle.tOpen - tRebase, label: "lever on", measured: true },
    { tick: cycle.tClose - tRebase, label: "lever off", measured: true },
  ];

  const summary: TickEvents[] = JSON.parse(sim.eventsSummaryJson());
  const byTick = new Map(
    summary.filter((r) => r.tick >= tRebase).map((r) => [r.tick - tRebase, r]),
  );
  const eventsPerTick: TickEvents[] = Array.from({ length: simTicks }, (_, t) => ({
    tick: t,
    piston: byTick.get(t)?.piston ?? 0,
    redstone: byTick.get(t)?.redstone ?? 0,
    // The engine does not report item movement yet; the series is carried so
    // the trace does not have to change shape when it does.
    items: byTick.get(t)?.items ?? 0,
    changes: byTick.get(t)?.changes ?? 0,
  }));

  // Two different bursts hide in the change log and they mean opposite
  // things: a cell whose *contents* changed is mass in flight, while a cell
  // that merely re-powered is the signal racing ahead of it. Reporting the
  // sum flatters the door — a 6x6's click tick is ~800 dust updates and no
  // movement at all. Piston bodies count as movement: extended=false->true
  // is the same block name but is the stroke itself.
  const isMovement = (c: ReplayChange) => {
    const from = baseName(c.from);
    const to = baseName(c.to);
    if (from !== to) return true;
    return from.endsWith("piston") || from.endsWith("piston_head");
  };
  const moveTick = new Map<number, number>();
  const signalTick = new Map<number, number>();
  for (const c of changes) {
    const m = isMovement(c) ? moveTick : signalTick;
    m.set(c.tick, (m.get(c.tick) ?? 0) + 1);
  }
  const peakOf = (m: Map<number, number>) => {
    let n = 0;
    let tick = 0;
    for (const [t, v] of m)
      if (v > n) {
        n = v;
        tick = t;
      }
    return { n, tick };
  };
  const peakMove = peakOf(moveTick);
  const peakSignal = peakOf(signalTick);
  const peakChanges = peakMove.n;
  const peakTick = peakMove.tick;

  // Heatmap: per-(x,y) column counts of block changes across the cycle.
  const values = Array.from({ length: h }, () => new Array<number>(w).fill(0));
  for (const c of changes) {
    const [x, y] = c.pos;
    if (x >= 0 && x < w && y >= 0 && y < h) values[y][x] += 1;
  }

  // Materials: base-name counts from the door at rest, air excluded.
  const counts = new Map<string, number>();
  for (const b of restBlocks) {
    const base = baseName(b.state);
    if (base.endsWith("air")) continue;
    counts.set(base, (counts.get(base) ?? 0) + 1);
  }
  const materials: Material[] = [...counts.entries()]
    .map(([id, count]) => ({ id, count }))
    .sort((a, b) => b.count - a.count || (a.id < b.id ? -1 : 1));

  return {
    certificate: {
      name: job.name,
      dims: [w, h, l] as Vec3,
      lever: [lx, ly, lz] as Vec3,
      open_ticks: openTicks,
      close_ticks: closeTicks,
      open_latency: openLatency,
      close_latency: closeLatency,
      materials,
      events_per_tick: eventsPerTick,
      heatmap: { w, h, values },
      sim_ticks: simTicks,
      seed: Number(SEED),
      verdict,
      moved_cells: movedCells,
      aperture: doorway,
      classification: analysis?.classification ?? null,
      peak_changes: peakChanges,
      peak_tick: peakTick,
      peak_signal: peakSignal.n,
      peak_signal_tick: peakSignal.tick,
      census: parts,
      cycles_per_minute: cyclesPerMinute,
      volume: w * h * l,
      needed_priming: neededPriming,
      priming_cycles: primingCycles,
      saved_state_drift: savedStateDrift,
    },
    replay: { blocks: restBlocks, changes, simTicks, flips },
  };
}

self.onmessage = async ({ data }: MessageEvent<WorkerJob>) => {
  try {
    const record = await certify(data);
    post({ type: "done", record });
  } catch (e) {
    post({ type: "error", error: e instanceof Error ? e.message : String(e) });
  }
};
