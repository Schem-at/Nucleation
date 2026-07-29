// Certification pipeline, ported from apps/door-cert/backend/main.py
// `process()` — but running entirely in this module worker on the wasm
// engine served from public/engine/. Only progress + the finished record
// cross the worker boundary; the UI thread never blocks on the sim.
import type {
  ApertureConflict,
  ApertureReading,
  Census,
  CertRecord,
  InputControl,
  LeverFlip,
  Material,
  ReplayBlock,
  ReplayChange,
  ResetTime,
  TickEvents,
  Vec3,
  Verdict,
  WorkerJob,
  WorkerMessage,
} from "../lib/types";
import { aperture } from "../lib/aperture";
import { engineering, isMovement } from "../lib/engineering";
import { decodeXray, xrayTransferables, type XrayData } from "../lib/xray";

const SEED = 12345n;
/** How far the reset search walks before giving up and saying so. */
const RESET_CAP = 200;
/** Ticks allowed for a trial to go quiet again. */
const SETTLE_BUDGET = 400;
/** Wall-clock guard on the reset search, so a pathological door reports
 *  "not found within N tried" instead of hanging the worker. */
const RESET_BUDGET_MS = 25_000;

/** ------------------------------------------------------------ size guard --
 *
 *  The simulator builds a full block-entity world before it can be asked
 *  anything, so an oversized file is not slow — it is an out-of-memory trap
 *  that takes the tab with it. `IRIS_B.schem` is 1.4 MB on disk and 9,654,922
 *  blocks in a 499 × 379 × 442 volume; nothing about the download says so.
 *  Both numbers are checked, because a sparse world export blows the volume
 *  and a dense one blows the block count.
 *
 *  The ceilings are deliberately far above any door. The pattern standard tops
 *  out at 32 × 32, and the largest door in the corpus is a 6 × 6 at 791 blocks
 *  in a 1,200-cell volume — so a 250k-block, 2M-cell limit leaves two orders of
 *  magnitude of headroom and still refuses a world. */
const MAX_BLOCKS = 250_000;
const MAX_VOLUME = 2_000_000;

/** ------------------------------------------------- input detection budget --
 *  Ticks a single actuation trial is allowed before it is called quiet, and the
 *  wall-clock ceiling on the whole search across every candidate. */
const DETECT_TICKS = 200;
const DETECT_BUDGET_MS = 20_000;

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

/** Refuse a build that is plainly not a door, before anything expensive. The
 *  message names the numbers it refused on, because "too large" on its own
 *  tells nobody whether they picked the wrong file or hit an arbitrary cap. */
function guardSize(name: string, volume: number, blocks: number | null) {
  const n = (v: number) => v.toLocaleString("en-US");
  if (blocks !== null && blocks > MAX_BLOCKS)
    throw new Error(
      `SIZE: "${name}" is ${n(blocks)} blocks. That is a world export, not a door — ` +
        `the simulator would run out of memory building it. Cut the door out and upload that.`,
    );
  if (volume > MAX_VOLUME)
    throw new Error(
      `SIZE: "${name}" spans ${n(volume)} cells. That is a world export, not a door — ` +
        `the simulator would run out of memory building it. Cut the door out and upload that.`,
    );
}

/** ------------------------------------------------------ input detection --
 *
 * Which block opens the door is MEASURED, never guessed.
 *
 * The old rule was "find `minecraft:lever`", which fails on every button, plate
 * and note-block door, and would fail worse if it were widened by guesswork:
 * levers and buttons are also the ordinary way to REDIRECT redstone inside a
 * build, so the presence of one proves nothing about what drives the door. A
 * 6 × 6 in the corpus carries ten note blocks and one lever; a static rule that
 * picked the rarest block would pick the lever there and be wrong the moment a
 * builder used two.
 *
 * So Purplers' brute force: take every block a player could actuate — levers,
 * buttons of any wood or stone, note blocks, pressure plates — checkpoint the
 * world, actuate it, run to quiescence, and ask whether A PASSAGE OPENED. Not
 * "did anything move": moving something is far too weak a test. Two note blocks
 * in `fast tgm 4x4` do move something — playing one BUDs a piston beside it and
 * four cells shuffle — and calling those inputs would put three drivers on a
 * one-lever door. The question the door answers is whether you can now walk
 * through it, so that is the question asked, using the same `aperture()` that
 * measures the doorway everywhere else.
 *
 * Two controls that open the SAME passage are two real inputs and the sheet
 * says so. A control that opens a different one-cell hole in the far corner is
 * a side effect of the mechanism, not a second front door, and is dropped.
 *
 * Rarity within the build orders the queue, so the odd-one-out is usually tried
 * first. It is a speed hint and nothing else: every candidate is still
 * actuated, because a static guess cannot work — levers and buttons are also
 * the ordinary way to redirect redstone inside a build.
 *
 * The trial runs on a THROWAWAY simulation built from the same schematic. The
 * change log and the per-tick event summary are cumulative and tick-indexed, so
 * a hundred speculative strokes on the measured world would poison both. */
type Detection = {
  input: InputControl | null;
  alternatives: InputControl[];
  /** Everything actuated, in the order it was tried. */
  tested: number;
  note: string | null;
};

/** A piston mid-stroke leaves this placeholder in the cell it is moving
 *  through. It is never a resting block, so a diff that finds only these has
 *  not moved anything yet — and counting them made two note blocks in the 6 × 6
 *  look like inputs because playing a note BUDs a piston two cells away. */
const MOVING_PISTON = "minecraft:moving_piston";

const inputKindOf = (state: string): InputControl["kind"] | null => {
  const n = baseName(state).replace(/^minecraft:/, "");
  if (n === "lever") return "lever";
  if (n === "note_block") return "note block";
  if (n.endsWith("_button")) return "button";
  if (n.endsWith("_pressure_plate")) return "pressure plate";
  return null;
};

/** The world as base block name per cell — power levels, repeater locks and
 *  note pitches all collapse away, so a diff of two of these counts MASS that
 *  moved and nothing else. */
function massMap(sim: any): Map<string, string> {
  const m = new Map<string, string>();
  for (const b of JSON.parse(sim.worldSnapshotJson()) as ReplayBlock[])
    m.set(posKey(b.pos), baseName(b.state));
  return m;
}

function massDiff(a: Map<string, string>, b: Map<string, string>): number {
  let n = 0;
  for (const [k, v] of b) {
    if (v === MOVING_PISTON || a.get(k) === MOVING_PISTON) continue;
    if (a.get(k) !== v) n++;
  }
  for (const k of a.keys()) {
    if (b.has(k) || a.get(k) === MOVING_PISTON) continue;
    n++;
  }
  return n;
}

/** A plate has no player to stand on it, so it is driven by writing its own
 *  powered state; everything else is right-clicked the way a player would. */
function actuateOnce(sim: any, c: InputControl) {
  const [x, y, z] = c.pos;
  if (c.kind === "pressure plate") {
    const s: string = sim.getBlock(x, y, z);
    const next = s.includes("powered=")
      ? s.replace(/powered=(true|false)/, (_m, v) => `powered=${v === "true" ? "false" : "true"}`)
      : s.replace(/power=(\d+)/, (_m, v) => `power=${Number(v) > 0 ? 0 : 15}`);
    sim.placeBlock(x, y, z, next);
  } else sim.useBlock(x, y, z);
}

function detectInputs(makeSim: () => any, settled: ReplayBlock[]): Detection {
  const counts = new Map<string, number>();
  for (const b of settled) {
    const n = baseName(b.state);
    counts.set(n, (counts.get(n) ?? 0) + 1);
  }
  const candidates = settled
    .map((b) => ({ b, kind: inputKindOf(b.state) }))
    .filter((c): c is { b: ReplayBlock; kind: InputControl["kind"] } => c.kind !== null)
    // Rarity first: the odd-one-out in a build is the likeliest control, so the
    // search usually stops on its first trial. Ties break on kind — a lever is
    // a control by trade, a note block is an instrument — and then on position
    // so the order is deterministic across runs.
    .sort((p, q) => {
      const cp = counts.get(baseName(p.b.state))! - counts.get(baseName(q.b.state))!;
      if (cp !== 0) return cp;
      const rank = (k: InputControl["kind"]) =>
        k === "lever" ? 0 : k === "button" ? 1 : k === "pressure plate" ? 2 : 3;
      return rank(p.kind) - rank(q.kind) || posKey(p.b.pos).localeCompare(posKey(q.b.pos));
    });

  if (candidates.length === 0)
    return {
      input: null,
      alternatives: [],
      tested: 0,
      note:
        "No control to actuate. The build has no lever, button, pressure plate " +
        "or note block anywhere in it, so nothing here can be thrown — include " +
        "the control that drives the door.",
    };

  const sim = makeSim();
  sim.setRngSeed(SEED);
  sim.runUntilQuiescent(200);
  const atRest = massMap(sim);
  const deadline = Date.now() + DETECT_BUDGET_MS;
  /** Every control that opened a walkable passage, with the passage it opened. */
  const opened: { control: InputControl; passage: Set<string> }[] = [];
  let movedButOpenedNothing = 0;
  let tested = 0;

  for (const c of candidates) {
    const cp: number = sim.checkpoint();
    const control: InputControl = {
      pos: c.b.pos,
      state: c.b.state,
      kind: c.kind,
      moved: 0,
      momentary: false,
    };
    actuateOnce(sim, control);
    // The world at PEAK travel, not at the end: a button springs back on its
    // own, so a door it drives stands fully open mid-trial and is shut again by
    // the time the world goes quiet. Reading only the end would score that door
    // zero and lose every momentary input there is.
    let peak = massDiff(atRest, massMap(sim));
    let peakWorld: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());
    const sample = () => {
      const d = massDiff(atRest, massMap(sim));
      if (d > peak) {
        peak = d;
        peakWorld = JSON.parse(sim.worldSnapshotJson());
      }
    };
    for (let i = 0; i < DETECT_TICKS && !sim.isQuiescent(); i++) {
      sim.step();
      if (i < 24 || i % 4 === 0) sample();
    }
    sample();
    control.moved = peak;
    // A control that is back in the state it started in, with no second
    // actuation, released itself: that is what makes a button momentary and a
    // lever not.
    const [x, y, z] = c.b.pos;
    control.momentary = c.kind !== "lever" && sim.getBlock(x, y, z) === c.b.state;
    tested++;
    sim.restore(cp);

    if (peak > 0) {
      // The actual question: is there something to walk through now?
      const geo = peak > 0 ? aperture(settled, peakWorld)?.geometry : null;
      if (geo && geo.passage.length > 0)
        opened.push({ control, passage: new Set(geo.passage.map(posKey)) });
      else movedButOpenedNothing++;
    }
    if (Date.now() > deadline) break;
  }

  const budgetNote =
    tested < candidates.length
      ? ` The search stopped after ${tested} of ${candidates.length} controls — it ran out of time, not out of candidates.`
      : "";

  if (opened.length === 0) {
    const kinds = [...new Set(candidates.map((c) => c.kind))].join(", ");
    return {
      input: null,
      alternatives: [],
      tested,
      note:
        movedButOpenedNothing > 0
          ? `Nothing opened. All ${tested} control${tested === 1 ? "" : "s"} in this build ` +
            `(${kinds}) were actuated one at a time; ${movedButOpenedNothing} of them moved ` +
            `blocks but none opened a passage anyone could walk through, so there is no ` +
            `doorway here to time.` + budgetNote
          : `Nothing moved. All ${tested} control${tested === 1 ? "" : "s"} in this build ` +
            `(${kinds}) were actuated one at a time and none of them shifted a single ` +
            `block, so this build has no door to time — or its driver is outside the file.` +
            budgetNote,
    };
  }

  // The widest passage leads. A second control counts as a second INPUT only if
  // it opens the same doorway; one that opens a different hole somewhere else
  // in the build is the mechanism talking, not a front door.
  opened.sort(
    (a, b) =>
      b.passage.size - a.passage.size ||
      b.control.moved - a.control.moved ||
      posKey(a.control.pos).localeCompare(posKey(b.control.pos)),
  );
  const winner = opened[0];
  const alternatives = opened
    .slice(1)
    .filter((o) => {
      let shared = 0;
      for (const k of o.passage) if (winner.passage.has(k)) shared++;
      return shared >= o.passage.size / 2;
    })
    .map((o) => o.control);
  const input = winner.control;
  const notes: string[] = [];
  if (alternatives.length)
    notes.push(
      `${alternatives.length + 1} controls open this same doorway independently. The cycle ` +
        `below was measured on the ${input.kind} at (${input.pos.join(", ")}); the ${
          alternatives.length === 1 ? "other is" : "others are"
        } ${alternatives.map((a) => `a ${a.kind} at (${a.pos.join(", ")})`).join(", ")}.`,
    );
  if (input.kind !== "lever")
    notes.push(
      `Driven by a ${input.kind}, not a lever — found by actuating every control in the ` +
        `build and watching for a passage to open.` +
        (input.momentary
          ? ` It is momentary: it releases itself, so the return stroke is the control's own, not a second throw.`
          : ""),
    );
  if (budgetNote) notes.push(budgetNote.trim());
  return { input, alternatives, tested, note: notes.length ? notes.join(" ") : null };
}

/** --------------------------------------------------------- engine errors --
 *
 * The engine throws `NucleationError.Simulation` and little else. That is a
 * type name, not a sentence, and it reached the upload screen verbatim on 20 of
 * 28 doors in the last batch run. Each known kind gets a sentence that says
 * what happened and what to do; the raw name is kept beside it as small print
 * so a screenshot is still triageable, and `detail` carries whatever the engine
 * could name — once it names the offending block, that block appears here with
 * no further change. */
function humanError(e: unknown): { message: string; code: string | null; detail: string | null } {
  const raw = e instanceof Error ? e.message : String(e);
  const m = raw.match(/NucleationError\.(\w+)\s*[:\-—]?\s*([\s\S]*)$/);
  if (!m) return { message: raw, code: null, detail: null };
  const kind = m[1];
  const detail = m[2].trim() || null;
  const tail = detail ? ` ${detail}` : "";
  switch (kind) {
    case "Simulation":
      return {
        message:
          "The redstone engine will not load this build: it contains a block the simulator " +
          "does not model yet, and one is enough to stop the whole schematic." + tail,
        code: `NucleationError.${kind}`,
        detail,
      };
    case "Parse":
      return {
        message:
          "This file could not be read as a schematic. It may be a newer save format than " +
          "the parser knows, or the download may be truncated." + tail,
        code: `NucleationError.${kind}`,
        detail,
      };
    case "Io":
      return {
        message: "The file could not be read off disk. Try the upload again." + tail,
        code: `NucleationError.${kind}`,
        detail,
      };
    default:
      return {
        message:
          "The engine stopped on this build and did not say why in words we can pass on." +
          tail,
        code: `NucleationError.${kind}`,
        detail,
      };
  }
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
  let makeSim: () => any;
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
    guardSize(job.name, w * h * l, null);
    makeSim = () => TickSimulation.fromSnbt(snbt, TickSettleMode.InWorld, 0, 0, 0, "");
    sim = makeSim();
  } else {
    const schem = Schematic.fromData(Array.from(bytes));
    const dims = schem.dimensions();
    [w, h, l] = [dims.x, dims.y, dims.z];
    // Parsing is cheap and answers the only question that matters here; it is
    // `fromSchematic` that materialises a world and takes the tab with it. So
    // the size is checked between the two.
    guardSize(job.name, w * h * l, typeof schem.blockCount === "function" ? schem.blockCount() : null);
    makeSim = () => TickSimulation.fromSchematic(schem, TickSettleMode.InWorld, 0, 0, 0, "");
    sim = makeSim();
  }
  sim.setRngSeed(SEED);

  // -- simulating ---------------------------------------------------------
  progress("simulating");
  // Settle the placement cascade in-world, then measure exactly one cycle:
  // open, close. If the door lands back where it started, that was its
  // steady state and there is nothing more to prove. Only a door saved
  // mid-cycle needs the extra conditioning pass, and it says so on the
  // certificate rather than padding every replay with four lever flips.
  sim.runUntilQuiescent(200);
  const settledSnapshot: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());

  progress("input");
  const detection = detectInputs(makeSim, settledSnapshot);
  if (!detection.input) throw new Error(`INPUT: ${detection.note}`);
  const lever = detection.input;
  const [lx, ly, lz] = lever.pos;
  const throwInput = () => actuateOnce(sim, lever);

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
    throwInput();
    const iOpenPost: number = sim.changesCount();
    sim.runUntilQuiescent(300);
    const openBlocks: ReplayBlock[] = JSON.parse(sim.worldSnapshotJson());
    const tClose: number = sim.tickCount();
    const iClose: number = sim.changesCount();
    throwInput();
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
  /** The first lap, kept for the cross-check below: it is the only cycle whose
   *  closed reference is the state the FILE was saved in. */
  const savedCycle = cycle;
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
    throw new Error(`INPUT: The ${lever.kind} moved nothing but itself.`);

  let verdict: Verdict =
    snapshotKey(cycle.endBlocks) === restKey ? "CERTIFIED" : "DID NOT RESET";

  // How much of the build actually travels on a stroke — a door that merely
  // returns to its reference state has proven nothing (a dead machine does
  // that too). This is the number that separates a working door from a
  // crippled one.
  const movedCells = symmetricDiff(restKey, snapshotKey(cycle.openBlocks));

  // The doorway itself, and how much of the build it costs to get it.
  progress("classifying");
  // Measured off the SETTLED cycle — the same pair of states the strokes below
  // are timed against — so the picture, the timings and the classification are
  // all one reading of one cycle.
  const analysis = aperture(restBlocks, cycle.openBlocks);
  const doorway = analysis?.aperture ?? null;
  const geo = analysis?.geometry ?? null;
  const parts = census(restBlocks);

  // ------------------------------------------------- is it the same door? --
  //
  // Priming is not free. It runs the machine onto a cycle it repeats, which is
  // the only cycle worth timing — but there is no guarantee that cycle opens
  // the doorway the file was saved with. `fast 4x4 vault door` is the proof:
  // its saved state is a closed 4 × 4 vault, and after two laps it settles into
  // a cycle in which twelve of those sixteen cells never come back and only a
  // 4 × 1 sliver still travels. Measure the settled cycle and the answer is a
  // 4 × 1 door opening in one tick. Both readings are honest measurements of
  // different things, and the run has no way to tell which one the builder
  // meant — its identical twin with the barrels filled needs no priming at all
  // and reads 4 × 4 throughout.
  //
  // So both are taken, and when they disagree the tool says so and declines to
  // name a pattern. A number nobody can trust is worse than no number.
  const reading = (a: ReturnType<typeof aperture>): ApertureReading | null =>
    a
      ? {
          w: a.aperture.w,
          h: a.aperture.h,
          cells: a.aperture.cells,
          depth: a.aperture.depth,
          name: a.classification?.name ?? null,
        }
      : null;
  let apertureConflict: ApertureConflict | null = null;
  if (neededPriming) {
    const savedReading = reading(aperture(savedRest, savedCycle.openBlocks));
    const settledReading = reading(analysis);
    if (
      savedReading &&
      settledReading &&
      (savedReading.cells !== settledReading.cells ||
        savedReading.w !== settledReading.w ||
        savedReading.h !== settledReading.h)
    ) {
      apertureConflict = {
        saved: savedReading,
        settled: settledReading,
        drift: savedStateDrift,
      };
      verdict = "INCONCLUSIVE";
    }
  }

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
      throwInput();
      watch();
      for (let i = 0; i < x; i++) {
        sim.step();
        watch();
      }
      throwInput();
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
      : verdict === "DID NOT RESET"
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
    throwInput();
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
  const verb = lever.kind === "lever" ? ["thrown", "back"] : ["pressed", "pressed again"];
  const flips: LeverFlip[] = [
    {
      tick: cycle.tOpen - tRebase,
      label: `${lever.kind} ${verb[0]} — ${restIsClosed ? "opens" : "closes"}`,
      measured: true,
    },
    {
      tick: cycle.tClose - tRebase,
      label: `${lever.kind} ${verb[1]} — ${restIsClosed ? "closes" : "opens"}`,
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
  // movement at all. `isMovement` lives in lib/engineering.ts because the
  // dead-weight and first-movement readings partition the same log the same
  // way, and two copies of that rule would drift.
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
    throwInput();
    sim.runUntilQuiescent(300);
    throwInput();
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

  // -- engineering --------------------------------------------------------
  // Cost, dead weight, first movement, symmetry, badges. Runs last because
  // three of the five need the update log the x-ray pass just recorded; the
  // other two would work without it and are computed the same way either way.
  const eng = engineering({
    rest: restBlocks,
    changes,
    xray,
    geometry: geo,
    census: parts,
    movedCells,
    cycleTicks: cycleTicks > 0 ? cycleTicks : null,
    input: lever,
  });

  const record: CertRecord = {
    certificate: {
      name: job.name,
      dims: [w, h, l] as Vec3,
      lever: [lx, ly, lz] as Vec3,
      input: lever,
      input_alternatives: detection.alternatives,
      input_note: detection.note,
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
      // Withheld while the two readings disagree. The classifier would happily
      // name the settled sliver "4 × 1 Regular Door" and be precisely, formally
      // wrong; the sheet shows the disagreement instead.
      classification: apertureConflict ? null : (analysis?.classification ?? null),
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
      aperture_conflict: apertureConflict,
      rest_is_closed: restIsClosed,
      engineering: eng,
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
    // Our own refusals are already sentences and carry their own prefix; only
    // the engine's enums need translating.
    const raw = e instanceof Error ? e.message : String(e);
    const own = raw.match(/^(SIZE|INPUT):\s*([\s\S]*)$/);
    if (own) post({ type: "error", error: own[2], code: own[1] === "SIZE" ? "size guard" : "input search" });
    else {
      const { message, code, detail } = humanError(e);
      post({ type: "error", error: message, code, detail });
    }
  }
};
