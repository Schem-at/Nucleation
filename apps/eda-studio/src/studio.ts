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
}

export interface LayerBlocks {
  layer: string; // "loose" | "bus:<name>" | "inst:<name>" | region name
  color: number | null; // bus color override, else null (block colors)
  failed: boolean;
  blocks: { x: number; y: number; z: number; name: string }[];
}

const BUS_PALETTE = [0x76d275, 0x4fc3f7, 0xffb74d, 0xba68c8, 0x4db6ac, 0xf06292, 0xa2cf6e, 0x7986cb];

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

  placeInstance(cell: string, at: Vec3, rot = 0): InstanceInfo {
    const name = `u${this.nextInst++}`;
    this.design.place(name, cell, at, rot);
    const info: InstanceInfo = { name, cell, at: [...at] as Vec3, rot };
    this.instances.set(name, info);
    this.bump();
    return info;
  }

  /** Drag an instance: the move always lands (the document's truth); the
   *  affected buses reroute, failures stay visibly failed. */
  moveInstance(name: string, at: Vec3, rot?: number): { rerouted: string[]; failed: Record<string, string> } {
    const inst = this.instances.get(name);
    if (!inst) throw new Error(`no instance ${name}`);
    const report = this.design.moveInstance(name, at, rot ?? inst.rot);
    inst.at = [...at] as Vec3;
    if (rot != null) inst.rot = rot;
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
    if (!this.instances.has(name)) throw new Error(`no instance ${name}`);
    const report = this.design.removeInstance(name);
    this.instances.delete(name);
    for (const b of report.removed_buses ?? []) this.buses.delete(b);
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
    const key = `${this.version}:${instance}.${port}`;
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
    const report = this.design.setPortMode(instance, port, mode);
    for (const name of report.removed_buses ?? []) this.buses.delete(name);
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

  routeBus(driver: string, sinks: string[], gates: GateInfo[] = []): BusInfo {
    const name = `bus${this.nextBus++}`;
    const color = BUS_PALETTE[this.buses.size % BUS_PALETTE.length];
    const bus = this.design.routeBus(name, { driver, sinks, gates });
    const info: BusInfo = {
      name, driver, sinks: [...sinks], color,
      gates: gates.map((g, i) => ({ ...g, name: g.name || `g${i}` })),
    };
    this.buses.set(name, info);
    this.bump();
    return info;
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
    this.bump();
    return state;
  }

  /** Drag a gate: exactly its two adjacent segments rip and reroute. */
  moveGate(busName: string, gateName: string, anchor: Vec3): { state: string; rerouted_segments: number } {
    const bus = this.buses.get(busName);
    const gate = bus?.gates.find((g) => g.name === gateName);
    if (!bus || !gate) throw new Error(`no gate ${busName}/${gateName}`);
    const report = this.design.moveGate(busName, gateName, anchor);
    gate.anchor = [...anchor] as Vec3;
    this.bump();
    return report;
  }

  ripBus(name: string): void {
    this.design.rip(name);
    this.bump();
  }

  /** Try the bus again from its stored declaration (heal after the blocker
   *  moved). Returns the resulting state, `failed: reason` included. */
  rerouteBus(name: string): string {
    const state = this.design.reroute(name);
    this.bump();
    return state;
  }

  /** Delete a bus outright, freeing its name. */
  removeBus(name: string): void {
    this.design.removeBus(name);
    this.buses.delete(name);
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

  /** The flattened document split into render layers.
   *
   *  flatten() writes each layer into a NAMED SCHEMATIC REGION
   *  (`inst:{name}`, `bus:{name}`) plus the default region for the loose
   *  base; each is read back with the per-region NON-AIR dump. The dense
   *  dumps (`getAllBlocksJson`, DefinitionRegion `blocksJson`) are
   *  deliberately avoided: they materialize every in-bounds air cell, and
   *  one placed HDL cell has a multi-million-cell bounding volume —
   *  enough to exhaust wasm memory. */
  layers(): LayerBlocks[] {
    const flat = this.design.flatten();
    const s = flat.raw;
    const names = JSON.parse(s.regionNamesJson()) as string[];
    const layers: LayerBlocks[] = [];
    const claimed = new Set<string>();
    for (const name of names) {
      if (!name.startsWith("bus:") && !name.startsWith("inst:")) continue;
      let blocks: LayerBlocks["blocks"];
      try {
        blocks = JSON.parse(s.getRegionNonAirBlocksJson(name));
      } catch {
        continue;
      }
      for (const b of blocks) claimed.add(`${b.x},${b.y},${b.z}`);
      let color: number | null = null;
      let failed = false;
      if (name.startsWith("bus:")) {
        const busName = name.slice(4);
        color = this.buses.get(busName)?.color ?? null;
        failed = this.busState(busName).startsWith("failed");
      }
      layers.push({ layer: name, color, failed, blocks });
    }
    const all = JSON.parse(s.getNonAirBlocksJson()) as LayerBlocks["blocks"];
    const loose = all.filter((b) => !claimed.has(`${b.x},${b.y},${b.z}`));
    layers.push({ layer: "loose", color: null, failed: false, blocks: loose });
    return layers;
  }
}
