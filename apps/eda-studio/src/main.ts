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
  /** Endpoints as of the last refresh, for the connect flow and panels. */
  let endpoints: ReturnType<Studio["allEndpoints"]> = [];

  const endpoint = (name: string) => endpoints.find((e) => e.name === name);

  /** Wall-clock accounting per named phase, `{n, ms}` — the numbers
   *  `scripts/profile.mjs` reports, and the only way to tell "the scene got
   *  faster" from "the scene got smaller". */
  const timings: Record<string, { n: number; ms: number }> = {};
  function timed<T>(label: string, fn: () => T): T {
    const t0 = performance.now();
    try {
      return fn();
    } finally {
      const e = (timings[label] ??= { n: 0, ms: 0 });
      e.n++;
      e.ms += performance.now() - t0;
    }
  }

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
    // While an interaction owns the pointer, the camera must not: dragging to
    // aim a bus (or to place/move a component) was also orbiting the scene.
    viewer.setCameraLock(
      next.kind === "connecting" ? "drawing a bus"
      : next.kind === "grabbing" ? "moving a component"
      : next.kind === "placing" ? "placing a component"
      : null
    );
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
        offerPromotion(p, name);
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
      dragMove(kind, id, ground);
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
      if (p) dragMove("instance", mode.instance, [Math.round(p.x), inst.at[1], Math.round(p.z)]);
    }
  });

  /** One drag frame.
   *
   *  The instance moves on the GPU FIRST — one matrix per (cell, colour) group,
   *  no engine call — so the gesture tracks the cursor at frame rate whatever
   *  the router is doing. The document is then committed at most once per
   *  animation frame. That replaces the old fixed 250 ms live-reroute throttle:
   *  there is no idle wait any more, the engine simply runs as often as it can
   *  finish, and the drag never waits for it.
   */
  let pendingLive: { kind: "instance" | "gate"; id: string; ground: Vec3 } | null = null;
  let liveScheduled = false;
  function dragMove(kind: "instance" | "gate", id: string, ground: Vec3) {
    if (kind === "instance") viewer.previewInstance(id, ground);
    if (!($("#live-reroute") as HTMLInputElement).checked) return;
    pendingLive = { kind, id, ground };
    if (liveScheduled) return;
    liveScheduled = true;
    requestAnimationFrame(() => {
      liveScheduled = false;
      const p = pendingLive;
      pendingLive = null;
      if (p) applyMove(p.kind, p.id, p.ground, "live");
    });
  }

  function applyMove(kind: "instance" | "gate", id: string, ground: Vec3, phase: "live" | "drop") {
    refreshPhase = phase;
    try {
      if (kind === "instance") {
        // Includes the refresh() the document's onChange fires: this IS the
        // whole cost of one drag frame.
        const report = timed("dragFrame", () => studio.moveInstance(id, ground));
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
    } finally {
      refreshPhase = "drop";
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

  /** "8 lever + 8 stone → 8 repeater + 8 stone": what a promotion actually did
   *  to the cell, from the engine's own per-cell change list. `from`-only
   *  entries were removed, `to`-only entries were added, both means replaced. */
  function promotionSummary(port: string, report: any): string {
    const tally = (xs: string[]) => {
      const m = new Map<string, number>();
      for (const s of xs) {
        const k = String(s).split("[")[0].replace("minecraft:", "");
        m.set(k, (m.get(k) ?? 0) + 1);
      }
      return [...m].map(([k, n]) => `${n} ${k}`).join(" + ");
    };
    const changed = (report?.changed ?? []) as { from?: string | null; to?: string | null }[];
    const before = tally(changed.filter((c) => c.from).map((c) => c.from as string));
    const after = tally(changed.filter((c) => c.to).map((c) => c.to as string));
    return `${port}: ${before || "nothing"} → ${after || "nothing"}`;
  }

  /** A two-state Executor/Bus switch on every instance port that has one.
   *
   *  Community cells name executor hardware — inputs are LEVERS, and nothing in
   *  redstone drives a lever — so a greyed ✗ port is not a dead end, it is a
   *  port waiting to be promoted. The switch is the affordance because the
   *  conversion is REVERSIBLE: Bus mode swaps the lever for a dust input, and
   *  Executor mode puts the shipped hardware back byte-exactly. */
  function modeToggle(p: (typeof endpoints)[number]): string {
    if (!p.instance || !p.promotable) return "";
    const bus = p.mode === "bus";
    return `<button class="mode-toggle${bus ? " is-bus" : ""}"
      data-mode="${esc(p.instance)}|${esc(p.port)}"
      title="${bus
        ? "Bus mode: a bus can land here. Switch back to drive it by hand."
        : "Executor mode: hand-drivable hardware. Switch to Bus so a bus can land here."}"
      >${bus ? "Bus" : "Exec"}</button>`;
  }

  /** Refusing a click on an executor-only port used to be the end of the
   *  conversation. Name the fix instead — and if the port can be promoted,
   *  offer it right here so the user does not have to find the outliner. */
  function offerPromotion(p: (typeof endpoints)[number], name: string) {
    if (p.instance && p.promotable && p.mode !== "bus") {
      toast(
        `${name} is executor-only IO (levers) — switching it to Bus mode so a bus can land on it`,
      );
      togglePortMode(p.instance, p.port);
      // Promotion moved the port; if it is now a driver, carry straight on
      // into the connect gesture rather than making the user click again.
      const now = endpoint(name);
      if (now?.routable && now.kind === "input") {
        setMode({ kind: "connecting", from: name });
        toast(`${name} promoted and armed — now click an input port`);
      }
      return;
    }
    toast(
      `${name} is executor-only IO — no bus can land on it. ${p.blocked ?? ""}` +
      (p.instance ? " (and it cannot be promoted — see the log)" : ""),
      "err",
    );
  }

  function togglePortMode(instance: string, port: string) {
    try {
      const before = studio.portMode(instance, port);
      const report = studio.togglePortMode(instance, port);
      const moved = (report.changed ?? []).length;
      const first = report.changed?.[0];
      if (before === "executor") {
        const where = report.patch?.wires?.[0];
        toast(
          `${instance}.${port} → Bus: removed ${first?.from?.split("[")[0] ?? "hardware"}` +
          ` at (${first?.at?.join(",")})` +
          (where ? `; ${port}[0] now lands on dust at (${where.join(",")})` : "") +
          (report.patch?.pivoted ? " · form adapter added" : "")
        );
      } else {
        toast(`${instance}.${port} → Executor: ${moved} cell(s) restored as shipped`);
      }
      log(`port mode: ${report.note}`);
      if (report.removed_buses?.length) {
        toast(`ripped with it: ${report.removed_buses.join(", ")}`, "err");
      }
      // Its geometry changed, so this cell's mesh is stale — nothing else is.
      viewer.invalidateCell(studio.instances.get(instance)?.cell ?? "");
      if (mode.kind === "connecting") setMode({ kind: "idle" });
      refresh();
      renderPanels();
    } catch (err) {
      toast(String(err), "err");
    }
  }

  /** `"live"` during a drag gesture: the blocks and the buses still update (the
   *  engine has already re-routed them), but the endpoint list, the marker
   *  layer and the outliner do not — nothing mid-gesture reads them, and
   *  together they were ~20 ms a frame. The drop refresh reconciles all three. */
  let refreshPhase: "live" | "drop" = "drop";

  function refresh() {
    timed(refreshPhase === "live" ? "refreshLive" : "refresh", () => refreshInner());
  }
  function refreshInner() {
    if (refreshPhase === "live") {
      try {
        const model = timed("studio.scene", () => studio.scene());
        timed("viewer.setScene", () => viewer.setScene(model));
      } catch (err) {
        log(`layer render failed: ${err}`);
      }
      return;
    }
    endpoints = timed("allEndpoints", () => studio.allEndpoints());
    try {
      const model = timed("studio.scene", () => studio.scene());
      timed("viewer.setScene", () => viewer.setScene(model));
    } catch (err) {
      log(`layer render failed: ${err}`);
    }
    // A drag changes an instance's TRANSFORM, never a cell's blocks, so the
    // textured mesh is still valid — re-meshing mid-gesture was the stutter.
    // Blocks change on a port-mode toggle (tracked as a stale cell) or a
    // reroute, and the drop below re-meshes once.
    if (viewer.isTextured() && mode.kind !== "grabbing" && !viewer.isDragging()) void remesh();
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
    timed("viewer.setMarkers", () => viewer.setMarkers(ports, gates, instances));
    if (selection?.kind === "instance" && !studio.instances.has(selection.id)) selection = null;
    viewer.setSelection(selection);
    timed("renderPanels", () => renderPanels());
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
          <span class="port-row">
          <button class="port-chip ${p.kind === "input" ? "drives" : "receives"}${p.routable ? "" : " blocked"}"
                  data-port="${esc(p.name)}"
                  title="${p.routable ? "click to start/finish a bus" : esc(p.blocked ?? "not routable")}">
            ${p.kind === "input" ? "▲" : "▼"} ${esc(p.port ?? p.name.split(".").pop())} : ${esc(p.ty)}${p.routable ? "" : " ✗"}
          </button>${modeToggle(p)}</span>`).join("")}</div>
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
    for (const el of $("#instance-list").querySelectorAll<HTMLElement>("[data-mode]")) {
      el.addEventListener("click", (ev) => {
        ev.stopPropagation();
        const [instance, port] = el.dataset.mode!.split("|");
        togglePortMode(instance, port);
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
      offerPromotion(p, name);
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

  /** Connect two endpoints, PROMOTING whatever has to be promoted.
   *
   *  The old flow refused the click and told you to press a toggle first, which
   *  is the thing everyone tripped over: an executor-only port is not a dead
   *  end, it is a port that has not been converted yet. So a connect gesture
   *  converts it — reporting exactly what it changed — and only refuses the
   *  ports that genuinely cannot be converted (a ceiling lever, a button),
   *  with the engine's own reason. The explicit Exec/Bus toggle stays for
   *  manual control and for converting back. */
  function connectPorts(a: string, b: string) {
    let pa = endpoint(a), pb = endpoint(b);
    if (!pa || !pb) return;
    const driverName = pa.kind === "input" ? a : b;
    const sinkName = pa.kind === "input" ? b : a;
    if (endpoint(driverName)!.kind !== "input" || endpoint(sinkName)!.kind !== "output") {
      toast("a bus needs one driver (green ▲) and one sink (blue ▼)", "err");
      return;
    }

    // (1) auto-promote either end that needs it, before any geometry is read:
    //     promotion MOVES the port (lever -> a routable cell in the port's own
    //     orientation), so width and type must be re-read afterwards.
    const promoted: string[] = [];
    for (const name of [driverName, sinkName]) {
      const p = endpoint(name);
      if (!p || p.routable) continue;
      if (!p.instance || !p.promotable) {
        toast(`${p.name} is executor-only IO and cannot be promoted: ` +
          `${p.blocked ?? "no dust connection cell"}`, "err");
        log(`route refused: ${p.name} — ${p.blocked}`);
        return;
      }
      try {
        const rep = studio.setPortMode(p.instance, p.port, "bus");
        // The cell's blocks changed, so exactly this cell's mesh is stale.
        viewer.invalidateCell(studio.instances.get(p.instance)?.cell ?? "");
        promoted.push(promotionSummary(p.name, rep));
        log(`auto-promoted ${p.name}: ${rep.note}`);
        if (rep.removed_buses?.length) log(`  ripped with it: ${rep.removed_buses.join(", ")}`);
      } catch (err) {
        toast(`${p.name} could not be promoted: ${String(err).replace(/^Error:\s*/, "")}`, "err");
        return;
      }
      refresh();
    }

    pa = endpoint(driverName); pb = endpoint(sinkName);
    if (!pa || !pb) return;
    const driver = pa, sink = pb;
    if (!driver.routable || !sink.routable) {
      const bad = !driver.routable ? driver : sink;
      toast(`${bad.name} is executor-only IO: ${bad.blocked ?? "no dust connection cell"}`, "err");
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
      // A core that promotes inside route_bus reports it here; either way the
      // summary names every conversion the one click caused.
      for (const p of (bus.promotions ?? []) as any[]) {
        promoted.push(typeof p === "string" ? p : (p?.note ?? JSON.stringify(p)));
      }
      const { state, reason } = studio.busStateDetail(bus.name);
      const promo = promoted.length ? `promoted ${promoted.join("; ")}; ` : "";
      log(`routed ${bus.name}: ${driver.name} -> ${sink.name} (${state}${reason ? `: ${reason}` : ""})` +
        (promoted.length ? `  [${promoted.join(" | ")}]` : ""));
      lastConnect = { bus: bus.name, state, promoted: [...promoted] };
      if (state === "failed") toast(`${promo}${bus.name} FAILED: ${reason}`, "err");
      else toast(`${promo}${bus.name} routed: ${driver.name} → ${sink.name}`);
    } catch (err) {
      toast(String(err).replace(/^Error:\s*/, ""), "err");
      log(`route failed: ${err}`);
    }
  }

  /** The last connect gesture's outcome, for the headless verify. */
  let lastConnect: { bus: string; state: string; promoted: string[] } | null = null;

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
      viewer.meshBuilds.texture++;
      viewer.clearStale();
      await viewer.setTexturedGlb(glb); // bus layers stay abstract on top
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
  /** One drag FRAME, exactly as the pointer handler drives it. */
  (window as any).__edaDragMove = (kind: "instance" | "gate", id: string, ground: Vec3) =>
    dragMove(kind, id, ground);
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
    /** Is the camera free to orbit? Must be false while connecting/grabbing. */
    cameraFree: () => viewer.cameraFree,
    cameraLock: () => viewer.cameraLockReason,
    /** Cumulative mesh builds, so a test can assert a drag causes none. */
    meshBuilds: () => ({ ...viewer.meshBuilds }),
    staleCells: () => viewer.stale(),
    portMode: (instance: string, port: string) => studio.portMode(instance, port),
    togglePortMode,
    setPortMode: (instance: string, port: string, m: "bus" | "executor") => {
      const r = studio.setPortMode(instance, port, m);
      refresh();
      renderPanels();
      return r;
    },
    remesh,
    demo: loadAdderDemo,
    crossing: loadCrossingDemo,
    /** `{bus, state, promoted}` from the last connect gesture — how the verify
     *  script checks that a connect AUTO-PROMOTED and said so. */
    lastConnect: () => lastConnect,
    /** Draw calls, mesh builds and the wasm round-trips the scene made. */
    sceneReads: () => ({ ...studio.sceneReads }),
    sceneMs: () => ({ ...studio.sceneMs }),
    scene: () => studio.scene(),
    // ---- profiling hooks (scripts/profile.mjs) ----------------------------
    profile: () => viewer.profile(),
    profileReset: () => viewer.profileReset(),
    timings: () => ({ ...timings }),
    timingsReset: () => { for (const k of Object.keys(timings)) delete timings[k]; },
    /** Rebuild the scene from scratch, as a state change would. */
    refresh,
  };
}

boot().catch((err) => {
  $("#status").textContent = `engine failed: ${err}`;
  console.error(err);
});
