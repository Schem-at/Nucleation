/** Does draining cost more the longer the session runs?
 *
 * `drainChanges` used to call `changesJson()`, which serialises the whole
 * accumulated log, then throw away everything before a cursor. That is
 * O(total changes) per frame and unbounded over a session. After the fix it
 * consumes and clears, so a drain costs what the last batch produced.
 *
 * `http://localhost:8455/` is where `vite preview` serves the app's `dist/`
 * build — run that (or `npm run preview` in this app) before this script.
 *
 * `placeBlock(31, 7, 13, "minecraft:air")` is BB-specific: it is BB's kick
 * block, and this build does nothing at all until it is kicked. Pass a
 * different `<BB>` fixture and this probe will step 400 quiescent ticks and
 * report meaningless, near-zero numbers for both samples.
 */
import { chromium } from "playwright";
const b = await chromium.launch(); const p = await b.newPage();
await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready", { timeout: 300000 });
console.log(await p.evaluate(async () => {
  const w = window.simlab.world, out = [];
  w.sim.placeBlock(31, 7, 13, "minecraft:air"); // BB's kick block; see header
  const sample = () => {
    const t0 = performance.now();
    const got = w.drainChanges().length;
    return { ms: performance.now() - t0, got };
  };
  let first = null, last = null;
  for (let t = 0; t < 400; t++) {
    w.sim.step();
    const s = sample();
    if (t === 20) first = s;
    if (t === 399) last = s;
  }
  out.push(`drain at tick  20: ${first.ms.toFixed(3)} ms for ${first.got} changes`);
  out.push(`drain at tick 400: ${last.ms.toFixed(3)} ms for ${last.got} changes`);
  const grew = last.ms > first.ms * 4 && last.ms > 0.5;
  out.push(grew
    ? `  ❌ draining got ${(last.ms / Math.max(first.ms, 0.001)).toFixed(1)}x more expensive as the log grew`
    : "  ✅ a drain costs what the last batch produced, not what the session accumulated");
  return out.join("\n");
}));
await b.close();
