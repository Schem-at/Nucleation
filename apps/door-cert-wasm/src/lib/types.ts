export type TickEvents = {
  tick: number;
  piston: number;
  redstone: number;
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
