import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { loadMeshEnv } from "./mesher";

const payloads: Record<string, string | null> = {
  "minecraft:minecart": null,
  "minecraft:chest_minecart": "minecraft:chest[facing=south,type=single,waterlogged=false]",
  "minecraft:hopper_minecart": "minecraft:hopper[facing=down,enabled=true]",
  "minecraft:furnace_minecart": "minecraft:furnace[facing=south,lit=false]",
  "minecraft:tnt_minecart": "minecraft:tnt[unstable=false]",
  "minecraft:spawner_minecart": "minecraft:spawner",
  "minecraft:command_block_minecart": "minecraft:command_block[facing=south,conditional=false]",
};
const templates = new Map<string, Promise<THREE.Group>>();
const loader = new GLTFLoader();

export function hasMinecartModel(kind: string): boolean {
  return Object.hasOwn(payloads, kind);
}

/** The engine's cart yaw is atan2(dz, dx), not the humanoid Minecraft yaw.
 * Our mesher's canonical cart points south (+Z), so Ry(π/2-yaw) follows it. */
export function minecartRotation(yaw: number): number {
  return Math.PI / 2 - yaw * Math.PI / 180;
}

async function loadTemplate(kind: string): Promise<THREE.Group> {
  const { eng, pack, cfg } = await loadMeshEnv();
  const mesh = async (block: string | null): Promise<THREE.Group> => {
    const scratch = eng.Schematic.create("live-minecart-model");
    try {
      if (block) scratch.setBlockFromString(0, 0, 0, block);
      else scratch.addEntityFromSnbt('{id:"minecraft:minecart",Pos:[0.0d,0.0d,0.0d],Rotation:[0.0f,0.0f]}');
      const result = eng.MeshResult.create(scratch, pack, cfg);
      const bytes = Uint8Array.from(atob(result.glbDataB64()), c => c.charCodeAt(0));
      const scene = (await loader.parseAsync(bytes.buffer, "")).scene;
      let vertices = 0;
      scene.traverse(object => {
        const part = object as THREE.Mesh;
        if (!part.isMesh) return;
        vertices += part.geometry.getAttribute("position")?.count ?? 0;
        const materials = Array.isArray(part.material) ? part.material : [part.material];
        for (const material of materials) {
          const map = (material as THREE.MeshStandardMaterial).map;
          if (map) {
            map.magFilter = THREE.NearestFilter;
            map.minFilter = THREE.NearestFilter;
            map.generateMipmaps = false;
            map.colorSpace = THREE.SRGBColorSpace;
          }
        }
      });
      if (!vertices) throw new Error(`No mesh vertices for ${kind}`);
      // The static mesher uses block-centred cells. Move this template to
      // feet-centre once; runtime instances use the exact engine position.
      const box = new THREE.Box3().setFromObject(scene);
      const center = box.getCenter(new THREE.Vector3());
      scene.position.set(-center.x, -box.min.y, -center.z);
      const group = new THREE.Group();
      group.add(scene);
      return group;
    } finally {
      scratch.clearContents();
    }
  };
  const hull = await mesh(null);
  const payload = payloads[kind];
  if (payload) {
    const content = await mesh(payload);
    content.scale.setScalar(0.75);
    content.position.y = 0.25;
    hull.add(content);
  }
  hull.userData.sharedEntityModel = true;
  return hull;
}

/** A bounded seven-kind cache. Clones share immutable mesh/material resources;
 * removing an instance must not dispose the shared template's geometry. */
export function minecartModelFor(kind: string): Promise<THREE.Group> {
  if (!hasMinecartModel(kind)) return Promise.reject(new Error(`Unsupported cart model ${kind}`));
  let pending = templates.get(kind);
  if (!pending) {
    pending = loadTemplate(kind);
    templates.set(kind, pending);
  }
  return pending.then(template => template.clone(true));
}
