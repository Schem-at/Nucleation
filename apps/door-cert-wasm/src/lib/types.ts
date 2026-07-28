export type TickEvents = {
  tick: number;
  piston: number;
  redstone: number;
  changes: number;
};

export type Material = { id: string; count: number };

export type Verdict = "CERTIFIED" | "DID NOT RESET";

export type Certificate = {
  name: string;
  dims: [number, number, number];
  lever: [number, number, number];
  open_ticks: number;
  close_ticks: number;
  materials: Material[];
  events_per_tick: TickEvents[];
  heatmap: { w: number; h: number; values: number[][] };
  sim_ticks: number;
  seed: number;
  verdict: Verdict;
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
