/** The machine's META STRUCTURE — the vocabulary, shared by every view.
 *
 * `TickSimulation.machineGraphJson` (crates/mc-tick/src/machine_graph.rs) is a
 * STATIC analysis of the rest state: adhesion groups, resolved push sets, the
 * drive graph, and the minimal self-translating subgraph that *is* the engine.
 * Nothing here simulates anything, and nothing here is re-derived per tick.
 *
 * This module owns the role precedence, the palette and the node/edge geometry
 * so the flat per-y-layer panel and the 3-D overlay cannot drift apart. They
 * are two projections of ONE classification; if they ever disagreed, a reader
 * would have no way to tell which one lied.
 *
 * ------------------------------------------------------------------ colour --
 *
 * Four role classes, and four categorical hues cannot be made safe. Every cell
 * in a 3-D scene is adjacent to every other, so the ALL-PAIRS gate applies, and
 * the reference palette only validates its first three slots that way. Adding
 * the flat panel's grey as a fourth colour was measured, not assumed
 * (`validate_palette.js "#2f9e6b,#3d7fd6,#d99a2b,#8a8f98" --pairs all`, light
 * surface #f4f4f1):
 *
 *   [FAIL] chroma floor        #8a8f98 → 0.015 (reads gray)
 *   [FAIL] CVD separation      #8a8f98 ↔ #2f9e6b ΔE 5.7 (deutan)
 *   [FAIL] normal-vision floor #8a8f98 ↔ #2f9e6b ΔE 13.2 — below the hard 15
 *
 * A normal-vision ΔE under 15 is a failure no secondary encoding excuses, so
 * dead weight does not get a hue. It escapes by MARK, the way `boundary` does
 * in door-cert's `lib/xray.ts` and the doorway does in its `lib/doorway.ts`: a
 * hueless wireframe CAGE around the cell. Being achromatic it enters no colour
 * pair, so it needs no all-pairs check — and "neither driven nor driving" is
 * exactly the thing that should read as an annotation rather than a class.
 *
 * The three that do take hues keep the flat panel's hue IDENTITIES (green
 * engine, blue payload, amber kicker — colour follows the entity, never its
 * rank) but are re-stepped per mode, because dark mode is selected, not
 * flipped. The panel's own steps failed the dark lightness band (#d99a2b at
 * L 0.73). Measured, all-pairs:
 *
 *   light  #1baf7a,#2a78d6,#c07000 on #f4f4f1 → worst CVD ΔE 10.4, normal 22.0  PASS
 *   dark   #199e70,#3987e5,#c98500 on #0d0d0d → worst CVD ΔE  8.4, normal 19.8  PASS
 *
 * The light green carries a contrast WARN (2.55:1). That obligates relief, and
 * it has it: every role is named with its count in the legend, and the 3-D
 * marks are labelled in their tooltips. Colour is never the only cue.
 *
 * The GRAPH layer takes no hue at all. Its four edge kinds and two node kinds
 * are drawn in secondary ink and separated purely by mark, so the meta
 * structure can never be confused for a fifth and sixth role, and the role
 * palette keeps its whole budget. */

import { useEffect, useState } from "react";

/* --------------------------------------------------------------- schema --- */

export type DeviceKind = "sticky_piston" | "piston" | "observer" | "source";

/** Exactly the shape of `TickSimulation.machineGraphJson`. */
export interface MachineGraph {
  groups: Array<{ id: number; cells: [number, number, number][] }>;
  devices: Array<{
    id: number;
    pos: [number, number, number];
    kind: DeviceKind;
    facing: string;
    group: number;
    extended: boolean;
    can_extend: boolean;
    push: [number, number, number][];
    pull: [number, number, number][];
    influence: [number, number, number][];
  }>;
  edges: Array<{ kind: string; from: string; to: string }>;
  engines: Array<{ devices: number[]; cells: [number, number, number][] }>;
  payload: [number, number, number][];
  kickers: number[];
  dead_weight: [number, number, number][];
  rejections: Array<{ code: string; unconditional: boolean; reason: string }>;
  rejected: boolean;
  rejected_for_sustained: boolean;
  engine_search_truncated: boolean;
}

export type Cell = [number, number, number];

export const cellKey = ([x, y, z]: Cell) => `${x},${y},${z}`;

/* ----------------------------------------------------------------- role --- */

export type Role = "engine" | "payload" | "kicker" | "dead";

export interface RoleStyle {
  hex: number;
  label: string;
  hint: string;
  /** `tint` — the block itself is coloured.
   *  `cage` — a hueless wireframe box on the cell boundary; NOT a colour. */
  mark: "tint" | "cage";
}

/** Ordered weakest-claim first, which is also the legend order. */
export const ROLE_ORDER: Role[] = ["engine", "payload", "kicker", "dead"];

const ROLE_LIGHT: Record<Role, RoleStyle> = {
  engine: {
    hex: 0x1baf7a,
    label: "engine",
    hint: "the minimal set that shoves itself along",
    mark: "tint",
  },
  payload: {
    hex: 0x2a78d6,
    label: "payload",
    hint: "carried by the engine, does no work",
    mark: "tint",
  },
  kicker: {
    hex: 0xc07000,
    label: "kicker",
    hint: "fires once to start it, then irrelevant",
    mark: "tint",
  },
  dead: {
    hex: 0x898781,
    label: "dead weight",
    hint: "neither driven nor driving — marked by a cage, not a colour",
    mark: "cage",
  },
};

const ROLE_DARK: Record<Role, RoleStyle> = {
  engine: { ...ROLE_LIGHT.engine, hex: 0x199e70 },
  payload: { ...ROLE_LIGHT.payload, hex: 0x3987e5 },
  kicker: { ...ROLE_LIGHT.kicker, hex: 0xc98500 },
  dead: { ...ROLE_LIGHT.dead, hex: 0x898781 },
};

export function rolePalette(dark: boolean): Record<Role, RoleStyle> {
  return dark ? ROLE_DARK : ROLE_LIGHT;
}

/** Every classified cell, in STRUCTURE space, with its winning role.
 *
 * Painted weakest-claim first, so the last writer wins. The precedence is
 * engine > kicker > payload > dead, and the kicker/payload order is the part
 * that is easy to get wrong: a kicker usually sits INSIDE the push closure —
 * it is bolted to the machine it starts — so painting payload after it
 * silently repainted it blue and the panel under-reported kickers against the
 * JSON (0 shown against 2, 1 against 3). Being carried does not stop a device
 * being the thing that starts the machine. */
export function roleMap(graph: MachineGraph | null): Map<string, Role> {
  const map = new Map<string, Role>();
  if (!graph) return map;
  for (const cell of graph.dead_weight) map.set(cellKey(cell), "dead");
  for (const cell of graph.payload) map.set(cellKey(cell), "payload");
  for (const id of graph.kickers) {
    const d = graph.devices[id];
    if (d) map.set(cellKey(d.pos), "kicker");
  }
  // Engine last: a cell that is engine is engine, whatever else claimed it.
  for (const e of graph.engines)
    for (const cell of e.cells) map.set(cellKey(cell), "engine");
  return map;
}

export function roleCounts(roles: Map<string, Role>): Record<Role, number> {
  const counts: Record<Role, number> = {
    engine: 0,
    payload: 0,
    kicker: 0,
    dead: 0,
  };
  for (const role of roles.values()) counts[role] += 1;
  return counts;
}

/* ---------------------------------------------------------------- graph --- */

export type EdgeKind = "sticks_to" | "pushes" | "powers" | "observes";

export interface EdgeStyle {
  label: string;
  hint: string;
  /** `bond`  — a plain unadorned segment.
   *  `arrow` — a segment with a cone head at the target.
   *  `dash`  — a broken segment, long dashes.
   *  `dot`   — a broken segment, short dashes. */
  mark: "bond" | "arrow" | "dash" | "dot";
}

/** Why these marks and not four colours.
 *
 * The two edge kinds that matter most are `sticks_to` and `pushes`, and they
 * mean opposite things. `sticks_to` is STRUCTURE: a rigid, unconditional
 * membership — this device is part of that body, always, and the relation
 * carries no force and never fires. `pushes` is an ACTION: it happens only
 * when the device extends, it has a direction, and it is the entire reason the
 * machine moves. Drawing them in two colours would say they are two flavours
 * of the same thing. They are not, so they differ by mark instead: matter is
 * an unbroken segment, force is an arrow that points where the body goes.
 *
 * `powers` and `observes` are neither matter nor force — they are signal, so
 * they are BROKEN segments, and they separate from each other by dash length
 * (a source driving a piston is a longer-reaching claim than an observer
 * watching one cell). The split is therefore readable in two questions: is the
 * line solid (matter/force) or broken (signal), and does it have a head
 * (directed action) or not. No hue is spent, and none is available — see the
 * colour note at the top of this file. */
export const EDGE_STYLE: Record<EdgeKind, EdgeStyle> = {
  sticks_to: {
    label: "sticks to",
    hint: "rigid membership — this device rides that body, unconditionally",
    mark: "bond",
  },
  pushes: {
    label: "pushes",
    hint: "a resolved push plan — extending moves that body, arrow points at it",
    mark: "arrow",
  },
  powers: {
    label: "powers",
    hint: "signal reaches a piston's power region (quasi-connectivity included)",
    mark: "dash",
  },
  observes: {
    label: "observes",
    hint: "an observer watching the cell this device occupies",
    mark: "dot",
  },
};

export interface MetaNode {
  id: string;
  kind: "group" | "device";
  /** Cell-CENTRE in render space (structure space minus the corridor offset). */
  pos: [number, number, number];
  label: string;
  /** Cells in the group; 1 for a device. */
  size: number;
  /** Devices only. */
  facing?: string;
  device?: DeviceKind;
}

export interface MetaEdge {
  kind: EdgeKind;
  from: MetaNode;
  to: MetaNode;
}

export interface MetaStructure {
  nodes: MetaNode[];
  edges: MetaEdge[];
  /** Classified cells, in RENDER space, keyed for the tint layer. */
  roles: Map<string, Role>;
  counts: Record<Role, number>;
}

const centroid = (cells: Cell[]): [number, number, number] => {
  let x = 0,
    y = 0,
    z = 0;
  for (const c of cells) {
    x += c[0];
    y += c[1];
    z += c[2];
  }
  const n = Math.max(cells.length, 1);
  return [x / n, y / n, z / n];
};

const DEVICE_LABEL: Record<DeviceKind, string> = {
  sticky_piston: "sticky piston",
  piston: "piston",
  observer: "observer",
  source: "power source",
};

/** Derive the drawable meta structure.
 *
 * `xOff` is the corridor offset the SNBT builder adds (`ga/snbt.ts` X_OFF): the
 * graph speaks STRUCTURE space, the viewer's block list speaks genome space, so
 * every position is shifted back by it exactly once, here. Positions come out
 * at cell CENTRES because a block instance occupies [x, x+1]. */
export function metaStructure(
  graph: MachineGraph | null,
  xOff: number,
): MetaStructure {
  const roles = new Map<string, Role>();
  for (const [k, v] of roleMap(graph)) {
    const [x, y, z] = k.split(",").map(Number);
    roles.set(cellKey([x - xOff, y, z]), v);
  }
  const out: MetaStructure = {
    nodes: [],
    edges: [],
    roles,
    counts: roleCounts(roles),
  };
  if (!graph) return out;

  const byId = new Map<string, MetaNode>();
  for (const g of graph.groups) {
    const [cx, cy, cz] = centroid(g.cells);
    const node: MetaNode = {
      id: `g${g.id}`,
      kind: "group",
      pos: [cx - xOff + 0.5, cy + 0.5, cz + 0.5],
      label: `group ${g.id} — ${g.cells.length} cell${g.cells.length === 1 ? "" : "s"}`,
      size: g.cells.length,
    };
    byId.set(node.id, node);
    out.nodes.push(node);
  }
  for (const d of graph.devices) {
    const node: MetaNode = {
      id: `d${d.id}`,
      kind: "device",
      pos: [d.pos[0] - xOff + 0.5, d.pos[1] + 0.5, d.pos[2] + 0.5],
      label: `${DEVICE_LABEL[d.kind]} facing ${d.facing}`,
      size: 1,
      facing: d.facing,
      device: d.kind,
    };
    byId.set(node.id, node);
    out.nodes.push(node);
  }
  for (const e of graph.edges) {
    const from = byId.get(e.from);
    const to = byId.get(e.to);
    if (!from || !to) continue;
    if (!(e.kind in EDGE_STYLE)) continue;
    out.edges.push({ kind: e.kind as EdgeKind, from, to });
  }
  return out;
}

/* ---------------------------------------------------------------- theme --- */

const luminance = (hex: string): number => {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return 1;
  const n = parseInt(m[1], 16);
  return (
    (0.2126 * ((n >> 16) & 255) +
      0.7152 * ((n >> 8) & 255) +
      0.0722 * (n & 255)) /
    255
  );
};

/** Which mode the palette must be selected for.
 *
 * Read off the live `--page` custom property rather than `prefers-color-scheme`
 * so an explicit `data-theme` override is honoured without this module having
 * to know how the toggle is wired. */
export function isDarkTheme(): boolean {
  if (typeof document === "undefined") return false;
  const page = getComputedStyle(document.documentElement)
    .getPropertyValue("--page")
    .trim();
  return luminance(page || "#ffffff") < 0.5;
}

/** Secondary ink for the graph layer, per mode. Achromatic by design — it
 * enters no colour pair with the role hues. */
export function graphInk(dark: boolean): number {
  return dark ? 0xc3c2b7 : 0x52514e;
}

/** Recessive ink for the SIGNAL edges (`powers`, `observes`).
 *
 * A nine-block machine can carry twenty-plus edges and half of them are
 * signal — every source reaching every piston's power region. Drawn at the
 * same weight as the mechanical edges that is a hairball, and the first shot
 * of it was exactly that. So the layer gets one step of hierarchy: matter and
 * force at full ink, signal recessive and thinner. It is a lightness step, not
 * a hue — the graph layer still spends no colour, and the mark distinctions
 * (solid / arrow / long dash / short dash) still carry the identity.
 *
 * One value serves both modes because it is achromatic and mid-range: 4.01:1
 * on the light page (#f4f4f1) and 4.40:1 on the dark one (#0d0d0d). */
export function graphInkSoft(): number {
  return 0x7a7871;
}

export function useIsDark(): boolean {
  const [dark, setDark] = useState(isDarkTheme);
  useEffect(() => {
    const sync = () => setDark(isDarkTheme());
    sync();
    const mq = matchMedia("(prefers-color-scheme: dark)");
    mq.addEventListener("change", sync);
    const obs = new MutationObserver(sync);
    obs.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme", "class", "style"],
    });
    return () => {
      mq.removeEventListener("change", sync);
      obs.disconnect();
    };
  }, []);
  return dark;
}

export const hexCss = (hex: number) => "#" + hex.toString(16).padStart(6, "0");
