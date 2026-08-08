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

  /** The flattened document split into render layers: named regions
   *  (`bus:x`, `inst:y`) with per-bus colors, plus the loose remainder.
   *
   *  Deliberately avoids the dense dumps (`getAllBlocksJson`, region
   *  `blocksJson`): both materialize every in-bounds AIR cell, and one
   *  placed HDL cell has a multi-million-cell bounding volume — enough to
   *  exhaust wasm memory. Instead: ONE non-air dump of the flat artifact,
   *  classified in JS against each region's box list (`boxesJson`). */
  layers(): LayerBlocks[] {
    const flat = this.design.flatten();
    const s = flat.raw;
    let all: LayerBlocks["blocks"];
    try {
      all = JSON.parse(s.getNonAirBlocksJson());
    } catch {
      // engine predates the non-air dump: dense fallback, filtered here.
      // Safe only while the design is small (the dense dump can OOM).
      all = (JSON.parse(s.getAllBlocksJson()) as LayerBlocks["blocks"])
        .filter((b) => b.name !== "minecraft:air");
    }
    const names = JSON.parse(this.core.SchematicRegions.namesJson(s)) as string[];
    // bus layers claim blocks first (an instance bbox may overlap a bus run)
    const rank = (n: string) => (n.startsWith("bus:") ? 0 : n.startsWith("inst:") ? 1 : 2);
    const defs: { name: string; boxes: number[]; layer: LayerBlocks }[] = [];
    for (const name of names) {
      let boxes: number[];
      try {
        boxes = JSON.parse(this.core.SchematicRegions.get(s, name).boxesJson());
      } catch {
        continue;
      }
      let color: number | null = null;
      let failed = false;
      if (name.startsWith("bus:")) {
        const busName = name.slice(4);
        color = this.buses.get(busName)?.color ?? null;
        failed = this.busState(busName).startsWith("failed");
      }
      defs.push({ name, boxes, layer: { layer: name, color, failed, blocks: [] } });
    }
    defs.sort((a, b) => rank(a.name) - rank(b.name));
    const loose: LayerBlocks = { layer: "loose", color: null, failed: false, blocks: [] };
    const inBoxes = (boxes: number[], x: number, y: number, z: number) => {
      for (let i = 0; i + 5 < boxes.length; i += 6) {
        if (x >= boxes[i] && y >= boxes[i + 1] && z >= boxes[i + 2] &&
            x <= boxes[i + 3] && y <= boxes[i + 4] && z <= boxes[i + 5]) return true;
      }
      return false;
    };
    for (const b of all) {
      const def = defs.find((d) => inBoxes(d.boxes, b.x, b.y, b.z));
      (def ? def.layer : loose).blocks.push(b);
    }
    return [...defs.map((d) => d.layer), loose];
  }
}
