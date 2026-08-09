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
  check(flew.after && flew.after.target.join() === said.h.at.join(),
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
    check(busMesh.after.mesh.buses - busMesh.before.mesh.buses === 1 &&
          busMesh.after.reads.busDump - busMesh.before.reads.busDump === 1 &&
          busMesh.after.mesh.cells === busMesh.before.mesh.cells,
      `re-routing ${busMesh.bus} re-meshes exactly 1 bus and 0 cells, in ` +
      `${busMesh.ms.toFixed(0)} ms`,
      JSON.stringify(busMesh));
  }


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

  await browser.close();
} finally {
  stop();
}

const good = results.filter((r) => r.ok).length;
console.log(`\n${good}/${results.length} checks passed`);
writeFileSync(path.join(docs, "verify-out.json"),
  JSON.stringify({ when: new Date().toISOString(), passed: good, total: results.length, results }, null, 2));
process.exit(good === results.length ? 0 : 1);
