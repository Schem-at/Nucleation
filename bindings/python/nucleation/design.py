"""Idiomatic veneer over the generated ``Design`` / ``CellExecutor`` core.

Thin by contract: every method here only marshals tuples, dataclasses and
keywords into the exact JSON strings and positional splats the generated
bridge (``nucleation.core``) expects, and parses the JSON it writes back.
No routing/design logic lives on this side. The JS veneer mirrors this
module 1:1.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Optional

from . import nucleation as _core

__all__ = [
    "Bus",
    "CheckReport",
    "Design",
    "DesignCheckError",
    "Executor",
    "Flat",
    "Gate",
    "Style",
]


# --------------------------------------------------------------------------
# small value objects
# --------------------------------------------------------------------------

@dataclass
class Gate:
    """A bus gate: ``Gate(anchor=(x, y, z), step=(sx, sy, sz))``.

    ``name`` is optional; unnamed gates are auto-named ``g0``, ``g1``, …
    in declaration order. Plain dicts of the same shape are accepted too.
    """

    anchor: tuple
    step: tuple
    name: Optional[str] = None


@dataclass
class Style:
    """Per-bus routing style; ``None`` fields fall back to router defaults."""

    bus_block: Optional[str] = None
    transparent_block: Optional[str] = None


def _style_json(style) -> str:
    if style is None:
        return "{}"
    if isinstance(style, Style):
        return json.dumps({k: v for k, v in vars(style).items() if v is not None})
    if isinstance(style, dict):
        return json.dumps(style)
    if isinstance(style, str):  # already wire-format JSON
        return style
    raise TypeError("style must be a Style, dict, JSON string or None")


def _gates_json(gates):
    """-> (wire JSON, [gate names]) with ``g<i>`` auto-naming."""
    items = []
    for i, g in enumerate(gates):
        if isinstance(g, Gate):
            items.append({"name": g.name or "g%d" % i,
                          "anchor": list(g.anchor), "step": list(g.step)})
        elif isinstance(g, dict):
            items.append({"name": g.get("name") or "g%d" % i,
                          "anchor": list(g["anchor"]), "step": list(g["step"])})
        else:
            raise TypeError("gates must be Gate instances or dicts")
    return json.dumps(items), [g["name"] for g in items]


def _names_json(names) -> str:
    if isinstance(names, str):
        names = [names]
    return json.dumps(list(names))


def _unwrap(schematic):
    return schematic.raw if isinstance(schematic, Flat) else schematic


# --------------------------------------------------------------------------
# reports
# --------------------------------------------------------------------------

class CheckReport:
    """Parsed ``d.check()``: ``.clean`` plus the DRC/LVS/rule sections."""

    def __init__(self, raw: dict):
        self.raw = raw

    clean = property(lambda self: self.raw.get("clean", False))
    drc = property(lambda self: self.raw.get("drc", []))
    lvs = property(lambda self: self.raw.get("lvs", {}))
    rules = property(lambda self: self.raw.get("rules", []))
    buses = property(lambda self: self.raw.get("buses", {}))
    skew = property(lambda self: self.raw.get("skew", self.raw.get("buses", {})))

    def __getitem__(self, key):
        return self.raw[key]

    def __repr__(self):
        return "CheckReport(clean=%r, drc=%d, rules=%d)" % (
            self.clean, len(self.drc), len(self.rules))


class DesignCheckError(RuntimeError):
    """Raised by ``d.check(strict=True)`` when the report is not clean."""

    def __init__(self, report: CheckReport):
        super().__init__("design check failed: %s" % json.dumps(report.raw, indent=2))
        self.report = report


# --------------------------------------------------------------------------
# executor sugar
# --------------------------------------------------------------------------

def _to_value(v):
    if isinstance(v, _core.Value):
        return v
    if isinstance(v, bool):
        return _core.Value.from_bool(v)
    if isinstance(v, int):
        return _core.Value.from_u32(v) if v >= 0 else _core.Value.from_i32(v)
    if isinstance(v, float):
        return _core.Value.from_f32(v)
    raise TypeError("cannot convert %r to a port Value" % (v,))


def _from_value(value):
    for conv in (value.as_u32, value.as_i32, value.as_bool, value.as_f32):
        try:
            return conv()
        except Exception:
            continue
    return value


class Executor:
    """``ex = baked.executor(); ex["a"] = 0x55; ex.settle(); ex["a_out"]``.

    Item access converts plain Python values to/from typed port ``Value``s;
    the explicit ``set_input`` / ``read_output`` / ``settle`` methods stay
    available (and everything else delegates to the core ``CellExecutor``).
    """

    def __init__(self, cell):
        self.cell = cell

    @classmethod
    def for_schematic(cls, schematic, settle: int = 4000) -> "Executor":
        ex = cls(_core.CellExecutor.for_schematic(_unwrap(schematic)))
        if settle:
            ex.cell.settle(settle)
        return ex

    def __setitem__(self, name: str, value):
        self.cell.set_input(name, _to_value(value))

    def __getitem__(self, name: str):
        return _from_value(self.cell.read_output(name))

    def set_input(self, name: str, value):
        self.cell.set_input(name, _to_value(value))

    def read_output(self, name: str):
        return self.cell.read_output(name)

    def settle(self, budget: int = 4000) -> bool:
        return self.cell.settle(budget)

    def __getattr__(self, attr):
        return getattr(self.cell, attr)


class Flat:
    """A flattened/baked artifact: the core ``Schematic`` plus sugar.

    ``.executor()`` builds a settled typed executor over the embedded
    contract; ``.save(path)`` writes the artifact; ``.raw`` is the core
    ``Schematic`` for APIs that want it (everything else delegates).
    """

    def __init__(self, raw):
        self.raw = raw

    def executor(self, settle: int = 4000) -> Executor:
        return Executor.for_schematic(self.raw, settle)

    def save(self, path) -> None:
        self.raw.save_to_file(str(path))

    def __getattr__(self, attr):
        return getattr(self.raw, attr)


# --------------------------------------------------------------------------
# bus handle
# --------------------------------------------------------------------------

class Bus:
    """Handle returned by ``route_bus``: live ``.state``, drag and rip."""

    def __init__(self, design: "Design", name: str, gate_names):
        self.design = design
        self.name = name
        self.gate_names = list(gate_names)

    @property
    def state(self) -> str:
        return self.design.bus_state(self.name)

    @property
    def skew(self) -> dict:
        return self.design.bus_skew(self.name)

    def _gate(self, gate) -> str:
        return self.gate_names[gate] if isinstance(gate, int) else gate

    def move_gate(self, gate, anchor) -> dict:
        """Drag a gate (by index or name) to ``anchor``; -> reroute report."""
        return self.design.move_gate(self.name, self._gate(gate), anchor=anchor)

    def add_gate(self, anchor, step, name: Optional[str] = None) -> str:
        name = name or "g%d" % len(self.gate_names)
        state = self.design.add_gate(self.name, name, anchor=anchor, step=step)
        self.gate_names.append(name)
        return state

    def rule(self, rule: Optional[dict] = None, **fields) -> None:
        self.design.set_bus_rule(self.name, rule, **fields)

    def rip(self) -> None:
        self.design.rip(self.name)

    def __repr__(self):
        return "Bus(%r, %s)" % (self.name, self.state)


# --------------------------------------------------------------------------
# the design document
# --------------------------------------------------------------------------

class Design:
    """Keyword/tuple veneer over the generated wire-format ``Design``.

    Anything not wrapped here (``set_block``, ``to_nucm_b64``, …) delegates
    to the core object unchanged; ``.raw`` exposes it directly.
    """

    def __init__(self, raw):
        self._d = raw

    raw = property(lambda self: self._d)

    # -- constructors ------------------------------------------------------

    @classmethod
    def create(cls, name: str) -> "Design":
        return cls(_core.Design.create(name))

    @classmethod
    def for_schematic(cls, name: str, schematic) -> "Design":
        return cls(_core.Design.for_schematic(name, _unwrap(schematic)))

    @classmethod
    def from_nucm(cls, data: bytes) -> "Design":
        return cls(_core.Design.from_nucm(data))

    @classmethod
    def load_nucm(cls, path) -> "Design":
        return cls(_core.Design.load_nucm(str(path)))

    @classmethod
    def from_litematic(cls, data: bytes) -> "Design":
        return cls(_core.Design.from_litematic(data))

    @classmethod
    def import_litematic(cls, path) -> "Design":
        return cls(_core.Design.import_litematic(str(path)))

    # -- ports -------------------------------------------------------------

    def declare_input(self, name, anchor, step, width, ty="uint") -> None:
        (ax, ay, az), (sx, sy, sz) = anchor, step
        self._d.declare_input(name, ax, ay, az, sx, sy, sz, width, ty)

    def declare_output(self, name, anchor, step, width, ty="uint") -> None:
        (ax, ay, az), (sx, sy, sz) = anchor, step
        self._d.declare_output(name, ax, ay, az, sx, sy, sz, width, ty)

    # -- buses -------------------------------------------------------------

    def route_bus(self, name, driver, sinks, gates=(), style=None) -> Bus:
        gates_json, gate_names = _gates_json(gates)
        self._d.route_bus(name, driver, _names_json(sinks), gates_json,
                          _style_json(style))
        return Bus(self, name, gate_names)

    def route_bus_or(self, name, drivers, sinks, gates=(), style=None) -> Bus:
        gates_json, gate_names = _gates_json(gates)
        self._d.route_bus_or(name, _names_json(drivers), _names_json(sinks),
                             gates_json, _style_json(style))
        return Bus(self, name, gate_names)

    def add_gate(self, bus, gate, anchor, step) -> str:
        (x, y, z), (sx, sy, sz) = anchor, step
        return self._d.add_gate(bus, gate, x, y, z, sx, sy, sz)

    def move_gate(self, bus, gate, anchor) -> dict:
        x, y, z = anchor
        return json.loads(self._d.move_gate(bus, gate, x, y, z))

    def set_bus_rule(self, bus, rule=None, **fields) -> None:
        merged = dict(rule or {})
        merged.update(fields)
        self._d.set_bus_rule(bus, json.dumps(merged))

    def bus_state(self, name) -> str:
        return self._d.bus_state(name)

    def bus_skew(self, name) -> dict:
        return json.loads(self._d.bus_skew(name))

    def rip(self, name) -> None:
        self._d.rip(name)

    # -- cells / instances -------------------------------------------------

    def add_cell(self, name, schematic) -> str:
        return self._d.add_cell(name, _unwrap(schematic))

    def place(self, name, cell, at, rot=0) -> None:
        x, y, z = at
        self._d.place(name, cell, x, y, z, rot)

    def move_instance(self, name, at, rot=0) -> dict:
        x, y, z = at
        return json.loads(self._d.move_instance(name, x, y, z, rot))

    # -- lifecycle ---------------------------------------------------------

    def check(self, strict: bool = False) -> CheckReport:
        report = CheckReport(json.loads(self._d.check()))
        if strict and not report.clean:
            raise DesignCheckError(report)
        return report

    def flatten(self) -> Flat:
        return Flat(self._d.flatten())

    def bake(self, budget: int = 4000) -> Flat:
        return Flat(self._d.bake(budget))

    # -- persistence -------------------------------------------------------

    def save(self, path) -> None:
        """``.nucm`` -> project tier, ``.litematic`` -> layered interchange,
        anything else -> the flattened artifact."""
        p = str(path)
        if p.endswith(".nucm"):
            self._d.save_nucm(p)
        elif p.endswith(".litematic"):
            self._d.export_litematic(p)
        else:
            self.flatten().save(p)

    def save_nucm(self, path) -> None:
        self._d.save_nucm(str(path))

    def export_litematic(self, path) -> None:
        self._d.export_litematic(str(path))

    # -- passthrough -------------------------------------------------------

    def __getattr__(self, attr):
        return getattr(self._d, attr)
