// Real WASM meshes and engine strokes; no synthetic substitute for the renderer.
import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';
const base = process.argv[2] || 'http://127.0.0.1:8767';
const output = process.argv[3] || '/tmp/skins-motion';
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true,
  args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
  ...(process.env.CHROME_PATH ? { executablePath: process.env.CHROME_PATH } : {}) });
const errors = [];
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on('pageerror', e => errors.push(e.message));
  await page.goto(`${base}/sim-lab/?archive=a16c594465e5664c1a29c4b87b66745916dc99b377d68be62a960847674cd8a0&embed=atlas`);
  await page.locator('.status.ready').waitFor({ timeout: 90000 });
  const result = await page.evaluate(async () => {
    const { world: initial } = window.simlab, { eng } = initial;
    const checks = [];
    const assert = (ok, message) => { if (!ok) throw new Error(message); checks.push(message); };
    assert(initial.skins.loaded === 2 && initial.skins.failed.length === 0, 'both saved creator skins loaded');
    assert(initial.pendingMeshes.size === 0, 'first activation does not race mesh warm-up');
    // Verify that the *mesher output* contains the saved skin's face pixels.
    // A successful CDN request alone would not establish that the head uses it.
    const heads = eng.Schematic.create('creator heads');
    heads.copyRegion(initial.schem, 12,7,16, 12,7,17, 0,0,0, '[]');
    const mesh = eng.MeshResult.create(heads, initial.pack, initial.cfg);
    const glb = Uint8Array.from(atob(mesh.glbDataB64()), c => c.charCodeAt(0));
    const jsonLength = new DataView(glb.buffer).getUint32(12, true);
    const document = JSON.parse(new TextDecoder().decode(glb.subarray(20, 20 + jsonLength)));
    const imageView = document.bufferViews[document.images[0].bufferView];
    const offset = 28 + jsonLength + (imageView.byteOffset || 0);
    const pixels = async bytes => {
      const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/png' }));
      const canvas = new OffscreenCanvas(bitmap.width, bitmap.height), context = canvas.getContext('2d');
      context.drawImage(bitmap, 0, 0);
      const result = context.getImageData(0, 0, bitmap.width, bitmap.height);
      bitmap.close(); return result;
    };
    const atlas = await pixels(glb.slice(offset, offset + imageView.byteLength));
    const urls = JSON.parse(eng.ResourcePack.playerSkinUrlsJson(heads));
    for (const url of urls) {
      const hash = new URL(url).pathname.split('/').pop();
      const skin = await pixels(await (await fetch(`/api/player-skin/${hash}`)).arrayBuffer());
      const matches = (x, y) => {
        for (let dy=0; dy<8; dy++) for (let dx=0; dx<8; dx++) for (let c=0; c<4; c++) {
          if (skin.data[((8+dy)*skin.width+8+dx)*4+c] !== atlas.data[((y+dy)*atlas.width+x+dx)*4+c]) return false;
        }
        return true;
      };
      let found = false;
      for (let y=0; y<=atlas.height-8 && !found; y++) for (let x=0; x<=atlas.width-8 && !found; x++) found = matches(x,y);
      assert(found, `saved skin face pixels present in actual mesh atlas: ${hash}`);
    }
    // Present the actual head-only geometry for visual review.
    const bytes = Uint8Array.from(atob(heads.toSnapshotB64()), c => c.charCodeAt(0));
    const portrait = await initial.constructor.load(bytes);
    window.simlab.scene.remove(initial.group); window.simlab.scene.add(portrait.group);
    window.simlab.player.frame(portrait.dims);
    const directions = { east:[1,0,0], west:[-1,0,0], up:[0,1,0], down:[0,-1,0], south:[0,0,1], north:[0,0,-1] };
    const strokes = [];
    for (const [name, direction] of Object.entries(directions)) {
      const sc = eng.Schematic.create(name); sc.setMcVersion(4189);
      sc.setBlockFromString(0,0,0,'minecraft:stone'); sc.setBlockFromString(5,5,5,'minecraft:air');
      sc.setBlockFromString(2,2,2,`minecraft:sticky_piston[facing=${name},extended=false]`);
      const load = direction.map(n=>n+2); sc.setBlockFromString(...load,'minecraft:stone');
      const power = name==='up'||name==='down' ? [1,2,2] : [2,1,2];
      const data = Uint8Array.from(atob(sc.toSnapshotB64()), c=>c.charCodeAt(0));
      const w = await initial.constructor.load(data);
      const reference = eng.TickSimulation.fromSchematic(sc,eng.TickSettleMode.InWorld,0,0,0,'');
      let disposed = 0, flights = 0;
      for (const proto of w.blockMeshes.values()) proto?.traverse(o=>o.geometry?.addEventListener('dispose',()=>disposed++));
      for (const powered of [true,false]) {
        w.sim.placeBlock(...power,powered?'minecraft:redstone_block':'minecraft:air');
        reference.placeBlock(...power,powered?'minecraft:redstone_block':'minecraft:air');
        let sawArm = false;
        for (let tick=0; tick<5; tick++) {
          w.sim.step(); reference.step(); w.applyChanges(w.drainChanges());
          assert(w.sim.worldSnapshotJson()===reference.worldSnapshotJson(), `${name}/${powered}/${tick}: rendering preserves engine state`);
          for (const f of w.flights.values()) {
            flights++;
            for (const phase of [0,.125,.25,.5,.75,1]) {
              w.animate(f.started + (f.lands-f.started)*phase);
              assert(f.object!==null,'flight geometry is ready on its first frame');
              assert(f.object.position.toArray().every((v,i)=>Math.abs(v-(f.from[i]+(f.to[i]-f.from[i])*phase))<1e-6), 'continuous stroke position');
              if (f.shortState) {
                sawArm = true;
                const short = f.extending ? phase<=.5 : phase>=.5;
                assert(f.drawnState===(short?f.shortState:f.state),'arm length follows extension/retraction phase');
              }
            }
          }
          await w.flush();
        }
        assert(sawArm, `${name}: ${powered?'extension':'retraction'} exercised`);
      }
      assert(disposed===0, `${name}: arm swaps retain shared geometry`);
      assert(w.flights.size===0 && w.handover.length===0, `${name}: landed meshes retire cleanly`);
      strokes.push({direction:name,flights,sharedGeometryDisposals:disposed});
    }
    return { status:'pass', skins:initial.skins, strokes, assertions:checks.length, checks };
  });
  await page.screenshot({ path: `${output}/creator-heads.png` });
  if(errors.length) throw new Error(errors.join('\n'));
  await writeFile(`${output}/result.json`,JSON.stringify({...result,errors,browser:browser.version()},null,2)+'\n');
  console.log(JSON.stringify({status:result.status,skins:result.skins,strokes:result.strokes,assertions:result.assertions,errors}));
} finally { await browser.close(); }
