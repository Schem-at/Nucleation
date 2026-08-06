import { chromium } from "playwright";
const b = await chromium.launch(); const p = await b.newPage();
const errs=[]; p.on("pageerror",e=>errs.push(String(e).slice(0,140)));
await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
for (const size of [16, 32, 0]) {
  await p.reload({ waitUntil: "networkidle" });
  await p.setInputFiles("input[type=file]", process.argv[2]);
  await p.waitForSelector(".status.ready", { timeout: 300000 });
  await p.selectOption(".settle select >> nth=1", String(size));
  await p.waitForSelector(".status.ready", { timeout: 300000 });
  const r = await p.evaluate(async () => {
    const w = window.simlab.world;
    w.sim.placeBlock(31,7,13,"minecraft:air");
    let flushMs=0, chunks=0, ticks=0;
    for (let t=0;t<60;t++) {
      w.sim.step(); w.applyChanges(w.drainChanges());
      const n=w.dirty.size; const m=performance.now(); await w.flush();
      flushMs+=performance.now()-m; chunks+=n; ticks++;
    }
    return { chunkSize:w.chunkSize, msPerTick:flushMs/ticks,
             chunksPerTick:chunks/ticks, live:w.chunks.size };
  });
  console.log(`chunk ${String(size===0?"whole":size).padEnd(6)} (resolved ${String(r.chunkSize).padEnd(4)}) ${r.msPerTick.toFixed(1).padStart(7)} ms/tick  ${r.chunksPerTick.toFixed(2)} dirty/tick  ${r.live} live`);
}
console.log("errors:", errs.length?errs.slice(0,2):"none");
await b.close();
