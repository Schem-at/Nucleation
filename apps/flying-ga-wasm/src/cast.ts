/** Member-cast replay reconstruction — TypeScript port of
 * examples/render_simulation_video.rs (`Member`/`Motion`/`build_cast`).
 *
 * The change log is the source of truth: per-tick block changes are
 * reconstructed into a cast of `(position, state, lifetime)` members, each
 * posed per frame. Blocks in flight — vanilla's `moving_piston` placeholders —
 * are never drawn as their own cube: they are resolved into the block they
 * CARRY, linearly interpolated from source to destination across the two-tick
 * move window. That is g4mespeed's default "pause at end" piston animation
 * (`(progress * steps + tickDelta) / steps`, clamped), which makes piston
 * motion continuous instead of stepping twice per move.
 *
 * A retracting piston additionally synthesises the vanilla visual: the base
 * renders as its extended shell while a piston head slides back into it. */

export type Vec3 = [number, number, number];
export type Dir = "down" | "up" | "north" | "south" | "west" | "east";

export const DIR_VEC: Record<Dir, Vec3> = {
  down: [0, -1, 0],
  up: [0, 1, 0],
  north: [0, 0, -1],
  south: [0, 0, 1],
  west: [-1, 0, 0],
  east: [1, 0, 0],
};

export const DIR_OPP: Record<Dir, Dir> = {
  down: "up",
  up: "down",
  north: "south",
  south: "north",
  west: "east",
  east: "west",
};

/* ------------------------------------------------------ state parsing ---- */

export interface ParsedState {
  /** Base name without the `minecraft:` prefix or properties. */
  name: string;
  props: Record<string, string>;
}

const parseMemo = new Map<string, ParsedState>();

/** Parse `minecraft:piston[facing=east,extended=true]` into name + props. */
export function parseState(state: string): ParsedState {
  const memo = parseMemo.get(state);
  if (memo) return memo;
  const bracket = state.indexOf("[");
  const base = bracket === -1 ? state : state.slice(0, bracket);
  const name = base.startsWith("minecraft:") ? base.slice(10) : base;
  const props: Record<string, string> = {};
  if (bracket !== -1) {
    const close = state.indexOf("]", bracket);
    const body = state.slice(bracket + 1, close === -1 ? undefined : close);
    for (const kv of body.split(",")) {
      const eq = kv.indexOf("=");
      if (eq > 0) props[kv.slice(0, eq).trim()] = kv.slice(eq + 1).trim();
    }
  }
  const parsed = { name, props };
  parseMemo.set(state, parsed);
  return parsed;
}

export function facingOf(state: string): Dir {
  const f = parseState(state).props.facing;
  return f !== undefined && f in DIR_VEC ? (f as Dir) : "north";
}

const AIR = new Set(["air", "cave_air", "void_air"]);
export function isAirState(state: string): boolean {
  return AIR.has(parseState(state).name);
}

/* ---------------------------------------------------------- the cast ----- */

export interface CastBlock {
  pos: Vec3;
  state: string;
}
export interface CastChange {
  tick: number;
  pos: Vec3;
  from: string;
  to: string;
}

/** One drawable: a block state at a position, alive for a tick interval. */
export interface CastMember {
  x: number;
  y: number;
  z: number;
  state: string;
  /** Alive for `start..end`, in game ticks (fractional times compare here). */
  start: number;
  end: number;
  /** Where it travels from during `start..until`, if it is in flight. */
  motion?: { fx: number; fy: number; fz: number; until: number };
}

const k3 = (p: Vec3) => `${p[0]},${p[1]},${p[2]}`;

function typeOf(movingDescriptor: string): "sticky" | "normal" {
  return parseState(movingDescriptor).props.type === "sticky" ? "sticky" : "normal";
}

/** Whether `next` is `previous` with extended flipped true -> false — the
 * signature of a retracting piston base. */
function isExtendedFalseOf(previous: string | undefined, next: string): boolean {
  if (previous === undefined) return false;
  return (
    (next.startsWith("minecraft:piston[") || next.startsWith("minecraft:sticky_piston[")) &&
    next.includes("extended=false") &&
    previous.replace("extended=true", "extended=false") === next
  );
}

/** The head that visually retracts into `base`. */
function headStateFor(base: string, facing: Dir): string {
  const kind = base.startsWith("minecraft:sticky_piston") ? "sticky" : "normal";
  return `minecraft:piston_head[facing=${facing},short=false,type=${kind}]`;
}

/** Turn initial state + change log into drawable members. `ticks` is the
 * exclusive end of the render window (members visible while t < end). */
export function buildCast(
  initial: CastBlock[],
  changes: CastChange[],
  ticks: number,
): CastMember[] {
  const sorted = [...changes].sort((a, b) => a.tick - b.tick);

  // Per-position state segments, in order: (state, start, end).
  const segments = new Map<string, Array<{ state: string; start: number; end: number }>>();
  const posOf = new Map<string, Vec3>();
  for (const b of initial) {
    if (isAirState(b.state)) continue;
    segments.set(k3(b.pos), [{ state: b.state, start: 0, end: ticks }]);
    posOf.set(k3(b.pos), b.pos);
  }
  for (const c of sorted) {
    const kk = k3(c.pos);
    if (!posOf.has(kk)) posOf.set(kk, c.pos);
    let list = segments.get(kk);
    if (!list) {
      list = [];
      segments.set(kk, list);
    }
    const last = list[list.length - 1];
    if (last) last.end = c.tick;
    else if (!isAirState(c.from)) {
      // A position first mentioned by a change: it held `from` since tick 0.
      list.push({ state: c.from, start: 0, end: c.tick });
    }
    list.push({ state: c.to, start: c.tick, end: ticks });
  }

  // Who lost which state when — the source lookup for travelling blocks.
  const vacated = new Map<string, Vec3[]>();
  for (const c of sorted) {
    const key = `${c.tick}|${c.from}`;
    const arr = vacated.get(key);
    if (arr) arr.push(c.pos);
    else vacated.set(key, [c.pos]);
  }

  const members: CastMember[] = [];
  for (const [kk, list] of segments) {
    const pos = posOf.get(kk)!;
    for (let i = 0; i < list.length; i++) {
      const { state, start, end } = list[i];
      if (isAirState(state)) continue;
      if (!state.startsWith("minecraft:moving_piston")) {
        members.push({ x: pos[0], y: pos[1], z: pos[2], state, start, end });
        continue;
      }

      // A flight: what does this placeholder become, and where from?
      const facing = facingOf(state);
      const fv = DIR_VEC[facing];
      const landed = list[i + 1]?.state;
      const previous = i > 0 ? list[i - 1].state : undefined;
      if (landed !== undefined && isExtendedFalseOf(previous, landed)) {
        // A retracting base: the piston does not travel — its head slides
        // back into it while the base shows its extended shell.
        members.push({ x: pos[0], y: pos[1], z: pos[2], state: previous!, start, end });
        members.push({
          x: pos[0],
          y: pos[1],
          z: pos[2],
          state: headStateFor(landed, facing),
          start,
          end,
          motion: { fx: pos[0] + fv[0], fy: pos[1] + fv[1], fz: pos[2] + fv[2], until: end },
        });
      } else if (
        landed !== undefined &&
        !isAirState(landed) &&
        !landed.startsWith("minecraft:moving_piston")
      ) {
        // A block in flight toward this position. Its source is the adjacent
        // position that lost the same state on this tick; a piston head has
        // no source block — it emerges from the base.
        const from =
          (vacated.get(`${start}|${landed}`) ?? []).find(
            (p) =>
              Math.abs(p[0] - pos[0]) + Math.abs(p[1] - pos[1]) + Math.abs(p[2] - pos[2]) === 1,
          ) ?? ([pos[0] - fv[0], pos[1] - fv[1], pos[2] - fv[2]] as Vec3);
        members.push({
          x: pos[0],
          y: pos[1],
          z: pos[2],
          state: landed,
          start,
          end: list[i + 1].end,
          motion: { fx: from[0], fy: from[1], fz: from[2], until: end },
        });
        // The landed segment itself is dropped below: the flight covers it.
      } else {
        // Interrupted flight (finalTicked to air, or replaced by another
        // placeholder): an emerging head that got yanked.
        members.push({
          x: pos[0],
          y: pos[1],
          z: pos[2],
          state: `minecraft:piston_head[facing=${facing},short=false,type=${typeOf(state)}]`,
          start,
          end,
          motion: { fx: pos[0] - fv[0], fy: pos[1] - fv[1], fz: pos[2] - fv[2], until: end },
        });
      }
    }
  }

  // Landed segments already covered by a flight member (same pos, same state,
  // inside the flight's interval) must not double-render.
  const flights = members.filter((m) => m.motion);
  return members.filter(
    (m) =>
      m.motion ||
      !flights.some(
        (f) =>
          f.x === m.x &&
          f.y === m.y &&
          f.z === m.z &&
          f.state === m.state &&
          m.start >= f.start &&
          m.end <= f.end,
      ),
  );
}

/* ------------------------------------------------------- per-frame pose -- */

export interface DrawItem {
  /** Member identity, stable across frames (React keys / caching). */
  id: number;
  x: number;
  y: number;
  z: number;
  /** Interpolated world-space offset (0 when at rest). */
  ox: number;
  oy: number;
  oz: number;
  state: string;
}

/** Visible members at fractional tick `t`, with interpolated offsets.
 * g4mespeed PAUSE_END: linear across the move window, clamped. */
export function castAt(members: CastMember[], t: number): DrawItem[] {
  const out: DrawItem[] = [];
  for (let id = 0; id < members.length; id++) {
    const m = members[id];
    if (t < m.start || t >= m.end) continue;
    let ox = 0;
    let oy = 0;
    let oz = 0;
    if (m.motion) {
      const span = m.motion.until - m.start;
      const progress = span > 0 ? Math.min(1, Math.max(0, (t - m.start) / span)) : 1;
      const remaining = 1 - progress;
      ox = (m.motion.fx - m.x) * remaining;
      oy = (m.motion.fy - m.y) * remaining;
      oz = (m.motion.fz - m.z) * remaining;
    }
    out.push({ id, x: m.x, y: m.y, z: m.z, ox, oy, oz, state: m.state });
  }
  return out;
}

/* ---------------------------------------------------------- geometry ----- */

/** A sub-cube of a block model, bounds in cell-local [0,1] units (an arm may
 * poke past). `tex` is the blockstate used for per-face texture resolution;
 * null draws flat colour only. */
export interface IsoBox {
  x0: number;
  y0: number;
  z0: number;
  x1: number;
  y1: number;
  z1: number;
  tex: string | null;
}

const UNIT_BOX = (state: string): IsoBox[] => [
  { x0: 0, y0: 0, z0: 0, x1: 1, y1: 1, z1: 1, tex: state },
];

/** Box spanning `[a, b]` along the facing axis measured from the BACK face
 * (0 = back, 1 = front/facing face), `[c0, c1]` on the two cross axes. */
function boxAlong(facing: Dir, a: number, b: number, c0 = 0, c1 = 1): Omit<IsoBox, "tex"> {
  const v = DIR_VEC[facing];
  const positive = v[0] + v[1] + v[2] > 0;
  const lo = positive ? a : 1 - b;
  const hi = positive ? b : 1 - a;
  const axis = v[0] !== 0 ? 0 : v[1] !== 0 ? 1 : 2;
  const bounds: Array<[number, number]> = [
    [c0, c1],
    [c0, c1],
    [c0, c1],
  ];
  bounds[axis] = [lo, hi];
  return {
    x0: bounds[0][0],
    x1: bounds[0][1],
    y0: bounds[1][0],
    y1: bounds[1][1],
    z0: bounds[2][0],
    z1: bounds[2][1],
  };
}

const boxMemo = new Map<string, IsoBox[]>();

/** The sub-boxes drawn for a blockstate:
 * - `piston[extended=true]` base: the open shell, 12/16 deep (front 4/16
 *   vacated by the departed plate; `piston_inner` lands on the open face).
 * - `piston_head`: a 4/16 plate flush with the facing face plus a thin arm
 *   reaching 4/16 into the base cell (flat-colour approximation).
 * - Everything else: the unit cube. */
export function boxesFor(state: string): IsoBox[] {
  const memo = boxMemo.get(state);
  if (memo) return memo;
  const { name, props } = parseState(state);
  const facing = (props.facing ?? "north") as Dir;
  let boxes: IsoBox[];
  if ((name === "piston" || name === "sticky_piston") && props.extended === "true") {
    boxes = [{ ...boxAlong(facing, 0, 0.75), tex: state }];
  } else if (name === "piston_head") {
    boxes = [
      { ...boxAlong(facing, -0.25, 0.75, 0.375, 0.625), tex: null }, // arm
      { ...boxAlong(facing, 0.75, 1), tex: state }, // plate
    ];
  } else {
    boxes = UNIT_BOX(state);
  }
  boxMemo.set(state, boxes);
  return boxes;
}
