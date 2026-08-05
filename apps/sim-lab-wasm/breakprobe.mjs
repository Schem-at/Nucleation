import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
const errs=[]; p.on('pageerror',e=>errs.push(String(e).slice(0,140)));
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', process.argv[2]);
await p.waitForSelector('.status.ready', { timeout: 300000 });
console.log(await p.evaluate(async () => {
  const { world } = window.simlab; const out=[];
  const [dx,dy,dz]=world.dims;
  // break a plain structural block
  let t=null;
  outer: for (let x=0;x<dx;x++) for (let y=0;y<dy;y++) for (let z=0;z<dz;z++)
    if (/concrete|stone|planks/.test(world.blockAt(x,y,z))) { t=[x,y,z]; break outer; }
  const before = world.blockAt(...t);
  world.sim.placeBlock(t[0],t[1],t[2],'minecraft:air');
  const ch = world.drainChanges(); world.applyChanges(ch);
  out.push(`broke ${before.replace('minecraft:','')} @${t} → ${world.blockAt(...t).replace('minecraft:','')} (${ch.length} change(s))`);
  out.push(`solid set updated: ${world.isSolid(...t) ? 'NO — still solid (bug)' : 'yes, cell is now empty'}`);
  const n = await world.flush();
  out.push(`re-meshed ${n} chunk(s) after the break`);
  // break something load-bearing: a block under redstone dust should pop the dust
  let d=null;
  outer2: for (let x=0;x<dx;x++) for (let y=1;y<dy;y++) for (let z=0;z<dz;z++)
    if (/redstone_wire/.test(world.blockAt(x,y,z)) && world.isSolid(x,y-1,z)) { d=[x,y,z]; break outer2; }
  if (d) {
    const support = [d[0],d[1]-1,d[2]];
    world.sim.placeBlock(...support,'minecraft:air');
    const ch2 = world.drainChanges(); world.applyChanges(ch2);
    out.push(`broke support under dust @${d}: dust is now ${world.blockAt(...d).replace('minecraft:','')} (${ch2.length} change(s))`);
  }
  return out.join('\n');
}));
console.log('errors:', errs.length?errs.slice(0,2):'none');
await b.close();
