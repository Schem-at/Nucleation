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

export type Selection = { kind: "instance" | "port" | "gate" | "bus"; id: string } | null;

/** How the abstract bus layers draw over the geometry they thread through.
 *
 *  A bus fragment IS real redstone — dust, repeaters, the blocks under them —
 *  so an opaque coloured slab over it hides the thing the user is trying to
 *  read. `solid` is the high-contrast tracing view, `translucent` lets the
 *  blocks (or a resource pack's textures) show through the colour, and
 *  `outline` draws only the fragment's silhouette in the bus colour so the
 *  redstone is completely unobscured and still identifiable. */
export type BusStyle = "solid" | "translucent" | "outline";

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

/** What the pointer is over: the same identity the click handlers use, plus
 *  the world position under the cursor so a context action can act THERE
 *  ("add a gate here" is only useful with the here). */
export interface PickTarget {
  kind: "instance" | "port" | "gate" | "bus" | "ground";
  id: string;
  /** The hit point, or the ground plane point when nothing was hit. */
  at: Vec3;
  /** Client coordinates, for placing a menu. */
  screen: [number, number];
  /** Additive click (shift/ctrl held): extend the selection. */
  additive: boolean;
}

/** One resolved pick, before it is turned into a gesture. */
interface Probe {
  type: "instance" | "port" | "gate" | "bus";
  id: string;
  /** The drag plane's height, for instances and gates. */
  y?: number;
  /** Where on the thing the pointer landed. */
  point: THREE.Vector3 | null;
}

export interface ViewerCallbacks {
  onPortClick(name: string): void;
  onPortHover(name: string | null): void;
  onInstanceClick(name: string, additive: boolean): void;
  /** A click on a bus's own geometry — buses are selectable objects, not just
   *  outliner rows. */
  onBusClick(name: string, additive: boolean): void;
  /** A click on a gate handle (`"<bus> <gate>"`), before its drag begins. */
  onGateClick(id: string): void;
  onDragMove(kind: "instance" | "gate", id: string, ground: Vec3): void;
  onDragEnd(kind: "instance" | "gate", id: string, ground: Vec3): void;
  /** A plain click on empty ground (cell-placement mode / deselect). */
  onGroundClick(ground: Vec3): void;
  /** Right-click, with everything an action needs to be context-sensitive. */
  onContextMenu(target: PickTarget): void;
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

/** The 12 edges of one cube, as vertex pairs. */
const EDGE_PAIRS: [number, number, number][][] = (() => {
  const c: [number, number, number][] = [
    [-1, -1, -1], [1, -1, -1], [1, -1, 1], [-1, -1, 1],
    [-1, 1, -1], [1, 1, -1], [1, 1, 1], [-1, 1, 1],
  ];
  const pairs: [number, number][] = [
    [0, 1], [1, 2], [2, 3], [3, 0], [4, 5], [5, 6], [6, 7], [7, 4],
    [0, 4], [1, 5], [2, 6], [3, 7],
  ];
  return pairs.map(([a, b]) => [c[a], c[b]]);
})();

/** Outline geometry for a block list: every cube's silhouette, one buffer.
 *
 *  Interior edges are not culled — for a 200-cell bus fragment the difference
 *  is invisible and the cull is a neighbour query per cell. This is built once
 *  per bus per style change, not per frame. */
function cubeEdges(blocks: Block[]): THREE.BufferGeometry {
  const solid = blocks.filter((b) => b.name !== "minecraft:air");
  const pos = new Float32Array(solid.length * EDGE_PAIRS.length * 2 * 3);
  let i = 0;
  for (const b of solid) {
    for (const [a, c] of EDGE_PAIRS) {
      pos[i++] = b.x + a[0] * CUBE; pos[i++] = b.y + a[1] * CUBE; pos[i++] = b.z + a[2] * CUBE;
      pos[i++] = b.x + c[0] * CUBE; pos[i++] = b.y + c[1] * CUBE; pos[i++] = b.z + c[2] * CUBE;
    }
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(pos, 3));
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

const EMPTY_SET: Set<string> = new Set();
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
  /** A body press that has NOT yet moved far enough to be a move.
   *
   *  The user's report was "I always accidentally move the component": a press
   *  used to become a drag on the first pointermove, so a 1-pixel jitter during
   *  a click nudged the instance and cost an undo. A press now parks HERE, and
   *  only crossing `DRAG_PX` promotes it to `drag`. Below the threshold the
   *  gesture stays what it looks like: a selection. */
  private pending:
    { kind: "instance" | "gate"; id: string; y: number; x: number; py: number } | null = null;
  private cb: ViewerCallbacks;
  private pickables: THREE.Object3D[] = [];
  private labelLayer: HTMLDivElement;
  /** `{el, world, prio}` for the projected label pass. `prio` breaks
   *  declutter ties: a selected/hovered label always survives. */
  private labels: { el: HTMLDivElement; world: THREE.Vector3; prio: number }[] = [];
  private ports: PortMarker[] = [];
  private instances: InstanceMarker[] = [];
  private selection: Selection = null;
  private hovered: string | null = null;
  private showIo = true;
  private textured = false;
  private ghost: { from: Vec3; to: THREE.Vector3 } | null = null;
  /** Connect mode: component BODIES are not pickable at all, so in a dense
   *  scene every press is a routing press. Held (Alt) or latched (C). */
  private connect = false;

  // ---- how a press is resolved -------------------------------------------
  //
  // Two numbers, both of them the answer to a bug report.

  /** A port's pick radius in CSS PIXELS. Ports are picked in SCREEN SPACE, not
   *  by ray-vs-cone: a 0.45-block cone is a 4-pixel target when the labels
   *  start to declutter, and "click the little arrow" cannot be the gesture the
   *  whole routing model rests on. 15 px is ~2.5x the cone's own radius at a
   *  comfortable working zoom and stays clickable when it is far smaller. */
  static PORT_PICK_PX = 15;
  /** How far a press on a BODY has to travel before it is a move rather than a
   *  click. Below this the instance does not move at all. */
  static DRAG_PX = 4;

  /** How presses were resolved, so the interaction contract is checkable and
   *  not just claimed (`npm run verify` asserts every one of these). */
  pickStats = {
    /** Presses that resolved to a port. */
    portPicks: 0,
    /** Presses that resolved to an instance body. */
    bodyPicks: 0,
    /** Ports that won a press the body geometry was in front of. */
    portOverBody: 0,
    /** Body presses promoted to a real move. */
    dragsStarted: 0,
    /** Body presses that stayed a selection (< DRAG_PX). */
    clicksBelowThreshold: 0,
    /** Bodies not offered to the raycast because connect mode was on. */
    bodiesSkippedInConnectMode: 0,
  };

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
    el.addEventListener("pointerdown", (e) => {
      // The right button is the context menu's, never a drag's.
      if (e.button === 2) return;
      this.pointerDown(e);
    });
    el.addEventListener("pointermove", (e) => this.pointerMove(e));
    el.addEventListener("pointerup", (e) => this.pointerUp(e));
    // A cancelled pointer (a gesture the browser took over) must not leave a
    // press parked: the camera would stay locked and the next move would
    // "resume" a drag the user never made.
    el.addEventListener("pointercancel", () => {
      this.pending = null;
      this.drag = null;
      this.controls.enabled = this.cameraLock == null;
    });
    el.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      this.cb.onContextMenu(this.pickAt(e as unknown as PointerEvent));
    });

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
      labelStats: { ...this.labelStats },
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
    /** The variant revision this mesh was built from. */
    rev: number;
    /** Cell-local blocks it was built from, for the consistency check. */
    blocks: Block[];
  }>();
  /** Unique (never instanced) layers: one per bus, one for the loose base. */
  private uniqueMeshes = new Map<string, {
    objects: THREE.Object3D[];
    /** The layer revision, failed flag and style this build reflects — the
     *  cache key. Any difference means rebuild, whatever `dirty` claimed. */
    rev: number;
    failed: boolean;
    style: string;
    blocks: Block[];
  }>();
  private cellGroup = new THREE.Group();
  private looseGroup = new THREE.Group();
  private busGroup = new THREE.Group();
  private lastScene: SceneModel | null = null;
  private scratch = new THREE.Matrix4();

  /** Update the scene from the document model, rebuilding ONLY what
   *  `model.dirty` names. A drag reports `placements` alone, so it writes one
   *  matrix per moved instance and touches no geometry at all. */
  /** `setScene` is REVISION-DRIVEN, not dirty-list-driven.
   *
   *  The perf pass made the document report a changed-set so a drag frame
   *  re-reads almost nothing, and that is worth keeping — but a renderer that
   *  believes the changed-set is complete renders a stale mesh the moment it
   *  is not. So every cached mesh carries the revision it was built from, and
   *  a mismatch rebuilds regardless of what `dirty` said. `dirty` still
   *  decides what gets RE-READ out of wasm (the expensive half); `rev` decides
   *  what gets RE-MESHED (the correct half). The revisions only move on a real
   *  content change, so this adds an integer compare per layer per frame and
   *  no rebuilds. */
  setScene(model: SceneModel) {
    this.lastScene = model;
    let rebuilt = 0;
    // The two triggers cover each other's failure mode, which is why both are
    // consulted rather than one replacing the other:
    //   * `dirty` can be INCOMPLETE (a changed-set that forgot a layer) — the
    //     revision catches that;
    //   * a revision can be UNFAMILIAR rather than newer (a fresh document, a
    //     restored file) — `dirty` catches that.
    const dirtyVariants = new Set(model.dirty.variants);
    const dirtyBuses = new Set(model.dirty.buses.map((n) => `bus:${n}`));

    // (a) cells: mesh each distinct variant once, then place it by matrix.
    for (const [key, variant] of model.variants) {
      const have = this.cellMeshes.get(key);
      if (have && have.rev === variant.rev && !dirtyVariants.has(key)) continue;
      if (have) this.disposeCell(key);
      rebuilt++;
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
        rev: variant.rev, blocks: variant.blocks,
      });
    }
    for (const key of [...this.cellMeshes.keys()]) {
      if (!model.variants.has(key)) this.disposeCell(key);
    }

    // (b) placements: matrices only.
    let t = performance.now();
    if (model.dirty.placements || rebuilt) this.writePlacements(model);
    this.sceneMs.placements += performance.now() - t;
    t = performance.now();

    // (c) buses + the loose base: unique geometry, rebuilt when their revision,
    // their FAILED flag or the bus style they draw in changed. In the textured
    // view the cells and the base hide (the pack draws them) but the buses stay
    // on top: the routing is what the colours encode, and a resource pack
    // cannot show it — which is exactly why the style has to be dialable.
    for (const layer of model.buses) {
      const built = this.uniqueMeshes.get(layer.layer);
      if (!built
        || built.rev !== layer.rev
        || built.failed !== layer.failed
        || built.style !== this.styleKey(layer.layer)
        || dirtyBuses.has(layer.layer)) {
        this.rebuildUnique(model, layer.layer);
      }
    }
    for (const name of [...this.uniqueMeshes.keys()]) {
      if (name === "loose") continue;
      if (!model.buses.some((b) => b.layer === name)) this.disposeUnique(name);
    }
    const loose = this.uniqueMeshes.get("loose");
    if (!loose || loose.rev !== model.loose.rev || model.dirty.loose) {
      this.rebuildUnique(model, "loose");
    }
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

  // ---- bus presentation ----------------------------------------------------
  //
  // "The bus lines are too obnoxious — they hide the underlying redstone."
  // They did, and the reason is worth stating: a bus fragment is not an
  // annotation drawn NEAR redstone, it IS redstone (dust, repeaters, the blocks
  // carrying them). Painting it as an opaque slab in the bus colour therefore
  // deletes information rather than adding it. What the colour is actually for
  // is IDENTITY — which of the eight buses this cell belongs to — and identity
  // survives a tint or an outline just as well as a fill.

  private busStyle: BusStyle = "translucent";
  /** Per-bus overrides, so one bus can be traced solid while the rest recede. */
  private busStyles = new Map<string, BusStyle>();

  setBusStyle(style: BusStyle) {
    if (this.busStyle === style) return;
    this.busStyle = style;
    this.restyleBuses();
  }

  busStyleOf(bus: string): BusStyle {
    return this.busStyles.get(bus) ?? this.busStyle;
  }

  globalBusStyle(): BusStyle {
    return this.busStyle;
  }

  setBusStyleFor(bus: string, style: BusStyle | null) {
    if (style) this.busStyles.set(bus, style); else this.busStyles.delete(bus);
    this.restyleBuses();
  }

  /** Re-mesh the bus layers for a presentation change (nothing else moves). */
  private restyleBuses() {
    if (!this.lastScene) return;
    for (const layer of this.lastScene.buses) this.rebuildUnique(this.lastScene, layer.layer);
  }

  /** The cache key for a layer's PRESENTATION: any change here is a rebuild,
   *  exactly like a geometry revision. */
  private styleKey(name: string): string {
    if (!name.startsWith("bus:")) return this.textured ? "t" : "a";
    const bus = name.slice(4);
    const emph = (this.selection?.kind === "bus" && this.selection.id === bus)
      || this.hoveredBus === bus;
    return `${this.busStyleOf(bus)}|${this.textured ? "t" : "a"}|${emph ? "e" : ""}`;
  }

  private rebuildUnique(model: SceneModel, name: string) {
    this.disposeUnique(name);
    const layer = this.layerOf(model, name);
    const style = this.styleKey(name);
    if (!layer) return;
    if (layer.blocks.length === 0) {
      // Still RECORD the build: an empty layer is a legitimate state (a ripped
      // or failed bus), and forgetting it means re-deciding every frame.
      this.uniqueMeshes.set(name, { objects: [], rev: layer.rev, failed: layer.failed, style, blocks: [] });
      return;
    }
    const isBus = name.startsWith("bus:");
    this.meshBuilds[isBus ? "buses" : "loose"]++;
    const override = layer.failed ? FAILED_COLOR : layer.color;
    const objects: THREE.Object3D[] = [];
    const bus = isBus ? name.slice(4) : null;
    const emph = !!bus && ((this.selection?.kind === "bus" && this.selection.id === bus)
      || this.hoveredBus === bus);
    const mode: BusStyle = isBus ? this.busStyleOf(bus!) : "solid";

    // Fill. `outline` keeps a mesh with COLOUR WRITES OFF rather than dropping
    // it: an invisible-but-present body is what keeps the bus clickable (a
    // raycast ignores `visible: false`, and a 1-pixel line is not a hit target).
    const showFill = mode !== "outline" || !this.textured || emph;
    const opacity = layer.failed ? 0.85
      : !isBus ? 1
      : mode === "solid" ? (this.textured ? 0.75 : 1)
      : mode === "translucent" ? (emph ? 0.7 : this.textured ? 0.34 : 0.62)
      : emph ? 0.3 : 0.12;
    // Buses are LIT, cells are not. Cell bodies are diffuse-only voxels in
    // muted block colours; a routed bus is the signal, and half the palette
    // (the teal, the mid-green) sat right on top of the stone/quartz greys it
    // threads through. A dim emissive term on the bus layers alone separates
    // "wiring" from "structure" by brightness as well as by hue, and it is the
    // same trick that keeps them readable over a resource-pack mesh.
    const emissive = new THREE.Color(override ?? 0xffffff)
      .multiplyScalar(layer.failed ? 0.5 : isBus ? (emph ? 0.6 : 0.42) : 0);
    const mat = new THREE.MeshLambertMaterial({
      vertexColors: true,
      emissive: isBus || layer.failed ? emissive : new THREE.Color(0x000000),
      transparent: opacity < 1,
      opacity,
      depthWrite: opacity >= 1,
      colorWrite: showFill,
    });
    const mesh = new THREE.Mesh(mergeCubes(layer.blocks, override), mat);
    mesh.userData = { type: isBus ? "bus" : "loose", id: bus ?? "loose", failed: layer.failed };
    (isBus ? this.busGroup : this.looseGroup).add(mesh);
    objects.push(mesh);

    // Silhouette. One LineSegments over the fragment's cube edges: it names the
    // bus without covering a single block face, which is the whole point.
    if (isBus && (mode === "outline" || (mode === "translucent" && emph))) {
      const line = new THREE.LineSegments(
        cubeEdges(layer.blocks),
        new THREE.LineBasicMaterial({
          color: override ?? 0xffffff,
          transparent: true,
          opacity: emph ? 1 : 0.85,
          depthTest: !emph,
        }));
      line.renderOrder = emph ? 9 : 1;
      this.busGroup.add(line);
      objects.push(line);
    }
    this.uniqueMeshes.set(name, { objects, rev: layer.rev, failed: layer.failed, style, blocks: layer.blocks });
  }

  private disposeUnique(name: string) {
    for (const m of this.uniqueMeshes.get(name)?.objects ?? []) {
      const g = m as Partial<THREE.Mesh>;
      g.geometry?.dispose();
      const mats = Array.isArray(g.material) ? g.material : g.material ? [g.material] : [];
      for (const mm of mats) (mm as THREE.Material).dispose();
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
    const wasBus = this.selection?.kind === "bus" ? this.selection.id : null;
    const isBus = sel?.kind === "bus" ? sel.id : null;
    this.selection = sel;
    this.restyleMarkers();
    // Which box is highlighted moved between the merged batch and its own
    // object, so the two line meshes are re-split (a dozen boxes: microseconds).
    if (wasInst !== isInst) this.rebuildEdges();
    // A selected bus draws with more emphasis, so its layer is re-meshed —
    // only the one that gained or lost the selection.
    if (wasBus !== isBus) {
      if (this.lastScene) {
        for (const b of [wasBus, isBus]) {
          if (b) this.rebuildUnique(this.lastScene, `bus:${b}`);
        }
      }
    }
    this.rebuildGizmo();
  }

  /** Instances highlighted alongside the primary selection (an area select). */
  private multi: string[] = [];

  setMultiSelection(names: string[]) {
    const same = names.length === this.multi.length && names.every((n, i) => this.multi[i] === n);
    if (same) return;
    this.multi = [...names];
    this.rebuildEdges();
  }

  setHovered(name: string | null) {
    if (this.hovered === name) return;
    this.hovered = name;
    this.restyleMarkers();
  }

  /** The bus under the cursor. Raising a hovered bus's emphasis is what keeps
   *  `outline` mode traceable: the fragment you are pointing at comes forward
   *  without the other seven coming with it. */
  private hoveredBus: string | null = null;

  setHoveredBus(bus: string | null) {
    if (this.hoveredBus === bus) return;
    const was = this.hoveredBus;
    this.hoveredBus = bus;
    if (!this.lastScene) return;
    for (const b of [was, bus]) if (b) this.rebuildUnique(this.lastScene, `bus:${b}`);
  }

  /** Which ports each bus terminates on, so selecting a bus can light up its
   *  two ends — the question "where does this thing go?" answered on the
   *  canvas instead of in a panel. */
  private busEnds = new Map<string, string[]>();

  setBusEndpoints(map: Map<string, string[]>) {
    this.busEnds = map;
    this.restyleMarkers();
  }

  /** Endpoints of the selected bus (empty when no bus is selected). */
  private selectedBusPorts(): Set<string> {
    if (this.selection?.kind !== "bus") return EMPTY_SET;
    return new Set(this.busEnds.get(this.selection.id) ?? []);
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

  private label(text: string, cls: string, world: THREE.Vector3, prio = 0) {
    const el = document.createElement("div");
    el.className = `mk-label ${cls}`;
    el.textContent = text;
    this.labelLayer.appendChild(el);
    const entry = { el, world, prio };
    this.labels.push(entry);
    return entry;
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
      /** The ring that tells a gate apart from a port at a glance. */
      ring: mk(new THREE.TorusGeometry(0.85, 0.07, 6, 20)),
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
    /** The projected-label entry, so selection/hover can raise its priority
     *  above the declutter rules without a lookup. */
    lab: { el: HTMLDivElement; world: THREE.Vector3; prio: number };
  }[] = [];
  private portMeshes: { solid: THREE.InstancedMesh | null; ghost: THREE.InstancedMesh | null } =
    { solid: null, ghost: null };
  private gateStyles: {
    id: string; mesh: THREE.Mesh; mat: THREE.MeshLambertMaterial;
    ring: THREE.Mesh; lab: { el: HTMLDivElement; world: THREE.Vector3; prio: number };
  }[] = [];
  private instStyles: { inst: InstanceMarker; pick: THREE.Mesh }[] = [];
  private edgesRest: THREE.LineSegments | null = null;
  private edgesSelected: THREE.LineSegments | null = null;
  /** Boxes of the OTHER instances in an area selection. */
  private edgesMulti: THREE.LineSegments | null = null;
  private gizmoOwner: string | null = null;
  private gizmoLabelWorld: THREE.Vector3 | null = null;
  private mScratch = new THREE.Matrix4();
  private qScratch = new THREE.Quaternion();
  private vScratch = new THREE.Vector3();
  private vScratch2 = new THREE.Vector3();
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
    this.edgesMulti = null;

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
        // NOT in `pickables`: ports are picked in screen space (`pickPort`), so
        // the cone geometry is for the eye only. Leaving it in the ray pass
        // would put ports back under the depth test the priority rule exists to
        // escape.
        this.portMeshes[key] = mesh;
        list.forEach((p, row) => {
          // A driver points OUT of its column, a sink points IN: the arrow
          // says which way signal flows before you read the label.
          const pos = markerPos(p.anchor, p.step, p.width);
          const world = new THREE.Vector3(pos[0], pos[1] + 1.1, pos[2]);
          const cls = [p.kind === "input" ? "io-driver" : "io-sink", p.routable ? "" : "io-blocked"]
            .join(" ");
          const lab = this.label(`${p.name} : ${p.ty}${p.routable ? "" : " ✗"}`, cls, world);
          this.portRows.push({ p, group: key, row, world, el: lab.el, pos, lab });
        });
      }
    }

    // GATES read as checkpoints, not as ports: an orange diamond wearing a RING,
    // labelled with its position in the run ("gate 1 of 2"). A port is a place
    // signal enters or leaves; a gate is a place the route must pass through,
    // and the two must never be mistaken for each other — deleting one changes
    // the netlist, deleting the other only straightens the path.
    const perBus = new Map<string, number>();
    for (const g of this.gates) perBus.set(g.bus, (perBus.get(g.bus) ?? 0) + 1);
    const seen = new Map<string, number>();
    for (const g of this.gates) {
      const id = `${g.bus} ${g.name}`;
      const ord = (seen.get(g.bus) ?? 0) + 1;
      seen.set(g.bus, ord);
      const mat = new THREE.MeshLambertMaterial({ color: 0xffb74d });
      const m = new THREE.Mesh(Viewer.shared.oct, mat);
      const pos = markerPos(g.anchor, g.step, g.width);
      m.position.set(...pos);
      m.userData = { type: "gate", id, y: g.anchor[1], shared: true };
      const ring = new THREE.Mesh(Viewer.shared.ring, new THREE.MeshBasicMaterial({
        color: 0xffe082, transparent: true, opacity: 0.9, side: THREE.DoubleSide,
      }));
      ring.userData.shared = true;
      ring.rotation.x = Math.PI / 2;
      m.add(ring);
      this.markerGroup.add(m);
      this.pickables.push(m);
      const lab = this.label(
        `◆ gate ${ord} of ${perBus.get(g.bus)} · ${g.bus}`, "gate-label",
        new THREE.Vector3(pos[0], pos[1] + 1.0, pos[2]));
      this.gateStyles.push({ id, mesh: m, mat, ring, lab });
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
    for (const o of [this.edgesRest, this.edgesSelected, this.edgesMulti]) {
      if (!o) continue;
      // The selected box carries a dark backing child; dispose both.
      o.traverse((c) => {
        const m = c as Partial<THREE.LineSegments>;
        m.geometry?.dispose();
        (m.material as THREE.Material | undefined)?.dispose();
      });
      o.removeFromParent();
    }
    this.edgesRest = null;
    this.edgesSelected = null;
    this.edgesMulti = null;
    const sel = this.selection?.kind === "instance" ? this.selection.id : null;
    const alsoSelected = new Set(this.multi.filter((n) => n !== sel));
    const rest: number[] = [];
    const extra: number[] = [];
    for (const inst of this.instances) {
      const { w, h, l, center } = Viewer.instBox(inst);
      if (alsoSelected.has(inst.name)) {
        // Everything else in an area selection: same yellow, batched, because
        // none of them is the one a drag moves alone.
        const edges = new THREE.EdgesGeometry(new THREE.BoxGeometry(w, h, l));
        const pos = edges.getAttribute("position");
        for (let i = 0; i < pos.count; i++) {
          extra.push(pos.getX(i) + center.x, pos.getY(i) + center.y, pos.getZ(i) + center.z);
        }
        edges.dispose();
        continue;
      }
      if (inst.name === sel) {
        const geo = new THREE.EdgesGeometry(new THREE.BoxGeometry(w, h, l));
        this.edgesSelected = new THREE.LineSegments(geo, new THREE.LineBasicMaterial({
          color: SELECT_COLOR, transparent: true, opacity: 1, depthTest: false,
        }));
        this.edgesSelected.position.copy(center);
        this.edgesSelected.renderOrder = 11;
        // A dark backing box just outside the yellow one. Yellow-on-yellow is
        // the one case a bright resource pack can hide the selection in, and
        // one extra LineSegments on ONE instance is not a cost worth arguing
        // about.
        const back = new THREE.LineSegments(
          new THREE.EdgesGeometry(new THREE.BoxGeometry(w + 0.22, h + 0.22, l + 0.22)),
          new THREE.LineBasicMaterial({ color: 0x0b0d11, transparent: true, opacity: 0.9, depthTest: false }));
        back.position.copy(center);
        back.renderOrder = 10;
        this.edgesSelected.add(back);
        back.position.set(0, 0, 0);
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
    if (extra.length) {
      const geo = new THREE.BufferGeometry();
      geo.setAttribute("position", new THREE.Float32BufferAttribute(extra, 3));
      this.edgesMulti = new THREE.LineSegments(geo, new THREE.LineBasicMaterial({
        color: SELECT_COLOR, transparent: true, opacity: 0.9, depthTest: false,
      }));
      this.edgesMulti.renderOrder = 11;
      this.markerGroup.add(this.edgesMulti);
    }
  }

  /** Repaint the marker layer for the current selection/hover: per-instance
   *  colours and one matrix each, no rebuild and no DOM churn. */
  private restyleMarkers() {
    const color = new THREE.Color();
    const busPorts = this.selectedBusPorts();
    for (const row of this.portRows) {
      const p = row.p;
      const selected = this.selection?.kind === "port" && this.selection.id === p.name;
      const hovered = this.hovered === p.name;
      // An endpoint of the selected bus is emphasised too: selecting a bus
      // should say WHERE IT GOES without reading a panel.
      const onBus = busPorts.has(p.name);
      const base = !p.routable ? BLOCKED_COLOR
        : p.kind === "input" ? DRIVER_COLOR : SINK_COLOR;
      const mesh = this.portMeshes[row.group];
      if (mesh) {
        mesh.setColorAt(row.row, color.setHex(
          selected ? SELECT_COLOR : hovered ? HOVER_COLOR : onBus ? SELECT_COLOR : base));
        mesh.setMatrixAt(row.row, this.portMatrix(row, selected || hovered || onBus));
      }
      row.el.className = `mk-label ${[
        p.kind === "input" ? "io-driver" : "io-sink",
        p.routable ? "" : "io-blocked",
        selected || onBus ? "is-selected" : "",
        hovered ? "is-hovered" : "",
      ].join(" ")}`;
      // What you are pointing at is never decluttered away.
      row.lab.prio = selected || hovered || onBus ? 2 : 0;
    }
    for (const mesh of [this.portMeshes.solid, this.portMeshes.ghost]) {
      if (!mesh) continue;
      mesh.instanceMatrix.needsUpdate = true;
      if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true;
      mesh.computeBoundingSphere();
    }
    // A gate belonging to the selected BUS lights up too: "what does this bus
    // have to pass through?" is the question a selected bus should answer.
    const selBus = this.selection?.kind === "bus" ? this.selection.id : null;
    for (const { id, mesh, mat, ring, lab } of this.gateStyles) {
      const selected = this.selection?.kind === "gate" && this.selection.id === id;
      const onBus = !!selBus && id.startsWith(`${selBus} `);
      mat.color.setHex(selected || onBus ? SELECT_COLOR : 0xffb74d);
      mesh.geometry = selected ? Viewer.shared.octBig : Viewer.shared.oct;
      (ring.material as THREE.MeshBasicMaterial).opacity = selected ? 1 : onBus ? 0.95 : 0.55;
      ring.scale.setScalar(selected ? 1.2 : 1);
      lab.prio = selected || onBus ? 2 : 0;
      lab.el.className = `mk-label gate-label${selected ? " is-selected" : ""}`;
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
      const arrow = new THREE.ArrowHelper(
        new THREE.Vector3(...dir).normalize(), origin,
        Math.max(3, Math.min(w, l) * 0.6), color, 1.0, 0.5);
      // A gizmo that disappears inside the cell it is describing is not a
      // gizmo. Drawing it last, without depth testing, is what makes it
      // readable against a dark abstract body AND against a bright
      // resource-pack block.
      Viewer.alwaysOnTop(arrow, 10);
      this.gizmoGroup.add(arrow);
    }
    this.gizmoLabelWorld = new THREE.Vector3(center.x, inst.at[1] + h + 0.8, center.z);
    this.gizmoLabels.push(this.label(
      `${inst.name} · ${inst.cell} · rot ${inst.rot}°`, "inst-label is-selected",
      this.gizmoLabelWorld, 2).el);
  }

  /** Draw an overlay object over everything, whatever its depth. */
  private static alwaysOnTop(obj: THREE.Object3D, order: number) {
    obj.renderOrder = order;
    obj.traverse((o) => {
      o.renderOrder = order;
      const mats = (o as THREE.Mesh).material;
      for (const m of Array.isArray(mats) ? mats : mats ? [mats] : []) {
        (m as THREE.Material).depthTest = false;
        (m as THREE.Material).transparent = true;
      }
    });
  }

  private gizmoLabels: HTMLDivElement[] = [];

  // ---- label legibility ---------------------------------------------------
  //
  // A 12-instance design carries ~50 port labels. Zoomed in they are the whole
  // point; zoomed out they are a grey mat over the geometry, and they hide the
  // thing they are labelling. Three rules, all keyed off ONE measurement — how
  // many screen pixels a block spans at that label's depth:
  //
  //   * below MIN_PX_PER_BLOCK the label is dropped entirely. Its 3-D cone
  //     marker stays, so the port is still visible and still clickable: you
  //     lose the name, not the affordance.
  //   * between there and FADE_PX_PER_BLOCK it fades, so labels recede with
  //     distance instead of all shouting equally (depth cue, not decoration).
  //   * labels that would land in the same screen cell collide, and the one
  //     NEAREST the camera wins. Selected and hovered labels outrank both
  //     rules and are never dropped.
  static LABELS = {
    /** Hide below this: at 3.2 px a block, a 10 px label is bigger than the
     *  thing it names. */
    minPxPerBlock: 3.2,
    /** Fully opaque at or above this. */
    fadePxPerBlock: 9,
    /** Declutter cell, in CSS pixels (a port label is ~80x14). */
    cell: [86, 15] as [number, number],
  };

  /** Last frame's label accounting, for the UI and for `npm run verify`. */
  labelStats = { total: 0, shown: 0, hiddenSmall: 0, hiddenOverlap: 0, hiddenBehind: 0, pxPerBlock: 0 };

  private labelScratch = new THREE.Vector3();
  private labelSlots = new Map<string, number>();
  private labelOrder: { el: HTMLDivElement; prio: number; d: number; x: number; y: number; px: number }[] = [];

  private projectLabels() {
    if (this.labels.length === 0) {
      this.labelStats = { total: 0, shown: 0, hiddenSmall: 0, hiddenOverlap: 0, hiddenBehind: 0, pxPerBlock: 0 };
      return;
    }
    const rect = this.renderer.domElement.getBoundingClientRect();
    const v = this.labelScratch;
    const L = Viewer.LABELS;
    // Pixels per world unit at depth d: rect.height / (2 tan(fov/2) d).
    const k = rect.height / (2 * Math.tan(((this.camera.fov / 2) * Math.PI) / 180));
    const stats = { total: this.labels.length, shown: 0, hiddenSmall: 0, hiddenOverlap: 0, hiddenBehind: 0, pxPerBlock: 0 };
    this.labelOrder.length = 0;

    for (const { el, world, prio } of this.labels) {
      const d = this.camera.position.distanceTo(world);
      const px = k / Math.max(d, 0.001);
      v.copy(world).project(this.camera);
      if (v.z > 1) { el.style.display = "none"; stats.hiddenBehind++; continue; }
      if (px < L.minPxPerBlock && prio <= 0) { el.style.display = "none"; stats.hiddenSmall++; continue; }
      this.labelOrder.push({
        el, prio, d, px,
        x: ((v.x + 1) / 2) * rect.width,
        y: ((-v.y + 1) / 2) * rect.height,
      });
    }

    // Nearest first, and priority ahead of distance: whoever claims a screen
    // cell first keeps it.
    this.labelOrder.sort((a, b) => b.prio - a.prio || a.d - b.d);
    this.labelSlots.clear();
    let pxSum = 0;
    for (const l of this.labelOrder) {
      const key = `${Math.round(l.x / L.cell[0])},${Math.round(l.y / L.cell[1])}`;
      if (l.prio <= 0 && this.labelSlots.has(key)) {
        l.el.style.display = "none";
        stats.hiddenOverlap++;
        continue;
      }
      this.labelSlots.set(key, 1);
      l.el.style.display = "";
      l.el.style.left = `${l.x}px`;
      l.el.style.top = `${l.y}px`;
      // Depth fade, floored so a far label is dim but never illegible.
      const t = Math.min(1, Math.max(0, (l.px - L.minPxPerBlock) / (L.fadePxPerBlock - L.minPxPerBlock)));
      l.el.style.opacity = l.prio > 0 ? "1" : (0.45 + 0.55 * t).toFixed(2);
      stats.shown++;
      pxSum += l.px;
    }
    stats.pxPerBlock = stats.shown ? pxSum / stats.shown : 0;
    this.labelStats = stats;
  }

  /** Frame a world position: keep the camera's direction, put the point at the
   *  centre of the orbit and pull in to `radius`. The one motion every
   *  click-to-focus affordance in the outliner uses. */
  focusOn(pos: Vec3, radius = 26) {
    const target = new THREE.Vector3(pos[0], pos[1], pos[2]);
    const dir = this.camera.position.clone().sub(this.controls.target);
    if (dir.lengthSq() < 1e-6) dir.set(1, 1, 1);
    dir.normalize().multiplyScalar(radius);
    this.controls.target.copy(target);
    this.camera.position.copy(target).add(dir);
    this.controls.update();
    this.lastFocus = { target: [...pos] as Vec3, radius };
  }

  /** Where `focusOn` last aimed — a headless check for click-to-focus. */
  lastFocus: { target: Vec3; radius: number } | null = null;

  /** Pull the camera in/out along its current direction. The declutter rules
   *  are a function of this distance, so this is how a test walks them. */
  setOrbitRadius(r: number) {
    const dir = this.camera.position.clone().sub(this.controls.target);
    if (dir.lengthSq() < 1e-6) dir.set(1, 1, 1);
    this.camera.position.copy(this.controls.target).add(dir.normalize().multiplyScalar(r));
    this.controls.update();
  }

  orbitRadius(): number {
    return this.camera.position.distanceTo(this.controls.target);
  }

  /** Frame the WHOLE document.
   *
   *  The default camera was pitched for the 16x16 crossing sketch; the verified
   *  chain is 140 blocks wide, so landing on it put the viewer inside a bus.
   *  A demo (or `F` with nothing selected) now fits the bounding volume of
   *  every placed cell and every realized bus, from a fixed three-quarter
   *  angle — the first frame has to read as a diagram, not as a green wall. */
  frameAll(padding = 1.25): boolean {
    const box = new THREE.Box3();
    for (const inst of this.instances) {
      const { w, h, l } = Viewer.instBox(inst);
      box.expandByPoint(new THREE.Vector3(inst.at[0] - 0.5, inst.at[1] - 0.5, inst.at[2] - 0.5));
      box.expandByPoint(new THREE.Vector3(inst.at[0] + w - 0.5, inst.at[1] + h - 0.5, inst.at[2] + l - 0.5));
    }
    const layers = [...(this.lastScene?.buses ?? [])];
    if (this.lastScene?.loose) layers.push(this.lastScene.loose);
    for (const layer of layers) {
      for (const b of layer.blocks) box.expandByPoint(new THREE.Vector3(b.x, b.y, b.z));
    }
    if (box.isEmpty()) return false;
    const center = box.getCenter(new THREE.Vector3());
    const r = Math.max(box.getSize(new THREE.Vector3()).length() / 2, 3);
    const fovY = (this.camera.fov * Math.PI) / 180;
    const fovX = 2 * Math.atan(Math.tan(fovY / 2) * Math.max(this.camera.aspect, 0.2));
    const dist = (r / Math.sin(Math.min(fovY, fovX) / 2)) * padding;
    this.controls.target.copy(center);
    this.camera.position.copy(center).add(
      new THREE.Vector3(0.62, 0.55, 0.56).normalize().multiplyScalar(dist));
    this.camera.far = Math.max(4000, dist * 4);
    this.camera.updateProjectionMatrix();
    this.controls.update();
    this.lastFocus = { target: [center.x, center.y, center.z], radius: dist };
    return true;
  }

  /** Frame an instance by its bounding box, so `F` fills the view with it. */
  focusInstance(name: string): boolean {
    const inst = this.instances.find((i) => i.name === name);
    if (!inst) return false;
    const { w, h, l, center } = Viewer.instBox(inst);
    this.focusOn([center.x, center.y, center.z], Math.max(12, Math.max(w, h, l) * 1.6));
    return true;
  }

  // -- picking / drag -------------------------------------------------------

  private ndc(cx: number, cy: number): THREE.Vector2 {
    const rect = this.renderer.domElement.getBoundingClientRect();
    return new THREE.Vector2(
      ((cx - rect.left) / rect.width) * 2 - 1,
      -((cy - rect.top) / rect.height) * 2 + 1,
    );
  }

  /** THE PORT IS THE TARGET, and screen space is how it wins.
   *
   *  Ray-vs-geometry cannot express "the port always wins". A port marker sits
   *  one block outside its column, which means it is often INSIDE or BEHIND the
   *  neighbouring cell's body box — and the body box is an invisible solid, so
   *  the ray hits its near face first and the press becomes a component drag.
   *  That is the reported bug, exactly: "it's awkward to select an IO and route
   *  it, I always accidentally move the component".
   *
   *  So ports are picked by projecting every marker to the screen and taking
   *  the nearest one within a pixel radius. Depth cannot steal the press, no
   *  occluder can hide it, and the target is a constant size on screen at every
   *  zoom instead of shrinking to nothing — which is what makes a port still
   *  clickable at the zoom where its label has been decluttered away.
   *
   *  Ties (dense columns) go to the port nearest the CAMERA: the one in front is
   *  the one you meant. */
  private pickPort(cx: number, cy: number): { p: PortMarker; pos: Vec3; px: number } | null {
    if (!this.showIo || this.portRows.length === 0) return null;
    const rect = this.renderer.domElement.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return null;
    // Pixels per world unit at depth d: rect.height / (2 tan(fov/2) d).
    const k = rect.height / (2 * Math.tan(((this.camera.fov / 2) * Math.PI) / 180));
    const v = this.vScratch;
    let best: { p: PortMarker; pos: Vec3; px: number } | null = null;
    let bestScore = Infinity;
    let bestDepth = Infinity;
    for (const row of this.portRows) {
      v.set(...row.pos).project(this.camera);
      if (v.z > 1) continue; // behind the camera
      const x = ((v.x + 1) / 2) * rect.width + rect.left;
      const y = ((-v.y + 1) / 2) * rect.height + rect.top;
      const d = this.camera.position.distanceTo(this.vScratch2.set(...row.pos));
      // The generous radius: 2.5x the cone's own on-screen radius, floored at
      // PORT_PICK_PX so a far port is still a real target.
      const radius = Math.max(Viewer.PORT_PICK_PX, (2.5 * 0.45 * k) / Math.max(d, 0.001));
      const dist = Math.hypot(x - cx, y - cy);
      if (dist > radius) continue;
      const score = dist / radius;
      if (score < bestScore - 1e-6 || (Math.abs(score - bestScore) <= 1e-6 && d < bestDepth)) {
        bestScore = score;
        bestDepth = d;
        best = { p: row.p, pos: row.pos, px: dist };
      }
    }
    return best;
  }

  /** What is under the cursor, in PRIORITY order: ports, then the solid marker
   *  layer (gates and instance bodies, nearest first), then the bus fragments.
   *
   *  Buses come last for the same reason ports come first: a port cone sits ON a
   *  bus's first cell, and "click the bus" must not steal the click that starts
   *  a route. */
  private probe(cx: number, cy: number): Probe | null {
    const port = this.pickPort(cx, cy);
    this.raycaster.setFromCamera(this.ndc(cx, cy), this.camera);
    if (port) {
      // Did the port beat a body that was in front of it? Worth counting: it is
      // the case the whole priority rule exists for.
      const blocker = this.raycaster.intersectObjects(this.solidPickables(), false)[0];
      if (blocker && (blocker.object.userData as { type?: string }).type === "instance") {
        this.pickStats.portOverBody++;
      }
      return { type: "port", id: port.p.name, point: new THREE.Vector3(...port.pos) };
    }
    const solid = this.raycaster.intersectObjects(this.solidPickables(), false)[0];
    if (solid) {
      const ud = solid.object.userData as { type?: string; id?: string; y?: number };
      if (ud.type) return { type: ud.type as Probe["type"], id: ud.id ?? "", y: ud.y, point: solid.point };
    }
    const busMeshes = this.busGroup.children.filter((o) => (o as THREE.Mesh).isMesh);
    const bus = this.raycaster.intersectObjects(busMeshes, false)[0];
    if (!bus) return null;
    const ud = bus.object.userData as { type?: string; id?: string; y?: number };
    return ud.type ? { type: ud.type as Probe["type"], id: ud.id ?? "", y: ud.y, point: bus.point } : null;
  }

  /** The ray-picked part of the marker layer. In connect mode the bodies are
   *  simply not in it: nothing to hit, nothing to nudge. */
  private solidPickables(): THREE.Object3D[] {
    if (!this.connect) return this.pickables;
    const out: THREE.Object3D[] = [];
    for (const o of this.pickables) {
      if ((o.userData as { type?: string }).type === "instance") continue;
      out.push(o);
    }
    return out;
  }

  private groundPoint(cx: number, cy: number, y: number): Vec3 | null {
    this.raycaster.setFromCamera(this.ndc(cx, cy), this.camera);
    this.ground.constant = -y;
    const hit = new THREE.Vector3();
    if (!this.raycaster.ray.intersectPlane(this.ground, hit)) return null;
    return [Math.round(hit.x), y, Math.round(hit.z)];
  }

  /** World point on the y-plane under the cursor (for the ghost line). */
  worldPoint(e: PointerEvent, y: number): THREE.Vector3 | null {
    this.raycaster.setFromCamera(this.ndc(e.clientX, e.clientY), this.camera);
    this.ground.constant = -y;
    const hit = new THREE.Vector3();
    return this.raycaster.ray.intersectPlane(this.ground, hit) ? hit : null;
  }

  // ---- connect mode -------------------------------------------------------

  /** Bodies unpickable. Idempotent; `Alt` holds it, `C` latches it. */
  setConnectMode(on: boolean) {
    if (this.connect === on) return;
    this.connect = on;
    const el = this.renderer.domElement;
    el.classList.toggle("connect-mode", on);
    if (on && this.cameraLock == null) el.style.cursor = "crosshair";
  }

  connectMode(): boolean {
    return this.connect;
  }

  /** Where a port's marker is ON SCREEN, in client coordinates — what a test
   *  (or a tutorial arrow) needs to put a real pointer on it. */
  portScreenPos(name: string): [number, number] | null {
    const row = this.portRows.find((r) => r.p.name === name);
    if (!row) return null;
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.vScratch.set(...row.pos).project(this.camera);
    if (this.vScratch.z > 1) return null;
    return [
      ((this.vScratch.x + 1) / 2) * rect.width + rect.left,
      ((-this.vScratch.y + 1) / 2) * rect.height + rect.top,
    ];
  }

  /** The canvas in client coordinates — where a pointer can actually go. */
  canvasRect(): { left: number; top: number; right: number; bottom: number } {
    const r = this.renderer.domElement.getBoundingClientRect();
    return { left: r.left, top: r.top, right: r.right, bottom: r.bottom };
  }

  /** What a press at these client coordinates would resolve to. The pick
   *  priority, readable without synthesising an event. */
  probeAt(cx: number, cy: number): { kind: string; id: string } | null {
    const p = this.probe(cx, cy);
    return p ? { kind: p.type, id: p.id } : null;
  }

  /** The marker position of a port, for anchoring a ghost line. */
  portMarkerPos(name: string): Vec3 | null {
    const p = this.ports.find((q) => q.name === name);
    return p ? markerPos(p.anchor, p.step, p.width) : null;
  }

  /** Everything a context action needs: WHAT is under the cursor and WHERE.
   *
   *  The where is the point of it. "Add a gate here" is the entry that makes
   *  gate steering discoverable at all, and it is meaningless without the world
   *  position of the click — so the pick carries the hit point (rounded to the
   *  block grid) and falls back to the ground plane at the hit's height. */
  pickAt(e: PointerEvent): PickTarget {
    const info = this.probe(e.clientX, e.clientY);
    const screen: [number, number] = [e.clientX, e.clientY];
    const additive = e.shiftKey || e.metaKey || e.ctrlKey;
    const point = info?.point;
    const at: Vec3 = point
      ? [Math.round(point.x), Math.round(point.y), Math.round(point.z)]
      : this.groundPoint(e.clientX, e.clientY, 0) ?? [0, 0, 0];
    if (!info) return { kind: "ground", id: "", at, screen, additive };
    return { kind: info.type, id: info.id, at, screen, additive };
  }

  // ---- rendered geometry readback (the consistency check) -------------------
  //
  // The regression guard for "sometimes the bus doesn't update when I move a
  // component": read back what is ACTUALLY IN THE SCENE GRAPH and compare it
  // with what the engine says the design is. Not the arrays the renderer was
  // handed — the vertex buffers it built and the instance matrices it wrote,
  // because a stale mesh is precisely the case where those two disagree.
  //
  // A cube's 24 vertices start at its `+X -Y -Z` corner (see `mergeCubes`), so
  // the block position is recoverable from one vertex per cube.

  private static cellsOfGeometry(geo: THREE.BufferGeometry, m?: THREE.Matrix4): Set<string> {
    const out = new Set<string>();
    const pos = geo.getAttribute("position") as THREE.BufferAttribute | undefined;
    if (!pos) return out;
    const v = new THREE.Vector3();
    for (let i = 0; i < pos.count; i += 24) {
      v.set(pos.getX(i) - CUBE, pos.getY(i) + CUBE, pos.getZ(i) + CUBE);
      if (m) v.applyMatrix4(m);
      out.add(`${Math.round(v.x)},${Math.round(v.y)},${Math.round(v.z)}`);
    }
    return out;
  }

  /** Every layer the viewer is DRAWING, as sets of `"x,y,z"` world cells:
   *  `bus:<name>`, `loose`, and one `inst:<name>` per placement. */
  renderedCells(): Map<string, Set<string>> {
    const out = new Map<string, Set<string>>();
    for (const [name, built] of this.uniqueMeshes) {
      const mesh = built.objects.find((o) => (o as THREE.Mesh).isMesh) as THREE.Mesh | undefined;
      out.set(name, mesh ? Viewer.cellsOfGeometry(mesh.geometry) : new Set());
    }
    const m = new THREE.Matrix4();
    for (const entry of this.cellMeshes.values()) {
      const group = entry.groups[0];
      if (!group) continue;
      for (const [inst, row] of entry.slots) {
        if (row >= group.count) continue; // an allocated but unused row
        group.getMatrixAt(row, m);
        out.set(`inst:${inst}`, Viewer.cellsOfGeometry(group.geometry, m));
      }
    }
    return out;
  }

  private downAt: [number, number] | null = null;

  private pointerDown(e: PointerEvent) {
    if (this.connect) this.pickStats.bodiesSkippedInConnectMode++;
    const info = this.probe(e.clientX, e.clientY);
    if (!info) {
      this.downAt = [e.clientX, e.clientY];
      return;
    }
    const { type, id, y } = info;
    if (type === "port") {
      // A PORT PRESS IS NEVER A DRAG. It does not park in `pending` and it does
      // not touch `drag`, so no amount of subsequent pointer movement can turn
      // it into a component move — whatever the body geometry underneath it is.
      this.pickStats.portPicks++;
      this.cb.onPortClick(id);
      return;
    }
    if (type === "bus") {
      this.cb.onBusClick(id, e.shiftKey || e.metaKey || e.ctrlKey);
      return;
    }
    if (type === "instance") {
      this.pickStats.bodyPicks++;
      this.cb.onInstanceClick(id, e.shiftKey || e.metaKey || e.ctrlKey);
      // A shift-click EXTENDS a selection; it must not also start dragging the
      // instance it just added, or building a group scatters the group.
      if (e.shiftKey || e.metaKey || e.ctrlKey) return;
      this.armPending("instance", id, y ?? 0, e);
      return;
    }
    if (type === "gate") {
      // Selecting it is what makes Del work on a gate; the drag starts in the
      // same gesture, exactly as it does for an instance.
      this.cb.onGateClick(id);
      this.armPending("gate", id, y ?? 0, e);
    }
  }

  /** A body press: selected, camera parked, but NOTHING MOVED yet. */
  private armPending(kind: "instance" | "gate", id: string, y: number, e: PointerEvent) {
    this.pending = { kind, id, y, x: e.clientX, py: e.clientY };
    // The camera is parked from the press, not from the promotion: a press on a
    // body is never an orbit, whether or not it becomes a move.
    this.controls.enabled = false;
    this.renderer.domElement.setPointerCapture(e.pointerId);
  }

  /** Where the cursor last was on the ground plane — where a paste lands, so
   *  Ctrl-V puts the group where the user is looking rather than at the origin. */
  private lastGround: Vec3 | null = null;

  cursorGround(): Vec3 | null {
    return this.lastGround;
  }

  private pointerMove(e: PointerEvent) {
    this.lastGround = this.groundPoint(e.clientX, e.clientY, 0);
    // A parked press becomes a move only once it has travelled far enough to be
    // one. Until then the instance is untouched — the click is a selection.
    if (this.pending) {
      const moved = Math.hypot(e.clientX - this.pending.x, e.clientY - this.pending.py);
      if (moved < Viewer.DRAG_PX) return;
      const { kind, id, y } = this.pending;
      this.pending = null;
      this.drag = { kind, id, y };
      this.pickStats.dragsStarted++;
    }
    if (this.drag) {
      const p = this.groundPoint(e.clientX, e.clientY, this.drag.y);
      if (p) this.cb.onDragMove(this.drag.kind, this.drag.id, p);
      return;
    }
    if (this.ghost) {
      const w = this.worldPoint(e, this.ghost.from[1]);
      if (w) { this.ghost.to = w; this.refreshGhost(); }
    }
    const data = this.probe(e.clientX, e.clientY);
    const name = data?.type === "port" ? data.id : null;
    if (name !== this.hovered) {
      this.setHovered(name);
      this.cb.onPortHover(name);
    }
    this.setHoveredBus(data?.type === "bus" ? data.id : null);
    if (this.cameraLock == null) {
      // The cursor says which gesture the press will be, before it is made:
      // crosshair = start a bus here, move = this will move, pointer = select.
      this.renderer.domElement.style.cursor =
        name ? "crosshair"
        : data?.type === "instance" ? "move"
        : data ? "pointer"
        : this.connect ? "crosshair" : "";
    }
  }

  private pointerUp(e: PointerEvent) {
    if (this.downAt) {
      const [dx, dy] = [e.clientX - this.downAt[0], e.clientY - this.downAt[1]];
      this.downAt = null;
      if (Math.hypot(dx, dy) < Viewer.DRAG_PX) {
        const p = this.groundPoint(e.clientX, e.clientY, 0);
        if (p) this.cb.onGroundClick(p);
      }
    }
    if (this.pending) {
      // Released without ever crossing the threshold: the selection that
      // happened on the press IS the whole gesture. No drag end, so no move to
      // commit and no undo entry for a click.
      this.pending = null;
      this.pickStats.clicksBelowThreshold++;
      this.controls.enabled = this.cameraLock == null;
      return;
    }
    if (!this.drag) return;
    const drag = this.drag;
    this.drag = null;
    this.controls.enabled = this.cameraLock == null;
    const p = this.groundPoint(e.clientX, e.clientY, drag.y);
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
    this.controls.enabled = reason == null && this.drag == null && this.pending == null;
    const el = this.renderer.domElement;
    el.style.cursor = reason || this.connect ? "crosshair" : "";
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
