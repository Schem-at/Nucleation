/** Does a piston stroke actually slide, or still teleport?
 *
 * Drives `animate()` with synthetic *tick* stamps rather than the wall clock:
 * flights now live on the simulation's clock, and the block meshes parse
 * asynchronously, so a real-time check races the parse and reports a failure
 * that is only in the harness.
 */
import { chromium } from "playwright";

const b = await chromium.launch();
const p = await b.newPage();
const errs = [];
p.on("pageerror", (e) => errs.push(String(e).slice(0, 200)));
p.on("console", (m) => { if (m.type() === "error") errs.push(m.text().slice(0, 200)); });
await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready, .status.error", { timeout: 300000 });

const out = await p.evaluate(async () => {
  const w = window.simlab.world;
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  w.sim.placeBlock(31, 7, 13, "minecraft:air");
  let n = 0;
  for (let t = 0; t < 40 && n === 0; t++) {
    w.sim.step();
    w.applyChanges(w.drainChanges());
    n = w.flights.size;
  }
  if (!n) return { flights: 0 };
  await sleep(700); // let the one-block meshes parse
  const air = [...w.flights.values()];
  const first = air[0];
  // Sample fractional ticks inside the stroke's own window, so the check is
  // in the same units the app now animates in.
  const at = (frac) => {
    w.animate(first.started + (first.lands - first.started) * frac);
    return [...w.flights.values()]
      .filter((f) => f.object)
      .map((f) => [+f.object.position.x.toFixed(2), +f.object.position.y.toFixed(2), +f.object.position.z.toFixed(2)]);
  };
  const a = at(0.1), mid = at(0.5);

  // A piston arm is drawn shortened while it is within half a block of its
  // body — full length all the way home and the shaft passes visibly through
  // the back of the piston. Vanilla flips at `progress <= 0.5` extending and
  // `progress >= 0.5` retracting; both mean the same thing from each end.
  const arms = {};
  for (let t = 0; t < 90 && !(arms.extend && arms.retract); t++) {
    w.sim.step(); w.applyChanges(w.drainChanges());
    w.animate(Number(w.sim.tickCount())); void w.flush();
    await new Promise((r) => requestAnimationFrame(r));
    for (const f of w.flights.values()) {
      const kind = f.extending ? "extend" : "retract";
      if (!f.shortState || arms[kind]) continue;
      w.blockMesh(f.shortState); w.blockMesh(f.state);
      for (let i = 0; i < 90 && !(w.blockMeshes.get(f.shortState) && w.blockMeshes.get(f.state)); i++)
        await new Promise((r) => requestAnimationFrame(r));
      const span = f.lands - f.started;
      arms[kind] = [0, 0.25, 0.5, 0.75, 1].map((frac) => {
        w.animate(f.started + span * frac);
        return f.drawnState.includes("short=true") ? "S" : "l";
      }).join("");
    }
  }

  return { flights: n, attached: a.length, a: a.slice(0, 3), mid: mid.slice(0, 3),
           from: first?.from, to: first?.to, window: [first?.started, first?.lands],
           states: air.slice(0, 3).map((f) => f.state), arms };
});

if (!out.flights) console.log("❌ nothing in flight — strokes are instantaneous");
else if (!out.attached) console.log(`❌ ${out.flights} flights but none got a mesh`);
else {
  console.log(`in flight: ${out.flights}, drawn: ${out.attached}`);
  console.log(`  ${out.states[0]}  ${JSON.stringify(out.from)} -> ${JSON.stringify(out.to)}  ticks ${out.window.join("..")}`);
  console.log(`  10% through: ${JSON.stringify(out.a[0])}`);
  console.log(`  50% through: ${JSON.stringify(out.mid[0])}`);
  const frac = out.mid.some((v) => v.some((n) => Math.abs(n - Math.round(n)) > 0.05));
  const moved = JSON.stringify(out.a) !== JSON.stringify(out.mid);
  console.log(frac && moved ? "  ✅ blocks slide smoothly between cells" : "  ❌ positions are still snapping to the grid");
  // Sampled at 0, ¼, ½, ¾, 1 of the stroke: S = shortened arm, l = full length.
  const want = { extend: "SSSll", retract: "llSSS" };
  for (const kind of ["extend", "retract"]) {
    const got = out.arms?.[kind];
    console.log(
      got === want[kind]
        ? `  ✅ ${kind} arm shortens beside the body (${got})`
        : `  ❌ ${kind} arm is ${got ?? "not seen"}, expected ${want[kind]}`,
    );
  }
}
console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
await b.close();
