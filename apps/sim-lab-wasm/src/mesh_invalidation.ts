/** Rendering identities, never simulation identities. Resource packs remain
 * authoritative: even a normally invisible property can select a custom model.
 * The pack's blockstate definitions stay fixed for the lifetime of a World. */
export class VisualStates {
  private properties = new Map<string, Set<string> | null>();
  private states = new Map<string, string>();

  private definition: (name: string) => string;
  constructor(definition: (name: string) => string) { this.definition = definition; }

  key(state: string): string {
    if (state === "minecraft:moving_piston" || state.startsWith("minecraft:moving_piston[")) return "minecraft:air";
    const cached = this.states.get(state);
    if (cached !== undefined) return cached;
    const bracket = state.indexOf("[");
    if (bracket < 0 || !state.endsWith("]")) return state;
    const name = state.slice(0, bracket);
    // These fields affect simulation only, unless the pack explicitly uses
    // them. Do not generalise to dust power (tint), water, or block entities.
    const candidates = name === "minecraft:scaffolding" ? ["distance"]
      : name === "minecraft:note_block" ? ["note", "powered", "instrument"]
      : name.startsWith("minecraft:") && name.endsWith("_leaves") ? ["distance", "persistent"] : [];
    if (!candidates.length) return state;
    if (!this.properties.has(name)) {
      try {
        const json = JSON.parse(this.definition(name));
        const used = new Set<string>();
        const condition = (value: unknown): void => {
          if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("unknown condition");
          for (const [prop, v] of Object.entries(value)) {
            if (prop === "OR" || prop === "AND") {
              if (!Array.isArray(v)) throw new Error("unknown condition group");
              v.forEach(condition);
            } else used.add(prop);
          }
        };
        if (json.variants && typeof json.variants === "object" && !Array.isArray(json.variants)) {
          for (const variant of Object.keys(json.variants)) {
            if (variant) for (const part of variant.split(",")) {
              if (!part.includes("=")) throw new Error("unknown variant");
              used.add(part.split("=")[0]);
            }
          }
        } else if (Array.isArray(json.multipart)) {
          for (const part of json.multipart) if (part.when !== undefined) condition(part.when);
        } else throw new Error("unknown blockstate definition");
        this.properties.set(name, used);
      } catch {
        this.properties.set(name, null); // conservative fallback
      }
    }
    const used = this.properties.get(name);
    if (!used) return state;
    const props = state.slice(bracket + 1, -1).split(",")
      .filter(p => !candidates.includes(p.split("=")[0]) || used.has(p.split("=")[0]));
    const result = props.length ? `${name}[${props.join(",")}]` : name;
    if (this.states.size >= 4096) this.states.clear();
    this.states.set(state, result);
    return result;
  }
}

/** Retain only the last write to each cell within a presentation batch. */
export function finalChanges<T extends { pos: [number, number, number] }>(changes: T[]): T[] {
  const cells = new Map<string, T>();
  for (const change of changes) cells.set(change.pos.join(","), change);
  return [...cells.values()];
}

/** Include the one-cell halo for faces/AO, including diagonal chunk seams and
 * negative coordinates. Each affected chunk appears once, not 27 times. */
export function affectedChunks(pos: [number, number, number], size: number): string[] {
  const min = pos.map(v => Math.floor((v - 1) / size));
  const max = pos.map(v => Math.floor((v + 1) / size));
  const result: string[] = [];
  for (let x = min[0]; x <= max[0]; x++)
    for (let y = min[1]; y <= max[1]; y++)
      for (let z = min[2]; z <= max[2]; z++) result.push(`${x},${y},${z}`);
  return result;
}

/** Pending cell differences relative to each chunk's last meshing snapshot.
 * A→B→A before that snapshot needs no rebuild. Once meshing starts, B is the
 * new baseline: changes during async GLB parsing must survive for another pass. */
export class MeshInvalidation {
  readonly dirty = new Set<string>();
  private baseline = new Map<string, Map<string, string>>();

  change(chunk: string, cell: string, before: string, after: string): void {
    let cells = this.baseline.get(chunk);
    if (!cells) { cells = new Map(); this.baseline.set(chunk, cells); }
    const original = cells.get(cell) ?? before;
    if (after === original) cells.delete(cell);
    else cells.set(cell, original);
    if (cells.size) this.dirty.add(chunk);
    else { this.baseline.delete(chunk); this.dirty.delete(chunk); }
  }

  take(chunk: string): boolean {
    this.baseline.delete(chunk);
    return this.dirty.delete(chunk);
  }
}
