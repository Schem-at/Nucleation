import { chromium } from 'playwright';
import { readFileSync } from 'fs';
const file = process.argv[2];
const b64 = readFileSync(file).toString('base64');
const b = await chromium.launch();
const p = await b.newPage();
p.on('pageerror', e => console.log('PAGEERROR', String(e).slice(0,120)));
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
const out = await p.evaluate(async (b64) => {
  const log = [];
  const eng = await import('/engine/index.mjs');
  const zip = await fetch('/pack/mesher-pack.zip').then(r => r.arrayBuffer());
  const pack = eng.ResourcePack.fromBytes(new Uint8Array(zip));
  const cfg = eng.MeshConfig.create();
  const bytes = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
  const s = eng.Schematic.fromData(bytes);
  log.push('dims ' + JSON.stringify(s.tightDimensions()));
  try {
    const cm = eng.ChunkMeshResult.createWithSize(s, pack, cfg, 16);
    log.push('chunks ' + cm.chunkCount());
    const c = cm.chunkCoordinateAt(0);
    log.push('coord0 ' + JSON.stringify(c) + ' keys=' + Object.keys(c));
    const m = cm.getMesh(c.x ?? c[0], c.y ?? c[1], c.z ?? c[2]);
    log.push('glb len ' + (m.glbDataB64()?.length ?? 'none'));
  } catch (e) { log.push('CHUNKMESH FAILED: ' + e); }
  try {
    const scratch = eng.Schematic.create('chunk');
    scratch.copyRegion(s, 0,0,0, 15,15,15, 0,0,0, '[]');
    log.push('copied, dims ' + JSON.stringify(scratch.tightDimensions()));
    const m2 = eng.MeshResult.create(scratch, pack, cfg);
    log.push('scratch glb len ' + (m2.glbDataB64()?.length ?? 'none'));
  } catch (e) { log.push('SCRATCH FAILED: ' + e); }
  return log;
}, b64);
console.log(out.join('\n'));
await b.close();
