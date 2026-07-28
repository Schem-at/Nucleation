export type DoorStatus = {
  status: "processing" | "done" | "error";
  step: string | null;
  error: string | null;
};

export type TickEvents = {
  tick: number;
  piston: number;
  redstone: number;
  changes: number;
};

export type Material = { id: string; count: number };

export type Certificate = {
  name: string;
  dims: [number, number, number];
  lever: [number, number, number];
  open_ticks: number;
  close_ticks: number;
  materials: Material[];
  events_per_tick: TickEvents[];
  heatmap: { w: number; h: number; values: number[][] };
  animation_url: string;
  sim_ticks: number;
  seed: number;
};
