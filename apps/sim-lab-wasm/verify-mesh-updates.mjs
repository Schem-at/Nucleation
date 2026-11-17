// Real WASM + GLB loading. Optional baseline module compares avoided rebuilds.
import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';
const base = process.argv[2] || 'http://127.0.0.1:8767';
const output = process.argv[3] || '/tmp/atlas-mesh-updates';
await mkdir(output, {recursive:true});
const browser = await chromium.launch({headless:true,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader'],...(process.env.CHROME_PATH?{executablePath:process.env.CHROME_PATH}:{})});
try {
  const page = await browser.newPage();
  const errors=[]; page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`${base}/sim-lab/?archive=a16c594465e5664c1a29c4b87b66745916dc99b377d68be62a960847674cd8a0&embed=atlas`);
  await page.locator('.status.ready').waitFor({timeout:90000});
  const result = await page.evaluate(async baselinePath => {
    const initial=window.simlab.world, eng=initial.eng, checks=[];
    const assert=(ok,message)=>{if(!ok)throw Error(message);checks.push(message);};
    const sc=eng.Schematic.create('mesh invalidation');sc.setMcVersion(4671);
    sc.setBlockFromString(4,3,4,'minecraft:stone');
    sc.setBlockFromString(20,3,4,'minecraft:stone');
    sc.setBlockFromString(7,3,4,'minecraft:note_block[instrument=harp,note=0,powered=false]');
    sc.setBlockFromString(9,3,4,'minecraft:scaffolding[bottom=false,distance=0,waterlogged=false]');
    sc.setBlockFromString(0,0,0,'minecraft:air');
    const bytes=Uint8Array.from(atob(sc.toSnapshotB64()),c=>c.charCodeAt(0));
    const World=initial.constructor;
    const c=(pos,to)=>({pos,to});
    const stone='minecraft:stone', alt='minecraft:cobblestone';
    const w=await World.load(bytes), original=w.sim.worldSnapshotJson();
    for(let i=0;i<40;i++)w.applyChanges([]);
    assert(await w.flush()===0,'idle/entity-only batches do not remesh');
    const note=n=>`minecraft:note_block[instrument=harp,note=${n},powered=false]`;
    const scaffold=d=>`minecraft:scaffolding[bottom=false,distance=${d},waterlogged=false]`;
    const meshOf=state=>{const s=eng.Schematic.create('identity');s.setBlockFromString(0,0,0,state);return eng.MeshResult.create(s,w.pack,w.cfg).glbDataB64();};
    assert(meshOf(note(0))===meshOf(note(12)),'note changes produce identical real GLB');
    assert(meshOf(scaffold(0))===meshOf(scaffold(6)),'scaffold distance changes produce identical real GLB');
    const cases=[
      ['batch reversal',[c([4,3,4],alt),c([4,3,4],stone)]],
      ['note pitch',[c([7,3,4],note(12))]],
      ['support distance',[c([9,3,4],scaffold(6))]],
    ];
    const comparisons=[];
    const Baseline=baselinePath?(await import(baselinePath)).World:null;
    for(const [name,changes] of cases){
      const candidate=await World.load(bytes);
      candidate.applyChanges(changes);const count=await candidate.flush();
      assert(count===0,`${name}: no unnecessary rebuild`);
      let baseline=null;
      if(Baseline){const old=await Baseline.load(bytes);old.applyChanges(changes);baseline=await old.flush();
        assert(old.schem.getBlockString(...changes[0].pos)===candidate.schem.getBlockString(...changes[0].pos),`${name}: mirror matches baseline`);}
      comparisons.push({case:name,baseline_chunks:baseline,candidate_chunks:count});
    }
    w.applyChanges([c([4,3,4],alt)]);w.applyChanges([c([4,3,4],stone)]);
    assert(await w.flush()===0,'reversal before mesh start cancels pending work');
    w.applyChanges([c([-1,3,4],stone)]);
    assert(w.dirty.has('-1,0,0')&&w.dirty.has('0,0,0'),'negative growth includes seam neighbours');
    assert(await w.flush()===2,'changed geometry really rebuilds both affected chunks');
    assert(w.chunks.has('-1,0,0'),'negative chunk is actually installed');
    // Hold chunk zero after capturing its geometry, then mutate both chunks.
    w.applyChanges([c([4,3,4],alt),c([20,3,4],alt)]);
    let release;const gate=new Promise(r=>release=r),seen=[];
    const remesh=w.remeshChunk.bind(w);
    w.remeshChunk=async(x,y,z)=>{seen.push(`${x},${y},${z}`);await remesh(x,y,z);if(x===0)await gate;};
    const flushing=w.flush();assert(w.flush()===flushing,'concurrent callers share one flush');
    w.applyChanges([c([4,3,4],stone),c([20,3,4],stone)]);
    release();await flushing;
    assert(!seen.includes('1,0,0'),'queued chunk cancelled before its turn is skipped');
    assert(w.dirty.has('0,0,0'),'changes after a mesh snapshot remain pending');
    w.remeshChunk=remesh;assert(await w.flush()===1,'only the stale in-flight snapshot is rebuilt');
    assert(await w.flush()===0,'finished queue stays clean');
    w.applyChanges([c([4,3,4],alt),c([20,3,4],alt)]);
    let releaseAgain;const gateAgain=new Promise(r=>releaseAgain=r),snapshots=[];
    w.remeshChunk=async(x,y,z)=>{snapshots.push([x,w.schem.getBlockString(20,3,4)]);await remesh(x,y,z);if(x===0)await gateAgain;};
    const next=w.flush();w.applyChanges([c([20,3,4],'minecraft:glass')]);releaseAgain();
    assert(await next===2,'each queued chunk is built once');
    assert(snapshots.find(row=>row[0]===1)?.[1]==='minecraft:glass','later chunk captures the latest state');
    assert(!w.needsRemesh(),'updates included in a later snapshot do not schedule duplicate work');
    w.remeshChunk=remesh;

    assert(w.sim.worldSnapshotJson()===original,'render-only changes never mutate engine state');
    return {checks,comparisons,renderStats:w.renderStats};
  }, process.argv[4] || null);
  if(errors.length)throw Error(errors.join('\n'));
  await writeFile(`${output}/result.json`,JSON.stringify({...result,errors,browser:browser.version()},null,2)+'\n');
  console.log(JSON.stringify({assertions:result.checks.length,comparisons:result.comparisons,renderStats:result.renderStats,errors}));
} finally {await browser.close();}
