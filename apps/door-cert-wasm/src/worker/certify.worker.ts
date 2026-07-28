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
  ResetTime,
  TickEvents,
  Vec3,
  WorkerJob,
  WorkerMessage,
} from "../lib/types";
import { aperture } from "../lib/aperture";
import { decodeXray, xrayTransferables, type XrayData } from "../lib/xray";

const SEED = 12345n;
/** How far the reset search walks before giving up and saying so. */
const RESET_CAP = 200;
/** Ticks allowed for a trial to go quiet again. */
const SETTLE_BUDGET = 400;
/** Wall-clock guard on the reset search, so a pathological door reports
 *  "not found within N tried" instead of hanging the worker. */
const RESET_BUDGET_MS = 25_000;

const post = (m: WorkerMessage, transfer?: Transferable[]) =>
  (self as unknown as Worker).postMessage(m, transfer ?? []);
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

/** A question asked of the running world after every tick of a stroke. */
type Probe = { key: string; test: () => boolean };

/** Replay a slice of the recorded change log forward over a running world and
 *  report the elapsed tick at which each probe first holds.
 *
 *  Replayed rather than re-simulated: the log already carries every write the
 *  engine made, so walking it costs nothing and reads the doorway at exactly
 *  the tick it cleared instead of at the tick the machine stopped fidgeting.
 *
 *  Elapsed counting: `useBlock` is a boundary action — it fires between level
 *  ticks, so the writes it causes directly (the lever, and whatever redstone
 *  propagation reaches on the spot) happen with no tick having run and count
 *  as 0. Everything after is `tick - clickTick + 1`, which is the number of
 *  game ticks that had to run. That is also what the activity chart calls
 *  "quiet at t=N · N ticks", so the two now agree by construction. */
function replayStroke(
  world: Map<string, string>,
  changes: ReplayChange[],
  from: number,
  to: number,
  boundary: number,
  clickTick: number,
  leverKey: string,
  probes: Probe[],
): { hits: Map<string, number>; latency: number | null } {
  const hits = new Map<string, number>();
  const check = (elapsed: number) => {
    for (const p of probes) if (!hits.has(p.key) && p.test()) hits.set(p.key, elapsed);
  };
  const elapsedOf = (i: number) => (i < boundary ? 0 : changes[i].tick - clickTick + 1);
  let latency: number | null = null;
  check(0);
  let i = from;
  while (i < to) {
    const e = elapsedOf(i);
    while (i < to && elapsedOf(i) === e) {
      const c = changes[i];
      const key = posKey(c.pos);
      if (isAir(c.to)) world.delete(key);
      else world.set(key, c.to);
      if (latency === null && key !== leverKey) latency = e;
      i++;
    }
    check(e);
  }
  return { hits, latency };
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

async function certify(job: WorkerJob): Promise<{ record: CertRecord; xray: XrayData | null }> {
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
    /** Checkpoint of the world the cycle starts from — the base every reset
     *  trial is restored to. */
    cp: number;
    tOpen: number;
    tClose: number;
    /** Change-log lengths: at each click, immediately after it (so the
     *  boundary writes can be told from the first tick's), and at the end. */
    iOpen: number;
    iOpenPost: number;
    iClose: number;
    iClosePost: number;
    iEnd: number;
    openBlocks: ReplayBlock[];
    endBlocks: ReplayBlock[];
  };
  const runCycle = (): Cycle => {
    const cp: number = sim.checkpoint();
    const tOpen: number = sim.tickCount();
    const iOpen: number = sim.changesCount();
    sim.useBlock(lx, ly, lz);
    const iOpenPost: number = sim.changesCount();
    sim.runUntilQuiescent(300);
    const openBlocks: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());
    const tClose: number = sim.tickCount();
    const iClose: number = sim.changesCount();
    sim.useBlock(lx, ly, lz);
    const iClosePost: number = sim.changesCount();
    sim.runUntilQuiescent(300);
    const endBlocks: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());
    const iEnd: number = sim.changesCount();
    return {
      cp,
      tOpen,
      tClose,
      iOpen,
      iOpenPost,
      iClose,
      iClosePost,
      iEnd,
      openBlocks,
      endBlocks,
    };
  };

  // -- measuring ----------------------------------------------------------
  progress("measuring");
  let restBlocks: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());
  let restKey = snapshotKey(restBlocks);
  const savedRest = restBlocks;
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
    cycle = runCycle();
  }
  // The change log is cumulative and already holds the placement writes, so
  // the measured cycle is everything appended from the last cycle's first
  // click on — a count, not a tick filter, because settling can complete
  // without advancing the clock.
  const tRebase = cycle.tOpen;
  const logBase = cycle.iOpen;
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

  if (allChanges.every((c) => posKey(c.pos) === leverKey))
    throw new Error("lever click caused no block changes");

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
  const geo = analysis?.geometry ?? null;
  const parts = census(restBlocks);

  // -- doorway timing -----------------------------------------------------
  // The old open/close numbers were the last tick carrying ANY change, which
  // is when the machine goes quiet, not when the door opened. A door whose
  // panels clear in 8 ticks and whose tape shuffles for another 12 reported
  // 20. Now that the passage is known cell by cell, both are measured off it:
  //   open  — the first tick at which every passage cell is air.
  //   close — the first tick at which every cell the pattern fills when shut
  //           is solid again. Not "the passage is full": a sissy bar or a
  //           checkerboard never fills its own doorway and never would.
  // Settle is kept, under its own name, because the gap between the two is a
  // real property of a door.
  const world = new Map<string, string>();
  for (const b of restBlocks) world.set(posKey(b.pos), b.state);
  const solid = (k: string) => {
    const s = world.get(k);
    return s !== undefined && !isAir(s);
  };
  const passageKeys = geo ? geo.passage.map(posKey) : [];
  const patternKeys = geo ? geo.closed.map(posKey) : [];
  const probes = (): Probe[] =>
    geo
      ? [
          { key: "clear", test: () => passageKeys.every((k) => !solid(k)) },
          { key: "shut", test: () => patternKeys.every((k) => solid(k)) },
        ]
      : [];

  const rel = (i: number) => i - logBase;
  const strokeA = replayStroke(
    world,
    allChanges,
    0,
    rel(cycle.iClose),
    rel(cycle.iOpenPost),
    cycle.tOpen,
    leverKey,
    probes(),
  );
  const strokeB = replayStroke(
    world,
    allChanges,
    rel(cycle.iClose),
    rel(cycle.iEnd),
    rel(cycle.iClosePost),
    cycle.tClose,
    leverKey,
    probes(),
  );

  // A file saved with the door already standing open measures its own closing
  // first, so the two strokes swap. `aperture()` knows which snapshot holds
  // the door blocks; take the answer from there rather than assuming.
  const restIsClosed = geo?.restIsClosed ?? true;
  const openStroke = restIsClosed ? strokeA : strokeB;
  const closeStroke = restIsClosed ? strokeB : strokeA;
  const openTicks = openStroke.hits.get("clear") ?? null;
  const closeTicks = closeStroke.hits.get("shut") ?? null;
  const openLatency = openStroke.latency;
  const closeLatency = closeStroke.latency;

  // Settle, read off the same per-tick series the activity chart draws, so
  // the "quiet at t=N" marker and this number cannot drift apart.
  const summary: TickEvents[] = JSON.parse(sim.eventsSummaryJson());
  const settleOf = (from: number, to: number): number | null => {
    let last = -1;
    for (const r of summary) {
      if (r.tick < from || r.tick >= to) continue;
      // The engine omits series it does not report, so every term is
      // defaulted — the chart does the same, and one NaN here would silently
      // erase the whole stat.
      if ((r.piston ?? 0) + (r.redstone ?? 0) + (r.items ?? 0) > 0) last = r.tick;
    }
    return last < 0 ? null : last + 1 - from;
  };
  const settleA = settleOf(cycle.tOpen, cycle.tClose);
  const settleB = settleOf(cycle.tClose, Infinity);
  const openSettle = restIsClosed ? settleA : settleB;
  const closeSettle = restIsClosed ? settleB : settleA;

  const timingNotes: string[] = [];
  if (!geo)
    timingNotes.push(
      "no walkable passage was extracted, so the doorway could not be timed",
    );
  else {
    if (openTicks === null)
      timingNotes.push("the passage never fully cleared on the opening stroke");
    if (closeTicks === null)
      timingNotes.push("the pattern never fully re-filled on the closing stroke");
  }
  const timingNote = timingNotes.length ? timingNotes.join("; ") : null;

  // -- reset time ---------------------------------------------------------
  // Purplers' algorithm: there is no closed form, so it is measured. From the
  // state the cycle starts in, click the lever, wait X ticks, click it back,
  // let the machine settle, and ask whether the world is exactly where it
  // started. The smallest X that works is the reset time.
  //
  // One extra condition, which the bare algorithm needs: the door must
  // actually have completed the stroke somewhere in the trial. Without it
  // X = 0 passes for every door on earth — two clicks in the same instant
  // cancel, nothing runs, and the world trivially matches. With it, a reset
  // shorter than the stroke means the machine took the re-trigger MID-stroke
  // and finished anyway, which is exactly the rare property section 2b is
  // after.
  progress("reset");
  const cellSolid = (p: Vec3) => !isAir(sim.getBlock(p[0], p[1], p[2]));
  const searchReset = (
    cp: number,
    base: string,
    reached: () => boolean,
  ): { ticks: number | null; searched: number } => {
    const deadline = Date.now() + RESET_BUDGET_MS;
    let searched = 0;
    for (let x = 0; x <= RESET_CAP; x++) {
      searched = x;
      sim.restore(cp);
      let done = false;
      const watch = () => {
        if (!done && reached()) done = true;
      };
      sim.useBlock(lx, ly, lz);
      watch();
      for (let i = 0; i < x; i++) {
        sim.step();
        watch();
      }
      sim.useBlock(lx, ly, lz);
      watch();
      for (let i = 0; i < SETTLE_BUDGET && !sim.isQuiescent(); i++) {
        sim.step();
        watch();
      }
      // The snapshot is the expensive half of a trial, so it is only taken
      // once the cheap condition has already passed.
      if (done && sim.worldSnapshotJson() === base) return { ticks: x, searched: x };
      if (Date.now() > deadline) break;
    }
    return { ticks: null, searched };
  };

  let resetRest: ResetTime | null = null;
  let resetOther: ResetTime | null = null;
  const resetSkip =
    !geo
      ? "not measured — there is no passage to check the stroke against"
      : verdict !== "CERTIFIED"
        ? "not measured — the door does not return to its own resting state"
        : null;
  if (!resetSkip && geo) {
    const clearNow = () => geo.passage.every((p) => !cellSolid(p));
    const shutNow = () => geo.closed.every((p) => cellSolid(p));
    // From the resting state the first click runs one stroke; from the state
    // it settles into, the other. Which is which follows `restIsClosed`.
    sim.restore(cycle.cp);
    const restSnap: string = sim.worldSnapshotJson();
    const a = searchReset(cycle.cp, restSnap, restIsClosed ? clearNow : shutNow);

    sim.restore(cycle.cp);
    sim.useBlock(lx, ly, lz);
    sim.runUntilQuiescent(SETTLE_BUDGET);
    const cpOther: number = sim.checkpoint();
    const otherSnap: string = sim.worldSnapshotJson();
    const b = searchReset(cpOther, otherSnap, restIsClosed ? shutNow : clearNow);

    const wrap = (
      r: { ticks: number | null; searched: number },
      stroke: number | null,
    ): ResetTime => ({
      ticks: r.ticks,
      searched: r.searched,
      stroke_ticks: stroke,
      negative: r.ticks !== null && stroke !== null && r.ticks < stroke,
      note:
        r.ticks === null
          ? `no delay up to ${r.searched} ticks brought the door back to this state`
          : null,
    });
    resetRest = wrap(a, restIsClosed ? openTicks : closeTicks);
    resetOther = wrap(b, restIsClosed ? closeTicks : openTicks);
  }
  const blankReset = (): ResetTime => ({
    ticks: null,
    searched: 0,
    stroke_ticks: null,
    negative: false,
    note: resetSkip,
  });
  const resetOpen = restIsClosed ? resetRest : resetOther;
  const resetClose = restIsClosed ? resetOther : resetRest;

  // The true safe re-trigger period is the pair of reset times, not the pair
  // of stroke times: a door can open in 8 ticks and still refuse the lever
  // for 40. Fall back to the doorway cycle only when the resets are unknown.
  const doorwayCycle =
    openTicks !== null && closeTicks !== null ? openTicks + closeTicks : null;
  const resetCycle =
    resetOpen?.ticks != null && resetClose?.ticks != null
      ? resetOpen.ticks + resetClose.ticks
      : null;
  const cycleTicks = resetCycle ?? doorwayCycle ?? (openSettle ?? 0) + (closeSettle ?? 0);
  const cyclesPerMinute = cycleTicks > 0 ? 1200 / cycleTicks : 0;


  // Everything below describes the MEASURED cycle only: the change log,
  // the activity trace and the replay are all rebased so tick 0 is the
  // moment the lever is clicked open.
  const simTicks = endTick - tRebase;
  const changes: ReplayChange[] = allChanges
    .filter((c) => c.tick >= tRebase)
    .map((c) => ({ ...c, tick: c.tick - tRebase }));
  // Labelled by effect, not by lever position: a file saved with its door
  // standing open has its first click CLOSE the thing, and calling that
  // "opens" would make every annotation on the replay a lie.
  const flips: LeverFlip[] = [
    {
      tick: cycle.tOpen - tRebase,
      label: restIsClosed ? "lever thrown — opens" : "lever thrown — closes",
      measured: true,
    },
    {
      tick: cycle.tClose - tRebase,
      label: restIsClosed ? "lever back — closes" : "lever back — opens",
      measured: true,
    },
  ];

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

  // -- x-ray: the update stream ------------------------------------------
  // Recording is off for everything above (it costs nothing when off, but a
  // reset search runs the cycle a hundred times and would record all of it).
  // So the measured cycle is replayed once more from its own checkpoint —
  // same seed, same starting state, same trace — with the recorder on.
  //
  // The two DRAWABLE views are pulled here and the 15.8 MB raw log never is:
  // the heat view (~900 KB/cycle) drives playback and the wave view (~310 KB
  // for the busiest tick) drives sub-tick stepping. Every tick's wave is
  // precomputed rather than fetched on demand — the whole cycle's waves cost
  // ~30 ms and ~1.3 MB packed, which is cheaper than keeping a wasm world
  // alive for the life of the page.
  progress("x-ray");
  let xray: XrayData | null = null;
  try {
    sim.restore(cycle.cp);
    const xBase: number = sim.tickCount();
    sim.recordUpdates(true);
    sim.useBlock(lx, ly, lz);
    sim.runUntilQuiescent(300);
    sim.useBlock(lx, ly, lz);
    sim.runUntilQuiescent(300);
    // Read BEFORE switching the recorder off: `record_updates(false)` drops
    // the log outright (`upd_log = None`), so disabling first returns an
    // empty trace rather than the one just recorded.
    xray = decodeXray(
      sim.updatesHeatJson(xBase, xBase + simTicks),
      (t: number) => sim.updatesWaveJson(t),
      xBase,
      simTicks,
    );
    sim.recordUpdates(false);
  } catch (e) {
    // A door whose trace does not fit is still a certified door; the replay
    // simply offers no x-ray.
    console.warn("[xray] update recording failed", e);
    xray = null;
  }

  const record: CertRecord = {
    certificate: {
      name: job.name,
      dims: [w, h, l] as Vec3,
      lever: [lx, ly, lz] as Vec3,
      open_ticks: openTicks,
      close_ticks: closeTicks,
      open_settle_ticks: openSettle,
      close_settle_ticks: closeSettle,
      timing_note: timingNote,
      open_latency: openLatency,
      close_latency: closeLatency,
      reset_open: resetOpen ?? blankReset(),
      reset_close: resetClose ?? blankReset(),
      materials,
      events_per_tick: eventsPerTick,
      heatmap: { w, h, values },
      sim_ticks: simTicks,
      seed: Number(SEED),
      verdict,
      moved_cells: movedCells,
      aperture: doorway,
      aperture_geometry: geo,
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
      rest_is_closed: restIsClosed,
    },
    replay: { blocks: restBlocks, changes, simTicks, flips },
  };
  return { record, xray };
}

self.onmessage = async ({ data }: MessageEvent<WorkerJob>) => {
  try {
    const { record, xray } = await certify(data);
    post({ type: "done", record, xray }, xray ? xrayTransferables(xray) : []);
  } catch (e) {
    post({ type: "error", error: e instanceof Error ? e.message : String(e) });
  }
};
