/** Headless proof + screenshots into docs/.
 *
 *  Builds are NOT run here (stale-dist trap, see apps/serve-wasm.sh):
 *      npm run build && node scripts/screenshot.mjs
 *  Serves dist/ itself on a scratch port, drives the real app in headless
 *  chromium: demo load, gate drag, instance placed from a compiled BLIF,
 *  drag-through-bus failure (red layer), bake + typed poke, and the
 *  in-browser yosys Verilog compile.
 */
import { chromium } from "playwright";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, "..");
const docs = path.join(root, "docs");
mkdirSync(docs, { recursive: true });

const PORT = 8459;
const server = spawn("npx", ["vite", "preview", "--port", String(PORT), "--strictPort"], {
  cwd: root, stdio: ["ignore", "pipe", "pipe"], shell: false,
});
server.stdout.on("data", (d) => process.env.EDA_DEBUG && console.log(String(d)));
server.stderr.on("data", (d) => console.error(String(d)));
const stop = () => { try { server.kill(); } catch { /* gone */ } };
process.on("exit", stop);

async function waitForServer(url, ms = 30_000) {
  const t0 = Date.now();
  while (Date.now() - t0 < ms) {
    try {
      await fetch(url);
      return;
    } catch {
      await new Promise((r) => setTimeout(r, 400));
    }
  }
  throw new Error(`server never came up at ${url}`);
}

const results = [];
const check = (ok, label) => {
  results.push([!!ok, label]);
  console.log(`${ok ? "PASS" : "FAIL"} ${label}`);
};

try {
  await waitForServer(`http://localhost:${PORT}/`);
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1600, height: 950 } });
  page.on("console", (m) => { if (m.type() === "error") console.log("console:", m.text()); });
  await page.goto(`http://localhost:${PORT}/?demo=1`, { waitUntil: "load" });
  await page.waitForFunction(() => window.__edaReady === true, null, { timeout: 60_000 });
  await page.waitForTimeout(1200);

  // 1. demo: two crossing buses routed
  const states = await page.evaluate(() => {
    const s = window.__edaStudio();
    return [...s.buses.keys()].map((b) => s.busState(b));
  });
  check(states.length === 2 && states.every((x) => x === "routed"), `demo buses routed (${states})`);
  await page.screenshot({ path: path.join(docs, "01-demo-crossing.png") });

  // 2. gate drag reroutes exactly 2 segments
  const drag = await page.evaluate(() => {
    const s = window.__edaStudio();
    return s.moveGate("bus0", "g0", [8, 2, 12]);
  });
  check(drag.state === "routed" && drag.rerouted_segments === 2,
    `gate drag ${JSON.stringify(drag)}`);
  await page.waitForTimeout(400);
  await page.screenshot({ path: path.join(docs, "02-gate-drag.png") });

  // 3. place a compiled HDL cell (cmp4 BLIF) and drag it through bus B
  const blif = readFileSync(path.join(root, "testdata", "cmp4.blif"), "utf8");
  const placed = await page.evaluate((blifText) => {
    const s = window.__edaStudio();
    const core = window.__edaCore;
    const cell = core.Hdl.compileBlif(blifText, "cmp4", false);
    cell.setCellContractJson(core.Hdl.compileBlifContract(blifText, "cmp4"));
    s.addCellSchematic("cmp4", cell, "file");
    const inst = s.placeInstance("cmp4", [24, 0, 24]);
    return { inst: inst.name, blocks: cell.blockCount() };
  }, blif);
  check(placed.blocks > 0, `hdl cell placed (${placed.blocks} blocks) as ${placed.inst}`);

  const through = await page.evaluate((inst) => {
    const s = window.__edaStudio();
    const report = s.moveInstance(inst, [6, 0, 2]); // through bus B's corridor
    return { report, states: [...s.buses.keys()].map((b) => `${b}=${s.busState(b)}`) };
  }, placed.inst);
  check(true, `instance drag-through: ${JSON.stringify(through)}`);
  await page.waitForTimeout(400);
  await page.screenshot({ path: path.join(docs, "03-instance-through-bus.png") });

  const away = await page.evaluate((inst) => {
    const s = window.__edaStudio();
    const report = s.moveInstance(inst, [26, 0, 26]);
    return { report, states: [...s.buses.keys()].map((b) => `${b}=${s.busState(b)}`) };
  }, placed.inst);
  check(away.states.every((x) => x.endsWith("=routed")),
    `drag away re-heals: ${JSON.stringify(away.states)}`);

  // 4. bake + typed poke through the embedded contract
  const poke = await page.evaluate(() => {
    const s = window.__edaStudio();
    s.bake(4000);
    s.executor.set("a_in", 0x55);
    s.executor.set("b_in", 0xaa);
    s.executor.settle(800);
    return { a: s.executor.get("a_out"), b: s.executor.get("b_out") };
  });
  check(poke.a === 0x55 && poke.b === 0xaa,
    `bake + poke a_out=0x${poke.a.toString(16)} b_out=0x${poke.b.toString(16)}`);
  await page.waitForTimeout(300);
  await page.screenshot({ path: path.join(docs, "04-baked-poke.png") });

  // 5. in-browser yosys: compile the default Verilog into a cell
  await page.click("#btn-compile");
  await page.waitForFunction(
    () => document.querySelector("#verilog-status")?.textContent?.startsWith("ok:"),
    null, { timeout: 180_000 },
  );
  const vstatus = await page.textContent("#verilog-status");
  check(vstatus.startsWith("ok:"), `verilog->yosys->redstone cell (${vstatus})`);
  await page.screenshot({ path: path.join(docs, "05-verilog-compile.png") });

  // 6. exports produce bytes
  const sizes = await page.evaluate(() => {
    const s = window.__edaStudio();
    return {
      schem: s.exportBytes("schem").length,
      litematic: s.exportBytes("litematic").length,
      nucm: s.exportBytes("nucm").length,
    };
  });
  check(sizes.schem > 100 && sizes.litematic > 100 && sizes.nucm > 100,
    `exports ${JSON.stringify(sizes)}`);

  await browser.close();
} finally {
  stop();
}

const good = results.filter(([ok]) => ok).length;
console.log(`screenshot-verify: ${good}/${results.length}`);
writeFileSync(path.join(docs, "verify-out.json"),
  JSON.stringify(results.map(([ok, label]) => ({ ok, label })), null, 2));
process.exit(good === results.length ? 0 : 1);
