// Replay an external vanilla protocol in the real Atlas World and verify its
// endpoint plus render mirror. This does not assert a successful circuit output.
// CHROME_PATH=... node verify-circuit-endpoint.mjs SOURCE CAPTURE[.gz] OUTPUT.json
import { chromium } from 'playwright';
import { readFile, writeFile } from 'node:fs/promises';
import { gunzipSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const source = await readFile(process.argv[2]), raw = await readFile(process.argv[3]);
const capture = JSON.parse(process.argv[3].endsWith('.gz') ? gunzipSync(raw) : raw);
assert.equal(createHash('sha256').update(source).digest('hex'), capture.source.sha256);
assert.equal(capture.snapshots.at(-1).tick, capture.capture_ticks);
const browser = await chromium.launch({headless:true,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader'],
  ...(process.env.CHROME_PATH ? {executablePath:process.env.CHROME_PATH} : {})});
try {
  const page = await browser.newPage();
  const errors = []; page.on('pageerror',e => errors.push(e.message));
  await page.goto('http://127.0.0.1:8767/sim-lab/?archive=a16c594465e5664c1a29c4b87b66745916dc99b377d68be62a960847674cd8a0&embed=atlas');
  await page.locator('.status.ready').waitFor({timeout:90000});
  const result = await page.evaluate(async ({bytes, ticks, actions, expected, watch, seed}) => {
    const w = await window.simlab.world.constructor.load(new Uint8Array(bytes));
    w.sim.setRngSeed(BigInt(seed));
    let firstRest = null;
    for (let tick=1;tick<=ticks;tick++) {
      for (const a of actions.filter(a => a.tick === tick)) {
        if (a.type === 'use_note_block' || a.type === 'pull_lever') w.sim.useBlock(...a.pos);
        else if (a.type === 'set_block') w.sim.placeBlock(...a.pos,a.state);
        else throw Error(`Unsupported action ${a.type}`);
      }
      w.sim.step();
      if (Number(w.sim.tickCount()) !== tick) throw Error(`Simulation stopped at tick ${tick}`);
      if (tick > (actions.at(-1)?.tick ?? 0) && w.sim.isQuiescent() && firstRest === null) firstRest = tick;
      if (tick%64===0 || tick===ticks) {
        w.applyChanges(w.drainChanges());w.animate(tick);await w.flush();
      }
    }
    const normalize = rows => JSON.stringify(rows.map(row => JSON.stringify(row)).sort());
    const rows = JSON.parse(w.sim.worldSnapshotJson());
    const watched = rows.filter(b => b.pos.every((n,i) => n>=watch.min[i] && n<=watch.max[i]));
    if (normalize(watched.map(b => [...b.pos,b.state])) !== normalize(expected)) throw Error('Atlas endpoint differs from vanilla');
    const mirror=[],[x0,y0,z0,x1,y1,z1]=JSON.parse(w.schem.boundingBoxJson());
    for(let x=x0;x<=x1;x++)for(let y=y0;y<=y1;y++)for(let z=z0;z<=z1;z++){
      const state=w.schem.getBlockString(x,y,z);if(state&&state!=='minecraft:air')mirror.push([x,y,z,state]);
    }
    const solids=rows.filter(b=>b.state!=='minecraft:air'&&!b.state.startsWith('minecraft:moving_piston')).map(b=>[...b.pos,b.state]);
    if(normalize(mirror)!==normalize(solids))throw Error('Atlas render mirror differs from simulation');
    return {ticks,first_quiescent_tick:firstRest,quiescent:w.sim.isQuiescent(),
      honey:rows.filter(b=>b.state==='minecraft:honey_block').map(b=>b.pos),
      endpoint_matches_vanilla:true,render_mirror_matches_simulation:true,
      origin:w.origin,render_stats:w.renderStats,remaining_flights:w.flights.size};
  }, {bytes:[...source],ticks:capture.capture_ticks,actions:capture.actions,expected:capture.snapshots.at(-1).blocks,watch:capture.watch,seed:capture.rng_seed});
  assert.deepEqual(errors,[]);
  await writeFile(process.argv[4],JSON.stringify({case:capture.case,source_sha256:capture.source.sha256,
    browser:browser.version(),engine:await (await page.request.get('http://127.0.0.1:8767/api/engine')).json(),...result,errors},null,2)+'\n');
  console.log(JSON.stringify({case:capture.case,...result,errors}));
} finally {await browser.close();}
