/** X-ray propagation data — the recorded redstone update stream, packed for
 * rendering.
 *
 * The engine records every delivered update (`TickSimulation.recordUpdates`)
 * and exposes three views of the log. Only two of them are for drawing:
 *
 *   updatesHeatJson(from,to)  per tick, per cell: how many updates landed,
 *                             split neighbour/shape and by tick phase.
 *                             ~900 KB for a 6x6 door's whole cycle.
 *   updatesWaveJson(tick)     ONE tick in delivery order, as parallel arrays.
 *                             ~310 KB for the busiest tick (19,834 updates).
 *
 * The raw `updatesJson()` is 15.8 MB per cycle and is never used here.
 *
 * Both views are decoded into typed arrays against a single **cell table** —
 * the union of every cell that ever receives an update — so a cell's index is
 * also its instance slot in the flare mesh, and a frame is a loop over floats.
 * The buffers are transferable, so the worker hands them over rather than
 * copying them.
 *
 * The certify worker imports this module, so it stays free of three.js; the
 * flare geometry lives with the renderer. */

/** One tick of the heat view. `cell` indexes `XrayData.cells`. */
export type XrayHeatTick = {
  /** Rebased so tick 0 is the lever click, like the replay. */
  tick: number;
  total: number;
  cell: Uint16Array;
  /** Updates delivered to that cell during the tick. */
  n: Uint16Array;
  /** Neighbour / shape split of `n`. */
  nb: Uint16Array;
  sh: Uint16Array;
  /** `cell.length * phases.length`, row-major: per-phase counts. */
  ph: Uint16Array;
};

/** One tick of the wave view, in delivery order. Index = `seq`. */
export type XrayWave = {
  tick: number;
  n: number;
  cell: Uint16Array;
  kind: Uint8Array;
  phase: Uint8Array;
  /** Index into `XrayData.dirs`; `>= dirs.length` means "no source". */
  from: Uint8Array;
  /** Index into `states`. */
  state: Uint16Array;
  states: string[];
};

export type XrayData = {
  /** Phases that actually fired, in engine (pipeline) order. */
  phases: string[];
  kinds: string[];
  dirs: string[];
  /** Flat x,y,z of every cell that receives at least one update. */
  cells: Int16Array;
  ticks: XrayHeatTick[];
  /** Indexed by rebased tick; every tick of the cycle is precomputed. */
  waves: XrayWave[];
  totalUpdates: number;
  /** 95th percentile of per-cell-per-tick counts — the intensity reference,
   *  so one 3,000-update cell cannot flatten the whole build to black. */
  heatRef: number;
  /** Raw engine JSON sizes, reported on the panel. */
  bytes: { heat: number; waves: number };
};

/* ------------------------------------------------------------- palette --- */

/** The x-ray stage is a darkroom in both themes: emissive colour only reads
 * against a dark plate, and validating one surface beats validating two. */
export const XRAY_SURFACE = 0x141312;

/** How a channel value is drawn. `cage` is a hollow wireframe box — a MARK,
 * not a hue, which is how the fourth phase escapes the three-hue ceiling
 * below. */
export type XrayStyle = { hex: number; label: string; mark: "glow" | "cage" };

/** Update-kind channel: two categories, two hues (categorical slots 1-2).
 * Validated all-pairs on #141312: CVD ΔE 26.8, normal-vision ΔE 31.8. */
const KIND_STYLE: Record<string, XrayStyle> = {
  neighbor: { hex: 0x3987e5, label: "neighbour", mark: "glow" },
  shape: { hex: 0xd95926, label: "shape", mark: "glow" },
};

/** Tick-phase channel. Colour follows the phase's identity, never its rank.
 *
 * A door fires exactly four phases (block_events 69%, block_entities 26%,
 * block_ticks 3%, boundary 2%), and four hues cannot be made safe here: every
 * cell is adjacent to every other in a 3-D scene, so the all-pairs gate
 * applies, and the reference palette documents that only its first THREE slots
 * clear it. Every four-hue candidate was run through
 * `validate_palette.js --pairs all` and every one failed the normal-vision
 * floor, which no secondary encoding excuses.
 *
 * So three phases take the three validated hues (all-pairs on #141312: worst
 * CVD ΔE 9.4, worst normal-vision ΔE 20.9) and `boundary` — which is not a
 * level-tick phase at all, but the out-of-tick action of the lever click —
 * takes a hueless wireframe cage. It is separated by mark, so it enters no
 * colour pair. Any phase the engine grows later lands on the cage too. */
const PHASE_STYLE: Record<string, XrayStyle> = {
  block_ticks: { hex: 0x3987e5, label: "block ticks", mark: "glow" },
  block_events: { hex: 0xd95926, label: "block events", mark: "glow" },
  block_entities: { hex: 0x199e70, label: "block entities", mark: "glow" },
  boundary: { hex: 0xe8e6df, label: "boundary", mark: "cage" },
};

const CAGE_FALLBACK = 0xe8e6df;

export function phaseStyle(phase: string): XrayStyle {
  return (
    PHASE_STYLE[phase] ?? {
      hex: CAGE_FALLBACK,
      label: phase.replace(/_/g, " "),
      mark: "cage",
    }
  );
}

export function kindStyle(kind: string): XrayStyle {
  return (
    KIND_STYLE[kind] ?? { hex: CAGE_FALLBACK, label: kind, mark: "cage" }
  );
}

export type Channel = "phase" | "kind";

/** Styles for a channel, in legend order. */
export function channelStyles(data: XrayData, channel: Channel): XrayStyle[] {
  return channel === "phase"
    ? data.phases.map(phaseStyle)
    : data.kinds.map(kindStyle);
}

export const hexCss = (hex: number) => "#" + hex.toString(16).padStart(6, "0");

/* -------------------------------------------------------------- decode --- */

type RawHeatCell = { p: [number, number, number]; n: number; nb: number; sh: number; ph: number[] };
type RawHeat = { phases: string[]; ticks: { tick: number; total: number; cells: RawHeatCell[] }[] };
type RawWave = {
  tick: number;
  n: number;
  pos: number[];
  kind: number[];
  phase: number[];
  from: number[];
  state: number[];
  states: string[];
  phases: string[];
  dirs: string[];
  kinds: string[];
};

/** Every ArrayBuffer in a payload, for `postMessage`'s transfer list. */
export function xrayTransferables(d: XrayData): ArrayBuffer[] {
  const out: ArrayBuffer[] = [d.cells.buffer as ArrayBuffer];
  for (const t of d.ticks)
    out.push(
      t.cell.buffer as ArrayBuffer,
      t.n.buffer as ArrayBuffer,
      t.nb.buffer as ArrayBuffer,
      t.sh.buffer as ArrayBuffer,
      t.ph.buffer as ArrayBuffer,
    );
  for (const w of d.waves)
    out.push(
      w.cell.buffer as ArrayBuffer,
      w.kind.buffer as ArrayBuffer,
      w.phase.buffer as ArrayBuffer,
      w.from.buffer as ArrayBuffer,
      w.state.buffer as ArrayBuffer,
    );
  return out;
}

/** Pack the engine's two JSON views into the render-ready payload.
 *
 * `heatJson` covers `[rebase, rebase + simTicks)`; `waveJson(t)` is called once
 * per tick of that range. Ticks are rebased so 0 is the lever click. */
export function decodeXray(
  heatJson: string,
  waveJson: (tick: number) => string,
  rebase: number,
  simTicks: number,
): XrayData {
  const raw = JSON.parse(heatJson) as RawHeat;

  // Cell table: the union of every updated cell. Index doubles as the flare
  // mesh's instance slot.
  const index = new Map<string, number>();
  const flat: number[] = [];
  const cellOf = (x: number, y: number, z: number): number => {
    const k = `${x},${y},${z}`;
    let i = index.get(k);
    if (i === undefined) {
      i = flat.length / 3;
      index.set(k, i);
      flat.push(x, y, z);
    }
    return i;
  };

  // Compact the 11-entry engine phase legend down to the phases that fire.
  const used = new Set<number>();
  for (const t of raw.ticks) for (const c of t.cells) {
    for (let i = 0; i < c.ph.length; i++) if (c.ph[i] > 0) used.add(i);
  }
  const phaseIdx = [...used].sort((a, b) => a - b);
  const phases = phaseIdx.map((i) => raw.phases[i]);
  const P = phases.length;
  const remap = new Int8Array(raw.phases.length).fill(-1);
  phaseIdx.forEach((src, dst) => (remap[src] = dst));

  const ticks: XrayHeatTick[] = [];
  const counts: number[] = [];
  let totalUpdates = 0;
  for (const t of raw.ticks) {
    const m = t.cells.length;
    const cell = new Uint16Array(m);
    const n = new Uint16Array(m);
    const nb = new Uint16Array(m);
    const sh = new Uint16Array(m);
    const ph = new Uint16Array(m * P);
    for (let i = 0; i < m; i++) {
      const c = t.cells[i];
      cell[i] = cellOf(c.p[0], c.p[1], c.p[2]);
      n[i] = Math.min(c.n, 65535);
      nb[i] = Math.min(c.nb, 65535);
      sh[i] = Math.min(c.sh, 65535);
      for (let p = 0; p < c.ph.length; p++) {
        const d = remap[p];
        if (d >= 0) ph[i * P + d] = Math.min(c.ph[p], 65535);
      }
      counts.push(c.n);
    }
    totalUpdates += t.total;
    ticks.push({ tick: t.tick - rebase, total: t.total, cell, n, nb, sh, ph });
  }

  // Waves: decoded into number[] first because the cell table's final width
  // is only known once every tick has been walked.
  type Pending = {
    tick: number;
    cell: number[];
    kind: number[];
    phase: number[];
    from: number[];
    state: number[];
    states: string[];
  };
  const pending: Pending[] = [];
  let kinds: string[] = ["neighbor", "shape"];
  let dirs: string[] = [];
  let waveBytes = 0;
  for (let t = 0; t < simTicks; t++) {
    const json = waveJson(rebase + t);
    waveBytes += json.length;
    const w = JSON.parse(json) as RawWave;
    if (w.kinds?.length) kinds = w.kinds;
    if (w.dirs?.length) dirs = w.dirs;
    const cell: number[] = [];
    const phase: number[] = [];
    for (let i = 0; i < w.n; i++) {
      cell.push(cellOf(w.pos[i * 3], w.pos[i * 3 + 1], w.pos[i * 3 + 2]));
      // An unfired phase can still appear here if it delivered zero-count
      // updates; park it past the legend so the readout says so.
      const d = remap[w.phase[i]];
      phase.push(d >= 0 ? d : P);
    }
    pending.push({
      tick: t,
      cell,
      kind: w.kind,
      phase,
      from: w.from,
      state: w.state,
      states: w.states,
    });
  }

  const waves: XrayWave[] = pending.map((p) => ({
    tick: p.tick,
    n: p.cell.length,
    cell: Uint16Array.from(p.cell),
    kind: Uint8Array.from(p.kind),
    phase: Uint8Array.from(p.phase),
    from: Uint8Array.from(p.from, (v) => (v >= 0 && v < 255 ? v : 255)),
    state: Uint16Array.from(p.state),
    states: p.states,
  }));

  counts.sort((a, b) => a - b);
  const heatRef = Math.max(4, counts[Math.floor(counts.length * 0.95)] ?? 4);

  return {
    phases,
    kinds,
    dirs,
    cells: Int16Array.from(flat),
    ticks,
    waves,
    totalUpdates,
    heatRef,
    bytes: { heat: heatJson.length, waves: waveBytes },
  };
}
