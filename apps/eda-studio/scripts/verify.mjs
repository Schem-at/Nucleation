/** Headless proof of the INTERACTION MODEL + screenshots into docs/.
 *
 *  Builds are NOT run here (stale-dist trap, see apps/serve-wasm.sh):
 *      npm run build && npm run verify
 *
 *  Everything is driven through the same code paths the mouse and keyboard
 *  drive (`window.__eda`), so a green run means the UI itself works — not
 *  just the engine underneath it.
 *
 *  Three parts:
 *
 *    0. the first thirty seconds — the DEFAULT landing state (the verified
 *       chain, framed, buses routed), the coach, the `?` legend, the grouped
 *       outliner, label declutter, undo/redo, destructive confirms, toasts;
 *    1. the interaction model on the two-adder demo — instance ports, connect,
 *       rotate/delete, a bus FAILED (as a SENTENCE, with a focus target) then
 *       healed, a typed poke, the export tiers, auto-promotion, and the
 *       performance contract;
 *    2. instancing at scale, on a page with `?empty=1` so nothing else is in
 *       the scene.
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
  const file = `${String(++shot).padStart(2, "0")}-${name}.png`;
  await page.screenshot({ path: path.join(docs, file) });
  return file;
}

try {
  await waitForServer(`http://localhost:${PORT}/`);
  const browser = await chromium.launch();

  // ======================================================================
  // PART 0 — the first thirty seconds.
  //
  // The default landing state has to teach the model with no reading: a
  // WORKING design, and a coach that names the four things you do to it. These
  // checks are on their own page because "what you get with no URL flags" is
  // exactly what they are about.
  // ======================================================================
  const p0 = await browser.newPage({ viewport: { width: 1680, height: 980 } });
  const p0errors = [];
  p0.on("console", (m) => { if (m.type() === "error") p0errors.push(m.text()); });
  await p0.goto(`http://localhost:${PORT}/`, { waitUntil: "load" });
  await p0.waitForFunction(() => window.__edaReady === true, null, { timeout: 120_000 });
  await p0.waitForTimeout(1200);

  const landing = await p0.evaluate(() => {
    const s = window.__edaStudio();
    return {
      instances: [...s.instances.values()].map((i) => ({ name: i.name, cell: i.cell, at: i.at })),
      buses: [...s.buses.keys()].map((b) => ({ name: b, state: s.busState(b) })),
      coach: window.__eda.coach(),
      hint: window.__eda.hint(),
      empty: window.__eda.emptyState(),
      history: window.__eda.history(),
    };
  });
  check(landing.instances.length >= 2 &&
        landing.instances.some((i) => /ADD007/.test(i.cell)) &&
        landing.instances.some((i) => /BINTOBCD001/.test(i.cell)),
    `the DEFAULT page lands on the verified chain, not an empty grid ` +
    `(${landing.instances.map((i) => `${i.name}:${i.cell.slice(0, 12)}`).join(", ")})`,
    JSON.stringify(landing.instances));
  check(landing.buses.length >= 1 && landing.buses.every((b) => b.state === "routed"),
    `...with its buses already ROUTED, so the model is visible before you click ` +
    `(${JSON.stringify(landing.buses)})`);
  check(landing.coach.open === true && landing.coach.steps === 4 && landing.coach.step === 0,
    `a 4-step coach overlay opens on the first visit, at step 1 ` +
    `("${landing.coach.title}")`, JSON.stringify(landing.coach));
  check(!landing.empty && landing.history.canUndo === false,
    `the empty state is hidden and the demo is NOT an undoable edit`,
    JSON.stringify({ empty: landing.empty, history: landing.history }));
  await snap(p0, "onboarding-coach");

  // The coach walks, and dismissal sticks.
  const coach = await p0.evaluate(async () => {
    const e = window.__eda;
    const seen = [];
    for (let i = 0; i < 4; i++) { seen.push(e.coach().title); e.coachNext(1); }
    const afterLast = e.coach();
    return { seen, afterLast, dismissed: afterLast.dismissed };
  });
  check(coach.seen.length === 4 && new Set(coach.seen).size === 4 &&
        coach.afterLast.open === false && coach.dismissed === true,
    `Next walks all 4 steps and the last one dismisses it for good ` +
    `(${coach.seen.map((t) => t.split("·")[0].trim()).join(" → ")})`,
    JSON.stringify(coach));

  // Reloading does not nag: a dismissed coach stays dismissed.
  await p0.reload({ waitUntil: "load" });
  await p0.waitForFunction(() => window.__edaReady === true, null, { timeout: 120_000 });
  await p0.waitForTimeout(1000);
  const again = await p0.evaluate(() => window.__eda.coach());
  check(again.open === false && again.dismissed === true,
    `a second visit does not re-open it (dismissal is remembered)`, JSON.stringify(again));
  await snap(p0, "chain-routed");

  // The keyboard legend is discoverable, and Esc closes it.
  const help = await p0.evaluate(async () => {
    const e = window.__eda;
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "?" }));
    const open = e.shortcuts();
    const rows = document.querySelectorAll("#shortcuts table tr").length;
    e.key("Escape");
    return { open, rows, closed: !e.shortcuts() };
  });
  check(help.open === true && help.closed === true && help.rows >= 12,
    `? opens the keyboard + colour legend (${help.rows} rows) and Esc closes it`,
    JSON.stringify(help));

  // Grouped outliner: instances by TYPE with counts, buses with state + timing.
  const outliner = await p0.evaluate(() => {
    const s = window.__edaStudio();
    const groups = [...document.querySelectorAll("#instance-list .group")].map((g) => ({
      name: g.querySelector(".gname")?.textContent?.trim(),
      count: g.querySelector(".gcount")?.textContent?.trim(),
    }));
    const chips = [...document.querySelectorAll("#instance-list .port-chip")].map((c) =>
      c.textContent.replace(/\s+/g, " ").trim());
    const bus = [...s.buses.keys()][0];
    return {
      groups,
      counts: {
        cells: document.querySelector("#cell-count")?.textContent,
        instances: document.querySelector("#instance-count")?.textContent,
        buses: document.querySelector("#bus-count")?.textContent,
      },
      chips: chips.slice(0, 4),
      typed: chips.filter((c) => /:\s*(uint\d+|bool|int\d+)/.test(c)).length,
      modeBadges: document.querySelectorAll("#instance-list .mode-toggle").length,
      busRowText: document.querySelector("#bus-list .item")?.textContent?.replace(/\s+/g, " ").trim().slice(0, 160),
      skew: window.__eda.busSkew(bus),
      focusables: document.querySelectorAll("#right [data-focus], #right [data-focus-btn]").length,
    };
  });
  check(outliner.groups.length >= 2 && outliner.groups.every((g) => /× \d+/.test(g.count ?? "")),
    `the outliner groups instances by cell TYPE with counts ` +
    `(${outliner.groups.map((g) => `${g.name?.slice(0, 14)} ${g.count}`).join(", ")})`,
    JSON.stringify(outliner.groups));
  check(/\d/.test(outliner.counts.instances ?? "") && /\d/.test(outliner.counts.buses ?? "") &&
        outliner.typed >= 3 && outliner.modeBadges >= 3,
    `...section headers carry counts ("${outliner.counts.instances}", ` +
    `"${outliner.counts.buses}"), chips read "name : type" (${outliner.typed} typed) ` +
    `and each carries a mode badge (${outliner.modeBadges})`,
    JSON.stringify(outliner));
  check(outliner.focusables >= 2 && /routed/.test(outliner.busRowText ?? ""),
    `...bus rows show endpoints + state${outliner.skew ? ` + timing (${outliner.skew.max_rt}t, skew ${outliner.skew.skew_rt}t)` : ""} ` +
    `and ${outliner.focusables} rows are click-to-focus`,
    outliner.busRowText);
  await p0.evaluate(() => {
    const s = window.__edaStudio();
    window.__eda.select([...s.instances.keys()][1] ?? [...s.instances.keys()][0]);
  });
  await snap(p0, "outliner-grouped");

  // ---- label declutter thresholds --------------------------------------
  //
  // Zoomed out, 40 labels are a grey mat over the geometry. The rule is a
  // measured one — screen pixels per block at the label's own depth — so it can
  // be checked rather than eyeballed. The cone MARKERS must survive: you lose
  // the name, never the affordance.
  const labels = await p0.evaluate(async () => {
    const e = window.__eda;
    const frame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    e.key("Escape");                       // nothing selected: no exempt labels
    await frame();
    e.zoom(120); await frame();
    const framed = e.labels();
    e.zoom(900); await frame();
    const wide = e.labels();
    const markers = e.profile().markerObjects;
    // The exemption: what you have SELECTED keeps its label at any distance.
    e.select([...window.__edaStudio().instances.keys()][0]);
    await frame();
    const wideSelected = e.labels();
    e.key("Escape");
    e.zoom(120); await frame();
    return { framed, wide, wideSelected, markers, thresholds: framed.thresholds };
  });
  check(labels.framed.shown > 0 && labels.wide.shown === 0 && labels.wide.hiddenSmall > 0,
    `labels declutter by projected size: ${labels.framed.shown}/${labels.framed.total} shown with ` +
    `the design framed (${labels.framed.pxPerBlock.toFixed(1)} px per block), ` +
    `${labels.wide.shown} zoomed out — all ${labels.wide.hiddenSmall} below the ` +
    `${labels.thresholds.minPxPerBlock} px-per-block legibility threshold`,
    JSON.stringify({ framed: labels.framed, wide: labels.wide }));
  check(labels.markers > 0,
    `...and the 3-D port markers stay visible and pickable when their labels go ` +
    `(${labels.markers} marker objects)`);
  check(labels.wideSelected.shown === 1,
    `...with one exemption: the SELECTED instance keeps its label at any zoom ` +
    `(${labels.wideSelected.shown} of ${labels.wideSelected.total} survive at 900 blocks out)`,
    JSON.stringify(labels.wideSelected));
  check(labels.framed.shown > 0 &&
        labels.framed.hiddenOverlap + labels.framed.shown + labels.framed.hiddenSmall +
          labels.framed.hiddenBehind === labels.framed.total,
    `...and colliding labels lose to the one nearest the camera, with every label ` +
    `accounted for (${labels.framed.shown} shown, ${labels.framed.hiddenOverlap} overlapping, ` +
    `${labels.framed.hiddenSmall} too small, ${labels.framed.hiddenBehind} behind the camera)`,
    JSON.stringify(labels.framed));

  // ---- undo / redo -------------------------------------------------------
  const undo = await p0.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const settle = (ms = 500) => new Promise((r) => setTimeout(r, ms));
    const cell = [...s.cells.keys()][0];

    // (1) place -> undo -> redo
    const u = e.place(cell, [400, -1, 400]);
    await settle();
    const afterPlace = s.instances.size;
    const label = e.history().undo;
    e.undo(); await settle();
    const afterUndo = s.instances.size;
    e.redo(); await settle();
    const afterRedo = s.instances.size;
    e.undo(); await settle();   // leave the document as we found it

    // (2) a DRAG is one undo step, not sixty
    const inst = [...s.instances.values()][0];
    const home = [...inst.at];
    const before = e.history();
    for (let i = 0; i < 20; i++) {
      window.__edaDragMove("instance", inst.name, [home[0] + i, home[1], home[2]]);
      await new Promise((r) => requestAnimationFrame(r));
    }
    window.__edaDrag("instance", inst.name, [home[0] + 19, home[1], home[2]]);
    await settle(800);
    const moved = [...s.instances.get(inst.name).at];
    e.undo(); await settle(800);
    const restored = [...s.instances.get(inst.name).at];

    // (3) deleting an instance that carries buses, then undoing it, brings the
    //     BUS back too — the only undo that has to rebuild more than a transform.
    const withBus = [...s.instances.values()].find((i) => e.busesOn(i.name).length > 0);
    let busUndo = { skipped: true };
    if (withBus) {
      const carried = e.busesOn(withBus.name);
      const before2 = { instances: s.instances.size, buses: s.buses.size };
      s.removeInstance(withBus.name);
      await settle(700);
      const mid = { instances: s.instances.size, buses: s.buses.size };
      e.undo();
      await settle(1200);
      busUndo = {
        skipped: false, inst: withBus.name, carried,
        before: before2, mid,
        after: { instances: s.instances.size, buses: s.buses.size },
        state: s.buses.size ? s.busState([...s.buses.keys()][0]) : "none",
      };
    }
    return {
      place: { name: u.name, afterPlace, afterUndo, afterRedo, label },
      drag: { home, moved, restored, undoStepsAdded: 1, before },
      busUndo,
    };
  });
  check(undo.place.afterUndo === undo.place.afterPlace - 1 &&
        undo.place.afterRedo === undo.place.afterPlace,
    `undo/redo a placement: ${undo.place.afterPlace} → ${undo.place.afterUndo} → ` +
    `${undo.place.afterRedo} instances ("${undo.place.label}")`, JSON.stringify(undo.place));
  check(undo.drag.moved.join() !== undo.drag.home.join() &&
        undo.drag.restored.join() === undo.drag.home.join(),
    `a 20-frame drag is ONE undo step: ${undo.drag.moved.join(",")} → undo → ` +
    `${undo.drag.restored.join(",")} (back where the gesture started)`,
    JSON.stringify(undo.drag));
  if (undo.busUndo.skipped) {
    results.push({ ok: true, label: "undo of a bus-carrying delete SKIPPED (no such instance)", skipped: true });
  } else {
    check(undo.busUndo.mid.buses < undo.busUndo.before.buses &&
          undo.busUndo.after.instances === undo.busUndo.before.instances &&
          undo.busUndo.after.buses === undo.busUndo.before.buses &&
          undo.busUndo.state === "routed",
      `undoing the delete of ${undo.busUndo.inst} (carrying ${undo.busUndo.carried.join(", ")}) ` +
      `restores the instance AND re-routes its bus ` +
      `(${JSON.stringify(undo.busUndo.before)} → ${JSON.stringify(undo.busUndo.mid)} → ` +
      `${JSON.stringify(undo.busUndo.after)}, ${undo.busUndo.state})`,
      JSON.stringify(undo.busUndo));
  }

  // ---- confirm on destructive, with the COUNT in the prompt --------------
  const confirmed = await p0.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const inst = [...s.instances.values()].find((i) => e.busesOn(i.name).length > 0);
    if (!inst) return { skipped: true };
    const carried = e.busesOn(inst.name);
    e.select(inst.name);
    e.key("Delete");
    await new Promise((r) => setTimeout(r, 200));
    const prompt = e.pendingConfirm();
    e.confirmRespond(false);                       // Cancel
    await new Promise((r) => setTimeout(r, 400));
    return {
      inst: inst.name, carried, prompt,
      stillThere: s.instances.has(inst.name),
      cleared: e.pendingConfirm() === null,
      toasts: e.toasts(),
    };
  });
  if (confirmed.skipped) {
    results.push({ ok: true, label: "destructive confirm SKIPPED (no bus-carrying instance)", skipped: true });
  } else {
    check(confirmed.prompt != null &&
          confirmed.prompt.title.includes(String(confirmed.carried.length)) &&
          confirmed.carried.every((b) => confirmed.prompt.body.includes(b)) &&
          confirmed.stillThere && confirmed.cleared,
      `deleting ${confirmed.inst} asks first and puts the COUNT in the prompt ` +
      `("${confirmed.prompt?.title}"), and Cancel keeps it`,
      JSON.stringify(confirmed));
  }

  // ---- toasts stack, and each one dismisses -----------------------------
  const toasts = await p0.evaluate(async () => {
    const e = window.__eda;
    document.querySelector("#toast").replaceChildren();
    e.focusOn([0, 0, 0]);                       // harmless, just to have UI up
    e.key("Escape");
    e.select([...window.__edaStudio().instances.keys()][0]);
    e.key("r"); await new Promise((r) => setTimeout(r, 500));
    e.key("f");
    e.key("r"); await new Promise((r) => setTimeout(r, 500));
    const n = document.querySelectorAll("#toast .toast-item").length;
    const overlapsHint = (() => {
      const t = document.querySelector("#toast").getBoundingClientRect();
      const h = document.querySelector("#hint").getBoundingClientRect();
      return t.bottom > h.top;
    })();
    document.querySelector("#toast .toast-item .x")?.click();
    const after = document.querySelectorAll("#toast .toast-item").length;
    return { n, after, overlapsHint };
  });
  check(toasts.n >= 2 && toasts.after === toasts.n - 1 && toasts.overlapsHint === false,
    `toasts STACK (${toasts.n} at once), each × dismisses one (${toasts.after} left), ` +
    `and the column never overlaps the hint bar`, JSON.stringify(toasts));

  check(p0errors.length === 0, `landing page: no console errors (${p0errors.length})`,
    p0errors.slice(0, 3).join(" | "));
  await p0.close();

  // ======================================================================
  // PART 1 — the interaction model, on the two-adder demo.
  // ======================================================================
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

  // ---- 5. the connect gesture arms and says what happens next -------------
  const armed = await page.evaluate(async () => {
    const e = window.__eda;
    e.clickPort("u0.sum");
    const midHint = e.hint();
    e.key("Escape");
    return { midHint };
  });
  check(/click a blue/i.test(armed.midHint),
    `starting a bus updates the hint to the next action`, armed.midHint);

  // ---- 6. delete by keyboard rips its bus -------------------------------
  const del = await page.evaluate(async () => {
    const e = window.__eda;
    e.key("Escape");
    e.select("u0");
    const carried = e.busesOn("u0");
    e.key("Delete");
    await new Promise((r) => setTimeout(r, 250));
    // u0 carries a bus, so this asks first — with the count.
    const prompt = e.pendingConfirm();
    e.confirmRespond(true);
    await new Promise((r) => setTimeout(r, 900));
    const s = window.__edaStudio();
    return {
      carried, prompt,
      instances: [...s.instances.keys()],
      buses: [...s.buses.keys()],
      toast: document.querySelector("#toast")?.textContent ?? "",
    };
  });
  check(del.prompt != null && del.prompt.title.includes(String(del.carried.length)) &&
        del.carried.every((b) => del.prompt.body.includes(b)),
    `Delete on a bus-carrying instance confirms first, naming the ${del.carried.length} bus(es) ` +
    `("${del.prompt?.title}")`, JSON.stringify(del.prompt));
  check(!del.instances.includes("u0"), `...and then removes u0 (${JSON.stringify(del.instances)})`);
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
    const e = window.__eda;
    const s = window.__edaStudio();
    const bus = [...s.buses.keys()][0];
    // A REAL, REPORTED failure — and the placement that produces one is a
    // moving target, so it is SEARCHED FOR rather than hard-coded. The router
    // keeps getting better at awkward geometry (lifting the driver off the bus
    // level used to be unroutable and now is not), and these checks are about
    // the REASON PIPELINE, not about any one placement staying impossible.
    const sink = e.endpoints().find((x) => x.name === (s.buses.get(bus)?.sinks?.[0] ?? ""));
    const cands = [
      ["lifted off the bus level", [0, 3, 24]],
      ["dropped on top of its own sink", sink ? [...sink.anchor] : null],
      ["lifted far above it", [0, 45, 24]],
    ].filter(([, at]) => at);
    const tried = [];
    for (const [why, at] of cands) {
      window.__edaDrag("instance", "u1", at);
      await new Promise((r) => setTimeout(r, 1500));
      const detail = s.busStateDetail(bus);
      tried.push({ why, at, state: detail.state });
      if (detail.state.startsWith("failed")) return { bus, why, at, state: detail.state, detail, tried };
    }
    return { bus, why: null, state: s.busState(bus), detail: s.busStateDetail(bus), tried };
  });
  check(failed.state.startsWith("failed"),
    `a driver ${failed.why ?? "(no placement found!)"} leaves the bus FAILED with a reason ` +
    `(${failed.tried.length} placement(s) tried)`,
    JSON.stringify(failed.tried));

  // The reason is 300+ characters of engine prose. What the UI must show is a
  // SENTENCE: what failed, where, and what to move — with the raw text still
  // one click away, and the coordinate wired to the camera.
  const said = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const bus = [...s.buses.keys()][0];
    const raw = s.busStateDetail(bus).reason ?? "";
    const h = e.humanReason(raw);
    // The Buses panel: the human line, the fix, the raw detail, and the row
    // that flies the camera to the blockage.
    const row = document.querySelector("#bus-list .item");
    return {
      bus, rawLen: raw.length, h,
      panelHeadline: row?.querySelector(".reason")?.textContent?.trim(),
      panelFix: row?.querySelector(".fix")?.textContent?.trim(),
      hasDetails: !!row?.querySelector("details .raw")?.textContent?.length,
      line: e.busFailureLine(bus, raw),
      toasts: e.toasts(),
    };
  });
  check(said.h.kind !== "raw" && said.h.headline.length < said.rawLen / 2 &&
        said.h.fix.length > 0 && Array.isArray(said.h.at),
    `the ${said.rawLen}-char engine reason becomes a sentence (${said.h.kind}): ` +
    `"${said.h.headline}" → "${said.h.fix}" @ ${JSON.stringify(said.h.at)}`,
    JSON.stringify(said.h));
  check(said.panelHeadline === said.h.headline && said.panelFix?.includes(said.h.fix.slice(0, 24)) &&
        said.hasDetails,
    `...the Buses panel shows headline + fix, and keeps the engine's own words ` +
    `behind a disclosure`, JSON.stringify({ headline: said.panelHeadline, fix: said.panelFix }));
  check(said.toasts.some((t) => /failed/i.test(t) && /Bus /.test(t)),
    `...a toast says "Bus <name> failed: <what> — <fix>"`, JSON.stringify(said.toasts));
  await snap(page, "bus-failed-reason");

  // ...and the row flies the camera to the coordinate the router named.
  const flew = await page.evaluate(async () => {
    const e = window.__eda;
    const before = e.focus();
    document.querySelector("#bus-list .item [data-focus-btn]")?.click();
    await new Promise((r) => setTimeout(r, 200));
    const after = e.focus();
    e.frameAll();                       // put the camera back for later shots
    return { before, after };
  });
  check(!!flew.after && !!said.h.at && flew.after.target.join() === said.h.at.join(),
    `...and the FAILED row is click-to-focus: the camera flew to ` +
    `${JSON.stringify(flew.after?.target)}, the coordinate the router named`,
    JSON.stringify(flew));

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

  // ---- 10b. camera lock, port-mode toggle, mesh accounting --------------
  //
  // Three live user reports, checked through the same paths the mouse drives.

  // (a) "when I want to connect two things dragging also affects the camera".
  const cam = await page.evaluate(() => {
    const e = window.__eda;
    const before = e.cameraFree();
    const out = e.endpoints().find((p) => p.routable && p.kind === "input");
    e.clickPort(out.name);
    const connecting = { free: e.cameraFree(), lock: e.cameraLock(), mode: e.mode().kind };
    e.key("Escape");
    return { before, connecting, after: e.cameraFree() };
  });
  check(cam.before === true && cam.connecting.free === false && cam.after === true,
    `camera locks while connecting ("${cam.connecting.lock}") and frees on Esc`,
    JSON.stringify(cam));

  // (a2) Esc cancels EVERY mode back to idle, and a click on empty ground
  //      deselects. Two escape hatches, neither of which may leave state behind.
  const cancels = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const out = {};
    const inst = [...s.instances.values()][0];
    // A copy: `inst.at` is the live document state and every move mutates it.
    const home = [...inst.at];

    e.arm([...s.cells.keys()][0]);
    out.placing = e.mode().kind;
    e.key("Escape");
    out.afterPlacing = { mode: e.mode().kind, camera: e.cameraFree() };

    e.select(inst.name);
    e.key("g");
    out.grabbing = e.mode().kind;
    window.__edaDragMove("instance", inst.name, [home[0] + 9, home[1], home[2] + 9]);
    await new Promise((r) => setTimeout(r, 400));
    e.key("Escape");
    await new Promise((r) => setTimeout(r, 700));
    out.afterGrab = {
      mode: e.mode().kind, camera: e.cameraFree(),
      at: [...s.instances.get(inst.name).at], home,
    };

    e.select(inst.name);
    e.groundClick([300, 0, 300]);
    out.afterGroundClick = e.selection();
    return out;
  });
  check(cancels.placing === "placing" && cancels.afterPlacing.mode === "idle" &&
        cancels.grabbing === "grabbing" && cancels.afterGrab.mode === "idle" &&
        cancels.afterGrab.camera === true &&
        cancels.afterGrab.at.join() === cancels.afterGrab.home.join(),
    `Esc cancels cleanly from placing AND grabbing — the grab is put back where ` +
    `it started (${cancels.afterGrab.at.join(",")}) and the camera is free again`,
    JSON.stringify(cancels));
  check(cancels.afterGroundClick === null,
    `clicking empty ground deselects`, JSON.stringify(cancels.afterGroundClick));

  // (b) a greyed executor-only port offers a REVERSIBLE mode toggle.
  const promo = await page.evaluate(() => {
    const e = window.__eda;
    const blocked = e.endpoints().find((p) => p.instance && !p.routable && p.promotable);
    if (!blocked) return { skipped: true };
    const before = e.portMode(blocked.instance, blocked.port);
    const rep = e.setPortMode(blocked.instance, blocked.port, "bus");
    const now = e.endpoints().find((p) => p.name === blocked.name);
    const chip = document.querySelector(`.mode-toggle[data-mode="${blocked.instance}|${blocked.port}"]`);
    const back = e.setPortMode(blocked.instance, blocked.port, "executor");
    const restored = e.endpoints().find((p) => p.name === blocked.name);
    return {
      port: blocked.name, before,
      promotedRoutable: !!now?.routable, promotedStep: now?.step,
      note: rep.note, hasToggle: !!chip, toggleLabel: chip?.textContent?.trim(),
      restoredRoutable: !!restored?.routable, restoredNote: back.note,
    };
  });
  if (promo.skipped) {
    results.push({ ok: true, label: "port promotion SKIPPED (no promotable blocked port)", skipped: true });
  } else {
    check(promo.promotedRoutable === true && promo.restoredRoutable === false && promo.hasToggle,
      `${promo.port}: Exec->Bus makes it routable (step ${JSON.stringify(promo.promotedStep)}), ` +
      `Bus->Exec restores it, and the outliner shows the toggle`,
      JSON.stringify(promo));
  }

  // (b2) AUTO-PROMOTION: connecting to a lever input just connects.
  //
  // The live report was "it still makes me click Promote first". An
  // executor-only port is not a dead end, it is a port nobody has converted
  // yet — so the connect gesture converts it, names what it changed, and stays
  // reversible. This is also the cell-to-cell chain the README said needed a
  // Bus-mode click first: sum of one adder into `a` of the next.
  //
  // Runs AFTER the export checks on purpose: promoting a port currently breaks
  // `Design::to_litematic` for the rest of the document's life (engine bug,
  // reproduced standalone — nothing this app does).
  const auto = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const cell = [...s.instances.values()][0].cell;
    e.key("Escape");
    const u = e.place(cell, [0, -1, 44]);        // a second adder to chain into
    await new Promise((r) => setTimeout(r, 400));
    const busesBefore = s.buses.size;
    const meshBefore = e.meshBuilds();
    const target = `${u.name}.a`;
    const modeBefore = e.portMode(u.name, "a");
    const blockedBefore = !e.endpoints().find((p) => p.name === target)?.routable;
    e.clickPort("u1.sum");                        // green driver
    e.clickPort(target);                          // a LEVER bank: must auto-promote
    await new Promise((r) => setTimeout(r, 1500));
    return {
      inst: u.name, target, modeBefore, blockedBefore, busesBefore,
      toast: document.querySelector("#toast")?.textContent ?? "",
      lastConnect: e.lastConnect(),
      modeAfter: e.portMode(u.name, "a"),
      routableAfter: !!e.endpoints().find((p) => p.name === target)?.routable,
      buses: s.buses.size,
      cellRemeshes: e.meshBuilds().cells - meshBefore.cells,
    };
  });
  check(auto.modeBefore === "executor" && auto.blockedBefore === true &&
        auto.modeAfter === "bus" && auto.routableAfter === true,
    `clicking executor-only ${auto.target} as a bus target AUTO-PROMOTES it ` +
    `(executor -> bus, now routable) with no separate Promote click`, JSON.stringify(auto));
  check(auto.buses === auto.busesBefore + 1 && auto.lastConnect?.state === "routed",
    `and the cell-to-cell bus lands: ${auto.lastConnect?.state} ` +
    `(${auto.busesBefore} -> ${auto.buses} buses)`, JSON.stringify(auto.lastConnect));
  check((auto.lastConnect?.promoted ?? []).some((p) => p.includes(auto.target) && p.includes("→")) &&
        /promoted/i.test(auto.toast),
    `the gesture REPORTS what it promoted: ` +
    `"${(auto.lastConnect?.promoted ?? []).join(" | ")}"`,
    `toast: ${auto.toast}`);
  check(auto.cellRemeshes === 1,
    `auto-promotion re-meshes exactly ONE cell variant, not the scene ` +
    `(${auto.cellRemeshes})`);
  await snap(page, "auto-promoted-cell-to-cell");

  // ...and back: Executor mode restores the shipped hardware.
  const restore = await page.evaluate(async (inst) => {
    const e = window.__eda;
    e.setPortMode(inst, "a", "executor");
    await new Promise((r) => setTimeout(r, 600));
    const out = {
      mode: e.portMode(inst, "a"),
      routable: !!e.endpoints().find((p) => p.name === `${inst}.a`)?.routable,
    };
    window.__edaStudio().removeInstance(inst);   // tidy up after ourselves
    return out;
  }, auto.inst);
  check(restore.mode === "executor" && restore.routable === false,
    `toggling ${auto.target} back to Executor restores the original hardware ` +
    `(${restore.mode}, routable=${restore.routable})`, JSON.stringify(restore));

  // A port that genuinely CANNOT be promoted still refuses, with the reason.
  const hard = await page.evaluate(async () => {
    const e = window.__eda;
    const p = e.endpoints().find((q) => q.instance && !q.routable && !q.promotable);
    if (!p) return { skipped: true };
    e.key("Escape");
    e.clickPort("u1.sum");
    e.clickPort(p.name);
    await new Promise((r) => setTimeout(r, 400));
    return { port: p.name, blocked: p.blocked, toast: document.querySelector("#toast")?.textContent ?? "" };
  });
  if (hard.skipped) {
    results.push({ ok: true, label: "un-promotable refusal SKIPPED (every port is promotable)", skipped: true });
    console.log("SKIP un-promotable refusal (every blocked port is promotable)");
  } else {
    check(/cannot be promoted|executor-only/i.test(hard.toast),
      `${hard.port} cannot be promoted and says why`, `${hard.toast} | ${hard.blocked}`);
  }
  await page.evaluate(() => window.__eda.key("Escape"));

  // (c) "i feel it remeshes as I drag things around" — it must not.
  //
  // GPU INSTANCING, asserted rather than asserted-to. A cell is meshed ONCE
  // and every placement of it is a row in that mesh's instance set, so:
  //   * K placements of one cell  -> 1 mesh build, 1 group per colour
  //   * dragging N frames         -> 0 mesh builds, 0 block dumps out of wasm
  //   * a port-mode toggle        -> exactly 1 (that cell's variant)
  //   * one bus re-route          -> exactly 1 (that bus)
  const N = 20;
  const meshes = await page.evaluate(async (N) => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const inst = [...s.instances.values()][0];
    const at = [...inst.at];
    document.querySelector("#live-reroute").checked = false; // preview-only path
    const start = { mesh: e.meshBuilds(), reads: e.sceneReads() };
    e.profileReset();
    // The SYNCHRONOUS cost of a drag frame — the number that decides whether
    // the gesture can keep up with the pointer. (Wall-clock fps in headless
    // chromium is bounded by software GL, not by this.)
    let sync = 0;
    for (let i = 0; i < N; i++) {
      const t = performance.now();
      window.__edaDragMove("instance", inst.name, [at[0] + (i % 7), at[1], at[2] + (i % 5)]);
      sync += performance.now() - t;
      await new Promise((r) => requestAnimationFrame(r));
    }
    const after = { mesh: e.meshBuilds(), reads: e.sceneReads() };
    document.querySelector("#live-reroute").checked = true;
    window.__edaDrag("instance", inst.name, at);
    return {
      start, after, syncMs: sync / N, prof: e.profile(),
      budgetFps: 1000 / Math.max(sync / N, 0.0001),
    };
  }, N);
  check(meshes.after.mesh.cells === meshes.start.mesh.cells &&
        meshes.after.mesh.texture === meshes.start.mesh.texture,
    `dragging an instance ${N} frames triggers 0 cell re-meshes and 0 texture re-meshes ` +
    `(cells ${meshes.start.mesh.cells} -> ${meshes.after.mesh.cells})`,
    JSON.stringify(meshes.after.mesh));
  check(meshes.after.reads.cellDump === meshes.start.reads.cellDump &&
        meshes.after.reads.instDump === meshes.start.reads.instDump &&
        meshes.after.reads.flatten === meshes.start.reads.flatten,
    `...and reads NOTHING back out of wasm: it is ` +
    `${meshes.after.mesh.matrixWrites - meshes.start.mesh.matrixWrites} matrix writes`,
    JSON.stringify({ before: meshes.start.reads, after: meshes.after.reads }));
  check(meshes.syncMs <= 33,
    `...costing ${meshes.syncMs.toFixed(2)} ms of main-thread work per frame — ` +
    `a ${meshes.budgetFps > 999 ? ">999" : meshes.budgetFps.toFixed(0)} fps budget, target >= 30 fps ` +
    `(<= 33 ms)`);

  // The same gesture with LIVE RE-ROUTE on: every frame also commits the move
  // to the document and re-routes the affected buses. This is the router's
  // floor, not the renderer's, and it is why the fixed 250 ms throttle is gone
  // — the drag previews at frame rate and the engine runs as often as it can.
  const live = await page.evaluate(async (N) => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const inst = [...s.instances.values()][0];
    const at = [...inst.at];
    document.querySelector("#live-reroute").checked = true;
    e.timingsReset();
    const t0 = performance.now();
    for (let i = 0; i < N; i++) {
      window.__edaDragMove("instance", inst.name, [at[0] + (i % 7), at[1], at[2] + (i % 5)]);
      await new Promise((r) => requestAnimationFrame(r));
    }
    const wall = performance.now() - t0;
    window.__edaDrag("instance", inst.name, at);
    const t = e.timings();
    return {
      wallPerFrame: wall / N,
      commits: t.dragFrame?.n ?? 0,
      commitMs: t.dragFrame ? t.dragFrame.ms / t.dragFrame.n : 0,
      scene: t["studio.scene"] ? t["studio.scene"].ms / t["studio.scene"].n : 0,
    };
  }, N);
  check(live.commits > 0 && live.commits <= N + 1, // +1: the drop that follows
    `live re-route commits at most one engine move per animation frame ` +
    `(${live.commits} commits over ${N} frames, ${live.commitMs.toFixed(0)} ms each, ` +
    `scene update ${live.scene.toFixed(1)} ms) — no fixed throttle`,
    JSON.stringify(live));

  const toggleMesh = await page.evaluate(async () => {
    const e = window.__eda;
    const p = e.endpoints().find((q) => q.instance && !q.routable && q.promotable);
    if (!p) return { skipped: true };
    const before = { mesh: e.meshBuilds(), reads: e.sceneReads() };
    e.setPortMode(p.instance, p.port, "bus");
    await new Promise((r) => requestAnimationFrame(r));
    const mid = { mesh: e.meshBuilds(), reads: e.sceneReads() };
    e.setPortMode(p.instance, p.port, "executor");
    return { port: p.name, before, mid };
  });
  if (toggleMesh.skipped) {
    results.push({ ok: true, label: "port-mode re-mesh SKIPPED (no promotable port)", skipped: true });
  } else {
    check(toggleMesh.mid.mesh.cells - toggleMesh.before.mesh.cells === 1 &&
          toggleMesh.mid.reads.instDump - toggleMesh.before.reads.instDump === 1,
      `a port-mode toggle on ${toggleMesh.port} re-meshes exactly 1 cell variant ` +
      `(+${toggleMesh.mid.mesh.cells - toggleMesh.before.mesh.cells} cell, ` +
      `+${toggleMesh.mid.reads.instDump - toggleMesh.before.reads.instDump} instance region read)`,
      JSON.stringify(toggleMesh));
  }

  const busMesh = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const bus = [...s.buses.keys()][0];
    if (!bus) return { skipped: true };
    const before = { mesh: e.meshBuilds(), reads: e.sceneReads() };
    const t0 = performance.now();
    s.rerouteBus(bus);
    const ms = performance.now() - t0;
    await new Promise((r) => requestAnimationFrame(r));
    return { bus, before, after: { mesh: e.meshBuilds(), reads: e.sceneReads() }, ms };
  });
  if (busMesh.skipped) {
    results.push({ ok: true, label: "bus re-route re-mesh SKIPPED (no bus)", skipped: true });
  } else {
    // Re-reading the bus is the CONTRACT (it may have moved); re-meshing it is
    // an OPTIMISATION DECISION, and the right one is "only if the blocks
    // actually changed". A reroute that lands the same fragment re-meshes
    // nothing — the mesh on screen is already correct, which is exactly what
    // the revision numbers are for.
    const remeshes = busMesh.after.mesh.buses - busMesh.before.mesh.buses;
    check(busMesh.after.reads.busDump - busMesh.before.reads.busDump === 1 &&
          busMesh.after.mesh.cells === busMesh.before.mesh.cells &&
          remeshes <= 1,
      `re-routing ${busMesh.bus} re-reads exactly 1 bus layer and 0 cells, and ` +
      `re-meshes it ${remeshes} time(s) — ${remeshes === 0
        ? "the route came back identical, so the mesh was already right"
        : "the route moved"} (${busMesh.ms.toFixed(0)} ms)`,
      JSON.stringify(busMesh));
  }

  // ---- 10b. the RENDER is consistent with the ENGINE, after everything -----
  //
  // The regression guard for "sometimes when I move a component the bus doesn't
  // update right". The perf pass made the renderer re-read only what the
  // document reports as changed, which is fast and is one incomplete
  // changed-set away from drawing a lie. So: run the whole battery of edits
  // (drag with live re-route, rotate, rip, re-route, promote a port, delete,
  // undo) and after EACH one compare every rendered layer — read back out of
  // the scene graph's own vertex buffers and instance matrices — against a
  // fresh read from the engine that bypasses every cache.
  const consistency = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const frame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const steps = [];
    const after = async (what) => {
      await frame();
      const c = e.consistency();
      steps.push({ what, ok: c.ok, layers: c.layers.length, mismatches: c.mismatches, orphans: c.orphans });
      return c;
    };
    await after("baseline");
    const inst = [...s.instances.keys()][0];
    const home = [...s.instances.get(inst).at];
    // A DRAG that re-routes buses, frame by frame, exactly as the pointer does.
    for (let i = 1; i <= 12; i++) window.__edaDragMove("instance", inst, [home[0] + i, home[1], home[2]]);
    window.__edaDrag("instance", inst, [home[0] + 12, home[1], home[2]]);
    await after("drag 12 frames + drop");
    // The RACE: queue a live frame and drop somewhere else in the same tick, so
    // the deferred commit is older than the drop when it fires.
    const staleBefore = e.staleCommits();
    window.__edaDragMove("instance", inst, [home[0] + 3, home[1], home[2]]);
    window.__edaDrag("instance", inst, [home[0] + 12, home[1], home[2]]);
    await after("stale live frame after a drop");
    const staleAfter = e.staleCommits();
    const racePos = [...s.instances.get(inst).at];
    e.select(inst);
    e.key("r");
    await after("rotate");
    const bus = [...s.buses.keys()][0];
    if (bus) { s.ripBus(bus); await after("rip a bus"); s.rerouteBus(bus); await after("re-route it"); }
    const promo = e.endpoints().find((p) => p.instance && p.promotable && p.mode !== "bus");
    if (promo) { e.setPortMode(promo.instance, promo.port, "bus"); await after("promote a port"); }
    s.removeInstance([...s.instances.keys()].pop());
    await after("delete an instance");
    e.undo();
    await after("undo it");
    return {
      steps, staleDropped: staleAfter - staleBefore,
      racePos, wanted: [home[0] + 12, home[1], home[2]],
    };
  });
  const badSteps = consistency.steps.filter((s) => !s.ok);
  check(badSteps.length === 0,
    `every rendered layer matches the engine's blocks after all ${consistency.steps.length} ` +
    `operations (drag, rotate, rip, re-route, promote, delete, undo) — ` +
    `${consistency.steps[0].layers} layers compared cell-for-cell against a fresh, ` +
    `cache-bypassing read`,
    JSON.stringify(badSteps.slice(0, 2)));
  check(consistency.staleDropped >= 1 &&
        consistency.racePos.join() === consistency.wanted.join(),
    `the drag race is closed: a live re-route frame queued BEFORE a drop is refused ` +
    `when it fires (${consistency.staleDropped} stale commit dropped), and the instance ` +
    `stays where the drop put it (${consistency.racePos.join(",")})`,
    JSON.stringify(consistency));

  // ---- 10c. copy / paste / cut / duplicate --------------------------------
  //
  // The model: an instance is a REFERENCE to a library cell plus a transform,
  // so a paste places another reference — never a copy of the blocks. What has
  // to survive the round trip is the transform, the port modes, and (for an
  // area copy) any bus whose two ends were both inside the group.
  const clip = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const frame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    // Promote a port first, so "port modes carry over" is a real assertion.
    const src = [...s.instances.keys()][0];
    const port = e.endpoints().find((p) => p.instance === src && p.promotable && p.mode !== "bus");
    if (port) e.setPortMode(src, port.port, "bus");
    await frame();
    const before = { instances: s.instances.size, mesh: e.meshBuilds() };
    const keysBefore = [...s.instances.keys()];
    e.select(src);
    e.copy();
    const copied = e.clipboard();
    const report = e.paste([120, s.instances.get(src).at[1], 120]);
    await frame();
    const pasted = report.instances[0];
    const modes = pasted
      ? e.endpoints().filter((p) => p.instance === pasted.name).map((p) => `${p.port}:${p.mode}`)
      : [];
    const srcModes = e.endpoints().filter((p) => p.instance === src).map((p) => `${p.port}:${p.mode}`);
    const consistent = e.consistency().ok;
    const undoLabel = e.history().undo;
    // Captured BEFORE the undo — measuring "after the paste" after undoing it
    // is how a test lies to itself.
    const after = { instances: s.instances.size, mesh: e.meshBuilds() };
    e.undo();
    await frame();
    return {
      copied, port: port?.port ?? null, report, before, after,
      afterUndo: s.instances.size, pasted, modes, srcModes, consistent, undoLabel,
      keysBefore, keysAfter: [...s.instances.keys()],
      sameCell: pasted?.cell === s.cells.get(pasted?.cell)?.name,
      variants: e.scene().variants.size,
      consistentAfterUndo: e.consistency().ok,
    };
  });
  check(clip.report.instances.length === 1 &&
        clip.after.instances === clip.before.instances + 1 &&
        clip.pasted.name !== clip.copied.instances[0].src,
    `⌘C then ⌘V places ONE new instance of the SAME cell, uniquely named ` +
    `(${clip.copied.instances[0].src} → ${clip.pasted.name}) at the paste point ` +
    `(${clip.pasted.at.join(",")})`,
    JSON.stringify({ before: clip.before.instances, after: clip.after.instances,
                     keysBefore: clip.keysBefore, keysAfter: clip.keysAfter,
                     report: clip.report, toast: clip.toast }));
  check(clip.after.mesh.cells === clip.before.mesh.cells,
    `...and it costs ZERO new cell meshes: the paste is a reference to the cell ` +
    `already meshed, placed by matrix (cells ${clip.before.mesh.cells} → ${clip.after.mesh.cells})`,
    JSON.stringify({ before: clip.before.mesh, after: clip.after.mesh }));
  if (clip.port) {
    check(clip.modes.join() === clip.srcModes.join() && clip.modes.some((m) => m.endsWith(":bus")),
      `...with the port modes carried over: ${clip.pasted.name} has the same ` +
      `Exec/Bus state as ${clip.copied.instances[0].src} (${clip.modes.filter((m) => m.endsWith(":bus")).join(", ")})`,
      JSON.stringify({ pasted: clip.modes, src: clip.srcModes }));
  } else {
    results.push({ ok: true, label: "port modes carried over SKIPPED (no promotable port)", skipped: true });
  }
  check(/^paste /.test(clip.undoLabel ?? "") && clip.afterUndo === clip.before.instances,
    `...and the whole paste is ONE undo step ("${clip.undoLabel}"): one ⌘Z removes ` +
    `every pasted instance (${clip.after.instances} → ${clip.afterUndo})`,
    JSON.stringify(clip));
  check(clip.consistent === true && clip.consistentAfterUndo === true,
    `...with the render consistent with the engine both after the paste and after the undo`,
    JSON.stringify({ paste: clip.consistent, undo: clip.consistentAfterUndo }));

  // ---- a page whose buses run CELL TO CELL ------------------------------
  //
  // The two-adder demo only routes instance -> declared port, so "both endpoints
  // inside the copied set" never happens there and the area-paste check was
  // skipping itself. The chain demo (ADD007 -> BINTOBCD001 -> NUMDISPLAY001) is
  // exactly the shape these two features are about, so they run there.
  const pageC = await browser.newPage({ viewport: { width: 1680, height: 980 } });
  pageC.on("console", (m) => { if (m.type() === "error") { errors.push(m.text()); console.log("console:", m.text()); } });
  await pageC.goto(`http://localhost:${PORT}/?chain=1`, { waitUntil: "load" });
  await pageC.waitForFunction(() => window.__edaReady === true, null, { timeout: 120_000 });
  await pageC.waitForFunction(() => window.__edaStudio().buses.size > 0, null, { timeout: 120_000 });
  await pageC.waitForTimeout(1500);

  // AREA copy: two instances plus the bus between them.
  const area = await pageC.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const frame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    // Find a bus whose driver and sink are both instance ports, and copy BOTH.
    const bus = [...s.buses.values()].find((b) =>
      b.driver.includes(".") && b.sinks.length && b.sinks.every((x) => x.includes(".")));
    if (!bus) return { skipped: true };
    const ends = [bus.driver, ...bus.sinks].map((n) => n.split(".")[0]);
    const group = [...new Set(ends)];
    e.select(group[0]);
    for (const g of group.slice(1)) e.extendSelection(g);
    const selected = e.selected();
    e.copy();
    const copied = e.clipboard();
    const before = { instances: s.instances.size, buses: s.buses.size };
    const report = e.paste([260, s.instances.get(group[0]).at[1], 260]);
    await frame();
    const undoLabel = e.history().undo;
    const consistent = e.consistency().ok;
    const newBus = report.buses[0] ? s.buses.get(report.buses[0]) : null;
    const state = report.buses[0] ? s.busStateDetail(report.buses[0]) : null;
    // Relative transforms preserved?
    const rel = report.instances.map((i) => {
      const src = s.instances.get(i.src) ?? copied.instances.find((c) => c.src === i.src);
      return { src: i.src, name: i.name, rot: s.instances.get(i.name).rot, srcRot: src?.rot ?? null };
    });
    // Compare offsets between the SAME pair of sources: the pasted list is in
    // selection order (primary first), which is not the group's order.
    const bySrc = new Map(report.instances.map((i) => [i.src, i]));
    const pair = group.length > 1 ? [group[0], group[1]] : null;
    const spans = pair && bySrc.has(pair[0]) && bySrc.has(pair[1]) ? [
      bySrc.get(pair[1]).at[0] - bySrc.get(pair[0]).at[0],
      bySrc.get(pair[1]).at[2] - bySrc.get(pair[0]).at[2],
    ] : null;
    const srcSpans = pair ? [
      s.instances.get(pair[1]).at[0] - s.instances.get(pair[0]).at[0],
      s.instances.get(pair[1]).at[2] - s.instances.get(pair[0]).at[2],
    ] : null;
    const driverSrc = bus.driver.split(".")[0];
    e.undo();
    await frame();
    return {
      bus: bus.name, group, selected, copied, report, before, undoLabel, consistent, driverSrc,
      newBus: newBus && { driver: newBus.driver, sinks: newBus.sinks },
      state, rel, spans, srcSpans,
      afterUndo: { instances: s.instances.size, buses: s.buses.size },
      consistentAfterUndo: e.consistency().ok,
    };
  });
  if (area.skipped) {
    results.push({ ok: true, label: "area paste SKIPPED (no instance-to-instance bus)", skipped: true });
  } else {
    check(area.selected.length === area.group.length && area.copied.buses.length === 1,
      `⇧-click builds an area selection (${area.selected.join(", ")}) and the copy takes the ` +
      `bus BETWEEN them (${area.bus}: both ends inside) — buses with one end outside are not copied`,
      JSON.stringify({ selected: area.selected, buses: area.copied.buses }));
    check(area.report.instances.length === area.group.length &&
          area.spans?.join() === area.srcSpans?.join() &&
          area.rel.every((r) => r.rot === r.srcRot),
      `...pasting replicates the whole group with its relative transforms intact ` +
      `(offsets ${area.spans?.join(",")} = ${area.srcSpans?.join(",")}, rotations preserved)`,
      JSON.stringify(area.rel));
    const pastedDriver = area.report.instances.find((i) => i.src === area.driverSrc)?.name;
    check(area.report.buses.length === 1 &&
          area.newBus.driver === `${pastedDriver}.${area.bus0Port ?? area.newBus.driver.split(".")[1]}` &&
          (area.state.state === "routed" || area.state.state === "failed"),
      `...and the internal bus is RECREATED for the pasted group — every endpoint remapped ` +
      `to the COPIES (${area.newBus.driver} → ${area.newBus.sinks.join(", ")}, ${area.state.state}` +
      `${area.state.reason ? `: ${area.state.reason.slice(0, 40)}` : ""}) — routed if it can be, ` +
      `left FAILED with the router's reason if not`,
      JSON.stringify({ bus: area.newBus, state: area.state, failed: area.report.failed }));
    check(/^paste /.test(area.undoLabel ?? "") &&
          area.afterUndo.instances === area.before.instances &&
          area.afterUndo.buses === area.before.buses &&
          area.consistent === true && area.consistentAfterUndo === true,
      `...and ONE ⌘Z removes the pasted instances AND the recreated bus ` +
      `(${area.before.instances}+${area.before.buses} → paste → back to ` +
      `${area.afterUndo.instances}+${area.afterUndo.buses}), render still consistent`,
      JSON.stringify(area));
  }

  // Duplicate + cut, and the screenshot of a pasted group.
  const dup = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const frame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const src = [...s.instances.keys()][0];
    e.select(src);
    const before = s.instances.size;
    const report = e.duplicate();
    await frame();
    const overlaps = report.instances.some((i) => {
      const a = s.instances.get(i.name);
      return [...s.instances.values()].some((b) =>
        b.name !== i.name && b.at.join() === a.at.join());
    });
    const selectedAfter = e.selected();
    const cutName = report.instances[0]?.name;
    e.select(cutName);
    const n = e.cut();
    await frame();
    const afterCut = s.instances.size;
    const pastedBack = e.paste([300, 0, 300]);
    await frame();
    return {
      before, report, overlaps, selectedAfter, cut: n, afterCut,
      afterPaste: s.instances.size, pastedBack, consistent: e.consistency().ok,
      nudged: report.nudged,
    };
  });
  check(dup.report.instances.length === 1 && dup.overlaps === false,
    `⌘D duplicates in place with an offset — the copy lands clear of the original ` +
    `(no two instances share an origin${dup.nudged ? `; nudged by ${dup.nudged.filter(Boolean).join(",")} to clear a keepout` : ""})`,
    JSON.stringify(dup.report));
  check(dup.selectedAfter.includes(dup.report.instances[0].name),
    `...and the PASTED group becomes the selection, so a second ⌘D chains off it ` +
    `(selected: ${dup.selectedAfter.join(", ")})`);
  check(dup.cut === 1 && dup.afterCut === dup.before &&
        dup.afterPaste === dup.afterCut + 1 && dup.consistent === true,
    `⌘X cuts to the clipboard (${dup.afterCut} instances) and ⌘V puts it back ` +
    `(${dup.afterPaste}), render consistent`,
    JSON.stringify(dup));
  await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const src = [...s.instances.keys()][0];
    const home = s.instances.get(src).at;
    e.select(src);
    e.copy();
    const r = e.paste([home[0] + 26, home[1], home[2] + 4]);
    await new Promise((x) => requestAnimationFrame(() => requestAnimationFrame(x)));
    // Frame the ORIGINAL and its copy together: "same cell, second placement".
    const at = r?.instances?.[0]?.at ?? home;
    e.focusOn([(home[0] + at[0]) / 2, home[1] + 6, (home[2] + at[2]) / 2], 46);
  });
  await page.waitForTimeout(700);
  await snap(page, "copy-paste-group");

  // ---- 10d. a bus is a first-class selectable object ----------------------
  const busSel = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const frame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const bus = [...s.buses.keys()][0];
    if (!bus) return { skipped: true };
    e.selectBus(bus);
    await frame();
    const sel = e.selection();
    const hint = e.hint();
    const row = document.querySelector(`#bus-list [data-busrow="${bus}"]`)?.className ?? "";
    // Del deletes it, through the same confirm policy an instance delete uses.
    const before = s.buses.size;
    e.key("Delete");
    await new Promise((r) => setTimeout(r, 200));
    const prompt = e.pendingConfirm();
    e.confirmRespond(true);
    await new Promise((r) => setTimeout(r, 400));
    const after = s.buses.size;
    const consistent = e.consistency().ok;
    e.undo();
    await frame();
    return {
      bus, sel, hint, row, before, after, prompt, consistent,
      afterUndo: s.buses.size, selectionCleared: e.selection(),
      consistentAfterUndo: e.consistency().ok,
    };
  });
  if (busSel.skipped) {
    results.push({ ok: true, label: "bus selection SKIPPED (no bus)", skipped: true });
  } else {
    check(busSel.sel?.kind === "bus" && busSel.sel.id === busSel.bus &&
          /Del/.test(busSel.hint) && /is-selected/.test(busSel.row),
      `clicking a bus SELECTS it (${busSel.bus}): the hint bar names Del/R/F, and its ` +
      `outliner row highlights — the canvas and the panel agree on what is selected`,
      JSON.stringify(busSel));
    check(busSel.prompt != null && busSel.after === busSel.before - 1 &&
          busSel.afterUndo === busSel.before && busSel.consistent === true &&
          busSel.consistentAfterUndo === true,
      `...and Del deletes it after the same destructive confirm ("${busSel.prompt?.title}"), ` +
      `${busSel.before} → ${busSel.after} buses, ⌘Z restores it, render consistent throughout`,
      JSON.stringify(busSel));
  }

  // ---- 10e. bus drawing: show the redstone --------------------------------
  const busDraw = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const frame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const bus = [...s.buses.keys()][0];
    const before = e.busStyle();
    e.setBusStyle("outline");
    await frame();
    const outline = { style: e.busStyle(), prof: e.profile(), consistent: e.consistency().ok };
    if (bus) e.setBusStyleFor(bus, "solid");
    await frame();
    const perBus = e.busStyle();
    if (bus) e.setBusStyleFor(bus, null);
    e.setBusStyle("translucent");
    await frame();
    return { before, outline, perBus, after: e.busStyle(), stored: localStorage.getItem("eda.busStyle") };
  });
  check(busDraw.outline.style.global === "outline" && busDraw.outline.consistent === true,
    `bus drawing has three presets and outline mode keeps the fragment's own blocks ` +
    `visible (silhouette only) without changing what is rendered as bus geometry`,
    JSON.stringify(busDraw.outline.style));
  check(Object.values(busDraw.perBus.perBus).includes("solid") &&
        busDraw.perBus.global === "outline",
    `...and one bus can be overridden solid while the rest stay outlined ` +
    `(per-bus: ${JSON.stringify(busDraw.perBus.perBus)})`,
    JSON.stringify(busDraw.perBus));
  check(busDraw.stored === "translucent" && busDraw.after.global === "translucent",
    `...with the choice remembered in localStorage ("${busDraw.stored}"), so it survives a reload`,
    JSON.stringify(busDraw));

  // ---- 10f. the right-click context menu ---------------------------------
  const ctx = await page.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const frame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const inst = [...s.instances.keys()][0];
    const onInstance = e.contextMenu({ kind: "instance", id: inst, at: s.instances.get(inst).at });
    const rotBefore = s.instances.get(inst).rot;
    e.contextFire("Rotate CW");
    await frame();
    const rotAfter = s.instances.get(inst).rot;
    const bus = [...s.buses.keys()][0];
    // A cell the bus actually occupies — the point a right-click on its
    // geometry would report.
    const frag = bus ? (s.engineLayers().buses.get(bus) ?? []) : [];
    const mid = frag[Math.floor(frag.length / 2)];
    const gateAt = mid ? [mid.x, mid.y, mid.z] : [4, 0, 4];
    let onBus = null, gates = null, gateState = null;
    if (bus) {
      onBus = e.contextMenu({ kind: "bus", id: bus, at: gateAt });
      const label = onBus.find((i) => /^Add gate here/.test(i.label))?.label;
      const gatesBefore = s.buses.get(bus).gates.length;
      e.contextFire(label);
      await new Promise((r) => setTimeout(r, 500));
      gates = { before: gatesBefore, after: s.buses.get(bus).gates.length,
                anchor: s.buses.get(bus).gates.at(-1)?.anchor ?? null,
                at: gateAt, label, gateable: e.gateable(bus),
                snapped: e.gateAnchorFor(bus, gateAt).at,
                refusal: e.lastGateRefusal(),
                toast: document.querySelector("#toast")?.textContent ?? "" };
      gateState = s.busStateDetail(bus);
      // Leave the document as we found it: a gate that FAILED the bus (this
      // router refuses a level change, and says so) must not be left behind for
      // the checks that follow.
      if (gateState.state === "failed" && gates.after > gates.before) {
        e.undo();
        await new Promise((r) => setTimeout(r, 400));
      }
    }
    const onGround = e.contextMenu({ kind: "ground", at: [10, 0, 10] });
    const openNow = e.contextOpen();
    e.key("Escape");
    const openAfterEsc = e.contextOpen();
    const onPort = e.contextMenu({ kind: "port", id: e.endpoints()[0]?.name ?? "" });
    e.closeContext();
    const delBefore = s.instances.size;
    e.contextMenu({ kind: "instance", id: inst });
    e.contextFire("Delete");
    await new Promise((r) => setTimeout(r, 250));
    const prompt = e.pendingConfirm();
    e.confirmRespond(true);
    await new Promise((r) => setTimeout(r, 500));
    const consistent = e.consistency().ok;
    const delAfter = s.instances.size;
    e.undo();
    await frame();
    return {
      inst, onInstance, rotBefore, rotAfter, onBus, gates, gateState, onGround, onPort,
      openNow, openAfterEsc, delBefore, delAfter, prompt, consistent,
      restored: s.instances.size,
      keys: [...s.instances.keys()],
      toast: document.querySelector("#toast")?.textContent?.slice(-300),
      logtail: document.querySelector("#log")?.textContent?.slice(0, 300),
    };
  });
  check(ctx.onInstance.some((i) => i.label === "Rotate CW") &&
        ctx.onInstance.some((i) => i.label === "Delete") &&
        ctx.onInstance.some((i) => i.sub) &&
        ctx.rotAfter === (ctx.rotBefore + 90) % 360,
    `right-clicking an instance offers its own verbs (${ctx.onInstance.filter((i) => i.label !== "-").length} entries: ` +
    `rotate, duplicate, copy/cut, a per-port Exec/Bus submenu, rip its buses, delete) and ` +
    `Rotate CW really rotates it (${ctx.rotBefore}° → ${ctx.rotAfter}°)`,
    JSON.stringify(ctx.onInstance.map((i) => i.label)));
  check(ctx.onGround.some((i) => /^Paste/.test(i.label)) &&
        ctx.onGround.some((i) => i.label === "Frame all") &&
        ctx.onGround.some((i) => i.sub?.length) &&
        ctx.onPort.some((i) => /Start a bus/.test(i.label)),
    `...and the menu is CONTEXT-SENSITIVE: empty space offers paste/frame-all/add-a-component ` +
    `and a port offers "start a bus from here"`,
    JSON.stringify({ ground: ctx.onGround.map((i) => i.label), port: ctx.onPort.map((i) => i.label) }));
  if (ctx.gates) {
    // The entry always carries the CLICKED position — that is the whole reason
    // it exists. Whether the engine accepts it is a separate question:
    // `Design::add_gate` resolves a bus's endpoints through the DECLARED port
    // table, so a bus between two placed cells is refused today (an engine gap
    // reported upstream, `move_gate` has no such limit). Either outcome is
    // acceptable here; SILENCE is not.
    const placed = ctx.gates.after === ctx.gates.before + 1;
    const want = ctx.gates.snapped;   // where a click there actually puts a gate
    check(ctx.gates.label === `Add gate here (${want.join(",")})` &&
          want[0] === ctx.gates.at[0] && want[2] === ctx.gates.at[2] &&
          (placed
            ? ctx.gates.anchor?.join() === want.join()
            : !!ctx.gates.refusal && ctx.gates.refusal.at.join() === want.join()
              && /will not take a checkpoint/.test(ctx.gates.toast)),
      `..."Add gate here" carries the clicked (x, z) into the call and snaps only the LEVEL to the ` +
      `bus's own (clicked ${ctx.gates.at.join(",")} → ${want.join(",")}: a bus is a 2y-pitch stack, ` +
      `so the block under the cursor can be any bit, and a gate off the trunk level is a level ` +
      `change this router refuses) — and ${placed
        ? `the checkpoint landed at ${ctx.gates.anchor?.join(",")} (bus is ${ctx.gateState.state})`
        : `the refusal is REPORTED, not swallowed: ${ctx.gates.refusal?.error}`}`,
      JSON.stringify(ctx.gates));
  } else {
    results.push({ ok: true, label: "context add-gate SKIPPED (no bus)", skipped: true });
  }
  check(ctx.openAfterEsc === false && ctx.openNow === true,
    `...Esc closes it (open ${ctx.openNow} → ${ctx.openAfterEsc}), so it is never a trap`);
  check(ctx.prompt != null && ctx.delAfter === ctx.delBefore - 1 &&
        ctx.restored === ctx.delBefore && ctx.consistent === true,
    `...and its Delete goes through the same confirm as every other delete ` +
    `("${ctx.prompt?.title}"), removes it (${ctx.delBefore} → ${ctx.delAfter}), leaves the ` +
    `render consistent, and ⌘Z brings it back (${ctx.restored})`,
    JSON.stringify(ctx));
  await page.evaluate(() => {
    const s = window.__edaStudio();
    window.__eda.contextMenu({
      kind: "instance", id: [...s.instances.keys()][0], screen: [560, 300],
    });
  });
  await snap(page, "context-menu-instance");
  await page.evaluate(() => window.__eda.closeContext());

  // ---- 10g. gates are CHECKPOINTS with a full lifecycle -------------------
  //
  // A gate and an endpoint are deliberately different things: an endpoint is
  // netlist, a gate is route. So adding one makes the run pass through a point,
  // and removing one lets the router take a straighter path — WITHOUT touching
  // what the bus connects. This walks add (via the context menu's clicked
  // position), drag (the 2-adjacent-segment fast path), remove, and undo.
  const gate = await pageC.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const settle = async (ms = 500) => {
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
      await new Promise((r) => setTimeout(r, ms));
    };
    const bus = [...s.buses.keys()].find((b) => s.busStateDetail(b).state === "routed");
    if (!bus) return { skipped: true };
    const cells = () => (s.engineLayers().buses.get(bus) ?? []);
    const has = (at) => cells().some((c) => c.x === at[0] && c.y === at[1] && c.z === at[2]);
    const frag = cells();
    const direct = frag.length;
    // A detour point: on the trunk's own level (a gate at another level is
    // refused by this router, and it says so), pushed sideways off the run.
    // A bus is a stack of bits at a 2y pitch: the cell under a click can be any
    // bit. The waypoint is the (x, z); the LEVEL belongs to the bus, and the app
    // snaps it (a gate off the trunk level is a level change, which this router
    // refuses and says so). Pick a real trunk column and let the app snap y.
    const cols = new Map();
    for (const c of frag) {
      const k = `${c.x},${c.z}`;
      cols.set(k, Math.min(cols.get(k) ?? Infinity, c.y));
    }
    const trunk = [...cols.keys()].map((k) => k.split(",").map(Number));
    const spanX = Math.max(...trunk.map((c) => c[0])) - Math.min(...trunk.map((c) => c[0]));
    const spanZ = Math.max(...trunk.map((c) => c[1])) - Math.min(...trunk.map((c) => c[1]));
    const pick = trunk[Math.floor(trunk.length / 2)];
    const away = spanX >= spanZ ? [0, 0, 6] : [6, 0, 0];
    const snap = (x, z) => e.gateAnchorFor(bus, [x, cols.get(`${x},${z}`) ?? 0, z]).at;
    const detour = (() => {
      const a = snap(pick[0], pick[1]);
      return [a[0] + away[0], a[1], a[2] + away[2]];
    })();
    const onPath = snap(pick[0], pick[1]);

    // ADD, through the context menu entry, with the clicked position.
    const items = e.contextMenu({ kind: "bus", id: bus, at: detour });
    const label = items.find((i) => /^Add gate here/.test(i.label))?.label;
    e.contextFire(label);
    await settle(700);
    let used = detour;
    let addState = s.busStateDetail(bus);
    let addedVia = "detour";
    if (addState.state !== "routed") {
      // The detour did not fit. A gate ON the existing path is still a real
      // checkpoint and keeps the assertion honest; the refusal was reported.
      e.undo();
      await settle(600);
      const items2 = e.contextMenu({ kind: "bus", id: bus, at: onPath });
      e.contextFire(items2.find((i) => /^Add gate here/.test(i.label))?.label);
      await settle(700);
      used = onPath;
      addState = s.busStateDetail(bus);
      addedVia = "on-path";
    }
    const afterAdd = {
      state: addState, gates: e.gates(bus), cells: cells().length,
      through: has(used), consistent: e.consistency().ok, undoLabel: e.history().undo,
    };
    // DRAG the handle, exactly as the pointer does.
    const gname = e.gates(bus).gates[0]?.name;
    const meshBefore = e.meshBuilds();
    const to = [used[0] + (away[2] ? 2 : 0), used[1], used[2] + (away[0] ? 2 : 0)];
    e.dragGate(bus, gname, to);
    await settle(700);
    const afterDrag = {
      anchor: e.gates(bus).gates[0]?.anchor, state: s.busStateDetail(bus),
      cells: cells().length, consistent: e.consistency().ok,
      busRemeshes: e.meshBuilds().buses - meshBefore.buses,
      cellRemeshes: e.meshBuilds().cells - meshBefore.cells,
      buses: s.buses.size,
      undoLabel: e.history().undo,
    };
    // REMOVE it: same endpoints, straighter route.
    e.removeGate(bus, 0);
    await settle(800);
    const afterRemove = {
      state: s.busStateDetail(bus), gates: e.gates(bus), cells: cells().length,
      consistent: e.consistency().ok, undoLabel: e.history().undo,
      driver: s.buses.get(bus).driver, sinks: s.buses.get(bus).sinks,
    };
    e.undo();
    await settle(800);
    const afterUndo = { gates: e.gates(bus), consistent: e.consistency().ok };
    // Leave the design with the gate REMOVED, so later checks see a clean bus.
    e.removeGate(bus, 0);
    await settle(700);
    return {
      bus, direct, used, addedVia, afterAdd, afterDrag, afterRemove, afterUndo,
      endpointsUnchanged: afterRemove.driver === s.buses.get(bus).driver,
      selection: e.selection(),
    };
  });
  if (gate.skipped) {
    results.push({ ok: true, label: "gate lifecycle SKIPPED (no routed bus)", skipped: true });
  } else {
    check(gate.afterAdd.gates.gates.length === 1 &&
          gate.afterAdd.gates.segments === 2 &&
          gate.afterAdd.state.state === "routed" &&
          gate.afterAdd.through === true,
      `right-click → "Add gate here" puts a CHECKPOINT at the clicked cell ` +
      `(${gate.used.join(",")}, ${gate.addedVia}): the bus stays routed, now in ` +
      `${gate.afterAdd.gates.segments} trunk spans, and its geometry passes THROUGH that cell`,
      JSON.stringify(gate.afterAdd));
    check(gate.afterDrag.anchor?.join() !== gate.used.join() &&
          gate.afterDrag.cellRemeshes === 0 &&
          gate.afterDrag.busRemeshes >= 1 &&
          gate.afterDrag.busRemeshes <= gate.afterDrag.buses &&
          gate.afterDrag.consistent === true,
      `...dragging the handle moves the checkpoint (${gate.used.join(",")} → ` +
      `${gate.afterDrag.anchor?.join(",")}) and costs ZERO cell re-meshes and at most one ` +
      `re-mesh per bus whose geometry moved (${gate.afterDrag.busRemeshes} of ` +
      `${gate.afterDrag.buses} bus layers; the router may amend the buses a re-routed span crosses)`,
      JSON.stringify(gate.afterDrag));
    check(gate.afterRemove.gates.gates.length === 0 &&
          gate.afterRemove.gates.segments === 1 &&
          gate.afterRemove.state.state === "routed" &&
          gate.afterRemove.cells <= gate.afterDrag.cells &&
          gate.endpointsUnchanged === true,
      `...and removing it STRAIGHTENS the route — ${gate.afterDrag.cells} cells with the ` +
      `checkpoint, ${gate.afterRemove.cells} without (direct route: ${gate.direct}) — while the ` +
      `endpoints stay exactly as they were (${gate.afterRemove.driver} → ` +
      `${gate.afterRemove.sinks.join(", ")}): a gate is ROUTE, an endpoint is NETLIST`,
      JSON.stringify({ add: gate.afterAdd.cells, drag: gate.afterDrag.cells, remove: gate.afterRemove.cells }));
    check(/^add gate /.test(gate.afterAdd.undoLabel ?? "") &&
          /gate/.test(gate.afterDrag.undoLabel ?? "") &&
          /^remove gate /.test(gate.afterRemove.undoLabel ?? "") &&
          gate.afterUndo.gates.gates.length === 1 &&
          gate.afterUndo.consistent === true,
      `...each step is its own undo entry ("${gate.afterAdd.undoLabel}", ` +
      `"${gate.afterDrag.undoLabel}", "${gate.afterRemove.undoLabel}") and ⌘Z after the ` +
      `removal puts the checkpoint back`,
      JSON.stringify(gate.afterUndo));
  }

  // Two gates on one bus, with the menu open on it: the screenshot the model
  // needs (checkpoints are visibly different objects from ports).
  const twoGates = await pageC.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const settle = async (ms = 600) => {
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
      await new Promise((r) => setTimeout(r, ms));
    };
    const bus = [...s.buses.keys()].find((b) => s.busStateDetail(b).state === "routed");
    if (!bus) return { skipped: true };
    const frag = () => s.engineLayers().buses.get(bus) ?? [];
    const lane = frag();
    const ys = new Map();
    for (const c of lane) ys.set(c.y, (ys.get(c.y) ?? 0) + 1);
    const y = [...ys.entries()].sort((a, b) => b[1] - a[1])[0][0];
    const row = lane.filter((c) => c.y === y).sort((a, b) => (a.x - b.x) || (a.z - b.z));
    for (const q of [0.33, 0.66]) {
      const c = row[Math.floor(row.length * q)];
      if (c) { try { e.addGate(bus, [c.x, c.y, c.z]); } catch { /* refused */ } await settle(500); }
    }
    e.selectBus(bus);
    await settle(400);
    return {
      bus, gates: e.gates(bus), state: s.busStateDetail(bus),
      rows: document.querySelectorAll("#bus-list .gate-chip").length,
      consistent: e.consistency().ok,
    };
  });
  if (!twoGates.skipped) {
    check(twoGates.gates.gates.length >= 1 &&
          twoGates.rows === twoGates.gates.gates.length &&
          twoGates.consistent === true,
      `a selected bus LISTS its checkpoints in the outliner (${twoGates.rows} rows, ordered, ` +
      `click-to-focus, each with its own ✕) alongside "${twoGates.gates.segments} trunk span(s)" — ` +
      `the model is on screen, not implied`,
      JSON.stringify(twoGates));
    await pageC.evaluate(() => {
      const s = window.__edaStudio();
      const bus = [...s.buses.keys()].find((b) => s.buses.get(b).gates.length) ?? [...s.buses.keys()][0];
      const g = s.buses.get(bus).gates[0];
      if (g) window.__eda.focusOn([g.anchor[0], g.anchor[1] + 2, g.anchor[2]], 34);
      window.__eda.contextMenu({ kind: "bus", id: bus, at: g?.anchor ?? [0, 0, 0], screen: [700, 340] });
    });
    await snap(pageC, "bus-gates-and-context-menu");
    await pageC.evaluate(() => window.__eda.closeContext());
  }
  if (twoGates.skipped) {
    results.push({ ok: true, label: "gate outliner listing SKIPPED (no routed bus)", skipped: true });
  }
  await pageC.close();

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
    // The bus-drawing point, made with pictures: the same textured scene with
    // the bus layers solid (they hide the redstone they are made of) and then
    // outlined (the redstone reads, the identity survives).
    // Frame the BUS: by now the clipboard checks have scattered instances
    // hundreds of blocks away, and a whole-design frame makes the point
    // invisible. These two shots are the argument, so they have to be readable.
    await page.evaluate(() => {
      const e = window.__eda;
      const s = window.__edaStudio();
      const bus = [...s.buses.keys()].find((b) => s.busStateDetail(b).state === "routed");
      const frag = bus ? (s.engineLayers().buses.get(bus) ?? []) : [];
      const mid = frag[Math.floor(frag.length / 2)];
      if (mid) e.focusOn([mid.x, mid.y, mid.z], 26);
      e.setBusStyle("solid");
    });
    await page.waitForTimeout(900);
    await snap(page, "textured-bus-solid");
    await page.evaluate(() => window.__eda.setBusStyle("outline"));
    await page.waitForTimeout(900);
    await snap(page, "textured-bus-outline-shows-redstone");
    const busOverTexture = await page.evaluate(() => ({
      style: window.__eda.busStyle(),
      consistent: window.__eda.consistency().ok,
      prof: window.__eda.profile(),
    }));
    check(busOverTexture.style.global === "outline" && busOverTexture.consistent === true,
      `...and over a resource pack the bus layers can be reduced to outlines, so the ` +
      `dust and repeaters a bus is MADE OF are visible through it ` +
      `(${busOverTexture.prof.busMeshes} bus objects, render still consistent)`,
      JSON.stringify(busOverTexture.style));
    await page.evaluate(() => {
      window.__eda.setBusStyle("translucent");
      document.querySelector("#textured").checked = false;
      document.querySelector("#textured").dispatchEvent(new Event("change"));
    });
    await page.waitForTimeout(400);
  } else {
    console.log("SKIP textured view (no pack.zip at repo root; it is not committed)");
    results.push({ ok: true, label: "textured view SKIPPED (no pack.zip)", skipped: true });
  }

  // ---- 12. instancing at scale (last: it adds instances) ------------------
  //
  // K placements of ONE cell, then ~10 placements over THREE distinct cells.
  // The draw-call number is the proof: it tracks distinct CELLS, not
  // placements, because the cell is meshed once and placed by matrix.
  const K = 5;
  const many = await page.evaluate(async (K) => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const cell = [...s.cells.keys()].find((c) => ![...s.instances.values()].some((i) => i.cell === c))
      ?? [...s.cells.keys()][0];
    const before = { mesh: e.meshBuilds(), reads: e.sceneReads(), prof: e.profile() };
    for (let i = 0; i < K; i++) e.place(cell, [200 + 30 * i, -1, 200]);
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    return { cell, before, after: { mesh: e.meshBuilds(), reads: e.sceneReads(), prof: e.profile() } };
  }, K);
  check(many.after.mesh.cells - many.before.mesh.cells === 1 &&
        many.after.prof.cellMeshes - many.before.prof.cellMeshes === 1,
    `${K} placements of ${many.cell} cost exactly 1 cell mesh build and ` +
    `${many.after.mesh.instancedGroups - many.before.mesh.instancedGroups} instanced group ` +
    `(one InstancedMesh, vertex-coloured, ${K} rows)`,
    JSON.stringify({ before: many.before.mesh, after: many.after.mesh }));
  check(many.after.reads.cellDump - many.before.reads.cellDump === 1 &&
        many.after.reads.instDump === many.before.reads.instDump,
    `...read out of wasm ONCE, from the cell itself — no per-instance region dumps ` +
    `(instDump ${many.before.reads.instDump} -> ${many.after.reads.instDump})`,
    JSON.stringify({ before: many.before.reads, after: many.after.reads }));

  // The headline number, measured on a CLEAN page so nothing else is in the
  // scene: 10 placements over 3 distinct cells.
  // `?empty=1`: the landing chain would be in the scene otherwise, and this
  // measurement is about exactly ten placements and nothing else.
  const page2 = await browser.newPage({ viewport: { width: 1680, height: 980 } });
  await page2.goto(`http://localhost:${PORT}/?empty=1`, { waitUntil: "load" });
  await page2.waitForFunction(() => window.__edaReady === true, null, { timeout: 120_000 });
  await page2.waitForFunction(() => window.__edaStudio().cells.size >= 3, null, { timeout: 120_000 });
  const emptyState = await page2.evaluate(() => ({
    open: window.__eda.emptyState(),
    text: document.querySelector("#empty-state .card")?.textContent?.replace(/\s+/g, " ").trim() ?? "",
    instances: window.__edaStudio().instances.size,
  }));
  check(emptyState.open === true && emptyState.instances === 0 &&
        /Click a cell in Library/i.test(emptyState.text) && /click the ground/i.test(emptyState.text),
    `with no design the canvas says what to do FIRST, in order ` +
    `("${emptyState.text.slice(0, 96)}…")`, JSON.stringify(emptyState));
  const draws = await page2.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const cells = [...s.cells.keys()].slice(0, 3);
    cells.forEach((c, i) => {
      // Tight enough that all ten are inside the default camera frustum, so
      // the draw-call count is measured with nothing culled away.
      for (let k = 0; k < (i === 0 ? 4 : 3); k++) e.place(c, [14 * k, -1, 14 * i]);
    });
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    e.profileReset();
    await new Promise((r) => setTimeout(r, 400));
    return { cells, instances: s.instances.size, prof: e.profile(), mesh: e.meshBuilds() };
  });
  check(draws.instances === 10 && draws.prof.cellMeshes === 3 && draws.mesh.cells === 3,
    `10 placements over 3 distinct cells: 3 meshed cells, ` +
    `${draws.prof.cellGroups} instanced groups, ` +
    `**${draws.prof.drawCalls} draw calls** for the whole scene ` +
    `(${draws.prof.triangles} triangles, ${draws.prof.geometries} geometries)`,
    JSON.stringify({ instances: draws.instances, mesh: draws.mesh, prof: draws.prof }));
  check(draws.prof.drawCalls < 10 * 3,
    `...and the draw calls track distinct CELLS (3), not the 10 placements: ` +
    `${draws.prof.drawCalls} calls, ${(draws.prof.drawCalls / draws.instances).toFixed(1)} per instance`,
    JSON.stringify(draws.prof));
  await snap(page2, "instanced-10-placements");
  await page2.close();

  check(errors.length === 0, `no console errors (${errors.length})`,
    errors.slice(0, 3).join(" | "));

  // ======================================================================
  // PART 3 — PORT vs BODY PICKING.
  //
  // The reported bug, in the user's words: "it's awkward to select an IO and
  // route it, I always accidentally move the component". Two causes, both of
  // them checked here with REAL pointer input (page.mouse, not a hook — the
  // whole point is that the browser's own events resolve the way we claim):
  //
  //   1. a port marker sits one block outside its column, so it is often inside
  //      or behind a neighbouring body's invisible pick box. Ray order gave the
  //      press to the box. Ports are now picked in SCREEN SPACE and always win.
  //   2. a press on a body became a drag on the first pointermove, so a jittery
  //      click nudged the component. A press under 4 px is now a selection.
  //
  // On its own page, with the two-adder demo: the checks drive the pointer, and
  // a scene that has been scattered by the clipboard section would make "find a
  // pixel that is a body" meaningless.
  // ======================================================================
  const p3 = await browser.newPage({ viewport: { width: 1680, height: 980 } });
  const p3errors = [];
  p3.on("console", (m) => { if (m.type() === "error") p3errors.push(m.text()); });
  await p3.goto(`http://localhost:${PORT}/?demo=1`, { waitUntil: "load" });
  await p3.waitForFunction(() => window.__edaReady === true, null, { timeout: 120_000 });
  await p3.waitForFunction(() => window.__edaStudio().instances.size >= 2, null, { timeout: 120_000 });
  await p3.waitForTimeout(1500);
  // The coach owns the first Escape. Dismiss it up front so every Esc in this
  // section unwinds the GESTURE, which is what these checks are about.
  await p3.evaluate(() => window.__eda.coachDismiss());
  await p3.waitForTimeout(200);

  // ---- 13. a port is a target at every zoom -------------------------------
  //
  // The zoom sweep is the point. A cone of radius 0.45 blocks is a 4-pixel
  // target once the labels start decluttering, and "click the little arrow" is
  // the gesture the entire routing model rests on.
  const zooms = await p3.evaluate(async () => {
    const e = window.__eda;
    const out = [];
    e.frameAll();
    for (const radius of [40, 70, 130, 320]) {
      e.zoom(radius);
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
      await new Promise((r) => setTimeout(r, 120));
      let onScreen = 0, hit = 0;
      const misses = [];
      for (const { name } of e.endpoints()) {
        const at = e.portScreen(name);
        // ON THE CANVAS, not merely in the viewport: a marker that projects over
        // the header is not something a pointer can press, and counting it would
        // make this check pass on a coordinate the mouse can never reach.
        if (!at || !e.onCanvas(at[0], at[1])) continue;
        onScreen++;
        const got = e.probeAt(at[0], at[1]);
        if (got?.kind === "port" && got.id === name) hit++;
        else misses.push({ name, got });
      }
      out.push({ radius, onScreen, hit, misses: misses.slice(0, 4), labels: e.labels() });
    }
    return { out, thresholds: e.pickThresholds() };
  });
  const allZoom = zooms.out.every((z) => z.onScreen > 0 && z.hit === z.onScreen);
  check(allZoom,
    `a port marker is pickable at EVERY zoom — ${zooms.out.map((z) => `${z.hit}/${z.onScreen}@r${z.radius}`).join(" ")} ` +
    `(${zooms.thresholds.portPickPx} px screen-space radius, not ray-vs-cone)`,
    JSON.stringify(zooms.out.map((z) => ({ radius: z.radius, onScreen: z.onScreen, hit: z.hit, misses: z.misses }))));
  // ...including at the zoom where the labels have decluttered away, which is
  // exactly where a shrinking hit target would have stopped working.
  const decluttered = zooms.out.filter((z) => z.labels.hiddenSmall + z.labels.hiddenOverlap > 0);
  check(decluttered.length > 0 && decluttered.every((z) => z.hit === z.onScreen),
    `...including at the ${decluttered.length} zoom level(s) where labels DECLUTTER ` +
    `(up to ${Math.max(0, ...decluttered.map((z) => z.labels.hiddenSmall + z.labels.hiddenOverlap))} labels hidden, ` +
    `every port still hit)`,
    JSON.stringify(decluttered.map((z) => ({ radius: z.radius, labels: z.labels, hit: z.hit, onScreen: z.onScreen }))));

  // ---- 14. a port beats the body geometry in front of it ------------------
  const overlap = await p3.evaluate(async () => {
    const e = window.__eda;
    e.frameAll();
    e.zoom(70);
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const before = e.pickStats();
    const ports = e.endpoints().map((p) => p.name);
    let probed = 0;
    for (const name of ports) {
      const at = e.portScreen(name);
      if (!at) continue;
      probed++;
      e.probeAt(at[0], at[1]);
    }
    return { probed, before, after: e.pickStats() };
  });
  check(overlap.after.portOverBody - overlap.before.portOverBody > 0,
    `${overlap.after.portOverBody - overlap.before.portOverBody} of ${overlap.probed} ports ` +
    `sit behind or inside a component's pick box and STILL win the press — the ` +
    `priority rule, measured (ray order would have given those to the body)`,
    JSON.stringify({ before: overlap.before, after: overlap.after }));

  /** A canvas pixel that is unambiguously a component body: `id`'s body, and
   *  far enough from every port marker that no reasonable pick radius reaches
   *  it. Scanned rather than computed, so it is a pixel the pointer can use. */
  //
  //  Kept well inside the canvas: at the edges the ground plane is at a grazing
  //  angle, so a press there maps to a wild world coordinate and the drag being
  //  measured stops being the drag a user makes.
  const bodyPixel = async (id) => p3.evaluate((id) => {
    const e = window.__eda;
    const r = e.canvasRect();
    const inx = (r.right - r.left) * 0.15, iny = (r.bottom - r.top) * 0.15;
    const ports = e.endpoints().map((p) => e.portScreen(p.name)).filter(Boolean);
    for (let y = r.top + iny; y < r.bottom - iny; y += 6) {
      for (let x = r.left + inx; x < r.right - inx; x += 6) {
        const got = e.probeAt(x, y);
        if (got?.kind !== "instance" || (id && got.id !== id)) continue;
        if (ports.some(([px, py]) => Math.hypot(px - x, py - y) < 40)) continue;
        return { x, y, id: got.id };
      }
    }
    return null;
  }, id);

  // ---- 15. a press on a PORT can never become a component move ------------
  //
  // A zoom where the design is fully on the canvas, so the port the pointer is
  // sent to is a port the pointer can reach.
  const portName = await p3.evaluate(async () => {
    const e = window.__eda;
    e.frameAll();
    e.zoom(90);
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    await new Promise((r) => setTimeout(r, 150));
    const reachable = (p) => {
      const at = e.portScreen(p.name);
      return at && e.onCanvas(at[0], at[1], 60) && e.probeAt(at[0], at[1])?.id === p.name;
    };
    const eps = e.endpoints();
    return (eps.find((p) => p.routable && p.kind === "input" && p.instance && reachable(p))
      ?? eps.find((p) => p.routable && p.kind === "input" && reachable(p)))?.name ?? null;
  });
  check(portName != null, `a routable driver port is reachable by the pointer (${portName})`);
  const portAt = await p3.evaluate((n) => window.__eda.portScreen(n), portName);
  const beforePress = await p3.evaluate(() => {
    const s = window.__edaStudio();
    return {
      stats: window.__eda.pickStats(),
      placements: [...s.instances.values()].map((i) => ({ name: i.name, at: [...i.at], rot: i.rot })),
      history: window.__eda.history(),
    };
  });
  // The gesture: press ON the port, then MOVE 40 px. Under the old code this
  // was a component drag; it must now be a connection and nothing else.
  await p3.mouse.move(portAt[0], portAt[1]);
  await p3.waitForTimeout(120);
  await p3.mouse.down();
  for (let i = 1; i <= 8; i++) {
    await p3.mouse.move(portAt[0] + i * 5, portAt[1] + i * 3);
    await p3.waitForTimeout(16);
  }
  await p3.mouse.up();
  await p3.waitForTimeout(400);
  const afterPress = await p3.evaluate(() => {
    const s = window.__edaStudio();
    return {
      stats: window.__eda.pickStats(),
      mode: window.__eda.mode(),
      selection: window.__eda.selection(),
      placements: [...s.instances.values()].map((i) => ({ name: i.name, at: [...i.at], rot: i.rot })),
      history: window.__eda.history(),
      hint: window.__eda.hint(),
    };
  });
  const moved = JSON.stringify(beforePress.placements) !== JSON.stringify(afterPress.placements);
  check(afterPress.mode?.kind === "connecting" && afterPress.mode.from === portName,
    `a mousedown on ${portName} + 40 px of pointer movement starts a BUS ` +
    `(mode ${afterPress.mode?.kind} from ${afterPress.mode?.from})`,
    JSON.stringify({ mode: afterPress.mode, hint: afterPress.hint }));
  check(!moved &&
        afterPress.stats.dragsStarted === beforePress.stats.dragsStarted &&
        afterPress.history.canUndo === beforePress.history.canUndo,
    `...and ZERO instance transform change: ${afterPress.placements.length} placements identical, ` +
    `0 drags started, no new undo entry — a port press never touches the drag state`,
    JSON.stringify({ before: beforePress.placements, after: afterPress.placements, stats: afterPress.stats }));
  const cancelled = await p3.evaluate(() => {
    window.__eda.key("Escape");
    return window.__eda.mode();
  });
  check(cancelled?.kind === "idle",
    `...and Esc cancels the bus the port press started (mode ${cancelled?.kind})`,
    JSON.stringify(cancelled));

  // ---- 16. a <4 px press on a BODY selects, and does NOT move it ----------
  const bodyA = await bodyPixel(null);
  check(bodyA != null, `a component body is pickable on the canvas (${JSON.stringify(bodyA)})`);
  const jitter = await (async () => {
    const before = await p3.evaluate((id) => ({
      stats: window.__eda.pickStats(),
      at: window.__eda.instanceAt(id),
      history: window.__eda.history(),
    }), bodyA.id);
    await p3.mouse.move(bodyA.x, bodyA.y);
    await p3.waitForTimeout(80);
    await p3.mouse.down();
    // Three one-pixel twitches: a hand, not a gesture. Total travel 3 px.
    for (const [dx, dy] of [[1, 0], [1, 1], [2, 1]]) {
      await p3.mouse.move(bodyA.x + dx, bodyA.y + dy);
      await p3.waitForTimeout(20);
    }
    await p3.mouse.up();
    await p3.waitForTimeout(300);
    const after = await p3.evaluate((id) => ({
      stats: window.__eda.pickStats(),
      at: window.__eda.instanceAt(id),
      history: window.__eda.history(),
      selection: window.__eda.selection(),
      cursor: document.querySelector("#canvas-wrap canvas").style.cursor,
    }), bodyA.id);
    return { before, after };
  })();
  check(jitter.after.selection?.kind === "instance" && jitter.after.selection.id === bodyA.id &&
        JSON.stringify(jitter.after.at) === JSON.stringify(jitter.before.at) &&
        jitter.after.stats.dragsStarted === jitter.before.stats.dragsStarted &&
        jitter.after.stats.clicksBelowThreshold === jitter.before.stats.clicksBelowThreshold + 1 &&
        jitter.after.history.canUndo === jitter.before.history.canUndo,
    `a 3 px press on ${bodyA.id} SELECTS it and moves it not one block ` +
    `(${JSON.stringify(jitter.before.at?.at)} -> ${JSON.stringify(jitter.after.at?.at)}, ` +
    `0 drags started, no undo entry) — the ${zooms.thresholds.dragPx} px threshold`,
    JSON.stringify(jitter));

  // ---- 17. past the threshold it moves, in ONE undo step, with no re-mesh --
  //
  // Also the perf contract for the new picking: 20 drag frames must cost zero
  // cell re-meshes and leave the draw-call count alone. Measured with the live
  // re-route off, so what is being measured is the DRAG, not the router.
  const dragged = await (async () => {
    const before = await p3.evaluate((id) => {
      document.querySelector("#live-reroute").checked = false;
      return {
        stats: window.__eda.pickStats(), at: window.__eda.instanceAt(id),
        mesh: window.__eda.meshBuilds(), prof: window.__eda.profile(),
        history: window.__eda.history(),
      };
    }, bodyA.id);
    await p3.mouse.move(bodyA.x, bodyA.y);
    await p3.mouse.down();
    for (let i = 1; i <= 20; i++) {
      await p3.mouse.move(bodyA.x + i * 7, bodyA.y + i * 4);
      await p3.waitForTimeout(16);
    }
    const mid = await p3.evaluate(() => ({
      mesh: window.__eda.meshBuilds(), prof: window.__eda.profile(),
      stats: window.__eda.pickStats(),
    }));
    await p3.mouse.up();
    await p3.waitForTimeout(700);
    const after = await p3.evaluate((id) => {
      document.querySelector("#live-reroute").checked = true;
      return {
        stats: window.__eda.pickStats(), at: window.__eda.instanceAt(id),
        history: window.__eda.history(),
      };
    }, bodyA.id);
    const undone = await p3.evaluate((id) => {
      window.__eda.undo();
      return { at: window.__eda.instanceAt(id), history: window.__eda.history() };
    }, bodyA.id);
    return { before, mid, after, undone };
  })();
  check(dragged.after.stats.dragsStarted === dragged.before.stats.dragsStarted + 1 &&
        JSON.stringify(dragged.after.at) !== JSON.stringify(dragged.before.at),
    `past the threshold the same press DOES move ${bodyA.id} ` +
    `(${JSON.stringify(dragged.before.at?.at)} -> ${JSON.stringify(dragged.after.at?.at)}, 1 drag started)`,
    JSON.stringify({ before: dragged.before.at, after: dragged.after.at, stats: dragged.after.stats }));
  check(JSON.stringify(dragged.undone.at) === JSON.stringify(dragged.before.at),
    `...as ONE undo step: a single undo puts it back at ` +
    `${JSON.stringify(dragged.undone.at?.at)}`,
    JSON.stringify(dragged.undone));
  check(dragged.mid.mesh.cells === dragged.before.mesh.cells &&
        dragged.mid.mesh.texture === dragged.before.mesh.texture &&
        dragged.mid.prof.drawCalls === dragged.before.prof.drawCalls,
    `...and 20 drag frames cost 0 cell re-meshes, 0 texture builds and leave the ` +
    `draw calls at ${dragged.mid.prof.drawCalls} (${dragged.mid.mesh.matrixWrites - dragged.before.mesh.matrixWrites} matrix writes instead)`,
    JSON.stringify({ before: dragged.before.mesh, mid: dragged.mid.mesh,
      draws: [dragged.before.prof.drawCalls, dragged.mid.prof.drawCalls] }));

  // ---- 18. hover affordances ----------------------------------------------
  const hover = await (async () => {
    // The zoom is SEARCHED for, not assumed: the one that matters is the
    // furthest one at which a port is still an unambiguous target, because that
    // is where the old ray-vs-cone pick had already become unusable. The
    // hovered port's label has to come back there — hovering is how you know
    // WHICH port you are about to route.
    const name = await p3.evaluate(async () => {
      const e = window.__eda;
      e.frameAll();
      let best = null;
      for (const radius of [90, 130, 200, 320]) {
        e.zoom(radius);
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
        await new Promise((r) => setTimeout(r, 200));
        const hidden = [...document.querySelectorAll(".mk-label")]
          .filter((el) => el.style.display === "none")
          .map((el) => el.textContent.split(" :")[0]);
        const p = e.endpoints().find((q) => {
          if (!q.routable || !hidden.includes(q.name)) return false;
          const at = e.portScreen(q.name);
          return at && e.onCanvas(at[0], at[1], 60) && e.probeAt(at[0], at[1])?.id === q.name;
        });
        if (p) best = { port: p.name, radius, hiddenCount: hidden.length, labels: e.labels() };
      }
      // Leave the camera where the winning measurement was made.
      if (best) {
        e.zoom(best.radius);
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
        await new Promise((r) => setTimeout(r, 200));
      }
      return best ?? { port: null, hiddenCount: 0, labels: e.labels() };
    });
    if (!name.port) return { skipped: true, name };
    const at = await p3.evaluate((n) => window.__eda.portScreen(n), name.port);
    await p3.mouse.move(at[0], at[1]);
    await p3.waitForTimeout(400);
    const onPort = await p3.evaluate((n) => {
      const el = [...document.querySelectorAll(".mk-label")]
        .find((q) => q.textContent.startsWith(`${n} :`));
      return {
        cursor: document.querySelector("#canvas-wrap canvas").style.cursor,
        hoverPort: window.__eda.hoverPort(),
        hint: window.__eda.hint(),
        labelShown: !!el && el.style.display !== "none",
        labelHovered: !!el?.classList.contains("is-hovered"),
        labels: window.__eda.labels(),
      };
    }, name.port);
    // The SCREENSHOT is taken close in, where it is readable: same hover, a
    // zoom a person would actually be at.
    const closeAt = await p3.evaluate(async (n) => {
      const e = window.__eda;
      for (const radius of [34, 48, 70, 90]) {
        e.zoom(radius);
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
        await new Promise((r) => setTimeout(r, 150));
        const at = e.portScreen(n);
        if (at && e.onCanvas(at[0], at[1], 80) && e.probeAt(at[0], at[1])?.id === n) return at;
      }
      return null;
    }, name.port);
    if (closeAt) {
      await p3.mouse.move(closeAt[0] + 40, closeAt[1] + 30);
      await p3.waitForTimeout(120);
      await p3.mouse.move(closeAt[0], closeAt[1]);
      await p3.waitForTimeout(500);
    }
    const shot = await snap(p3, "hover-port-affordance");
    const bodyB = await bodyPixel(null);
    let onBody = null;
    if (bodyB) {
      await p3.mouse.move(bodyB.x, bodyB.y);
      await p3.waitForTimeout(300);
      onBody = await p3.evaluate(() => ({
        cursor: document.querySelector("#canvas-wrap canvas").style.cursor,
        hoverPort: window.__eda.hoverPort(),
        hint: window.__eda.hint(),
      }));
    }
    return { name, onPort, onBody, shot };
  })();
  if (hover.skipped) {
    results.push({ ok: true, label: "hover affordance SKIPPED (no decluttered port on screen)", skipped: true });
  } else {
    check(hover.onPort.cursor === "crosshair" && hover.onBody?.cursor === "move",
      `the cursor says which gesture the press will be: CROSSHAIR over a port, ` +
      `MOVE over a body (${hover.onPort.cursor} / ${hover.onBody?.cursor})`,
      JSON.stringify({ port: hover.onPort.cursor, body: hover.onBody?.cursor }));
    check(hover.onPort.labelShown && hover.onPort.labelHovered && hover.onPort.hoverPort === hover.name.port,
      `hovering ${hover.name.port} brings back its label at orbit radius ` +
      `${hover.name.radius}, where ${hover.name.hiddenCount} labels are decluttered ` +
      `away, and emphasises the marker`,
      JSON.stringify(hover.onPort));
    check(/click to start a bus from/i.test(hover.onPort.hint) &&
          hover.onPort.hint.includes(hover.name.port) &&
          /will NOT move/i.test(hover.onPort.hint),
      `...and the hint bar says what the click DOES: "${hover.onPort.hint.replace(/\s+/g, " ").slice(0, 92)}…"`,
      hover.onPort.hint);
  }

  // ---- 19. connect mode: bodies are not there ------------------------------
  const connectMode = await p3.evaluate(async () => {
    const e = window.__eda;
    const s = window.__edaStudio();
    const rect = document.querySelector("#canvas-wrap canvas").getBoundingClientRect();
    const bodyHits = () => {
      let n = 0;
      for (let y = rect.top + 8; y < rect.bottom - 8; y += 12) {
        for (let x = rect.left + 8; x < rect.right - 8; x += 12) {
          if (e.probeAt(x, y)?.kind === "instance") n++;
        }
      }
      return n;
    };
    const portHits = () => e.endpoints().filter((p) => {
      const at = e.portScreen(p.name);
      return at && e.probeAt(at[0], at[1])?.kind === "port";
    }).length;
    const off = { bodies: bodyHits(), ports: portHits() };
    e.key("c");
    await new Promise((r) => requestAnimationFrame(r));
    const on = { bodies: bodyHits(), ports: portHits(), mode: e.connectMode(), hint: e.hint() };
    const placements = [...s.instances.values()].map((i) => [...i.at]);
    e.key("c");
    return { off, on, placements, back: e.connectMode() };
  });
  check(connectMode.on.mode === true && connectMode.on.bodies === 0 &&
        connectMode.off.bodies > 0 && connectMode.on.ports === connectMode.off.ports &&
        connectMode.back === false,
    `connect mode (C, or hold Alt): ${connectMode.off.bodies} body pixels become 0 and all ` +
    `${connectMode.on.ports} ports stay pickable — in a dense scene no press can be a move`,
    JSON.stringify(connectMode));
  check(/connect mode/i.test(connectMode.on.hint),
    `...and the hint bar says so ("${connectMode.on.hint.replace(/\s+/g, " ").slice(0, 72)}…")`,
    connectMode.on.hint);
  await p3.evaluate(() => window.__eda.setConnectMode(true));
  await p3.waitForTimeout(500);
  await snap(p3, "connect-mode-bodies-unpickable");
  await p3.evaluate(() => window.__eda.setConnectMode(false));

  // The scene still draws what the engine says it is, after all of that
  // pointer traffic.
  const p3consistent = await p3.evaluate(() => window.__eda.consistency());
  check(p3consistent.ok === true,
    `after the whole picking suite the render still matches the engine ` +
    `(${p3consistent.layers.length} layers)`,
    JSON.stringify(p3consistent.mismatches ?? []));
  check(p3errors.length === 0, `no console errors on the picking page (${p3errors.length})`,
    p3errors.slice(0, 3).join(" | "));
  await p3.close();

  await browser.close();
} finally {
  stop();
}

const good = results.filter((r) => r.ok).length;
console.log(`\n${good}/${results.length} checks passed`);
writeFileSync(path.join(docs, "verify-out.json"),
  JSON.stringify({ when: new Date().toISOString(), passed: good, total: results.length, results }, null, 2));
process.exit(good === results.length ? 0 : 1);
