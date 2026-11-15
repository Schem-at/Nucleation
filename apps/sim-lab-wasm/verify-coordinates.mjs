// Real-WASM regression for saved-space -> local simulation/render coordinates.
import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';
const base = process.argv[2] || 'http://127.0.0.1:8767';
const output = process.argv[3] || '/tmp/simlab-coordinates';
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true,
  args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
  ...(process.env.CHROME_PATH ? { executablePath: process.env.CHROME_PATH } : {}) });
const errors = [], results = [];
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on('pageerror', e => errors.push(e.message));
  const cases = ['3f5adf997012ab95a0293fcdd16e4de749547f3739ffce921c09f63aa8f0af0b',
    '6130b77d5b51cb1cdfe0b86528c441d58a93d3c9db91178fe4e706ce2a1c92b1',
    '9ba0f2e5d68c9cee4f0c20bb0674e2792463c225f63de8c4b42384df515db5fd'];
  for (const id of [...cases, 'positive-multi-region', 'negative-multi-region']) {
    await page.goto(`${base}/sim-lab/?archive=${cases[1]}`);
    await page.locator('.status.ready').waitFor({ timeout: 90000 });
    const result = await page.evaluate(async ({ id, synthetic }) => {
      const { world: initial, player } = window.simlab;
      const { eng } = initial;
      const assert = (ok, message) => { if (!ok) throw new Error(`${id}: ${message}`); };
      let bytes;
      if (synthetic) {
        const s = eng.Schematic.create(id);
        s.setMcVersion(4189);
        s.setBlockFromString(0, 0, 0, 'minecraft:stone');
        s.setBlockFromString(0, 1, 0, 'minecraft:lever[face=floor,facing=north,powered=false]');
        s.setBlockFromString(1, 0, 0, 'minecraft:sticky_piston[facing=east,extended=false]');
        s.setBlockFromString(2, 0, 0, 'minecraft:stone');
        s.setBlockFromString(2, 0, 4, 'minecraft:chest[facing=north,type=single,waterlogged=false]');
        s.setBlockEntity(2, 0, 4, 'minecraft:chest', '{Items:[{Slot:0b,id:"minecraft:redstone",count:7}]}');
        s.setBlockFromString(3, 0, 4, 'minecraft:stone');
        s.setBlockFromString(3, 1, 4, 'minecraft:rail[shape=north_south,waterlogged=false]');
        s.addEntity('minecraft:minecart', 3.5, 1.1, 4.5, '{}');
        // Empty padding distinguishes full bounds from tight content bounds.
        s.setBlockFromString(-2, -1, -2, 'minecraft:air');
        s.createRegion('second');
        s.setBlockInRegion('second', 18, 0, 4, 'minecraft:stone');
        const offset = id.startsWith('positive') ? [161, 50, 272] : [-173, -29, -300];
        s.translateSchematic(...offset);
        // Snapshot retains the air padding; the Litematic exporter compacts it.
        bytes = Uint8Array.from(atob(s.toSnapshotB64()), c => c.charCodeAt(0));
      } else bytes = new Uint8Array(await (await fetch(`/api/file/${id}`)).arrayBuffer());
      const source = eng.Schematic.fromData(bytes);
      const before = source.boundingBoxJson();
      const bb = JSON.parse(before), origin = bb.slice(0, 3);
      const reference = eng.TickSimulation.fromSchematic(source, eng.TickSettleMode.InWorld, ...origin, '');
      const w = await initial.constructor.load(bytes);
      assert(w.sim, 'simulation loads');
      assert(JSON.stringify(w.origin) === JSON.stringify(origin), 'saved origin retained');
      assert(JSON.parse(w.schem.boundingBoxJson()).slice(0, 3).every(n => n === 0), 'mirror rebased to zero');
      assert(w.dims.every((n, i) => n === bb[i + 3] - bb[i] + 1), 'full bounds, including air margin');
      assert(w.schem.entityCount() === source.entityCount(), 'authored entities retained');
      assert(w.schem.mcVersion() === source.mcVersion(), 'source data version retained');
      assert(source.boundingBoxJson() === before, 'source remains unchanged');
      const mirror = (x,y,z) => { try { return w.schem.getBlockString(x,y,z); } catch { return 'minecraft:air'; } };
      let cellsChecked = 0;
      const checkCells = () => {
        const snapshot = JSON.parse(w.sim.worldSnapshotJson());
        const cells = new Map(snapshot.map(b => [b.pos.join(','), b.pos]));
        for (const b of JSON.parse(w.schem.getNonAirBlocksJson())) cells.set(`${b.x},${b.y},${b.z}`, [b.x,b.y,b.z]);
        for (const p of cells.values()) {
          const actual = w.blockAt(...p), expected = actual.startsWith('minecraft:moving_piston') ? 'minecraft:air' : actual;
          assert(mirror(...p) === expected, `mirror/sim at ${p}: ${mirror(...p)} vs ${expected}`);
          assert(w.isSolid(...p) === (expected !== 'minecraft:air'), `picking grid at ${p}`);
          cellsChecked++;
        }
        assert(w.sim.worldSnapshotJson() === reference.worldSnapshotJson(), 'state matches original-origin reference');
        assert(w.sim.itemEntitiesJson() === reference.itemEntitiesJson(), 'entities match original-origin reference');
      };
      checkCells();
      // Inspect actual GLB vertex coordinates, not only the schematic mirror.
      let vertices = 0;
      const checkGeometry = () => {
        for (const chunk of w.chunks.values()) {
          chunk.updateMatrixWorld(true);
          chunk.traverse(o => {
            if (!o.isMesh) return;
            const a = o.geometry.attributes.position, m = o.matrixWorld.elements;
            for (let i=0; i<a.count; i++) {
              const x=a.getX(i),y=a.getY(i),z=a.getZ(i);
              const p=[m[0]*x+m[4]*y+m[8]*z+m[12],m[1]*x+m[5]*y+m[9]*z+m[13],m[2]*x+m[6]*y+m[10]*z+m[14]];
              assert(p.every((v,j)=>v>=-2 && v<=w.dims[j]+2), `displaced chunk vertex ${p}`);
              vertices++;
            }
          });
        }
      };
      checkGeometry();
      assert(vertices>0,'drawable geometry');
      const snapshot = JSON.parse(w.sim.worldSnapshotJson());
      const control = snapshot.find(b => /minecraft:(lever|stone_button|note_block)(\[|$)/.test(b.state));
      let changes = 0, flights = 0;
      if (control) {
        const [x,y,z] = control.pos;
        // A vertical grid ray must hit the top occupied cell at this column.
        const top = Math.max(...snapshot.filter(b=>b.pos[0]===x&&b.pos[2]===z).map(b=>b.pos[1]));
        player.camera.position.set(x, top+3, z);player.pitch=-Math.PI/2;player.yaw=0;
        assert(JSON.stringify(player.pick((...p)=>w.isSolid(...p))?.pos)===JSON.stringify([x,top,z]), 'ray picks the local rendered column');
        w.sim.useBlock(x,y,z); reference.useBlock(x,y,z);
        const drain=()=>{const c=w.drainChanges();changes+=c.length;w.applyChanges(c);};
        drain();checkCells();
        for(let tick=0;tick<24;tick++) {
          w.sim.step();reference.step();drain();
          flights+=JSON.parse(w.sim.movingBlocksJson()).length;
          checkCells();
        }
        await w.flush();checkGeometry();
      }
      if (synthetic) {
        assert(source.tightBoundsMin().x > origin[0], 'air padding differs from tight bounds');
        assert(changes>0 && flights>0,'piston moves after interaction');
        assert(source.entityCount()>0,'entity translation exercised');
        assert(JSON.parse(w.schem.getAllBlockEntitiesJson()).length>0,'container metadata survives');
      }
      // Show the inspected world for the visual record.
      window.simlab.scene.remove(initial.group);window.simlab.scene.add(w.group);player.frame(w.dims);
      return { id, origin, dimensions:w.dims, cellsChecked, vertices, changes, flights, entities:w.schem.entityCount() };
    }, { id, synthetic: !cases.includes(id) });
    results.push(result);
    if (id === cases[1]) await page.screenshot({ path: `${output}/plot-door-aligned.png` });
    console.log(JSON.stringify(result));
  }
  if(errors.length)throw new Error(errors.join('\n'));
  await writeFile(`${output}/result.json`, JSON.stringify({status:'pass',results,errors,scope:'Coordinate alignment and differential simulation against the original source at its saved origin; not new vanilla certification.'},null,2)+'\n');
} finally { await browser.close(); }
