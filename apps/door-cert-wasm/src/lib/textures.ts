/** Real Minecraft block textures for the isometric renderers.
 *
 * Assets live in public/pack/textures/ — 1,269 vanilla-named PNGs plus
 * index.json (array of available names, no extension). The index is fetched
 * once and used to test existence; no 404 probing. Textures are decoded once
 * into a per-name canvas (animated strips cropped to the top square frame,
 * optional tint pre-multiplied) and shared across frames. Anything that
 * doesn't resolve keeps the existing flat colour.
 */

import { DIR_OPP, parseState, type Dir } from "./cast";

export type Face = "top" | "side";

/** The three faces an isometric cube shows: +y, +z (screen-left panel) and
 * +x (screen-right panel). */
export type VisFace = "up" | "south" | "east";

export interface ResolvedTexture {
  name: string;
  /** Multiply tint (e.g. redstone dust). Baked into the cached canvas. */
  tint?: string;
  /** Inner-core texture drawn beneath `name`, inset 1/16 per side — the
   * vanilla honey/slime model: an opaque core inside a translucent shell.
   * Baked into the cached canvas. */
  core?: string;
  /** Flat colour painted under the core/shell stack so the translucent
   * shell reads as the block's material, not whatever sits behind it. */
  backing?: string;
}

const INDEX_URL = "/pack/textures/index.json";
const texUrl = (name: string) => `/pack/textures/${name}.png`;

const REDSTONE_TINT = "#c22f21";
/** Leaves ship greyscale and are tinted per biome at render time; plains
 *  foliage is the sane default for a schematic with no world around it. */
const FOLIAGE_TINT = "#79c05a";

/* ---------------------------------------------------------------- index -- */

let index: Set<string> | null = null;
let indexStarted = false;

const listeners = new Set<() => void>();
function notify(): void {
  for (const l of listeners) l();
}

/** Subscribe to "a texture became available" events (index load or image
 * decode). Returns an unsubscribe function. */
export function onTexturesChanged(cb: () => void): () => void {
  listeners.add(cb);
  return () => {
    listeners.delete(cb);
  };
}

/** Kick off the one-time index fetch (idempotent). */
export function ensureTextureIndex(): void {
  if (indexStarted) return;
  indexStarted = true;
  fetch(INDEX_URL)
    .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
    .then((names: string[]) => {
      index = new Set(names);
      notify();
    })
    .catch(() => {
      /* index unavailable — everything stays flat colour */
    });
}

const has = (n: string): boolean => index !== null && index.has(n);

/* ------------------------------------------------------------- resolver -- */

/** Candidate texture names for a block, in priority order. */
function candidates(name: string, face: Face): ResolvedTexture[] {
  const out: ResolvedTexture[] = [];
  const push = (...ns: string[]) => {
    for (const n of ns) out.push({ name: n });
  };

  switch (name) {
    case "sticky_piston":
      if (face === "top") push("piston_top_sticky", "piston_top");
      else push("piston_side");
      break;
    case "piston":
      if (face === "top") push("piston_top");
      else push("piston_side");
      break;
    case "piston_head":
    case "moving_piston":
      if (face === "top") push("piston_top");
      else push("piston_side");
      break;
    case "observer":
      if (face === "top") push("observer_top");
      else push("observer_front", "observer_side", "observer_top");
      break;
    case "redstone_wire":
      out.push({ name: "redstone_dust_dot", tint: REDSTONE_TINT });
      break;
    case "redstone_torch":
    case "redstone_wall_torch":
      push("redstone_torch");
      break;
    case "repeater":
    case "comparator":
      // top carries the device art; sides are the smooth-stone slab body.
      if (face === "top") push(name);
      else push("smooth_stone");
      break;
    case "hopper":
      if (face === "top") push("hopper_top");
      else push("hopper_outside");
      break;
    case "dropper":
    case "dispenser":
      if (face === "top") push("furnace_top");
      else push(`${name}_front`, "furnace_side");
      break;
    case "honey_block":
      // Vanilla model: translucent shell (honey_block_bottom on ALL outer
      // faces) over an opaque inner core (top/side art, inset 1/16). The
      // amber backing keeps it reading as honey, not the ground behind.
      out.push({
        name: "honey_block_bottom",
        core: face === "top" ? "honey_block_top" : "honey_block_side",
        backing: "rgba(234,160,49,0.9)",
      });
      if (face === "top") push("honey_block_top");
      else push("honey_block_side");
      break;
    case "slime_block":
      // Same two-element model as honey: shell + inner core, both from the
      // one slime_block texture; the double pass restores in-game density.
      out.push({ name: "slime_block", core: "slime_block", backing: "rgba(111,190,85,0.85)" });
      push("slime_block");
      break;
    case "quartz_block":
      if (face === "top") push("quartz_block_top");
      else push("quartz_block_side");
      break;
    case "smooth_quartz":
      push("quartz_block_bottom");
      break;
    case "quartz_pillar":
      if (face === "top") push("quartz_pillar_top");
      else push("quartz_pillar");
      break;
    case "target":
      if (face === "top") push("target_top");
      else push("target_side");
      break;
  }

  if (name.endsWith("_trapdoor")) push(name, "oak_trapdoor");
  if (name.endsWith("_leaves")) out.push({ name, tint: FOLIAGE_TINT });
  // "wood" is the six-sided log: it has no art of its own and wears the log's
  // bark on every face (stripped variants included).
  if (name.endsWith("_wood")) push(`${name.slice(0, -5)}_log`);

  // Generic resolution: face-specific first, then the plain name, then any
  // other face-ish suffix so e.g. furnace/side-only art still lands.
  push(
    `${name}_${face}`,
    name,
    `${name}_top`,
    `${name}_side`,
    `${name}_front`,
    `${name}_end`,
    `${name}_bottom`,
  );
  return out;
}

/* Memoized only once the index is loaded, so early nulls don't stick. */
const resolved = new Map<string, ResolvedTexture | null>();

/** Resolve a blockstate string + face to an available texture, or null. */
export function textureFor(blockState: string, face: Face): ResolvedTexture | null {
  if (!index) {
    ensureTextureIndex();
    return null;
  }
  const key = `${blockState}|${face}`;
  const memo = resolved.get(key);
  if (memo !== undefined) return memo;
  const name = blockState.replace(/^minecraft:/, "").replace(/\[.*$/, "");
  let hit: ResolvedTexture | null = null;
  for (const cand of candidates(name, face)) {
    if (has(cand.name) && (cand.core === undefined || has(cand.core))) {
      hit = cand;
      break;
    }
  }
  resolved.set(key, hit);
  return hit;
}

/* ------------------------------------------------ directional face art -- */

/** Which world direction the texture's content-up points for each visible
 * face's default (unrotated) UV frame, and the quarter-turn (clockwise) that
 * re-aims content-up at a given direction. Frames in the renderers:
 * up face: tex-x -> +x, tex-y -> +z (content-up = north);
 * south face: tex-x -> +x, tex-y -> down (content-up = up);
 * east face: tex-x -> -z (north), tex-y -> down (content-up = up). */
const ROT: Record<VisFace, Partial<Record<Dir, 0 | 1 | 2 | 3>>> = {
  up: { north: 0, east: 1, south: 2, west: 3 },
  south: { up: 0, east: 1, down: 2, west: 3 },
  east: { up: 0, north: 1, down: 2, south: 3 },
};

const FACE_NORMAL: Record<VisFace, Dir> = { up: "up", south: "south", east: "east" };

interface FaceSpec {
  name: string;
  rot: 0 | 1 | 2 | 3;
  tint?: string;
}

/** Vanilla per-face art for directional blocks: the face the block points at
 * gets the front texture, the opposite face the back texture, and the
 * remaining faces the side texture rotated so its axis follows `facing`. */
function directionalSpec(
  name: string,
  props: Record<string, string>,
  face: VisFace,
): FaceSpec | null {
  const facing = props.facing as Dir | undefined;
  if (facing === undefined || !(facing in DIR_OPP)) return null;
  const normal = FACE_NORMAL[face];
  const rot = ROT[face][facing] ?? 0;
  switch (name) {
    case "piston":
    case "sticky_piston": {
      if (facing === normal)
        return {
          name:
            props.extended === "true"
              ? "piston_inner"
              : name === "sticky_piston"
                ? "piston_top_sticky"
                : "piston_top",
          rot: 0,
        };
      if (facing === DIR_OPP[normal]) return { name: "piston_bottom", rot: 0 };
      return { name: "piston_side", rot };
    }
    case "piston_head": {
      if (facing === normal)
        return { name: props.type === "sticky" ? "piston_top_sticky" : "piston_top", rot: 0 };
      return { name: "piston_side", rot };
    }
    case "observer": {
      if (facing === normal) return { name: "observer_front", rot: 0 };
      if (facing === DIR_OPP[normal]) return { name: "observer_back", rot: 0 };
      if (face === "up" && facing !== "up" && facing !== "down")
        return { name: "observer_top", rot };
      return { name: "observer_side", rot };
    }
    case "dropper":
    case "dispenser": {
      const vertical = facing === "up" || facing === "down";
      if (facing === normal)
        return { name: vertical ? `${name}_front_vertical` : `${name}_front`, rot: 0 };
      if (face === "up") return { name: "furnace_top", rot: 0 };
      return { name: "furnace_side", rot: 0 };
    }
    default:
      return null;
  }
}

const faceSpecMemo = new Map<string, FaceSpec | null>();

function faceTexEntry(blockState: string, face: VisFace): TexEntry | null {
  if (!index) {
    ensureTextureIndex();
    return null;
  }
  const key = `${blockState}|f:${face}`;
  let spec = faceSpecMemo.get(key);
  if (spec === undefined) {
    const { name, props } = parseState(blockState);
    const s = directionalSpec(name, props, face);
    spec = s !== null && has(s.name) ? s : null;
    faceSpecMemo.set(key, spec);
  }
  if (spec) return load({ name: spec.name, tint: spec.tint }, spec.rot);
  const rt = textureFor(blockState, face === "up" ? "top" : "side");
  return rt ? load(rt) : null;
}

/** Per-visible-face texture canvas (facing-aware), or null until ready. */
export function faceCanvas(blockState: string, face: VisFace): HTMLCanvasElement | null {
  return faceTexEntry(blockState, face)?.canvas ?? null;
}

/** Per-visible-face texture data-URL for SVG <image>, or null until ready. */
export function faceURL(blockState: string, face: VisFace): string | null {
  return faceTexEntry(blockState, face)?.url ?? null;
}

/* ---------------------------------------------------------- image cache -- */

interface TexEntry {
  canvas: HTMLCanvasElement | null;
  url: string | null;
}

const cache = new Map<string, TexEntry>();

function load(rt: ResolvedTexture, rot: 0 | 1 | 2 | 3 = 0): TexEntry {
  const key = `${rt.name}|${rt.tint ?? ""}|${rt.core ?? ""}|${rt.backing ?? ""}|r${rot}`;
  let e = cache.get(key);
  if (e) return e;
  e = { canvas: null, url: null };
  cache.set(key, e);

  // Optional inner-core layer (honey/slime): loaded alongside the shell,
  // composed once both have decoded.
  let coreImg: HTMLImageElement | null = null;
  let coreReady = rt.core === undefined;
  if (rt.core !== undefined) {
    coreImg = new Image();
    coreImg.decoding = "async";
    coreImg.onload = () => {
      coreReady = true;
      compose();
    };
    coreImg.src = texUrl(rt.core);
  }

  const img = new Image();
  img.decoding = "async";
  let shellReady = false;
  const compose = () => {
    if (!shellReady || !coreReady) return;
    // Animated textures are vertical strips (height > width): crop to the
    // top square frame.
    const s = img.naturalWidth;
    let c = document.createElement("canvas");
    c.width = s;
    c.height = s;
    const g = c.getContext("2d");
    if (!g) return;
    g.imageSmoothingEnabled = false;
    if (rt.backing) {
      g.fillStyle = rt.backing;
      g.fillRect(0, 0, s, s);
    }
    if (coreImg) {
      // The core element is inset 1/16th of a block on every side.
      const inset = s / 16;
      const cs = coreImg.naturalWidth;
      g.drawImage(coreImg, 0, 0, cs, cs, inset, inset, s - 2 * inset, s - 2 * inset);
    }
    g.drawImage(img, 0, 0, s, s, 0, 0, s, s);
    if (rt.tint) {
      g.globalCompositeOperation = "multiply";
      g.fillStyle = rt.tint;
      g.fillRect(0, 0, s, s);
      // restore the source alpha the multiply pass flattened
      g.globalCompositeOperation = "destination-in";
      g.drawImage(img, 0, 0, s, s, 0, 0, s, s);
      g.globalCompositeOperation = "source-over";
    }
    if (rot !== 0) {
      // Bake the quarter-turn into its own cached canvas (per name+rot), so
      // both the canvas and SVG paths draw it with the plain UV frame.
      const r = document.createElement("canvas");
      r.width = s;
      r.height = s;
      const rg = r.getContext("2d");
      if (rg) {
        rg.imageSmoothingEnabled = false;
        rg.translate(s / 2, s / 2);
        rg.rotate((rot * Math.PI) / 2);
        rg.drawImage(c, -s / 2, -s / 2);
        c = r;
      }
    }
    e!.canvas = c;
    e!.url = c.toDataURL();
    notify();
  };
  img.onload = () => {
    shellReady = true;
    compose();
  };
  img.src = texUrl(rt.name);
  return e;
}

/** Decoded texture canvas for canvas renderers, or null until ready. */
export function textureCanvas(blockState: string, face: Face): HTMLCanvasElement | null {
  const rt = textureFor(blockState, face);
  return rt ? load(rt).canvas : null;
}

/** Data-URL of the decoded texture for SVG <image>, or null until ready. */
export function textureURL(blockState: string, face: Face): string | null {
  const rt = textureFor(blockState, face);
  return rt ? load(rt).url : null;
}

/* Per-face brightness overlays: textures replace the shaded flat fills, so
 * depth reading is preserved by compositing these on top of the texture. */
export const FACE_OVERLAY: Record<"top" | "left" | "right", string | null> = {
  top: "rgba(255,255,255,0.10)",
  left: "rgba(0,0,0,0.16)",
  right: "rgba(0,0,0,0.34)",
};
