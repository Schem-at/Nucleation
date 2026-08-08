// Node test for the generated mc-tick bindings (TickSimulation).
//
// Run with: node tests/node_mc_tick_sim_test.mjs
// Prereq: cargo build --release --target wasm32-unknown-unknown --lib --features bridge,mc-tick
//
// Replays the shulker-pipeline scenario the Rust case
// (crates/mc-tick/tests/cases/shulker_pipeline.test.json) pins, and opens the
// 6x6 in-world door — asserting the same end states through the bindings.

import fs from "node:fs";
import { TickSimulation, TickSettleMode, ResourcePack, Schematic, MeshResult } from "../bindings/js/index.mjs";

let pass = 0;
let fail = 0;
function expect(cond, what) {
  if (cond) {
    pass++;
    console.log("  ok  " + what);
  } else {
    fail++;
    console.log("  FAIL " + what);
  }
}

// --- shulker pipeline, seeded ---
const snbt = fs.readFileSync(
  "crates/mc-tick/tests/corpus/structures/shulker_pipeline.snbt",
  "utf8",
);
const sim = TickSimulation.fromSnbt(snbt, TickSettleMode.Placement, 0, 0, 0, "");
sim.setRngSeed(12345n);

const RB = "minecraft:redstone_block";
const actions = [
  [5, -1, 2, 1, RB],
  [30, 2, 0, 1, RB],
  [34, 2, 0, 1, "minecraft:air"],
  [38, 2, 0, 1, RB],
  [42, 2, 0, 1, "minecraft:air"],
  [46, 1, 1, 2, RB],
  [50, 1, 4, 1, RB],
  [54, 1, 4, 1, "minecraft:air"],
  [60, 1, 1, 2, "minecraft:air"],
  [82, 2, 0, 1, RB],
  [86, 2, 0, 1, "minecraft:air"],
];
for (let t = 0; t <= 110; t++) {
  if (t === 12) {
    expect(
      sim.getBlock(1, 2, 1).startsWith("minecraft:white_shulker_box"),
      "shulker placed by the dispenser at tick 12",
    );
  }
  if (t === 64) {
    expect(sim.getBlock(1, 2, 1) === "minecraft:air", "shulker broken by tick 64");
  }
  for (const [at, x, y, z, state] of actions) {
    if (at === t) sim.placeBlock(x, y, z, state);
  }
  if (t < 110) sim.step();
}
expect(sim.tickCount() === 110, "tickCount reaches 110");

const entities = JSON.parse(sim.itemEntitiesJson());
const east = (e) => e.pos[0] >= 3 && e.pos[0] < 6;
const diamonds = entities.items
  .filter((e) => e.item === "minecraft:diamond" && east(e))
  .reduce((n, e) => n + e.count, 0);
expect(diamonds === 2, `two diamonds land east of the dropper (got ${diamonds})`);
const shulkers = entities.items.filter(
  (e) => e.item === "minecraft:white_shulker_box" && east(e),
);
expect(shulkers.length === 1, "one shulker item lands east");
expect(
  shulkers.length === 1 && shulkers[0].contents.length === 0,
  "the shipped shulker is empty",
);

const summary = JSON.parse(sim.eventsSummaryJson());
expect(summary.length > 10, "events summary has per-tick rows");
expect(
  summary.some((r) => r.piston > 0),
  "piston activity shows in the summary",
);
const changes = JSON.parse(sim.changesJson());
expect(changes.length > 20, "block changes recorded");
const snapshot = JSON.parse(sim.worldSnapshotJson());
expect(snapshot.length > 20, "world snapshot lists blocks");

// --- checkpoint/restore ---
const cp = sim.checkpoint();
sim.placeBlock(0, 3, 0, RB);
sim.step();
sim.restore(cp);
expect(sim.getBlock(0, 3, 0) === "minecraft:air", "restore rewinds a write");

// --- the 6x6 door opens on its lever ---
const door = fs.readFileSync(
  "crates/mc-tick/tests/corpus/structures/door_6x6_inworld.snbt",
  "utf8",
);
const doorSim = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
doorSim.useBlock(10, 4, 1);
doorSim.run(40);
const doorChanges = JSON.parse(doorSim.changesJson());
expect(
  doorChanges.some((c) => c.to.includes("moving_piston")),
  "the 6x6 door's pistons move after the lever click",
);

// --- what the engine currently has in flight ---
//
// The renderer's answer to "which block is sliding, from where, arriving
// when". Reconstructing it from `changesJson` is a reimplementation of piston
// mechanics in the host, which is how an animation ends up on a different
// clock from the simulation that decides the landing.
const flightSim = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
flightSim.useBlock(10, 4, 1);
let air = [];
for (let t = 0; t < 40 && air.length === 0; t++) {
  flightSim.step();
  air = JSON.parse(flightSim.movingBlocksJson());
}
expect(air.length > 0, `the door reports blocks in flight (got ${air.length})`);
const m = air[0] ?? {};
expect(
  typeof m.state === "string" && !m.state.startsWith("minecraft:moving_piston"),
  `a flight names the block being carried, not the placeholder (${m.state})`,
);
const step = { east: [1, 0, 0], west: [-1, 0, 0], up: [0, 1, 0], down: [0, -1, 0], south: [0, 0, 1], north: [0, 0, -1] };
const d = step[m.dir] ?? [0, 0, 0];
expect(
  m.from?.every?.((v, i) => v + d[i] === m.to[i]),
  `it travels one cell ${m.dir}: ${JSON.stringify(m.from)} -> ${JSON.stringify(m.to)}`,
);
expect(
  air.every((f) => f.started === m.started && f.lands === m.lands),
  "blocks dispatched together share one flight window",
);
expect(
  m.lands > m.started && m.started === flightSim.tickCount() - 1,
  `the window is the tick it set off to the tick it lands (${m.started}..${m.lands} at tick ${flightSim.tickCount()})`,
);

// A retracting piston is the one move where what lands is not what travels:
// vanilla's PistonHeadRenderer draws a `piston_head` on the interpolated slot
// and the body, EXTENDED=true, parked outside it. Run the door long enough for
// its pistons to come home and check the split holds everywhere.
let retractions = 0;
let split = true;
for (let t = 0; t < 120; t++) {
  flightSim.step();
  for (const f of JSON.parse(flightSim.movingBlocksJson())) {
    const retracting = f.source_piston && !f.extending;
    if (retracting) {
      retractions++;
      if (
        !f.carried.startsWith("minecraft:piston_head") ||
        !(f.remains ?? "").includes("extended=true") ||
        !f.state.includes("extended=false")
      ) split = false;
    } else if (f.carried !== f.state || f.remains !== null) split = false;
    // A moving arm comes in two lengths — the game shortens it while the head
    // is beside its body, or the shaft passes through the back of the piston.
    if (f.carried.startsWith("minecraft:piston_head")) {
      if (
        !f.carried_short?.includes("short=true") ||
        f.carried_short.replace("short=true", "short=false") !== f.carried
      ) split = false;
    } else if (f.carried_short !== null) split = false;
  }
}
expect(retractions > 0, `the door's pistons retract (${retractions} flight-ticks seen)`);
expect(
  split,
  "a retraction carries a piston_head and parks an extended body; every other move carries itself and parks nothing; " +
    "every moving arm offers both lengths",
);

// --- the event timeline ---
const tl = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
tl.recordTimeline();
tl.useBlock(10, 4, 1);
tl.run(40);
tl.stopTimeline();
// A stopped span stays readable — that is what makes it exportable.
const activity = JSON.parse(tl.timelineActivityJson());
expect(activity.ticks.length > 0, `the door records activity (${activity.ticks.length} active ticks)`);
expect(
  activity.ticks.every((t) => t.changes > 0 || t.inputs > 0 || t.pistons > 0),
  "every listed tick did something — idle ticks are absent, so a still build does not advance the strip",
);
expect(
  activity.ticks.every((t, i, a) => i === 0 || t.tick > a[i - 1].tick),
  "ticks are strictly ascending",
);
expect(
  activity.ticks.length < activity.end - activity.start + 1,
  `the door has idle ticks that were skipped (${activity.ticks.length} of ${activity.end - activity.start + 1})`,
);

// --- cycles, projection, and a selection's schematic ---
const cycles = JSON.parse(tl.timelineCyclesJson());
expect("exact" in cycles && "translated" in cycles, "cycles report has both kinds");

// A flying machine repeats itself displaced; an absent cycle is null, not an error.
const fly = TickSimulation.fromSnbt(
  fs.readFileSync("crates/mc-tick/tests/corpus/structures/flying_machine_east.snbt", "utf8"),
  TickSettleMode.InWorld, 0, 0, 0, "",
);
// Kick timing matches the CLI's verified `--place 2:...=redstone_block
// --place 4:...=air` (see GLOBAL CONSTRAINTS): 2 quiet ticks, a 2-tick pulse,
// then run to 60. A kick held longer (e.g. placed at tick 0) does not launch
// this machine at all — placement timing is load-bearing here, not cosmetic.
fly.recordTimeline();
fly.run(2);
fly.placeBlock(2, 1, 1, RB);
fly.run(2);
fly.placeBlock(2, 1, 1, "minecraft:air");
fly.run(56);
const flyCycles = JSON.parse(fly.timelineCyclesJson());
expect(flyCycles.translated !== null, "the flying machine repeats itself, displaced");
expect(flyCycles.translated.drift[0] !== 0, `it travels along x (drift ${flyCycles.translated.drift})`);

const projected = JSON.parse(fly.animationTimelineJson(0, 20, 50.0));
expect(Array.isArray(projected.events) && projected.events.length > 0, "the projection has events");
expect(
  projected.events.every((e) => e.kind === "piston" || e.kind === "set_block"),
  "every event is one the mesher understands",
);
expect(typeof projected.tick_ms === "number" && projected.origin.length === 3, "origin and tick_ms are present");

const seed = fly.selectionSchematicB64(0, 20);
expect(seed.length > 0, "the selection's initial schematic comes back as bytes");

// --- pinned bridge-vs-CLI agreement ---
//
// `nucleation-cli animate crates/mc-tick/tests/corpus/structures/flying_machine_east.snbt
// --ticks 60 --place 2:2,1,1=minecraft:redstone_block --place 4:2,1,1=minecraft:air`
// prints these exact numbers (verified by hand against this run — see GLOBAL
// CONSTRAINTS in the task brief):
//   exact:      start=0 end=7  period=7  drift=(0,0,0)
//   translated: start=5 end=15 period=10 drift=(-1,0,0)
// `Placement` settle reproduces the CLI's own numbers exactly; `InWorld` settle
// (the case above) legitimately gives different numbers because it simulates
// more of the world around the kick. Pinning this here catches a sign or
// period regression in either the bridge or the CLI path.
const pinned = TickSimulation.fromSnbt(
  fs.readFileSync("crates/mc-tick/tests/corpus/structures/flying_machine_east.snbt", "utf8"),
  TickSettleMode.Placement, 0, 0, 0, "",
);
pinned.recordTimeline();
pinned.run(2);
pinned.placeBlock(2, 1, 1, RB);
pinned.run(2);
pinned.placeBlock(2, 1, 1, "minecraft:air");
pinned.run(56);
const pinnedCycles = JSON.parse(pinned.timelineCyclesJson());
const pExact = pinnedCycles.exact;
const pTranslated = pinnedCycles.translated;
expect(
  pExact !== null &&
    pExact.start === 0 &&
    pExact.end === 7 &&
    pExact.period === 7 &&
    JSON.stringify(pExact.drift) === JSON.stringify([0, 0, 0]),
  `exact cycle matches the CLI's pinned numbers (got ${JSON.stringify(pExact)})`,
);
expect(
  pTranslated !== null &&
    pTranslated.start === 5 &&
    pTranslated.end === 15 &&
    pTranslated.period === 10 &&
    JSON.stringify(pTranslated.drift) === JSON.stringify([-1, 0, 0]),
  `translated cycle matches the CLI's pinned numbers (got ${JSON.stringify(pTranslated)})`,
);

// --- record, project, export ---
const packPath = "apps/sim-lab-wasm/public/pack/mesher-pack.zip";
if (fs.existsSync(packPath)) {
  const packZip = fs.readFileSync(packPath);
  const rp = ResourcePack.fromBytes(new Uint8Array(packZip));
  const seedSchem = Schematic.fromData(Buffer.from(seed, "base64"));
  const glbB64 = MeshResult.animatedGlbB64(seedSchem, rp, fly.animationTimelineJson(0, 20, 50.0));
  const glb = Buffer.from(glbB64, "base64");
  expect(glb.length > 1000, `an animated GLB comes back (${glb.length} bytes)`);
  expect(glb.toString("ascii", 0, 4) === "glTF", "and it is really a GLB");

  // Parse the GLB's JSON chunk and check for animation channels — the
  // property that distinguishes an animated export from a static mesh.
  const jsonChunkLength = glb.readUInt32LE(12);
  const gltf = JSON.parse(glb.toString("utf8", 20, 20 + jsonChunkLength));
  expect(
    Array.isArray(gltf.animations) && gltf.animations.length > 0,
    `the GLB has animations (${gltf.animations?.length ?? 0})`,
  );
  const channelCount = (gltf.animations ?? []).reduce((n, a) => n + (a.channels?.length ?? 0), 0);
  expect(channelCount > 0, `the animation(s) have channels (${channelCount})`);
  expect(Array.isArray(gltf.nodes) && gltf.nodes.length > 0, `the GLB has nodes (${gltf.nodes?.length ?? 0})`);
} else {
  console.log(`  skip animated GLB export — ${packPath} is absent (build artifact, not source)`);
}

// --- dropping a consumed change log ---
//
// The log grows for as long as the simulation runs and nothing empties it, so
// a long-running host accumulates every block change forever. A host that has
// already consumed the changes needs to say so.
const clr = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
clr.useBlock(10, 4, 1);
clr.run(20);
const before = Number(clr.changesCount());
expect(before > 0, `the door recorded changes (${before})`);
clr.clearChanges();
expect(Number(clr.changesCount()) === 0, "clearChanges empties the log");
expect(JSON.parse(clr.changesJson()).length === 0, "and changesJson agrees");
// This door has already fully opened and settled by tick 20 (verified: it
// never produces another change no matter how much longer it runs), so a
// second lever click is needed to give the still-recording sim something new
// to see — otherwise the door would stay quiescent and the assertion below
// would fail regardless of whether clearChanges had secretly turned
// recording off, telling us nothing about which one happened.
clr.useBlock(10, 4, 1);
clr.run(20);
expect(
  Number(clr.changesCount()) > 0,
  "recording continues after a clear — it drops what happened, it does not stop recording",
);

// --- a clear is refused while a run timeline is recording ---
//
// A timeline is a seed plus the change log: the recorder keeps only its start
// tick and an initial frame, and replay applies the log forward from there.
// Dropping the log mid-recording would leave every later frame reconstructing
// a world that never existed, so the engine refuses and says so.
const rec = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
rec.useBlock(10, 4, 1);
rec.run(10);
expect(rec.clearChanges() === true, "a clear succeeds with no timeline recording");
rec.recordTimeline();
rec.run(10);
const changesWhileRecording = Number(rec.changesCount());
expect(changesWhileRecording > 0, `the recording captured changes (${changesWhileRecording})`);
expect(rec.clearChanges() === false, "the clear is refused while a timeline is recording");
expect(
  Number(rec.changesCount()) === changesWhileRecording,
  "and the log is untouched — a refusal is not a partial clear",
);
rec.stopTimeline();
expect(rec.clearChanges() === true, "once stopped, a clear succeeds again");

// --- reading only what is new ---
//
// While a run timeline is recording the engine refuses to drop the change log,
// so a host must be able to read the tail without paying for the whole
// backlog on every frame.
const inc = TickSimulation.fromSnbt(door, TickSettleMode.InWorld, 15, -64, 0, "");
inc.useBlock(10, 4, 1);
inc.run(20);
const whole = JSON.parse(inc.changesJson());
expect(whole.length > 10, `the door recorded a backlog (${whole.length})`);
const tail = JSON.parse(inc.changesJsonFrom(whole.length - 5));
expect(tail.length === 5, `asking from n returns exactly the tail (${tail.length})`);
expect(
  JSON.stringify(tail) === JSON.stringify(whole.slice(-5)),
  "and it is the same entries changes_json would have given",
);
expect(JSON.parse(inc.changesJsonFrom(0)).length === whole.length, "from 0 is the whole log");
expect(JSON.parse(inc.changesJsonFrom(whole.length)).length === 0, "from the end is empty");
expect(
  JSON.parse(inc.changesJsonFrom(whole.length + 1000)).length === 0,
  "and past the end is empty, not an error",
);

console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail === 0 ? 0 : 1);
