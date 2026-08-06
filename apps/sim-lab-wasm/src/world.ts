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

/** Default mesh-chunk edge, in blocks.
 *
 * Not a fixed property of the format — it is a tuning knob, and the right
 * value depends on the build. Small chunks re-mesh less geometry when one
 * block changes but split the build across many texture atlases; one chunk
 * spanning the whole build means every re-mesh redoes all its geometry, but
 * there is then exactly one atlas and it is always a cache hit. A flying
 * machine is small enough that the second wins outright; a cathedral is not.
 */
export const CHUNK = 16;
/** `chunkSize` values the UI offers. `0` means "one chunk for the whole
 * build", resolved against its dimensions at load. */
export const CHUNK_CHOICES = [16, 32, 64, 0] as const;
export const loader = new GLTFLoader();

function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

const key = (x: number, y: number, z: number) => `${x},${y},${z}`;

export type Change = { pos: [number, number, number]; to: string };

/** One block the engine currently has in flight — `movingBlocksJson`.
 *
 * Read, not derived. The app used to reconstruct this from the block-change
 * stream: regex `moving_piston[facing=…]` out of it, guess the source as
 * `pos - facing`, and look the carried block up in a schematic mirror that
 * lagged a batch behind. That is piston mechanics reimplemented in TypeScript
 * downstream of an engine that models them exactly, and it animated on a
 * wall-clock timer the simulation did not share — which is what drew a block
 * twice, left a gap where one should be, and sheared a piston head off its
 * load. See `Simulation::moving_blocks` in `crates/mc-tick/src/sim.rs`.
 */
type MovingBlock = {
  to: [number, number, number];
  from: [number, number, number];
  /** What lands at `to`. Not always what is drawn — see `carried`. */
  state: string;
  /** What to draw travelling. Differs from `state` for a retracting piston:
   * its body stays put and a `piston_head` comes home. */
  carried: string;
  /** `carried` with a piston arm's `short=true`, or null if it is not an arm.
   * Drawn while the head is within half a block of its body. */
  carried_short: string | null;
  /** What to draw parked at `to` for the whole move — the retracting piston's
   * body — or null when the cell is genuinely empty until the block lands. */
  remains: string | null;
  dir: string;
  extending: boolean;
  started: number;
  lands: number;
  source_piston: boolean;
};

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

/** Decoded materials, keyed by the bytes of the atlas they were decoded from.
 *
 * Each chunk's GLB embeds a texture atlas, and decoding that PNG is **366 ms
 * of a 367 ms chunk parse** — the geometry itself takes half a millisecond.
 * Re-meshing one chunk because one block moved paid that every time.
 *
 * The atlas is built per chunk from the blocks that chunk actually contains,
 * so it is *not* shared between chunks and materials cannot simply be reused
 * globally — checked, and the six chunks of one flying machine embed six
 * different atlases. What is true is that a chunk's atlas depends only on
 * which block *types* are in it, and a piston shoving a slime block around
 * does not change that set. So the key is the atlas content itself: identical
 * bytes cannot decode to a different texture, which makes reuse correct by
 * construction rather than by assumption, and still hits nearly every time.
 */
const atlasCache = new Map<string, Map<string, THREE.Material>>();
/** Bounded so a long session over many builds cannot grow it without limit.
 * Eviction is oldest-first, which for this access pattern is close enough to
 * least-recently-used and far simpler. */
const ATLAS_CACHE_MAX = 64;

/** A content hash of the atlas bytes. FNV-1a: not cryptographic, but this
 * only has to distinguish a few dozen small images, and it costs microseconds
 * against the 366 ms it decides whether to spend. */
function hashBytes(bytes: Uint8Array): string {
  let h = 2166136261;
  for (let i = 0; i < bytes.length; i++) {
    h ^= bytes[i];
    h = Math.imul(h, 16777619);
  }
  return `${bytes.length}:${(h >>> 0).toString(16)}`;
}

/** How a material blends, which is the only thing distinguishing the three a
 * chunk uses. The mesher leaves them unnamed but their alpha modes — OPAQUE,
 * MASK, BLEND — are stable and mean exactly opaque, cutout and translucent,
 * so they key the cache without depending on the order primitives arrive in. */
function blendKey(m: Any): string {
  if (m.transparent) return "BLEND";
  if ((m.alphaTest ?? 0) > 0) return "MASK";
  return "OPAQUE";
}

/** The same GLB with its embedded image, textures and samplers removed.
 *
 * Geometry is untouched — only the parts that would trigger a redundant PNG
 * decode are dropped, and the caller puts the cached materials back. Returns
 * null if the container is not something we recognise, so the caller can fall
 * back to parsing it whole rather than handing the loader a broken file.
 */
function stripEmbeddedImages(bytes: Uint8Array): { buffer: ArrayBuffer; atlas: string } | null {
  try {
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    if (view.getUint32(0, true) !== 0x46546c67) return null; // not 'glTF'
    const jsonLen = view.getUint32(12, true);
    const json = JSON.parse(new TextDecoder().decode(bytes.subarray(20, 20 + jsonLen)));
    if (!json.images?.length) return null; // nothing to save
    // Hash the atlas before dropping it — it is what decides whether an
    // already-decoded texture may stand in for this one.
    const bv = json.bufferViews?.[json.images[0].bufferView];
    if (!bv) return null;
    const binStart = 20 + jsonLen + 8; // past the JSON chunk and the BIN header
    const offset = binStart + (bv.byteOffset ?? 0);
    const atlas = hashBytes(bytes.subarray(offset, offset + bv.byteLength));
    delete json.images;
    delete json.textures;
    delete json.samplers;
    for (const m of json.materials ?? []) {
      delete m.normalTexture;
      delete m.emissiveTexture;
      delete m.occlusionTexture;
      if (m.pbrMetallicRoughness) {
        delete m.pbrMetallicRoughness.baseColorTexture;
        delete m.pbrMetallicRoughness.metallicRoughnessTexture;
      }
    }
    // The JSON chunk must stay four-byte aligned, padded with spaces.
    let text = JSON.stringify(json);
    while (text.length % 4 !== 0) text += " ";
    const jsonBytes = new TextEncoder().encode(text);
    // Everything from the end of the JSON chunk on is the BIN chunk, header
    // included — copied verbatim, since the buffer views still index into it.
    const rest = bytes.subarray(20 + jsonLen);
    const out = new Uint8Array(20 + jsonBytes.length + rest.length);
    const dv = new DataView(out.buffer);
    dv.setUint32(0, 0x46546c67, true); // 'glTF'
    dv.setUint32(4, 2, true); // version
    dv.setUint32(8, out.length, true);
    dv.setUint32(12, jsonBytes.length, true);
    dv.setUint32(16, 0x4e4f534a, true); // 'JSON'
    out.set(jsonBytes, 20);
    out.set(rest, 20 + jsonBytes.length);
    return { buffer: out.buffer, atlas };
  } catch {
    return null;
  }
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
  /** Mesh-chunk edge for this build; see [`CHUNK`]. Never zero here — a
   * requested 0 is resolved to something that spans the build at load. */
  chunkSize: number = CHUNK;

  private constructor(eng: Any, pack: Any, cfg: Any) {
    this.eng = eng;
    this.pack = pack;
    this.cfg = cfg;
    this.group.add(this.entityGroup);
  }

  static async load(
    bytes: Uint8Array,
    settle: SettleName = "in-world",
    chunkSize: number = CHUNK,
  ): Promise<World> {
    const { eng, pack, cfg } = await loadMeshEnv();
    const w = new World(eng, pack, cfg);
    w.settle = settle;
    w.schem = eng.Schematic.fromData(bytes);
    const d = w.schem.tightDimensions();
    w.dims = [Number(d.x), Number(d.y), Number(d.z)];
    // 0 means "one chunk for the whole build": round the longest edge up so a
    // single box covers it, and the build then has exactly one atlas.
    w.chunkSize = chunkSize > 0 ? chunkSize : Math.max(16, ...w.dims) + 1;
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
      this.chunkSize,
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
    void this.warmBlockMeshes();
  }

  /** Parse a one-block mesh for every state in the build, in the background.
   *
   * A flight whose mesh has not parsed yet is not drawn *at all* — its cell is
   * blank for the whole stroke, and the chunk cannot cover for it because the
   * source cell has already been cleared. Measured on BB: 601 of 5484 flights
   * over 120 ticks, arriving in bursts, one per block type the first time that
   * type set off. The old comment here called that "a frame or two, which is
   * invisible at these speeds"; it is a GLB parse, and it is not.
   *
   * Not `paletteJson()`, which was the first thing tried here and is wrong: it
   * returns bare block *names* with no properties, so every one of its 155
   * entries on BB was a cache key no flight could ever match — 155 parses paid
   * for and zero hits. Flights are keyed on the full state string. The world
   * snapshot is the same set with the properties still attached.
   *
   * Even that is a starting point, not the whole set. Some of what flies is
   * produced by the simulation and is in no initial state at all: a
   * `piston_head` only exists once a piston extends, an
   * `observer[powered=true]` only once one fires. Those are caught by
   * [`World.warmState`] off the change stream — one flight late the first time
   * each appears, which is the residual, and it is bounded by the number of
   * distinct states rather than by how long the machine runs.
   *
   * A few at a time and off the critical path: the user is looking at the
   * build while this runs, and firing two hundred parses at once would stall
   * the view to fix a flicker.
   */
  private async warmBlockMeshes(): Promise<void> {
    const token = ++this.warmToken;
    let states: string[] = [];
    try {
      const snapshot: { state: string }[] = JSON.parse(this.sim.worldSnapshotJson());
      states = [...new Set(snapshot.map((b) => b.state))];
    } catch {
      return;
    }
    for (const state of states) {
      if (token !== this.warmToken) return; // a new build was loaded
      this.warmState(state);
      // Let a handful run together — serialising one per frame took fifteen
      // seconds on a 161-state build, by which time the flicker it exists to
      // prevent has already happened.
      while (this.pendingMeshes.size >= 6 && token === this.warmToken) {
        await new Promise((r) => requestAnimationFrame(r));
      }
    }
  }

  /** Parse this state's one-block mesh now, if it is not known already. */
  private warmState(state: string): void {
    if (!state || state === "minecraft:air") return;
    if (state.startsWith("minecraft:moving_piston")) return; // never drawn
    if (this.blockMeshes.has(state) || this.pendingMeshes.has(state)) return;
    this.blockMesh(state);
  }

  private warmToken = 0;

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
        const whole = bytes.buffer.slice(
          bytes.byteOffset,
          bytes.byteOffset + bytes.byteLength,
        ) as ArrayBuffer;
        // Read the atlas hash out of the container; if this exact atlas has
        // been decoded before, parse without the image and reuse it.
        const lean = stripEmbeddedImages(bytes);
        const cached = lean ? atlasCache.get(lean.atlas) : undefined;
        const gltf = await loader.parseAsync(cached ? lean!.buffer : whole, "");
        if (cached) {
          // Swap the loader's texture-less placeholders for the real thing.
          // Disposing them matters — one abandoned material per chunk per
          // re-mesh is a leak that outruns the garbage collector at 20 tps.
          gltf.scene.traverse((o: Any) => {
            if (!o.isMesh || !o.material) return;
            const shared = cached.get(blendKey(o.material));
            if (!shared) return;
            o.material.dispose?.();
            o.material = shared;
          });
        } else if (lean) {
          // First sight of this atlas: this parse paid for the decode, so keep
          // what it produced for every chunk that turns out to use it too.
          const harvested = new Map<string, THREE.Material>();
          gltf.scene.traverse((o: Any) => {
            if (o.isMesh && o.material) harvested.set(blendKey(o.material), o.material);
          });
          if (harvested.size > 0) {
            if (atlasCache.size >= ATLAS_CACHE_MAX) {
              const oldest = atlasCache.keys().next().value;
              if (oldest !== undefined) atlasCache.delete(oldest);
            }
            atlasCache.set(lean.atlas, harvested);
          }
        }
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
        cx * this.chunkSize,
        cy * this.chunkSize,
        cz * this.chunkSize,
        cx * this.chunkSize + this.chunkSize - 1,
        cy * this.chunkSize + this.chunkSize - 1,
        cz * this.chunkSize + this.chunkSize - 1,
        // Copy the box to where it actually lives, so the re-meshed chunk
        // comes back in the same world space as the initial pass.
        cx * this.chunkSize,
        cy * this.chunkSize,
        cz * this.chunkSize,
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

  /** A block mid-flight between two cells, drawn by [`World.animate`].
   *
   * One per entry of the engine's own in-flight list, keyed by destination —
   * created and retired by [`World.syncFlights`], never by a timer here.
   */
  private flights = new Map<
    string,
    {
      state: string;
      /** The shortened piston arm, drawn while the head is beside its body;
       * null for anything that is not an arm. */
      shortState: string | null;
      /** Which of `state`/`shortState` `object` was cloned from. */
      drawnState: string;
      /** True while the arm travels away from its body, false while it
       * returns — which end of the move the body sits at. */
      extending: boolean;
      /** Null until the block's mesh finishes parsing; see [`World.animate`]. */
      object: THREE.Object3D | null;
      /** A block parked at `to` for the whole move, drawn but never moved — a
       * retracting piston's body. The chunk cannot draw it: the cell holds a
       * `moving_piston` placeholder, which meshes as a hole. */
      still: string | null;
      stillObject: THREE.Object3D | null;
      from: [number, number, number];
      to: [number, number, number];
      /** Engine tick the stroke was dispatched on. */
      started: number;
      /** Engine tick the block is written at `to`. */
      lands: number;
    }
  >();
  /** How many times each chunk has been marked dirty, and the stand-ins
   * waiting on it.
   *
   * A flight retires on the tick its block lands, which is correct and exact.
   * The chunk that must then draw that block is re-meshed **asynchronously** —
   * one `MeshResult` per chunk, tens of milliseconds each. Disposing the
   * flight at retirement left the cell drawn by nothing in between: measured
   * on BB at 1446 of 4965 retirements, up to 8 painted frames each. That is
   * the blink.
   *
   * So a flight whose cell now holds a real block is not disposed; it is
   * parked at its destination and released by [`World.releaseHandovers`] in
   * the same turn as the chunk swap that takes over from it.
   */
  private chunkRevision = new Map<string, number>();
  private handover: {
    /** The destination cell, for probes and debugging. */
    cell: string;
    chunk: string;
    /** The chunk revision that will draw this block for real. */
    revision: number;
    object: THREE.Object3D | null;
    stillObject: THREE.Object3D | null;
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

  /** Match the drawn flights to the ones the engine currently has in the air.
   *
   * Called after every batch of steps. The engine's list *is* the truth: an
   * entry appears on the tick the stroke was dispatched and is gone on the
   * tick the real block is written, so a flight created and retired from it
   * can never be drawn alongside the block it is carrying, nor vanish before
   * it arrives. Both artefacts were the old wall-clock lifetimes racing the
   * simulation for the same cell.
   */
  syncFlights(): void {
    let live: MovingBlock[];
    try {
      live = JSON.parse(this.sim.movingBlocksJson?.() ?? "[]") as MovingBlock[];
    } catch {
      live = [];
    }
    if (live.length === 0 && this.flights.size === 0) return;

    const seen = new Set<string>();
    for (const m of live) {
      const k = key(m.to[0], m.to[1], m.to[2]);
      seen.add(k);
      // A cell is not an identity. Consecutive strokes reuse a destination —
      // a sticky piston extends its head into a slot and two ticks later
      // retracts and pulls a block back into the same one — so "do I already
      // have a flight here" is the wrong question. Asking it kept the head
      // flight: the pulled block was never drawn at all, and the head stayed
      // on screen past its landing beside the real one. Compare what is
      // flying, from where, since when.
      const have = this.flights.get(k);
      if (have && have.started === m.started && have.state === m.carried &&
          have.from[0] === m.from[0] && have.from[1] === m.from[1] && have.from[2] === m.from[2]) {
        continue;
      }
      if (have) this.retire(k, have);
      // `carried`, not `state`: a retracting piston lands its base but what
      // travels is its head. Recorded even though the mesh may still be
      // parsing — the first sighting of any block state is always a cache
      // miss, and dropping those meant the first stroke of every kind never
      // animated at all.
      this.blockMesh(m.carried);
      if (m.carried_short) this.blockMesh(m.carried_short);
      if (m.remains) this.blockMesh(m.remains);
      this.flights.set(k, {
        state: m.carried,
        shortState: m.carried_short,
        drawnState: m.carried,
        extending: m.extending,
        object: null,
        still: m.remains,
        stillObject: null,
        from: m.from,
        to: m.to,
        started: m.started,
        lands: m.lands,
      });
    }
    for (const [k, f] of this.flights) {
      if (seen.has(k)) continue;
      this.retire(k, f);
      this.flights.delete(k);
    }
  }

  /** Take a flight off the live list without leaving its cell blank.
   *
   * If the cell now holds a real block, the chunk will draw it — but not until
   * it has been re-meshed, which is asynchronous. Park the flight's meshes at
   * the destination until that happens. If the cell is empty (a stroke
   * cancelled mid-flight: vanilla's `finalTick` lands air for a source piston)
   * nothing should be drawn there and they go immediately.
   */
  private retire(k: string, f: { object: THREE.Object3D | null; stillObject: THREE.Object3D | null }): void {
    if (!this.solid.has(k) || (!f.object && !f.stillObject)) {
      this.dispose(f.object);
      this.dispose(f.stillObject);
      return;
    }
    const [x, y, z] = k.split(",").map(Number);
    // Snapped, not left wherever the last frame put it: a flight retired in
    // the same frame its mesh finished parsing has never been positioned, and
    // `animate` will not touch it again.
    f.object?.position.set(x, y, z);
    const chunk = key(
      Math.floor(x / this.chunkSize),
      Math.floor(y / this.chunkSize),
      Math.floor(z / this.chunkSize),
    );
    this.handover.push({
      cell: k,
      chunk,
      revision: this.chunkRevision.get(chunk) ?? 0,
      object: f.object,
      stillObject: f.stillObject,
    });
  }

  /** Draw every in-flight block at its position as of `tickNow`.
   *
   * `tickNow` is a *fractional tick*: completed ticks plus how far the pacing
   * has carried toward the next one. Interpolating against it rather than
   * against `performance.now()` is what keeps a stroke in step with the
   * simulation — frame jitter, a throttled frame, or fifty ticks stepped at
   * once all move this clock and the stroke with it, instead of leaving an
   * animation running against a world that has moved on.
   */
  animate(tickNow: number): void {
    if (this.flights.size === 0) return;
    for (const f of this.flights.values()) {
      // A block that holds its cell for the whole move — the retracting
      // piston's body. Placed once and never touched again; only `object`
      // below is interpolated, exactly as vanilla submits its `base` slot
      // outside the translate that carries the moving one.
      if (f.still && !f.stillObject) {
        const proto = this.blockMesh(f.still);
        if (proto) {
          f.stillObject = proto.clone(true);
          f.stillObject.position.set(f.to[0], f.to[1], f.to[2]);
          this.group.add(f.stillObject);
        }
      }
      const span = f.lands - f.started;
      // Clamped, and deliberately reaching 1 before the flight retires: a
      // piston's blocks are visually home a tick before the world is written,
      // which is vanilla's own `progressO` having reached 1.0 on the landing
      // tick. Holding them at the destination for that tick is correct.
      const t = span > 0 ? Math.min(1, Math.max(0, (tickNow - f.started) / span)) : 1;
      // A piston arm is drawn shortened while it is within half a block of its
      // body and full length beyond that, or its shaft passes visibly through
      // the back of the piston as it comes home. The body sits at `from` on an
      // extension and at `to` on a retraction, so "near the body" flips with
      // the direction of travel — which is exactly the pair of opposite
      // comparisons vanilla's `PistonHeadRenderer` spells out (`SHORT` at
      // `progress <= 0.5` extending, `progress >= 0.5` retracting).
      const want =
        f.shortState && (f.extending ? t <= 0.5 : t >= 0.5) ? f.shortState : f.state;
      // Late arrival: the mesh finished parsing after the flight began. Also
      // the swap between the two arm lengths — a clone, not a re-parse.
      if (!f.object || f.drawnState !== want) {
        const proto = this.blockMesh(want);
        if (!proto) {
          if (!f.object) continue; // still parsing; try again next frame
        } else {
          this.dispose(f.object);
          f.object = proto.clone(true);
          f.drawnState = want;
          this.group.add(f.object);
        }
      }
      // Linear, like the game's own `getExtendedProgress` — a piston does not
      // ease in or out, and faking it reads as lag.
      f.object.position.set(
        f.from[0] + (f.to[0] - f.from[0]) * t,
        f.from[1] + (f.to[1] - f.from[1]) * t,
        f.from[2] + (f.to[2] - f.from[2]) * t,
      );
    }
  }

  private dispose(object: THREE.Object3D | null): void {
    if (!object) return;
    this.group.remove(object);
    object.traverse((o: Any) => o.geometry?.dispose?.());
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
    for (const f of this.flights.values()) {
      this.dispose(f.object);
      this.dispose(f.stillObject);
    }
    this.flights.clear();
    for (const h of this.handover) {
      this.dispose(h.object);
      this.dispose(h.stillObject);
    }
    this.handover = [];
    this.chunkRevision.clear();
  }

  /** Apply what the last tick changed: patch the schematic, mark chunks.
   *
   * Animation is not decided here. Strokes come from
   * [`World.syncFlights`] — the engine's own in-flight list — because the
   * only thing this stream says about a stroke is that some cell became a
   * placeholder, and everything else the app used to infer from that (which
   * block set off, from where, for how long) was a guess racing the
   * simulation.
   */
  applyChanges(changes: Change[]): void {
    // Entities first, and unconditionally: an entity can move on a tick that
    // changes no block at all, so gating this on `changes.length` would freeze
    // a boat drifting through still air.
    this.syncEntities();
    for (const c of changes) {
      const [x, y, z] = c.pos;
      // A placeholder is not a block anyone should see. The chunk shows a
      // hole for the two ticks the stroke lasts and the sliding mesh fills
      // it — meshing `moving_piston` instead drew a stand-in that popped.
      // Anything the simulation writes is a candidate for flying later — a
      // `piston_head` that has just landed will fly again the next time that
      // piston retracts. Warming here is what covers the states the build's
      // palette never contained.
      this.warmState(c.to);
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
            const k = key(
              Math.floor((x + dx) / this.chunkSize),
              Math.floor((y + dy) / this.chunkSize),
              Math.floor((z + dz) / this.chunkSize),
            );
            this.dirty.add(k);
            this.chunkRevision.set(k, (this.chunkRevision.get(k) ?? 0) + 1);
          }
    }
    // *After* the writes, not before. A retiring flight has to know whether the
    // cell it is vacating now holds a real block — if it does, the flight hands
    // over to the chunk instead of disappearing; if the stroke was cancelled
    // and the cell is air, nothing should be drawn and it goes at once. Both
    // answers live in `solid` and `chunkRevision`, which the loop above has
    // just brought up to date.
    this.syncFlights();
  }

  /** Re-mesh everything marked dirty. Called once per animation frame, not
   * per tick, so a fast-forward costs one re-mesh rather than dozens. */
  async flush(): Promise<number> {
    if (this.dirty.size === 0) return 0;
    const todo = [...this.dirty];
    this.dirty.clear();
    // The schematic already holds every change these chunks are being re-meshed
    // for, so this is the revision each one is about to catch up to. Captured
    // before the first `await`: a change landing mid-flush belongs to the next
    // pass, and a hand-off must not be released on a mesh that predates it.
    const caughtUpTo = new Map(todo.map((k) => [k, this.chunkRevision.get(k) ?? 0]));
    for (const k of todo) {
      const [cx, cy, cz] = k.split(",").map(Number);
      await this.remeshChunk(cx, cy, cz);
      // Same turn as the chunk swap inside `installChunk`: the new geometry is
      // in the scene, so whatever was standing in for it can go now. A frame
      // later would show both; the retirement that queued it would have shown
      // neither.
      this.releaseHandovers(k, caughtUpTo.get(k) ?? 0);
    }
    return todo.length;
  }

  /** Drop the stand-ins for `chunk` that `revision` has now drawn for real. */
  private releaseHandovers(chunk: string, revision: number): void {
    if (this.handover.length === 0) return;
    this.handover = this.handover.filter((h) => {
      if (h.chunk !== chunk || h.revision > revision) return true;
      this.dispose(h.object);
      this.dispose(h.stillObject);
      return false;
    });
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
