// Count real chunk rebuilds for identical simulation batches. Construction and
// warm-up are excluded. These are work counts, not a browser FPS benchmark.
// Usage: CHROME_PATH=... node profile-mesh-updates.mjs SOURCE hipster|note [BASELINE_MODULE]
import { chromium } from 'playwright';
import { readFile, writeFile } from 'node:fs/promises';
const bytes=[...await readFile(process.argv[2])], scenario=process.argv[3];
if(!['hipster','note'].includes(scenario))throw Error('Specify hipster or note');
const browser=await chromium.launch({headless:true,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader'],
  ...(process.env.CHROME_PATH?{executablePath:process.env.CHROME_PATH}:{})});
try {
  const page=await browser.newPage();
  await page.goto('http://127.0.0.1:8767/sim-lab/?archive=a16c594465e5664c1a29c4b87b66745916dc99b377d68be62a960847674cd8a0&embed=atlas');
  await page.locator('.status.ready').waitFor({timeout:90000});
  const result=await page.evaluate(async({bytes,scenario,baseline})=>{
    const constructors=[['candidate',window.simlab.world.constructor]];
    if(baseline)constructors.unshift(['baseline',(await import(baseline)).World]);
    const results=[];
    for(const [variant,World]of constructors){
      const w=await World.load(new Uint8Array(bytes));
      const ticks=scenario==='hipster'?320:160;
      let writes=0,chunks=0,records=0,maxFlights=0;
      const set=w.schem.setBlockFromString.bind(w.schem),remesh=w.remeshChunk.bind(w);
      w.schem.setBlockFromString=(...args)=>{writes++;return set(...args);};
      w.remeshChunk=(...args)=>{chunks++;return remesh(...args);};
      for(let tick=1;tick<=ticks;tick++){
        if(scenario==='hipster'&&(tick===10||tick===170))w.sim.useBlock(12,7,13);
        if(scenario==='note'&&tick===21)w.sim.useBlock(0,3,0);
        w.sim.step();
        if(tick%8===0||tick===ticks){
          const changes=w.drainChanges();records+=changes.length;w.applyChanges(changes);
          w.animate(Number(w.sim.tickCount()));await w.flush();
          maxFlights=Math.max(maxFlights,w.flights.size);
        }
      }
      const mirror=[],[x0,y0,z0,x1,y1,z1]=JSON.parse(w.schem.boundingBoxJson());
      for(let x=x0;x<=x1;x++)for(let y=y0;y<=y1;y++)for(let z=z0;z<=z1;z++){
        const state=w.schem.getBlockString(x,y,z);
        if(state&&state!=='minecraft:air')mirror.push([x,y,z,state]);
      }
      const hash=async text=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text)))].map(n=>n.toString(16).padStart(2,'0')).join('');
      const mirrorHash=await hash(JSON.stringify(mirror));
      const expectedMirror=JSON.parse(w.sim.worldSnapshotJson())
        .filter(b=>b.state!=='minecraft:air'&&!b.state.startsWith('minecraft:moving_piston'))
        .map(b=>[...b.pos,b.state]);
      const normalize=rows=>JSON.stringify(rows.map(row=>JSON.stringify(row)).sort());
      if(normalize(mirror)!==normalize(expectedMirror))throw Error('Render mirror differs from solid simulation blocks');
      results.push({variant,ticks,batch_ticks:8,change_records:records,mirror_writes:writes,chunk_rebuilds:chunks,
        max_rendered_piston_flights:maxFlights,render_mirror_matches_simulation:true,
        render_mirror_sha256:mirrorHash,final_state_sha256:await hash(w.sim.worldSnapshotJson())});
    }
    if(results.length===2&&results[0].final_state_sha256!==results[1].final_state_sha256)throw Error('Engine states diverged');
    if(results.length===2&&results[0].render_mirror_sha256!==results[1].render_mirror_sha256)throw Error('Render mirrors diverged');
    return {scenario,results};
  },{bytes,scenario,baseline:process.argv[4]||null});
  await writeFile(`/tmp/atlas-mesh-${scenario}.json`,JSON.stringify(result,null,2)+'\n');
  console.log(JSON.stringify(result));
}finally{await browser.close();}
