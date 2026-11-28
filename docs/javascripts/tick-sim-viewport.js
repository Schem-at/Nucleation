/**
 * Real Three.js + Nucleation WASM MeshResult interactive 3D viewport for the
 * mc-tick redstone simulator.
 *
 * Uses the exact same meshing + interaction pipeline as `apps/sim-lab-wasm`:
 *   - Real Rust/WASM `MeshResult.createOrEmpty(schematic, pack, cfg)` -> GLB
 *   - `GLTFLoader.parseAsync` with `NearestFilter` pixel-crisp textures
 *   - FNV-1a embedded-atlas cache (`stripEmbeddedImages`) for ~1–4 ms re-meshes
 *   - Amanatides–Woo voxel raycasting matching schematic-mesher's `[i-0.5, i+0.5]` cells
 *   - Smooth `sim.movingBlocksJson()` piston interpolation across fractional ticks
 */

let threePromise = null;

export function loadThree() {
  threePromise ??= Promise.all([
    import("https://esm.sh/three@0.170.0"),
    import("https://esm.sh/three@0.170.0/examples/jsm/controls/OrbitControls.js"),
    import("https://esm.sh/three@0.170.0/examples/jsm/loaders/GLTFLoader.js"),
  ]).then(([THREE, { OrbitControls }, { GLTFLoader }]) => ({
    THREE,
    OrbitControls,
    GLTFLoader,
  }));
  return threePromise;
}

const INTERACTIVE_RE = /lever|button|note_block|trapdoor|_door|_gate|repeater|comparator|daylight/;

function decodeBase64(b64) {
  const bin = globalThis.atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i += 1) out[i] = bin.charCodeAt(i);
  return out;
}

function hashBytes(bytes) {
  let h = 2166136261;
  for (let i = 0; i < bytes.length; i += 1) {
    h ^= bytes[i];
    h = Math.imul(h, 16777619);
  }
  return `${bytes.length}:${(h >>> 0).toString(16)}`;
}

function blendKey(mat) {
  if (mat.transparent) return "BLEND";
  if ((mat.alphaTest ?? 0) > 0) return "MASK";
  return "OPAQUE";
}

function configurePixelMaterial(THREE, mat) {
  if (mat.map) {
    mat.map.magFilter = THREE.NearestFilter;
    mat.map.minFilter = THREE.NearestFilter;
    mat.map.generateMipmaps = false;
    mat.map.colorSpace = THREE.SRGBColorSpace;
    mat.map.needsUpdate = true;
  }
  // Keep matte voxel finish so baked AO & pixel art shine cleanly
  if (typeof mat.roughness === "number") mat.roughness = 0.92;
  if (typeof mat.metalness === "number") mat.metalness = 0.0;
}

/**
 * Strip embedded atlas images/textures/samplers from a GLB so repeat re-meshes
 * with the same atlas hash skip redundant PNG decode (mirrors sim-lab-wasm).
 */
function stripEmbeddedImages(bytes) {
  try {
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    if (view.getUint32(0, true) !== 0x46546c67) return null;
    const jsonLen = view.getUint32(12, true);
    const json = JSON.parse(new TextDecoder().decode(bytes.subarray(20, 20 + jsonLen)));
    if (!json.images?.length) return null;
    const bv = json.bufferViews?.[json.images[0].bufferView];
    if (!bv) return null;
    const binStart = 20 + jsonLen + 8;
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
    let text = JSON.stringify(json);
    while (text.length % 4 !== 0) text += " ";
    const jsonBytes = new TextEncoder().encode(text);
    const rest = bytes.subarray(20 + jsonLen);
    const out = new Uint8Array(20 + jsonBytes.length + rest.length);
    const dv = new DataView(out.buffer);
    dv.setUint32(0, 0x46546c67, true);
    dv.setUint32(4, 2, true);
    dv.setUint32(8, out.length, true);
    dv.setUint32(12, jsonBytes.length, true);
    dv.setUint32(16, 0x4e4f534a, true);
    out.set(jsonBytes, 20);
    out.set(rest, 20 + jsonBytes.length);
    return { buffer: out.buffer, atlas };
  } catch {
    return null;
  }
}

/**
 * Amanatides–Woo grid march matching Nucleation's `[i - 0.5, i + 0.5]` block centering.
 */
function pickVoxel(ray, isSolid, reach = 48) {
  const ox = ray.origin.x + 0.5;
  const oy = ray.origin.y + 0.5;
  const oz = ray.origin.z + 0.5;
  const dx = ray.direction.x;
  const dy = ray.direction.y;
  const dz = ray.direction.z;

  const cell = [Math.floor(ox), Math.floor(oy), Math.floor(oz)];
  const step = [Math.sign(dx), Math.sign(dy), Math.sign(dz)];
  const tDelta = [
    dx === 0 ? Infinity : Math.abs(1 / dx),
    dy === 0 ? Infinity : Math.abs(1 / dy),
    dz === 0 ? Infinity : Math.abs(1 / dz),
  ];
  const dist = (o, c, s) => (s > 0 ? c + 1 - o : o - c);
  const tMax = [
    step[0] === 0 ? Infinity : dist(ox, cell[0], step[0]) * tDelta[0],
    step[1] === 0 ? Infinity : dist(oy, cell[1], step[1]) * tDelta[1],
    step[2] === 0 ? Infinity : dist(oz, cell[2], step[2]) * tDelta[2],
  ];

  let face = [0, 1, 0];
  let travelled = 0;
  for (let i = 0; i < reach * 3 && travelled <= reach; i += 1) {
    if (isSolid(cell[0], cell[1], cell[2])) {
      return { pos: [cell[0], cell[1], cell[2]], face: [...face] };
    }
    let axis = 0;
    if (tMax[1] < tMax[0]) axis = 1;
    if (tMax[2] < tMax[axis]) axis = 2;
    travelled = tMax[axis];
    cell[axis] += step[axis];
    tMax[axis] += tDelta[axis];
    face = [0, 0, 0];
    face[axis] = -step[axis];
  }
  return null;
}

export class TickSimViewport {
  constructor(container, threeKit, callbacks) {
    const { THREE, OrbitControls, GLTFLoader } = threeKit;
    this.THREE = THREE;
    this.loader = new GLTFLoader();
    this.container = container;
    this.callbacks = callbacks;

    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(42, 1, 0.1, 500);
    this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.NoToneMapping;
    this.renderer.domElement.className = "tick-3d-canvas";
    this.container.appendChild(this.renderer.domElement);

    // Balanced lighting calibrated for Nucleation's baked vertex AO (matches sim-lab-wasm)
    this.scene.add(new THREE.AmbientLight(0xffffff, 1.9));
    const key = new THREE.DirectionalLight(0xfffaf0, 1.45);
    key.position.set(10, 18, 12);
    this.scene.add(key);
    const fill = new THREE.DirectionalLight(0xd2e4ff, 0.55);
    fill.position.set(-10, 11, -12);
    this.scene.add(fill);

    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.09;
    this.controls.maxDistance = 120;
    this.controls.minDistance = 2.5;

    this.worldGroup = new THREE.Group();
    this.flightGroup = new THREE.Group();
    this.scene.add(this.worldGroup);
    this.scene.add(this.flightGroup);

    this.gridHelper = null;

    // Hover wireframe outline around target voxel
    this.outlineMat = new THREE.LineBasicMaterial({ color: 0x5fd38d, depthTest: true });
    this.outline = new THREE.LineSegments(
      new THREE.EdgesGeometry(new THREE.BoxGeometry(1.006, 1.006, 1.006)),
      this.outlineMat,
    );
    this.outline.visible = false;
    this.scene.add(this.outline);

    // Placement ghost preview for Build mode
    this.ghostGroup = new THREE.Group();
    this.ghostMesh = new THREE.Mesh(
      new THREE.BoxGeometry(1.0, 1.0, 1.0),
      new THREE.MeshBasicMaterial({
        color: 0x53c7b4,
        transparent: true,
        opacity: 0.22,
        depthWrite: false,
      }),
    );
    this.ghostEdge = new THREE.LineSegments(
      new THREE.EdgesGeometry(new THREE.BoxGeometry(1.004, 1.004, 1.004)),
      new THREE.LineBasicMaterial({ color: 0x53c7b4, transparent: true, opacity: 0.85 }),
    );
    this.ghostGroup.add(this.ghostMesh);
    this.ghostGroup.add(this.ghostEdge);
    this.ghostGroup.visible = false;
    this.scene.add(this.ghostGroup);

    // Brief click flash highlight
    this.flashMat = new THREE.MeshBasicMaterial({
      color: 0x6fd66a,
      transparent: true,
      opacity: 0.42,
      depthWrite: false,
    });
    this.flashMesh = new THREE.Mesh(new THREE.BoxGeometry(1.05, 1.05, 1.05), this.flashMat);
    this.flashMesh.visible = false;
    this.flashUntil = 0;
    this.scene.add(this.flashMesh);

    this.raycaster = new THREE.Raycaster();
    this.ndc = new THREE.Vector2();
    this.pointerInside = false;
    this.pointerDownPos = null;
    this.shiftHeld = false;
    this.currentHit = null;
    this.currentTool = "interact";

    this.atlasCache = new Map();
    this.blockMeshCache = new Map();
    this.pendingBlockMeshes = new Set();
    this.activeMeshObj = null;
    this.flights = [];
    this.meshSeq = 0;
    this.lastMeshStats = { tris: 0, verts: 0, ms: 0 };
    this.lastBounds = null;

    this.setTheme(document.body.getAttribute("data-md-color-scheme") === "vellum");

    this._onResize = () => this.resize();
    this.resizeObserver = new ResizeObserver(this._onResize);
    this.resizeObserver.observe(this.container);
    this.resize();

    this._bindEvents();
    this._running = true;
    this._lastTime = performance.now();
    this._raf = requestAnimationFrame((t) => this._frame(t));
  }

  setTheme(isLight) {
    const bg = isLight ? 0xe7e3d9 : 0x0e1318;
    this.scene.background = new this.THREE.Color(bg);
    this._rebuildGrid(isLight);
  }

  _rebuildGrid(isLight) {
    if (this.gridHelper) {
      this.scene.remove(this.gridHelper);
      this.gridHelper.geometry?.dispose?.();
      this.gridHelper.material?.dispose?.();
    }
    const major = isLight ? 0xbcb6a8 : 0x26323d;
    const minor = isLight ? 0xd3cec1 : 0x18212a;
    const grid = new this.THREE.GridHelper(32, 32, major, minor);
    grid.position.set(3.5, -0.502, 1.5);
    this.scene.add(grid);
    this.gridHelper = grid;
  }

  setTool(tool) {
    this.currentTool = tool;
    this._updateHoverVisuals();
  }

  clearCaches() {
    this.atlasCache.clear();
    this.blockMeshCache.clear();
    this.pendingBlockMeshes.clear();
  }

  resize() {
    const w = Math.max(this.container.clientWidth, 1);
    const h = Math.max(this.container.clientHeight, 1);
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
  }

  /** Horizontal cardinal direction the camera is looking toward. */
  cameraFacing() {
    const dir = new this.THREE.Vector3();
    this.camera.getWorldDirection(dir);
    if (Math.abs(dir.x) >= Math.abs(dir.z)) {
      return dir.x >= 0 ? "east" : "west";
    }
    return dir.z >= 0 ? "south" : "north";
  }

  /** Frame the current schematic cleanly from a 3/4 isometric perspective. */
  frameBounds(minX, minY, minZ, maxX, maxY, maxZ) {
    const cx = (minX + maxX) * 0.5;
    const cy = (minY + maxY) * 0.5;
    const cz = (minZ + maxZ) * 0.5;
    const dx = Math.max(1, maxX - minX + 1);
    const dy = Math.max(1, maxY - minY + 1);
    const dz = Math.max(1, maxZ - minZ + 1);
    const radius = Math.max(Math.hypot(dx, dy, dz) * 0.5, 2.8);
    const fovRad = (this.camera.fov * Math.PI) / 180;
    const dist = (radius * 0.92) / Math.sin(fovRad * 0.5);

    this.controls.target.set(cx, cy, cz);
    // Front-south-east elevated angle so redstone rows along +X read left-to-right!
    const offset = new this.THREE.Vector3(0.28, 0.72, 0.88).normalize().multiplyScalar(dist);
    this.camera.position.copy(this.controls.target).add(offset);
    this.camera.near = Math.max(0.05, dist / 200);
    this.camera.far = Math.max(200, dist * 16);
    this.camera.updateProjectionMatrix();
    this.controls.update();

    if (this.gridHelper) {
      this.gridHelper.position.set(cx, minY - 0.502, cz);
    }
    this.lastBounds = { minX, minY, minZ, maxX, maxY, maxZ };
  }

  resetCamera() {
    if (this.lastBounds) {
      const b = this.lastBounds;
      this.frameBounds(b.minX, b.minY, b.minZ, b.maxX, b.maxY, b.maxZ);
    }
  }

  flashVoxel(x, y, z, colorHex) {
    this.flashMesh.position.set(x, y, z);
    this.flashMat.color.setHex(colorHex);
    this.flashMat.opacity = 0.44;
    this.flashMesh.visible = true;
    this.flashUntil = performance.now() + 340;
  }

  /**
   * Mesh `schem` through Nucleation's real Rust/WASM `MeshResult.createOrEmpty`
   * and swap it into the Three.js scene using the embedded-atlas material cache.
   */
  async updateSchematicMesh(nuc, schem, pack, refitCamera = false) {
    const seq = ++this.meshSeq;
    const t0 = performance.now();

    const cfg = nuc.MeshConfig.create();
    cfg.setAmbientOcclusion(true);
    cfg.setAoIntensity(0.45);
    cfg.setGreedyMeshing(false);

    const meshRes = nuc.MeshResult.createOrEmpty(schem, pack, cfg);
    const tris = Number(meshRes.triangleCount());
    const verts = Number(meshRes.vertexCount());
    let nextObj = null;

    if (tris > 0) {
      const glbB64 = meshRes.glbDataB64();
      if (glbB64 && glbB64.length > 0) {
        const bytes = decodeBase64(glbB64);
        const lean = stripEmbeddedImages(bytes);
        const cached = lean ? this.atlasCache.get(lean.atlas) : undefined;
        const buffer = cached
          ? lean.buffer
          : bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);

        const gltf = await this.loader.parseAsync(buffer, "");
        if (seq !== this.meshSeq) return null;

        if (cached) {
          gltf.scene.traverse((o) => {
            if (!o.isMesh || !o.material) return;
            const shared = cached.get(blendKey(o.material));
            if (!shared) return;
            o.material.dispose?.();
            o.material = shared;
            o.renderOrder = shared.transparent ? 1 : 0;
          });
        } else {
          const harvested = new Map();
          gltf.scene.traverse((o) => {
            if (!o.isMesh || !o.material) return;
            configurePixelMaterial(this.THREE, o.material);
            o.renderOrder = o.material.transparent ? 1 : 0;
            harvested.set(blendKey(o.material), o.material);
          });
          if (lean && harvested.size > 0) {
            if (this.atlasCache.size >= 32) {
              const oldest = this.atlasCache.keys().next().value;
              if (oldest !== undefined) this.atlasCache.delete(oldest);
            }
            this.atlasCache.set(lean.atlas, harvested);
          }
        }
        nextObj = gltf.scene;
      }
    }

    if (seq !== this.meshSeq) return null;

    if (this.activeMeshObj) {
      this.worldGroup.remove(this.activeMeshObj);
      this.activeMeshObj.traverse((o) => {
        if (o.isMesh) o.geometry?.dispose?.();
      });
    }
    this.activeMeshObj = nextObj;
    if (nextObj) this.worldGroup.add(nextObj);

    const ms = performance.now() - t0;
    this.lastMeshStats = { tris, verts, ms };

    if (refitCamera && nextObj) {
      const box = new this.THREE.Box3().setFromObject(nextObj);
      if (!box.isEmpty()) {
        this.frameBounds(
          Math.floor(box.min.x + 0.5),
          Math.floor(box.min.y + 0.5),
          Math.floor(box.min.z + 0.5),
          Math.ceil(box.max.x - 0.5),
          Math.ceil(box.max.y - 0.5),
          Math.ceil(box.max.z - 0.5),
        );
      }
    }
    return this.lastMeshStats;
  }

  /** Cache a 1×1×1 Nucleation mesh for smooth moving piston interpolation. */
  _ensureBlockMesh(nuc, pack, state) {
    if (!state || state === "minecraft:air" || state.startsWith("minecraft:moving_piston")) {
      return null;
    }
    if (this.blockMeshCache.has(state)) return this.blockMeshCache.get(state);
    if (this.pendingBlockMeshes.has(state)) return null;
    this.pendingBlockMeshes.add(state);
    void (async () => {
      let obj = null;
      try {
        const scratch = nuc.Schematic.create("flight");
        scratch.setBlockFromString(0, 0, 0, state);
        const cfg = nuc.MeshConfig.create();
        cfg.setAmbientOcclusion(false);
        const mr = nuc.MeshResult.createOrEmpty(scratch, pack, cfg);
        if (mr.triangleCount() > 0) {
          const bytes = decodeBase64(mr.glbDataB64());
          const gltf = await this.loader.parseAsync(
            bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength),
            "",
          );
          gltf.scene.traverse((o) => {
            if (!o.isMesh || !o.material) return;
            configurePixelMaterial(this.THREE, o.material);
            o.renderOrder = o.material.transparent ? 1 : 0;
          });
          obj = gltf.scene;
        }
      } catch {
        obj = null;
      }
      this.blockMeshCache.set(state, obj);
      this.pendingBlockMeshes.delete(state);
    })();
    return null;
  }

  syncFlights(nuc, pack, movingList) {
    // Clear old flight objects
    while (this.flightGroup.children.length > 0) {
      const child = this.flightGroup.children[0];
      this.flightGroup.remove(child);
    }
    this.flights = (movingList || []).map((m) => {
      this._ensureBlockMesh(nuc, pack, m.carried);
      if (m.carried_short) this._ensureBlockMesh(nuc, pack, m.carried_short);
      if (m.remains) this._ensureBlockMesh(nuc, pack, m.remains);
      return {
        ...m,
        obj: null,
        drawnState: null,
        stillObj: null,
      };
    });
  }

  animateFlights(nuc, pack, tickNow) {
    if (!this.flights.length) return;
    for (const f of this.flights) {
      if (f.remains && !f.stillObj) {
        const proto = this._ensureBlockMesh(nuc, pack, f.remains);
        if (proto) {
          f.stillObj = proto.clone(true);
          f.stillObj.position.set(f.to[0], f.to[1], f.to[2]);
          this.flightGroup.add(f.stillObj);
        }
      }
      const span = f.lands - f.started;
      const t = span > 0 ? Math.min(1, Math.max(0, (tickNow - f.started) / span)) : 1;
      const want =
        f.carried_short && (f.extending ? t <= 0.5 : t >= 0.5) ? f.carried_short : f.carried;
      if (!f.obj || f.drawnState !== want) {
        const proto = this._ensureBlockMesh(nuc, pack, want);
        if (proto) {
          if (f.obj) this.flightGroup.remove(f.obj);
          f.obj = proto.clone(true);
          f.drawnState = want;
          this.flightGroup.add(f.obj);
        }
      }
      if (f.obj) {
        f.obj.position.set(
          f.from[0] + (f.to[0] - f.from[0]) * t,
          f.from[1] + (f.to[1] - f.from[1]) * t,
          f.from[2] + (f.to[2] - f.from[2]) * t,
        );
      }
    }
  }

  /** Project a world voxel center `(x, y, z)` to canvas CSS pixel coordinates. */
  projectVoxelToClient(x, y, z) {
    const v = new this.THREE.Vector3(x, y, z).project(this.camera);
    const rect = this.renderer.domElement.getBoundingClientRect();
    return {
      x: rect.left + ((v.x + 1) * 0.5) * rect.width,
      y: rect.top + ((1 - v.y) * 0.5) * rect.height,
      visible: v.z >= -1 && v.z <= 1,
    };
  }

  _pickAtClient(clientX, clientY) {
    const rect = this.renderer.domElement.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return null;
    this.ndc.x = ((clientX - rect.left) / rect.width) * 2 - 1;
    this.ndc.y = -((clientY - rect.top) / rect.height) * 2 + 1;
    this.raycaster.setFromCamera(this.ndc, this.camera);
    return pickVoxel(this.raycaster.ray, (x, y, z) => this.callbacks.isSolid(x, y, z));
  }

  _updateHoverVisuals() {
    const hit = this.currentHit;
    if (!hit) {
      this.outline.visible = false;
      this.ghostGroup.visible = false;
      this.callbacks.onHover?.(null);
      return;
    }

    const state = this.callbacks.getBlock(hit.pos[0], hit.pos[1], hit.pos[2]);
    const isBreak = this.currentTool === "break" || this.shiftHeld;
    const isInteract = this.currentTool === "interact" && !this.shiftHeld;

    this.outline.position.set(hit.pos[0], hit.pos[1], hit.pos[2]);
    this.outline.visible = true;

    if (isBreak) {
      this.outlineMat.color.setHex(0xff6b6b);
      this.ghostGroup.visible = false;
    } else if (isInteract) {
      this.outlineMat.color.setHex(INTERACTIVE_RE.test(state) ? 0x5fd38d : 0xd8dee9);
      this.ghostGroup.visible = false;
    } else {
      this.outlineMat.color.setHex(0x9fb3c8);
      const nx = hit.pos[0] + hit.face[0];
      const ny = hit.pos[1] + hit.face[1];
      const nz = hit.pos[2] + hit.face[2];
      this.ghostGroup.position.set(nx, ny, nz);
      this.ghostGroup.visible = true;
    }
    this.callbacks.onHover?.({ ...hit, state });
  }

  _bindEvents() {
    const dom = this.renderer.domElement;

    this._onPointerMove = (e) => {
      this.pointerInside = true;
      this.shiftHeld = e.shiftKey;
      this.currentHit = this._pickAtClient(e.clientX, e.clientY);
      this._updateHoverVisuals();
    };

    this._onPointerLeave = () => {
      this.pointerInside = false;
      this.currentHit = null;
      this._updateHoverVisuals();
    };

    this._onPointerDown = (e) => {
      if (e.button === 0 || e.button === 2) {
        this.pointerDownPos = { x: e.clientX, y: e.clientY, button: e.button };
      }
    };

    this._onPointerUp = (e) => {
      if (!this.pointerDownPos) return;
      const dist = Math.hypot(e.clientX - this.pointerDownPos.x, e.clientY - this.pointerDownPos.y);
      const btn = this.pointerDownPos.button;
      this.pointerDownPos = null;
      if (dist > 5) return; // Dragged to orbit/pan

      const hit = this._pickAtClient(e.clientX, e.clientY);
      this.currentHit = hit;
      this.shiftHeld = e.shiftKey;
      if (hit) {
        this.callbacks.onVoxelClick?.(hit, {
          button: btn,
          shiftKey: e.shiftKey,
          facing: this.cameraFacing(),
        });
      }
      this._updateHoverVisuals();
    };

    this._onKey = (e) => {
      if (this.shiftHeld !== e.shiftKey) {
        this.shiftHeld = e.shiftKey;
        if (this.pointerInside) this._updateHoverVisuals();
      }
    };

    dom.addEventListener("pointermove", this._onPointerMove);
    dom.addEventListener("pointerleave", this._onPointerLeave);
    dom.addEventListener("pointerdown", this._onPointerDown);
    dom.addEventListener("pointerup", this._onPointerUp);
    window.addEventListener("keydown", this._onKey);
    window.addEventListener("keyup", this._onKey);
  }

  _frame(now) {
    if (!this._running) return;
    this._raf = requestAnimationFrame((t) => this._frame(t));
    const dt = Math.min((now - this._lastTime) / 1000, 0.1);
    this._lastTime = now;

    this.controls.update();
    this.callbacks.onFrame?.(dt, now);

    if (this.flashMesh.visible) {
      const left = this.flashUntil - now;
      if (left <= 0) {
        this.flashMesh.visible = false;
      } else {
        this.flashMat.opacity = (left / 340) * 0.44;
      }
    }

    this.renderer.render(this.scene, this.camera);
  }

  destroy() {
    this._running = false;
    cancelAnimationFrame(this._raf);
    this.resizeObserver.disconnect();
    const dom = this.renderer.domElement;
    dom.removeEventListener("pointermove", this._onPointerMove);
    dom.removeEventListener("pointerleave", this._onPointerLeave);
    dom.removeEventListener("pointerdown", this._onPointerDown);
    dom.removeEventListener("pointerup", this._onPointerUp);
    window.removeEventListener("keydown", this._onKey);
    window.removeEventListener("keyup", this._onKey);
    this.controls.dispose();
    this.renderer.dispose();
    if (dom.parentNode === this.container) this.container.removeChild(dom);
  }
}
