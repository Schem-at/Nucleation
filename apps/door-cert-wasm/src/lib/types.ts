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

export type Certificate = {
  name: string;
  dims: [number, number, number];
  lever: [number, number, number];
  open_ticks: number;
  close_ticks: number;
  /** Ticks from the lever click to the first block that moves. */
  open_latency: number;
  close_latency: number;
  materials: Material[];
  events_per_tick: TickEvents[];
  heatmap: { w: number; h: number; values: number[][] };
  /** Length of the measured cycle, in ticks. */
  sim_ticks: number;
  seed: number;
  verdict: Verdict;
  /** Cells that travel on one stroke — the door's working mass. */
  moved_cells: number;
  /** Whether the door still works when pasted (vanilla's placement pass
   *  re-derives redstone state and can leave a memory cell unlatched). */
  paste_safe: boolean;
  paste_moved_cells: number;
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
