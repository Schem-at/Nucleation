// Certification pipeline, ported from apps/door-cert/backend/main.py
// `process()` — but running entirely in this module worker on the wasm
// engine served from public/engine/. Only progress + the finished record
// cross the worker boundary; the UI thread never blocks on the sim.
import type {
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

function snapshotKey(blocks: ReplayBlock[]): string {
  return blocks
    .map((b) => `${b.pos[0]},${b.pos[1]},${b.pos[2]}|${b.state}`)
    .sort()
    .join("\n");
}

function lastChangeTickAfter(changes: ReplayChange[], afterTick: number): number | null {
  let last: number | null = null;
  for (const c of changes) {
    if (c.tick >= afterTick && (last === null || c.tick > last)) last = c.tick;
  }
  return last;
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
    sim = TickSimulation.fromSnbt(snbt, TickSettleMode.Placement, 0, 0, 0, "");
  } else {
    const schem = Schematic.fromData(Array.from(bytes));
    const dims = schem.dimensions();
    [w, h, l] = [dims.x, dims.y, dims.z];
    sim = TickSimulation.fromSchematic(schem, TickSettleMode.Placement, 0, 0, 0, "");
  }
  sim.setRngSeed(SEED);

  // -- simulating ---------------------------------------------------------
  progress("simulating");
  const initialSnapshot: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());

  const lever = initialSnapshot.find((b) => b.state.startsWith("minecraft:lever"));
  if (!lever) throw new Error("no lever found — the door must include its lever");
  const [lx, ly, lz] = lever.pos;

  // Settle any placement cascade, then run one full conditioning cycle
  // (open + close) before taking the reference snapshot: a schematic can
  // be saved with its panels anywhere, and doors are certified against
  // their steady-state cycle, not the accident of how they were saved.
  const flips: LeverFlip[] = [];
  const flip = (label: string, measured: boolean) => {
    flips.push({ tick: sim.tickCount(), label, measured });
    sim.useBlock(lx, ly, lz);
  };
  sim.runUntilQuiescent(200);
  flip("conditioning open", false);
  sim.runUntilQuiescent(300);
  flip("conditioning close", false);
  sim.runUntilQuiescent(300);
  const snapshotA = snapshotKey(JSON.parse(sim.worldSnapshotJson()));

  // -- measuring ----------------------------------------------------------
  progress("measuring");
  const tOpenClick: number = sim.tickCount();
  flip("lever on", true);
  sim.runUntilQuiescent(300);
  let changes: ReplayChange[] = JSON.parse(sim.changesJson());
  const lastOpen = lastChangeTickAfter(changes, tOpenClick);
  if (lastOpen === null) throw new Error("lever click caused no block changes");
  const openTicks = lastOpen - tOpenClick;

  const tCloseClick: number = sim.tickCount();
  flip("lever off", true);
  sim.runUntilQuiescent(300);
  changes = JSON.parse(sim.changesJson());
  const lastClose = lastChangeTickAfter(changes, tCloseClick);
  const closeTicks = lastClose !== null ? lastClose - tCloseClick : 0;

  const snapshotFinal = snapshotKey(JSON.parse(sim.worldSnapshotJson()));
  const verdict = snapshotFinal === snapshotA ? "CERTIFIED" : "DID NOT RESET";

  const simTicks: number = sim.tickCount();
  const summary: TickEvents[] = JSON.parse(sim.eventsSummaryJson());

  // Stats: events per tick, zero-filled over the whole run.
  const byTick = new Map(summary.map((r) => [r.tick, r]));
  const eventsPerTick: TickEvents[] = Array.from({ length: simTicks }, (_, t) => ({
    tick: t,
    piston: byTick.get(t)?.piston ?? 0,
    redstone: byTick.get(t)?.redstone ?? 0,
    changes: byTick.get(t)?.changes ?? 0,
  }));

  // Heatmap: per-(x,y) column counts of block changes across the run.
  const values = Array.from({ length: h }, () => new Array<number>(w).fill(0));
  for (const c of changes) {
    const [x, y] = c.pos;
    if (x >= 0 && x < w && y >= 0 && y < h) values[y][x] += 1;
  }

  // Materials: base-name counts from the initial snapshot, air excluded.
  const counts = new Map<string, number>();
  for (const b of initialSnapshot) {
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
      materials,
      events_per_tick: eventsPerTick,
      heatmap: { w, h, values },
      sim_ticks: simTicks,
      seed: Number(SEED),
      verdict,
    },
    replay: { blocks: initialSnapshot, changes, simTicks, flips },
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
