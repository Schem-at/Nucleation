/** The world: a schematic, a live TickSimulation over it, and a chunked
 * three.js scene that re-meshes only what the simulation changed.
 *
 * Whole-build meshing is what makes big machines viewable at all — a
 * 54k-block ship is one `ChunkMeshResult` rather than 54k models. Keeping
 * the chunks addressable is what makes it *live*: a tick that flips six
 * blocks dirties one or two 16³ chunks, and only those are re-meshed, so a
 * running machine costs a few milliseconds a tick instead of a rebuild.
 */

import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { loadMeshEnv } from "./mesher";

/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any;

export const CHUNK = 16;
const loader = new GLTFLoader();

function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

const key = (x: number, y: number, z: number) => `${x},${y},${z}`;

export type Change = { pos: [number, number, number]; to: string };

/** How a build is settled before tick 0.
 *
 * `placement` runs vanilla's full paste — observers pulse, machines twitch,
 * exactly as they would when a player pastes a schematic. `quiet` runs
 * `onPlace` only (the gametest framework's knownShape placement). `in-world`
 * does neither: the build is taken as already standing there, at rest, which
 * is what you want when the thing you came to watch is the machine and not
 * its arrival.
 */
export type SettleName = "placement" | "quiet" | "in-world";

const SETTLE_MODE = (eng: Any): Record<SettleName, Any> => ({
  placement: eng.TickSettleMode.Placement,
  quiet: eng.TickSettleMode.Quiet,
  "in-world": eng.TickSettleMode.InWorld,
});

export class World {
  eng: Any;
  pack: Any;
  cfg: Any;
  /** The schematic the sim was built from, kept in step with every change
   * so a dirty chunk can be re-meshed from current truth. */
  schem: Any;
  sim: Any;
  dims: [number, number, number] = [0, 0, 0];
  group = new THREE.Group();
  /** One three.js object per chunk, by chunk coordinate. */
  private chunks = new Map<string, THREE.Object3D>();
  private dirty = new Set<string>();
  /** Solid cells, for the player's raycast and collision. */
  private solid = new Set<string>();
  /** How the build is settled before tick 0; see [`SettleName`]. */
  settle: SettleName = "in-world";

  private constructor(eng: Any, pack: Any, cfg: Any) {
    this.eng = eng;
    this.pack = pack;
    this.cfg = cfg;
  }

  static async load(bytes: Uint8Array, settle: SettleName = "in-world"): Promise<World> {
    const { eng, pack, cfg } = await loadMeshEnv();
    const w = new World(eng, pack, cfg);
    w.settle = settle;
    w.schem = eng.Schematic.fromData(bytes);
    const d = w.schem.tightDimensions();
    w.dims = [Number(d.x), Number(d.y), Number(d.z)];
    w.startSim();
    await w.meshAll();
    return w;
  }

  /** (Re)build the simulation from the current schematic. `extraStates`
   * carries the blocks an interaction may introduce — a redstone block for
   * manual poking is the one the engine always allows. */
  startSim(): string | null {
    try {
      this.sim = this.eng.TickSimulation.fromSchematic(
        this.schem,
        SETTLE_MODE(this.eng)[this.settle],
        0,
        0,
        0,
        "",
      );
      return null;
    } catch (e) {
      const detail = this.eng.TickSimulation.lastErrorDetail?.() ?? "";
      return `${e}${detail ? ` — ${detail}` : ""}`;
    }
  }

  /** Mesh the whole build once, one three.js child per populated chunk. */
  private async meshAll(): Promise<void> {
    const result = this.eng.ChunkMeshResult.createWithSize(
      this.schem,
      this.pack,
      this.cfg,
      CHUNK,
    );
    const count = result.chunkCount();
    const jobs: Promise<void>[] = [];
    for (let i = 0; i < count; i++) {
      // `BlockPos` is an opaque handle with getters, not a plain record:
      // reading it as `{x,y,z}` fields yields undefined and every mesh
      // lands on chunk zero.
      const c = result.chunkCoordinateAt(i);
      const [cx, cy, cz] = [Number(c.x), Number(c.y), Number(c.z)];
      let mesh: Any = null;
      try {
        mesh = result.getMesh(cx, cy, cz);
      } catch {
        mesh = null;
      }
      jobs.push(this.installChunk(cx, cy, cz, mesh));
    }
    await Promise.all(jobs);
    this.rebuildSolidSet();
  }

  private async installChunk(cx: number, cy: number, cz: number, mesh: Any): Promise<void> {
    const k = key(cx, cy, cz);
    let object: THREE.Object3D | null = null;
    try {
      const glb = mesh ? mesh.glbDataB64() : "";
      if (glb && glb.length > 0) {
        const bytes = b64ToBytes(glb);
        // `.buffer` is typed `ArrayBufferLike`, which admits SharedArrayBuffer;
        // the loader takes a plain ArrayBuffer, and a slice of a Uint8Array
        // backed by wasm memory is always one.
        const gltf = await loader.parseAsync(
          bytes.buffer.slice(
            bytes.byteOffset,
            bytes.byteOffset + bytes.byteLength,
          ) as ArrayBuffer,
          "",
        );
        // The mesher emits **world** coordinates, not chunk-local ones —
        // measured, after translating each chunk by its origin doubled the
        // build's span (175 wide became 334, one empty chunk between each).
        // Both paths here keep world space, so nothing needs moving.
        object = gltf.scene;
        // A GLB can parse to a scene with no drawable mesh at all; adding
        // that is an invisible child that still counts and still costs.
        let drawable = false;
        object.traverse((o: Any) => {
          if (o.isMesh) drawable = true;
        });
        if (!drawable) object = null;
      }
    } catch {
      object = null; // an unmeshable chunk shows as empty rather than killing the frame
    }
    const previous = this.chunks.get(k);
    if (previous) {
      this.group.remove(previous);
      previous.traverse((o: Any) => o.geometry?.dispose?.());
    }
    if (object) {
      this.chunks.set(k, object);
      this.group.add(object);
    } else {
      this.chunks.delete(k);
    }
  }

  /** Re-mesh one chunk from the current schematic, by copying that box into
   * a scratch schematic the mesher can chew on its own. */
  private async remeshChunk(cx: number, cy: number, cz: number): Promise<void> {
    let mesh: Any = null;
    try {
      const scratch = this.eng.Schematic.create("chunk");
      scratch.copyRegion(
        this.schem,
        cx * CHUNK,
        cy * CHUNK,
        cz * CHUNK,
        cx * CHUNK + CHUNK - 1,
        cy * CHUNK + CHUNK - 1,
        cz * CHUNK + CHUNK - 1,
        // Copy the box to where it actually lives, so the re-meshed chunk
        // comes back in the same world space as the initial pass.
        cx * CHUNK,
        cy * CHUNK,
        cz * CHUNK,
        "[]",
      );
      // An empty box has nothing to mesh and the mesher says so by
      // throwing. A dirty chunk legitimately empties out — a piston pulls
      // the last block out of it — and that must clear the chunk, not
      // raise.
      if ((scratch.blockCount?.() ?? 0) > 0) {
        mesh = this.eng.MeshResult.create(scratch, this.pack, this.cfg);
      }
    } catch {
      mesh = null;
    }
    await this.installChunk(cx, cy, cz, mesh);
  }

  /** Apply what the last tick changed: patch the schematic, mark chunks. */
  applyChanges(changes: Change[]): void {
    for (const c of changes) {
      const [x, y, z] = c.pos;
      try {
        this.schem.setBlockFromString(x, y, z, c.to);
      } catch {
        /* a state the schematic cannot express is still simulated */
      }
      if (c.to === "minecraft:air") this.solid.delete(key(x, y, z));
      else this.solid.add(key(x, y, z));
      // The block's own chunk, plus any neighbour whose face meshing it
      // changes — a block on a seam alters the chunk next door.
      //
      // Deliberately *not* clamped to the build's own chunks. The engine's
      // region grows to follow a machine that travels, so the loaded
      // dimensions stop describing where blocks are the moment a flying
      // machine sets off; clamping to them left it flying somewhere the
      // screen could not show. This set is self-limiting anyway — a chunk
      // only enters it because a block in it actually changed — and
      // `remeshChunk` skips any box that turns out empty, which is what
      // stops the stray geometry a clamp was once used for.
      for (let dx = -1; dx <= 1; dx++)
        for (let dy = -1; dy <= 1; dy++)
          for (let dz = -1; dz <= 1; dz++) {
            this.dirty.add(
              key(
                Math.floor((x + dx) / CHUNK),
                Math.floor((y + dy) / CHUNK),
                Math.floor((z + dz) / CHUNK),
              ),
            );
          }
    }
  }

  /** Re-mesh everything marked dirty. Called once per animation frame, not
   * per tick, so a fast-forward costs one re-mesh rather than dozens. */
  async flush(): Promise<number> {
    if (this.dirty.size === 0) return 0;
    const todo = [...this.dirty];
    this.dirty.clear();
    for (const k of todo) {
      const [cx, cy, cz] = k.split(",").map(Number);
      await this.remeshChunk(cx, cy, cz);
    }
    return todo.length;
  }

  private rebuildSolidSet(): void {
    this.solid.clear();
    const [dx, dy, dz] = this.dims;
    for (let x = 0; x < dx; x++)
      for (let y = 0; y < dy; y++)
        for (let z = 0; z < dz; z++) {
          const s = this.blockAt(x, y, z);
          if (s && s !== "minecraft:air") this.solid.add(key(x, y, z));
        }
  }

  blockAt(x: number, y: number, z: number): string {
    try {
      return this.sim ? this.sim.getBlock(x, y, z) : this.schem.getBlockString(x, y, z);
    } catch {
      return "minecraft:air";
    }
  }

  isSolid(x: number, y: number, z: number): boolean {
    return this.solid.has(key(x, y, z));
  }

  /** How much of the change log has already been applied. */
  private seen = 0;

  /** Everything the simulation has changed since the last drain.
   *
   * The engine's log is **cumulative** and has no reset, so this keeps a
   * cursor rather than re-applying history: draining per step without one
   * re-read the whole log every step and reported 44k changes for 50 ticks
   * of a 1200-block door. Call it once per batch, not once per step.
   */
  drainChanges(): Change[] {
    if (!this.sim) return [];
    // Cheap gate: no new entries, no parse.
    try {
      if (Number(this.sim.changesCount?.() ?? -1) === this.seen) return [];
    } catch {
      /* fall through to the parse */
    }
    let raw = "";
    try {
      raw = this.sim.changesJson();
    } catch {
      return [];
    }
    if (!raw) return [];
    let parsed: Any;
    try {
      parsed = JSON.parse(raw);
    } catch {
      return [];
    }
    const all: Any[] = Array.isArray(parsed) ? parsed : (parsed.changes ?? []);
    const list = all.slice(this.seen);
    this.seen = all.length;
    return list
      .map((c: Any) => ({
        pos: [c.x ?? c.pos?.[0], c.y ?? c.pos?.[1], c.z ?? c.pos?.[2]] as [number, number, number],
        to: c.to ?? c.state ?? "minecraft:air",
      }))
      .filter((c: Change) => Number.isFinite(c.pos[0]));
  }
}
