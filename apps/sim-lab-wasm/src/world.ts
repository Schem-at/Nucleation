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

/** Unit step per `facing` value, for reading a `moving_piston`'s travel. */
const FACING: Record<string, [number, number, number]> = {
  east: [1, 0, 0],
  west: [-1, 0, 0],
  up: [0, 1, 0],
  down: [0, -1, 0],
  south: [0, 0, 1],
  north: [0, 0, -1],
};

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

/** A dropped item's box. Vanilla's is 0.25 cubed; drawn a little larger
 * because a quarter-block at any sane camera distance is a pixel. */
const ITEM_SIZE: [number, number, number] = [0.35, 0.35, 0.35];
/** `minecart::cart_aabb` — the engine reports item and cart positions but not
 * their boxes, and these two are fixed. */
const CART_SIZE: [number, number, number] = [0.98, 0.7, 0.98];

/** What colour an entity is drawn, by what it is. Broad classes only — the
 * point is to tell a boat from a dropped item at a glance, not to be a legend. */
function entityColour(kind: string): number {
  if (kind === "item") return 0xffd166; // dropped stack
  if (kind.includes("minecart")) return 0xc0c6cc; // rolling stock
  if (kind.includes("boat")) return 0xb07b4f; // wood
  return 0xe0e0e0; // any other frozen body: mobs, armour stands, riders
}

/** A translucent box standing in for an entity.
 *
 * Deliberately not a model: the engine knows an entity's *hitbox* and its
 * kind, and nothing else about it. Drawing a real boat here would be drawing
 * something the simulation does not have — and a box that is exactly the
 * measured collision volume is the more useful picture anyway, because that
 * volume is the thing that decides whether a piston can shove it.
 *
 * A leashed entity gets a second, brighter wireframe. There is no rope: the
 * leash *target* is discarded at parse time, on purpose, because a litematic
 * keeps a fence knot's source-world coordinates while storing the entity
 * relative to its region — so the anchor cannot be trusted to be anywhere
 * near the build. A tether drawn to the wrong place is worse than none.
 */
function entityBox(
  kind: string,
  size: [number, number, number],
  leashed: boolean,
): THREE.Object3D {
  const group = new THREE.Group();
  const geometry = new THREE.BoxGeometry(size[0], size[1], size[2]);
  group.add(
    new THREE.Mesh(
      geometry,
      new THREE.MeshBasicMaterial({
        color: entityColour(kind),
        transparent: true,
        opacity: 0.35,
        depthWrite: false,
      }),
    ),
  );
  group.add(
    new THREE.LineSegments(
      new THREE.EdgesGeometry(geometry),
      new THREE.LineBasicMaterial({ color: leashed ? 0xffd166 : entityColour(kind) }),
    ),
  );
  return group;
}

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
  /** One object per live entity, by engine id. Kept apart from the chunk
   * meshes because entities move without dirtying a chunk. */
  private entityMeshes = new Map<number, THREE.Object3D>();
  private entityGroup = new THREE.Group();
  /** How the build is settled before tick 0; see [`SettleName`]. */
  settle: SettleName = "in-world";

  private constructor(eng: Any, pack: Any, cfg: Any) {
    this.eng = eng;
    this.pack = pack;
    this.cfg = cfg;
    this.group.add(this.entityGroup);
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
    // Whatever was mid-flight or alive belongs to the run being replaced.
    this.clearFlights();
    this.clearEntities();
    try {
      this.sim = this.eng.TickSimulation.fromSchematic(
        this.schem,
        SETTLE_MODE(this.eng)[this.settle],
        0,
        0,
        0,
        "",
      );
      // Draw the entities the build was authored with, before any tick runs.
      // A boat that only appears once you press play looks like the
      // simulation spawned it.
      this.syncEntities();
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

  /** A block mid-flight between two cells, drawn by [`World.animate`]. */
  private flights: {
    state: string;
    /** Null until the block's mesh finishes parsing; see [`World.animate`]. */
    object: THREE.Object3D | null;
    from: [number, number, number];
    to: [number, number, number];
    start: number;
    dur: number;
  }[] = [];
  /** One parsed mesh per block state, so a hundred sliding slime blocks cost
   * one mesh and a hundred cheap clones. */
  private blockMeshes = new Map<string, THREE.Object3D | null>();
  private pendingMeshes = new Set<string>();

  /** A 1x1x1 mesh of `state`, built once and cloned thereafter.
   *
   * Returns null until the parse finishes; a flight with no mesh yet simply
   * is not drawn for a frame or two, which is invisible at these speeds.
   */
  private blockMesh(state: string): THREE.Object3D | null {
    const have = this.blockMeshes.get(state);
    if (have !== undefined) return have;
    if (this.pendingMeshes.has(state)) return null;
    this.pendingMeshes.add(state);
    void (async () => {
      let object: THREE.Object3D | null = null;
      try {
        const scratch = this.eng.Schematic.create("one");
        scratch.setBlockFromString(0, 0, 0, state);
        const mesh = this.eng.MeshResult.create(scratch, this.pack, this.cfg);
        const glb = mesh?.glbDataB64?.() ?? "";
        if (glb) {
          const bytes = b64ToBytes(glb);
          const gltf = await loader.parseAsync(
            bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer,
            "",
          );
          let drawable = false;
          gltf.scene.traverse((o: Any) => {
            if (o.isMesh) drawable = true;
          });
          object = drawable ? gltf.scene : null;
        }
      } catch {
        object = null;
      }
      this.blockMeshes.set(state, object);
      this.pendingMeshes.delete(state);
    })();
    return null;
  }

  /** Start drawing `state` sliding from `from` to `to` over `dur` seconds.
   *
   * The flight is recorded even though its mesh is still parsing — the first
   * sighting of any block state is always a cache miss, and dropping those
   * meant the first stroke of every kind never animated at all.
   */
  private launch(state: string, from: [number, number, number], to: [number, number, number], dur: number) {
    this.blockMesh(state); // kick off the parse; picked up in `animate`
    // Milliseconds, to match `performance.now()`. Holding the caller's
    // seconds here made every flight expire on its first frame.
    this.flights.push({ state, object: null, from, to, start: performance.now(), dur: dur * 1000 });
  }

  /** Advance every in-flight block; drop the ones that have arrived.
   *
   * Called once a frame. A move takes two game ticks, so at the rates worth
   * watching it spans many frames — which is the whole point: without this a
   * piston stroke is two instantaneous jumps.
   */
  animate(now: number): void {
    if (this.flights.length === 0) return;
    this.flights = this.flights.filter((f) => {
      const t = f.dur > 0 ? (now - f.start) / f.dur : 1;
      if (t >= 1) {
        if (f.object) {
          this.group.remove(f.object);
          f.object.traverse((o: Any) => o.geometry?.dispose?.());
        }
        return false;
      }
      // Late arrival: the mesh finished parsing after the flight began.
      if (!f.object) {
        const proto = this.blockMesh(f.state);
        if (!proto) return true; // still parsing; try again next frame
        f.object = proto.clone(true);
        this.group.add(f.object);
      }
      // Linear, like the game's own `getExtendedProgress` — a piston does not
      // ease in or out, and faking it reads as lag.
      f.object.position.set(
        f.from[0] + (f.to[0] - f.from[0]) * t,
        f.from[1] + (f.to[1] - f.from[1]) * t,
        f.from[2] + (f.to[2] - f.from[2]) * t,
      );
      return true;
    });
  }

  /** Redraw every live entity from the engine's own view.
   *
   * Entities cannot ride the chunk mesher. A block is a cell that is either
   * this state or that one; an entity sits at a continuous position, is not
   * aligned to anything, and moves on ticks that dirty no chunk at all — a
   * boat shoved by a piston travels without a single block changing. So they
   * get their own group, rebuilt from `itemEntitiesJson` after every tick.
   *
   * Rebuilding rather than diffing is deliberate: the list is short (a build
   * with a hundred entities is a large one), and an entity that is removed and
   * an entity that moved look the same to a diff keyed on anything but id.
   */
  syncEntities(): void {
    const raw = this.sim?.itemEntitiesJson?.();
    if (!raw) return;
    let view: Any;
    try {
      view = JSON.parse(raw);
    } catch {
      return;
    }

    const seen = new Set<number>();
    const place = (id: number, kind: string, pos: number[], size: [number, number, number], leashed: boolean) => {
      seen.add(id);
      let mesh = this.entityMeshes.get(id);
      if (!mesh) {
        mesh = entityBox(kind, size, leashed);
        this.entityMeshes.set(id, mesh);
        this.entityGroup.add(mesh);
      }
      // `pos` is feet-centre — centred on x and z, bottom on y — which is
      // where vanilla puts an entity's position and not where three.js wants
      // a box's origin.
      mesh.position.set(pos[0], pos[1] + size[1] / 2, pos[2]);
    };

    for (const item of view.items ?? []) {
      place(item.id, "item", item.pos, ITEM_SIZE, false);
    }
    for (const cart of view.minecarts ?? []) {
      place(cart.id, cart.kind, cart.pos, CART_SIZE, false);
    }
    for (const body of view.frozen ?? []) {
      // The engine reports the measured hitbox; guessing one from the kind
      // name would draw a boat the size of a villager.
      place(body.id, body.kind, body.pos, body.size ?? [1, 1, 1], !!body.leashed);
    }

    for (const [id, mesh] of this.entityMeshes) {
      if (seen.has(id)) continue;
      this.entityGroup.remove(mesh);
      mesh.traverse((o: Any) => o.geometry?.dispose?.());
      this.entityMeshes.delete(id);
    }
  }

  /** How many entities are on screen, for the HUD. */
  entityCount(): number {
    return this.entityMeshes.size;
  }

  /** Drop every entity — on reload or when the simulation is rebuilt. */
  clearEntities(): void {
    for (const mesh of this.entityMeshes.values()) {
      this.entityGroup.remove(mesh);
      mesh.traverse((o: Any) => o.geometry?.dispose?.());
    }
    this.entityMeshes.clear();
  }

  /** Drop every in-flight block — on reload, reset, or a jump in time. */
  clearFlights(): void {
    for (const f of this.flights) {
      if (!f.object) continue;
      this.group.remove(f.object);
      f.object.traverse((o: Any) => o.geometry?.dispose?.());
    }
    this.flights = [];
  }

  /** Apply what the last tick changed: patch the schematic, mark chunks.
   *
   * `moveSeconds` is how long a piston stroke should take on screen — two
   * game ticks at the current rate. Zero disables the animation, which is
   * what a fast-forward wants: interpolating a stroke that is already over
   * by the next frame just smears it.
   */
  applyChanges(changes: Change[], moveSeconds = 0): void {
    // Entities first, and unconditionally: an entity can move on a tick that
    // changes no block at all, so gating this on `changes.length` would freeze
    // a boat drifting through still air.
    this.syncEntities();
    // Before anything is written: a cell turning into `moving_piston` is a
    // block arriving from the far side, and the only place its identity still
    // exists is the schematic we are about to overwrite. Read the sources
    // first, animate second, write third.
    if (moveSeconds > 0) {
      for (const c of changes) {
        const dir = /moving_piston\[facing=(\w+)/.exec(c.to);
        if (!dir) continue;
        const [dx, dy, dz] = FACING[dir[1]] ?? [0, 0, 0];
        const [x, y, z] = c.pos;
        const src: [number, number, number] = [x - dx, y - dy, z - dz];
        // The *schematic*, not `blockAt` — the simulation has already stepped
        // and cleared the source, so it is the mirror lagging one batch behind
        // that still knows what set off.
        let carried = "";
        try {
          carried = this.schem.getBlockString(src[0], src[1], src[2]) ?? "";
        } catch {
          carried = "";
        }
        if (!carried || carried === "minecraft:air") continue;
        if (carried.startsWith("minecraft:moving_piston")) continue; // already in flight
        // A piston's own head slot: the base stays put and the head slides
        // out of it, so draw the head rather than a second copy of the base.
        const head = new RegExp(`^minecraft:(sticky_)?piston\\[.*facing=${dir[1]}`).exec(carried);
        const state = head
          ? `minecraft:piston_head[facing=${dir[1]},short=false,type=${head[1] ? "sticky" : "normal"}]`
          : carried;
        this.launch(state, src, [x, y, z], moveSeconds);
      }
    }
    for (const c of changes) {
      const [x, y, z] = c.pos;
      // A placeholder is not a block anyone should see. The chunk shows a
      // hole for the two ticks the stroke lasts and the sliding mesh fills
      // it — meshing `moving_piston` instead drew a stand-in that popped.
      const solidified = c.to.startsWith("minecraft:moving_piston") ? "minecraft:air" : c.to;
      try {
        this.schem.setBlockFromString(x, y, z, solidified);
      } catch {
        /* a state the schematic cannot express is still simulated */
      }
      if (solidified === "minecraft:air") this.solid.delete(key(x, y, z));
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
