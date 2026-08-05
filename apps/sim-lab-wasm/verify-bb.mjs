/** BB in the sim lab: break the obsidian, run, check it is still flying.
 *
 * The trigger is mining the lone obsidian; the machine then cycles for
 * hundreds of ticks and creeps west. A stalled engine goes quiet within ~80
 * ticks, so "still changing blocks at tick 400" is the check that matters.
 */
import { chromium } from "playwright";

const file = process.argv[2];
const b = await chromium.launch();
const p = await b.newPage();
const errs = [];
p.on("pageerror", (e) => errs.push(String(e).slice(0, 200)));
p.on("console", (m) => { if (m.type() === "error") errs.push(m.text().slice(0, 200)); });
await p.goto("http://127.0.0.1:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", file);
await p.waitForSelector(".status.ready, .status.error", { timeout: 300000 });
console.log("load:", (await p.textContent(".status")).trim());

const out = await p.evaluate(async () => {
  const w = window.simlab.world;
  if (!w?.sim) return { error: "no sim" };
  const footprint = () => {
    let lo = Infinity, hi = -Infinity, n = 0;
    const [dx, dy, dz] = w.dims;
    for (let x = -8; x < dx + 8; x++)
      for (let y = 0; y < dy; y++)
        for (let z = 0; z < dz; z++) {
          const s = w.blockAt(x, y, z);
          if (s && s !== "minecraft:air") { n++; lo = Math.min(lo, x); hi = Math.max(hi, x); }
        }
    return { lo, hi, n };
  };
  const before = footprint();
  // The trigger: mine the obsidian.
  w.sim.placeBlock(31, 7, 13, "minecraft:air");
  const active = [];
  let seen = 0;
  for (let t = 0; t < 400; t++) {
    w.sim.step();
    const n = Number(w.sim.changesCount?.() ?? 0);
    if (n > seen) { active.push(t); seen = n; }
  }
  w.applyChanges(w.drainChanges());
  await w.flush();
  return {
    activeTicks: active.length,
    lastActive: active[active.length - 1] ?? null,
    totalChanges: seen,
    before,
    after: footprint(),
    chunks: w.group.children.length,
  };
}, );

if (out.error) console.log("ERROR:", out.error);
else {
  console.log(`  active ticks: ${out.activeTicks}   last active: t${out.lastActive}   changes: ${out.totalChanges}`);
  console.log(`  footprint x ${out.before.lo}..${out.before.hi} -> ${out.after.lo}..${out.after.hi}`);
  console.log(`  chunks in scene: ${out.chunks}`);
  console.log(out.lastActive > 350 ? "  ✅ still flying at tick 400" : "  ❌ STALLED");
}
console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
await b.close();
