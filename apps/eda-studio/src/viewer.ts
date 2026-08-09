/** The design canvas: a purpose-built three.js voxel renderer, plus an
 * optional TEXTURED view meshed by nucleation itself.
 *
 * RENDERER DECISION (documented in README.md). Two views, one scene:
 *
 *  - ABSTRACT (default): the design document's flatten() already splits the
 *    build into named layers (`bus:x`, `inst:y`, loose base), and an EDA
 *    canvas wants per-bus colours and red failed layers more than it wants
 *    textured terrain. Blocks render as flat-shaded instanced cubes coloured
 *    by layer/block kind — rebuilds in milliseconds on every document edit.
 *  - TEXTURED: nucleation's own meshing pipeline (`meshing` feature, in the
 *    wasm build) turns the composited design into a GLB against a
 *    user-supplied resource pack; three.js loads it with GLTFLoader. This is
 *    what the blocks actually look like in Minecraft. Bus colours stay
 *    readable because the abstract bus/failed layers can be overlaid on top.
 *
 * Port markers, instance gizmos and labels are drawn in BOTH views: the
 * schematic's meaning must never be hidden by its skin.
 */
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import type { Block, LayerBlocks, SceneModel, Vec3 } from "./studio";

/** A connectable endpoint: a declared design port, or an instance port. */
export interface PortMarker {
  name: string;
  /** Signal direction as the FABRIC sees it: an input drives buses. */
  kind: "input" | "output";
  anchor: Vec3;
  step: Vec3;
  width: number;
  /** Rendered in the label, e.g. `"uint8"`. */
  ty: string;
  /** False for executor-only hardware (a lever input a bus cannot drive). */
  routable: boolean;
  /** Why not, when `routable` is false. */
  blocked?: string;
  /** Owning instance, for instance ports. */
  instance?: string;
}
export interface GateMarker { bus: string; name: string; anchor: Vec3; step: Vec3; width: number; }
export interface InstanceMarker { name: string; cell: string; at: Vec3; dims: Vec3; rot: number; }

export type Selection = { kind: "instance" | "port" | "gate"; id: string } | null;

/** Markers float just past the LAST bit of the column (anchor + step*(w-1))
 *  so they never sit inside the rendered stack. */
function markerPos(anchor: Vec3, step: Vec3, width: number): Vec3 {
  const w = Math.max(width - 1, 0);
  return [
    anchor[0] + step[0] * w + (step[0] ? Math.sign(step[0]) : 0),
    anchor[1] + step[1] * w + (step[1] ? Math.sign(step[1]) * 2 : 2),
    anchor[2] + step[2] * w + (step[2] ? Math.sign(step[2]) : 0),
  ];
}

export interface ViewerCallbacks {
  onPortClick(name: string): void;
  onPortHover(name: string | null): void;
  onInstanceClick(name: string): void;
  onDragMove(kind: "instance" | "gate", id: string, ground: Vec3): void;
  onDragEnd(kind: "instance" | "gate", id: string, ground: Vec3): void;
  /** A plain click on empty ground (cell-placement mode / deselect). */
  onGroundClick(ground: Vec3): void;
}

const BLOCK_COLORS: [RegExp, number][] = [
  [/redstone_wire/, 0xb71c1c],
  [/redstone_torch|redstone_wall_torch/, 0xff5252],
  [/repeater|comparator/, 0xb0a8b9],
  [/lever|button/, 0x8d6e63],
  [/redstone_lamp/, 0xffd54f],
  [/redstone_block/, 0xd32f2f],
  [/piston/, 0xcabf9b],
  [/glass/, 0xb3e5fc],
  [/observer|target/, 0x9e9e9e],
  [/slab|stone|deepslate|quartz/, 0x78909c],
];
const DYE_COLORS: Record<string, number> = {
  white: 0xe8e8e8, orange: 0xf9801d, magenta: 0xc74ebd, light_blue: 0x3ab3da,
  yellow: 0xfed83d, lime: 0x80c71f, pink: 0xf38baa, gray: 0x474f52,
  light_gray: 0x9d9d97, cyan: 0x169c9c, purple: 0x8932b8, blue: 0x3c44aa,
  brown: 0x835432, green: 0x5e7c16, red: 0xb02e26, black: 0x1d1d21,
};

function blockColor(name: string): number {
  const short = name.replace("minecraft:", "");
  const dye = /^([a-z_]+?)_(concrete|wool|stained_glass|terracotta)/.exec(short);
  if (dye && DYE_COLORS[dye[1]] != null) return DYE_COLORS[dye[1]];
  for (const [re, c] of BLOCK_COLORS) if (re.test(short)) return c;
  return 0x90a4ae;
}

// ---- voxel merge ----------------------------------------------------------
//
// One cell is meshed ONCE, into a merged buffer geometry per colour, in
// cell-local space. Its placements are then instances of that geometry, so
// a drag or a rotate is one Matrix4 write — no geometry is rebuilt and no
// blocks are re-read. Faces are NOT culled against neighbours: the cubes are
// 0.94 wide, so the 0.06 gap between them would show straight through a
// hollowed shell, and the gapped look is the abstract view's whole idiom.
const CUBE = 0.47; // half of 0.94
const FACES: { v: [number, number, number][]; n: [number, number, number] }[] = [
  { n: [1, 0, 0], v: [[CUBE, -CUBE, -CUBE], [CUBE, CUBE, -CUBE], [CUBE, CUBE, CUBE], [CUBE, -CUBE, CUBE]] },
  { n: [-1, 0, 0], v: [[-CUBE, -CUBE, CUBE], [-CUBE, CUBE, CUBE], [-CUBE, CUBE, -CUBE], [-CUBE, -CUBE, -CUBE]] },
  { n: [0, 1, 0], v: [[-CUBE, CUBE, CUBE], [CUBE, CUBE, CUBE], [CUBE, CUBE, -CUBE], [-CUBE, CUBE, -CUBE]] },
  { n: [0, -1, 0], v: [[-CUBE, -CUBE, -CUBE], [CUBE, -CUBE, -CUBE], [CUBE, -CUBE, CUBE], [-CUBE, -CUBE, CUBE]] },
  { n: [0, 0, 1], v: [[-CUBE, -CUBE, CUBE], [CUBE, -CUBE, CUBE], [CUBE, CUBE, CUBE], [-CUBE, CUBE, CUBE]] },
  { n: [0, 0, -1], v: [[CUBE, -CUBE, -CUBE], [-CUBE, -CUBE, -CUBE], [-CUBE, CUBE, -CUBE], [CUBE, CUBE, -CUBE]] },
];

/** Merge unit cubes into ONE buffer geometry, carrying each block's colour as
 *  a vertex attribute.
 *
 *  Colour-as-attribute rather than colour-as-material is what gets a cell down
 *  to a single draw call: an ADD007 body uses seven different block colours,
 *  and one material per colour meant seven InstancedMeshes (and seven draw
 *  calls) per cell however few cells there were. */
function mergeCubes(blocks: Block[], override: number | null): THREE.BufferGeometry {
  const solid = blocks.filter((b) => b.name !== "minecraft:air");
  const n = solid.length;
  const pos = new Float32Array(n * 24 * 3);
  const nor = new Float32Array(n * 24 * 3);
  const col = new Float32Array(n * 24 * 3);
  const idx = new Uint32Array(n * 36);
  const c = new THREE.Color();
  let vi = 0, ii = 0, base = 0;
  for (const b of solid) {
    c.setHex(override ?? blockColor(b.name)); // -> the renderer's working space
    for (const f of FACES) {
      for (const [vx, vy, vz] of f.v) {
        pos[vi] = b.x + vx; nor[vi] = f.n[0]; col[vi++] = c.r;
        pos[vi] = b.y + vy; nor[vi] = f.n[1]; col[vi++] = c.g;
        pos[vi] = b.z + vz; nor[vi] = f.n[2]; col[vi++] = c.b;
      }
      idx[ii++] = base; idx[ii++] = base + 1; idx[ii++] = base + 2;
      idx[ii++] = base; idx[ii++] = base + 2; idx[ii++] = base + 3;
      base += 4;
    }
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(pos, 3));
  geo.setAttribute("normal", new THREE.BufferAttribute(nor, 3));
  geo.setAttribute("color", new THREE.BufferAttribute(col, 3));
  geo.setIndex(new THREE.BufferAttribute(idx, 1));
  geo.computeBoundingBox();   // InstancedMesh.computeBoundingSphere needs it
  geo.computeBoundingSphere();
  return geo;
}

/** `Design::transform_pos` is a quarter-turn about the cell's min corner then
 *  a translation, which is exactly one Matrix4:
 *
 *      world = T(at + offset(rot, size)) · R_y(-rot) · local
 *
 *  The engine's turn maps `(x,z) -> (sz-1-z, x)`; the linear half of that is
 *  three.js's `R_y(-90°)` and the constant half is the offset below. Four
 *  rotations are therefore four matrices over ONE mesh. */
function placementMatrix(at: Vec3, rot: number, size: Vec3, out: THREE.Matrix4): THREE.Matrix4 {
  const q = (((rot % 360) + 360) % 360) / 90;
  out.makeRotationY((-q * Math.PI) / 2);
  const ox = q === 1 ? size[2] - 1 : q === 2 ? size[0] - 1 : 0;
  const oz = q === 2 ? size[2] - 1 : q === 3 ? size[0] - 1 : 0;
  out.setPosition(at[0] + ox, at[1], at[2] + oz);
  return out;
}

const FAILED_COLOR = 0xff4040;
/** Drives a bus (fabric input). */
const DRIVER_COLOR = 0x7bd88f;
/** Receives a bus (fabric output). */
const SINK_COLOR = 0x4fc3f7;
/** Executor-only hardware: real IO, but no bus can land on it. */
const BLOCKED_COLOR = 0x8d8d8d;
const SELECT_COLOR = 0xffe082;
const HOVER_COLOR = 0xffffff;

export class Viewer {
  scene = new THREE.Scene();
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  controls: OrbitControls;
  /** Why the camera is locked, or `null`. While locked, orbit/pan/zoom stay
   *  OFF: a drag belongs to the interaction in progress (drawing a bus,
   *  moving an instance), and letting OrbitControls also see it means every
   *  connect gesture spins the scene. */
  private cameraLock: string | null = null;
  private texturedGroup = new THREE.Group();
  private markerGroup = new THREE.Group();
  private gizmoGroup = new THREE.Group();
  private ghostGroup = new THREE.Group();
  private raycaster = new THREE.Raycaster();
  private ground = new THREE.Plane(new THREE.Vector3(0, 1, 0), 0);
  private drag: { kind: "instance" | "gate"; id: string; y: number } | null = null;
  private cb: ViewerCallbacks;
  private pickables: THREE.Object3D[] = [];
  private labelLayer: HTMLDivElement;
  /** name -> {el, world} for the projected label pass. */
  private labels: { el: HTMLDivElement; world: THREE.Vector3 }[] = [];
  private ports: PortMarker[] = [];
  private instances: InstanceMarker[] = [];
  private selection: Selection = null;
  private hovered: string | null = null;
  private showIo = true;
  private textured = false;
  private ghost: { from: Vec3; to: THREE.Vector3 } | null = null;

  constructor(container: HTMLElement, cb: ViewerCallbacks) {
    this.cb = cb;
    this.renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    container.appendChild(this.renderer.domElement);
    this.labelLayer = document.createElement("div");
    this.labelLayer.className = "label-layer";
    container.appendChild(this.labelLayer);
    this.camera = new THREE.PerspectiveCamera(50, 1, 0.1, 4000);
    this.camera.position.set(28, 34, 42);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.target.set(9, 4, 9);
    this.scene.background = new THREE.Color(0x14161a);
    this.scene.add(new THREE.AmbientLight(0xffffff, 0.75));
    const sun = new THREE.DirectionalLight(0xffffff, 1.4);
    sun.position.set(40, 80, 20);
    this.scene.add(sun);
    const grid = new THREE.GridHelper(200, 200, 0x2c313c, 0x20242c);
    grid.position.set(0, -0.51, 0);
    this.texturedGroup.visible = false;
    this.scene.add(grid, this.cellGroup, this.looseGroup, this.busGroup,
      this.texturedGroup, this.markerGroup, this.gizmoGroup, this.ghostGroup);

    const resize = () => {
      const w = container.clientWidth, h = container.clientHeight;
      this.renderer.setSize(w, h);
      this.camera.aspect = w / h;
      this.camera.updateProjectionMatrix();
    };
    new ResizeObserver(resize).observe(container);
    resize();

    const el = this.renderer.domElement;
    el.addEventListener("pointerdown", (e) => this.pointerDown(e));
    el.addEventListener("pointermove", (e) => this.pointerMove(e));
    el.addEventListener("pointerup", (e) => this.pointerUp(e));

    let last = performance.now();
    const tick = () => {
      requestAnimationFrame(tick);
      const t0 = performance.now();
      this.perf.frameGapMs = t0 - last;
      last = t0;
      this.controls.update();
      this.projectLabels();
      this.renderer.render(this.scene, this.camera);
      const dt = performance.now() - t0;
      this.perf.frames++;
      this.perf.lastFrameMs = dt;
      this.perf.totalFrameMs += dt;
      this.perf.totalGapMs += this.perf.frameGapMs;
    };
    tick();
  }

  // ---- profiling ----------------------------------------------------------

  /** Cumulative frame accounting; `profileReset()` zeroes it, `profile()`
   *  reads it. Cheap enough (two subtractions) to leave always-on. */
  perf = { frames: 0, lastFrameMs: 0, totalFrameMs: 0, frameGapMs: 0, totalGapMs: 0 };

  profileReset() {
    this.perf = { frames: 0, lastFrameMs: 0, totalFrameMs: 0, frameGapMs: 0, totalGapMs: 0 };
    this.sceneMs = { placements: 0, unique: 0 };
  }

  /** Draw calls are the instancing proof: one InstancedMesh per (variant,
   *  colour) group is ONE call however many placements it draws. */
  profile() {
    const r = this.renderer.info;
    return {
      frames: this.perf.frames,
      avgFrameMs: this.perf.frames ? this.perf.totalFrameMs / this.perf.frames : 0,
      avgGapMs: this.perf.frames ? this.perf.totalGapMs / this.perf.frames : 0,
      fps: this.perf.totalGapMs ? (this.perf.frames * 1000) / this.perf.totalGapMs : 0,
      drawCalls: r.render.calls,
      triangles: r.render.triangles,
      geometries: r.memory.geometries,
      textures: r.memory.textures,
      programs: r.programs?.length ?? 0,
      cellGroups: this.cellGroup.children.length,
      cellMeshes: this.cellMeshes.size,
      busMeshes: this.busGroup.children.length,
      looseMeshes: this.looseGroup.children.length,
      sceneObjects: this.cellGroup.children.length + this.busGroup.children.length
        + this.looseGroup.children.length,
      markerObjects: this.markerGroup.children.length,
      labels: this.labels.length,
      sceneMs: { ...this.sceneMs },
    };
  }

  // -- scene rebuild --------------------------------------------------------

  private static disposeDeep(group: THREE.Group) {
    group.traverse((obj) => {
      // Shared marker geometries outlive any one rebuild.
      if (obj.userData?.shared) return;
      const m = obj as Partial<THREE.Mesh>;
      (m.geometry as THREE.BufferGeometry | undefined)?.dispose();
      const mats = Array.isArray(m.material) ? m.material : m.material ? [m.material] : [];
      for (const mat of mats) (mat as THREE.Material).dispose();
    });
    group.clear();
  }

  /** Meshed cells, keyed by variant. One `InstancedMesh` per (geometry,
   *  material) group; every placement of the cell is a row in the same
   *  instance set. */
  private cellMeshes = new Map<string, {
    size: Vec3;
    groups: THREE.InstancedMesh[];
    /** instance name -> row in the instance set. */
    slots: Map<string, number>;
    /** Allocated rows (>= slots.size); growing reuses the GEOMETRY. */
    capacity: number;
    /** Last matrix written per slot, so an unchanged placement writes nothing. */
    written: Map<string, string>;
  }>();
  /** Unique (never instanced) layers: one per bus, one for the loose base. */
  private uniqueMeshes = new Map<string, THREE.Mesh[]>();
  private cellGroup = new THREE.Group();
  private looseGroup = new THREE.Group();
  private busGroup = new THREE.Group();
  private lastScene: SceneModel | null = null;
  private scratch = new THREE.Matrix4();

  /** Update the scene from the document model, rebuilding ONLY what
   *  `model.dirty` names. A drag reports `placements` alone, so it writes one
   *  matrix per moved instance and touches no geometry at all. */
  setScene(model: SceneModel) {
    this.lastScene = model;
    const dirtyVariants = new Set(model.dirty.variants);

    // (a) cells: mesh each distinct variant once, then place it by matrix.
    for (const [key, variant] of model.variants) {
      const have = this.cellMeshes.get(key);
      if (have && !dirtyVariants.has(key)) continue;
      if (have) this.disposeCell(key);
      this.meshBuilds.cells++;
      const groups: THREE.InstancedMesh[] = [];
      const n = Math.max(1, model.placements.filter((p) => p.variant === key).length);
      const capacity = Math.max(4, 1 << Math.ceil(Math.log2(n)));
      const mesh = new THREE.InstancedMesh(
        mergeCubes(variant.blocks, null),
        new THREE.MeshLambertMaterial({ vertexColors: true }),
        capacity);
      mesh.count = 0;
      this.meshBuilds.instancedGroups++;
      groups.push(mesh);
      this.cellGroup.add(mesh);
      this.cellMeshes.set(key, {
        size: variant.size, groups, capacity,
        slots: new Map(), written: new Map(),
      });
    }
    for (const key of [...this.cellMeshes.keys()]) {
      if (!model.variants.has(key)) this.disposeCell(key);
    }

    // (b) placements: matrices only.
    let t = performance.now();
    if (model.dirty.placements || dirtyVariants.size) this.writePlacements(model);
    this.sceneMs.placements += performance.now() - t;
    t = performance.now();

    // (c) buses + the loose base: unique geometry, rebuilt only when their
    // own blocks changed. In the textured view the cells and the base hide
    // (the pack draws them) but the buses stay abstract on top: the routing is
    // what the colours encode, and a resource pack cannot show it.
    for (const name of model.dirty.buses) this.rebuildUnique(model, `bus:${name}`);
    for (const layer of model.buses) {
      const built = this.uniqueMeshes.get(layer.layer);
      // A bus can flip to FAILED (red) without its blocks moving.
      if (built && built[0]?.userData.failed !== layer.failed) this.rebuildUnique(model, layer.layer);
    }
    for (const name of [...this.uniqueMeshes.keys()]) {
      if (name === "loose") continue;
      if (!model.buses.some((b) => b.layer === name)) this.disposeUnique(name);
    }
    if (model.dirty.loose || !this.uniqueMeshes.has("loose")) this.rebuildUnique(model, "loose");
    this.sceneMs.unique += performance.now() - t;

    this.cellGroup.visible = !this.textured;
    this.looseGroup.visible = !this.textured;
  }

  /** Where `setScene` spends its time: matrices vs unique (bus/loose) meshes. */
  sceneMs = { placements: 0, unique: 0 };

  /** Move ONE instance, at frame rate, without asking the document anything.
   *
   *  This is what per-cell instancing buys: the cell is already meshed, so a
   *  drag frame is one Matrix4 per (cell, colour) group plus a rigid nudge of
   *  that instance's own markers. Nothing is re-read, re-meshed or rebuilt.
   *  The document is committed separately (on drop, or coalesced per frame
   *  when live re-route is on) and `setMarkers` reconciles afterwards. */
  previewInstance(name: string, at: Vec3, rot?: number): boolean {
    const inst = this.instances.find((i) => i.name === name);
    if (!inst) return false;
    const r = rot ?? inst.rot;
    const d = [at[0] - inst.at[0], at[1] - inst.at[1], at[2] - inst.at[2]] as Vec3;
    if (!d[0] && !d[1] && !d[2] && r === inst.rot) return true;
    inst.at = [...at] as Vec3;
    inst.rot = r;

    for (const entry of this.cellMeshes.values()) {
      const row = entry.slots.get(name);
      if (row == null) continue;
      placementMatrix(at, r, entry.size, this.scratch);
      for (const g of entry.groups) {
        g.setMatrixAt(row, this.scratch);
        g.instanceMatrix.needsUpdate = true;
        g.computeBoundingSphere();
      }
      entry.written.set(name, `${row}|${at[0]},${at[1]},${at[2]}|${r}`);
      this.meshBuilds.matrixWrites++;
    }
    const touched = new Set<"solid" | "ghost">();
    for (const row of this.portRows) {
      if (row.p.instance !== name) continue;
      row.pos = [row.pos[0] + d[0], row.pos[1] + d[1], row.pos[2] + d[2]];
      row.world.x += d[0]; row.world.y += d[1]; row.world.z += d[2];
      const mesh = this.portMeshes[row.group];
      const emph = this.selection?.kind === "port" && this.selection.id === row.p.name
        || this.hovered === row.p.name;
      mesh?.setMatrixAt(row.row, this.portMatrix(row, emph));
      touched.add(row.group);
    }
    for (const g of touched) {
      const mesh = this.portMeshes[g];
      if (!mesh) continue;
      mesh.instanceMatrix.needsUpdate = true;
      mesh.computeBoundingSphere();
    }
    for (const s of this.instStyles) {
      if (s.inst.name !== name) continue;
      s.pick.position.x += d[0]; s.pick.position.y += d[1]; s.pick.position.z += d[2];
      s.pick.userData.y = at[1];
    }
    if (this.edgesSelected && this.selection?.kind === "instance" && this.selection.id === name) {
      this.edgesSelected.position.x += d[0];
      this.edgesSelected.position.y += d[1];
      this.edgesSelected.position.z += d[2];
    }
    if (this.gizmoOwner === name) {
      this.gizmoGroup.position.x += d[0];
      this.gizmoGroup.position.y += d[1];
      this.gizmoGroup.position.z += d[2];
      if (this.gizmoLabelWorld) {
        this.gizmoLabelWorld.x += d[0];
        this.gizmoLabelWorld.y += d[1];
        this.gizmoLabelWorld.z += d[2];
      }
    }
    return true;
  }

  private layerOf(model: SceneModel, name: string): LayerBlocks | null {
    if (name === "loose") return model.loose;
    return model.buses.find((b) => b.layer === name) ?? null;
  }

  private rebuildUnique(model: SceneModel, name: string) {
    this.disposeUnique(name);
    const layer = this.layerOf(model, name);
    if (!layer || layer.blocks.length === 0) return;
    const isBus = name.startsWith("bus:");
    this.meshBuilds[isBus ? "buses" : "loose"]++;
    const override = layer.failed ? FAILED_COLOR : layer.color;
    const mat = new THREE.MeshLambertMaterial({
      vertexColors: true,
      transparent: layer.failed || (this.textured && isBus),
      opacity: layer.failed ? 0.85 : this.textured && isBus ? 0.75 : 1,
    });
    const mesh = new THREE.Mesh(mergeCubes(layer.blocks, override), mat);
    mesh.userData.failed = layer.failed;
    (isBus ? this.busGroup : this.looseGroup).add(mesh);
    this.uniqueMeshes.set(name, [mesh]);
  }

  private disposeUnique(name: string) {
    for (const m of this.uniqueMeshes.get(name) ?? []) {
      m.geometry.dispose();
      (m.material as THREE.Material).dispose();
      m.removeFromParent();
    }
    this.uniqueMeshes.delete(name);
  }

  private disposeCell(key: string) {
    for (const m of this.cellMeshes.get(key)?.groups ?? []) {
      m.geometry.dispose();
      (m.material as THREE.Material).dispose();
      m.removeFromParent();
    }
    this.cellMeshes.delete(key);
  }

  /** Write the instance matrices. Rows are stable per instance name, and a
   *  placement whose transform is unchanged writes nothing — so dragging one
   *  instance out of ten touches exactly one row. */
  private writePlacements(model: SceneModel) {
    const rows = new Map<string, typeof model.placements>();
    for (const p of model.placements) {
      let list = rows.get(p.variant);
      if (!list) rows.set(p.variant, (list = []));
      list.push(p);
    }
    for (const [key, entry] of this.cellMeshes) {
      const list = rows.get(key) ?? [];
      if (list.length > entry.capacity) {
        // Grow the instance set. The GEOMETRY and MATERIAL carry over — this
        // is an allocation, not a re-mesh, and the counters keep them apart.
        const capacity = Math.max(4, 1 << Math.ceil(Math.log2(list.length)));
        const grown: THREE.InstancedMesh[] = [];
        for (const old of entry.groups) {
          const mesh = new THREE.InstancedMesh(old.geometry, old.material as THREE.Material, capacity);
          mesh.count = 0;
          old.removeFromParent();
          grown.push(mesh);
          this.cellGroup.add(mesh);
        }
        entry.groups = grown;
        entry.capacity = capacity;
        entry.written.clear();
        this.meshBuilds.instanceAllocs++;
      }
      let dirty = false;
      entry.slots.clear();
      list.forEach((p, row) => {
        entry.slots.set(p.instance, row);
        const stamp = `${row}|${p.at[0]},${p.at[1]},${p.at[2]}|${p.rot}`;
        if (entry.written.get(p.instance) === stamp) return;
        entry.written.set(p.instance, stamp);
        placementMatrix(p.at, p.rot, entry.size, this.scratch);
        for (const g of entry.groups) g.setMatrixAt(row, this.scratch);
        dirty = true;
        this.meshBuilds.matrixWrites++;
      });
      for (const g of entry.groups) {
        if (g.count !== list.length) { g.count = list.length; dirty = true; }
        if (dirty) {
          g.instanceMatrix.needsUpdate = true;
          // Bounds over the INSTANCE matrices, so a group whose placements are
          // all off-screen is still frustum-culled.
          g.computeBoundingSphere();
        }
      }
      for (const name of [...entry.written.keys()]) {
        if (!entry.slots.has(name)) entry.written.delete(name);
      }
    }
  }

  /** Swap in a GLB meshed by nucleation against a resource pack. Passing
   *  `null` drops the textured view and returns to pure abstract. */
  async setTexturedGlb(glb: ArrayBuffer | null): Promise<void> {
    Viewer.disposeDeep(this.texturedGroup);
    if (!glb) {
      this.textured = false;
      this.texturedGroup.visible = false;
      return;
    }
    const loader = new GLTFLoader();
    const gltf = await loader.parseAsync(glb, "");
    // The mesher emits the schematic in block coordinates already; keep the
    // scene graph as-is so markers and gizmos line up with the voxels.
    this.texturedGroup.add(gltf.scene);
    this.textured = true;
    this.texturedGroup.visible = true;
    this.applyTextured();
  }

  /** Whether the textured view is currently shown. */
  isTextured(): boolean {
    return this.textured;
  }

  setTexturedVisible(on: boolean) {
    this.textured = on && this.texturedGroup.children.length > 0;
    this.texturedGroup.visible = this.textured;
    this.applyTextured();
  }

  /** Cells and the loose base hide behind the pack; the bus layers stay
   *  abstract on top, translucent so the terrain reads through them. Only the
   *  bus materials change, so this re-meshes the buses and nothing else. */
  private applyTextured() {
    this.cellGroup.visible = !this.textured;
    this.looseGroup.visible = !this.textured;
    if (!this.lastScene) return;
    for (const layer of this.lastScene.buses) this.rebuildUnique(this.lastScene, layer.layer);
  }

  setShowIo(on: boolean) {
    this.showIo = on;
    this.refreshMarkers();
  }

  /** Selection and hover are STYLE, not structure. They used to tear down and
   *  rebuild every marker, gizmo and label element — 54 DOM nodes recreated on
   *  each mouse move across a port, which is what made hovering feel sticky.
   *  Now only colours, the emphasis geometry and the gizmo change. */
  setSelection(sel: Selection) {
    const wasInst = this.selection?.kind === "instance" ? this.selection.id : null;
    const isInst = sel?.kind === "instance" ? sel.id : null;
    this.selection = sel;
    this.restyleMarkers();
    // Which box is highlighted moved between the merged batch and its own
    // object, so the two line meshes are re-split (a dozen boxes: microseconds).
    if (wasInst !== isInst) this.rebuildEdges();
    this.rebuildGizmo();
  }

  setHovered(name: string | null) {
    if (this.hovered === name) return;
    this.hovered = name;
    this.restyleMarkers();
  }

  /** Draw a pending connection from a port's marker to a world point. */
  setGhost(from: Vec3 | null, to?: THREE.Vector3) {
    this.ghost = from && to ? { from, to } : null;
    this.refreshGhost();
  }

  private refreshGhost() {
    Viewer.disposeDeep(this.ghostGroup);
    if (!this.ghost) return;
    const a = new THREE.Vector3(...this.ghost.from);
    const geo = new THREE.BufferGeometry().setFromPoints([a, this.ghost.to]);
    this.ghostGroup.add(new THREE.Line(geo, new THREE.LineDashedMaterial({
      color: 0xffe082, dashSize: 0.6, gapSize: 0.4,
    })));
    const dot = new THREE.Mesh(
      new THREE.SphereGeometry(0.35, 12, 8),
      new THREE.MeshBasicMaterial({ color: 0xffe082 }),
    );
    dot.position.copy(this.ghost.to);
    this.ghostGroup.add(dot);
  }

  private gates: GateMarker[] = [];

  setMarkers(ports: PortMarker[], gates: GateMarker[], instances: InstanceMarker[]) {
    this.ports = ports;
    this.gates = gates;
    this.instances = instances;
    this.refreshMarkers();
  }

  private label(text: string, cls: string, world: THREE.Vector3): HTMLDivElement {
    const el = document.createElement("div");
    el.className = `mk-label ${cls}`;
    el.textContent = text;
    this.labelLayer.appendChild(el);
    this.labels.push({ el, world });
    return el;
  }

  /** Cone/octahedron geometries are shared and pre-built in both sizes: the
   *  emphasis a selection or hover applies is then a geometry SWAP, not an
   *  allocation. */
  private static shared = (() => {
    const mk = (g: THREE.BufferGeometry) => { g.userData.shared = true; return g; };
    return {
      cone: mk(new THREE.ConeGeometry(0.45, 0.9, 4)),
      coneBig: mk(new THREE.ConeGeometry(0.55, 0.9, 4)),
      oct: mk(new THREE.OctahedronGeometry(0.55)),
      octBig: mk(new THREE.OctahedronGeometry(0.7)),
    };
  })();

  // ---- the marker layer, batched ------------------------------------------
  //
  // Ports were one Mesh each: ten placed cells is ~40 cones, ~40 draw calls,
  // all of them rebuilt whenever anything changed. They are now two
  // InstancedMeshes (opaque routable, translucent blocked) with a per-instance
  // colour, and the instance bounding boxes are one merged LineSegments plus a
  // separate object for the selected one (the only one that ever moves alone).
  // Hover and selection write a colour and a matrix; nothing is rebuilt.

  private portRows: {
    p: PortMarker; group: "solid" | "ghost"; row: number; world: THREE.Vector3;
    el: HTMLDivElement; pos: Vec3;
  }[] = [];
  private portMeshes: { solid: THREE.InstancedMesh | null; ghost: THREE.InstancedMesh | null } =
    { solid: null, ghost: null };
  private gateStyles: { id: string; mesh: THREE.Mesh; mat: THREE.MeshLambertMaterial }[] = [];
  private instStyles: { inst: InstanceMarker; pick: THREE.Mesh }[] = [];
  private edgesRest: THREE.LineSegments | null = null;
  private edgesSelected: THREE.LineSegments | null = null;
  private gizmoOwner: string | null = null;
  private gizmoLabelWorld: THREE.Vector3 | null = null;
  private mScratch = new THREE.Matrix4();
  private qScratch = new THREE.Quaternion();
  private vScratch = new THREE.Vector3();
  private sScratch = new THREE.Vector3();

  /** Emphasis is the CONE RADIUS, so it is an x/z scale on the shared cone —
   *  identical geometry to the old 0.45 -> 0.55 swap, one matrix write. */
  private portMatrix(row: { p: PortMarker; pos: Vec3 }, big: boolean): THREE.Matrix4 {
    const s = big ? 0.55 / 0.45 : 1;
    this.vScratch.set(...row.pos);
    this.qScratch.setFromAxisAngle(new THREE.Vector3(1, 0, 0), row.p.kind === "input" ? 0 : Math.PI);
    this.sScratch.set(s, 1, s);
    return this.mScratch.compose(this.vScratch, this.qScratch, this.sScratch);
  }

  private refreshMarkers() {
    Viewer.disposeDeep(this.markerGroup);
    this.pickables = [];
    this.labelLayer.replaceChildren();
    this.labels = [];
    this.portRows = [];
    this.gateStyles = [];
    this.instStyles = [];
    this.portMeshes = { solid: null, ghost: null };
    this.edgesRest = null;
    this.edgesSelected = null;

    if (this.showIo) {
      const sorted = ports_sorted(this.ports);
      const solid = sorted.filter((p) => p.routable);
      const ghost = sorted.filter((p) => !p.routable);
      for (const [key, list] of [["solid", solid], ["ghost", ghost]] as const) {
        if (list.length === 0) continue;
        const mat = new THREE.MeshLambertMaterial({
          transparent: key === "ghost", opacity: key === "ghost" ? 0.55 : 1,
        });
        const mesh = new THREE.InstancedMesh(Viewer.shared.cone, mat, list.length);
        mesh.userData = { type: "ports", ids: list.map((p) => p.name), shared: true };
        this.markerGroup.add(mesh);
        this.pickables.push(mesh);
        this.portMeshes[key] = mesh;
        list.forEach((p, row) => {
          // A driver points OUT of its column, a sink points IN: the arrow
          // says which way signal flows before you read the label.
          const pos = markerPos(p.anchor, p.step, p.width);
          const world = new THREE.Vector3(pos[0], pos[1] + 1.1, pos[2]);
          const cls = [p.kind === "input" ? "io-driver" : "io-sink", p.routable ? "" : "io-blocked"]
            .join(" ");
          const el = this.label(`${p.name} : ${p.ty}${p.routable ? "" : " ✗"}`, cls, world);
          this.portRows.push({ p, group: key, row, world, el, pos });
        });
      }
    }

    for (const g of this.gates) {
      const id = `${g.bus} ${g.name}`;
      const mat = new THREE.MeshLambertMaterial({ color: 0xffb74d });
      const m = new THREE.Mesh(Viewer.shared.oct, mat);
      m.position.set(...markerPos(g.anchor, g.step, g.width));
      m.userData = { type: "gate", id, y: g.anchor[1], shared: true };
      this.markerGroup.add(m);
      this.pickables.push(m);
      this.gateStyles.push({ id, mesh: m, mat });
    }

    for (const inst of this.instances) {
      const { w, h, l, center } = Viewer.instBox(inst);
      const pick = new THREE.Mesh(
        new THREE.BoxGeometry(w, h, l), new THREE.MeshBasicMaterial({ visible: false }));
      pick.position.copy(center);
      pick.userData = { type: "instance", id: inst.name, y: inst.at[1] };
      this.markerGroup.add(pick);
      this.pickables.push(pick);
      this.instStyles.push({ inst, pick });
    }
    this.rebuildEdges();
    this.restyleMarkers();
    this.rebuildGizmo();
  }

  private static instBox(inst: InstanceMarker) {
    const [w, h, l] = inst.rot % 180 === 0 ? inst.dims : [inst.dims[2], inst.dims[1], inst.dims[0]];
    return {
      w, h, l,
      center: new THREE.Vector3(
        inst.at[0] + w / 2 - 0.5, inst.at[1] + h / 2 - 0.5, inst.at[2] + l / 2 - 0.5),
    };
  }

  /** Every unselected instance's box in ONE LineSegments; the selected one in
   *  its own object, because it is the only box a drag ever moves alone. */
  private rebuildEdges() {
    for (const o of [this.edgesRest, this.edgesSelected]) {
      if (!o) continue;
      o.geometry.dispose();
      (o.material as THREE.Material).dispose();
      o.removeFromParent();
    }
    this.edgesRest = null;
    this.edgesSelected = null;
    const sel = this.selection?.kind === "instance" ? this.selection.id : null;
    const rest: number[] = [];
    for (const inst of this.instances) {
      const { w, h, l, center } = Viewer.instBox(inst);
      if (inst.name === sel) {
        const geo = new THREE.EdgesGeometry(new THREE.BoxGeometry(w, h, l));
        this.edgesSelected = new THREE.LineSegments(geo, new THREE.LineBasicMaterial({
          color: SELECT_COLOR, transparent: true, opacity: 1,
        }));
        this.edgesSelected.position.copy(center);
        this.markerGroup.add(this.edgesSelected);
        continue;
      }
      const edges = new THREE.EdgesGeometry(new THREE.BoxGeometry(w, h, l));
      const pos = edges.getAttribute("position");
      for (let i = 0; i < pos.count; i++) {
        rest.push(pos.getX(i) + center.x, pos.getY(i) + center.y, pos.getZ(i) + center.z);
      }
      edges.dispose();
    }
    if (rest.length) {
      const geo = new THREE.BufferGeometry();
      geo.setAttribute("position", new THREE.Float32BufferAttribute(rest, 3));
      this.edgesRest = new THREE.LineSegments(geo, new THREE.LineBasicMaterial({
        color: 0x4fc3f7, transparent: true, opacity: 0.5,
      }));
      this.markerGroup.add(this.edgesRest);
    }
  }

  /** Repaint the marker layer for the current selection/hover: per-instance
   *  colours and one matrix each, no rebuild and no DOM churn. */
  private restyleMarkers() {
    const color = new THREE.Color();
    for (const row of this.portRows) {
      const p = row.p;
      const selected = this.selection?.kind === "port" && this.selection.id === p.name;
      const hovered = this.hovered === p.name;
      const base = !p.routable ? BLOCKED_COLOR
        : p.kind === "input" ? DRIVER_COLOR : SINK_COLOR;
      const mesh = this.portMeshes[row.group];
      if (mesh) {
        mesh.setColorAt(row.row, color.setHex(selected ? SELECT_COLOR : hovered ? HOVER_COLOR : base));
        mesh.setMatrixAt(row.row, this.portMatrix(row, selected || hovered));
      }
      row.el.className = `mk-label ${[
        p.kind === "input" ? "io-driver" : "io-sink",
        p.routable ? "" : "io-blocked",
        selected ? "is-selected" : "",
        hovered ? "is-hovered" : "",
      ].join(" ")}`;
    }
    for (const mesh of [this.portMeshes.solid, this.portMeshes.ghost]) {
      if (!mesh) continue;
      mesh.instanceMatrix.needsUpdate = true;
      if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true;
      mesh.computeBoundingSphere();
    }
    for (const { id, mesh, mat } of this.gateStyles) {
      const selected = this.selection?.kind === "gate" && this.selection.id === id;
      mat.color.setHex(selected ? SELECT_COLOR : 0xffb74d);
      mesh.geometry = selected ? Viewer.shared.octBig : Viewer.shared.oct;
    }
  }

  /** The selected instance's shell, axis indicator and label — one instance's
   *  worth of objects, so this is cheap to rebuild on every selection change. */
  private rebuildGizmo() {
    Viewer.disposeDeep(this.gizmoGroup);
    this.gizmoGroup.position.set(0, 0, 0); // previewInstance nudges this
    this.gizmoOwner = null;
    this.gizmoLabelWorld = null;
    for (const l of this.gizmoLabels) l.remove();
    this.gizmoLabels = [];
    this.labels = this.labels.filter((l) => l.el.isConnected);
    if (this.selection?.kind !== "instance") return;
    const inst = this.instances.find((i) => i.name === this.selection!.id);
    if (!inst) return;
    this.gizmoOwner = inst.name;
    const [w, h, l] = inst.rot % 180 === 0 ? inst.dims : [inst.dims[2], inst.dims[1], inst.dims[0]];
    const center = new THREE.Vector3(
      inst.at[0] + w / 2 - 0.5, inst.at[1] + h / 2 - 0.5, inst.at[2] + l / 2 - 0.5);
    // A translucent shell so the selection reads from any angle...
    const shell = new THREE.Mesh(new THREE.BoxGeometry(w, h, l), new THREE.MeshBasicMaterial({
      color: SELECT_COLOR, transparent: true, opacity: 0.12, depthWrite: false,
    }));
    shell.position.copy(center);
    this.gizmoGroup.add(shell);
    // ...plus an axis indicator at the instance origin showing which way
    // its local +X/+Z point after `rot`, so R reads as a real rotation.
    const rad = (inst.rot * Math.PI) / 180;
    const origin = new THREE.Vector3(inst.at[0] - 0.5, inst.at[1] - 0.5, inst.at[2] - 0.5);
    const axes: [Vec3, number][] = [
      [[Math.cos(rad), 0, -Math.sin(rad)], 0xff5252],
      [[0, 1, 0], 0x7bd88f],
      [[Math.sin(rad), 0, Math.cos(rad)], 0x4fc3f7],
    ];
    for (const [dir, color] of axes) {
      this.gizmoGroup.add(new THREE.ArrowHelper(
        new THREE.Vector3(...dir).normalize(), origin,
        Math.max(3, Math.min(w, l) * 0.6), color, 1.0, 0.5));
    }
    this.gizmoLabelWorld = new THREE.Vector3(center.x, inst.at[1] + h + 0.8, center.z);
    this.gizmoLabels.push(this.label(
      `${inst.name} · ${inst.cell} · rot ${inst.rot}°`, "inst-label is-selected",
      this.gizmoLabelWorld));
  }

  private gizmoLabels: HTMLDivElement[] = [];

  /** Project label anchors to screen space once per frame. */
  private labelScratch = new THREE.Vector3();
  private projectLabels() {
    if (this.labels.length === 0) return;
    const rect = this.renderer.domElement.getBoundingClientRect();
    const v = this.labelScratch;
    for (const { el, world } of this.labels) {
      v.copy(world).project(this.camera);
      if (v.z > 1) { el.style.display = "none"; continue; }
      el.style.display = "";
      el.style.left = `${((v.x + 1) / 2) * rect.width}px`;
      el.style.top = `${((-v.y + 1) / 2) * rect.height}px`;
    }
  }

  // -- picking / drag -------------------------------------------------------

  private ndc(e: PointerEvent): THREE.Vector2 {
    const rect = this.renderer.domElement.getBoundingClientRect();
    return new THREE.Vector2(
      ((e.clientX - rect.left) / rect.width) * 2 - 1,
      -((e.clientY - rect.top) / rect.height) * 2 + 1,
    );
  }

  private castPointer(e: PointerEvent): THREE.Intersection | null {
    this.raycaster.setFromCamera(this.ndc(e), this.camera);
    return this.raycaster.intersectObjects(this.pickables, false)[0] ?? null;
  }

  /** What a hit means. Ports live in an InstancedMesh, so the port's identity
   *  is the `instanceId`, not the object. */
  private static pickInfo(hit: THREE.Intersection | null):
    { type: string; id: string; y?: number } | null {
    if (!hit) return null;
    const ud = hit.object.userData as { type?: string; id?: string; y?: number; ids?: string[] };
    if (ud.type === "ports") {
      const id = ud.ids?.[hit.instanceId ?? -1];
      return id ? { type: "port", id } : null;
    }
    return ud.type ? { type: ud.type, id: ud.id ?? "", y: ud.y } : null;
  }

  private groundPoint(e: PointerEvent, y: number): Vec3 | null {
    this.raycaster.setFromCamera(this.ndc(e), this.camera);
    this.ground.constant = -y;
    const hit = new THREE.Vector3();
    if (!this.raycaster.ray.intersectPlane(this.ground, hit)) return null;
    return [Math.round(hit.x), y, Math.round(hit.z)];
  }

  /** World point on the y-plane under the cursor (for the ghost line). */
  worldPoint(e: PointerEvent, y: number): THREE.Vector3 | null {
    this.raycaster.setFromCamera(this.ndc(e), this.camera);
    this.ground.constant = -y;
    const hit = new THREE.Vector3();
    return this.raycaster.ray.intersectPlane(this.ground, hit) ? hit : null;
  }

  /** The marker position of a port, for anchoring a ghost line. */
  portMarkerPos(name: string): Vec3 | null {
    const p = this.ports.find((q) => q.name === name);
    return p ? markerPos(p.anchor, p.step, p.width) : null;
  }

  private downAt: [number, number] | null = null;

  private pointerDown(e: PointerEvent) {
    const info = Viewer.pickInfo(this.castPointer(e));
    if (!info) {
      this.downAt = [e.clientX, e.clientY];
      return;
    }
    const { type, id, y } = info;
    if (type === "port") {
      this.cb.onPortClick(id);
      return;
    }
    if (type === "instance") {
      this.cb.onInstanceClick(id);
      this.drag = { kind: "instance", id, y: y ?? 0 };
      this.controls.enabled = false;
      this.renderer.domElement.setPointerCapture(e.pointerId);
      return;
    }
    if (type === "gate") {
      this.drag = { kind: "gate", id, y: y ?? 0 };
      this.controls.enabled = false;
      this.renderer.domElement.setPointerCapture(e.pointerId);
    }
  }

  private pointerMove(e: PointerEvent) {
    if (this.drag) {
      const p = this.groundPoint(e, this.drag.y);
      if (p) this.cb.onDragMove(this.drag.kind, this.drag.id, p);
      return;
    }
    if (this.ghost) {
      const w = this.worldPoint(e, this.ghost.from[1]);
      if (w) { this.ghost.to = w; this.refreshGhost(); }
    }
    const hit = this.castPointer(e);
    const data = Viewer.pickInfo(hit);
    const name = data?.type === "port" ? data.id : null;
    if (name !== this.hovered) {
      this.setHovered(name);
      this.cb.onPortHover(name);
    }
    if (this.cameraLock == null) {
      this.renderer.domElement.style.cursor = hit ? "pointer" : "";
    }
  }

  private pointerUp(e: PointerEvent) {
    if (this.downAt) {
      const [dx, dy] = [e.clientX - this.downAt[0], e.clientY - this.downAt[1]];
      this.downAt = null;
      if (Math.hypot(dx, dy) < 4) {
        const p = this.groundPoint(e, 0);
        if (p) this.cb.onGroundClick(p);
      }
    }
    if (!this.drag) return;
    const drag = this.drag;
    this.drag = null;
    this.controls.enabled = this.cameraLock == null;
    const p = this.groundPoint(e, drag.y);
    if (p) this.cb.onDragEnd(drag.kind, drag.id, p);
  }

  // ---- mesh-build accounting (dev diagnostic + a verify assertion) -------
  //
  // The user's complaint was "it remeshes as I drag things around". It should
  // not: an instance is a cell REFERENCE plus a transform, so a drag is a
  // matrix write. These counters make that claim checkable rather than
  // aspirational — `npm run verify` asserts dragging adds 0 texture builds.
  //
  // `cells` is the number the instancing claim rests on: K placements of one
  // cell must add exactly ONE. `matrixWrites` is what a drag frame costs
  // instead.
  meshBuilds = {
    texture: 0, cells: 0, instancedGroups: 0, instanceAllocs: 0,
    matrixWrites: 0, buses: 0, loose: 0,
  };
  /** Cells whose blocks changed since their mesh was built. */
  private staleCells = new Set<string>();

  /** A cell's blocks changed (a port-mode toggle removed or restored a lever),
   *  so its mesh is stale. Nothing else in the scene is. */
  invalidateCell(cell: string) {
    if (cell) this.staleCells.add(cell);
  }

  /** Is a pointer drag in progress? */
  isDragging(): boolean {
    return this.drag != null;
  }

  /** Cells awaiting a re-mesh. */
  stale(): string[] {
    return [...this.staleCells];
  }

  clearStale() {
    this.staleCells.clear();
  }

  /** Lock (reason) or unlock (`null`) the camera. Idempotent. */
  setCameraLock(reason: string | null) {
    this.cameraLock = reason;
    this.controls.enabled = reason == null && this.drag == null;
    const el = this.renderer.domElement;
    el.style.cursor = reason ? "crosshair" : "";
    el.classList.toggle("camera-locked", reason != null);
  }

  /** For the headless verify: is the camera free to move? */
  get cameraFree(): boolean {
    return this.controls.enabled;
  }

  /** Why the camera is locked, if it is. */
  get cameraLockReason(): string | null {
    return this.cameraLock;
  }

  screenshotDataUrl(): string {
    this.renderer.render(this.scene, this.camera);
    return this.renderer.domElement.toDataURL("image/png");
  }
}

/** Routable ports first: the ones you can actually click to wire. */
function ports_sorted(ports: PortMarker[]): PortMarker[] {
  return [...ports].sort((a, b) => Number(b.routable) - Number(a.routable));
}
