import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
const errs=[]; p.on('pageerror',e=>errs.push(String(e).slice(0,140)));
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', process.argv[2]);
await p.waitForSelector('.status.ready', { timeout: 300000 });

// 1. step-N: the +10 button and the custom field
await p.click('button:has-text("+10")'); await p.waitForTimeout(200);
console.log('after +10 →', (await p.textContent('.tick')).trim(), '|', (await p.textContent('.info'))?.trim());
await p.fill('.stepn', '50');
await p.click('button:has-text("›")'); await p.waitForTimeout(300);
console.log('after ›50 →', (await p.textContent('.tick')).trim(), '|', (await p.textContent('.info'))?.trim());

// 2. outline follows the crosshair: aim the camera at a known block
const outline = await p.evaluate(() => {
  const { world, player, scene } = window.simlab;
  // find an interactive block and look straight at it from 3 blocks away
  const [dx,dy,dz]=world.dims; let t=null;
  outer: for (let x=0;x<dx;x++) for (let y=0;y<dy;y++) for (let z=0;z<dz;z++)
    if (/lever|button|note_block/.test(world.blockAt(x,y,z))) { t=[x,y,z]; break outer; }
  if (!t) return 'no interactive block';
  player.camera.position.set(t[0]+0.5, t[1]+0.5, t[2]+6.5);
  player.yaw = Math.PI; player.pitch = 0;   // face -Z
  player.update(0.001);
  const hit = player.pick((x,y,z)=>world.isSolid(x,y,z));
  const box = scene.children.find(o=>o.isLineSegments);
  return { target: t.join(','), hit: hit?.pos?.join(','), boxVisible: box?.visible,
           boxAt: box?.position?.toArray().map(n=>n-0.5).join(','),
           colour: '#'+box?.material?.color?.getHexString() };
});
console.log('outline:', JSON.stringify(outline));

// 3. click feedback: flash mesh + info text
const click = await p.evaluate(() => {
  const { world, player, scene } = window.simlab;
  const hit = player.pick((x,y,z)=>world.isSolid(x,y,z));
  if (!hit) return 'nothing targeted';
  const before = world.blockAt(...hit.pos);
  world.sim.useBlock(...hit.pos); world.applyChanges(world.drainChanges());
  return `${before.replace('minecraft:','')} → ${world.blockAt(...hit.pos).replace('minecraft:','')}`;
});
console.log('interaction:', click);
await p.mouse.click(400, 300);   // real click through the UI path
await p.waitForTimeout(150);
console.log('info after real click:', (await p.textContent('.info'))?.trim());
console.log('errors:', errs.length?errs.slice(0,2):'none');
await b.close();
