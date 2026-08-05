/** A flying machine with no runway, flown far past where the build ends.
 *
 * The engine's region grows to follow it; this checks that the browser sees
 * the same thing — the machine stays assembled, keeps moving at a block per
 * ten ticks, and the renderer meshes the chunks it flies into.
 */
import { chromium } from "playwright";

const MACHINE = [
  [1, 0, 0, "minecraft:observer[facing=west,powered=false]"],
  [2, 0, 0, "minecraft:slime_block"],
  [3, 0, 0, "minecraft:sticky_piston[extended=false,facing=west]"],
  [2, 0, 1, "minecraft:sticky_piston[extended=false,facing=east]"],
  [3, 0, 1, "minecraft:slime_block"],
  [4, 0, 1, "minecraft:observer[facing=east,powered=false]"],
];

const b = await chromium.launch();
const p = await b.newPage();
const errs = [];
p.on("pageerror", (e) => errs.push(String(e).slice(0, 200)));
p.on("console", (m) => { if (m.type() === "error") errs.push(m.text().slice(0, 200)); });
await p.goto("http://127.0.0.1:8455/", { waitUntil: "networkidle" });
// Any file, just to get a World built; its contents are replaced below.
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready, .status.error", { timeout: 300000 });

const out = await p.evaluate(async (MACHINE) => {
  const w = window.simlab.world;
  const eng = w.eng;
  const schem = eng.Schematic.create("fly");
  for (const [x, y, z, state] of MACHINE) schem.setBlockFromString(x, y, z, state);
  w.schem = schem;
  w.dims = [5, 1, 2];
  w.settle = "quiet";
  const err = w.startSim();
  if (err) return { error: err };

  const machine = () => {
    const found = [];
    // Sweep well past the original build to find where it got to.
    for (let x = -8; x < 120; x++)
      for (let z = 0; z < 2; z++) {
        const s = w.blockAt(x, 0, z);
        if (s && s !== "minecraft:air") found.push([x, z, s.replace("minecraft:", "")]);
      }
    return found;
  };

  const start = machine();
  // Kick it: a redstone block above the east-facing piston for two ticks.
  // Derived, not hardcoded — `Schematic.create` normalises the build's
  // origin, so the machine does not sit where its source structure did.
  const kick = start.find((b) => b[2].startsWith("sticky_piston[extended=false,facing=east"));
  if (!kick) return { error: `no east piston among ${JSON.stringify(start)}` };
  const [kx, kz] = [kick[0], kick[1]];
  w.sim.placeBlock(kx, 1, kz, "minecraft:redstone_block");
  w.sim.step();
  w.sim.step();
  w.sim.placeBlock(kx, 1, kz, "minecraft:air");
  for (let i = 0; i < 400; i++) w.sim.step();
  w.applyChanges(w.drainChanges());
  await w.flush();

  const end = machine();
  return {
    startX: start.map((b) => b[0]),
    endX: end.map((b) => b[0]),
    endParts: end.length,
    end,
    chunks: w.group.children.length,
    chunkKeys: [...w.chunks.keys()],
    tick: Number(w.sim.tickCount?.() ?? -1),
  };
}, MACHINE);

if (out.error) console.log("ERROR:", out.error);
else {
  const span = (xs) => (xs.length ? `${Math.min(...xs)}..${Math.max(...xs)}` : "(gone)");
  console.log(`start x ${span(out.startX)} (${out.startX.length} blocks)`);
  console.log(`end   x ${span(out.endX)} (${out.endParts} blocks)  tick ${out.tick}  chunks ${out.chunks}`);
  console.log("meshed chunks:", out.chunkKeys, "(chunk 2 = x 32..47)");
  console.log("end parts:", out.end.map((b) => `${b[0]},${b[1]} ${b[2].split("[")[0]}`).join(" | "));
}
console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
await b.close();
