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
    this.onChange?.();
  }

  // -- undo / redo ----------------------------------------------------------

  private undoStack: Op[] = [];
  private redoStack: Op[] = [];
  /** While replaying an op, mutators must not journal their own inverse. */
  private replaying = 0;

  private record(op: Op) {
    if (this.replaying) return;
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

  /** Every bus is suspect — for the rare structural edits where working out
   *  which ones moved costs more than re-reading them. */
  private dirtyAllBuses() {
    for (const n of this.buses.keys()) this.dirtyBuses.add(n);
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
    this.bump();
    return info;
  }

  // -- instances -----------------------------------------------------------

  placeInstance(cell: string, at: Vec3, rot = 0, forceName?: string): InstanceInfo {
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

  addGate(busName: string, anchor: Vec3, step: Vec3): string {
    const bus = this.buses.get(busName);
    if (!bus) throw new Error(`no bus ${busName}`);
    const gname = `g${bus.gates.length}`;
    const state = this.design.addGate(busName, gname, anchor, step);
    bus.gates.push({ name: gname, anchor: [...anchor] as Vec3, step: [...step] as Vec3 });
    this.dirtyBuses.add(busName);
    this.bump();
    return state;
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
    return { key, cell: rep.cell, blocks, size };
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
    const newVariants = [...wanted.keys()].filter((k) => !this.variants.has(k));
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

    for (const key of newVariants) {
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
    }
    for (const name of [...this.busBlocks.keys()]) {
      if (!this.buses.has(name)) this.busBlocks.delete(name);
    }
    this.dirtyBuses.clear();

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
      this.dirtyLoose = false;
    }

    const buses: LayerBlocks[] = [...this.buses.values()].map((b) => ({
      layer: `bus:${b.name}`,
      color: b.color,
      failed: this.busState(b.name).startsWith("failed"),
      blocks: this.busBlocks.get(b.name) ?? [],
    }));

    const dirty = {
      variants: newVariants,
      placements: this.dirtyPlacements,
      buses: dirtyBusNames,
      loose: looseWas,
    };
    this.dirtyPlacements = false;
    return {
      variants: new Map(this.variants),
      placements,
      buses,
      loose: { layer: "loose", color: null, failed: false, blocks: this.looseBlocks },
      dirty,
    };
  }

  /** Set alongside `dirtyLoose` so `scene()` can report it after clearing. */
  private looseDirtyLast = true;
}
