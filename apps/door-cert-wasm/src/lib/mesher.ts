/** Real-mesher bridge: nucleation wasm (public/engine/) + the client.jar
 * resource-pack subset (public/pack/mesher-pack.zip) → three.js models.
 *
 * Each unique blockstate string is meshed once as a 1-block schematic at the
 * origin (`Schematic.create` + `setBlockFromString`, then `MeshResult.create`)
 * and delivered as a GLB (`glbDataB64`). The mesher's GLB already carries the
 * split we need: an OPAQUE material, a MASK(0.5, doubleSided) material for
 * cutout (torches, dust, levers) and a BLEND(doubleSided) material for
 * translucents (slime/honey/glass), plus NEAREST samplers and baked-AO vertex
 * colors — GLTFLoader maps all of it onto three.js materials directly. */

import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";

/* eslint-disable @typescript-eslint/no-explicit-any */
type Engine = any;

let enginePromise: Promise<Engine> | null = null;
export function loadEngine(): Promise<Engine> {
  return (enginePromise ??= import(
    /* @vite-ignore */ new URL("/engine/index.mjs", window.location.origin).href
  ));
}

type MeshEnv = { eng: Engine; pack: any; cfg: any };
let envPromise: Promise<MeshEnv> | null = null;

/** Engine + resource pack + default MeshConfig, loaded once. */
export function loadMeshEnv(): Promise<MeshEnv> {
  return (envPromise ??= (async () => {
    const [eng, zip] = await Promise.all([
      loadEngine(),
      fetch("/pack/mesher-pack.zip").then((r) => {
        if (!r.ok) throw new Error(`mesher-pack.zip: HTTP ${r.status}`);
        return r.arrayBuffer();
      }),
    ]);
    const pack = eng.ResourcePack.fromBytes(new Uint8Array(zip));
    const cfg = eng.MeshConfig.create();
    return { eng, pack, cfg };
  })());
}

function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

const loader = new GLTFLoader();
const modelCache = new Map<string, Promise<THREE.Group>>();
/** The resolved env, so the synchronous encode path can reach it. */
let envSync: MeshEnv | null = null;
const baseState = (state: string) => state.split("[", 1)[0];

/** Instrumentation: unique states meshed + summed SYNC wasm mesh ms (the
 * schematic build + mesh + GLB encode; GLTF parse is async and overlaps). */
export const meshStats = { states: 0, totalMs: 0 };

/** GLB bytes for one blockstate, or null if the mesher will not encode it.
 *
 *  Two kinds of failure, and only one of them is recoverable. A state that
 *  fails on its PROPERTIES can be drawn from the bare id: wrong about the
 *  rotation, right about everything else, and better than a hole. A block the
 *  mesher cannot encode at all cannot be drawn — `minecraft:chain` raises
 *  `NucleationError.Serialize` for `axis=x`, `axis=z` AND its own default
 *  `axis=y`, so there is nothing to fall back to. That one is reported: the
 *  caller leaves the cells empty and the replay says which block is missing,
 *  because a hole nobody names is the worst of the three outcomes. */
function encode(state: string): Uint8Array | null {
  const { eng, pack, cfg } = envSync!;
  try {
    const sc = eng.Schematic.create("cell");
    sc.setBlockFromString(0, 0, 0, state);
    return b64ToBytes(eng.MeshResult.create(sc, pack, cfg).glbDataB64());
  } catch {
    return null;
  }
}

async function buildModel(state: string): Promise<THREE.Group> {
  envSync = await loadMeshEnv();
  const t0 = performance.now();
  const bare = baseState(state);
  let glb = encode(state);
  if (!glb && bare !== state) {
    glb = encode(bare);
    if (glb)
      console.warn(
        `[mesh] ${state} could not be encoded; drawn as ${bare} in its default orientation`,
      );
  }
  if (!glb) throw new Error(`the mesher cannot encode ${state}`);
  meshStats.states += 1;
  meshStats.totalMs += performance.now() - t0;
  const gltf = await loader.parseAsync(glb.buffer as ArrayBuffer, "");
  const group = gltf.scene;
  group.traverse((o: THREE.Object3D) => {
    const mesh = o as THREE.Mesh;
    if (!mesh.isMesh) return;
    const mat = mesh.material as THREE.MeshStandardMaterial;
    if (mat.map) {
      // GLB samplers are already NEAREST; make it explicit and kill mips.
      mat.map.magFilter = THREE.NearestFilter;
      mat.map.minFilter = THREE.NearestFilter;
      mat.map.generateMipmaps = false;
      mat.map.colorSpace = THREE.SRGBColorSpace;
    }
    // Translucent (BLEND) geometry renders after everything opaque/cutout.
    mesh.renderOrder = mat.transparent ? 1 : 0;
  });
  return group;
}

/** Shared model for a blockstate (do not add to a scene — clone it). */
export function modelFor(state: string): Promise<THREE.Group> {
  let p = modelCache.get(state);
  if (!p) {
    p = buildModel(state);
    modelCache.set(state, p);
  }
  return p;
}

/** Scene-insertable instance sharing geometry + materials with the cache. */
export async function instanceFor(state: string): Promise<THREE.Group> {
  const model = await modelFor(state);
  return model.clone(true);
}

/** Ambient + key light approximating the native renderer's flat-ish look
 * (AO is baked into vertex colors, so lighting stays gentle). */
export function addDefaultLights(scene: THREE.Scene): void {
  scene.add(new THREE.AmbientLight(0xffffff, 2.0));
  const key = new THREE.DirectionalLight(0xffffff, 1.6);
  key.position.set(0.8, 1.6, 0.5);
  scene.add(key);
  const fill = new THREE.DirectionalLight(0xffffff, 0.5);
  fill.position.set(-0.6, 0.8, -1.0);
  scene.add(fill);
}

/** Camera position/target fitting a world-space box from the classic
 * front-right-top isometric direction, like the native --tight framing. */
export function fitCamera(
  camera: THREE.PerspectiveCamera,
  box: THREE.Box3,
  margin = 0.95,
): THREE.Vector3 {
  const center = box.getCenter(new THREE.Vector3());
  const sphere = box.getBoundingSphere(new THREE.Sphere());
  const dist =
    (sphere.radius * margin) / Math.sin((camera.fov * Math.PI) / 360);
  const dir = new THREE.Vector3(1, 0.85, 1).normalize();
  camera.position.copy(center).addScaledVector(dir, dist);
  camera.near = Math.max(0.1, dist / 100);
  camera.far = dist * 20;
  camera.lookAt(center);
  camera.updateProjectionMatrix();
  return center;
}
