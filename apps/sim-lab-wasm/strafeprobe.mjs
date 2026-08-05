import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', process.argv[2]);
await p.waitForSelector('.status.ready', { timeout: 300000 });
console.log(await p.evaluate(() => {
  const pl = window.simlab.player;
  const out = [];
  // Face +X (yaw = pi/2): forward = (sin, 0, cos) = (1,0,0).
  pl.yaw = Math.PI/2; pl.pitch = 0;
  const f = pl.direction();
  out.push(`facing [${f.x.toFixed(2)},${f.y.toFixed(2)},${f.z.toFixed(2)}] (+X)`);
  const before = pl.camera.position.clone();
  // press D for one 100ms frame
  pl.keys?.add?.('KeyD');
  window.dispatchEvent(new KeyboardEvent('keydown', {code:'KeyD'}));
  pl.update(0.1);
  window.dispatchEvent(new KeyboardEvent('keyup', {code:'KeyD'}));
  const d = pl.camera.position.clone().sub(before);
  out.push(`D moved [${d.x.toFixed(2)},${d.y.toFixed(2)},${d.z.toFixed(2)}]`);
  out.push(`expected: facing +X, right-hand strafe is +Z → ${d.z > 0.1 ? 'CORRECT' : 'INVERTED'}`);
  return out.join('\n');
}));
await b.close();
