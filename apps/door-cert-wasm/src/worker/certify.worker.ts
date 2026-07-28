// Certification pipeline, ported from apps/door-cert/backend/main.py
// `process()` — but running entirely in this module worker on the wasm
// engine served from public/engine/. Only progress + the finished record
// cross the worker boundary; the UI thread never blocks on the sim.
import type {
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
import { aperture } from "../lib/aperture";

const SEED = 12345n;

const post = (m: WorkerMessage) => (self as unknown as Worker).postMessage(m);
const progress = (step: string) => post({ type: "progress", step });

const baseName = (state: string) => state.split("[", 1)[0];
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
