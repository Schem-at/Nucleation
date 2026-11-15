// Run against `npm run dev -- --host 127.0.0.1 --port 8787` after preparing WASM.
// Uses a fresh headless browser profile; never touches the user's browser session.
import { chromium } from 'playwright';
import { writeFile, mkdir, readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';

const base = process.argv[2] || 'http://127.0.0.1:8787';
const output = process.argv[3] || '/tmp/atlas-entity-model-check';
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true,
  ...(process.env.CHROME_PATH ? { executablePath: process.env.CHROME_PATH } : {}),
  args: ['--enable-webgl', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
try {
  const page = await browser.newPage({ viewport: { width: 900, height: 640 } });
  await page.goto(base);
  const result = await page.evaluate(async () => {
    const THREE = await import('/node_modules/.vite/deps/three.js');
    const { minecartModelFor, minecartRotation } = await import('/src/entity_models.ts');
    const { loadMeshEnv, addDefaultLights, fitCamera } = await import('/src/mesher.ts');
    const { World } = await import('/src/world.ts');
    const { eng } = await loadMeshEnv();
    const ensure = (yes, text) => { if (!yes) throw new Error(text); };
    const kinds = ['minecart', 'chest_minecart', 'hopper_minecart', 'furnace_minecart', 'tnt_minecart', 'spawner_minecart', 'command_block_minecart'];
    const models = [];
    const scene = new THREE.Scene(); scene.background = new THREE.Color('#202925');
    addDefaultLights(scene);
    for (const [i, kind] of kinds.entries()) {
      const model = await minecartModelFor(`minecraft:${kind}`);
      let vertices = 0, textured = 0;
      model.traverse(part => {
        if (!part.isMesh) return;
        vertices += part.geometry.getAttribute('position').count;
        for (const mat of Array.isArray(part.material) ? part.material : [part.material]) if (mat.map) textured++;
      });
      ensure(vertices > 0 && textured > 0, `${kind}: no textured model`);
      const bounds = new THREE.Box3().setFromObject(model);
      ensure(Math.abs(bounds.min.y) < 0.00001, `${kind}: model not at feet origin`);
      models.push({ kind, vertices, textured, size: bounds.getSize(new THREE.Vector3()).toArray() });
      model.position.set(i % 4 * 2.2, 0, Math.floor(i / 4) * 2.2);
      scene.add(model);
    }
    ensure(models.slice(1).every(m => m.vertices > models[0].vertices), 'a cart payload has no visible geometry');
    for (const [yaw, expected] of [[0,[1,0,0]],[90,[0,0,1]],[180,[-1,0,0]],[270,[0,0,-1]]]) {
      const front = new THREE.Vector3(0,0,1).applyAxisAngle(new THREE.Vector3(0,1,0), minecartRotation(yaw));
      ensure(front.distanceTo(new THREE.Vector3(...expected)) < 0.00001, `yaw ${yaw} points wrong way`);
    }
    const schematic = eng.Schematic.create('moving-model-check');
    schematic.setSourceDataVersion(4671);
    for (let x=0; x<12; x++) {
      schematic.setBlockFromString(x,0,0,'minecraft:stone');
      schematic.setBlockFromString(x,1,0,'minecraft:rail[shape=east_west,waterlogged=false]');
    }
    schematic.addEntityFromSnbt('{id:"minecraft:minecart",Pos:[1.5d,1.0625d,0.5d],Motion:[0.2d,0.0d,0.0d],Rotation:[0.0f,0.0f]}');
    const bytes = Uint8Array.from(atob(schematic.toSchematicB64()), c => c.charCodeAt(0));
    const world = await World.load(bytes);
    ensure(world.sim, 'fixture failed simulation import');
    schematic.removeEntity(0);
    const bare = await World.load(Uint8Array.from(atob(schematic.toSchematicB64()), c => c.charCodeAt(0)));
    const chunkVertices = w => {
      let count = 0;
      for (const chunk of w.chunks.values()) chunk.traverse(p => {
        if (p.isMesh) count += p.geometry.getAttribute('position').count;
      });
      return count;
    };
    ensure(chunkVertices(world) === chunkVertices(bare), 'cart baked into static chunks as a ghost');
    const get = () => JSON.parse(world.sim.itemEntitiesJson()).minecarts[0];
    const first = get();
    ensure(Number.isFinite(first.yaw), 'engine JSON missing finite yaw; rebuild WASM');
    const wait = async () => {
      for (let n=0;n<100;n++) {
        if (world.entityMeshes.get(first.id)?.userData.sharedEntityModel) return;
        await new Promise(r=>setTimeout(r,20));
      }
      throw new Error('live cart remained a hitbox placeholder');
    };
    await wait();
    ensure(world.entityMeshes.size === 1, 'expected one live cart model');
    world.sim.run(8); world.applyChanges([]);
    const after = get(); const mesh = world.entityMeshes.get(first.id);
    ensure(after.pos[0] > first.pos[0] + 0.5, 'cart did not actually move');
    ensure(Math.abs(mesh.position.x-after.pos[0]) < 1e-9, 'live model did not follow engine position');
    ensure(Math.abs(mesh.rotation.y-minecartRotation(after.yaw)) < 1e-9, 'model yaw differs from engine yaw');
    world.clearEntities();
    ensure(world.entityCount() === 0, 'clear/reset leaves stale cart model');
    world.syncEntities(); await wait();
    ensure(world.entityCount() === 1, 'model cannot be recreated after reset');
    // Shared geometry must survive removal of another instance.
    let recreatedVertices = 0;
    world.entityMeshes.get(first.id).traverse(part => { if(part.isMesh) recreatedVertices += part.geometry.getAttribute('position').count; });
    ensure(recreatedVertices > 0, 'model cache invalidated');
    const renderer = new THREE.WebGLRenderer({ antialias: false, preserveDrawingBuffer: true });
    renderer.setSize(900,640); renderer.setPixelRatio(1);
    const camera = new THREE.PerspectiveCamera(42,900/640,0.1,100);
    fitCamera(camera, new THREE.Box3().setFromObject(scene),1.3);
    document.body.replaceChildren(renderer.domElement);
    renderer.render(scene,camera);
    return { status:'pass', models, yaw_cases:4, live_model_count:world.entityCount(), first, after,
      assertions:['seven actual textured GLBs','feet-centred models','four yaw quadrants','no static ghost cart','real cart moves','visual follows position/yaw','clear/recreate retains shared mesh'],
      scope:'Rendering/API regression, not a new vanilla mechanics parity claim.' };
  });
  result.browser = await browser.version();
  result.engine = await (await page.request.get(`${base}/engine/build-info.json`)).json();
  result.sources_sha256 = {};
  for (const file of ['src/entity_models.ts', 'src/world.ts']) {
    result.sources_sha256[file] = createHash('sha256').update(await readFile(new URL(file, import.meta.url))).digest('hex');
  }
  await page.screenshot({ path: path.join(output, 'minecart-models.png') });
  await writeFile(path.join(output, 'result.json'), JSON.stringify(result,null,2)+'\n');
  console.log(JSON.stringify(result));
} finally { await browser.close(); }
