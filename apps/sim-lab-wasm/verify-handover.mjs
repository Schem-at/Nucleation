/** Is every cell drawn by something at every painted frame?
 *
 * Two ways a cell ends up drawn by nothing, both of which read as blinking:
 *
 *  1. **The hand-off.** A flight retires on the tick its block lands — exact,
 *     and decided by the simulation. The chunk that must then draw that block
 *     is re-meshed asynchronously, one `MeshResult` at a time. Cutting the
 *     flight away at retirement left the cell empty until the chunk caught up:
 *     1446 of 4965 retirements on BB, up to 8 painted frames each. `retire`
 *     parks the meshes instead and `flush` releases them in the same turn as
 *     the chunk swap.
 *  2. **A flight with no mesh.** A one-block mesh is a GLB parse; until it
 *     finishes the flight is not drawn and its cell is blank for the whole
 *     stroke. `warmBlockMeshes` pre-parses everything in the world snapshot,
 *     and `warmState` catches states the simulation invents later. What is
 *     left is the first flight of each such state — bounded by the number of
 *     distinct states, not by how long the machine runs.
 */
import { chromium } from "playwright";

const b = await chromium.launch();
const p = await b.newPage();
const errs = [];
p.on("pageerror", (e) => errs.push(String(e).slice(0, 200)));
await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready", { timeout: 300000 });

const out = await p.evaluate(async () => {
  const w = window.simlab.world;
  const size = w.chunkSize;
  const chunkOf = (cell) =>
    cell.split(",").map(Number).map((v) => Math.floor(v / size)).join(",");
  const raf = () => new Promise((r) => requestAnimationFrame(r));
  let frame = 0;
  const installedAt = new Map();
  const orig = w.installChunk.bind(w);
  w.installChunk = async (cx, cy, cz, mesh) => {
    const r = await orig(cx, cy, cz, mesh);
    installedAt.set([cx, cy, cz].join(","), frame);
    return r;
  };

  // Let the background mesh warm-up the app kicks off on load finish.
  for (let i = 0; i < 180 && w.pendingMeshes.size > 0; i++) await raf();

  const waiting = [], gaps = [], life = new Map();
  let emptied = 0;
  w.sim.placeBlock(31, 7, 13, "minecraft:air");
  for (let t = 0; t < 120; t++) {
    const before = new Set(w.flights.keys());
    w.sim.step();
    w.applyChanges(w.drainChanges());
    w.animate(Number(w.sim.tickCount()));
    const parked = new Set(w.handover.map((h) => h.cell));
    for (const k of before) {
      if (w.flights.has(k)) continue;
      // A stroke cancelled mid-flight leaves its destination genuinely empty
      // (vanilla's `finalTick` lands air for a source piston). Nothing should
      // be drawn there, so it is not a gap.
      if (!w.solid.has(k)) { emptied++; continue; }
      if (parked.has(k)) continue;
      waiting.push({ cell: k, chunk: chunkOf(k), since: frame });
    }
    void w.flush();
    await raf();
    frame++;
    for (const [k, f] of w.flights) {
      const id = `${k}|${f.started}|${f.state}`;
      const e = life.get(id) ?? { drawn: 0 };
      if (f.object || f.stillObject) e.drawn++;
      life.set(id, e);
    }
    for (let i = waiting.length - 1; i >= 0; i--) {
      const at = installedAt.get(waiting[i].chunk);
      if (at !== undefined && at >= waiting[i].since) {
        gaps.push(at - waiting[i].since);
        waiting.splice(i, 1);
      }
    }
  }
  for (const _ of waiting) gaps.push(999);
  const flights = [...life.values()];
  return {
    size, emptied,
    uncovered: gaps.length,
    blinking: gaps.filter((g) => g > 0).length,
    flights: flights.length,
    undrawn: flights.filter((e) => e.drawn === 0).length,
  };
});

console.log(`chunk size ${out.size}`);
console.log(`hand-off: ${out.uncovered} retirements left a drawn block uncovered ` +
            `(${out.emptied} more retired into a cell that should be empty)`);
console.log(
  out.blinking === 0
    ? "  ✅ no cell is left drawn by nothing between a flight and its chunk"
    : `  ❌ ${out.blinking}/${out.uncovered} blink for at least one painted frame`,
);
const pct = ((100 * out.undrawn) / Math.max(1, out.flights)).toFixed(1);
console.log(`meshes: ${out.undrawn}/${out.flights} flights (${pct}%) were never drawn at all`);
console.log(
  out.undrawn / Math.max(1, out.flights) < 0.02
    ? "  ✅ mesh warm-up keeps first-sighting misses to the tail"
    : `  ❌ ${pct}% of flights never got a mesh — check warmBlockMeshes`,
);
console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
await b.close();
