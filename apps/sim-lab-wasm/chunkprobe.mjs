import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', process.argv[2]);
await p.waitForSelector('.status.ready', { timeout: 300000 });
console.log(await p.evaluate(() => {
  const w = window.simlab.world;
  const span = (applyChildPos) => {
    let min=[1e9,1e9,1e9], max=[-1e9,-1e9,-1e9];
    w.group.children.forEach((child) => {
      const saved = child.position.clone();
      if (!applyChildPos) child.position.set(0,0,0);
      child.updateMatrixWorld(true);
      child.traverse((m) => {
        if (!m.isMesh) return;
        m.geometry.computeBoundingBox();
        const bb = m.geometry.boundingBox.clone().applyMatrix4(m.matrixWorld);
        min = [Math.min(min[0],bb.min.x),Math.min(min[1],bb.min.y),Math.min(min[2],bb.min.z)];
        max = [Math.max(max[0],bb.max.x),Math.max(max[1],bb.max.y),Math.max(max[2],bb.max.z)];
      });
      child.position.copy(saved); child.updateMatrixWorld(true);
    });
    return `min=[${min.map(n=>n.toFixed(0))}] max=[${max.map(n=>n.toFixed(0))}]`;
  };
  return `build is 175x53x31\nWITH child offsets:    ${span(true)}\nWITHOUT child offsets: ${span(false)}`;
}));
await b.close();
