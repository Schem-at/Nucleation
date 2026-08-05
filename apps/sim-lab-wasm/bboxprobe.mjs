import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', process.argv[2]);
await p.waitForSelector('.status.ready', { timeout: 300000 });
console.log(await p.evaluate(() => {
  const w = window.simlab.world;
  let min=[1e9,1e9,1e9], max=[-1e9,-1e9,-1e9];
  w.group.updateMatrixWorld(true);
  w.group.traverse((m)=>{ if(!m.isMesh) return;
    m.geometry.computeBoundingBox();
    const bb=m.geometry.boundingBox.clone().applyMatrix4(m.matrixWorld);
    min=[Math.min(min[0],bb.min.x),Math.min(min[1],bb.min.y),Math.min(min[2],bb.min.z)];
    max=[Math.max(max[0],bb.max.x),Math.max(max[1],bb.max.y),Math.max(max[2],bb.max.z)];});
  return `dims=${w.dims}\nmesh bbox min=[${min.map(n=>n.toFixed(3))}] max=[${max.map(n=>n.toFixed(3))}]\n` +
    `if cells were [i,i+1]: expect min 0, max ${w.dims[0]}\n` +
    `if cells centred on i: expect min -0.5, max ${w.dims[0]-0.5}`;
}));
await b.close();
