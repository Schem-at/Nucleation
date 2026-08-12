/** Numbers, not opinions: the performance profile of the real app.
 *
 *      npm run build && npm run profile          # -> docs/profile-out.json
 *      EDA_PROFILE_TAG=before npm run profile    # tag a baseline
 *
 *  Drives the same `window.__eda` surface the verify script does, in headless
 *  chromium, and reports the four numbers the performance pass is judged on:
 *  time to first render, frame time while dragging, re-route latency, and the
 *  DRAW-CALL count for a scene of ~10 instances across 3 distinct cells (the
 *  instancing proof), plus per-phase timings and JS heap.
 */
import { chromium } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, "..");
const docs = path.join(root, "docs");
mkdirSync(docs, { recursive: true });

const TAG = process.env.EDA_PROFILE_TAG ?? "current";
const PORT = Number(process.env.EDA_PROFILE_PORT ?? 8463);
const server = spawn("npx", ["vite", "preview", "--port", String(PORT), "--strictPort"], {
  cwd: root, stdio: ["ignore", "pipe", "pipe"], shell: false,
});
server.stderr.on("data", (d) => process.env.EDA_DEBUG && console.error(String(d)));
const stop = () => { try { server.kill(); } catch { /* gone */ } };
process.on("exit", stop);

async function waitForServer(url, ms = 30_000) {
  const t0 = Date.now();
  while (Date.now() - t0 < ms) {
    try { await fetch(url); return; } catch { await new Promise((r) => setTimeout(r, 400)); }
  }
  throw new Error(`server never came up at ${url}`);
}

const round = (v, n = 1) => (typeof v === "number" ? Number(v.toFixed(n)) : v);
const out = { tag: TAG, when: new Date().toISOString() };

try {
  await waitForServer(`http://localhost:${PORT}/`);
  const browser = await chromium.launch({ args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 1680, height: 980 } });
  page.on("pageerror", (e) => console.error("pageerror:", e.message));

  // ---- 1. time to first render on demo load -------------------------------
  //
  // Wall clock from navigation to "the demo's blocks are on screen": engine
  // boot + wasm + library load + two cell placements + a routed bus + the
  // first scene build. The number a user experiences as "it opened".
  const t0 = Date.now();
  await page.goto(`http://localhost:${PORT}/?demo=1`, { waitUntil: "load" });
  await page.waitForFunction(() => window.__edaReady === true, null, { timeout: 120_000 });
  const readyMs = Date.now() - t0;
  await page.waitForFunction(() => window.__edaStudio().instances.size > 0, null, { timeout: 120_000 });
  await page.waitForFunction(() => window.__eda.profile().drawCalls > 0, null, { timeout: 60_000 });
  const firstRenderMs = Date.now() - t0;
  out.load = { engineReadyMs: readyMs, firstRenderMs };
  out.loadPhases = await page.evaluate(() => window.__eda.timings());

  // ---- 2. the ~10-instance / 3-cell scene: draw calls + mesh builds -------
  //
  // The instancing claim in one number. 10 placements of 3 distinct cells must
  // cost 3 cells' worth of geometry, not 10.
  const scene = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const cells = [...s.cells.keys()].filter((n) => /^(ADD007|REGISTER001|BINTOBCD001)/.test(n)).slice(0, 3);
    const pick = cells.length === 3 ? cells : [...s.cells.keys()].slice(0, 3);
    e.timingsReset();
    const before = e.meshBuilds();
    // 10 placements spread over 3 cells (4/3/3), on a grid clear of the demo.
    const plan = [];
    for (let i = 0; i < 10; i++) plan.push(pick[i % pick.length]);
    let k = 0;
    for (const cell of plan) {
      e.place(cell, [80 + 40 * (k % 4), -1, 80 + 40 * Math.floor(k / 4)]);
      k++;
    }
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    e.profileReset();
    await new Promise((r) => setTimeout(r, 500));
    return {
      cells: pick, placed: plan.length,
      instances: s.instances.size,
      meshBuilds: { before, after: e.meshBuilds() },
      profile: e.profile(),
      timings: e.timings(),
    };
  });
  out.scene10 = {
    distinctCells: scene.cells, placements: scene.placed, instances: scene.instances,
    drawCalls: scene.profile.drawCalls,
    triangles: scene.profile.triangles,
    geometries: scene.profile.geometries,
    sceneObjects: scene.profile.sceneObjects,
    markerObjects: scene.profile.markerObjects,
    meshBuilds: scene.meshBuilds,
    buildMs: Object.fromEntries(Object.entries(scene.timings).map(([k, v]) => [k, round(v.ms)])),
  };

  // ---- 3. frame time while dragging --------------------------------------
  //
  // 30 drag frames on one instance, the gesture the user complained about.
  // `dragFrameMs` is the synchronous work per frame (engine move + scene
  // update); `fps` is what the render loop actually achieved alongside it.
  // Both paths the checkbox selects, driven through the real pointer handler:
  //   preview  — adaptive re-route OFF: pure matrix writes until drop
  //   adaptive — adaptive re-route ON: cheap known routes may refresh after a
  //              pause; unknown/slow routes still commit exactly once on drop
  const dragOf = async (liveReroute) => page.evaluate(async (liveReroute) => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const inst = [...s.instances.values()][0];
    const at = [...inst.at];
    document.querySelector("#live-reroute").checked = liveReroute;
    e.timingsReset();
    e.profileReset();
    const meshBefore = e.meshBuilds();
    const readsBefore = e.sceneReads();
    const msBefore = e.sceneMs();
    let sync = 0;
    const t0 = performance.now();
    let final = at;
    for (let i = 0; i < 30; i++) {
      final = [at[0] + 1 + (i % 6), at[1], at[2] + 1 + (i % 5)];
      const t = performance.now();
      window.__edaDragMove("instance", inst.name, final);
      sync += performance.now() - t;
      await new Promise((r) => requestAnimationFrame(r));
    }
    const wallMs = performance.now() - t0;
    document.querySelector("#live-reroute").checked = true;
    window.__edaDrag("instance", inst.name, final);
    const result = {
      wallMs, frames: 30, syncMs: sync / 30,
      timings: structuredClone(e.timings()), profile: structuredClone(e.profile()),
      meshBuilds: { before: meshBefore, after: e.meshBuilds() },
      reads: { before: readsBefore, after: e.sceneReads() },
      engineMs: { before: msBefore, after: e.sceneMs() },
      routingPolicy: structuredClone(e.routingPolicy()),
    };
    // Keep the two profile modes comparable without including cleanup in the
    // captured numbers.
    window.__edaDrag("instance", inst.name, at);
    return result;
  }, liveReroute);
  const shape = (d) => ({
    frames: d.frames,
    pointerSyncMsAvg: round(d.syncMs, 3),
    engineCommits: d.timings.dragFrame?.n ?? 0,
    engineCommitMsAvg: round(d.timings.dragFrame ? d.timings.dragFrame.ms / d.timings.dragFrame.n : 0, 2),
    wallMsPerFrame: round(d.wallMs / d.frames),
    renderFps: round(d.profile.fps),
    renderFrameMsAvg: round(d.profile.avgFrameMs, 2),
    drawCalls: d.profile.drawCalls,
    cellRemeshes: d.meshBuilds.after.cells - d.meshBuilds.before.cells,
    matrixWrites: d.meshBuilds.after.matrixWrites - d.meshBuilds.before.matrixWrites,
    wasmReads: Object.fromEntries(Object.keys(d.reads.after)
      .map((k) => [k, d.reads.after[k] - d.reads.before[k]])),
    /** Per-FRAME ms inside each engine read, so the remaining floor has a name. */
    wasmReadMsPerFrame: Object.fromEntries(Object.keys(d.engineMs.after)
      .map((k) => [k, round((d.engineMs.after[k] - d.engineMs.before[k]) / d.frames, 2)])),
    phases: Object.fromEntries(Object.entries(d.timings)
      .map(([k, v]) => [k, { n: v.n, msAvg: round(v.ms / Math.max(v.n, 1), 2) }])),
    routingPolicy: d.routingPolicy,
  });
  const drag = await dragOf(false);
  out.dragPreview = shape(drag);
  out.dragLiveReroute = shape(await dragOf(true));
  // Back-compat key for the before/after table.
  out.drag = out.dragLiveReroute;

  // ---- 4. re-route latency ------------------------------------------------
  //
  // The number that decides whether the 250 ms live-reroute throttle can go.
  const reroute = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const bus = [...s.buses.keys()][0];
    if (!bus) return { skipped: true };
    const runs = [];
    for (let i = 0; i < 5; i++) {
      const t0 = performance.now();
      s.rerouteBus(bus);
      runs.push(performance.now() - t0);
      await new Promise((r) => requestAnimationFrame(r));
    }
    return { bus, state: s.busState(bus), runs };
  });
  out.reroute = reroute.skipped ? { skipped: true } : {
    bus: reroute.bus, state: reroute.state,
    msAvg: round(reroute.runs.reduce((a, b) => a + b, 0) / reroute.runs.length),
    msMin: round(Math.min(...reroute.runs)),
    msMax: round(Math.max(...reroute.runs)),
  };

  // ---- 5. port-mode toggle: how much of the scene does one cell edit cost? -
  const toggle = await page.evaluate(async () => {
    const e = window.__eda;
    const p = e.endpoints().find((q) => q.instance && !q.routable && q.promotable);
    if (!p) return { skipped: true };
    e.timingsReset();
    const before = e.meshBuilds();
    e.setPortMode(p.instance, p.port, "bus");
    await new Promise((r) => requestAnimationFrame(r));
    const mid = e.meshBuilds();
    e.setPortMode(p.instance, p.port, "executor");
    return { port: p.name, timings: e.timings(), meshBuilds: { before, mid, after: e.meshBuilds() } };
  });
  out.portModeToggle = toggle.skipped ? { skipped: true } : {
    port: toggle.port,
    meshBuilds: toggle.meshBuilds,
    ms: Object.fromEntries(Object.entries(toggle.timings).map(([k, v]) => [k, round(v.ms)])),
  };

  // ---- 6. memory ----------------------------------------------------------
  const mem = await page.evaluate(() => {
    const m = performance.memory;
    const p = window.__eda.profile();
    return {
      jsHeapMB: m ? round2(m.usedJSHeapSize / 1048576) : null,
      geometries: p.geometries, textures: p.textures, programs: p.programs,
      sceneObjects: p.sceneObjects, markerObjects: p.markerObjects, labels: p.labels,
    };
    function round2(v) { return Number(v.toFixed(1)); }
  });
  out.memory = mem;

  await browser.close();
} finally {
  stop();
}

const file = path.join(docs, `profile-${TAG}.json`);
writeFileSync(file, JSON.stringify(out, null, 2));
console.log(JSON.stringify(out, null, 2));
console.log(`\nwrote ${path.relative(root, file)}`);
