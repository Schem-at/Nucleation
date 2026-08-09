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
import { humanReason, busFailureLine, fmtAt, type HumanReason } from "./reasons";

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

/** A stacked, dismissible toast.
 *
 *  The old one was a single node whose text the next message overwrote: a
 *  connect gesture that promoted a port and then failed to route replaced the
 *  first half of its own story before you could read it. Now they stack (newest
 *  at the bottom of the column, oldest evicted past four), each has an ×, an
 *  optional second line naming the FIX, and an optional button that flies the
 *  camera to the coordinate the engine complained about.
 *
 *  `#toast` is the container, so `#toast`.textContent is still every visible
 *  message — which is what the headless verify reads. */
interface ToastOpts {
  kind?: "ok" | "err";
  /** Second line: what to do about it. */
  fix?: string;
  /** Adds a "→ show me" button that focuses this world position. */
  at?: Vec3 | null;
  /** ms; errors linger. */
  ms?: number;
}
let onToastFocus: ((at: Vec3) => void) | null = null;
const MAX_TOASTS = 4;
function toast(msg: string, kindOrOpts: "ok" | "err" | ToastOpts = "ok") {
  const o: ToastOpts = typeof kindOrOpts === "string" ? { kind: kindOrOpts } : kindOrOpts;
  const kind = o.kind ?? "ok";
  const box = $("#toast");
  const item = document.createElement("div");
  item.className = `toast-item${kind === "err" ? " is-err" : ""}`;
  item.innerHTML =
    `<span class="msg">${esc(msg)}${o.fix ? `<span class="fix">${esc(o.fix)}</span>` : ""}</span>` +
    (o.at ? `<button class="go" title="fly the camera there">→ ${esc(fmtAt(o.at))}</button>` : "") +
    `<button class="x" title="dismiss">×</button>`;
  const kill = () => { clearTimeout(timer); item.remove(); };
  item.querySelector(".x")!.addEventListener("click", kill);
  if (o.at) {
    item.querySelector(".go")!.addEventListener("click", () => onToastFocus?.(o.at as Vec3));
  }
  box.appendChild(item);
  while (box.children.length > MAX_TOASTS) box.firstElementChild!.remove();
  const timer = window.setTimeout(kill, o.ms ?? (kind === "err" ? 9000 : 3200));
  return item;
}
/** Every visible toast, for the headless verify. */
const toastTexts = () =>
  [...$("#toast").children].map((c) => c.textContent?.replace(/[×]|→ \(.*?\)/g, "").trim() ?? "");

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

  // ---- first-30-seconds coach -------------------------------------------
  //
  // The app's model is three ideas deep (ports have a DIRECTION, ports have a
  // MODE, buses have a STATE) and none of them are guessable from an empty
  // grid. So the default landing state is a WORKING design — the verified
  // add -> bcd -> 7-seg chain, both buses routed — and this walks through the
  // four things you do to it. Dismissed once, dismissed for good.

  const COACH_KEY = "eda.coach.done";
  const COACH: { title: string; body: string }[] = [
    {
      title: "1/4 · this is a routed design",
      body: "Three community cells (adder → binary-to-BCD → 7-segment) with two " +
        "routed buses between them. Drag to orbit, wheel to zoom, click a cell to select it.",
    },
    {
      title: "2/4 · click a port to start a bus",
      body: "Green ▲ ports drive a bus, blue ▼ ports receive one. Click a green ▲ " +
        "then a blue ▼ and the router builds it. A grey ✗ port is executor-only " +
        "(a lever) — clicking it as a target promotes it for you.",
    },
    {
      title: "3/4 · R rotates, Del deletes",
      body: "With something selected: R / ⇧R rotates 90°, G grabs it, F frames it, " +
        "Del removes it (and names the buses that go with it). Esc always cancels. " +
        "⌘/Ctrl-Z undoes. Press ? for every key.",
    },
    {
      title: "4/4 · Exec/Bus, then Bake and Export",
      body: "Each port chip in the outliner carries a reversible Exec/Bus switch. " +
        "Bake turns the design into a typed executor you can poke (right panel), " +
        "and Export writes .schem / .litematic / .nucm.",
    },
  ];
  let coachStep = 0;
  let coachOpen = false;

  function renderCoach() {
    $("#coach").classList.toggle("is-open", coachOpen);
    if (!coachOpen) return;
    const s = COACH[coachStep];
    $("#coach-title").textContent = s.title;
    $("#coach-body").textContent = s.body;
    $("#coach-dots").innerHTML = COACH.map((_, i) =>
      `<i class="${i === coachStep ? "on" : ""}"></i>`).join("");
    ($("#coach-back") as HTMLButtonElement).disabled = coachStep === 0;
    $("#coach-next").textContent = coachStep === COACH.length - 1 ? "Done" : "Next";
  }
  function coachShow(force = false) {
    if (!force && localStorage.getItem(COACH_KEY) === "1") return;
    coachOpen = true;
    coachStep = 0;
    renderCoach();
  }
  function coachDismiss() {
    coachOpen = false;
    try { localStorage.setItem(COACH_KEY, "1"); } catch { /* private mode */ }
    renderCoach();
  }
  function coachNext(delta = 1) {
    coachStep += delta;
    if (coachStep >= COACH.length) { coachDismiss(); return; }
    if (coachStep < 0) coachStep = 0;
    renderCoach();
  }
  $("#coach-next").addEventListener("click", () => coachNext(1));
  $("#coach-back").addEventListener("click", () => coachNext(-1));
  $("#coach-skip").addEventListener("click", coachDismiss);

  // ---- shortcuts overlay (press ?) --------------------------------------

  const shortcutsOpen = () => $("#shortcuts").classList.contains("is-open");
  function setShortcuts(on: boolean) {
    $("#shortcuts").classList.toggle("is-open", on);
  }
  $("#btn-help").addEventListener("click", () => setShortcuts(!shortcutsOpen()));
  $("#shortcuts-close").addEventListener("click", () => setShortcuts(false));

  // ---- destructive confirm ----------------------------------------------
  //
  // In-app, not `window.confirm`: a native dialog blocks the engine, cannot
  // carry the count, and is auto-dismissed by every headless driver. Enter
  // accepts, Esc cancels, and `window.__eda.confirmRespond` drives it in tests.

  let pendingConfirm: { title: string; body: string; resolve: (ok: boolean) => void } | null = null;
  function confirmDestructive(title: string, body: string, okLabel = "Delete"): Promise<boolean> {
    return new Promise((resolve) => {
      pendingConfirm = { title, body, resolve };
      $("#confirm-title").textContent = title;
      $("#confirm-body").textContent = body;
      $("#confirm-ok").innerHTML = `${esc(okLabel)} <kbd>↵</kbd>`;
      $("#confirm").classList.add("is-open");
    });
  }
  function confirmRespond(ok: boolean) {
    const p = pendingConfirm;
    if (!p) return false;
    pendingConfirm = null;
    $("#confirm").classList.remove("is-open");
    p.resolve(ok);
    return true;
  }
  $("#confirm-ok").addEventListener("click", () => confirmRespond(true));
  $("#confirm-cancel").addEventListener("click", () => confirmRespond(false));

  // ---- empty state -------------------------------------------------------

  function renderEmptyState() {
    const empty = studio.instances.size === 0 && studio.ports.size === 0;
    $("#empty-state").classList.toggle("is-open", empty);
  }
  $("#empty-chain").addEventListener("click", () => void loadChainDemo());
  $("#empty-adder").addEventListener("click", () => void loadAdderDemo());
  $("#empty-help").addEventListener("click", () => setShortcuts(true));

  // ---- undo / redo -------------------------------------------------------

  function renderHistory() {
    const u = $("#btn-undo") as HTMLButtonElement;
    const r = $("#btn-redo") as HTMLButtonElement;
    u.disabled = !studio.canUndo();
    r.disabled = !studio.canRedo();
    u.title = studio.canUndo() ? `Undo ${studio.undoLabel()} (⌘/Ctrl-Z)` : "Nothing to undo";
    r.title = studio.canRedo() ? `Redo ${studio.redoLabel()} (⇧⌘/Ctrl-Z)` : "Nothing to redo";
  }
  function doUndo() {
    const label = studio.undo();
    if (!label) { toast("nothing left to undo"); return; }
    if (selection?.kind === "instance" && !studio.instances.has(selection.id)) select(null);
    log(`undo: ${label}`);
    toast(`undid ${label}`);
    refresh();
  }
  function doRedo() {
    const label = studio.redo();
    if (!label) { toast("nothing to redo"); return; }
    log(`redo: ${label}`);
    toast(`redid ${label}`);
    refresh();
  }
  $("#btn-undo").addEventListener("click", doUndo);
  $("#btn-redo").addEventListener("click", doRedo);

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
        `<kbd>R</kbd> rotate 90° &nbsp; <kbd>G</kbd> move &nbsp; <kbd>F</kbd> frame &nbsp; ` +
        `<kbd>Del</kbd> remove &nbsp; <kbd>Esc</kbd> deselect &nbsp;·&nbsp; ${nav} &nbsp;·&nbsp; ` +
        `<kbd>?</kbd> keys`;
      return;
    }
    const driver = endpoints.some((e) => e.routable && e.kind === "input");
    el.innerHTML = driver
      ? `<b>click a green ▲ output port</b> to start a bus, or click a component to select it` +
        ` &nbsp;·&nbsp; ${nav} &nbsp;·&nbsp; <kbd>?</kbd> keys`
      : `<b>Chain demo</b>, or click a cell in the Library and then the ground` +
        ` &nbsp;·&nbsp; ${nav} &nbsp;·&nbsp; <kbd>?</kbd> keys`;
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
      handleGroundClick(ground);
    },
  });

  /** A click on empty ground. Placing puts the cell down, grabbing drops it,
   *  connecting cancels — and otherwise it DESELECTS, which is the escape
   *  hatch people reach for before they find Esc. */
  function handleGroundClick(ground: Vec3) {
    if (mode.kind === "placing") {
      const cell = mode.cell;
      setMode({ kind: "idle" });
      try {
        const inst = studio.placeInstance(cell, ground);
        log(`placed ${inst.name} (${cell}) at ${ground.join(",")}`);
        select({ kind: "instance", id: inst.name });
        toast(`${inst.name} placed`, { fix: "R rotates · G moves · Del removes · its ports are on the canvas" });
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
  }

  // A toast's "→ (x,y,z)" button, and every click-to-focus row in the
  // outliner, go through the one camera move.
  onToastFocus = (at: Vec3) => viewer.focusOn(at);

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
        //
        // `coalesce` on the live frames is what makes a drag ONE undo step:
        // sixty committed moves collapse into "where the gesture started" ->
        // "where it ended".
        const report = timed("dragFrame", () =>
          studio.moveInstance(id, ground, undefined, phase === "live"));
        if (phase === "drop") {
          studio.endGesture();
          reportBuses(`move ${id} -> ${ground.join(",")}`, report);
        }
      } else {
        const [busName, gateName] = id.split(" ");
        const bus = studio.buses.get(busName);
        const gate = bus?.gates.find((g) => g.name === gateName);
        if (!gate) return;
        const report = studio.moveGate(busName, gateName, [ground[0], gate.anchor[1], ground[2]]);
        if (phase === "drop") {
          studio.endGesture();
          log(`gate ${id} -> ${ground[0]},${gate.anchor[1]},${ground[2]}  state=${report.state} segments=${report.rerouted_segments}`);
          if (report.state.startsWith("failed")) failToast(busName, report.state.replace(/^failed:?\s*/, ""));
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

  /** One failed bus, as a sentence: what happened, what to do, and a button
   *  that flies the camera to the coordinate the router named. */
  function failToast(bus: string, reason: string) {
    const h = humanReason(reason);
    lastFailure = { bus, ...h };
    toast(`Bus ${bus} failed: ${h.headline}`, { kind: "err", fix: h.fix, at: h.at });
    return h;
  }
  /** The last failure the UI reported, for the outliner and the verify. */
  let lastFailure: ({ bus: string } & HumanReason) | null = null;

  function reportBuses(what: string, report: { rerouted: string[]; failed: Record<string, string>; removed_buses?: string[] }) {
    const failed = Object.entries(report.failed ?? {});
    const removed = report.removed_buses ?? [];
    log(`${what}` +
      (removed.length ? `  removed=[${removed}]` : "") +
      `  rerouted=[${report.rerouted ?? []}]` +
      (failed.length ? `  FAILED: ${failed.map(([b, r]) => `${b}: ${r}`).join("; ")}` : ""));
    for (const [bus, reason] of failed.slice(0, 2)) failToast(bus, reason);
    if (failed.length > 2) toast(`${failed.length - 2} more bus(es) failed — see the Buses panel`, "err");
    if (!failed.length && removed.length) {
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
    const h = humanReason(p.blocked);
    toast(`${name}: ${h.headline}`, {
      kind: "err",
      fix: p.instance
        ? "and it cannot be promoted here — drive it by hand after Bake, or move the cell so a form adapter fits"
        : h.fix,
      at: h.at,
    });
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
    renderHistory();
    renderEmptyState();
    setHint();
  }
  studio.onChange = refresh;

  // ---- keyboard ---------------------------------------------------------

  /** Delete an instance, confirming first if buses die with it — with the
   *  COUNT in the prompt, because "3 buses will be ripped" is the whole
   *  reason to ask. */
  async function deleteInstance(name: string) {
    const carried = studio.busesOn(name);
    if (carried.length) {
      const ok = await confirmDestructive(
        `Delete ${name} and rip ${carried.length} bus${carried.length === 1 ? "" : "es"}?`,
        `${name} carries ${carried.length} routed bus${carried.length === 1 ? "" : "es"}: ` +
        `${carried.join(", ")}. Deleting it removes ${carried.length === 1 ? "that bus" : "them"} too. ` +
        `Undo (⌘/Ctrl-Z) puts everything back.`,
        `Delete ${name}`,
      );
      if (!ok) { toast(`kept ${name}`); return; }
    }
    try {
      const report = studio.removeInstance(name);
      if (selection?.kind === "instance" && selection.id === name) select(null);
      reportBuses(`deleted ${name}`, report);
      toast(`${name} deleted` +
        (report.removed_buses.length ? ` — ripped ${report.removed_buses.join(", ")}` : "") +
        " · ⌘/Ctrl-Z to undo");
    } catch (err) {
      toast(String(err), "err");
    }
  }

  window.addEventListener("keydown", (e) => {
    const t = e.target as HTMLElement;
    if (t && /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName)) return;
    const key = e.key.toLowerCase();
    // Esc unwinds ONE layer at a time, outermost first: a modal, then an
    // overlay, then the gesture, then the selection. Never a dead key.
    if (e.key === "Escape") {
      if (pendingConfirm) { confirmRespond(false); return; }
      if (shortcutsOpen()) { setShortcuts(false); return; }
      if (coachOpen) { coachDismiss(); return; }
      if (mode.kind === "grabbing") {
        applyMove("instance", mode.instance, mode.origin, "drop");
      }
      setMode({ kind: "idle" });
      select(null);
      return;
    }
    if (pendingConfirm) {
      if (e.key === "Enter") { e.preventDefault(); confirmRespond(true); }
      return;
    }
    if (e.key === "?" || (key === "/" && e.shiftKey)) {
      e.preventDefault();
      setShortcuts(!shortcutsOpen());
      return;
    }
    if (key === "z" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      if (e.shiftKey) doRedo(); else doUndo();
      return;
    }
    if (key === "y" && e.ctrlKey) { e.preventDefault(); doRedo(); return; }
    // `A` frames the whole design; so does `F` when nothing is selected.
    if (key === "a" && !e.metaKey && !e.ctrlKey) {
      e.preventDefault();
      if (viewer.frameAll()) toast("framed the whole design");
      return;
    }
    if (!selection || selection.kind !== "instance") {
      if (key === "f") {
        e.preventDefault();
        if (viewer.frameAll()) toast("framed the whole design");
      }
      return;
    }
    const name = selection.id;
    if (key === "r") {
      e.preventDefault();
      try {
        const report = studio.rotateInstance(name, e.shiftKey ? -90 : 90);
        studio.endGesture();
        reportBuses(`rotate ${name} -> ${report.rot}°`, report);
        toast(`${name} rotated to ${report.rot}°`);
      } catch (err) {
        toast(String(err), "err");
      }
      return;
    }
    if (key === "f") {
      e.preventDefault();
      if (viewer.focusInstance(name)) toast(`framed ${name}`);
      return;
    }
    if (e.key === "Delete" || e.key === "Backspace") {
      e.preventDefault();
      void deleteInstance(name);
      return;
    }
    if (key === "g") {
      e.preventDefault();
      const inst = studio.instances.get(name);
      if (inst) setMode({ kind: "grabbing", instance: name, origin: [...inst.at] as Vec3 });
    }
  });

  // ---- outliner panels --------------------------------------------------

  /** A port chip: `▲ name : type` plus its Exec/Bus mode badge. */
  function portChip(p: (typeof endpoints)[number], withMode: boolean): string {
    const dir = p.kind === "input" ? "drives" : "receives";
    const arrow = p.kind === "input" ? "▲" : "▼";
    const title = p.routable
      ? `${p.name} · ${p.kind === "input" ? "drives a bus — click to start one" : "receives a bus — click to finish one"}` +
        ` · ${p.ty}[${p.width}] @ ${p.anchor.join(",")}`
      : humanReason(p.blocked).headline;
    return `<span class="port-row">
      <button class="port-chip ${dir}${p.routable ? "" : " blocked"}"
              data-port="${esc(p.name)}" title="${esc(title)}">
        ${arrow} ${esc(p.port ?? p.name.split(".").pop())} <span class="pty">: ${esc(p.ty)}</span>${p.routable ? "" : " ✗"}
      </button>${withMode ? modeToggle(p) : ""}</span>`;
  }

  function renderPanels() {
    // Counts in every section header, so the panel says how big the design is
    // before you scroll it.
    $("#cell-count").textContent = `${studio.cells.size}`;
    $("#instance-count").textContent =
      `${studio.instances.size} in ${new Set([...studio.instances.values()].map((i) => i.cell)).size} type(s)`;
    $("#port-count").textContent = `${studio.ports.size}`;
    const busStates = [...studio.buses.keys()].map((n) => studio.busStateDetail(n).state);
    $("#bus-count").textContent = busStates.length
      ? `${busStates.length} · ${busStates.filter((s) => s === "routed").length} routed` +
        (busStates.some((s) => s === "failed") ? ` · ${busStates.filter((s) => s === "failed").length} failed` : "")
      : "0";

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

    // (b) INSTANCES, GROUPED BY CELL TYPE
    //
    // Ten placements of one adder used to be ten indistinguishable cards. The
    // grouping is also the mental model the renderer uses (one mesh per cell,
    // N placements), so the panel and the machine now agree.
    const insts = [...studio.instances.values()];
    const byCell = new Map<string, typeof insts>();
    for (const i of insts) {
      let list = byCell.get(i.cell);
      if (!list) byCell.set(i.cell, (list = []));
      list.push(i);
    }
    $("#instance-list").innerHTML = [...byCell.entries()].map(([cell, list]) => {
      const head = `<div class="group">
        <span class="gname">${esc(cell)}</span>
        <span class="gcount">× ${list.length}</span>
      </div>`;
      return head + list.map((i) => {
        const mine = endpoints.filter((p) => p.instance === i.name);
        const buses = studio.busesOn(i.name);
        const sel = selection?.kind === "instance" && selection.id === i.name;
        return `<div class="item${sel ? " is-selected" : ""}" data-inst="${esc(i.name)}"
                     title="click to select and frame it">
          <span class="name">${esc(i.name)}</span>
          <span class="meta">@ ${i.at.join(",")} · rot ${i.rot}°${
            buses.length ? ` · ${buses.length} bus${buses.length === 1 ? "" : "es"}` : ""}</span>
          <div class="ports">${mine.map((p) => portChip(p, true)).join("")}</div>
          <div class="row">
            <button data-rot="${esc(i.name)}" title="rotate 90°">R ↻</button>
            <button data-focus-inst="${esc(i.name)}" title="frame it (F)">Frame</button>
            <button data-del="${esc(i.name)}" class="danger"
                    title="${buses.length ? `asks first: ${buses.length} bus(es) go with it` : "delete"}">Del ✕</button>
          </div>
        </div>`;
      }).join("");
    }).join("") || `<span class="meta">none — click a cell in the Library, then click the ground</span>`;
    for (const el of $("#instance-list").querySelectorAll<HTMLElement>("[data-focus-inst]")) {
      el.addEventListener("click", (ev) => {
        ev.stopPropagation();
        viewer.focusInstance(el.dataset.focusInst!);
      });
    }
    for (const el of $("#instance-list").querySelectorAll<HTMLElement>("[data-inst]")) {
      el.addEventListener("click", (ev) => {
        if ((ev.target as HTMLElement).closest("button")) return;
        select({ kind: "instance", id: el.dataset.inst! });
      });
      // Double-click frames it, the way an outliner is expected to behave —
      // single click must NOT move the camera, or every selection is a jump.
      el.addEventListener("dblclick", (ev) => {
        if ((ev.target as HTMLElement).closest("button")) return;
        viewer.focusInstance(el.dataset.inst!);
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
      el.addEventListener("click", (ev) => {
        ev.stopPropagation();
        const r = studio.rotateInstance(el.dataset.rot!);
        studio.endGesture();
        reportBuses(`rotate ${el.dataset.rot} -> ${r.rot}°`, r);
      });
    }
    for (const el of $("#instance-list").querySelectorAll<HTMLElement>("[data-del]")) {
      el.addEventListener("click", (ev) => {
        ev.stopPropagation();
        void deleteInstance(el.dataset.del!);
      });
    }
    for (const el of $("#instance-list").querySelectorAll<HTMLElement>("[data-port]")) {
      el.addEventListener("click", () => viewerPortClick(el.dataset.port!));
    }

    // design ports (declared on loose hardware)
    const declared = endpoints.filter((p) => !p.instance);
    $("#port-list").innerHTML = declared.map((p) => `
      <div class="item" data-focus="${esc(p.anchor.join(","))}" title="click to fly here">
        ${portChip(p, false)}
        <span class="meta where">@ ${p.anchor.join(",")} step ${p.step.join(",")} · ${p.width} bit</span></div>`).join("")
      || `<span class="meta">none — the demos declare typed ports on loose hardware</span>`;
    for (const el of $("#port-list").querySelectorAll<HTMLElement>("[data-port]")) {
      el.addEventListener("click", (ev) => { ev.stopPropagation(); viewerPortClick(el.dataset.port!); });
    }

    // (c) BUSES — endpoints, state, timing, and a FAILED bus you can click
    //     to fly to whatever is in the way.
    $("#bus-list").innerHTML = [...studio.buses.values()].map((b) => {
      const { state, reason } = studio.busStateDetail(b.name);
      const cls = state === "failed" ? "state-failed"
        : state === "routed" ? "state-routed" : "state-intended";
      const h = reason ? humanReason(reason) : null;
      const skew = state === "routed" ? studio.busSkew(b.name) : null;
      // A failed bus focuses the BLOCKAGE; a routed one focuses its driver.
      const focus = h?.at ?? endpoint(b.driver)?.anchor ?? null;
      return `<div class="item"${focus ? ` data-focus="${esc(focus.join(","))}"` : ""}
                   title="${focus ? "click to fly to this bus" : ""}">
        <span class="swatch" style="background:${state === "failed" ? "var(--c-failed)"
          : `#${b.color.toString(16).padStart(6, "0")}`}"
          title="${state === "failed" ? "failed buses draw red" : "this bus's colour on the canvas"}"></span>
        <span class="name">${esc(b.name)}</span> <span class="${cls}">${state}</span>
        ${skew ? `<span class="badge" title="round-trip ticks: worst bit, and the spread across bits">${skew.max_rt}t · skew ${skew.skew_rt}t</span>` : ""}
        <span class="meta">${esc(b.driver)} → ${esc(b.sinks.join(", "))} · ${
          endpoint(b.driver)?.width ?? "?"} bit${b.gates.length ? ` · gates ${esc(b.gates.map((g) => g.name).join(","))}` : ""}</span>
        ${h ? `<span class="meta reason">${esc(h.headline)}</span>` : ""}
        ${h?.fix ? `<span class="meta fix">↳ ${esc(h.fix)}</span>` : ""}
        ${h ? `<details><summary>the engine's own words${h.at ? ` · ${esc(fmtAt(h.at))}` : ""}</summary><div class="raw">${esc(h.detail)}</div></details>` : ""}
        <div class="row">
          <button data-rip="${esc(b.name)}" title="remove its blocks, keep the declaration">Rip</button>
          <button data-reroute="${esc(b.name)}" title="try again from the stored declaration">Re-route</button>
          ${focus ? `<button data-focus-btn="${esc(focus.join(","))}" title="fly to ${esc(fmtAt(focus))}">Focus</button>` : ""}
          <button data-delbus="${esc(b.name)}" class="danger">Delete</button>
        </div>
      </div>`;
    }).join("") || `<span class="meta">click a green ▲ output port then a blue ▼ input port to route one</span>`;
    // Click-to-focus, on the row and on the explicit button.
    for (const el of $("#right").querySelectorAll<HTMLElement>("[data-focus], [data-focus-btn]")) {
      el.addEventListener("click", (ev) => {
        if (el.hasAttribute("data-focus") && (ev.target as HTMLElement).closest("button")) return;
        const raw = el.dataset.focus ?? el.dataset.focusBtn ?? "";
        const at = raw.split(",").map(Number) as Vec3;
        if (at.length === 3 && at.every((n) => Number.isFinite(n))) viewer.focusOn(at);
      });
    }
    for (const el of $("#bus-list").querySelectorAll<HTMLElement>("[data-rip]")) {
      el.addEventListener("click", (ev) => {
        ev.stopPropagation();
        studio.ripBus(el.dataset.rip!);
        log(`ripped ${el.dataset.rip}`);
        toast(`ripped ${el.dataset.rip} — its declaration is kept, press Re-route or ⌘/Ctrl-Z`);
      });
    }
    for (const el of $("#bus-list").querySelectorAll<HTMLElement>("[data-reroute]")) {
      el.addEventListener("click", (ev) => {
        ev.stopPropagation();
        const name = el.dataset.reroute!;
        const state = studio.rerouteBus(name);
        log(`re-routed ${name}: ${state}`);
        if (state.startsWith("failed")) failToast(name, state.replace(/^failed:?\s*/, ""));
        else toast(`${name} ${state}`);
      });
    }
    for (const el of $("#bus-list").querySelectorAll<HTMLElement>("[data-delbus]")) {
      el.addEventListener("click", async (ev) => {
        ev.stopPropagation();
        const name = el.dataset.delbus!;
        const ok = await confirmDestructive(
          `Delete bus ${name}?`,
          `${name} is ${studio.busStateDetail(name).state}. Deleting it drops the declaration ` +
          `as well as the blocks (Rip keeps the declaration). ⌘/Ctrl-Z puts it back.`,
          `Delete ${name}`,
        );
        if (!ok) return;
        studio.removeBus(name);
        log(`deleted bus ${name}`);
        toast(`deleted ${name} · ⌘/Ctrl-Z to undo`);
      });
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
      log(`routed ${bus.name}: ${driver.name} -> ${sink.name} (${state}${reason ? `: ${reason}` : ""})` +
        (promoted.length ? `  [${promoted.join(" | ")}]` : ""));
      lastConnect = { bus: bus.name, state, promoted: [...promoted] };
      if (promoted.length) toast(`promoted ${promoted.join("; ")}`);
      if (state === "failed") failToast(bus.name, reason ?? "");
      else toast(`${bus.name} routed: ${driver.name} → ${sink.name}`);
    } catch (err) {
      const h = humanReason(String(err).replace(/^Error:\s*/, ""));
      toast(h.headline, { kind: "err", fix: h.fix, at: h.at });
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
    studio.clearHistory();
    refresh();
    viewer.frameAll();
    toast("adder demo loaded", { fix: "Bake, then poke u0.a=99 u0.b=28 → sum_out = 127" });
    coachShow();
  }

  /** THE LANDING STATE: the chain `tests/design_promotion.rs` verifies end to
   *  end in the tick engine — ADD007 → BINTOBCD001 → NUMDISPLAY001, both buses
   *  routed, 8/8 BCD values and 8/8 segment patterns exact.
   *
   *  It is the default because it is the only starting point that shows all
   *  three ideas at once: ports have a direction (green drives, blue receives),
   *  ports have a mode (both `bin` and `bcd` are lever banks the connect
   *  gesture promotes), and buses have a state.
   *
   *  The placements are the test's, not guesses. A bus realizes a single-level
   *  2y-pitch stack, so each stage sits at the y that puts its bit-0
   *  connection cell on the previous stage's level:
   *  add at y=0 → bcd at y=-2 → seg at y=1. */
  async function loadChainDemo(): Promise<boolean> {
    const s = core.Schematic.create("add-bcd-7seg");
    studio = new Studio(core, d, "add-bcd-7seg", s);
    studio.onChange = refresh;
    await loadLibrary();
    const find = (p: string) => [...studio.cells.keys()].find((n) => n.startsWith(p));
    const add = find("ADD007"), bcd = find("BINTOBCD001"), seg = find("NUMDISPLAY001");
    if (!add || !bcd) {
      log("chain demo: the enhanced library is unavailable; falling back to the adder demo");
      await loadAdderDemo();
      return false;
    }
    const t0 = performance.now();
    const u0 = studio.placeInstance(add, [0, 0, 0]);
    const u1 = studio.placeInstance(bcd, [60, -2, 40]);
    const routed: string[] = [];
    const failed: string[] = [];
    const link = (driver: string, sink: string) => {
      try {
        // `sink` is a lever bank: the engine promotes it, exactly as a click
        // on the canvas would.
        const p = studio.allEndpoints().find((e) => e.name === sink);
        if (p && !p.routable && p.instance && p.promotable) {
          studio.setPortMode(p.instance, p.port, "bus");
        }
        const bus = studio.routeBus(driver, [sink]);
        const { state, reason } = studio.busStateDetail(bus.name);
        if (state === "routed") routed.push(`${bus.name} (${driver} → ${sink})`);
        else failed.push(`${bus.name}: ${humanReason(reason).headline}`);
      } catch (err) {
        failed.push(`${driver} → ${sink}: ${String(err).replace(/^Error:\s*/, "").slice(0, 160)}`);
      }
    };
    link(`${u0.name}.sum`, `${u1.name}.bin`);
    if (seg) {
      const p = studio.allEndpoints().find((e) => e.name === `${u1.name}.bcd_ones`);
      if (p && !p.routable && p.promotable) studio.setPortMode(u1.name, "bcd_ones", "bus");
      const u2 = studio.placeInstance(seg, [110, 1, 40]);
      link(`${u1.name}.bcd_ones`, `${u2.name}.bcd`);
    }
    // A demo is a starting point, not an edit you undo your way out of.
    studio.clearHistory();
    refresh();
    viewer.frameAll();
    log(`chain: ${add} → ${bcd}${seg ? ` → ${seg}` : ""} in ${(performance.now() - t0).toFixed(0)}ms`);
    for (const r of routed) log(`  routed ${r}`);
    for (const f of failed) log(`  FAILED ${f}`);
    log("  drive it: Bake, then poke u0.a / u0.b and read the BCD digits + segments.");
    if (failed.length) {
      toast(`${routed.length} bus(es) routed, ${failed.length} failed — see the Buses panel`,
        { kind: "err", fix: "click a failed bus to fly to whatever is in the way" });
    } else {
      toast(`chain loaded: ${routed.length} buses routed through ${studio.instances.size} cells`,
        { fix: "Bake, then poke u0.a=99 u0.b=28" });
    }
    coachShow();
    return failed.length === 0;
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
    studio.clearHistory();
    void loadLibrary();
    refresh();
    viewer.frameAll();
    coachShow();
  }

  $("#btn-demo").addEventListener("click", () => void loadAdderDemo());
  $("#btn-demo-chain").addEventListener("click", () => void loadChainDemo());
  $("#btn-demo-crossing").addEventListener("click", loadCrossingDemo);

  await loadLibrary();
  refresh();
  // Landing state. A blank grid teaches nothing, so the DEFAULT is the
  // verified chain; `?empty=1` opts out (the instancing benchmark needs a
  // scene it fully controls), and the other flags pick a specific demo.
  const params = new URLSearchParams(location.search);
  if (params.has("demo")) await loadAdderDemo();
  else if (params.has("crossing")) loadCrossingDemo();
  else if (params.has("chain")) await loadChainDemo();
  else if (params.has("empty")) { renderEmptyState(); coachShow(); }
  else await loadChainDemo();
  if (params.has("coach")) coachShow(true);

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
    /** A click on empty ground, through the same path the pointer drives. */
    groundClick: (at: Vec3 = [0, 0, 0]) => handleGroundClick(at),
    /** Arm cell placement, as clicking a Library row does. */
    arm: (cell: string) => { setMode({ kind: "placing", cell }); },
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
    chain: loadChainDemo,
    crossing: loadCrossingDemo,
    // ---- UX surface, so the polish is checkable and not just claimed ------
    /** Every visible toast, oldest first. */
    toasts: toastTexts,
    /** Coach state: `{open, step, steps, dismissed}`. */
    coach: () => ({
      open: coachOpen, step: coachStep, steps: COACH.length,
      title: $("#coach-title").textContent, body: $("#coach-body").textContent,
      dismissed: localStorage.getItem(COACH_KEY) === "1",
    }),
    coachShow: (force = true) => { coachShow(force); },
    coachNext: (d = 1) => coachNext(d),
    coachDismiss,
    /** Is the empty state visible? */
    emptyState: () => $("#empty-state").classList.contains("is-open"),
    /** The `?` overlay. */
    shortcuts: () => shortcutsOpen(),
    /** The pending destructive confirm, `null` when none is up. */
    pendingConfirm: () => pendingConfirm && { title: pendingConfirm.title, body: pendingConfirm.body },
    confirmRespond,
    /** Undo/redo, and what they would do. */
    history: () => ({
      canUndo: studio.canUndo(), canRedo: studio.canRedo(),
      undo: studio.undoLabel(), redo: studio.redoLabel(),
    }),
    undo: doUndo,
    redo: doRedo,
    /** Buses that terminate on an instance — what a delete would rip. */
    busesOn: (name: string) => studio.busesOn(name),
    /** The last bus failure as the UI phrased it, plus its focus target. */
    lastFailure: () => lastFailure,
    /** The reason translator, so its parsing is testable on real strings. */
    humanReason,
    busFailureLine,
    /** Label declutter: counts and the thresholds they are measured against. */
    labels: () => ({ ...viewer.labelStats, thresholds: { ...Viewer.LABELS } }),
    /** Where the camera was last sent by a click-to-focus. */
    focus: () => viewer.lastFocus,
    focusOn: (at: Vec3) => viewer.focusOn(at),
    /** Orbit distance — what the label thresholds are a function of. */
    frameAll: () => viewer.frameAll(),
    zoom: (radius: number) => { viewer.setOrbitRadius(radius); return viewer.orbitRadius(); },
    /** Per-bus timing, when the engine reports it. */
    busSkew: (name: string) => studio.busSkew(name),
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
