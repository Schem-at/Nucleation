import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
const errs=[]; p.on('pageerror', e=>errs.push(String(e).slice(0,120)));
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', process.argv[2]);
await p.waitForSelector('.status.ready', { timeout: 300000 });
const span = () => p.evaluate(() => {
  const w = window.simlab.world;
  let min=[1e9,1e9,1e9], max=[-1e9,-1e9,-1e9];
  w.group.updateMatrixWorld(true);
  w.group.traverse((m) => { if(!m.isMesh) return;
    m.geometry.computeBoundingBox();
    const bb = m.geometry.boundingBox.clone().applyMatrix4(m.matrixWorld);
    min=[Math.min(min[0],bb.min.x),Math.min(min[1],bb.min.y),Math.min(min[2],bb.min.z)];
    max=[Math.max(max[0],bb.max.x),Math.max(max[1],bb.max.y),Math.max(max[2],bb.max.z)];});
  return `min=[${min.map(n=>n.toFixed(0))}] max=[${max.map(n=>n.toFixed(0))}] children=${w.group.children.length}`;
});
console.log('before:', await span());
// drive real change: use an interactive block, run ticks, flush the re-mesh
console.log('used:', await p.evaluate(async () => {
  const w = window.simlab.world; const [dx,dy,dz]=w.dims;
  for (let x=0;x<dx;x++) for (let y=0;y<dy;y++) for (let z=0;z<dz;z++) {
    if (/lever|button|note_block/.test(w.blockAt(x,y,z))) {
      w.sim.useBlock(x,y,z); w.applyChanges(w.drainChanges());
      for (let i=0;i<20;i++){ w.sim.step(); w.applyChanges(w.drainChanges()); }
      const n = await w.flush();
      return `${x},${y},${z} → re-meshed ${n} chunk(s)`;
    }
  }
  return 'none';
}));
console.log('after: ', await span());
console.log('errors:', errs.length?errs.slice(0,2):'none');
await b.close();
