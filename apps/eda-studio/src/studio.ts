/** The studio's design-document state: a thin stateful shell around the
 * veneer `Design`. The document (wasm side) is the truth for geometry and
 * bus states; this class only tracks what the UI needs to draw handles —
 * cell library entries, instance transforms, declared ports and gates —
 * and re-reads everything else live from the engine after each mutation.
 */
import type { Core, VeneerSurface } from "./engine";

export type Vec3 = [number, number, number];

export interface CellInfo {
  name: string;
  schematic: any;
  dims: Vec3;
  warnings: string[];
  source: "file" | "verilog";
  /** Contract port summary for the library panel. */
  ports: { name: string; dir: "in" | "out"; width: number }[];
}

export interface InstanceInfo {
  name: string;
  cell: string;
  at: Vec3;
  rot: number;
}

export interface PortInfo {
  name: string;
  kind: "input" | "output";
  anchor: Vec3;
  step: Vec3;
  width: number;
  ty: string;
}

/** An endpoint contributed by a placed instance, straight from the engine's
 *  `instancePorts()`. `name` is `{instance}.{port}` — what routeBus takes.
 *
 *  `role` is the CELL-facing direction (`"output"` drives a bus). A cell
 *  contract names EXECUTOR hardware — levers/buttons in, lamps out — while a
 *  bus lands on DUST, so the engine derives `wires` by hardware scan. A port
 *  with no dust to tap (a bare lever input) is real IO that no bus can drive:
 *  `routable: false`, with the reason in `blocked`. */
export interface InstancePortInfo {
  name: string;
  instance: string;
  port: string;
  role: "input" | "output";
  ty: unknown;
  width: number;
  hardware: Vec3[];
  wires: Vec3[] | null;
  step: Vec3 | null;
  routable: boolean;
  blocked: string | null;
}

/** The cell's own contract ports, for the library listing. Reads the
 *  schematic's EMBEDDED contract (these cells are one artifact, schematic +
 *  contract); a cell without one simply lists nothing. */
export function cellPortSummary(schematic: any): CellInfo["ports"] {
  try {
    const c = JSON.parse(schematic.cellContractJson());
    const grab = (m: any, dir: "in" | "out") =>
      Object.entries(m ?? {}).map(([name, p]: [string, any]) => ({
        name, dir, width: (p.positions ?? []).length || 1,
      }));
    return [...grab(c.io?.inputs, "in"), ...grab(c.io?.outputs, "out")];
  } catch {
    return [];
  }
}

/** Human-readable type name for a label: `uint8`, `bool`, ... */
export function tyName(ty: unknown, width: number): string {
  if (typeof ty === "string") return ty.toLowerCase();
  if (ty && typeof ty === "object") {
    const o = ty as Record<string, any>;
    if (o.UnsignedInt) return `uint${o.UnsignedInt.bits}`;
    if (o.SignedInt) return `int${o.SignedInt.bits}`;
    if (o.Float) return "float";
    const k = Object.keys(o)[0];
    if (k) return `${k.toLowerCase()}${width > 1 ? width : ""}`;
  }
  return width > 1 ? `uint${width}` : "bool";
}

export interface GateInfo {
  name: string;
  anchor: Vec3;
  step: Vec3;
}

export interface BusInfo {
  name: string;
  driver: string;
  sinks: string[];
  gates: GateInfo[];
  color: number;
  /** Executor-only ports the ROUTER promoted so this bus could land (empty on
   *  cores that route without promoting, or that do not report it). */
  promotions?: unknown[];
}

export interface Block { x: number; y: number; z: number; name: string }

export interface LayerBlocks {
  layer: string; // "loose" | "bus:<name>" | "inst:<name>" | region name
  color: number | null; // bus color override, else null (block colors)
  failed: boolean;
  blocks: Block[];
  /** GEOMETRY REVISION: bumped whenever `blocks` actually changes content.
   *
   *  This is what makes the renderer self-correcting rather than dependent on
   *  a changed-set being perfect. `dirty.buses` is a HINT about what to re-read
   *  from wasm; `rev` is a FACT about what the blocks are. A renderer that
   *  caches a mesh per layer keys that cache on `rev`, so a layer whose
   *  geometry moved can never keep a stale mesh — even if the engine forgot to
   *  name it, or the app mis-keyed the name it was given. Comparing one
   *  integer per layer per frame costs nothing, and it does not cause rebuilds
   *  when nothing changed (that is the whole point of only bumping on a real
   *  content change). */
  rev: number;
}

/** One distinct thing to mesh: a library cell plus whatever per-instance
 *  port-mode patches are in effect, in CELL-LOCAL coordinates.
 *
 *  `Design::transform_pos` places a cell as `at + R_rot(p - cellBounds.min)`,
 *  where `cellBounds` is the min/max over the PRISTINE cell's non-air blocks
 *  and `R_rot` is a quarter-turn about the min corner. So the geometry below
 *  is rotation-independent: every placement of this variant, at every one of
 *  the four rotations, is the SAME mesh under a different matrix. */
export interface CellVariant {
  key: string;
  cell: string;
  /** `p - cellBounds.min` for each non-air block. */
  blocks: Block[];
  /** Rotation footprint `(sx, sy, sz)` from those same bounds. */
  size: Vec3;
  /** Geometry revision — same contract as `LayerBlocks.rev`. A variant KEY can
   *  outlive a change to its blocks (re-uploading a library cell under the same
   *  name), so the renderer keys its mesh cache on this, not on the key. */
  rev: number;
}

/** Where one instance puts a variant. Everything a renderer needs to build
 *  the instance matrix — and nothing that would make it re-mesh. */
export interface Placement {
  instance: string;
  cell: string;
  variant: string;
  at: Vec3;
  rot: number;
}

/** The renderable document, with an explicit statement of what changed since
 *  the previous call. The renderer rebuilds only the named parts; everything
 *  else it already holds is still valid by construction. */
export interface SceneModel {
  variants: Map<string, CellVariant>;
  placements: Placement[];
  /** Bus layers, world coordinates, one unique mesh each (never instanced —
   *  no two buses are the same shape). */
  buses: LayerBlocks[];
  /** The design's own loose hardware, world coordinates, one unique mesh. */
  loose: LayerBlocks;
  /** What was re-read since the last call — a PERFORMANCE HINT, not the
   *  renderer's source of truth. Correctness lives in the per-layer `rev`
   *  numbers: a renderer that trusts `dirty` alone shows a stale mesh the
   *  moment a changed-set is incomplete. */
  dirty: {
    /** Variants whose geometry must be (re)built. */
    variants: string[];
    /** Instance transforms changed: matrix writes, no re-mesh. */
    placements: boolean;
    /** Bus layers whose blocks changed. */
    buses: string[];
    loose: boolean;
  };
}

/** One instance in a clipboard, in coordinates RELATIVE to the copied group's
 *  origin, so a paste is a translation of the whole set.
 *
 *  A copy carries the CELL NAME, never the cell's blocks: an instance is a
 *  reference plus a transform (that is the whole model, and what makes ten
 *  placements one mesh), so pasting places another reference to the same
 *  library cell. `modes` carries the per-port Exec/Bus promotions, which are
 *  per-INSTANCE patches and therefore genuinely part of what was copied. */
export interface ClipInstance {
  /** The name it had when copied — the basis for the pasted name. */
  src: string;
  cell: string;
  rel: Vec3;
  rot: number;
  /** Ports promoted to Bus mode on the source instance. */
  busPorts: string[];
}

/** A bus whose DRIVER AND EVERY SINK were inside the copied set, so the
 *  pasted group can carry the same intent. Endpoint names are the SOURCE
 *  instance names; paste remaps them. A bus with an endpoint outside the set
 *  is not copied — half a bus is not a bus. */
export interface ClipBus {
  driver: string;
  sinks: string[];
  gates: { name: string; rel: Vec3; step: Vec3 }[];
}

export interface Clip {
  instances: ClipInstance[];
  buses: ClipBus[];
  /** Min corner of the copied instances, the origin `rel` is measured from. */
  origin: Vec3;
}

/** What a paste did, so the UI can report it in one sentence. */
export interface PasteReport {
  instances: { name: string; src: string; cell: string; at: Vec3 }[];
  buses: string[];
  /** Buses whose intent was recreated but could not route: name -> reason. */
  failed: Record<string, string>;
  /** Blocks the requested offset was nudged by to clear existing keepouts. */
  nudged: Vec3 | null;
  /** Why nothing (or less than everything) was pasted. */
  error?: string;
}

/** Monotonic across every document in the page — see `bumpRev`. */
let GEOMETRY_REV = 0;

const BUS_PALETTE = [0x76d275, 0x4fc3f7, 0xffb74d, 0xba68c8, 0x4db6ac, 0xf06292, 0xa2cf6e, 0x7986cb];

/** One reversible document edit.
 *
 *  The design layer offers `rip` / `reroute` / `remove_*` / `set_port_mode`
 *  but no document-level history, so the studio keeps an OPERATION JOURNAL:
 *  every mutator records the inverse call alongside the forward one. That is
 *  enough for the edits a user makes with the mouse (place, move, rotate,
 *  delete, route, rip, delete-bus, port-mode) and it is honest about the ones
 *  it cannot invert — `declarePort` and `addGate` have no engine inverse, so
 *  they are not journalled rather than pretending. */
interface Op {
  /** What the user did, in the past tense, for the button's tooltip. */
  label: string;
  undo: () => void;
  redo: () => void;
  /** Identity of a coalescing group: consecutive ops that share one (a drag) collapse. */
  coalesce?: string;
}

export class Studio {
  core: Core;
  d: VeneerSurface;
  design: any; // veneer Design
  name: string;
  cells = new Map<string, CellInfo>();
  instances = new Map<string, InstanceInfo>();
  ports = new Map<string, PortInfo>();
  buses = new Map<string, BusInfo>();
  baked: any | null = null; // veneer Flat
  executor: any | null = null;
  version = 0;
  onChange: (() => void) | null = null;
  private nextInst = 0;
  private nextBus = 0;

  constructor(core: Core, d: VeneerSurface, name = "studio", base: any = null) {
    this.core = core;
    this.d = d;
    this.name = name;
    this.design = base ? d.Design.forSchematic(name, base) : d.Design.create(name);
  }

  private bump() {
    this.version++;
    this.baked = null; // any edit invalidates the baked artifact
    this.executor = null;
    // Inside a transaction the notification is deferred to the commit: a paste
    // is a dozen edits and ONE thing that happened, so it should cost one
    // refresh. It is not only about speed — refreshing per inner edit makes the
    // renderer mesh states the user never sees (an instance placed, then
    // promoted port by port, is N throwaway cell variants).
    if (this.txns.length || this.deferNotify) { this.pendingNotify = true; return; }
    this.onChange?.();
  }

  private pendingNotify = false;
  private deferNotify = 0;

  /** Run `fn` with the change notification deferred to the end — one refresh
   *  for a group of edits. Used by transactions and by their inverses. */
  private batched(fn: () => void) {
    this.deferNotify++;
    try {
      fn();
    } finally {
      this.deferNotify--;
      this.flushNotify();
    }
  }

  private flushNotify() {
    if (this.deferNotify || this.txns.length || !this.pendingNotify) return;
    this.pendingNotify = false;
    this.onChange?.();
  }

  // -- undo / redo ----------------------------------------------------------

  private undoStack: Op[] = [];
  private redoStack: Op[] = [];
  /** While replaying an op, mutators must not journal their own inverse. */
  private replaying = 0;

  /** Open transactions, innermost last. While one is open every recorded op is
   *  collected into it instead of onto the undo stack. */
  private txns: Op[][] = [];

  /** Run `fn` as ONE undo step.
   *
   *  A paste is a dozen document edits (place, promote a port, route a bus) and
   *  exactly one thing the user did, so it has to be one press of ⌘Z. Rather
   *  than teach every mutator about grouping, the journal collects the inverses
   *  they already record and folds them into a single op: undo runs them in
   *  REVERSE (the buses come out before the instances they land on), redo runs
   *  them forward. Nested transactions flatten into the outermost one.
   *
   *  A throw does not lose the partial work: whatever was recorded so far is
   *  still committed as an undoable step, so a half-finished paste is
   *  reversible rather than stuck. */
  transaction<T>(label: string, fn: () => T): T {
    const own: Op[] = [];
    this.txns.push(own);
    let out: T;
    try {
      out = fn();
    } finally {
      const i = this.txns.indexOf(own);
      if (i >= 0) this.txns.splice(i, 1);
      this.flushNotify();
      if (own.length) {
        const ops = [...own];
        this.record({
          label,
          undo: () => this.batched(() => {
            for (let k = ops.length - 1; k >= 0; k--) ops[k].undo();
          }),
          redo: () => this.batched(() => {
            for (const op of ops) op.redo();
          }),
        });
      }
    }
    return out;
  }

  private record(op: Op) {
    if (this.replaying) return;
    // Inside a transaction the op belongs to the group, not to the stack.
    const txn = this.txns[this.txns.length - 1];
    if (txn) { txn.push(op); return; }
    const top = this.undoStack[this.undoStack.length - 1];
    if (op.coalesce && top?.coalesce === op.coalesce) {
      // A drag is one undo step, not sixty: keep the ORIGINAL undo (where the
      // gesture started) and take the newest redo (where it ended).
      top.redo = op.redo;
      top.label = op.label;
    } else {
      this.undoStack.push(op);
      if (this.undoStack.length > 200) this.undoStack.shift();
    }
    this.redoStack.length = 0;
  }

  /** Run an inverse without journalling it. */
  private replay(fn: () => void) {
    this.replaying++;
    try {
      fn();
    } finally {
      this.replaying--;
    }
  }

  /** End the current coalescing group, so the next edit starts a new undo
   *  step even if it touches the same instance (pointer up, key released). */
  endGesture() {
    const top = this.undoStack[this.undoStack.length - 1];
    if (top) top.coalesce = undefined;
  }

  canUndo(): boolean { return this.undoStack.length > 0; }
  canRedo(): boolean { return this.redoStack.length > 0; }
  undoLabel(): string | null { return this.undoStack[this.undoStack.length - 1]?.label ?? null; }
  redoLabel(): string | null { return this.redoStack[this.redoStack.length - 1]?.label ?? null; }

  /** Undo one edit; returns its label, or `null` when there is nothing to undo. */
  undo(): string | null {
    const op = this.undoStack.pop();
    if (!op) return null;
    op.coalesce = undefined;
    this.replay(op.undo);
    this.redoStack.push(op);
    return op.label;
  }

  redo(): string | null {
    const op = this.redoStack.pop();
    if (!op) return null;
    this.replay(op.redo);
    this.undoStack.push(op);
    return op.label;
  }

  /** A fresh document has no history to walk back into. */
  clearHistory() {
    this.undoStack.length = 0;
    this.redoStack.length = 0;
  }

  /** Buses that terminate on one of this instance's ports — what a delete
   *  takes with it, and therefore what the confirm prompt has to count. */
  busesOn(instance: string): string[] {
    const p = `${instance}.`;
    return [...this.buses.values()]
      .filter((b) => b.driver.startsWith(p) || b.sinks.some((s) => s.startsWith(p)))
      .map((b) => b.name);
  }

  /** Re-declare a bus from its stored intent (the inverse of a rip/remove). */
  private restoreBuses(list: BusInfo[]) {
    for (const b of list) {
      try {
        this.design.routeBus(b.name, { driver: b.driver, sinks: b.sinks, gates: b.gates });
        this.buses.set(b.name, b);
        this.dirtyBuses.add(b.name);
      } catch { /* the geometry it needed is gone; leave it out */ }
    }
  }

  /** Which of this instance's ports are in Bus mode, so an undo can put the
   *  promotions back before the buses that depend on them. */
  private promotedPortsOf(instance: string): string[] {
    const out: string[] = [];
    for (const [name, mode] of this.portModes()) {
      if (mode !== "bus") continue;
      if (name.startsWith(`${instance}.`)) out.push(name.slice(instance.length + 1));
    }
    return out;
  }

  // -- what changed, and therefore what must be re-read from wasm ----------
  //
  // The baseline profile said the whole cost of a drag frame was JSON coming
  // back across the wasm boundary: a full non-air dump (36 ms at 2.5k blocks)
  // plus one dense region dump PER INSTANCE (~14 ms each). None of it had
  // changed. These flags are how the scene stops asking.

  /** Buses whose realized blocks may have changed. */
  private dirtyBuses = new Set<string>();
  /** The loose layer's blocks may have changed. */
  private dirtyLoose = true;
  /** Instance transforms changed (cheap: matrix writes only). */
  private dirtyPlacements = true;
  /** Variants whose cell-local geometry may have changed under an unchanged
   *  KEY — re-uploading a library cell of the same name is the case that
   *  matters, and it used to leave the old body on screen forever. */
  private dirtyVariants = new Set<string>();

  /** Every bus is suspect — for the rare structural edits where working out
   *  which ones moved costs more than re-reading them. */
  private dirtyAllBuses() {
    for (const n of this.buses.keys()) this.dirtyBuses.add(n);
  }

  /** Re-read EVERYTHING next scene.
   *
   *  The safety net for the one failure mode a changed-set cannot survive: if
   *  the renderer throws half way through applying a scene, the flags that
   *  said what to re-read have already been consumed and the stale geometry is
   *  stale for good. A caller that fails to apply a scene calls this, and the
   *  next refresh heals the whole document. */
  invalidateAll() {
    this.dirtyAllBuses();
    this.dirtyLoose = true;
    this.dirtyPlacements = true;
    for (const k of this.variants.keys()) this.dirtyVariants.add(k);
  }

  /** A library cell's blocks changed (re-uploaded, re-compiled): drop the
   *  cached geometry AND flag every variant built from it. */
  invalidateCell(cell: string) {
    this.cellLocal.delete(cell);
    this.cellBounds.delete(cell);
    for (const [key, v] of this.variants) if (v.cell === cell) this.dirtyVariants.add(key);
    this.dirtyPlacements = true;
  }

  /** Content signature of a block list: order-independent enough to be honest
   *  (the engine emits a stable order) and cheap enough to run per changed
   *  layer. Used to bump a `rev` ONLY on a real change, so an over-eager
   *  changed-set costs a wasm read but never a re-mesh. */
  private static sig(blocks: Block[]): string {
    let h = 0x811c9dc5;
    for (const b of blocks) {
      // FNV-1a over the cell, name included: a block that changed IN PLACE
      // (dust -> repeater after a reroute) has to register as a change.
      for (const part of [b.x, b.y, b.z]) {
        h = ((h ^ (part & 0xffff)) * 0x01000193) >>> 0;
      }
      for (let i = 0; i < b.name.length; i++) {
        h = ((h ^ b.name.charCodeAt(i)) * 0x01000193) >>> 0;
      }
    }
    return `${blocks.length}:${h.toString(36)}`;
  }

  /** Per-layer geometry revisions and the signatures they are derived from. */
  private revs = new Map<string, { rev: number; sig: string }>();

  /** Record a layer's new blocks; returns its revision, bumped only if the
   *  content actually differs from what the layer held before.
   *
   *  Revisions come from a PROCESS-WIDE counter, not a per-layer one. Loading a
   *  new document builds a new `Studio`, and a per-layer counter would hand the
   *  new document's first `loose` layer revision 1 — the number a renderer that
   *  outlives the swap is already holding for the OLD document's empty base. It
   *  then keeps that empty mesh, because 1 === 1. (This is not hypothetical: it
   *  is what the consistency check caught first.) A monotonic counter cannot
   *  collide across documents. */
  private bumpRev(layer: string, blocks: Block[]): number {
    const sig = Studio.sig(blocks);
    const have = this.revs.get(layer);
    if (have && have.sig === sig) return have.rev;
    const rev = ++GEOMETRY_REV;
    this.revs.set(layer, { rev, sig });
    return rev;
  }

  private revOf(layer: string): number {
    return this.revs.get(layer)?.rev ?? 0;
  }

  /** Wasm round-trips the scene made, by kind. The performance assertions in
   *  `scripts/verify.mjs` are written against these: a drag must add zero
   *  `cellDump`s, a port-mode toggle exactly one. */
  sceneReads = { flatten: 0, cellDump: 0, instDump: 0, busDump: 0, looseDump: 0 };
  /** Wall time in each of those, so the report can name the engine call that
   *  still dominates a live-reroute drag frame. */
  sceneMs = { flatten: 0, cellDump: 0, instDump: 0, busDump: 0, looseDump: 0 };

  // -- library -------------------------------------------------------------

  addCellFromBytes(name: string, bytes: Uint8Array): CellInfo {
    const schematic = this.core.Schematic.fromData(Array.from(bytes));
    return this.addCellSchematic(name, schematic, "file");
  }

  addCellSchematic(name: string, schematic: any, source: CellInfo["source"]): CellInfo {
    const warnings = JSON.parse(this.design.addCell(name, schematic)) as string[];
    const dm = schematic.dimensions(); // Dimensions_obj: {x, y, z}
    const info: CellInfo = {
      name, schematic, source, warnings,
      dims: [dm.x, dm.y, dm.z] as Vec3,
      ports: cellPortSummary(schematic),
    };
    this.cells.set(name, info);
    // Re-uploading a cell under a name that is already placed keeps the variant
    // KEY (it is the cell name) while replacing its blocks — the one case where
    // "is this variant new?" is the wrong question to ask.
    this.invalidateCell(name);
    this.bump();
    return info;
  }

  // -- instances -----------------------------------------------------------

  placeInstance(cell: string, at: Vec3, rot = 0, forceName?: string): InstanceInfo {
    // A paste can take `u1` before the counter gets there, so the counter has
    // to skip what is already placed rather than collide with it.
    while (!forceName && this.instances.has(`u${this.nextInst}`)) this.nextInst++;
    const name = forceName ?? `u${this.nextInst++}`;
    this.design.place(name, cell, at, rot);
    const info: InstanceInfo = { name, cell, at: [...at] as Vec3, rot };
    this.instances.set(name, info);
    this.dirtyPlacements = true;
    this.dirtyAllBuses(); // a new body is a new obstacle
    this.record({
      label: `place ${name}`,
      undo: () => this.removeInstance(name),
      redo: () => this.placeInstance(cell, at, rot, name),
    });
    this.bump();
    return info;
  }

  /** Drag an instance: the move always lands (the document's truth); the
   *  affected buses reroute, failures stay visibly failed. */
  moveInstance(
    name: string, at: Vec3, rot?: number,
    /** `true` during a drag: fold this frame into the gesture's one undo step. */
    coalesce = false,
  ): { rerouted: string[]; failed: Record<string, string> } {
    const inst = this.instances.get(name);
    if (!inst) throw new Error(`no instance ${name}`);
    const was: Vec3 = [...inst.at] as Vec3;
    const wasRot = inst.rot;
    const to: Vec3 = [...at] as Vec3;
    const toRot = rot ?? inst.rot;
    const report = this.design.moveInstance(name, at, rot ?? inst.rot);
    if (was[0] !== to[0] || was[1] !== to[1] || was[2] !== to[2] || wasRot !== toRot) {
      this.record({
        label: wasRot !== toRot ? `rotate ${name} to ${toRot}°` : `move ${name}`,
        coalesce: coalesce ? `move:${name}` : undefined,
        undo: () => this.moveInstance(name, was, wasRot),
        redo: () => this.moveInstance(name, to, toRot),
      });
    }
    inst.at = [...at] as Vec3;
    if (rot != null) inst.rot = rot;
    // A move changes this instance's MATRIX and nothing else about its blocks.
    // The only geometry that can have changed is the buses the engine names in
    // the report — so a drag over open ground re-reads nothing at all.
    this.dirtyPlacements = true;
    for (const b of report.rerouted ?? []) this.dirtyBuses.add(b);
    for (const b of Object.keys(report.failed ?? {})) this.dirtyBuses.add(b);
    this.bump();
    return report;
  }

  /** Rotate in place by `delta` degrees (snapped to 90). */
  rotateInstance(name: string, delta = 90) {
    const inst = this.instances.get(name);
    if (!inst) throw new Error(`no instance ${name}`);
    const rot = (((inst.rot + delta) % 360) + 360) % 360;
    return { rot, ...this.moveInstance(name, inst.at, rot) };
  }

  /** Delete an instance. Buses that terminated on one of its ports are
   *  deleted with it (they lost an endpoint) and reported by name. */
  removeInstance(name: string): { removed_buses: string[]; rerouted: string[]; failed: Record<string, string> } {
    const info = this.instances.get(name);
    if (!info) throw new Error(`no instance ${name}`);
    // Everything the delete destroys, captured BEFORE it happens: the body,
    // which of its ports were promoted, and the buses that end on it.
    const snapshot = { ...info, at: [...info.at] as Vec3 };
    const promoted = this.promotedPortsOf(name);
    const carried = this.busesOn(name).map((n) => this.buses.get(n)!).filter(Boolean)
      .map((b) => ({ ...b, sinks: [...b.sinks], gates: b.gates.map((g) => ({ ...g })) }));
    const report = this.design.removeInstance(name);
    this.instances.delete(name);
    this.dirtyAllBuses();
    for (const b of report.removed_buses ?? []) this.buses.delete(b);
    this.dirtyPlacements = true;
    this.record({
      label: `delete ${name}`,
      undo: () => {
        this.design.place(name, snapshot.cell, snapshot.at, snapshot.rot);
        this.instances.set(name, { ...snapshot, at: [...snapshot.at] as Vec3 });
        this.dirtyPlacements = true;
        this.dirtyAllBuses();
        for (const p of promoted) {
          try { this.design.setPortMode(name, p, "bus"); } catch { /* cell changed */ }
        }
        this.modeCache = null;
        this.restoreBuses(carried);
        this.bump();
      },
      redo: () => this.removeInstance(name),
    });
    this.bump();
    return report;
  }

  // -- copy / paste ---------------------------------------------------------

  /** A free instance name derived from `src`: `u0` -> `u1` -> `u2`, and
   *  `add0` -> `add1`; a name with no trailing number gets `_copy`, `_copy2`.
   *  Never returns a name the document already uses. */
  uniqueInstanceName(src: string): string {
    const m = /^(.*?)(\d+)$/.exec(src);
    if (m) {
      let n = Number(m[2]) + 1;
      while (this.instances.has(`${m[1]}${n}`)) n++;
      return `${m[1]}${n}`;
    }
    if (!this.instances.has(`${src}_copy`)) return `${src}_copy`;
    let n = 2;
    while (this.instances.has(`${src}_copy${n}`)) n++;
    return `${src}_copy${n}`;
  }

  /** Copy instances (and the buses wholly inside them) to a clipboard value.
   *
   *  Buses are NOT copied for a single instance — a bus is a route between two
   *  endpoints and one of them would be outside the copy. For an AREA copy
   *  where both ends are inside, the INTENT is copied (driver, sinks, gates)
   *  and the paste re-routes it: the blocks belong to the router, not to the
   *  clipboard. */
  copy(names: string[]): Clip | null {
    const insts = names.map((n) => this.instances.get(n)).filter((i): i is InstanceInfo => !!i);
    if (!insts.length) return null;
    const origin: Vec3 = [
      Math.min(...insts.map((i) => i.at[0])),
      Math.min(...insts.map((i) => i.at[1])),
      Math.min(...insts.map((i) => i.at[2])),
    ];
    const inSet = new Set(insts.map((i) => i.name));
    const modes = this.portModes();
    const instances: ClipInstance[] = insts.map((i) => {
      const busPorts: string[] = [];
      for (const [name, mode] of modes) {
        if (mode !== "bus") continue;
        const dot = name.indexOf(".");
        if (name.slice(0, dot) === i.name) busPorts.push(name.slice(dot + 1));
      }
      return {
        src: i.name, cell: i.cell, rot: i.rot,
        rel: [i.at[0] - origin[0], i.at[1] - origin[1], i.at[2] - origin[2]] as Vec3,
        busPorts,
      };
    });
    const owner = (endpoint: string) => endpoint.slice(0, endpoint.indexOf("."));
    const inside = (endpoint: string) =>
      endpoint.includes(".") && inSet.has(owner(endpoint));
    const buses: ClipBus[] = [];
    for (const b of this.buses.values()) {
      if (!inside(b.driver) || !b.sinks.length || !b.sinks.every(inside)) continue;
      buses.push({
        driver: b.driver,
        sinks: [...b.sinks],
        gates: b.gates.map((g) => ({
          name: g.name,
          rel: [g.anchor[0] - origin[0], g.anchor[1] - origin[1], g.anchor[2] - origin[2]] as Vec3,
          step: [...g.step] as Vec3,
        })),
      });
    }
    return { instances, buses, origin };
  }

  /** Does an axis-aligned box overlap any placed instance's footprint? The
   *  engine refuses an overlapping `place` (that is its keepout), so this is
   *  only a first guess used to pick a landing spot without a throw-and-roll-
   *  back round trip for the obvious cases. */
  private footprint(cell: string, at: Vec3, rot: number): { min: Vec3; max: Vec3 } {
    const dims = this.cells.get(cell)?.dims ?? [1, 1, 1];
    const [w, h, l] = rot % 180 === 0 ? dims : [dims[2], dims[1], dims[0]];
    return { min: [...at] as Vec3, max: [at[0] + w - 1, at[1] + h - 1, at[2] + l - 1] as Vec3 };
  }

  private overlapsAnything(cell: string, at: Vec3, rot: number, ignore: Set<string>): boolean {
    const a = this.footprint(cell, at, rot);
    for (const i of this.instances.values()) {
      if (ignore.has(i.name)) continue;
      const b = this.footprint(i.cell, i.at, i.rot);
      if (a.min[0] <= b.max[0] && a.max[0] >= b.min[0] &&
          a.min[1] <= b.max[1] && a.max[1] >= b.min[1] &&
          a.min[2] <= b.max[2] && a.max[2] >= b.min[2]) return true;
    }
    return false;
  }

  /** Paste a clipboard so its origin lands at `at`.
   *
   *  ONE undo step (a transaction), relative transforms preserved, port modes
   *  carried over, and any bus whose two ends were both inside the copy
   *  re-routed for the new group. A bus that cannot route is left FAILED with
   *  the router's own reason rather than silently dropped — same philosophy as
   *  every other edit here.
   *
   *  Occupancy: the group is nudged along +X (then +Z) until no pasted body
   *  starts inside an existing one, because the engine refuses that placement
   *  and a refusal is not a useful answer to Ctrl-V. */
  paste(clip: Clip, at: Vec3): PasteReport {
    const report: PasteReport = { instances: [], buses: [], failed: {}, nudged: null };
    if (!clip.instances.length) return { ...report, error: "clipboard is empty" };
    // Pick a landing spot: the asked-for one, else step clear of the keepouts.
    const span = Math.max(
      1,
      ...clip.instances.map((c) => {
        const dims = this.cells.get(c.cell)?.dims ?? [1, 1, 1];
        return c.rel[0] + Math.max(dims[0], dims[2]);
      }),
    );
    let base: Vec3 | null = null;
    const tries: Vec3[] = [];
    for (let k = 0; k < 24; k++) {
      const step = Math.ceil(k / 2);
      tries.push(k === 0 ? ([...at] as Vec3)
        : k % 2 === 1 ? [at[0] + step * (span + 2), at[1], at[2]] as Vec3
        : [at[0], at[1], at[2] + step * (span + 2)] as Vec3);
    }
    const ignore = new Set<string>();
    for (const cand of tries) {
      const clash = clip.instances.some((c) => this.overlapsAnything(
        c.cell,
        [cand[0] + c.rel[0], cand[1] + c.rel[1], cand[2] + c.rel[2]] as Vec3,
        c.rot, ignore));
      if (!clash) { base = cand; break; }
    }
    if (!base) return { ...report, error: "no free space within 24 offsets — move the view and try again" };
    if (base[0] !== at[0] || base[2] !== at[2]) {
      report.nudged = [base[0] - at[0], base[1] - at[1], base[2] - at[2]] as Vec3;
    }
    const landing = base;

    return this.transaction(
      `paste ${clip.instances.length} instance${clip.instances.length === 1 ? "" : "s"}`,
      () => {
        const remap = new Map<string, string>();
        for (const c of clip.instances) {
          const name = this.uniqueInstanceName(c.src);
          const to: Vec3 = [landing[0] + c.rel[0], landing[1] + c.rel[1], landing[2] + c.rel[2]];
          try {
            this.placeInstance(c.cell, to, c.rot, name);
          } catch (err) {
            report.error = `${c.src}: ${err}`;
            continue;
          }
          remap.set(c.src, name);
          report.instances.push({ name, src: c.src, cell: c.cell, at: to });
          // Port modes are per-instance patches, so they are part of the copy.
          for (const port of c.busPorts) {
            try { this.setPortMode(name, port, "bus"); } catch { /* cell changed */ }
          }
        }
        const move = (endpoint: string) => {
          const dot = endpoint.indexOf(".");
          const to = remap.get(endpoint.slice(0, dot));
          return to ? `${to}${endpoint.slice(dot)}` : null;
        };
        for (const b of clip.buses) {
          const driver = move(b.driver);
          const sinks = b.sinks.map(move);
          if (!driver || sinks.some((s) => !s)) continue; // an end failed to place
          const gates: GateInfo[] = b.gates.map((g) => ({
            name: g.name,
            anchor: [landing[0] + g.rel[0], landing[1] + g.rel[1], landing[2] + g.rel[2]] as Vec3,
            step: [...g.step] as Vec3,
          }));
          try {
            const bus = this.routeBus(driver, sinks as string[], gates);
            report.buses.push(bus.name);
            const st = this.busStateDetail(bus.name);
            if (st.state === "failed") report.failed[bus.name] = st.reason ?? "unroutable";
          } catch (err) {
            // The declaration itself was refused: name it, do not hide it.
            report.failed[`${driver} → ${sinks.join(", ")}`] = String(err).replace(/^Error:\s*/, "");
          }
        }
        return report;
      });
  }

  /** Cut: copy, then delete the originals as ONE undo step. */
  cut(names: string[]): Clip | null {
    const clip = this.copy(names);
    if (!clip) return null;
    this.transaction(`cut ${names.length} instance${names.length === 1 ? "" : "s"}`, () => {
      for (const n of names) {
        try { this.removeInstance(n); } catch { /* already gone */ }
      }
    });
    return clip;
  }

  // -- instance ports ------------------------------------------------------

  /** Every endpoint the placed instances expose, live from the engine. */
  instancePorts(): InstancePortInfo[] {
    if (this.instances.size === 0) return [];
    try {
      return this.design.instancePorts() as InstancePortInfo[];
    } catch {
      return [];
    }
  }

  /** Declared design ports and instance ports in one connectable list. */
  allEndpoints(): {
    name: string; port: string; kind: "input" | "output"; anchor: Vec3; step: Vec3;
    width: number; ty: string; routable: boolean; blocked?: string; instance?: string;
    /** Port mode, for instance ports only: `"bus"` once promoted. */
    mode?: "bus" | "executor";
    /** Whether this port CAN be promoted (and what that would do), so a UI can
     *  offer the toggle rather than only reporting the refusal. */
    promotable?: boolean;
  }[] {
    const out = [...this.ports.values()].map((p) => ({
      name: p.name, port: p.name, kind: p.kind, anchor: p.anchor, step: p.step,
      width: p.width, ty: p.ty === "bool" ? "bool" : `${p.ty}${p.width}`,
      routable: true as boolean, blocked: undefined as string | undefined,
      instance: undefined as string | undefined,
      mode: undefined as "bus" | "executor" | undefined,
      promotable: false as boolean,
    }));
    for (const ip of this.instancePorts()) {
      // The engine's `role` is CELL-facing; the canvas speaks FABRIC
      // direction, where a cell output is what drives a bus.
      const kind: "input" | "output" = ip.role === "output" ? "input" : "output";
      const anchor = (ip.wires?.[0] ?? ip.hardware[0]) as Vec3;
      const mode = this.portMode(ip.instance, ip.port);
      out.push({
        name: ip.name, port: ip.port, kind, anchor,
        step: (ip.step ?? [0, 2, 0]) as Vec3,
        width: ip.width, ty: tyName(ip.ty, ip.width),
        routable: ip.routable, blocked: ip.blocked ?? undefined,
        instance: ip.instance,
        mode,
        promotable: mode === "bus" || this.canPromote(ip.instance, ip.port),
      });
    }
    return out;
  }

  // -- port modes (promotion) ----------------------------------------------

  /** Which ports have been switched out of executor mode. Cached per document
   *  version so the outliner can ask per chip without hitting wasm each time. */
  private modeCache: { version: number; map: Map<string, "bus" | "executor"> } | null = null;

  portModes(): Map<string, "bus" | "executor"> {
    if (this.modeCache?.version === this.version) return this.modeCache.map;
    const map = new Map<string, "bus" | "executor">();
    for (const e of this.design.portModes() as Array<{ name: string; mode: "bus" | "executor" }>) {
      map.set(e.name, e.mode);
    }
    this.modeCache = { version: this.version, map };
    return map;
  }

  portMode(instance: string, port: string): "bus" | "executor" {
    return this.portModes().get(`${instance}.${port}`) ?? "executor";
  }

  /** Would promoting this port work? Asks the engine to PLAN the patch, which
   *  is where every refusal (a ceiling lever, no room for a form adapter) comes
   *  from — so the UI never offers a toggle that cannot fire. */
  private promoCache = new Map<string, boolean>();
  canPromote(instance: string, port: string): boolean {
    // Keyed by CELL, not by document version: `plan_input`/`plan_output` are
    // planned against the library cell's own schematic and its port mapping,
    // so the answer cannot change as the document is edited. Keying it by
    // version (as this did) re-planned every port on every drag frame — 12 ms
    // of the old 21 ms `allEndpoints()`.
    const cell = this.instances.get(instance)?.cell ?? instance;
    const key = `${cell}.${port}`;
    const hit = this.promoCache.get(key);
    if (hit != null) return hit;
    let ok = false;
    try {
      this.design.planPortPromotion(instance, port);
      ok = true;
    } catch {
      ok = false;
    }
    this.promoCache.set(key, ok);
    return ok;
  }

  /** Toggle a port between executor hardware and a routable dust input. The
   *  engine rips any bus that terminated on it (its endpoint stops existing)
   *  and co-reroutes the rest. */
  setPortMode(instance: string, port: string, mode: "bus" | "executor") {
    const before = this.portMode(instance, port);
    const report = this.design.setPortMode(instance, port, mode);
    const ripped = ((report.removed_buses ?? []) as string[])
      .map((n) => this.buses.get(n)!).filter(Boolean)
      .map((b) => ({ ...b, sinks: [...b.sinks], gates: b.gates.map((g) => ({ ...g })) }));
    this.dirtyAllBuses();
    for (const name of report.removed_buses ?? []) this.buses.delete(name);
    if (before !== mode) {
      this.record({
        label: `${instance}.${port} → ${mode === "bus" ? "Bus" : "Executor"}`,
        undo: () => {
          this.setPortMode(instance, port, before);
          this.restoreBuses(ripped);
          this.bump();
        },
        redo: () => this.setPortMode(instance, port, mode),
      });
    }
    // This is the ONE edit that changes a placed cell's blocks, so it is the
    // one edit that costs a cell re-mesh — of the affected instance's variant
    // only. Every other instance keeps the mesh it already has.
    this.dirtyPlacements = true;
    this.bump();
    return report;
  }

  togglePortMode(instance: string, port: string) {
    const next = this.portMode(instance, port) === "bus" ? "executor" : "bus";
    return this.setPortMode(instance, port, next);
  }

  // -- ports ---------------------------------------------------------------

  declarePort(port: PortInfo): void {
    if (port.kind === "input") this.design.declareInput(port.name, port);
    else this.design.declareOutput(port.name, port);
    this.ports.set(port.name, port);
    this.bump();
  }

  // -- buses ---------------------------------------------------------------

  routeBus(driver: string, sinks: string[], gates: GateInfo[] = [], forceName?: string): BusInfo {
    const name = forceName ?? `bus${this.nextBus++}`;
    const color = BUS_PALETTE[this.buses.size % BUS_PALETTE.length];
    const bus = this.design.routeBus(name, { driver, sinks, gates });
    const info: BusInfo = {
      name, driver, sinks: [...sinks], color,
      gates: gates.map((g, i) => ({ ...g, name: g.name || `g${i}` })),
      // Ports the ROUTER promoted on its own, when the core reports them.
      promotions: bus.promotions ?? [],
    };
    this.buses.set(name, info);
    // Realizing a bus can amend the buses it crosses, so they are suspect too.
    this.dirtyAllBuses();
    this.record({
      label: `route ${name}`,
      undo: () => this.removeBus(name),
      redo: () => this.routeBus(driver, sinks, gates, name),
    });
    this.bump();
    return info;
  }

  /** Per-bus timing from the engine's STA machinery: `{per_bit_rt, skew_rt,
   *  max_rt}` in redstone ticks, or `null` for a bus that has not realized. */
  busSkew(name: string): { per_bit_rt: number[]; skew_rt: number; max_rt: number } | null {
    try {
      const s = this.design.busSkew(name);
      return s && typeof s.max_rt === "number" ? s : null;
    } catch {
      return null;
    }
  }

  busState(name: string): string {
    return this.design.busState(name);
  }

  // -- gates (bus checkpoints) ----------------------------------------------
  //
  // A GATE and an ENDPOINT are different things and the difference is the
  // whole point: an endpoint is netlist ("this bus drives that port"), a gate is
  // ROUTE ("and it must pass through here"). So removing a gate must leave the
  // net alone and let the router take a straighter path, while removing an
  // endpoint changes what the design means. Nothing below ever conflates them.
  //
  // Two engine paths, and which one is used matters for speed only:
  //   * `Design::add_gate` / `remove_gate` keep the untouched segments and
  //     re-route only the affected span — the fast path;
  //   * re-declaring the bus with a new gate LIST routes the whole thing.
  // The fast path is tried first and the fallback is exact, so a core without
  // `remove_gate` (or `add_gate` on a bus between placed cells, which it
  // refuses today) still gets working gates rather than a greyed button.

  /** Anchor of an endpoint by name, declared port or instance port. */
  private anchorOf(endpoint: string): Vec3 | null {
    const declared = this.ports.get(endpoint);
    if (declared) return declared.anchor;
    for (const ip of this.instancePorts()) {
      if (ip.name === endpoint) return (ip.wires?.[0] ?? ip.hardware[0]) ?? null;
    }
    return null;
  }

  /** Where a new gate belongs in the trunk order: the waypoint pair whose
   *  midpoint it is nearest, exactly the rule `Design::add_gate` uses, so the
   *  fast path and the fallback agree on the resulting route. */
  private gateInsertIndex(bus: BusInfo, anchor: Vec3): number {
    const first = this.anchorOf(bus.driver);
    const last = this.anchorOf(bus.sinks[0]);
    if (!first || !last) return bus.gates.length;
    const wps: Vec3[] = [first, ...bus.gates.map((g) => g.anchor), last];
    let best = 0, bestD = Infinity;
    for (let i = 0; i + 1 < wps.length; i++) {
      const mid = [
        Math.trunc((wps[i][0] + wps[i + 1][0]) / 2),
        Math.trunc((wps[i][1] + wps[i + 1][1]) / 2),
        Math.trunc((wps[i][2] + wps[i + 1][2]) / 2),
      ];
      const d = Math.abs(anchor[0] - mid[0]) + Math.abs(anchor[1] - mid[1]) + Math.abs(anchor[2] - mid[2]);
      if (d < bestD) { best = i; bestD = d; }
    }
    return best;
  }

  /** Re-declare a bus with a different gate list, keeping its name, colour and
   *  endpoints. This is what makes a gate removal STRAIGHTEN the route: the
   *  router plans A→C afresh instead of keeping the detour's segments. */
  private redeclareBus(name: string, gates: GateInfo[], label: string): string {
    const bus = this.buses.get(name);
    if (!bus) throw new Error(`no bus ${name}`);
    const color = bus.color;
    const driver = bus.driver;
    const sinks = [...bus.sinks];
    const was = bus.gates.map((g) => ({ ...g }));
    this.transaction(label, () => {
      this.removeBus(name);
      try {
        this.routeBus(driver, sinks, gates.map((g) => ({ ...g })), name);
      } catch (err) {
        // Re-declaring must never LOSE the bus. Put the old declaration back
        // (failed is a state; missing is data loss) and report the refusal.
        this.routeBus(driver, sinks, was, name);
        throw err;
      }
      const now = this.buses.get(name);
      if (now) now.color = color; // the palette index moved; the identity did not
    });
    this.dirtyAllBuses();
    return this.busState(name);
  }

  /** Add a checkpoint the route must pass through. Returns where it landed in
   *  the trunk order and which engine path was taken. */
  addGate(busName: string, anchor: Vec3, step: Vec3): string {
    const r = this.addGateAt(busName, anchor, step);
    return r.state;
  }

  addGateAt(busName: string, anchor: Vec3, step: Vec3):
    { state: string; index: number; fast: boolean; name: string } {
    const bus = this.buses.get(busName);
    if (!bus) throw new Error(`no bus ${busName}`);
    // Names have to be unique for the lifetime of the bus, not just now: `g0`
    // freed by a removal must not be reused while the engine still has it.
    let n = bus.gates.length;
    while (bus.gates.some((g) => g.name === `g${n}`)) n++;
    const gname = `g${n}`;
    const index = this.gateInsertIndex(bus, anchor);
    const gate: GateInfo = { name: gname, anchor: [...anchor] as Vec3, step: [...step] as Vec3 };
    try {
      const state = this.design.addGate(busName, gname, anchor, step);
      bus.gates.splice(index, 0, gate);
      this.dirtyBuses.add(busName);
      this.record({
        label: `add gate ${busName}/${gname}`,
        undo: () => { this.removeGate(busName, index); },
        redo: () => { this.addGateAt(busName, anchor, step); },
      });
      this.bump();
      return { state, index, fast: true, name: gname };
    } catch (err) {
      // The core refused the fast path (today: any bus whose endpoints are
      // instance ports). Re-declare with the full list — same route, more work.
      const next = [...bus.gates];
      next.splice(index, 0, gate);
      const state = this.redeclareBus(busName, next, `add gate ${busName}/${gname}`);
      if (this.busState(busName).startsWith("failed") && !state.startsWith("failed")) {
        throw err; // the fallback did not actually work; do not claim it did
      }
      return { state, index, fast: false, name: gname };
    }
  }

  /** Remove a checkpoint. The bus KEEPS its endpoints and re-routes over the
   *  merged span, so the path genuinely straightens. */
  removeGate(busName: string, index: number): { state: string; removed: GateInfo; fast: boolean } {
    const bus = this.buses.get(busName);
    if (!bus) throw new Error(`no bus ${busName}`);
    const removed = bus.gates[index];
    if (!removed) throw new Error(`bus ${busName} has no gate at ${index}`);
    const d = this.design as { removeGate?: (b: string, i: number) => string };
    if (typeof d.removeGate === "function") {
      try {
        const state = d.removeGate(busName, index);
        bus.gates.splice(index, 1);
        this.dirtyBuses.add(busName);
        this.record({
          label: `remove gate ${busName}/${removed.name}`,
          undo: () => { this.addGateAt(busName, removed.anchor, removed.step); },
          redo: () => { this.removeGate(busName, index); },
        });
        this.bump();
        return { state, removed, fast: true };
      } catch { /* fall through to the re-declare */ }
    }
    const next = bus.gates.filter((_, i) => i !== index);
    const state = this.redeclareBus(busName, next, `remove gate ${busName}/${removed.name}`);
    return { state, removed, fast: false };
  }

  /** `{gates, segments}` — the model made visible: N gates means N+1 trunk
   *  spans, which is why removing one can only shorten the route. */
  gateSummary(busName: string): { gates: GateInfo[]; segments: number } {
    const bus = this.buses.get(busName);
    return { gates: bus ? bus.gates.map((g) => ({ ...g })) : [], segments: (bus?.gates.length ?? 0) + 1 };
  }

  /** Drag a gate: exactly its two adjacent segments rip and reroute. */
  moveGate(busName: string, gateName: string, anchor: Vec3): { state: string; rerouted_segments: number } {
    const bus = this.buses.get(busName);
    const gate = bus?.gates.find((g) => g.name === gateName);
    if (!bus || !gate) throw new Error(`no gate ${busName}/${gateName}`);
    const was = [...gate.anchor] as Vec3;
    const report = this.design.moveGate(busName, gateName, anchor);
    gate.anchor = [...anchor] as Vec3;
    this.dirtyBuses.add(busName);
    this.record({
      label: `move gate ${busName}/${gateName}`,
      coalesce: `gate:${busName}/${gateName}`,
      undo: () => this.moveGate(busName, gateName, was),
      redo: () => this.moveGate(busName, gateName, [...anchor] as Vec3),
    });
    this.bump();
    return report;
  }

  ripBus(name: string): void {
    this.design.rip(name);
    this.dirtyBuses.add(name);
    this.record({
      label: `rip ${name}`,
      undo: () => { this.rerouteBus(name); },
      redo: () => this.ripBus(name),
    });
    this.bump();
  }

  /** Try the bus again from its stored declaration (heal after the blocker
   *  moved). Returns the resulting state, `failed: reason` included. */
  rerouteBus(name: string): string {
    const state = this.design.reroute(name);
    this.dirtyBuses.add(name);
    this.bump();
    return state;
  }

  /** Delete a bus outright, freeing its name. */
  removeBus(name: string): void {
    const info = this.buses.get(name);
    const snapshot = info
      ? { ...info, sinks: [...info.sinks], gates: info.gates.map((g) => ({ ...g })) }
      : null;
    this.design.removeBus(name);
    this.buses.delete(name);
    this.dirtyBuses.add(name);
    if (snapshot) {
      this.record({
        label: `delete ${name}`,
        undo: () => { this.restoreBuses([snapshot]); this.bump(); },
        redo: () => this.removeBus(name),
      });
    }
    this.bump();
  }

  /** `{state, reason}` — `busState()` returns `failed: <reason>` as one
   *  string; the panel wants the halves apart. */
  busStateDetail(name: string): { state: "routed" | "intended" | "failed"; reason?: string } {
    const raw = this.busState(name);
    if (raw.startsWith("failed")) {
      return { state: "failed", reason: raw.replace(/^failed:?\s*/, "") || "unroutable" };
    }
    return { state: raw as "routed" | "intended" };
  }

  // -- textured view ---------------------------------------------------------

  /** A resource pack loaded from a user-supplied ZIP, kept for re-meshing. */
  pack: any | null = null;
  packInfo: { blockstates: number; models: number; textures: number } | null = null;
  private meshVersion = -1;
  private meshCache: ArrayBuffer | null = null;

  loadPack(bytes: Uint8Array): void {
    this.pack = this.core.ResourcePack.fromBytes(Array.from(bytes));
    this.packInfo = {
      blockstates: this.pack.blockstateCount(),
      models: this.pack.modelCount(),
      textures: this.pack.textureCount(),
    };
    this.meshVersion = -1;
    this.meshCache = null;
  }

  /** Mesh the COMPOSITED design to a GLB against the loaded pack.
   *
   *  Composited, not layered: `.glb` has no layer concept, and the region
   *  merge would drop named-layer cells shadowed by the loose layer's
   *  bounding box. Cached per document version so orbiting is free and only
   *  a real edit re-meshes. */
  meshGlb(): ArrayBuffer | null {
    if (!this.pack) return null;
    if (this.meshCache && this.meshVersion === this.version) return this.meshCache;
    const flat = this.design.flattenComposite();
    const cfg = this.core.MeshConfig.create();
    const mesh = this.core.MeshResult.create(flat.raw, this.pack, cfg);
    const b64 = mesh.glbDataB64();
    const bin = atob(b64);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    this.meshCache = bytes.buffer;
    this.meshVersion = this.version;
    return this.meshCache;
  }

  // -- loose layer -----------------------------------------------------------

  setBlock(x: number, y: number, z: number, block: string): void {
    this.design.setBlock(x, y, z, block);
    this.dirtyLoose = true;
    // no bump: demo hardware placement calls this in bulk, caller bumps
  }

  // -- lifecycle -------------------------------------------------------------

  check(): any {
    return this.design.check();
  }

  bake(budget = 4000): any {
    this.baked = this.design.bake(budget);
    this.executor = this.baked.executor(budget);
    this.onChange?.();
    return this.baked;
  }

  /** Ports of the baked artifact's embedded contract (for the poke panel). */
  contractPorts(): { inputs: { name: string; width: number }[]; outputs: { name: string; width: number }[] } {
    const empty = { inputs: [], outputs: [] };
    if (!this.baked) return empty;
    try {
      const resolved = JSON.parse(this.baked.raw.resolveCellContractJson());
      const io = resolved.contract?.io ?? {};
      const list = (kind: string) =>
        Object.entries(io[kind] ?? {}).map(([name, p]: [string, any]) => ({
          name, width: p.width ?? (p.bits?.length ?? 1),
        }));
      return { inputs: list("inputs"), outputs: list("outputs") };
    } catch {
      return empty;
    }
  }

  // -- persistence -----------------------------------------------------------

  exportBytes(suffix: "schem" | "litematic" | "nucm"): Uint8Array {
    return this.design.toBytes(`design.${suffix}`);
  }

  // -- scene extraction ------------------------------------------------------

  /** Per-cell rotation bounds, mirroring the engine's `cell_bounds`: the
   *  min/max over the PRISTINE cell's non-air blocks (NOT the region bounding
   *  box, which can include air). */
  private cellBounds = new Map<string, { min: Vec3; size: Vec3 }>();
  /** Cell-local blocks of an unpatched cell, straight from its own schematic
   *  (~1 ms) — no flatten, no instance region dump. */
  private cellLocal = new Map<string, Block[]>();
  /** Built variants, keyed by `variantKey`. */
  private variants = new Map<string, CellVariant>();
  private busBlocks = new Map<string, Block[]>();
  private looseBlocks: Block[] = [];

  /** Which variant an instance draws: its cell, plus the ports it has
   *  promoted. `Design::instance_local_blocks` is exactly "the library body
   *  PLUS its Bus-mode port patches", so two instances agreeing on both agree
   *  on every block — and can share one mesh. */
  variantKey(instance: string): string {
    const inst = this.instances.get(instance);
    if (!inst) return "";
    const promoted: string[] = [];
    for (const [name, mode] of this.portModes()) {
      if (mode !== "bus") continue;
      const dot = name.indexOf(".");
      if (name.slice(0, dot) === instance) promoted.push(name.slice(dot + 1));
    }
    promoted.sort();
    return promoted.length ? `${inst.cell}#${promoted.join(",")}` : inst.cell;
  }

  /** Where every instance sits — pure JS state, no wasm call. */
  placements(): Placement[] {
    return [...this.instances.values()].map((i) => ({
      instance: i.name, cell: i.cell, variant: this.variantKey(i.name),
      at: i.at, rot: i.rot,
    }));
  }

  private boundsFor(cell: string): { min: Vec3; size: Vec3 } {
    const hit = this.cellBounds.get(cell);
    if (hit) return hit;
    const blocks = this.cellBlocks(cell);
    let mn: Vec3 = [0, 0, 0], mx: Vec3 = [0, 0, 0];
    if (blocks.length) {
      mn = [blocks[0].x, blocks[0].y, blocks[0].z];
      mx = [...mn] as Vec3;
      for (const b of blocks) {
        if (b.x < mn[0]) mn[0] = b.x; if (b.x > mx[0]) mx[0] = b.x;
        if (b.y < mn[1]) mn[1] = b.y; if (b.y > mx[1]) mx[1] = b.y;
        if (b.z < mn[2]) mn[2] = b.z; if (b.z > mx[2]) mx[2] = b.z;
      }
    }
    const out = { min: mn, size: [mx[0] - mn[0] + 1, mx[1] - mn[1] + 1, mx[2] - mn[2] + 1] as Vec3 };
    this.cellBounds.set(cell, out);
    return out;
  }

  private cellBlocks(cell: string): Block[] {
    const hit = this.cellLocal.get(cell);
    if (hit) return hit;
    const sch = this.cells.get(cell)?.schematic;
    let blocks: Block[] = [];
    if (sch) {
      this.sceneReads.cellDump++;
      try {
        blocks = JSON.parse(sch.getNonAirBlocksJson()) as Block[];
      } catch {
        blocks = [];
      }
    }
    this.cellLocal.set(cell, blocks);
    return blocks;
  }

  /** Invert `Design::transform_pos`: world -> cell-local. */
  private static toLocal(b: Block, at: Vec3, rot: number, sx: number, sz: number): Block {
    const rx = b.x - at[0], ry = b.y - at[1], rz = b.z - at[2];
    switch ((((rot % 360) + 360) % 360) / 90) {
      case 1: return { x: rz, y: ry, z: sz - 1 - rx, name: b.name };
      case 2: return { x: sx - 1 - rx, y: ry, z: sz - 1 - rz, name: b.name };
      case 3: return { x: sx - 1 - rz, y: ry, z: rx, name: b.name };
      default: return { x: rx, y: ry, z: rz, name: b.name };
    }
  }

  /** Build a variant's cell-local geometry.
   *
   *  Unpatched: read the cell's own schematic (~1 ms). Patched: read ONE
   *  representative instance's flattened region and invert its transform —
   *  the engine stays the authority on what a promotion did to the body,
   *  and it is read once per variant, not once per instance. */
  private buildVariant(key: string, rep: Placement, flat: () => any): CellVariant {
    const { min, size } = this.boundsFor(rep.cell);
    let blocks: Block[];
    if (key === rep.cell) {
      blocks = this.cellBlocks(rep.cell).map((b) => ({
        x: b.x - min[0], y: b.y - min[1], z: b.z - min[2], name: b.name,
      }));
    } else {
      this.sceneReads.instDump++;
      const world = JSON.parse(flat().getRegionNonAirBlocksJson(`inst:${rep.instance}`)) as Block[];
      blocks = world.map((b) => Studio.toLocal(b, rep.at, rep.rot, size[0], size[2]));
    }
    return { key, cell: rep.cell, blocks, size, rev: this.bumpRev(`var:${key}`, blocks) };
  }

  /** The renderable document, with only the changed parts re-read.
   *
   *  flatten() writes each layer into a NAMED SCHEMATIC REGION
   *  (`inst:{name}`, `bus:{name}`) plus the default region for the loose
   *  base; each is read back with the per-region NON-AIR dump. The dense
   *  dumps (`getAllBlocksJson`, DefinitionRegion `blocksJson`) are
   *  deliberately avoided: they materialize every in-bounds air cell, and
   *  one placed HDL cell has a multi-million-cell bounding volume — enough
   *  to exhaust wasm memory.
   *
   *  What is new here is that NONE of it happens on a transform-only edit.
   *  The old `layers()` re-read the whole document on every drag frame
   *  (a full non-air dump plus one region dump per instance, ~340 ms at 12
   *  instances); a drag now re-reads only the buses the engine says moved. */
  scene(): SceneModel {
    const placements = this.placements();
    const wanted = new Map<string, Placement>();
    for (const p of placements) if (!wanted.has(p.variant)) wanted.set(p.variant, p);
    const rebuildVariants = [...wanted.keys()]
      .filter((k) => !this.variants.has(k) || this.dirtyVariants.has(k));
    const dirtyBusNames = [...this.dirtyBuses].filter((n) => this.buses.has(n));

    // ONE flatten, and only if something actually needs it.
    let flatRaw: any = null;
    const flat = () => {
      if (!flatRaw) {
        this.sceneReads.flatten++;
        const t = performance.now();
        flatRaw = this.design.flatten().raw;
        this.sceneMs.flatten += performance.now() - t;
      }
      return flatRaw;
    };

    for (const key of rebuildVariants) {
      this.variants.set(key, this.buildVariant(key, wanted.get(key)!, flat));
    }
    // Drop variants nothing places any more (a port toggled back, a cell
    // deleted): their meshes go with them.
    for (const key of [...this.variants.keys()]) {
      if (!wanted.has(key)) this.variants.delete(key);
    }

    for (const name of dirtyBusNames) {
      try {
        const raw = flat();
        const t = performance.now();
        this.busBlocks.set(name, JSON.parse(raw.getRegionNonAirBlocksJson(`bus:${name}`)) as Block[]);
        this.sceneMs.busDump += performance.now() - t;
        this.sceneReads.busDump++;
      } catch {
        this.busBlocks.set(name, []);
      }
      this.bumpRev(`bus:${name}`, this.busBlocks.get(name) ?? []);
    }
    for (const name of [...this.busBlocks.keys()]) {
      if (!this.buses.has(name)) this.busBlocks.delete(name);
    }

    const looseWas = this.dirtyLoose;
    if (this.dirtyLoose) {
      // The loose layer is exactly the design's own region(s) — everything
      // flatten() did NOT put in an `inst:`/`bus:` layer. Verified equal to
      // the base schematic's non-air blocks, so no set subtraction over a
      // full-document dump is needed (that dump was 36 ms at 2.5k blocks and
      // grows with every placed cell).
      const names = JSON.parse(flat().regionNamesJson()) as string[];
      const out: Block[] = [];
      for (const n of names) {
        if (n.startsWith("bus:") || n.startsWith("inst:")) continue;
        try {
          out.push(...(JSON.parse(flat().getRegionNonAirBlocksJson(n)) as Block[]));
        } catch { /* a region with no blocks */ }
      }
      this.sceneReads.looseDump++;
      this.looseBlocks = out;
      this.bumpRev("loose", out);
    }

    const buses: LayerBlocks[] = [...this.buses.values()].map((b) => ({
      layer: `bus:${b.name}`,
      color: b.color,
      failed: this.busState(b.name).startsWith("failed"),
      blocks: this.busBlocks.get(b.name) ?? [],
      rev: this.revOf(`bus:${b.name}`),
    }));

    const dirty = {
      variants: rebuildVariants,
      placements: this.dirtyPlacements,
      buses: dirtyBusNames,
      loose: looseWas,
    };
    // CLEARED LAST, on purpose. Everything above can throw (a wasm read, a
    // JSON parse), and a throw after the flags were cleared is permanent
    // staleness: the work was never done and nothing remembers it was owed.
    // Clearing here means a failed scene() simply happens again.
    this.dirtyBuses.clear();
    this.dirtyVariants.clear();
    this.dirtyLoose = false;
    this.dirtyPlacements = false;
    return {
      variants: new Map(this.variants),
      placements,
      buses,
      loose: {
        layer: "loose", color: null, failed: false,
        blocks: this.looseBlocks, rev: this.revOf("loose"),
      },
      dirty,
    };
  }

  /** The engine's OWN geometry for every layer, read fresh and bypassing every
   *  cache — the ground truth a full-scene consistency check compares the
   *  rendered geometry against.
   *
   *  Deliberately NOT the incremental path: this exists to catch the case where
   *  the incremental path is wrong. `bus_blocks_json` / `instance_blocks_json`
   *  read one fragment directly when the loaded engine has them; otherwise one
   *  flatten plus the per-region non-air dumps say the same thing more slowly. */
  engineLayers(): { instances: Map<string, Block[]>; buses: Map<string, Block[]>; loose: Block[] } {
    const out = { instances: new Map<string, Block[]>(), buses: new Map<string, Block[]>(), loose: [] as Block[] };
    const d = this.design as {
      busBlocksJson?: (n: string) => unknown;
      instanceBlocksJson?: (n: string) => unknown;
    };
    /** `[[x,y,z,"block"],..]` (the compact accessors) or `[{x,y,z,name},..]`. */
    const asBlocks = (raw: unknown): Block[] => {
      const list = (typeof raw === "string" ? JSON.parse(raw) : raw) as unknown[];
      return list.map((e) => Array.isArray(e)
        ? { x: e[0] as number, y: e[1] as number, z: e[2] as number, name: e[3] as string }
        : e as Block);
    };
    let flatRaw: any = null;
    const flat = () => (flatRaw ??= this.design.flatten().raw);
    const region = (name: string): Block[] => {
      try {
        return JSON.parse(flat().getRegionNonAirBlocksJson(name)) as Block[];
      } catch {
        return [];
      }
    };
    for (const name of this.buses.keys()) {
      if (typeof d.busBlocksJson === "function") {
        try { out.buses.set(name, asBlocks(d.busBlocksJson(name))); continue; } catch { /* fall through */ }
      }
      out.buses.set(name, region(`bus:${name}`));
    }
    for (const name of this.instances.keys()) {
      if (typeof d.instanceBlocksJson === "function") {
        try { out.instances.set(name, asBlocks(d.instanceBlocksJson(name))); continue; } catch { /* fall through */ }
      }
      out.instances.set(name, region(`inst:${name}`));
    }
    const names = JSON.parse(flat().regionNamesJson()) as string[];
    for (const n of names) {
      if (n.startsWith("bus:") || n.startsWith("inst:")) continue;
      out.loose.push(...region(n));
    }
    return out;
  }
}
