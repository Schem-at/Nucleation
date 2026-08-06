import { chromium } from "playwright";
const b = await chromium.launch(); const p = await b.newPage();
await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready", { timeout: 300000 });
console.log(await p.evaluate(async () => {
  const w = window.simlab.world, out = [];
  w.sim.placeBlock(31,7,13,"minecraft:air");
  for (let t = 0; t < 14; t++) {
    w.sim.step();
    const ch = w.drainChanges();
    const before = w.flights.length;
    w.applyChanges(ch, 2.0);
    const made = w.flights.slice(before);
    if (!ch.length) continue;
    out.push(`tick ${t}: ${ch.length} changes, ${made.length} flights`);
    for (const c of ch.slice(0, 6))
      out.push(`   change ${c.pos.join(",")} -> ${c.to.slice(0, 46)}`);
    for (const f of made)
      out.push(`   FLIGHT ${f.state.slice(0,34)} ${f.from.join(",")}->${f.to.join(",")} start=${f.start.toFixed(0)} dur=${f.dur}`);
    // Are concurrent flights in lockstep?
    if (w.flights.length > 1) {
      const starts = new Set(w.flights.map(f => Math.round(f.start)));
      const durs = new Set(w.flights.map(f => f.dur));
      out.push(`   >> ${w.flights.length} in air; distinct starts=${starts.size} durs=${durs.size} ${starts.size>1?"DESYNCED":""}`);
    }
  }
  return out.join("\n");
}));
await b.close();
