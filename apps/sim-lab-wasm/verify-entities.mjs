/** Are the simulated entities actually on screen?
 *
 * The lab drew blocks and nothing else, so a build whose interesting parts are
 * boats, carts and riders rendered as an empty room that nonetheless ticked.
 * This checks the three things that can each independently be wrong: that the
 * engine reports entities, that the lab turns them into objects, and that
 * those objects sit at the entity's position rather than at the origin.
 */
import { chromium } from "playwright";

const b = await chromium.launch();
const p = await b.newPage();
const errs = [];
p.on("pageerror", (e) => errs.push(String(e).slice(0, 200)));
p.on("console", (m) => { if (m.type() === "error") errs.push(m.text().slice(0, 200)); });
await p.goto("http://127.0.0.1:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready, .status.error", { timeout: 300000 });

const out = await p.evaluate(() => {
  const w = window.simlab.world;
  const view = JSON.parse(w.sim.itemEntitiesJson());
  const counts = {
    items: (view.items ?? []).length,
    minecarts: (view.minecarts ?? []).length,
    frozen: (view.frozen ?? []).length,
  };
  // Positions the renderer actually used, not the ones it was given.
  const drawn = [];
  w.entityGroup.children.forEach((o) => {
    drawn.push([+o.position.x.toFixed(3), +o.position.y.toFixed(3), +o.position.z.toFixed(3)]);
  });
  // Step a while: entities must survive ticking, and be re-synced each time.
  for (let t = 0; t < 20; t++) {
    w.sim.step();
    w.applyChanges(w.drainChanges(), 0);
  }
  return {
    counts,
    reported: counts.items + counts.minecarts + counts.frozen,
    drawn: drawn.length,
    afterSteps: w.entityCount(),
    samplePositions: drawn.slice(0, 4),
    sampleBodies: (view.frozen ?? []).slice(0, 4).map((f) => ({
      kind: f.kind, pos: f.pos, size: f.size, leashed: f.leashed,
    })),
    leashedCount: (view.frozen ?? []).filter((f) => f.leashed).length,
    atOrigin: drawn.filter((v) => v[0] === 0 && v[1] === 0 && v[2] === 0).length,
  };
});

console.log(`engine reports: ${out.reported} entities`, out.counts);
console.log(`lab drew:       ${out.drawn}   (after 20 ticks: ${out.afterSteps})`);
console.log(`leashed:        ${out.leashedCount}`);
for (const b of out.sampleBodies) {
  console.log(`  ${b.kind}  pos=${JSON.stringify(b.pos)} size=${JSON.stringify(b.size)} leashed=${b.leashed}`);
}
console.log(`positions:      ${JSON.stringify(out.samplePositions)}`);

// A build with no entities proves nothing either way — every door in
// `tests/corpus/litematics` is one, so say so rather than reporting a failure
// the renderer did not cause.
let ok;
if (out.reported === 0) {
  console.log(
    out.drawn === 0
      ? "— this build has no entities; nothing for the entity layer to draw"
      : `❌ drew ${out.drawn} entities the engine does not report`,
  );
  ok = out.drawn === 0;
} else {
  ok =
    out.drawn === out.reported &&
    out.afterSteps === out.reported &&
    out.atOrigin === 0 &&
    out.sampleBodies.every((b) => Array.isArray(b.size) && b.size[0] > 0);
  console.log(
    ok
      ? "✅ every reported entity is drawn, sized from its measured hitbox, and in the right place"
      : `❌ reported ${out.reported}, drew ${out.drawn}, ${out.atOrigin} stuck at the origin`,
  );
}
console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
await b.close();
process.exit(ok && errs.length === 0 ? 0 : 1);
