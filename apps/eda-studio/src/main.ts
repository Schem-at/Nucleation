/** Boot + UI wiring: the studio state, the canvas, the outliner panels and
 *  the keyboard/selection interaction model.
 *
 *  Interaction model (the thing the first version got wrong): nothing is
 *  modal-by-accident. There is exactly one `mode`, the hint bar always spells
 *  out what the next click does, and Esc always cancels back to `idle`.
 */
import { loadEngine } from "./engine";
import { Studio, type Vec3, type PortInfo } from "./studio";
import {
  Viewer, type PortMarker, type GateMarker, type InstanceMarker, type Selection,
} from "./viewer";
import { verilogToBlif, guessTop } from "./yosys";

const $ = <T extends HTMLElement = HTMLElement>(sel: string) =>
  document.querySelector(sel) as T;

const esc = (s: unknown) =>
  String(s).replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);

const log = (msg: string) => {
  const el = $("#log");
  el.textContent = `${msg}\n${el.textContent ?? ""}`.slice(0, 8000);
  console.log(`[eda] ${msg}`);
};
const toast = (() => {
  let timer = 0;
  return (msg: string, kind: "ok" | "err" = "ok") => {
    const el = $("#toast");
    el.textContent = msg;
    el.className = kind === "err" ? "is-err" : "";
    el.style.display = "block";
    clearTimeout(timer);
    timer = window.setTimeout(() => (el.style.display = "none"), kind === "err" ? 6000 : 2800);
  };
})();

const DEMO_VERILOG = `module add2(input [1:0] a, input [1:0] b, output [2:0] y);
  assign y = a + b;
endmodule
`;

/** What the next canvas click means. */
type Mode =
  | { kind: "idle" }
  | { kind: "placing"; cell: string }
  | { kind: "connecting"; from: string }
  | { kind: "grabbing"; instance: string; origin: Vec3 };

async function boot() {
  const { core, d } = await loadEngine();
  $("#status").textContent = "engine ready";
  (window as any).__edaCore = core;

  let studio = new Studio(core, d, "studio");
  let mode: Mode = { kind: "idle" };
  let selection: Selection = null;
  let lastLive = 0;
  /** Endpoints as of the last refresh, for the connect flow and panels. */
  let endpoints: ReturnType<Studio["allEndpoints"]> = [];

  const endpoint = (name: string) => endpoints.find((e) => e.name === name);

  // ---- hint bar: always says what the next click does --------------------

  function setHint() {
    const el = $("#hint");
    const nav = "drag: orbit · wheel: zoom";
    if (mode.kind === "placing") {
      el.innerHTML = `<b>click the ground</b> to place <code>${esc(mode.cell)}</code>` +
        ` &nbsp;·&nbsp; <kbd>Esc</kbd> cancel`;
      return;
    }
    if (mode.kind === "connecting") {
      const from = endpoint(mode.from);
      el.innerHTML = `bus from <code>${esc(mode.from)}</code> ` +
        `(${esc(from?.ty ?? "")}) &nbsp;→&nbsp; <b>click a blue ▼ input port</b> to finish` +
        ` &nbsp;·&nbsp; <kbd>Esc</kbd> cancel`;
      return;
    }
    if (mode.kind === "grabbing") {
      el.innerHTML = `moving <code>${esc(mode.instance)}</code> — <b>move the mouse, click to drop</b>` +
        ` &nbsp;·&nbsp; <kbd>Esc</kbd> put it back`;
      return;
    }
    if (selection?.kind === "instance") {
      el.innerHTML = `<code>${esc(selection.id)}</code> selected &nbsp;·&nbsp; ` +
        `<kbd>R</kbd> rotate 90° &nbsp; <kbd>G</kbd> move &nbsp; <kbd>Del</kbd> remove &nbsp; ` +
        `<kbd>Esc</kbd> deselect &nbsp;·&nbsp; ${nav}`;
      return;
    }
    const driver = endpoints.some((e) => e.routable && e.kind === "input");
    el.innerHTML = driver
      ? `<b>click a green ▲ output port</b> to start a bus, or click a component to select it` +
        ` &nbsp;·&nbsp; ${nav}`
      : `<b>Load demo</b>, or place a cell from the Library` + ` &nbsp;·&nbsp; ${nav}`;
  }

  function setMode(next: Mode) {
    mode = next;
    if (next.kind !== "connecting") viewer.setGhost(null);
    // The ghost's far end follows the cursor; the pointermove handler below
    // draws it as soon as the mouse moves.
    setHint();
  }

  function select(sel: Selection) {
    selection = sel;
    viewer.setSelection(sel);
    renderPanels();
    setHint();
  }

  // ---- rendering --------------------------------------------------------

  const viewer = new Viewer($("#canvas-wrap"), {
    onPortClick(name) {
      const p = endpoint(name);
      if (!p) return;
      if (mode.kind === "connecting") {
        if (mode.from === name) { setMode({ kind: "idle" }); return; }
        const from = mode.from;
        setMode({ kind: "idle" });
        connectPorts(from, name);
        return;
      }
      select({ kind: "port", id: name });
      if (!p.routable) {
        toast(`${name} is executor-only IO — no bus can land on it. ${p.blocked ?? ""}`, "err");
        log(`port ${name}: NOT routable — ${p.blocked ?? "no dust connection cell"}`);
        return;
      }
      if (p.kind !== "input") {
        toast(`${name} receives a bus; start from an output (green ▲) instead`, "err");
        return;
      }
      setMode({ kind: "connecting", from: name });
      toast(`bus from ${name} — now click an input port`);
    },
    onPortHover(name) {
      const p = name ? endpoint(name) : null;
      $("#hover-info").textContent = p
        ? `${p.name} · ${p.kind === "input" ? "drives" : "receives"} · ${p.ty}[${p.width}]` +
          ` @ ${p.anchor.join(",")}${p.routable ? "" : ` · NOT ROUTABLE: ${p.blocked ?? ""}`}`
        : "";
    },
    onInstanceClick(name) {
      select({ kind: "instance", id: name });
    },
    onDragMove(kind, id, ground) {
      if (!($("#live-reroute") as HTMLInputElement).checked) return;
      const now = performance.now();
      if (now - lastLive < 250) return; // bounded live-reroute rate
      lastLive = now;
      applyMove(kind, id, ground, "live");
    },
    onDragEnd(kind, id, ground) {
      applyMove(kind, id, ground, "drop");
    },
    onGroundClick(ground) {
      if (mode.kind === "placing") {
        const cell = mode.cell;
        setMode({ kind: "idle" });
        try {
          const inst = studio.placeInstance(cell, ground);
          log(`placed ${inst.name} (${cell}) at ${ground.join(",")}`);
          select({ kind: "instance", id: inst.name });
          toast(`${inst.name} placed — R rotates, Del removes, its ports are on the canvas`);
        } catch (err) {
          toast(String(err), "err");
          log(`place failed: ${err}`);
        }
        return;
      }
      if (mode.kind === "grabbing") {
        const inst = mode.instance;
        setMode({ kind: "idle" });
        applyMove("instance", inst, ground, "drop");
        return;
      }
      if (mode.kind === "connecting") { setMode({ kind: "idle" }); return; }
      select(null);
    },
  });

  // Ghost line + grab preview follow the cursor.
  $("#canvas-wrap").addEventListener("pointermove", (e) => {
    if (mode.kind === "connecting") {
      const from = viewer.portMarkerPos(mode.from);
      const to = from ? viewer.worldPoint(e as PointerEvent, from[1]) : null;
      if (from && to) viewer.setGhost(from, to);
    } else if (mode.kind === "grabbing") {
      const inst = studio.instances.get(mode.instance);
      if (!inst) return;
      const p = viewer.worldPoint(e as PointerEvent, inst.at[1]);
      if (p) applyMove("instance", mode.instance, [Math.round(p.x), inst.at[1], Math.round(p.z)], "live");
    }
  });

  function applyMove(kind: "instance" | "gate", id: string, ground: Vec3, phase: "live" | "drop") {
    try {
      if (kind === "instance") {
        const report = studio.moveInstance(id, ground);
        if (phase === "drop") reportBuses(`move ${id} -> ${ground.join(",")}`, report);
      } else {
        const [busName, gateName] = id.split(" ");
        const bus = studio.buses.get(busName);
        const gate = bus?.gates.find((g) => g.name === gateName);
        if (!gate) return;
        const report = studio.moveGate(busName, gateName, [ground[0], gate.anchor[1], ground[2]]);
        if (phase === "drop") {
          log(`gate ${id} -> ${ground[0]},${gate.anchor[1]},${ground[2]}  state=${report.state} segments=${report.rerouted_segments}`);
          if (report.state.startsWith("failed")) toast(`bus ${busName} ${report.state} (shown red)`, "err");
        }
      }
    } catch (err) {
      if (phase === "drop") {
        toast(String(err), "err");
        log(`move failed: ${err}`);
      }
    }
  }

  function reportBuses(what: string, report: { rerouted: string[]; failed: Record<string, string>; removed_buses?: string[] }) {
    const failed = Object.entries(report.failed ?? {});
    const removed = report.removed_buses ?? [];
    log(`${what}` +
      (removed.length ? `  removed=[${removed}]` : "") +
      `  rerouted=[${report.rerouted ?? []}]` +
      (failed.length ? `  FAILED: ${failed.map(([b, r]) => `${b}: ${r}`).join("; ")}` : ""));
    if (failed.length) {
      toast(`bus failed: ${failed.map(([b]) => b).join(", ")} — shown red, see Buses panel`, "err");
    } else if (removed.length) {
      toast(`removed with it: ${removed.join(", ")}`);
    }
  }

  function refresh() {
    endpoints = studio.allEndpoints();
    try {
      viewer.setLayers(studio.layers());
    } catch (err) {
      log(`layer render failed: ${err}`);
    }
    if (viewer.isTextured()) void remesh();
    const ports: PortMarker[] = endpoints.map((p) => ({
      name: p.name, kind: p.kind, anchor: p.anchor, step: p.step,
      width: p.width, ty: p.ty, routable: p.routable, blocked: p.blocked,
      instance: p.instance,
    }));
    const gates: GateMarker[] = [...studio.buses.values()].flatMap((b) => {
      const width = endpoint(b.driver)?.width ?? 1;
      return b.gates.map((g) => ({ bus: b.name, name: g.name, anchor: g.anchor, step: g.step, width }));
    });
    const instances: InstanceMarker[] = [...studio.instances.values()].map((i) => ({
      name: i.name, cell: i.cell, at: i.at, rot: i.rot,
      dims: studio.cells.get(i.cell)?.dims ?? [1, 1, 1],
    }));
    viewer.setMarkers(ports, gates, instances);
    if (selection?.kind === "instance" && !studio.instances.has(selection.id)) selection = null;
    viewer.setSelection(selection);
    renderPanels();
    setHint();
  }
  studio.onChange = refresh;

  // ---- keyboard ---------------------------------------------------------

  window.addEventListener("keydown", (e) => {
    const t = e.target as HTMLElement;
    if (t && /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName)) return;
    const key = e.key.toLowerCase();
    if (e.key === "Escape") {
      if (mode.kind === "grabbing") {
        applyMove("instance", mode.instance, mode.origin, "drop");
      }
      setMode({ kind: "idle" });
      select(null);
      return;
    }
    if (!selection || selection.kind !== "instance") return;
    const name = selection.id;
    if (key === "r") {
      e.preventDefault();
      try {
        const report = studio.rotateInstance(name, e.shiftKey ? -90 : 90);
        reportBuses(`rotate ${name} -> ${report.rot}°`, report);
        toast(`${name} rotated to ${report.rot}°`);
      } catch (err) {
        toast(String(err), "err");
      }
      return;
    }
    if (e.key === "Delete" || e.key === "Backspace") {
      e.preventDefault();
      try {
        const report = studio.removeInstance(name);
        select(null);
        reportBuses(`deleted ${name}`, report);
        toast(`${name} deleted` +
          (report.removed_buses.length ? ` — ripped ${report.removed_buses.join(", ")}` : ""));
      } catch (err) {
        toast(String(err), "err");
      }
      return;
    }
    if (key === "g") {
      e.preventDefault();
      const inst = studio.instances.get(name);
      if (inst) setMode({ kind: "grabbing", instance: name, origin: [...inst.at] as Vec3 });
    }
  });

  // ---- outliner panels --------------------------------------------------

  function renderPanels() {
    // (a) LIBRARY
    $("#cell-list").innerHTML = [...studio.cells.values()].map((c) => {
      const io = c.ports
        ? c.ports.map((p) => `${p.dir === "in" ? "▸" : "◂"}${esc(p.name)}`).join(" ")
        : "";
      return `<div class="item${mode.kind === "placing" && mode.cell === c.name ? " is-active" : ""}"
                   data-place="${esc(c.name)}" title="click to place">
        <span class="name">${esc(c.name)}</span>
        <span class="meta">${c.dims.join("×")} · ${c.source}${c.warnings.length ? ` · ⚠ ${c.warnings.length}` : ""}</span>
        ${io ? `<span class="meta io">${io}</span>` : ""}
      </div>`;
    }).join("") || `<span class="meta">loading the enhanced cell library…</span>`;
    for (const el of $("#cell-list").querySelectorAll<HTMLElement>("[data-place]")) {
      el.addEventListener("click", () => {
        setMode({ kind: "placing", cell: el.dataset.place! });
        toast(`click the canvas ground to place ${el.dataset.place}`);
        renderPanels();
      });
    }

    // (b) INSTANCES + their ports
    const insts = [...studio.instances.values()];
    $("#instance-list").innerHTML = insts.map((i) => {
      const mine = endpoints.filter((p) => p.instance === i.name);
      const sel = selection?.kind === "instance" && selection.id === i.name;
      return `<div class="item${sel ? " is-selected" : ""}" data-inst="${esc(i.name)}">
        <span class="name">${esc(i.name)}</span>
        <span class="meta">${esc(i.cell)} @ ${i.at.join(",")} · rot ${i.rot}°</span>
        <div class="ports">${mine.map((p) => `
          <button class="port-chip ${p.kind === "input" ? "drives" : "receives"}${p.routable ? "" : " blocked"}"
                  data-port="${esc(p.name)}"
                  title="${p.routable ? "click to start/finish a bus" : esc(p.blocked ?? "not routable")}">
            ${p.kind === "input" ? "▲" : "▼"} ${esc(p.port ?? p.name.split(".").pop())} : ${esc(p.ty)}${p.routable ? "" : " ✗"}
          </button>`).join("")}</div>
        <div class="row">
          <button data-rot="${esc(i.name)}">R ↻</button>
          <button data-del="${esc(i.name)}">Del ✕</button>
        </div>
      </div>`;
    }).join("") || `<span class="meta">none — click a cell in the Library, then click the ground</span>`;
    for (const el of $("#instance-list").querySelectorAll<HTMLElement>("[data-inst]")) {
      el.addEventListener("click", (ev) => {
        if ((ev.target as HTMLElement).closest("button")) return;
        select({ kind: "instance", id: el.dataset.inst! });
      });
    }
    for (const el of $("#instance-list").querySelectorAll<HTMLElement>("[data-rot]")) {
      el.addEventListener("click", () => {
        const r = studio.rotateInstance(el.dataset.rot!);
        reportBuses(`rotate ${el.dataset.rot} -> ${r.rot}°`, r);
      });
    }
    for (const el of $("#instance-list").querySelectorAll<HTMLElement>("[data-del]")) {
      el.addEventListener("click", () => {
        const r = studio.removeInstance(el.dataset.del!);
        reportBuses(`deleted ${el.dataset.del}`, r);
      });
    }
    for (const el of $("#instance-list").querySelectorAll<HTMLElement>("[data-port]")) {
      el.addEventListener("click", () => viewerPortClick(el.dataset.port!));
    }

    // design ports (declared on loose hardware)
    const declared = endpoints.filter((p) => !p.instance);
    $("#port-list").innerHTML = declared.map((p) => `
      <div class="item"><button class="port-chip ${p.kind === "input" ? "drives" : "receives"}"
             data-port="${esc(p.name)}">${p.kind === "input" ? "▲" : "▼"} ${esc(p.name)} : ${esc(p.ty)}</button>
        <span class="meta">@ ${p.anchor.join(",")} step ${p.step.join(",")}</span></div>`).join("")
      || `<span class="meta">none — Load demo declares typed ports</span>`;
    for (const el of $("#port-list").querySelectorAll<HTMLElement>("[data-port]")) {
      el.addEventListener("click", () => viewerPortClick(el.dataset.port!));
    }

    // (c) BUSES
    $("#bus-list").innerHTML = [...studio.buses.values()].map((b) => {
      const { state, reason } = studio.busStateDetail(b.name);
      const cls = state === "failed" ? "state-failed"
        : state === "routed" ? "state-routed" : "state-intended";
      return `<div class="item">
        <span class="swatch" style="background:#${b.color.toString(16).padStart(6, "0")}"></span>
        <span class="name">${esc(b.name)}</span> <span class="${cls}">${state}</span>
        <span class="meta">${esc(b.driver)} → ${esc(b.sinks.join(","))}${b.gates.length ? ` · gates: ${esc(b.gates.map((g) => g.name).join(","))}` : ""}</span>
        ${reason ? `<span class="meta reason">${esc(reason)}</span>` : ""}
        <div class="row">
          <button data-rip="${esc(b.name)}">Rip</button>
          <button data-reroute="${esc(b.name)}">Re-route</button>
          <button data-delbus="${esc(b.name)}">Delete</button>
        </div>
      </div>`;
    }).join("") || `<span class="meta">click an output port then an input port to route one</span>`;
    for (const el of $("#bus-list").querySelectorAll<HTMLElement>("[data-rip]")) {
      el.addEventListener("click", () => { studio.ripBus(el.dataset.rip!); log(`ripped ${el.dataset.rip}`); });
    }
    for (const el of $("#bus-list").querySelectorAll<HTMLElement>("[data-reroute]")) {
      el.addEventListener("click", () => {
        const state = studio.rerouteBus(el.dataset.reroute!);
        log(`re-routed ${el.dataset.reroute}: ${state}`);
        if (state.startsWith("failed")) toast(`${el.dataset.reroute} still failed: ${state}`, "err");
        else toast(`${el.dataset.reroute} ${state}`);
      });
    }
    for (const el of $("#bus-list").querySelectorAll<HTMLElement>("[data-delbus]")) {
      el.addEventListener("click", () => { studio.removeBus(el.dataset.delbus!); log(`deleted bus ${el.dataset.delbus}`); });
    }

    renderPoke();
  }

  /** A port chip in the outliner behaves exactly like clicking the marker on
   *  the canvas: same connect flow, same refusals. */
  function viewerPortClick(name: string) {
    select({ kind: "port", id: name });
    onPortChip(name);
  }
  function onPortChip(name: string) {
    const p = endpoint(name);
    if (!p) return;
    if (mode.kind === "connecting" && mode.from !== name) {
      const from = mode.from;
      setMode({ kind: "idle" });
      connectPorts(from, name);
      return;
    }
    if (!p.routable) {
      toast(`${name} is executor-only IO — no bus can land on it. ${p.blocked ?? ""}`, "err");
      return;
    }
    if (p.kind === "input") {
      setMode({ kind: "connecting", from: name });
      toast(`bus from ${name} — now click an input port`);
    } else {
      toast(`${name} receives a bus; start from an output (green ▲)`, "err");
    }
  }

  function renderPoke() {
    const panel = $("#poke-panel");
    if (!studio.executor) {
      panel.innerHTML = `<span class="meta">Bake first — then drive inputs and read outputs</span>`;
      return;
    }
    const { inputs, outputs } = studio.contractPorts();
    panel.innerHTML = `
      ${inputs.map((p) => `
        <div class="row"><label>${esc(p.name)}[${p.width}]</label>
          <input type="number" style="width:90px" data-poke-in="${esc(p.name)}" value="0" /></div>`).join("")}
      <div class="row"><button id="btn-poke" class="primary">Set + settle</button></div>
      ${outputs.map((p) => `
        <div class="row"><label>${esc(p.name)}[${p.width}]</label>
          <span data-poke-out="${esc(p.name)}" class="state-routed">–</span></div>`).join("")}`;
    $("#btn-poke")?.addEventListener("click", () => {
      try {
        for (const el of panel.querySelectorAll<HTMLInputElement>("[data-poke-in]")) {
          studio.executor.set(el.dataset.pokeIn!, Number(el.value) >>> 0);
        }
        studio.executor.settle(2000);
        for (const el of panel.querySelectorAll<HTMLElement>("[data-poke-out]")) {
          const v = studio.executor.get(el.dataset.pokeOut!);
          el.textContent = `${v} (0x${Number(v).toString(16)})`;
        }
        log("poke: inputs set, settled, outputs read");
      } catch (err) {
        toast(String(err), "err");
        log(`poke failed: ${err}`);
      }
    });
  }

  // ---- actions ----------------------------------------------------------

  function connectPorts(a: string, b: string) {
    const pa = endpoint(a), pb = endpoint(b);
    if (!pa || !pb) return;
    const driver = pa.kind === "input" ? pa : pb;
    const sink = pa.kind === "input" ? pb : pa;
    if (driver.kind !== "input" || sink.kind !== "output") {
      toast("a bus needs one driver (green ▲) and one sink (blue ▼)", "err");
      return;
    }
    if (!driver.routable || !sink.routable) {
      const bad = !driver.routable ? driver : sink;
      toast(`${bad.name} is executor-only IO: ${bad.blocked ?? "no dust connection cell"}`, "err");
      log(`route refused: ${bad.name} — ${bad.blocked}`);
      return;
    }
    if (driver.width !== sink.width) {
      toast(`width mismatch: ${driver.name}[${driver.width}] → ${sink.name}[${sink.width}]`, "err");
      return;
    }
    if (driver.ty !== sink.ty) {
      toast(`type mismatch: ${driver.name} is ${driver.ty}, ${sink.name} is ${sink.ty}`, "err");
      return;
    }
    try {
      const bus = studio.routeBus(driver.name, [sink.name]);
      const { state, reason } = studio.busStateDetail(bus.name);
      log(`routed ${bus.name}: ${driver.name} -> ${sink.name} (${state}${reason ? `: ${reason}` : ""})`);
      if (state === "failed") toast(`${bus.name} FAILED: ${reason}`, "err");
      else toast(`${bus.name} routed: ${driver.name} → ${sink.name}`);
    } catch (err) {
      toast(String(err).replace(/^Error:\s*/, ""), "err");
      log(`route failed: ${err}`);
    }
  }

  $("#cell-file").addEventListener("change", async (e) => {
    const files = (e.target as HTMLInputElement).files ?? [];
    for (const f of files) {
      try {
        const bytes = new Uint8Array(await f.arrayBuffer());
        const name = f.name.replace(/\.[^.]+$/, "");
        const info = studio.addCellFromBytes(name, bytes);
        log(`cell ${name}: ${info.dims.join("×")}${info.warnings.length ? `, warnings: ${info.warnings.join("; ")}` : ""}`);
      } catch (err) {
        toast(`${f.name}: ${err}`, "err");
        log(`cell load failed (${f.name}): ${err}`);
      }
    }
  });

  // ---- textured view ----------------------------------------------------

  async function remesh(): Promise<boolean> {
    if (!studio.pack) return false;
    const t0 = performance.now();
    try {
      const glb = studio.meshGlb();
      if (!glb) return false;
      await viewer.setTexturedGlb(glb);
      viewer.setLayers(studio.layers()); // bus layers stay abstract on top
      log(`meshed ${(glb.byteLength / 1024).toFixed(0)}KB GLB in ${(performance.now() - t0).toFixed(0)}ms`);
      return true;
    } catch (err) {
      toast(`meshing failed: ${err}`, "err");
      log(`meshing failed: ${err}`);
      return false;
    }
  }

  $("#pack-file").addEventListener("change", async (e) => {
    const f = (e.target as HTMLInputElement).files?.[0];
    if (!f) return;
    try {
      $("#pack-status").textContent = "loading pack…";
      studio.loadPack(new Uint8Array(await f.arrayBuffer()));
      const i = studio.packInfo!;
      $("#pack-status").textContent =
        `${i.blockstates} blockstates · ${i.models} models · ${i.textures} textures`;
      log(`resource pack: ${f.name} (${i.blockstates} blockstates, ${i.textures} textures)`);
      ($("#textured") as HTMLInputElement).checked = true;
      if (await remesh()) toast("textured view on");
    } catch (err) {
      $("#pack-status").textContent = String(err).slice(0, 200);
      toast(`pack failed: ${err}`, "err");
    }
  });

  $("#textured").addEventListener("change", async (e) => {
    const on = (e.target as HTMLInputElement).checked;
    if (on && !studio.pack) {
      toast("load a resource pack ZIP first", "err");
      (e.target as HTMLInputElement).checked = false;
      return;
    }
    if (on) await remesh();
    viewer.setTexturedVisible(on);
    viewer.setLayers(studio.layers());
  });

  $("#show-io").addEventListener("change", (e) => {
    viewer.setShowIo((e.target as HTMLInputElement).checked);
  });

  ($("#verilog") as HTMLTextAreaElement).value = DEMO_VERILOG;
  $("#btn-compile").addEventListener("click", async () => {
    const src = ($("#verilog") as HTMLTextAreaElement).value;
    const name = ($("#verilog-name") as HTMLInputElement).value || guessTop(src) || "cell";
    const status = $("#verilog-status");
    try {
      status.textContent = "yosys…";
      const blif = await verilogToBlif(src, guessTop(src) ?? name);
      status.textContent = "compiling to redstone…";
      await new Promise((r) => setTimeout(r)); // let the status paint
      const cell = core.Hdl.compileBlif(blif, name, false);
      const contract = core.Hdl.compileBlifContract(blif, name);
      cell.setCellContractJson(contract);
      const info = studio.addCellSchematic(name, cell, "verilog");
      status.textContent = `ok: ${cell.blockCount()} blocks, ${info.dims.join("×")}`;
      log(`verilog cell ${name}: ${cell.blockCount()} blocks`);
    } catch (err) {
      status.textContent = String(err).slice(0, 300);
      log(`verilog compile failed: ${err}`);
    }
  });

  $("#btn-check").addEventListener("click", () => {
    try {
      const r = studio.check();
      log(`check: clean=${r.clean} drc=${r.drc.length} rules=${r.rules.length}` +
          (r.clean ? "" : `\n${JSON.stringify(r.raw, null, 1).slice(0, 1500)}`));
      toast(r.clean ? "check clean" : `check DIRTY — ${r.drc.length} DRC, see log`,
        r.clean ? "ok" : "err");
    } catch (err) {
      toast(String(err), "err");
    }
  });

  $("#btn-bake").addEventListener("click", () => {
    try {
      const t0 = performance.now();
      studio.bake(4000);
      log(`baked + executor ready in ${(performance.now() - t0).toFixed(0)}ms`);
      toast("baked — poke panel is live");
    } catch (err) {
      toast(String(err), "err");
      log(`bake failed: ${err}`);
    }
  });

  const download = (bytes: Uint8Array, filename: string) => {
    const url = URL.createObjectURL(new Blob([bytes as unknown as BlobPart]));
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  };
  for (const [id, suffix] of [
    ["#btn-export-schem", "schem"], ["#btn-export-litematic", "litematic"], ["#btn-export-nucm", "nucm"],
  ] as const) {
    $(id).addEventListener("click", () => {
      try {
        const bytes = studio.exportBytes(suffix);
        download(bytes, `${studio.name}.${suffix}`);
        log(`exported ${studio.name}.${suffix} (${bytes.length} bytes)`);
      } catch (err) {
        toast(String(err), "err");
        log(`export failed: ${err}`);
      }
    });
  }

  // ---- cell library: the enhanced community cells ------------------------

  /** The enhanced `.schem` cells are copied into `public/cells/` with a
   *  manifest by `npm run sync-cells`; each carries an embedded contract, so
   *  placing one immediately exposes typed ports. */
  async function loadLibrary() {
    try {
      const res = await fetch(new URL("cells/manifest.json", document.baseURI));
      if (!res.ok) throw new Error(`manifest ${res.status}`);
      const names = (await res.json()) as string[];
      for (const file of names) {
        try {
          const buf = await (await fetch(new URL(`cells/${file}`, document.baseURI))).arrayBuffer();
          const name = file.replace(/_enhanced\.schem$/, "").replace(/\.schem$/, "");
          studio.addCellFromBytes(name, new Uint8Array(buf));
        } catch (err) {
          log(`library: ${file} skipped (${err})`);
        }
      }
      log(`library: ${studio.cells.size} cells with embedded contracts`);
    } catch (err) {
      log(`library unavailable (${err}) — use "Load cell" to add .schem files`);
    }
  }

  // ---- demo -------------------------------------------------------------

  const STONE = "minecraft:stone";
  const DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";
  const LAMP = "minecraft:redstone_lamp[lit=false]";
  const LEVER = "minecraft:lever[face=floor,facing=north,powered=false]";

  /** The MEANINGFUL demo: a real 8-bit adder cell from the community
   *  library, its `sum` bussed to a lamp readout, and a second adder placed
   *  ready to wire — which is also how the app teaches the one thing that
   *  surprises everyone: you CANNOT bus into `a`/`b`, because those ports are
   *  levers, and nothing in redstone drives a lever. The adder's `sum` has
   *  dust beside its lamps, so THAT end is routable; the executor drives the
   *  levers instead. */
  async function loadAdderDemo() {
    const s = core.Schematic.create("adder-chain");
    const N = 8, STEP: Vec3 = [0, 2, 0];
    const lampBank = (x: number, z: number): Vec3 => {
      for (let i = 0; i < N; i++) {
        const y = 2 + 2 * i;
        s.setBlockFromString(x, y - 1, z, LAMP);
        s.setBlockFromString(x, y, z, DUST);
      }
      return [x, 2, z];
    };
    const readout = lampBank(40, 4);

    studio = new Studio(core, d, "adder-chain", s);
    studio.onChange = refresh;
    await loadLibrary();
    const add = [...studio.cells.keys()].find((n) => n.startsWith("ADD007"));
    if (!add) {
      log("demo: ADD007 not in the library; falling back to the crossing demo");
      loadCrossingDemo();
      return;
    }
    // ADD007's `sum` dust tap sits at cell-local y=3; at.y=-1 lands bit 0 on
    // the canonical y=2 bus level so the trunk is level end to end.
    studio.placeInstance(add, [0, -1, 4]);
    studio.placeInstance(add, [0, -1, 24]);
    studio.declarePort({ name: "sum_out", kind: "output", anchor: readout, step: STEP, width: N, ty: "uint" });
    try {
      const bus = studio.routeBus("u0.sum", ["sum_out"]);
      const { state, reason } = studio.busStateDetail(bus.name);
      log(`demo: bus u0.sum -> sum_out = ${state}${reason ? `: ${reason}` : ""}`);
    } catch (err) {
      log(`demo bus failed: ${err}`);
    }
    log("demo: two ADD007 adders + an 8-bit lamp readout.");
    log("  u0.sum is routable (dust beside its lamp column) and is bussed to sum_out.");
    log("  u0.a / u0.b / u1.a are LEVER banks — greyed ✗ on the canvas: no bus can");
    log("  drive a lever, so cell-to-cell chaining needs a cell with dust ports");
    log("  (compile one from Verilog). Drive the levers with Bake -> poke instead:");
    log("  set u0.a=99, u0.b=28 and sum_out reads 127.");
    refresh();
    toast("Bake, then poke u0.a=99 u0.b=28 → sum_out = 127");
  }

  /** The original DESIGN_SPEC acceptance sketch: two crossing 8-bit buses. */
  function loadCrossingDemo() {
    const N = 8, STEP: Vec3 = [0, 2, 0];
    const s = core.Schematic.create("crossing");
    const leverBank = (x: number, z: number, dx: number, dz: number): Vec3 => {
      for (let i = 0; i < N; i++) {
        const y = 2 + 2 * i;
        s.setBlockFromString(x, y - 1, z, STONE);
        s.setBlockFromString(x, y, z, LEVER);
        s.setBlockFromString(x + dx, y - 1, z + dz, STONE);
        s.setBlockFromString(x + dx, y, z + dz, DUST);
      }
      return [x + dx, 2, z + dz];
    };
    const lampBank = (x: number, z: number): Vec3 => {
      for (let i = 0; i < N; i++) {
        const y = 2 + 2 * i;
        s.setBlockFromString(x, y - 1, z, LAMP);
        s.setBlockFromString(x, y, z, DUST);
      }
      return [x, 2, z];
    };
    const aIn = leverBank(0, 8, 1, 0), aOut = lampBank(16, 8);
    const bIn = leverBank(8, 0, 0, 1), bOut = lampBank(8, 16);

    studio = new Studio(core, d, "crossing", s);
    studio.onChange = refresh;
    const port = (name: string, kind: PortInfo["kind"], anchor: Vec3): PortInfo =>
      ({ name, kind, anchor, step: STEP, width: N, ty: "uint" });
    studio.declarePort(port("a_in", "input", aIn));
    studio.declarePort(port("a_out", "output", aOut));
    studio.declarePort(port("b_in", "input", bIn));
    studio.declarePort(port("b_out", "output", bOut));
    const busA = studio.routeBus("a_in", ["a_out"], [{ name: "g0", anchor: [8, 2, 8] as Vec3, step: STEP }]);
    const busB = studio.routeBus("b_in", ["b_out"]);
    log(`crossing demo: ${busA.name}=${studio.busState(busA.name)} ${busB.name}=${studio.busState(busB.name)}`);
    void loadLibrary();
    refresh();
  }

  $("#btn-demo").addEventListener("click", () => void loadAdderDemo());
  $("#btn-demo-crossing").addEventListener("click", loadCrossingDemo);

  await loadLibrary();
  refresh();
  const params = new URLSearchParams(location.search);
  if (params.has("demo")) await loadAdderDemo();
  if (params.has("crossing")) loadCrossingDemo();

  (window as any).__edaReady = true;
  (window as any).__edaStudio = () => studio;
  (window as any).__edaShot = () => viewer.screenshotDataUrl();
  (window as any).__edaDrag = (kind: "instance" | "gate", id: string, ground: Vec3) =>
    applyMove(kind, id, ground, "drop");
  // Headless hooks for the verification script: the same paths the mouse and
  // keyboard drive, so a passing check means the UI itself works.
  (window as any).__eda = {
    select: (id: string) => select({ kind: "instance", id }),
    selection: () => selection,
    mode: () => mode,
    hint: () => $("#hint").textContent,
    endpoints: () => endpoints,
    clickPort: (name: string) => onPortChip(name),
    key: (key: string, shift = false) =>
      window.dispatchEvent(new KeyboardEvent("keydown", { key, shiftKey: shift })),
    place: (cell: string, at: Vec3) => studio.placeInstance(cell, at),
    remesh,
    demo: loadAdderDemo,
    crossing: loadCrossingDemo,
  };
}

boot().catch((err) => {
  $("#status").textContent = `engine failed: ${err}`;
  console.error(err);
});
