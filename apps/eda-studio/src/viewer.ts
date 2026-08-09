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
import type { LayerBlocks, Vec3 } from "./studio";

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
  private blockGroup = new THREE.Group();
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
    this.scene.add(grid, this.blockGroup, this.texturedGroup, this.markerGroup,
      this.gizmoGroup, this.ghostGroup);

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

    const tick = () => {
      requestAnimationFrame(tick);
      this.controls.update();
      this.projectLabels();
      this.renderer.render(this.scene, this.camera);
    };
    tick();
  }

  // -- scene rebuild --------------------------------------------------------

  private static disposeDeep(group: THREE.Group) {
    group.traverse((obj) => {
      const m = obj as Partial<THREE.Mesh>;
      (m.geometry as THREE.BufferGeometry | undefined)?.dispose();
      const mats = Array.isArray(m.material) ? m.material : m.material ? [m.material] : [];
      for (const mat of mats) (mat as THREE.Material).dispose();
    });
    group.clear();
  }

  setLayers(layers: LayerBlocks[]) {
    Viewer.disposeDeep(this.blockGroup);
    const box = new THREE.BoxGeometry(0.94, 0.94, 0.94);
    for (const layer of layers) {
      // In the textured view only the bus/failed layers stay abstract: the
      // routing is the thing the colours encode, and the pack cannot show it.
      const isBus = layer.layer.startsWith("bus:");
      if (this.textured && !isBus && !layer.failed) continue;
      const byColor = new Map<number, { x: number; y: number; z: number }[]>();
      for (const b of layer.blocks) {
        if (b.name === "minecraft:air") continue;
        const color = layer.failed ? FAILED_COLOR : layer.color ?? blockColor(b.name);
        let list = byColor.get(color);
        if (!list) byColor.set(color, (list = []));
        list.push(b);
      }
      for (const [color, blocks] of byColor) {
        const mat = new THREE.MeshLambertMaterial({
          color,
          transparent: layer.failed || (this.textured && isBus),
          opacity: layer.failed ? 0.85 : this.textured && isBus ? 0.75 : 1,
        });
        const mesh = new THREE.InstancedMesh(box, mat, blocks.length);
        const m = new THREE.Matrix4();
        blocks.forEach((b, i) => mesh.setMatrixAt(i, m.makeTranslation(b.x, b.y, b.z)));
        mesh.instanceMatrix.needsUpdate = true;
        this.blockGroup.add(mesh);
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
  }

  /** Whether the textured view is currently shown. */
  isTextured(): boolean {
    return this.textured;
  }

  setTexturedVisible(on: boolean) {
    this.textured = on && this.texturedGroup.children.length > 0;
    this.texturedGroup.visible = this.textured;
  }

  setShowIo(on: boolean) {
    this.showIo = on;
    this.refreshMarkers();
  }

  setSelection(sel: Selection) {
    this.selection = sel;
    this.refreshMarkers();
  }

  setHovered(name: string | null) {
    if (this.hovered === name) return;
    this.hovered = name;
    this.refreshMarkers();
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

  private label(text: string, cls: string, world: THREE.Vector3) {
    const el = document.createElement("div");
    el.className = `mk-label ${cls}`;
    el.textContent = text;
    this.labelLayer.appendChild(el);
    this.labels.push({ el, world });
  }

  private refreshMarkers() {
    Viewer.disposeDeep(this.markerGroup);
    Viewer.disposeDeep(this.gizmoGroup);
    this.pickables = [];
    this.labelLayer.replaceChildren();
    this.labels = [];

    if (this.showIo) {
      for (const p of ports_sorted(this.ports)) {
        const selected = this.selection?.kind === "port" && this.selection.id === p.name;
        const hovered = this.hovered === p.name;
        const base = !p.routable ? BLOCKED_COLOR
          : p.kind === "input" ? DRIVER_COLOR : SINK_COLOR;
        const color = selected ? SELECT_COLOR : hovered ? HOVER_COLOR : base;
        // A driver points OUT of its column, a sink points IN: the arrow
        // says which way signal flows before you read the label.
        const geo = new THREE.ConeGeometry(selected || hovered ? 0.55 : 0.45, 0.9, 4);
        const mat = new THREE.MeshLambertMaterial({
          color, transparent: !p.routable, opacity: p.routable ? 1 : 0.55,
        });
        const cone = new THREE.Mesh(geo, mat);
        const pos = markerPos(p.anchor, p.step, p.width);
        cone.position.set(...pos);
        cone.rotation.x = p.kind === "input" ? 0 : Math.PI;
        cone.userData = { type: "port", id: p.name };
        this.markerGroup.add(cone);
        this.pickables.push(cone);
        const cls = [
          p.kind === "input" ? "io-driver" : "io-sink",
          p.routable ? "" : "io-blocked",
          selected ? "is-selected" : "",
          hovered ? "is-hovered" : "",
        ].join(" ");
        this.label(`${p.name} : ${p.ty}${p.routable ? "" : " ✗"}`, cls,
          new THREE.Vector3(pos[0], pos[1] + 1.1, pos[2]));
      }
    }

    for (const g of this.gates) {
      const selected = this.selection?.kind === "gate" && this.selection.id === `${g.bus} ${g.name}`;
      const geo = new THREE.OctahedronGeometry(selected ? 0.7 : 0.55);
      const mat = new THREE.MeshLambertMaterial({ color: selected ? SELECT_COLOR : 0xffb74d });
      const m = new THREE.Mesh(geo, mat);
      m.position.set(...markerPos(g.anchor, g.step, g.width));
      m.userData = { type: "gate", id: `${g.bus} ${g.name}`, y: g.anchor[1] };
      this.markerGroup.add(m);
      this.pickables.push(m);
    }

    for (const inst of this.instances) {
      const [w, h, l] = inst.rot % 180 === 0 ? inst.dims : [inst.dims[2], inst.dims[1], inst.dims[0]];
      const selected = this.selection?.kind === "instance" && this.selection.id === inst.name;
      const geo = new THREE.BoxGeometry(w, h, l);
      const center = new THREE.Vector3(
        inst.at[0] + w / 2 - 0.5, inst.at[1] + h / 2 - 0.5, inst.at[2] + l / 2 - 0.5);
      const edges = new THREE.LineSegments(
        new THREE.EdgesGeometry(geo),
        new THREE.LineBasicMaterial({
          color: selected ? SELECT_COLOR : 0x4fc3f7,
          transparent: true, opacity: selected ? 1 : 0.5,
        }),
      );
      const pick = new THREE.Mesh(geo, new THREE.MeshBasicMaterial({ visible: false }));
      edges.position.copy(center);
      pick.position.copy(center);
      pick.userData = { type: "instance", id: inst.name, y: inst.at[1] };
      this.markerGroup.add(edges, pick);
      this.pickables.push(pick);

      if (selected) {
        // A translucent shell so the selection reads from any angle...
        const shell = new THREE.Mesh(geo.clone(), new THREE.MeshBasicMaterial({
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
          this.gizmoGroup.add(arrow);
        }
        this.label(`${inst.name} · ${inst.cell} · rot ${inst.rot}°`, "inst-label is-selected",
          new THREE.Vector3(center.x, inst.at[1] + h + 0.8, center.z));
      }
    }
  }

  /** Project label anchors to screen space once per frame. */
  private projectLabels() {
    if (this.labels.length === 0) return;
    const rect = this.renderer.domElement.getBoundingClientRect();
    const v = new THREE.Vector3();
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
    const hit = this.castPointer(e);
    if (!hit) {
      this.downAt = [e.clientX, e.clientY];
      return;
    }
    const { type, id, y } = hit.object.userData as { type: string; id: string; y?: number };
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
    const data = hit?.object.userData as { type?: string; id?: string } | undefined;
    const name = data?.type === "port" ? data.id ?? null : null;
    if (name !== this.hovered) {
      this.setHovered(name);
      this.cb.onPortHover(name);
    }
    this.renderer.domElement.style.cursor = hit ? "pointer" : "";
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
    this.controls.enabled = true;
    const p = this.groundPoint(e, drag.y);
    if (p) this.cb.onDragEnd(drag.kind, drag.id, p);
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
