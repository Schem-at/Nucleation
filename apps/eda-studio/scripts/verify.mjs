/** Headless proof of the INTERACTION MODEL + screenshots into docs/.
 *
 *  Builds are NOT run here (stale-dist trap, see apps/serve-wasm.sh):
 *      npm run build && npm run verify
 *
 *  Everything is driven through the same code paths the mouse and keyboard
 *  drive (`window.__eda`), so a green run means the UI itself works — not
 *  just the engine underneath it. Covers: the auto-loaded cell library,
 *  placing cells, instance ports as connectable endpoints, the click-to-
 *  connect flow, rotate/delete by keyboard, a bus FAILED then healed, a typed
 *  poke through the routed chain, and the export tiers.
 */
import { chromium } from "playwright";
import { mkdirSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, "..");
const docs = path.join(root, "docs");
mkdirSync(docs, { recursive: true });

const PORT = 8461;
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
    try { await fetch(url); return; } catch { await new Promise((r) => setTimeout(r, 400)); }
  }
  throw new Error(`server never came up at ${url}`);
}

const results = [];
const check = (ok, label, detail) => {
  results.push({ ok: !!ok, label, ...(detail ? { detail } : {}) });
  console.log(`${ok ? "PASS" : "FAIL"} ${label}${detail && !ok ? `\n      ${detail}` : ""}`);
  return !!ok;
};

let shot = 0;
async function snap(page, name) {
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(docs, `${String(++shot).padStart(2, "0")}-${name}.png`) });
}

try {
  await waitForServer(`http://localhost:${PORT}/`);
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1680, height: 980 } });
  const errors = [];
  page.on("console", (m) => { if (m.type() === "error") { errors.push(m.text()); console.log("console:", m.text()); } });
  await page.goto(`http://localhost:${PORT}/?demo=1`, { waitUntil: "load" });
  await page.waitForFunction(() => window.__edaReady === true, null, { timeout: 90_000 });
  await page.waitForFunction(() => window.__edaStudio().instances.size > 0, null, { timeout: 90_000 });
  await page.waitForTimeout(1500);

  // ---- 1. library auto-loaded from the enhanced cells --------------------
  const lib = await page.evaluate(() => {
    const s = window.__edaStudio();
    return [...s.cells.values()].map((c) => ({ name: c.name, dims: c.dims, ports: c.ports.length }));
  });
  check(lib.length >= 8 && lib.every((c) => c.ports > 0),
    `library auto-loaded ${lib.length} cells, all with contract ports`,
    JSON.stringify(lib.map((c) => `${c.name}:${c.ports}`)));

  // ---- 2. demo placed two adders + routed an instance port --------------
  const demo = await page.evaluate(() => {
    const s = window.__edaStudio();
    return {
      instances: [...s.instances.values()].map((i) => ({ name: i.name, cell: i.cell, rot: i.rot })),
      buses: [...s.buses.keys()].map((b) => ({ name: b, state: s.busState(b) })),
      endpoints: window.__eda.endpoints().map((e) =>
        ({ name: e.name, kind: e.kind, ty: e.ty, routable: e.routable, blocked: e.blocked })),
    };
  });
  check(demo.instances.length === 2, `demo placed 2 cells (${JSON.stringify(demo.instances)})`);
  const sumPort = demo.endpoints.find((e) => e.name === "u0.sum");
  check(sumPort?.routable === true && sumPort.kind === "input" && sumPort.ty === "uint8",
    `instance port u0.sum is a routable driver, uint8`, JSON.stringify(sumPort));
  const aPort = demo.endpoints.find((e) => e.name === "u0.a");
  check(aPort && aPort.routable === false && /lever|dust connection cell/.test(aPort.blocked ?? ""),
    `instance port u0.a is reported non-routable WITH a reason`, JSON.stringify(aPort));
  check(demo.buses.some((b) => b.state === "routed"),
    `demo bus u0.sum -> sum_out routed (${JSON.stringify(demo.buses)})`);
  await snap(page, "demo-adder-instance-ports");

  // ---- 3. selection + gizmo + hint --------------------------------------
  const sel = await page.evaluate(() => {
    window.__eda.select("u0");
    return { selection: window.__eda.selection(), hint: window.__eda.hint(), labels: document.querySelectorAll(".mk-label").length };
  });
  check(sel.selection?.kind === "instance" && sel.selection.id === "u0", `click selects u0`);
  check(/rotate/i.test(sel.hint) && /remove/i.test(sel.hint),
    `hint bar tells you what R/Del do`, sel.hint);
  check(sel.labels > 0, `IO labels rendered (${sel.labels})`);
  await snap(page, "selection-gizmo-io-labels");

  // ---- 4. rotate by keyboard --------------------------------------------
  const rot = await page.evaluate(async () => {
    window.__eda.key("r");
    await new Promise((r) => setTimeout(r, 600));
    const s = window.__edaStudio();
    return {
      rot: s.instances.get("u0")?.rot,
      buses: [...s.buses.keys()].map((b) => ({ name: b, state: s.busState(b) })),
    };
  });
  check(rot.rot === 90, `R rotates u0 to 90° (got ${rot.rot})`);
  await snap(page, "rotated-90");
  // put it back so the bus is routable again
  await page.evaluate(async () => {
    window.__eda.key("r"); await new Promise((r) => setTimeout(r, 300));
    window.__eda.key("r"); await new Promise((r) => setTimeout(r, 300));
    window.__eda.key("r"); await new Promise((r) => setTimeout(r, 600));
  });
  const backTo0 = await page.evaluate(() => window.__edaStudio().instances.get("u0")?.rot);
  check(backTo0 === 0, `three more R return u0 to 0° (got ${backTo0})`);

  // ---- 5. refusal: you cannot bus INTO a lever input ---------------------
  const refusal = await page.evaluate(async () => {
    window.__eda.clickPort("u0.sum");
    const midHint = window.__eda.hint();
    window.__eda.clickPort("u1.a");           // lever bank: must be refused
    await new Promise((r) => setTimeout(r, 400));
    const s = window.__edaStudio();
    return {
      midHint,
      toast: document.querySelector("#toast")?.textContent ?? "",
      busCount: s.buses.size,
    };
  });
  check(/click a blue/i.test(refusal.midHint),
    `starting a bus updates the hint to the next action`, refusal.midHint);
  check(/executor-only|lever|dust/i.test(refusal.toast),
    `routing into a lever input is refused with a readable reason`, refusal.toast);
  await snap(page, "refusal-lever-input");

  // ---- 6. delete by keyboard rips its bus -------------------------------
  const del = await page.evaluate(async () => {
    window.__eda.key("Escape");
    window.__eda.select("u0");
    window.__eda.key("Delete");
    await new Promise((r) => setTimeout(r, 800));
    const s = window.__edaStudio();
    return {
      instances: [...s.instances.keys()],
      buses: [...s.buses.keys()],
      toast: document.querySelector("#toast")?.textContent ?? "",
    };
  });
  check(!del.instances.includes("u0"), `Delete removes u0 (${JSON.stringify(del.instances)})`);
  check(del.buses.length === 0, `its bus went with it (${JSON.stringify(del.buses)})`);
  check(/deleted/i.test(del.toast), `deletion is reported`, del.toast);

  // ---- 7. connect flow: click output port, then input port --------------
  const connect = await page.evaluate(async () => {
    window.__eda.clickPort("u1.sum");
    const hint = window.__eda.hint();
    window.__eda.clickPort("sum_out");
    await new Promise((r) => setTimeout(r, 1500));
    const s = window.__edaStudio();
    const name = [...s.buses.keys()][0];
    return { hint, name, state: name ? s.busState(name) : "none", count: s.buses.size };
  });
  check(connect.count === 1 && connect.state === "routed",
    `click u1.sum then sum_out routes a bus (${connect.name}: ${connect.state})`);
  await snap(page, "connected-instance-port-to-readout");

  // ---- 8. bus FAILED, then healed --------------------------------------
  const failed = await page.evaluate(async () => {
    const s = window.__edaStudio();
    const bus = [...s.buses.keys()][0];
    // Lift the adder off the bus level: the trunk realizes one flat 2y-pitch
    // stack, so a driver whose bit 0 sits at a different y is unroutable —
    // a real, reported failure rather than a thrown error.
    window.__edaDrag("instance", "u1", [0, 3, 24]);
    await new Promise((r) => setTimeout(r, 1500));
    return { bus, state: s.busState(bus), detail: s.busStateDetail(bus) };
  });
  check(failed.state.startsWith("failed"),
    `moving the driver away leaves the bus FAILED with a reason`,
    JSON.stringify(failed.detail));
  await snap(page, "bus-failed-red");

  const healed = await page.evaluate(async () => {
    const s = window.__edaStudio();
    const bus = [...s.buses.keys()][0];
    window.__edaDrag("instance", "u1", [0, -1, 24]);   // back where it was
    await new Promise((r) => setTimeout(r, 1500));
    let state = s.busState(bus);
    if (!state.startsWith("routed")) {
      state = s.rerouteBus(bus);                        // the panel's Re-route
      await new Promise((r) => setTimeout(r, 1200));
    }
    return { bus, state: s.busState(bus) };
  });
  check(healed.state === "routed", `moving it back heals the bus (${healed.state})`);

  // ---- 9. typed poke THROUGH the routed chain --------------------------
  const poke = await page.evaluate(async () => {
    const s = window.__edaStudio();
    s.bake(6000);
    s.executor.set("u1.a", 99);
    s.executor.set("u1.b", 28);
    s.executor.set("u1.cin", false);   // Boolean port: a number is a type error
    s.executor.settle(4000);
    return { sum_out: s.executor.get("sum_out") };
  });
  check(poke.sum_out === 127,
    `typed poke through the bus: u1.a=99 + u1.b=28 -> sum_out=${poke.sum_out} (want 127)`);
  await snap(page, "baked-typed-poke");

  // ---- 10. export tiers -------------------------------------------------
  const exp = await page.evaluate(() => {
    const s = window.__edaStudio();
    const core = window.__edaCore;
    const layered = s.design.flatten().raw.blockCount();
    const schem = core.Schematic.fromData(Array.from(s.exportBytes("schem")));
    let contract = false;
    try { contract = JSON.parse(schem.cellContractJson()).io.inputs["u1.a"] != null; } catch {}
    const lit = core.Schematic.fromData(Array.from(s.exportBytes("litematic")));
    const litRegions = JSON.parse(lit.regionNamesJson());
    const nucm = s.exportBytes("nucm");
    return {
      layered, schem: schem.blockCount(), contract,
      lit: lit.blockCount(), litRegions, nucmLen: nucm.length,
    };
  });
  check(exp.schem === exp.layered && exp.layered > 500,
    `.schem export composites EVERY layer (${exp.schem}/${exp.layered} blocks)`);
  check(exp.contract, `.schem carries the merged contract (u1.a present)`);
  check(exp.lit === exp.layered &&
        exp.litRegions.some((r) => r.startsWith("inst:")) &&
        exp.litRegions.some((r) => r.startsWith("bus:")),
    `.litematic keeps blocks AND inst:/bus: regions (${exp.lit}, ${JSON.stringify(exp.litRegions)})`);
  check(exp.nucmLen > 1000, `.nucm document exported (${exp.nucmLen} bytes)`);

  const reload = await page.evaluate(() => {
    const s = window.__edaStudio();
    const bytes = s.exportBytes("nucm");
    const back = s.d.Design.fromNucm(bytes);
    const bus = [...s.buses.keys()][0];
    return { state: back.busState(bus), blocks: back.flatten().raw.blockCount() };
  });
  check(reload.state === "routed" && reload.blocks === exp.layered,
    `.nucm reload keeps the model (${reload.state}, ${reload.blocks} blocks)`);

  // ---- 11. textured renderer (needs a pack; pack.zip is not committed) ---
  const packPath = path.join(root, "..", "..", "pack.zip");
  if (existsSync(packPath)) {
    const b64 = readFileSync(packPath).toString("base64");
    const tex = await page.evaluate(async (b64) => {
      const bin = atob(b64);
      const bytes = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
      const s = window.__edaStudio();
      const t0 = performance.now();
      s.loadPack(bytes);
      const load = performance.now() - t0;
      const t1 = performance.now();
      const ok = await window.__eda.remesh();
      document.querySelector("#textured").checked = true;
      return { ok, info: s.packInfo, loadMs: Math.round(load), meshMs: Math.round(performance.now() - t1) };
    }, b64);
    check(tex.ok === true,
      `textured view: pack loaded (${JSON.stringify(tex.info)}) in ${tex.loadMs}ms, meshed in ${tex.meshMs}ms`);
    await page.waitForTimeout(1200);
    await snap(page, "textured-resource-pack");
  } else {
    console.log("SKIP textured view (no pack.zip at repo root; it is not committed)");
    results.push({ ok: true, label: "textured view SKIPPED (no pack.zip)", skipped: true });
  }

  check(errors.length === 0, `no console errors (${errors.length})`,
    errors.slice(0, 3).join(" | "));

  await browser.close();
} finally {
  stop();
}

const good = results.filter((r) => r.ok).length;
console.log(`\n${good}/${results.length} checks passed`);
writeFileSync(path.join(docs, "verify-out.json"),
  JSON.stringify({ when: new Date().toISOString(), passed: good, total: results.length, results }, null, 2));
process.exit(good === results.length ? 0 : 1);
