// Per-tick changes vs flights: the §3 evidence, and now the check on its fix.
//
// Two things must hold once flights come from the engine's own in-flight list
// rather than from a regex over the change stream:
//
//   * concurrent flights of one stroke report `distinct starts=1` — a rigid
//     body is animated as one object, not N that shear apart;
//   * no cell is ever both a live flight destination and freshly written by a
//     change on the same tick — that overlap is the double-draw, and its
//     mirror image (a flight gone before its block is written) is the gap;
//   * every drawn flight is the flight the engine currently has in that cell.
//     Consecutive strokes reuse a destination — a piston that pulses extends
//     and retracts into the same slot on successive ticks — and a reconciler
//     that only asks "do I have something here" keeps the stale one: the new
//     block is never drawn and the old one outstays its landing.
import { chromium } from "playwright";
const b = await chromium.launch(); const p = await b.newPage();
await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready", { timeout: 300000 });
console.log(await p.evaluate(async () => {
  const w = window.simlab.world, out = [];
  const cell = (a) => a.join(",");
  let overlaps = 0, desyncs = 0, stale = 0;
  w.sim.placeBlock(31, 7, 13, "minecraft:air");
  for (let t = 0; t < 60; t++) {
    w.sim.step();
    const ch = w.drainChanges();
    const before = new Set(w.flights.keys());
    w.applyChanges(ch);
    const air = [...w.flights.values()];
    const made = air.filter((f) => !before.has(cell(f.to)));
    const gone = [...before].filter((k) => !w.flights.has(k));
    // Is every drawn flight the one the engine has in that cell right now?
    for (const e of JSON.parse(w.sim.movingBlocksJson())) {
      const f = w.flights.get(cell(e.to));
      if (!f) { stale++; out.push(`   >> MISSING flight for ${cell(e.to)} (${e.carried})`); continue; }
      if (f.started !== e.started || cell(f.from) !== cell(e.from) || f.state !== e.carried) {
        stale++;
        out.push(`   >> STALE at ${cell(e.to)}: drawing ${f.state} from ${cell(f.from)} started=${f.started}` +
          `, engine has ${e.carried} from ${cell(e.from)} started=${e.started}`);
      }
    }

    if (!ch.length && !gone.length) continue;
    out.push(`tick ${t}: ${ch.length} changes, ${made.length} new flights, ${gone.length} retired`);
    for (const c of ch.slice(0, 6))
      out.push(`   change ${c.pos.join(",")} -> ${c.to.slice(0, 46)}`);
    for (const f of made)
      out.push(`   FLIGHT ${f.state.slice(0, 34)} ${cell(f.from)}->${cell(f.to)} started=${f.started} lands=${f.lands}`);
    for (const k of gone) out.push(`   RETIRE ${k}`);

    // A drawn block and a flight into the same cell, on the same tick. Only
    // states the chunk actually meshes count: `air` is nothing, and a
    // `moving_piston` placeholder is written as air precisely so the sliding
    // mesh can fill the hole.
    //
    // Last write per cell, not every write. `applyChanges` replays the batch
    // in order and re-meshes once from the result, so an intermediate state —
    // a head landing in a slot that the same tick's retraction immediately
    // turns back into a placeholder — is never on screen at all.
    const finalState = new Map();
    for (const c of ch) finalState.set(cell(c.pos), c.to);
    const landed = new Set(
      [...finalState]
        .filter(([, to]) => to !== "minecraft:air" && !to.startsWith("minecraft:moving_piston"))
        .map(([k]) => k),
    );
    for (const k of w.flights.keys())
      if (landed.has(k)) { overlaps++; out.push(`   >> OVERLAP at ${k} — drawn twice`); }

    // Flights dispatched together must set off together. Distinct starts
    // among the ones *created this tick* is the shear that detached a piston
    // head from its load; distinct windows among everything in the air is
    // just two strokes overlapping, which is fine.
    if (made.length > 1) {
      const starts = new Set(made.map((f) => f.started));
      if (starts.size > 1) desyncs++;
      out.push(`   >> ${made.length} launched together; distinct starts=${starts.size}` +
        (starts.size > 1 ? "  DESYNCED" : ""));
    }
    if (air.length > 1) {
      const windows = new Set(air.map((f) => `${f.started}-${f.lands}`));
      out.push(`   >> ${air.length} in air across ${windows.size} stroke window(s)`);
    }
  }
  out.push(`\n== ${overlaps} overlaps, ${desyncs} ticks with desynced flights, ${stale} stale/missing flights ==`);
  return out.join("\n");
}));
await b.close();
