import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
const base=process.argv[2]||'http://127.0.0.1:8767', output=process.argv[3]||'/tmp/skin-retry-pause';
await mkdir(output,{recursive:true});
const browser=await chromium.launch({headless:true,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader'],
  ...(process.env.CHROME_PATH?{executablePath:process.env.CHROME_PATH}:{})});
const errors=[];
try {
  const page=await browser.newPage();page.on('pageerror',e=>errors.push(e.message));
  const entry='a16c594465e5664c1a29c4b87b66745916dc99b377d68be62a960847674cd8a0';
  const blocked='**/api/player-skin/494a19852adb51e91090b00b03b2ad6e00b07d8653074da1693762c8882c87c9';
  await page.route(blocked,route=>route.fulfill({status:503,body:'temporary test outage'}));
  await page.goto(`${base}/sim-lab/?archive=${entry}&embed=atlas`);
  await page.locator('.status.ready').waitFor({timeout:90000});
  assert.equal(await page.evaluate(()=>window.simlab.world.skins.failed.length),1);
  assert.match(await page.locator('.status').innerText(),/unavailable skins use defaults/);
  await page.unroute(blocked);
  const retry=await page.evaluate(async entry=>{
    const w=window.simlab.world, bytes=new Uint8Array(await(await fetch(`/api/file/${entry}`)).arrayBuffer());
    return (await w.constructor.load(bytes)).skins;
  },entry);
  assert.equal(retry.loaded,2);assert.equal(retry.failed.length,0);
  // Real UI frame pacing, not a synthetic call to animate().
  await page.getByRole('button',{name:'reset',exact:true}).click();
  await page.locator('.status.ready').waitFor({timeout:90000});
  await page.locator('input[type=range]').evaluate(el=>{
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(el,'60');
    el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}));
  });
  await page.waitForFunction(()=>document.querySelector('.rate').textContent.includes('2.00'));
  await page.evaluate(()=>{
    const w=window.simlab.world, animate=w.animate.bind(w);
    window.motionFrames=[];
    w.animate=t=>{
      animate(t);
      for(const [key,f] of w.flights) if(f.object) {
        window.motionFrames.push({id:`${key}/${f.started}`,progress:(t-f.started)/(f.lands-f.started),position:f.object.position.toArray()});
      }
    };
    w.sim.useBlock(12,7,13);w.applyChanges(w.drainChanges());
  });
  await page.getByRole('button',{name:'run',exact:true}).click();
  await page.waitForFunction(()=>window.motionFrames.some(f=>f.progress>.15&&f.progress<.4),{},{timeout:30000});
  await page.getByRole('button',{name:'pause',exact:true}).click();
  const paused=await page.evaluate(async()=>{
    const snapshots=[];
    for(let i=0;i<8;i++){
      await new Promise(requestAnimationFrame);
      snapshots.push([...window.simlab.world.flights.values()].filter(f=>f.object).map(f=>f.object.position.toArray()));
    }
    return {snapshots,first:window.motionFrames[0],frames:window.motionFrames.length};
  });
  assert(paused.first.progress<.5,'first painted stroke includes the first half-step');
  assert(paused.snapshots[0].length>0);
  for(const snapshot of paused.snapshots.slice(1))assert.deepEqual(snapshot,paused.snapshots[0],'pause freezes the exact pose');
  await page.getByRole('button',{name:'run',exact:true}).click();
  await page.evaluate(async()=>{for(let i=0;i<8;i++)await new Promise(requestAnimationFrame);});
  await page.getByRole('button',{name:'pause',exact:true}).click();
  const frames=await page.evaluate(()=>window.motionFrames);
  const last=new Map();
  for(const f of frames){assert(f.progress>=(last.get(f.id)??-Infinity),'pause/resume cannot reverse a stroke');last.set(f.id,f.progress);}
  assert.equal(errors.length,0);
  const result={status:'pass',retry,firstFrameProgress:paused.first.progress,pausedFrames:8,drawnFrames:frames.length,errors,
    scope:'Failed skin fetch falls back and retries within the same module session; real UI piston presentation includes its first half-step and holds/resumes without reversing.'};
  await writeFile(`${output}/result.json`,JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
}finally{await browser.close();}
