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
  /** For piston_head members: the BASE cell this head belongs to. Lets the
   * pose compute the live extension (0 = seated, 1 = fully out) so the arm
   * can be clamped to span base front face → plate inner face instead of
   * poking fixed-length out the back of the base. */
  headBase?: Vec3;
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
        //
        // Seat the head HALF A TICK EARLY (`until: end - 0.5`): extension is
        // pause-at-end by construction (the flight member outlives its move
        // window, so the clamp holds the seated pose), but this member DIES
        // at `end` — with `until: end` the head still floats a sub-tick step
        // proud of its seat on the last drawn frame, and the swap to the
        // closed extended=false cube pops. Seating early gives retraction
        // the same pause-at-end: seated head + extended shell compose the
        // exact closed-cube geometry before the swap, so the swap is
        // invisible.
        const seatAt = Math.max(start + (end - start) / 2, end - 0.5);
        members.push({ x: pos[0], y: pos[1], z: pos[2], state: previous!, start, end });
        members.push({
          x: pos[0],
          y: pos[1],
          z: pos[2],
          state: headStateFor(landed, facing),
          start,
          end,
          motion: { fx: pos[0] + fv[0], fy: pos[1] + fv[1], fz: pos[2] + fv[2], until: seatAt },
          headBase: pos,
        });
      } else if (
        landed !== undefined &&
        !isAirState(landed) &&
        !landed.startsWith("minecraft:moving_piston") &&
        previous !== undefined &&
        parseState(previous).name === parseState(landed).name
      ) {
        // An in-place state swap of the SAME block family passing through a
        // moving_piston phase — the signature of a piston BASE extending
        // (extended=false → moving_piston → extended=true). The block does
        // not travel; inferring motion here (via the vacancy lookup or the
        // behind-the-facing fallback) made the base visibly slide ~half a
        // block backward during the stroke. Draw the landed state in place
        // for the placeholder phase; the landed segment then takes over.
        members.push({ x: pos[0], y: pos[1], z: pos[2], state: landed, start, end });
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
          // An emerging head extends out of the source cell it came from.
          ...(parseState(landed).name === "piston_head"
            ? { headBase: from }
            : null),
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
          headBase: [pos[0] - fv[0], pos[1] - fv[1], pos[2] - fv[2]],
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
  /** piston_head only: live extension 0 (seated) .. 1 (fully out), from the
   * member's `headBase`. Undefined means fully extended (static heads). */
  ext?: number;
}

/** Live extension of a piston_head member at world offset (ox,oy,oz):
 * signed distance of the head cell from its base cell along the facing. */
export function headExtension(m: CastMember, ox: number, oy: number, oz: number): number {
  if (!m.headBase) return 1;
  const fv = DIR_VEC[facingOf(m.state)];
  const e =
    (m.x + ox - m.headBase[0]) * fv[0] +
    (m.y + oy - m.headBase[1]) * fv[1] +
    (m.z + oz - m.headBase[2]) * fv[2];
  return Math.min(1, Math.max(0, e));
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
    const item: DrawItem = { id, x: m.x, y: m.y, z: m.z, ox, oy, oz, state: m.state };
    if (m.headBase) item.ext = headExtension(m, ox, oy, oz);
    out.push(item);
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
 * - `*_trapdoor`: a 3/16 plate — closed: horizontal at the cell floor
 *   (`half=bottom`) or ceiling (`half=top`); open: vertical against the
 *   hinge face (the face opposite `facing`, matching the vanilla
 *   `trapdoor_open` model).
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
      // Fully-extended arm: base front face (head-local -0.25) → plate
      // inner face. Posed heads mid-move clamp this via `boxesForItem`.
      { ...boxAlong(facing, -0.25, 0.75, 0.375, 0.625), tex: null }, // arm
      { ...boxAlong(facing, 0.75, 1), tex: state }, // plate
    ];
  } else if (name.endsWith("trapdoor")) {
    const t = 3 / 16;
    if (props.open === "true") {
      // boxAlong measures from the BACK face; the open plate hangs on the
      // hinge (back) face, opposite `facing`.
      boxes = [{ ...boxAlong(facing, 0, t), tex: state }];
    } else if (props.half === "top") {
      boxes = [{ x0: 0, y0: 1 - t, z0: 0, x1: 1, y1: 1, z1: 1, tex: state }];
    } else {
      boxes = [{ x0: 0, y0: 0, z0: 0, x1: 1, y1: t, z1: 1, tex: state }];
    }
  } else {
    boxes = UNIT_BOX(state);
  }
  boxMemo.set(state, boxes);
  return boxes;
}

/** GL-renderer counterpart of the arm clamp: meshed vanilla piston_head
 * models carry a fixed-length arm poking 4/16 out the back of the cell, so
 * a head near its seat pushes the arm through the base and out its rear.
 * This returns the world-space half-space to KEEP (n·p ≥ min): everything
 * in front of the base's front face. The plane is static per member (the
 * base cell never moves), so renderers can clip once at instancing time. */
export function headClipHalfSpace(
  m: CastMember,
): { nx: number; ny: number; nz: number; min: number } | null {
  if (!m.headBase) return null;
  const fv = DIR_VEC[facingOf(m.state)];
  const axis = fv[0] !== 0 ? 0 : fv[1] !== 0 ? 1 : 2;
  const min =
    fv[axis] > 0 ? m.headBase[axis] + 0.75 : -(m.headBase[axis] + 0.25);
  return { nx: fv[0], ny: fv[1], nz: fv[2], min };
}

const posedMemo = new Map<string, IsoBox[]>();

/** Pose-aware boxes for a draw item. Identical to `boxesFor` except for a
 * piston_head mid-move: its arm is clamped to span exactly from the base's
 * front face (head-local `0.75 - ext`) to the plate's inner face, so the
 * arm's length shrinks to 0 as the head seats instead of the fixed-length
 * arm passing through the base and out its back. */
export function boxesForItem(item: Pick<DrawItem, "state" | "ext">): IsoBox[] {
  const ext = item.ext;
  if (ext === undefined || ext >= 1) return boxesFor(item.state);
  const { name, props } = parseState(item.state);
  if (name !== "piston_head") return boxesFor(item.state);
  const facing = (props.facing ?? "north") as Dir;
  // Quantize so the memo stays small across an animation's frames.
  const q = Math.round(Math.max(0, ext) * 64) / 64;
  const key = `${item.state}|${q}`;
  const memo = posedMemo.get(key);
  if (memo) return memo;
  const boxes: IsoBox[] =
    q <= 0
      ? [{ ...boxAlong(facing, 0.75, 1), tex: item.state }] // seated: plate only
      : [
          { ...boxAlong(facing, 0.75 - q, 0.75, 0.375, 0.625), tex: null }, // arm
          { ...boxAlong(facing, 0.75, 1), tex: item.state }, // plate
        ];
  posedMemo.set(key, boxes);
  return boxes;
}

/* --------------------------------------------------------------- entities -- */

/** Where an entity is at continuous replay time `t`, or null if it is not
 *  alive then.
 *
 *  Entity positions are floating point and change every tick, so the replay
 *  interpolates between the two bracketing samples exactly as the video
 *  renderer does (`render_simulation_video.rs`, the `item_tracks` loop): a
 *  cart crossing a rail at 0.18 blocks/tick has to slide, not step. The last
 *  sample of a track holds rather than vanishing, so an entity that comes to
 *  rest stays put instead of blinking out on the final frame. */
export function entityPosAt(track: (Vec3 | null)[], t: number): Vec3 | null {
  const t0 = Math.floor(t);
  const alpha = t - t0;
  const here = track[t0] ?? null;
  const next = track[t0 + 1] ?? null;
  if (!here) return null;
  if (!next) return here;
  return [
    here[0] + (next[0] - here[0]) * alpha,
    here[1] + (next[1] - here[1]) * alpha,
    here[2] + (next[2] - here[2]) * alpha,
  ];
}

/** The block a dropped item is drawn as.
 *
 *  Item ids are not block ids. Most of what a redstone build drops has a block
 *  form under a predictable name, and the ones that do not (`diamond`, and the
 *  other gems and dusts) get theirs named here — the same table the native
 *  renderer uses. A shulker box passes straight through, because the mesher's
 *  block-entity path draws the real base + lid from `entity/shulker/*.png`
 *  rather than a cube. Anything with no block form at all returns null and is
 *  simply not drawn, which is honest: a stray cube in the wrong shape would
 *  claim the build drops something it does not. */
const GEM_BLOCKS = new Set([
  "diamond",
  "emerald",
  "lapis_lazuli",
  "coal",
  "redstone",
  "quartz",
  "amethyst_shard",
]);

export function itemDrawState(item: string): string | null {
  const bare = item.startsWith("minecraft:") ? item.slice(10) : item;
  if (bare.endsWith("shulker_box")) return `minecraft:${bare}`;
  if (GEM_BLOCKS.has(bare)) return `minecraft:${bare.replace("_shard", "")}_block`;
  if (bare.endsWith("_ingot")) return `minecraft:${bare.replace("_ingot", "_block")}`;
  // Blocks dropped as themselves: the id already names a block.
  if (
    bare.endsWith("_block") ||
    bare.endsWith("_concrete") ||
    bare.endsWith("_wool") ||
    bare.endsWith("_planks") ||
    bare.endsWith("_terracotta") ||
    ["stone", "obsidian", "glass", "sand", "gravel", "dirt", "target", "observer"].includes(bare)
  )
    return `minecraft:${bare}`;
  return null;
}
