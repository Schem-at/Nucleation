import { chromium } from 'playwright';
const [file, label] = process.argv.slice(2);
const b = await chromium.launch();
const p = await b.newPage();
const errs = [];
p.on('pageerror', e => errs.push(String(e).slice(0,160)));
p.on('console', m => { if (m.type()==='error') errs.push(m.text().slice(0,160)); });
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
const t0 = Date.now();
await p.setInputFiles('input[type=file]', file);
await p.waitForSelector('.status.ready, .status.error', { timeout: 300000 });
const load = ((Date.now()-t0)/1000).toFixed(1);
console.log(`${label}: ${(await p.textContent('.status')).trim()}  (${load}s)`);
// scene contents = chunks actually meshed
const meshed = await p.evaluate(() => {
  const w = window.simlab?.world; let meshes = 0;
  w?.group.traverse(o => { if (o.isMesh) meshes++; });
  return { children: w?.group.children.length ?? 0, meshes };
});
console.log(`  chunks in scene: ${meshed.children}, meshes: ${meshed.meshes}`);
// find an interactive block and use it, through the app's own world object
const used = await p.evaluate(() => {
  const w = window.simlab?.world; if (!w?.sim) return 'no sim';
  const [dx,dy,dz] = w.dims;
  for (let x=0;x<dx;x++) for (let y=0;y<dy;y++) for (let z=0;z<dz;z++) {
    const s = w.blockAt(x,y,z);
    if (/lever|button|note_block/.test(s)) {
      const before = s;
      w.sim.useBlock(x,y,z);
      const ch = w.drainChanges();
      w.applyChanges(ch);
      for (let i=0;i<6;i++){ w.sim.step(); w.applyChanges(w.drainChanges()); }
      return `used ${before} @${x},${y},${z} -> ${w.blockAt(x,y,z)}; ${ch.length} immediate change(s), tick ${w.sim.tickCount?.()}`;
    }
  }
  return 'no interactive block found';
});
console.log('  interact:', used);
await p.screenshot({ path: `/tmp/simlab-${label}.png` });
console.log('  errors:', errs.length ? errs.slice(0,3) : 'none');
await b.close();
