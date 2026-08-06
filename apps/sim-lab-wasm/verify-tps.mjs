/** How fast can the lab actually tick, and where does it stop keeping up?
 *
 * Two separate numbers, and conflating them is the trap. The *engine ceiling*
 * is how many ticks a second `sim.step()` can retire with nothing else running
 * — measured by stepping in a tight loop. The *achieved rate* is what the lab
 * delivers with meshing, entity sync and rendering competing for the same
 * frame. The second is always lower, and the gap is the render cost.
 *
 * Usage: node verify-tps.mjs <schematic>
 */
import { chromium } from "playwright";

const b = await chromium.launch();
const p = await b.newPage();
const errs = [];
p.on("pageerror", (e) => errs.push(String(e).slice(0, 160)));
p.on("console", (m) => { if (m.type() === "error") errs.push(m.text().slice(0, 160)); });
await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready, .status.error", { timeout: 300000 });

// Optionally start the machine first. A quiescent build retires empty ticks
// at a rate that says nothing about the engine, so a number measured on one
// is not a throughput figure — it is the cost of asking "is there anything to
// do?" ten million times.
//   --break x,y,z   remove a block (the flying machines start this way)
//   --use   x,y,z   right-click one (levers)
const kickArg = (flag) => {
  const i = process.argv.indexOf(flag);
  return i > 0 ? process.argv[i + 1].split(",").map(Number) : null;
};
const brk = kickArg("--break");
const use = kickArg("--use");
if (brk || use) {
  await p.evaluate(({ brk, use }) => {
    const w = window.simlab.world;
    if (brk) w.sim.placeBlock(brk[0], brk[1], brk[2], "minecraft:air");
    if (use) w.sim.useBlock(use[0], use[1], use[2]);
    w.applyChanges(w.drainChanges(), 0);
  }, { brk, use });
}

const loadAndKick = async () => {
  await p.setInputFiles("input[type=file]", process.argv[2]);
  await p.waitForSelector(".status.ready, .status.error", { timeout: 300000 });
  if (brk || use) {
    await p.evaluate(({ brk, use }) => {
      const w = window.simlab.world;
      if (brk) w.sim.placeBlock(brk[0], brk[1], brk[2], "minecraft:air");
      if (use) w.sim.useBlock(use[0], use[1], use[2]);
      w.applyChanges(w.drainChanges(), 0);
    }, { brk, use });
  }
};

// The ceiling: no rendering, no change drain, just the engine. Reported
// alongside how much of it was real work, because the two are not comparable.
const ceiling = await p.evaluate(() => {
  const w = window.simlab.world;
  for (let i = 0; i < 200; i++) w.sim.step(); // warm the JIT before timing
  const t0 = performance.now();
  let n = 0;
  let active = 0;
  while (performance.now() - t0 < 1000) {
    for (let i = 0; i < 200; i++) {
      w.sim.step();
      if (!w.sim.isQuiescent?.()) active++;
    }
    n += 200;
  }
  const secs = (performance.now() - t0) / 1000;
  // The `isQuiescent` calls are themselves part of the measured loop, so this
  // number is a floor on the ceiling — good enough to tell idle from busy.
  return { tps: Math.round(n / secs), ticks: n, activeFraction: active / n };
});
const busy = ceiling.activeFraction > 0.01;
console.log(
  `engine ceiling: ${ceiling.tps.toLocaleString()} tps  ` +
    (busy
      ? `(${(ceiling.activeFraction * 100).toFixed(0)}% of ticks had work pending)`
      : `⚠ build is QUIESCENT — this is idle-tick throughput, not a workload figure`),
);

// Reload before the sweep. The ceiling run just retired thousands of ticks,
// which on a flying machine means it travelled thousands of blocks and grew
// the region to match — sweeping on that state would measure a world the
// ceiling run created, not the one the user loads.

// The slider is not linear, so drive it by travel and read back what the UI
// says it asked for — that also proves the mapping is what it claims.
const setSlider = async (pos) => {
  await p.locator(".rate input").fill(String(pos));
  await p.waitForTimeout(50);
  return (await p.textContent(".rate"))?.trim().split("\n")[0].trim();
};

console.log("\nslider  target       achieved     throttled");
const rows = [];
for (const pos of [60, 150, 300, 450, 600, 700, 800, 900, 1000]) {
  // Fresh build per row. A flying machine left running between rows travels
  // and grows its region, so row nine would be measuring a world the earlier
  // rows built rather than the one under test.
  await p.reload({ waitUntil: "networkidle" });
  await loadAndKick();
  const target = await setSlider(pos);
  if (!(await p.locator("header button").first().textContent())?.includes("pause")) {
    await p.locator("header button").first().click();
  }
  await p.waitForTimeout(2900); // 2 tps needs a >2 s measurement window
  const readout = (await p.textContent(".rate-actual"))?.trim() ?? "";
  const m = readout.match(/([\d.,]+)(k?) actual/);
  const achieved = m ? parseFloat(m[1].replace(/,/g, "")) * (m[2] === "k" ? 1000 : 1) : 0;
  const isThrottled = readout.includes("throttled");
  rows.push({ pos, target, achieved, isThrottled });
  console.log(
    `${String(pos).padStart(4)}   ${String(target).padEnd(11)}  ${String(achieved.toLocaleString()).padStart(9)}    ${isThrottled ? "yes" : "no"}`,
  );
}
// Leave it stopped so the page is idle when the probe exits.
if ((await p.locator("header button").first().textContent())?.includes("pause")) {
  await p.locator("header button").first().click();
}

const knee = rows.find((r) => r.isThrottled);
console.log(
  knee
    ? `\nthrottles from slider ${knee.pos} (${knee.target}) — sustained ceiling in-app ≈ ${Math.max(...rows.map((r) => r.achieved)).toLocaleString()} tps`
    : `\nnever throttled — peak ${Math.max(...rows.map((r) => r.achieved)).toLocaleString()} tps`,
);

// Low rates must be *exact*: the point of the fine regime is landing on 3 tps
// and getting 3. Anything under 20 that misses by more than a tick a second is
// a scheduling bug, not a performance limit.
const fine = rows.filter((r) => {
  const t = parseFloat(r.target);
  return t > 0 && t <= 20 && !r.target.includes("k") && r.target !== "max";
});
const sloppy = fine.filter((r) => Math.abs(r.achieved - parseFloat(r.target)) > 1.5);
console.log(
  sloppy.length
    ? `❌ fine regime inaccurate: ${sloppy.map((r) => `${r.target}→${r.achieved}`).join(", ")}`
    : `✅ fine regime accurate (${fine.map((r) => r.target).join(", ")} tps all hit)`,
);
console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
await b.close();
process.exit(sloppy.length === 0 && errs.length === 0 ? 0 : 1);
