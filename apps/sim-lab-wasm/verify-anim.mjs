/** Does a piston stroke actually slide, or still teleport?
 *
 * Drives `animate()` with synthetic timestamps rather than wall clock: the
 * block meshes parse asynchronously, so a real-time check races the parse
 * and reports a failure that is only in the harness.
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
    w.applyChanges(w.drainChanges(), 10.0); // long flight, sampled synthetically
    n = w.flights.length;
  }
  if (!n) return { flights: 0 };
  await sleep(700); // let the one-block meshes parse
  const t0 = w.flights[0].start;
  const at = (ms) => {
    w.animate(t0 + ms);
    return w.flights
      .filter((f) => f.object)
      .map((f) => [+f.object.position.x.toFixed(2), +f.object.position.y.toFixed(2), +f.object.position.z.toFixed(2)]);
  };
  const a = at(1000), mid = at(5000);
  const first = w.flights[0];
  return { flights: n, attached: a.length, a: a.slice(0, 3), mid: mid.slice(0, 3),
           from: first?.from, to: first?.to, states: w.flights.slice(0, 3).map((f) => f.state) };
});

if (!out.flights) console.log("❌ nothing in flight — strokes are instantaneous");
else if (!out.attached) console.log(`❌ ${out.flights} flights but none got a mesh`);
else {
  console.log(`in flight: ${out.flights}, drawn: ${out.attached}`);
  console.log(`  ${out.states[0]}  ${JSON.stringify(out.from)} -> ${JSON.stringify(out.to)}`);
  console.log(`  10% through: ${JSON.stringify(out.a[0])}`);
  console.log(`  50% through: ${JSON.stringify(out.mid[0])}`);
  const frac = out.mid.some((v) => v.some((n) => Math.abs(n - Math.round(n)) > 0.05));
  const moved = JSON.stringify(out.a) !== JSON.stringify(out.mid);
  console.log(frac && moved ? "  ✅ blocks slide smoothly between cells" : "  ❌ positions are still snapping to the grid");
}
console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
await b.close();
