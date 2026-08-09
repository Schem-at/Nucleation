/** Idiomatic veneer over the generated `Design` / `CellExecutor` core.
 *
 * Thin by contract: every method here only marshals arrays, object
 * literals and options into the exact JSON strings and positional splats
 * the generated bridge expects, and parses the JSON it writes back. No
 * routing/design logic lives on this side. This module mirrors
 * `bindings/python/nucleation/design.py` 1:1 (same names in camelCase,
 * same shapes; object literals where Python takes dataclasses/kwargs).
 *
 * Import as `nucleation/design`, passing the loaded core module once:
 *
 *   import * as core from "nucleation";        // or the runtime engine URL
 *   import { veneer } from "nucleation/design";
 *   const { Design, Gate, Style, Executor } = veneer(core);
 *
 * `veneer(core)` exists because the wasm core is loaded at RUNTIME by URL
 * in the browser apps; there is no static import this module could take.
 */

// --------------------------------------------------------------------------
// small value objects (plain objects accepted anywhere these are)
// --------------------------------------------------------------------------

/** A bus gate: `Gate([x, y, z], [sx, sy, sz], name?)`.
 *  Unnamed gates are auto-named `g0`, `g1`, ... in declaration order. */
export function Gate(anchor, step, name = null) {
  return { anchor: [...anchor], step: [...step], name };
}

/** Per-bus routing style; omitted fields fall back to router defaults. */
export function Style({ busBlock = null, transparentBlock = null, ...rest } = {}) {
  // Python spells the wire keys snake_case; accept both spellings.
  const s = { ...rest };
  if (busBlock != null) s.bus_block = busBlock;
  if (transparentBlock != null) s.transparent_block = transparentBlock;
  return s;
}

function styleJson(style) {
  if (style == null) return "{}";
  if (typeof style === "string") return style; // already wire-format JSON
  if (typeof style === "object") return JSON.stringify(style);
  throw new TypeError("style must be a Style/object, JSON string, null or undefined");
}

/** -> [wire JSON, [gate names]] with `g<i>` auto-naming. */
function gatesJson(gates) {
  const items = (gates ?? []).map((g, i) => {
    if (typeof g !== "object" || !g.anchor || !g.step) {
      throw new TypeError("gates must be Gate(...) results or {anchor, step, name?} objects");
    }
    return { name: g.name ?? `g${i}`, anchor: [...g.anchor], step: [...g.step] };
  });
  return [JSON.stringify(items), items.map((g) => g.name)];
}

function namesJson(names) {
  return JSON.stringify(typeof names === "string" ? [names] : [...names]);
}

function unwrap(schematic) {
  return schematic instanceof FlatBase ? schematic.raw : schematic;
}

function b64ToBytes(b64) {
  if (typeof Buffer !== "undefined") return Buffer.from(b64, "base64");
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

// --------------------------------------------------------------------------
// reports
// --------------------------------------------------------------------------

/** Parsed `d.check()`: `.clean` plus the DRC/LVS/rule sections. */
export class CheckReport {
  constructor(raw) {
    this.raw = raw;
  }
  get clean() { return this.raw.clean ?? false; }
  get drc() { return this.raw.drc ?? []; }
  get lvs() { return this.raw.lvs ?? {}; }
  get rules() { return this.raw.rules ?? []; }
  get buses() { return this.raw.buses ?? {}; }
  get skew() { return this.raw.skew ?? this.raw.buses ?? {}; }
  /** `report.get("drc")` — the Python `report["drc"]` spelling. */
  get(key) { return this.raw[key]; }
  toString() {
    return `CheckReport(clean=${this.clean}, drc=${this.drc.length}, rules=${this.rules.length})`;
  }
}

/** Thrown by `d.check({ strict: true })` when the report is not clean. */
export class DesignCheckError extends Error {
  constructor(report) {
    super(`design check failed: ${JSON.stringify(report.raw, null, 2)}`);
    this.name = "DesignCheckError";
    this.report = report;
  }
}

// --------------------------------------------------------------------------
// executor sugar
// --------------------------------------------------------------------------

function toValue(core, v) {
  if (v instanceof core.Value) return v;
  if (typeof v === "boolean") return core.Value.fromBool(v);
  if (typeof v === "number") {
    if (Number.isInteger(v)) {
      return v >= 0 ? core.Value.fromU32(v) : core.Value.fromI32(v);
    }
    return core.Value.fromF32(v);
  }
  if (typeof v === "string") return core.Value.fromString(v);
  throw new TypeError(`cannot convert ${v} to a port Value`);
}

function fromValue(value) {
  for (const conv of ["asU32", "asI32", "asBool", "asF32"]) {
    try {
      return value[conv]();
    } catch {
      /* next */
    }
  }
  return value;
}

/** `const ex = baked.executor(); ex.set("a", 0x55); ex.settle(); ex.get("a_out")`.
 *
 *  `set`/`get` convert plain JS values to/from typed port `Value`s (the JS
 *  spelling of Python's `ex["a"] = 0x55; ex["a_out"]`); the explicit
 *  `setInput` / `readOutput` / `settle` methods stay available, and
 *  everything else is reachable on `.cell` (the core `CellExecutor`).
 */
class ExecutorBase {
  constructor(core, cell) {
    this._core = core;
    this.cell = cell;
  }
  set(name, value) { this.cell.setInput(name, toValue(this._core, value)); }
  get(name) { return fromValue(this.cell.readOutput(name)); }
  setInput(name, value) { this.cell.setInput(name, toValue(this._core, value)); }
  readOutput(name) { return this.cell.readOutput(name); }
  settle(budget = 4000) { return this.cell.settle(budget); }
  reset() { return this.cell.reset(); }
}

/** A flattened/baked artifact: the core `Schematic` plus sugar.
 *
 *  `.executor()` builds a settled typed executor over the embedded
 *  contract; `.toBytes(format)` serializes it (`"schem"`, `"litematic"`,
 *  `"snapshot"`, `"mcstructure"`); `.save(path)` writes it in Node;
 *  `.raw` is the core `Schematic` for APIs that want it.
 */
class FlatBase {
  constructor(core, raw) {
    this._core = core;
    this.raw = raw;
  }
  executor(settle = 4000) {
    return surfaces.get(this._core).Executor.forSchematic(this.raw, settle);
  }
  /** Artifact bytes; `format` from the path suffix in `.save()`. */
  toBytes(format = "schem") {
    const b64 = {
      schem: () => this.raw.toSchematicB64(),
      schematic: () => this.raw.toSchematicB64(),
      litematic: () => this.raw.toLitematicB64(),
      snapshot: () => this.raw.toSnapshotB64(),
      nuc: () => this.raw.toSnapshotB64(),
      mcstructure: () => this.raw.toMcstructureB64(),
    }[format];
    if (!b64) throw new Error(`unknown artifact format ${JSON.stringify(format)}`);
    return b64ToBytes(b64());
  }
  /** Node-only path save (browsers: use `.toBytes()` + a download). */
  async save(path) {
    const fs = await import("node:fs");
    fs.writeFileSync(path, this.toBytes(String(path).split(".").pop()));
  }
}

// --------------------------------------------------------------------------
// bus handle
// --------------------------------------------------------------------------

/** Handle returned by `routeBus`: live `.state`, drag and rip. */
export class Bus {
  constructor(design, name, gateNames) {
    this.design = design;
    this.name = name;
    this.gateNames = [...gateNames];
  }
  get state() { return this.design.busState(this.name); }
  get skew() { return this.design.busSkew(this.name); }
  _gate(gate) { return typeof gate === "number" ? this.gateNames[gate] : gate; }
  /** Drag a gate (by index or name) to `[x, y, z]`; -> reroute report. */
  moveGate(gate, anchor) { return this.design.moveGate(this.name, this._gate(gate), anchor); }
  addGate(anchor, step, name = null) {
    name = name ?? `g${this.gateNames.length}`;
    const state = this.design.addGate(this.name, name, anchor, step);
    this.gateNames.push(name);
    return state;
  }
  rule(rule = {}) { this.design.setBusRule(this.name, rule); }
  rip() { this.design.rip(this.name); }
  toString() { return `Bus(${JSON.stringify(this.name)}, ${this.state})`; }
}

// --------------------------------------------------------------------------
// the design document
// --------------------------------------------------------------------------

/** Options-object/array veneer over the generated wire-format `Design`.
 *
 *  Anything not wrapped here (`setBlock`, `toNucmB64`, ...) is reachable
 *  on `.raw` (the core object) unchanged.
 */
class DesignBase {
  constructor(core, raw) {
    this._core = core;
    this._d = raw;
  }
  get raw() { return this._d; }

  // -- ports ---------------------------------------------------------------

  declareInput(name, { anchor, step, width, ty = "uint" }) {
    const [ax, ay, az] = anchor, [sx, sy, sz] = step;
    this._d.declareInput(name, ax, ay, az, sx, sy, sz, width, ty);
  }
  declareOutput(name, { anchor, step, width, ty = "uint" }) {
    const [ax, ay, az] = anchor, [sx, sy, sz] = step;
    this._d.declareOutput(name, ax, ay, az, sx, sy, sz, width, ty);
  }

  // -- buses ---------------------------------------------------------------

  routeBus(name, { driver, sinks, gates = [], style = null }) {
    const [gj, gateNames] = gatesJson(gates);
    this._d.routeBus(name, driver, namesJson(sinks), gj, styleJson(style));
    return new Bus(this, name, gateNames);
  }
  routeBusOr(name, { drivers, sinks, gates = [], style = null }) {
    const [gj, gateNames] = gatesJson(gates);
    this._d.routeBusOr(name, namesJson(drivers), namesJson(sinks), gj, styleJson(style));
    return new Bus(this, name, gateNames);
  }
  addGate(bus, gate, anchor, step) {
    const [x, y, z] = anchor, [sx, sy, sz] = step;
    return this._d.addGate(bus, gate, x, y, z, sx, sy, sz);
  }
  moveGate(bus, gate, anchor) {
    const [x, y, z] = anchor;
    return JSON.parse(this._d.moveGate(bus, gate, x, y, z));
  }
  setBusRule(bus, rule = {}) { this._d.setBusRule(bus, JSON.stringify(rule)); }
  busState(name) { return this._d.busState(name); }
  busSkew(name) { return JSON.parse(this._d.busSkew(name)); }
  rip(name) { this._d.rip(name); }
  /** Re-realize a ripped/failed bus from its stored declaration; -> state. */
  reroute(name) { return this._d.reroute(name); }
  /** Delete a bus outright — fragment AND declaration, freeing the name. */
  removeBus(name) { this._d.removeBus(name); }

  // -- cells / instances ---------------------------------------------------

  addCell(name, schematic) { return this._d.addCell(name, unwrap(schematic)); }
  place(name, cell, at, rot = 0) {
    const [x, y, z] = at;
    this._d.place(name, cell, x, y, z, rot);
  }
  moveInstance(name, at, rot = 0) {
    const [x, y, z] = at;
    return JSON.parse(this._d.moveInstance(name, x, y, z, rot));
  }
  /** Remove an instance. Buses that terminated on one of its ports are
   *  DELETED (they lost an endpoint) and named in the report; buses that
   *  merely crossed its space are ripped and co-rerouted.
   *  -> `{removed_buses, rerouted, failed}`. */
  removeInstance(name) { return JSON.parse(this._d.removeInstance(name)); }

  // -- instance ports (derived routing endpoints) ---------------------------

  /** Every endpoint the placed instances expose, as
   *  `{name: "u0.sum", instance, port, role, ty, width, hardware, wires,
   *    step, routable, blocked}`.
   *
   *  `name` is exactly what `routeBus` accepts. `role` is the CELL-facing
   *  direction: `"output"` drives a bus, `"input"` receives one. A cell
   *  contract names EXECUTOR hardware (levers/buttons in, lamps out) while a
   *  bus lands on dust, so `wires` carries the derived dust connection cells
   *  — and a port with no dust to tap (a bare lever input) reports
   *  `routable: false` with the reason in `blocked`. */
  instancePorts() { return JSON.parse(this._d.instancePorts()); }

  /** Switch a port between executor hardware and a routable dust input —
   *  `mode` is `"bus"` or `"executor"`.
   *
   *  This is the composability switch. A community cell's inputs are LEVERS,
   *  and nothing in redstone drives a lever, so `add.sum -> bcd.bin` needs
   *  `bin` in `"bus"` mode first. The conversion is a reversible per-instance
   *  patch: `"executor"` restores the shipped blocks byte-exactly.
   *
   *  Returns `{port, mode, note, changed: [{at, from, to}], removed_buses,
   *  moves, patch}`. `note` is a ready-made toast; `changed` is in WORLD
   *  coordinates. A bus that terminated on the port is RIPPED and named in
   *  `removed_buses` — its endpoint physically stopped existing. */
  setPortMode(instance, port, mode) {
    return JSON.parse(this._d.setPortMode(instance, port, mode));
  }

  /** `setPortMode(instance, port, "bus")`. */
  promotePort(instance, port) { return this.setPortMode(instance, port, "bus"); }

  /** Ports whose mode has been switched: `[{name, mode, patch}]`. Anything
   *  absent is in `"executor"` mode. */
  portModes() { return JSON.parse(this._d.portModes()); }

  /** What promoting a port WOULD do, without doing it: `{wires, hardware,
   *  step, removed, added, pivoted, note}`. Throws with the reason when the
   *  port cannot be promoted (a ceiling lever, say). */
  planPortPromotion(instance, port) {
    return JSON.parse(this._d.planPortPromotion(instance, port));
  }

  /** Resolve one endpoint name to the geometry a bus would use:
   *  `{name, anchor, step, width, direction, connectable}`. `direction` is
   *  DESIGN-facing, so `"input"` drives buses. Throws with the reason when
   *  the name is unknown or the port cannot terminate a bus. */
  resolvePort(name) { return JSON.parse(this._d.resolvePort(name)); }

  // -- loose layer (explicit forward; more via `.raw`) ---------------------

  setBlock(x, y, z, block) { this._d.setBlock(x, y, z, block); }

  // -- lifecycle -----------------------------------------------------------

  check({ strict = false } = {}) {
    const report = new CheckReport(JSON.parse(this._d.check()));
    if (strict && !report.clean) throw new DesignCheckError(report);
    return report;
  }
  flatten() { return new (surfaces.get(this._core).Flat)(this._d.flatten()); }
  /** The flattened stack composited into ONE region — what an interchange
   *  artifact wants. `flatten()` keeps `inst:*`/`bus:*` regions (the
   *  renderer needs them); a single-region schematic cannot lose a
   *  named-layer cell to the loose layer's bounding box. */
  flattenComposite() {
    return new (surfaces.get(this._core).Flat)(this._d.flattenComposite());
  }
  bake(budget = 4000) { return new (surfaces.get(this._core).Flat)(this._d.bake(budget)); }

  // -- persistence ---------------------------------------------------------

  toNucmB64() { return this._d.toNucmB64(); }
  toLitematicB64() { return this._d.toLitematicB64(); }
  toSchemB64() { return this._d.toSchemB64(); }
  /** Document/artifact bytes by suffix: `.nucm` -> project tier,
   *  `.litematic` -> layered interchange, anything else -> the flattened
   *  artifact, composited (the browser-side spelling of Python's
   *  `d.save(path)`). */
  toBytes(pathOrSuffix) {
    const suffix = String(pathOrSuffix).split(".").pop();
    if (suffix === "nucm") return b64ToBytes(this._d.toNucmB64());
    if (suffix === "litematic") return b64ToBytes(this._d.toLitematicB64());
    if (suffix === "schem" || suffix === "schematic") return b64ToBytes(this._d.toSchemB64());
    return this.flattenComposite().toBytes(suffix);
  }
  /** Node-only path save, tier dispatched by suffix like Python's. */
  async save(path) {
    const fs = await import("node:fs");
    fs.writeFileSync(path, this.toBytes(path));
  }
}

// --------------------------------------------------------------------------
// binding to a loaded core
// --------------------------------------------------------------------------

const surfaces = new WeakMap(); // module namespaces are frozen; memoize here

/** Bind the veneer to a loaded wasm core module; returns the surface. */
export function veneer(core) {
  const memo = surfaces.get(core);
  if (memo) return memo;

  class Executor extends ExecutorBase {
    constructor(cell) { super(core, cell); }
    static forSchematic(schematic, settle = 4000) {
      const ex = new Executor(core.CellExecutor.forSchematic(unwrap(schematic)));
      if (settle) ex.cell.settle(settle);
      return ex;
    }
  }
  class Flat extends FlatBase {
    constructor(raw) { super(core, raw); }
  }
  class Design extends DesignBase {
    constructor(raw) { super(core, raw); }
    static create(name) { return new Design(core.Design.create(name)); }
    static forSchematic(name, schematic) {
      return new Design(core.Design.forSchematic(name, unwrap(schematic)));
    }
    static fromNucm(data) { return new Design(core.Design.fromNucm(Array.from(data))); }
    static fromLitematic(data) { return new Design(core.Design.fromLitematic(Array.from(data))); }
    static async loadNucm(path) {
      const fs = await import("node:fs");
      return Design.fromNucm(fs.readFileSync(String(path)));
    }
    static async importLitematic(path) {
      const fs = await import("node:fs");
      return Design.fromLitematic(fs.readFileSync(String(path)));
    }
  }

  const surface = {
    Design, Executor, Flat, Bus, CheckReport, DesignCheckError, Gate, Style,
  };
  surfaces.set(core, surface);
  return surface;
}
