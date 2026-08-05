import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', process.argv[2]);
await p.waitForSelector('.status.ready', { timeout: 300000 });
console.log(await p.evaluate(() => {
  const { world, player } = window.simlab;
  const [dx,dy,dz]=world.dims; const out=[]; let checked=0, ok=0;
  // Cast along +Z from outside, at several (x,y) lines. The pick must equal
  // the first solid cell on that line — that is what proves the half-block
  // shift is right.
  for (let x=0; x<dx && checked<8; x+=3)
    for (let y=0; y<dy && checked<8; y+=3) {
      let expect=null;
      for (let z=0; z<dz; z++) if (world.isSolid(x,y,z)) { expect=z; break; }
      if (expect===null) continue;
      player.camera.position.set(x, y, -8);
      player.yaw = 0; player.pitch = 0;              // face +Z
      const hit = player.pick((a,b2,c)=>world.isSolid(a,b2,c));
      checked++;
      const got = hit ? hit.pos.join(',') : 'none';
      const want = `${x},${y},${expect}`;
      if (got===want) ok++; else out.push(`  line x=${x} y=${y}: want ${want} got ${got}`);
    }
  return `ray alignment: ${ok}/${checked} lines hit the exact first solid block` +
         (out.length ? '\n'+out.join('\n') : '');
}));
await b.close();
