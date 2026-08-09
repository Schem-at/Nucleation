/** Headless smoke: exercises the exact wasm calls the studio makes.
 *
 *  1. build endpoint hardware with raw setBlock (design_demo2 geometry),
 *  2. declare typed ports, route a bus, strict-check, bake,
 *  3. typed walking-ones through the embedded contract (CellExecutor),
 *  4. .nucm document roundtrip (toNucmB64 -> fromNucm -> still editable),
 *  5. Hdl.compileBlif + contract on the checked-in cmp4.blif,
 *  6. move_gate drag reroute report.
 *
 *  Run:  node scripts/smoke.mjs            (needs ../../dist/npm-eda or
 *        a synced public/engine — see package.json sync-engine)
 */
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const candidates = [
  path.join(here, "..", "public", "engine", "index.mjs"),
  path.join(here, "..", "..", "..", "dist", "npm-eda", "index.mjs"),
];
const enginePath = candidates.find((p) => existsSync(p));
if (!enginePath) {
  console.error("no engine found; run: npm run sync-engine (or tools/package-npm.sh dist/npm-eda)");
  process.exit(2);
}
const core = await import(`file://${enginePath}`);
const { veneer } = await import(
  `file://${path.join(path.dirname(enginePath), "veneer", "design.mjs")}`
);
const { Design, Executor, Gate, Style } = veneer(core);

let good = 0, total = 0;
const check = (ok, label) => {
  total++;
  good += !!ok;
  console.log(`${ok ? "PASS" : "FAIL"} ${label}`);
  return ok;
};

// -- 1. endpoint hardware (demo2 lever/lamp banks, one 8-bit bus) ----------
const STONE = "minecraft:stone";
const DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";
const LAMP = "minecraft:redstone_lamp[lit=false]";
const LEVER = "minecraft:lever[face=floor,facing=north,powered=false]";
const N = 8, STEP = [0, 2, 0];

const s = core.Schematic.create("smoke");
for (let i = 0; i < N; i++) {
  const y = 2 + 2 * i;
  s.setBlockFromString(0, y - 1, 8, STONE);
  s.setBlockFromString(0, y, 8, LEVER);
  s.setBlockFromString(1, y - 1, 8, STONE);
  s.setBlockFromString(1, y, 8, DUST);
  s.setBlockFromString(16, y - 1, 8, LAMP);
  s.setBlockFromString(16, y, 8, DUST);
}

const d = Design.forSchematic("smoke", s);
d.declareInput("a_in", { anchor: [1, 2, 8], step: STEP, width: N });
d.declareOutput("a_out", { anchor: [16, 2, 8], step: STEP, width: N });
const bus = d.routeBus("bus_a", {
  driver: "a_in",
  sinks: ["a_out"],
  gates: [Gate([8, 2, 8], STEP)],
  style: Style({ busBlock: "minecraft:lime_concrete" }),
});
check(bus.state === "routed", `bus routed (${bus.state})`);

// -- 2. strict check + bake ------------------------------------------------
d.check({ strict: true });
check(true, "check strict clean");
const baked = d.bake(4000);
check(baked.raw.blockCount() > 0, "baked artifact non-empty");

// -- 3. typed walking-ones -------------------------------------------------
const ex = baked.executor();
let walked = 0;
for (let i = 0; i < N; i++) {
  ex.set("a_in", 1 << i);
  ex.settle(400);
  walked += ex.get("a_out") === 1 << i;
}
check(walked === N, `walking-ones ${walked}/${N}`);

// -- 4. .nucm document roundtrip ------------------------------------------
const nucm = d.toBytes("x.nucm");
check(nucm.length > 4, `nucm bytes (${nucm.length})`);
const d2 = Design.fromNucm(nucm);
check(d2.busState("bus_a") === "routed", "reloaded design keeps bus state");
const moved = d2.moveGate("bus_a", "g0", [8, 2, 12]);
check(moved.state === "routed" && moved.rerouted_segments === 2,
  `reloaded design still drags (state=${moved.state}, segs=${moved.rerouted_segments})`);

// -- 5. gate drag on the live design --------------------------------------
const report = bus.moveGate(0, [8, 2, 4]);
check(report.state === "routed" && report.rerouted_segments === 2,
  `gate drag reroutes exactly 2 segments (${JSON.stringify(report)})`);

// -- 6. HDL compile on the checked-in BLIF --------------------------------
const blif = readFileSync(path.join(here, "..", "testdata", "cmp4.blif"), "utf8");
const cell = core.Hdl.compileBlif(blif, "cmp4", false);
check(cell.blockCount() > 0, `Hdl.compileBlif blocks=${cell.blockCount()}`);
const contract = JSON.parse(core.Hdl.compileBlifContract(blif, "cmp4"));
check(!!contract.io, `Hdl.compileBlifContract has io (${Object.keys(contract).join(",")})`);
cell.setCellContractJson(JSON.stringify(contract));
const resolved = JSON.parse(cell.resolveCellContractJson());
check(resolved.contract?.name === "cmp4", "contract embeds + autodetects");
const dd = Design.create("lib");
const warnings = dd.addCell("cmp4", cell);
check(Array.isArray(JSON.parse(warnings)), `addCell accepts hdl cell (warnings=${warnings})`);
dd.place("u0", "cmp4", [0, 0, 0], 0);
const mv = dd.moveInstance("u0", [4, 0, 4]);
check(Array.isArray(mv.rerouted), `moveInstance report ${JSON.stringify(mv)}`);

console.log(`smoke: ${good}/${total}`);
process.exit(good === total ? 0 : 1);
