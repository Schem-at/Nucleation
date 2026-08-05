import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', process.argv[2]);
await p.waitForSelector('.status.ready', { timeout: 300000 });
// Put the camera right in front of an interactive block, facing it.
console.log(await p.evaluate(() => {
  const { world, player } = window.simlab;
  const [dx,dy,dz]=world.dims;
  for (let x=0;x<dx;x++) for (let y=0;y<dy;y++) for (let z=0;z<dz;z++)
    if (/lever|button|note_block/.test(world.blockAt(x,y,z))) {
      // stand 2 blocks away on +Z looking -Z, but only if that path is clear
      player.camera.position.set(x+0.5, y+0.5, z+2.5);
      player.yaw = Math.PI; player.pitch = 0;
      return `aimed at ${world.blockAt(x,y,z).replace('minecraft:','')} @${x},${y},${z}`;
    }
  return 'none';
}));
await p.waitForTimeout(400);           // let real frames run
console.log(await p.evaluate(() => {
  const { scene } = window.simlab;
  const box = scene.children.find(o=>o.isLineSegments);
  const flash = scene.children.find(o=>o.isMesh && o.material?.transparent);
  return `outline visible=${box.visible} at=[${box.position.toArray().map(n=>n-0.5)}] colour=#${box.material.color.getHexString()}`;
}));
console.log('HUD target:', (await p.textContent('.target'))?.trim());
await b.close();
