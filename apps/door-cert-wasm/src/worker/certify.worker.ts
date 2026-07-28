// Certification pipeline, ported from apps/door-cert/backend/main.py
// `process()` — but running entirely in this module worker on the wasm
// engine served from public/engine/. Only progress + the finished record
// cross the worker boundary; the UI thread never blocks on the sim.
import type {
  Aperture,
  Census,
  CertRecord,
  LeverFlip,
  Material,
  ReplayBlock,
  ReplayChange,
  TickEvents,
  Vec3,
  WorkerJob,
  WorkerMessage,
} from "../lib/types";

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
function aperture(closed: ReplayBlock[], open: ReplayBlock[]): Aperture | null {
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
  const cells = forward.length >= backward.length ? forward : backward;
  if (cells.length === 0) return null;

  // 1. The plane holding the most vacated cells.
  let best = { axis: 0, coord: 0, n: 0 };
  for (let axis = 0; axis < 3; axis++) {
    const byCoord = new Map<number, number>();
    for (const p of cells) byCoord.set(p[axis], (byCoord.get(p[axis]) ?? 0) + 1);
    for (const [coord, n] of byCoord) if (n > best.n) best = { axis, coord, n };
  }
  const [u, v] = [0, 1, 2].filter((i) => i !== best.axis) as [number, number];
  const flat = (p: Vec3) => `${p[u]},${p[v]}`;

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
  let loU = Infinity, hiU = -Infinity, loV = Infinity, hiV = -Infinity;
  for (const key of patch) {
    const [a, b] = key.split(",").map(Number);
    if (a < loU) loU = a;
    if (a > hiU) hiU = a;
    if (b < loV) loV = b;
    if (b > hiV) hiV = b;
  }

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
  let depth = 1;
  for (let c = best.coord - 1; repeats(c); c--) depth++;
  for (let c = best.coord + 1; repeats(c); c++) depth++;

  const eU = hiU - loU + 1;
  const eV = hiV - loV + 1;
  // y is height whenever it lies in the plane; a floor hatch has no height,
  // so its two horizontal spans are reported widest-first.
  const h = u === 1 ? eU : v === 1 ? eV : Math.min(eU, eV);
  const w = u === 1 ? eV : v === 1 ? eU : Math.max(eU, eV);
  return { cells: patch.length, w, h, depth };
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
  /** Builds a second sim under vanilla's paste semantics, for the survival probe. */
  let pasteProbe: () => any;
  let w: number, h: number, l: number;
  if (job.ext === ".snbt") {
    // Vanilla gametest-style structure SNBT ("blocks:" flavor) is parsed
    // by mc-tick itself; Schematic.fromData only knows the nucleation
    // "data:" flavor, so go straight to the simulator.
    const snbt = new TextDecoder().decode(bytes);
    const m = snbt.match(/size:\s*\[([^\]]+)\]/);
    if (!m) throw new Error("structure SNBT has no size field");
    [w, h, l] = m[1].split(",").map((v) => parseInt(v.replace(/[^-\d]/g, ""), 10));
    // AS BUILT, not as pasted. Vanilla's placement pass re-derives repeater
    // `locked` and wire connections and loads block-entity NBT after the
    // block writes, so a door's memory cell can come up unlatched and the
    // machine runs crippled — measuring that would certify a broken variant
    // of the user's door. Paste behaviour is probed separately below.
    sim = TickSimulation.fromSnbt(snbt, TickSettleMode.InWorld, 0, 0, 0, "");
    pasteProbe = () =>
      TickSimulation.fromSnbt(snbt, TickSettleMode.Placement, 0, 0, 0, "");
  } else {
    const schem = Schematic.fromData(Array.from(bytes));
    const dims = schem.dimensions();
    [w, h, l] = [dims.x, dims.y, dims.z];
    sim = TickSimulation.fromSchematic(schem, TickSettleMode.InWorld, 0, 0, 0, "");
    pasteProbe = () =>
      TickSimulation.fromSchematic(schem, TickSettleMode.Placement, 0, 0, 0, "");
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
  let tRebase: number = sim.tickCount();
  // The change log is cumulative and already holds the placement writes, so
  // the measured cycle is everything appended from here on — a count, not a
  // tick filter, because settling can complete without advancing the clock.
  let logBase: number = (JSON.parse(sim.changesJson()) as ReplayChange[]).length;
  let cycle = runCycle();
  let neededPriming = false;

  if (snapshotKey(cycle.endBlocks) !== restKey) {
    // Not at steady state when it was saved — that first cycle becomes the
    // conditioning pass, and the door is measured from where it settled.
    neededPriming = true;
    restBlocks = cycle.endBlocks;
    restKey = snapshotKey(restBlocks);
    tRebase = sim.tickCount();
    logBase = (JSON.parse(sim.changesJson()) as ReplayChange[]).length;
    cycle = runCycle();
  }

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
  const doorway = aperture(restBlocks, cycle.openBlocks);
  const parts = census(restBlocks);
  const cycleTicks = openTicks + closeTicks;
  const cyclesPerMinute = cycleTicks > 0 ? 1200 / cycleTicks : 0;

  // Paste survival: the same door taken through vanilla's placement pass.
  // If its stroke is materially weaker than the as-built one, the design
  // needs priming after being pasted — worth telling the owner.
  progress("paste check");
  let pasteMovedCells = movedCells;
  try {
    const probe = pasteProbe();
    probe.setRngSeed(SEED);
    probe.runUntilQuiescent(200);
    const p0 = snapshotKey(JSON.parse(probe.worldSnapshotJson()));
    probe.useBlock(lx, ly, lz);
    probe.runUntilQuiescent(300);
    const p1 = snapshotKey(JSON.parse(probe.worldSnapshotJson()));
    pasteMovedCells = symmetricDiff(p0, p1);
  } catch {
    pasteMovedCells = 0;
  }
  const pasteSafe = pasteMovedCells >= movedCells * 0.9;

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
      paste_safe: pasteSafe,
      paste_moved_cells: pasteMovedCells,
      aperture: doorway,
      peak_changes: peakChanges,
      peak_tick: peakTick,
      peak_signal: peakSignal.n,
      peak_signal_tick: peakSignal.tick,
      census: parts,
      cycles_per_minute: cyclesPerMinute,
      volume: w * h * l,
      needed_priming: neededPriming,
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
