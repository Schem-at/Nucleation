export type TickEvents = {
  tick: number;
  piston: number;
  redstone: number;
  /** Item-movement events. The engine does not report these yet, so this is
   *  zero everywhere; the stacked trace already carries the series. */
  items: number;
  changes: number;
};

export type Material = { id: string; count: number };

export type Verdict = "CERTIFIED" | "DID NOT RESET";

/** The doorway itself: cells that are solid at rest and air once open. */
export type Aperture = {
  /** How many cells open up. */
  cells: number;
  /** Horizontal span of the opening, in blocks. */
  w: number;
  /** Vertical span of the opening, in blocks. */
  h: number;
  /** Depth of the opening along the remaining axis. */
  depth: number;
  /** Set when the measurement needed a caveat — several separate openings, or
   *  no walkable passage at all. Null when the doorway read cleanly. */
  note?: string | null;
};

/** The cells the doorway timing is measured over, in world coordinates.
 *  `aperture()` already knows both of these exactly; timing them is the whole
 *  point of extracting them. */
export type ApertureGeometry = {
  /** Every cell of the walkable passage, across the wall's full depth. The
   *  door is OPEN when all of these are air. */
  passage: Vec3[];
  /** The passage cells that hold a door block in the closed reference state —
   *  the pattern, not the passage. The door is SHUT when all of these are
   *  solid again; a sissy bar or a checkerboard never fills the rest. */
  closed: Vec3[];
  /** True when the first snapshot handed to `aperture()` is the closed one,
   *  i.e. the first lever click opens the door. A file saved open measures
   *  its own closing first, and the strokes swap. */
  restIsClosed: boolean;
};

/** One door block, in the standard's matrix coordinates: row counted down
 *  from the top of the pattern, column left to right, layer front to back. */
export type PatternCell = { r: number; c: number; k: number; id: string };

/** What sits around the doorway — needed for the Section 5 compositions. */
export type PatternSurroundings = {
  /** Blocks in the ring immediately around the pattern, door layers, closed. */
  frameIds: string[];
  /** The ring outside that one. */
  outerIds: string[];
  /** What the doorway stands on once it is open. */
  sillIds: string[];
};

/** The door read against MYuen222, "Door Pattern Definitions v1.1". */
export type Classification = {
  /** The formal name, e.g. "6 × 6 Flush Regular Door". */
  name: string;
  /** Pattern length and height (Def 2.9). */
  m: number;
  n: number;
  /** Pattern depth: parallel planes the door blocks occupy. */
  layers: number;
  /** Def 2.4. */
  orientation: "Door" | "Skydoor" | "Ceiling Skydoor";
  /** Flush / Deluxe / Trapdoor (Defs 2.6-2.8), when one of them applies. */
  qualifiers: string[];
  /** Where the outermost door layer sits relative to the frame face, in
   *  plain language — the standard has no term for every case. */
  frameNote: string | null;
  /** True when every door block spans the same run of layers: a flat pattern
   *  extruded through the wall rather than a depth-varying one. */
  extruded: boolean;
  /** Matched pattern name, or null when nothing matched exactly. */
  pattern: string | null;
  /** Section of the standard the pattern is defined in. */
  patternRef: string | null;
  /** Which member of the symmetry group matched, when not the identity. */
  transform: string | null;
  /** Section 5 block compositions found. */
  composition: { label: string; ref: string }[];
  /** Binary pattern matrix: 1 where a door block sits when closed. */
  matrix: number[][];
  /** Layer index per cell; -1 where empty. */
  depth: number[][];
  unclassified: boolean;
};

/** Counts of the parts that make the door move. */
export type Census = {
  sticky_piston: number;
  piston: number;
  observer: number;
  repeater: number;
  /** Distinct repeater delays present, ascending. */
  repeater_delays: number[];
  comparator: number;
  redstone_block: number;
  redstone_torch: number;
  redstone_wire: number;
  slime_block: number;
  honey_block: number;
};

/** Purplers' reset measurement: the shortest delay after a lever click that
 *  still lets the input be used again without breaking the machine. There is
 *  no closed form — every value here comes from a trial. */
export type ResetTime = {
  /** Shortest delay that worked, in ticks. Null when nothing under `searched`
   *  did — reported as such rather than guessed at. */
  ticks: number | null;
  /** How far the search actually went, so a null is honest about its reach. */
  searched: number;
  /** The stroke this reset re-triggers into, for the comparison below. */
  stroke_ticks: number | null;
  /** True when the reset lands before the stroke it interrupts has finished:
   *  the input can be used again mid-stroke. Rare. */
  negative: boolean;
  /** Set when the measurement was skipped or qualified. */
  note: string | null;
};

export type Certificate = {
  name: string;
  dims: [number, number, number];
  lever: [number, number, number];
  /** Ticks from the click until every passage cell is clear — the doorway is
   *  walkable. Null when the passage never fully cleared. */
  open_ticks: number | null;
  /** Ticks from the click until every cell of the closed pattern is solid
   *  again. Null when the doorway never fully re-filled. */
  close_ticks: number | null;
  /** Ticks from the click until the whole machine goes quiet. Always >= the
   *  doorway time above: the tape can still be shuffling long after you can
   *  walk through. This is the number the certificate used to call "opens in". */
  open_settle_ticks: number | null;
  close_settle_ticks: number | null;
  /** Set when a doorway time is missing, saying why. */
  timing_note: string | null;
  /** Ticks from the lever click to the first block that moves. */
  open_latency: number | null;
  close_latency: number | null;
  /** Shortest wait after the opening click before the lever may be thrown
   *  back, and the same after the closing click. */
  reset_open: ResetTime | null;
  reset_close: ResetTime | null;
  materials: Material[];
  events_per_tick: TickEvents[];
  heatmap: { w: number; h: number; values: number[][] };
  /** Length of the measured cycle, in ticks. */
  sim_ticks: number;
  seed: number;
  verdict: Verdict;
  /** Cells that travel on one stroke — the door's working mass. */
  moved_cells: number;
  /** The doorway the machine actually opens. Null if nothing opened. */
  aperture: Aperture | null;
  /** The doorway read against the community pattern standard. */
  classification: Classification | null;
  /** Busiest tick of the cycle: how many cells changed, and when. */
  peak_changes: number;
  peak_tick: number;
  /** Busiest tick for pure signal traffic — the redstone cascade, which
   *  peaks on the click tick and moves nothing. */
  peak_signal: number;
  peak_signal_tick: number;
  /** Parts census, taken at rest. */
  census: Census;
  /** Fastest safe re-trigger, as cycles per minute. */
  cycles_per_minute: number;
  /** Total cells inside the scanned bounds. */
  volume: number;
  /** True when the door was saved mid-cycle and had to be run to its
   *  steady state before the measured cycle could start. */
  needed_priming: boolean;
  /** Cycles needed before the machine repeated itself. */
  priming_cycles: number;
  /** Cells between the saved state and the cycle it settles into. */
  saved_state_drift: number;
  /** False when the file was saved with its doorway already standing open —
   *  the first lever click then closes it, and the two strokes swap. */
  rest_is_closed: boolean;
};

export type Vec3 = [number, number, number];

export type ReplayBlock = { pos: Vec3; state: string };
export type ReplayChange = { tick: number; pos: Vec3; from: string; to: string };
export type LeverFlip = { tick: number; label: string; measured: boolean };

/** Everything the voxel replay needs: t=0 world + the recorded change log. */
export type Replay = {
  blocks: ReplayBlock[];
  changes: ReplayChange[];
  simTicks: number;
  flips: LeverFlip[];
};

export type CertRecord = { certificate: Certificate; replay: Replay };

/** Worker → page protocol. */
export type WorkerProgress = { type: "progress"; step: string };
export type WorkerDone = { type: "done"; record: CertRecord };
export type WorkerError = { type: "error"; error: string };
export type WorkerMessage = WorkerProgress | WorkerDone | WorkerError;

/** Page → worker payload. */
export type WorkerJob = { name: string; ext: string; buffer: ArrayBuffer };
