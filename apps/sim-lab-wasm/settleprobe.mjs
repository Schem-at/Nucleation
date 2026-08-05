import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
const errs=[]; p.on('pageerror',e=>errs.push(String(e).slice(0,120)));
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', process.argv[2]);
await p.waitForSelector('.status.ready', { timeout: 300000 });

// default should be in-world: stepping must be quiet on a build at rest
await p.click('button:has-text("+10")'); await p.waitForTimeout(300);
console.log('in-world, +10 ticks :', (await p.textContent('.info'))?.trim());

// switch to placement and step again — this SHOULD churn
await p.selectOption('.settle select', 'placement');
await p.waitForSelector('.status.ready', { timeout: 300000 });
await p.click('button:has-text("+10")'); await p.waitForTimeout(300);
console.log('placement, +10 ticks:', (await p.textContent('.info'))?.trim());

// outline alignment: stand exactly on a known block's centre line
console.log(await p.evaluate(() => {
  const { world, player } = window.simlab;
  const [dx,dy,dz]=world.dims;
  for (let x=0;x<dx;x++) for (let y=0;y<dy;y++) for (let z=0;z<dz;z++)
    if (/note_block|lever|button/.test(world.blockAt(x,y,z))) {
      player.camera.position.set(x, y, z + 3);  // dead centre, 3 blocks away
      player.yaw = Math.PI; player.pitch = 0;
      const hit = player.pick((a,b2,c)=>world.isSolid(a,b2,c));
      return `aim at ${x},${y},${z} → pick ${hit? hit.pos.join(',') : 'none'} ` +
             `(${hit? world.blockAt(...hit.pos).replace('minecraft:','').slice(0,28):''})`;
    }
  return 'none';
}));
await p.waitForTimeout(300);
console.log('outline:', await p.evaluate(() => {
  const box = window.simlab.scene.children.find(o=>o.isLineSegments);
  return `visible=${box.visible} at=[${box.position.toArray()}]`;
}));
console.log('HUD:', (await p.textContent('.target'))?.trim());
console.log('errors:', errs.length?errs.slice(0,2):'none');
await b.close();
