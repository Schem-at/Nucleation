"""Corpus harness: run a declarative bus-routing scenario through the SOLVER,
verify it in mc-tick, and record numbers.

THE RULE THIS FILE EXISTS TO ENFORCE
------------------------------------
Nothing in a corpus entry's *bus* geometry is authored here.  The harness only
stamps ENDPOINT HARDWARE (lever banks, lamp banks), OBSTACLES, and verified
FIXTURES (pre-existing, separately-probed mechanisms placed as loose blocks).
Every redstone cell that carries a bus is produced by
``nucleation.Design.route_bus`` / ``route_bus_adapted``.  A scenario the solver
cannot route is recorded UNSOLVED with the router's own failure string; the
harness never fills in for it.

A scenario is a plain dict (JSON-serializable) so the gallery can print it
verbatim.  See ``scenarios/`` for the schema in use and ``SCHEMA`` below.
"""

from __future__ import annotations

import json
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
EDA = os.path.dirname(HERE)
ROOT = os.path.dirname(EDA)
sys.path.insert(0, EDA)  # reuse rs.py (build/sim helpers)

import nucleation as n  # noqa: E402
import rs  # noqa: E402
import vforms  # noqa: E402  the verified vertical forms (do not edit)

# mc-tick needs every block state INTERNED before construction or it sits inert
# (memory: "A schematic is not a world").  The fixtures use comparators and all
# four repeater delays, so widen rs's default set once, for the whole process.
DIRS = ("north", "south", "east", "west")
rs.EXTRA_STATES = rs.EXTRA_STATES + ";" + ";".join(
    ["minecraft:comparator[facing=%s,mode=%s,powered=%s]" % (d, m, p)
     for d in DIRS for m in ("compare", "subtract") for p in ("true", "false")]
    + ["minecraft:repeater[facing=%s,delay=%d,locked=%s,powered=%s]"
       % (d, dl, lk, pw)
       for d in DIRS for dl in (1, 2, 3, 4)
       for lk in ("true", "false") for pw in ("true", "false")])

STONE = "minecraft:stone"
DUST = ("minecraft:redstone_wire[east=none,north=none,power=0,"
        "south=none,west=none]")
LAMP = "minecraft:redstone_lamp[lit=false]"
LEVER = "minecraft:lever[face=floor,facing=north,powered=false]"
BLOCKER = "minecraft:polished_andesite"

SCHEMA = '''
SCENARIO = {
  "id":       "X01_cross8",             # file stem, gallery anchor
  "title":    "...",                    # one line
  "question": "...",                    # what this entry asks of the solver
  "ports": [
     {"name": "a_in", "dir": "in", "form": "vertical", "anchor": [1, 2, 8],
      "width": 8, "ty": "uint", "feed": [-1, 0, 0]}],   # feed = lever side
  "obstacles": [{"min": [...], "max": [...], "block": "..."}],
  "fixtures":  [{"kind": "hex_trunk", "name": "hex0", "at": [0, 0, 0],
                 "values": [1, 3, 7, 15]}],
  "buses": [
     {"name": "bus_a", "driver": "a_in", "sinks": ["a_out"],
      "style":   {"bus_block": "..."},              # -> nucleation.Style
      "rule":    {"y_band": [0, 20]},               # -> NetClassRule
      "adapted": {"align": 0, "shift": 0, "truncate": false}}],
  "verify": {
     "words":  [{"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
                 "map": {"kind": "identity"}}],
     "analog": [{"fixture": "hex0", "hold_ports": ["a_in"], "hold_bits": 170,
                 "also_read": ["a_out"]}],
     "hold_analog": {"fixture": "hex0", "level": 11}},
  "render": {"yaw": 135, "pitch": 30, "zoom": 1.8},
  "expect": "solved" | "unsolved",      # our prediction, recorded either way
  "notes":  "..."
}
'''

# ---------------------------------------------------------------------------
# forms

FORM_STEP = {
    "vertical": (0, 2, 0),        # dense 2y stack: bit i at y0 + 2i
    "vertical_desc": (0, -2, 0),  # msb-at-bottom stack (bit 0 highest)
    "flat_x": (2, 0, 0),          # flat 2-pitch along +X
    "flat_x_desc": (-2, 0, 0),
    "flat_z": (0, 0, 2),          # flat 2-pitch along +Z
    "flat_z_desc": (0, 0, -2),
}

DEFAULT_FEED = {
    "vertical": (-1, 0, 0),
    "vertical_desc": (-1, 0, 0),
    "flat_x": (0, 0, -1),
    "flat_x_desc": (0, 0, -1),
    "flat_z": (-1, 0, 0),
    "flat_z_desc": (-1, 0, 0),
}


def _add(a, b, k=1):
    return (a[0] + k * b[0], a[1] + k * b[1], a[2] + k * b[2])


class Layer:
    """The loose layer: endpoint hardware, obstacles and fixtures.

    Records every lever and every readback cell so verification can drive the
    real build without any knowledge of what the router did in between.
    """

    def __init__(self, name):
        self.name = name
        self.s = n.Schematic.create(name)
        self.cells = {}
        self.levers = {}   # port name -> [(x,y,z)] in bit order
        self.reads = {}    # port name -> [(x,y,z)] in bit order
        self.ports = []    # declare_* arguments, in declaration order
        self.analog = {}   # fixture name -> {"levels": {v: pos}, "inj", "out"}
        self.forms = {}    # fixture name -> geometry of a placed vertical form

    # -- raw stamping -------------------------------------------------------
    def put(self, x, y, z, block, force=False):
        prev = self.cells.get((x, y, z))
        if prev is not None and prev != block and not force:
            raise AssertionError("loose-layer collision at (%d,%d,%d): %s vs %s"
                                 % (x, y, z, prev, block))
        self.cells[(x, y, z)] = block
        self.s.set_block_from_string(x, y, z, block)

    def solid(self, x, y, z, block=STONE):
        if (x, y, z) not in self.cells:
            self.put(x, y, z, block)

    # -- endpoint hardware --------------------------------------------------
    def port(self, spec):
        """Stamp one port's hardware; remember how to drive and read it."""
        name = spec["name"]
        step = FORM_STEP[spec["form"]]
        feed = tuple(spec.get("feed") or DEFAULT_FEED[spec["form"]])
        anchor = tuple(spec["anchor"])
        stamp = spec.get("stamp", True)
        levers, reads = [], []
        for i in range(spec["width"]):
            cell = _add(anchor, step, i)
            reads.append(cell)
            if not stamp:
                # the hardware is already there (a fixture's own tap); declare
                # the port over it and let something else drive it
                continue
            if spec["dir"] == "in":
                lev = _add(cell, feed)
                self.solid(lev[0], lev[1] - 1, lev[2])
                self.put(lev[0], lev[1], lev[2], LEVER)
                self.solid(cell[0], cell[1] - 1, cell[2])
                self.put(cell[0], cell[1], cell[2], DUST)
                levers.append(lev)
            else:
                # the lamp IS the support: an arriving bit lights it
                self.put(cell[0], cell[1] - 1, cell[2], LAMP)
                self.put(cell[0], cell[1], cell[2], DUST)
        if levers:
            self.levers[name] = levers
        self.reads[name] = reads
        self.ports.append(dict(name=name, dir=spec["dir"], anchor=anchor,
                               step=step, width=spec["width"],
                               ty=spec.get("ty", "uint")))
        return anchor

    def pseudo_port(self, name, direction, cells, levers=None, width=None):
        """A drivable/readable bank that is NOT declared to the Design.

        Used for the endpoints of a placed FIXTURE: the verification machinery
        addresses it exactly like a port, but the solver never sees it, so no
        entry using one can be mistaken for solver output.
        """
        self.reads[name] = list(cells)
        if levers:
            self.levers[name] = list(levers)
        self.ports.append(dict(name=name, dir=direction, anchor=cells[0],
                               step=(0, 0, 0), width=width or len(cells),
                               ty="uint", declare=False))

    def obstacle(self, box):
        block = box.get("block", BLOCKER)
        lo, hi = box["min"], box["max"]
        for x in range(lo[0], hi[0] + 1):
            for y in range(lo[1], hi[1] + 1):
                for z in range(lo[2], hi[2] + 1):
                    self.put(x, y, z, block, force=True)

    def fixture(self, spec):
        FIXTURES[spec["kind"]](self, spec)


# ---------------------------------------------------------------------------
# fixtures: separately-probed mechanisms placed as loose blocks.
# NOT solver output -- the gallery labels them so.


def _hex_trunk(layer, spec):
    """The hex analog transport stage, reproduced from the CORPUS mechanism.

    Geometry is the parametric reconstruction verified 66/66 by
    ``redstone-eda/probe_hex_transmit.py`` (class ``Rig``) against the
    user-supplied ``corpus/TRANSMIT002_hex_transmit_flat.schem``: INPUT dust
    lane, a repeater comb, OUTPUT dust lane, one tap comparator; 3x2
    cross-section, 16 z per stage, value-preserving for all 16 levels.

    The barrel source is replaced by the probe's attenuator + injector run so
    the analog value can be selected while the sim is live.  Transport runs
    along -Z; the trunk occupies x = at.x .. at.x+2 and y = at.y .. at.y+1.
    """
    name = spec.get("name", "hex")
    ox, oy, oz = spec.get("at", (0, 0, 0))
    comb_len = spec.get("comb_len", 15)
    ztop = spec.get("ztop", 16)
    values = spec.get("values", [1, 3, 7, 15])
    zlo = ztop - comb_len + 1
    tap = zlo
    comp = "minecraft:comparator[facing=%s,mode=compare,powered=false]"

    def put(x, y, z, b):
        layer.put(ox + x, oy + y, oz + z, b)

    def dust(x, y, z):
        put(x, y - 1, z, STONE)
        put(x, y, z, DUST)

    for z in range(zlo, ztop + 1):
        dust(0, 1, z)                       # INPUT lane
        put(1, 0, z, STONE)
        put(1, 1, z, rs.repeater("west", 1))  # the comb, input from -X
        if z >= tap:
            dust(2, 1, z)                   # OUTPUT lane
    za = ztop + 2
    put(0, 0, ztop + 1, STONE)
    put(0, 1, ztop + 1, comp % "south")     # injects the analog value
    for x in range(0, 15):
        dust(x, 1, za)                      # attenuator run
    levels = {}
    for v in values:
        x = 15 - v                          # decays to exactly v on arrival
        put(x, 0, za + 1, STONE)
        put(x, 1, za + 1, LEVER)
        levels[v] = (ox + x, oy + 1, oz + za + 1)
    put(2, 0, tap - 1, STONE)
    put(2, 1, tap - 1, comp % "south")      # the tap
    dust(2, 1, tap - 2)

    layer.analog[name] = {
        "levels": levels,
        "inj": (ox + 0, oy + 1, oz + ztop),
        "out": (ox + 2, oy + 1, oz + tap - 2),
        "kind": "hex_comb_stage",
        "provenance": ("probe_hex_transmit.py::Rig (66/66) against "
                       "corpus/TRANSMIT002_hex_transmit_flat.schem"),
    }


class _BuildShim:
    """Just enough of ``rs.Build`` for the ``vforms`` constructors.

    ``vforms`` only ever calls ``force``, so the whole adapter is one method.
    The point is that the vertical forms are built by THEIR OWN verified
    constructors, character for character -- this harness does not reimplement
    them.
    """

    def __init__(self, layer):
        self.layer = layer

    def force(self, x, y, z, block):
        self.layer.put(x, y, z, block, force=True)


def _torch_ladder_bus(layer, spec):
    """The dense torch-ladder vertical bus, from ``vforms.ladder_bus``.

    1x1 column per bit at x-pitch 1, 2 y per torch, 1 gt/y, output refreshed to
    15 so there is no reach limit.  Verified 33/33 in
    ``probe_vertical_forms.py`` over all 256 patterns on 8 towers.  Ports must
    alternate +z/-z sides at pitch 1 (the POINTING LAW), which
    ``ladder_bus`` does for us.

    The harness adds only the lever that drives each entry and the lamp that
    reads each exit -- never a cell of the form itself.
    """
    name = spec.get("name", "ladder")
    x0, y0, z0 = spec.get("at", (0, 1, 0))
    nbits = spec.get("nbits", 8)
    torches = spec.get("torches", 4)
    axis = spec.get("axis", "x")
    pairs = vforms.ladder_bus(_BuildShim(layer), x0, z0, y0, nbits, torches,
                              axis=axis)

    entries = [p[0] for p in pairs]
    exits = [p[1] for p in pairs]
    levers = []
    for i, entry in enumerate(entries):
        side = -1 if i % 2 == 0 else +1        # matches ladder_bus's alternation
        d = (0, side) if axis == "x" else (side, 0)
        lev = (entry[0] + d[0], entry[1], entry[2] + d[1])
        layer.put(lev[0], lev[1] - 1, lev[2], STONE, force=True)
        layer.put(lev[0], lev[1], lev[2], LEVER, force=True)
        levers.append(lev)
    for ex in exits:
        layer.put(ex[0], ex[1] - 1, ex[2], LAMP, force=True)

    layer.pseudo_port(name + "_in", "in", entries, levers)
    layer.pseudo_port(name + "_out", "out", exits)
    layer.forms[name] = {
        "kind": "torch_ladder_bus",
        "constructor": ("vforms.ladder_bus(b, x0=%d, z=%d, y0=%d, nbits=%d, "
                        "torches=%d, axis=%r)"
                        % (x0, z0, y0, nbits, torches, axis)),
        "provenance": ("vforms.py, probed 33/33 in probe_vertical_forms.py "
                       "(8 towers x all 256 patterns)"),
        "rise_y": 2 * torches,
        "xz_cells_per_bit": 1,
        "inverting": (torches % 2 == 1),
        "provided_by": "fixture (NOT solver output)",
    }


def _ring_riser_bus(layer, spec):
    """The ring / spiral riser, from ``vforms.ring_bus``.

    A chordless perimeter where y equals the path index, so bits `sep` apart in
    phase are `sep` apart in y in every shared column; sep >= 3 is legal and a
    ring holds floor(perimeter/3) bits.  Passive: 1 y per cell, 0 gt, -1 ss per
    y, so the drop here is kept inside dust's reach.  Verified 26/26 in
    ``probe_spiral_tiling.py``.

    Driven from the TOP and read at the BOTTOM: this is the descending
    direction, for which there is no active carrier at all.
    """
    name = spec.get("name", "ring")
    sx, sz = spec.get("size", (3, 3))
    ox, oy, oz = spec.get("at", (0, 1, 0))
    n = spec.get("levels", 9)
    sep = spec.get("sep", 3)
    bits = vforms.ring_bus(_BuildShim(layer), sx, sz, ox, oz, oy, n, sep=sep)

    tops = [cells[-1] for cells in bits]
    bottoms = [cells[0] for cells in bits]
    levers = []
    for cell in tops:
        dx, dz = vforms.ring_outward(cell, ox, oz, sx, sz)
        lev = (cell[0] + dx, cell[1], cell[2] + dz)
        layer.put(lev[0], lev[1] - 1, lev[2], STONE, force=True)
        layer.put(lev[0], lev[1], lev[2], LEVER, force=True)
        levers.append(lev)
    for cell in bottoms:
        layer.put(cell[0], cell[1] - 1, cell[2], LAMP, force=True)

    layer.pseudo_port(name + "_in", "in", tops, levers)
    layer.pseudo_port(name + "_out", "out", bottoms)
    layer.forms[name] = {
        "kind": "ring_riser_bus",
        "constructor": ("vforms.ring_bus(b, sx=%d, sz=%d, ox=%d, oz=%d, "
                        "y0=%d, n=%d, sep=%d)" % (sx, sz, ox, oz, oy, n, sep)),
        "provenance": ("vforms.py, probed 26/26 in probe_spiral_tiling.py "
                       "(legal() predictor matched all 17 sims)"),
        "bits": len(bits),
        "drop_y": n - 1,
        "xz_cells_per_bit": round(sx * sz / float(len(bits)), 2),
        "phases": vforms.ring_bits(len(vforms.ring(sx, sz)), sep),
        "provided_by": "fixture (NOT solver output)",
    }


FIXTURES = {
    "hex_trunk": _hex_trunk,
    "torch_ladder_bus": _torch_ladder_bus,
    "ring_riser_bus": _ring_riser_bus,
}


# ---------------------------------------------------------------------------
# the solver invocation


def route(scn, layer):
    """Declare the ports, then hand every bus to the SOLVER.

    Returns (design, per-bus report, seconds).  Nothing here draws redstone.
    """
    d = n.Design.for_schematic(scn["id"], layer.s)
    for p in layer.ports:
        if p.get("declare") is False:
            continue                      # a fixture's own bank, not a port
        fn = d.declare_input if p["dir"] == "in" else d.declare_output
        try:
            fn(p["name"], anchor=p["anchor"], step=p["step"],
               width=p["width"], ty=p["ty"])
        except Exception as exc:
            # a port the API will not accept is a result, not a crash
            why = ("declare_%s(%r, anchor=%r, step=%r, width=%d, ty=%r) "
                   "refused: %s: %s" % (p["dir"], p["name"], p["anchor"],
                                        p["step"], p["width"], p["ty"],
                                        type(exc).__name__, exc))
            return d, {b["name"]: {"state": "not attempted", "error": why,
                                   "call": "port declaration failed first",
                                   "seconds": 0.0}
                       for b in scn["buses"]}, 0.0

    reports, t_total = {}, 0.0
    for b in scn["buses"]:
        style = n.Style(**b["style"]) if b.get("style") else None
        t0 = time.perf_counter()
        try:
            if b.get("adapted") is not None:
                a = b["adapted"]
                call = ("d.raw.route_bus_adapted(%r, %r, %r, '[]', %r, "
                        "align=%d, shift=%d, truncate=%r)"
                        % (b["name"], b["driver"], ",".join(b["sinks"]),
                           json.dumps(b.get("style") or {}),
                           a.get("align", 0), a.get("shift", 0),
                           bool(a.get("truncate", False))))
                d.raw.route_bus_adapted(
                    b["name"], b["driver"], ",".join(b["sinks"]),
                    json.dumps(b.get("gates", [])),
                    json.dumps(b.get("style") or {}),
                    int(a.get("align", 0)), int(a.get("shift", 0)),
                    bool(a.get("truncate", False)))
                state = d.bus_state(b["name"])
            elif len(b.get("drivers", [])) > 1:
                call = ("d.route_bus_or(%r, drivers=%r, sinks=%r)"
                        % (b["name"], b["drivers"], b["sinks"]))
                d.route_bus_or(b["name"], drivers=b["drivers"],
                               sinks=b["sinks"], style=style)
                state = d.bus_state(b["name"])
            else:
                call = ("d.route_bus(%r, driver=%r, sinks=%r%s)"
                        % (b["name"], b["driver"], b["sinks"],
                           (", style=n.Style(**%r)" % (b["style"],))
                           if b.get("style") else ""))
                state = d.route_bus(b["name"], driver=b["driver"],
                                    sinks=b["sinks"], style=style).state
            err = None
        except Exception as exc:                       # a router refusal
            state, err = "raised", "%s: %s" % (type(exc).__name__, exc)
            call = locals().get("call") or "?"
        dt = time.perf_counter() - t0
        t_total += dt
        if b.get("rule"):
            try:
                d.set_bus_rule(b["name"], **b["rule"])
                call += "\nd.set_bus_rule(%r, **%r)" % (b["name"], b["rule"])
            except Exception as exc:
                err = (err or "") + " | set_bus_rule: %s" % exc
        reports[b["name"]] = {"state": state, "error": err, "call": call,
                              "seconds": round(dt, 4)}
    return d, reports, t_total


# ---------------------------------------------------------------------------
# metrics


def bus_metrics(d, name):
    """Measure one routed bus: the router's own numbers where it has them
    (delay/skew), geometry measured off the cells it emitted."""
    try:
        cells = json.loads(d.raw.bus_blocks_json(name))
    except Exception:
        cells = []
    m = {
        "cells": len(cells),
        "wire": sum(1 for c in cells if "redstone_wire" in c[3]),
        "devices": sum(1 for c in cells if "repeater" in c[3]
                       or "comparator" in c[3] or "torch" in c[3]),
        "glass": sum(1 for c in cells if "glass" in c[3]),
    }
    if cells:
        xs, ys, zs = ([c[i] for c in cells] for i in (0, 1, 2))
        bbox = [[min(xs), min(ys), min(zs)], [max(xs), max(ys), max(zs)]]
        m["bbox"] = bbox
        m["footprint"] = ((bbox[1][0] - bbox[0][0] + 1)
                          * (bbox[1][1] - bbox[0][1] + 1)
                          * (bbox[1][2] - bbox[0][2] + 1))
    try:
        sk = d.bus_skew(name)
        m["delay_rt"] = sk.get("max_rt")
        m["skew_rt"] = sk.get("skew_rt")
        m["per_bit_rt"] = sk.get("per_bit_rt")
    except Exception:
        pass
    return m


def config_effect(scn, per_bus, outdir):
    """Did the config actually change the route?

    A knob that is accepted and then ignored is worse than one that is
    refused, so every configurability entry names a BASELINE and this compares
    the emitted geometry against it.  Identical geometry means the request
    never reached the router.
    """
    probe = scn.get("config_probe")
    if not probe:
        return None
    base = os.path.join(outdir, "results", probe["baseline"] + ".json")
    if not os.path.exists(base):
        return {"baseline": probe["baseline"], "verdict": "baseline missing",
                "note": "run the baseline scenario first"}
    with open(base) as fh:
        b = json.load(fh)
    keys = ("cells", "wire", "devices", "glass", "footprint", "bbox",
            "delay_rt", "skew_rt", "per_bit_rt")
    mine = {k: {kk: v.get(kk) for kk in keys} for k, v in per_bus.items()}
    theirs = {k: {kk: v.get(kk) for kk in keys}
              for k, v in (b.get("per_bus") or {}).items()}
    same = mine == theirs
    return {
        "baseline": probe["baseline"],
        "expect_change": bool(probe.get("expect_change", True)),
        "geometry_identical": same,
        "verdict": ("ignored" if same else "changed the route"),
        "mine": mine, "baseline_geometry": theirs,
    }


def cost_vector(per_bus):
    """The five-component vector the gallery reports.

    `coherence` is deliberately null.  `BusCostVector` IS computed in
    src/design.rs, but src/bridge/design.rs exposes no reader for it, so no
    binding can see the router's own number.  Inventing one here would be a
    different quantity wearing its name.
    """
    got = [m for m in per_bus.values() if m.get("cells")]
    if not got:
        return None
    return {
        "length": sum(m["wire"] for m in got),
        "delay_rt": max((m.get("delay_rt") or 0) for m in got),
        "skew_rt": max((m.get("skew_rt") or 0) for m in got),
        "coherence": None,
        "footprint": sum(m.get("footprint", 0) for m in got),
    }


# ---------------------------------------------------------------------------
# verification in mc-tick


def _true_min(schematic):
    """Minimum non-air coordinate, measured off the blocks themselves.

    NOT ``tight_bounds_min()``: on a reopened .schem that reports the
    ALLOCATED region's corner, which is one block below the lowest real block
    here -- and an off-by-one in the frame makes every readback return -1.
    """
    blocks = json.loads(schematic.get_non_air_blocks_json())
    return (min(b["x"] for b in blocks), min(b["y"] for b in blocks),
            min(b["z"] for b in blocks))


def open_sim(flat, settle=600):
    """A sim addressable in DESIGN coordinates.

    The engine sizes its world from the ALLOCATED region, not from the blocks
    in it, and that region is padded well past the content -- enough to blow
    the cell cap on a big build.  A ``.schem`` save normalises, so round-trip
    through a file first (what ``rs.Build.sim`` does) and recover the shift the
    save applied by comparing the two block sets' true minima.
    """
    tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"),
                       "_corpus_%d.schem" % os.getpid())
    flat.save(tmp)
    tight = n.Schematic.open(tmp)
    before, after = _true_min(flat.raw), _true_min(tight)
    off = tuple(b - a for b, a in zip(before, after))
    sim = n.TickSimulation.from_schematic(
        tight, n.TickSettleMode.Placement, 0, 0, 0, rs.EXTRA_STATES)
    sim.run_until_quiescent(settle)
    return rs.Sim(sim, off)


def patterns_for(width, kind):
    if kind == "exhaustive":
        return list(range(1 << width))
    pats = [1 << i for i in range(width)]
    pats += [0, (1 << width) - 1]
    if width >= 2:
        pats += [int("10" * (width // 2), 2), int("01" * (width // 2), 2)]
    return pats


def expected(word, mapping, width):
    """What the far end must read, given the bit mapping the SCENARIO asked
    the solver to realize."""
    kind = mapping.get("kind", "identity")
    mask = (1 << width) - 1
    if kind == "identity":
        return word & mask
    if kind == "reverse":
        w = mapping["width"]
        return int(format(word & ((1 << w) - 1), "0%db" % w)[::-1], 2) & mask
    if kind == "permute":
        out = 0                     # sink bit j takes source bit perm[j]
        for j, src in enumerate(mapping["perm"]):
            if word >> src & 1:
                out |= 1 << j
        return out & mask
    if kind == "shift":
        k = mapping["by"]
        return ((word << k) if k >= 0 else (word >> -k)) & mask
    raise ValueError("unknown mapping %r" % kind)


def _group(spec):
    if spec is None:
        return []
    return [spec] if isinstance(spec, str) else list(spec)


class Driver:
    """Drives and reads PORT GROUPS.  A group is one multi-bit port or a list
    of narrower ports concatenated bit-0-first.  Levers are flipped one at a
    time: a player flips levers one by one, and simultaneous flips inject
    transients a ripple chain can latch."""

    def __init__(self, sim, layer, settle):
        self.sim, self.layer, self.settle = sim, layer, settle
        self._cache = {}

    def _cells(self, group, table):
        out = []
        for name in group:
            out.extend(table[name])
        return out

    def _bank(self, group):
        key = ("lev",) + tuple(group)
        if key not in self._cache:
            # rs.Levers speaks the rs.Sim wrapper's DESIGN coordinates
            self._cache[key] = rs.Levers(
                self.sim, self._cells(group, self.layer.levers))
        return self._cache[key]

    def drive(self, group, word):
        bank = self._bank(group)
        bank.set([bool(word >> i & 1) for i in range(len(bank.positions))],
                 self.settle)

    def read(self, group):
        val = 0
        for i, c in enumerate(self._cells(group, self.layer.reads)):
            if self.sim.power(*c) > 0:
                val |= 1 << i
        return val

    def hold_level(self, fixture, level, on=True):
        f = self.layer.analog[fixture]
        key = ("analog", fixture)
        if key not in self._cache:
            keys = sorted(f["levels"])
            self._cache[key] = (keys, rs.Levers(
                self.sim, [f["levels"][k] for k in keys]))
        keys, bank = self._cache[key]
        bank.set([bool(on) and k == level for k in keys], self.settle)


def verify(scn, layer, flat, log):
    """Drive the REAL routed build and read the far end.

    Two primitives, composable in one scenario:
      words  -- drive a word in, read the far end, compare against the bit
                mapping the scenario asked the solver for
      analog -- sweep every analog level through a hex fixture and read it back
    Either may run while the other is held live, which is the only way to prove
    a crossing corrupts neither side.
    """
    v = scn.get("verify") or {}
    if not v:
        return {"kind": "none", "passed": 0, "total": 0, "ok": None}
    settle = v.get("settle", 600)
    sim = open_sim(flat, settle)
    drv = Driver(sim, layer, settle)
    by = {p["name"]: p for p in layer.ports}
    res = {"cases": [], "passed": 0, "total": 0, "sections": {}}

    def note(section, row):
        row["section"] = section
        res["cases"].append(row)
        res["total"] += 1
        res["passed"] += bool(row["ok"])
        s = res["sections"].setdefault(section, {"passed": 0, "total": 0})
        s["total"] += 1
        s["passed"] += bool(row["ok"])

    hold = v.get("hold_analog")
    if hold:
        drv.hold_level(hold["fixture"], hold["level"], True)

    for w in v.get("words", []):
        gin, gout = _group(w["in"]), _group(w["out"])
        win = sum(by[p]["width"] for p in gin)
        wout = sum(by[p]["width"] for p in gout)
        mapping = dict(w.get("map") or {"kind": "identity"})
        mapping.setdefault("width", win)
        pats = w.get("pattern_list")
        if pats is None:
            pats = patterns_for(win, w.get("patterns", "walking+extremes"))
        label = w.get("label") or "%s -> %s" % ("+".join(gin), "+".join(gout))
        for pat in pats:
            drv.drive(gin, pat)
            sim.settle(settle)
            got, want = drv.read(gout), expected(pat, mapping, wout)
            row = {"in": pat, "got": got, "want": want, "ok": got == want}
            if hold:
                a = sim.power(*layer.analog[hold["fixture"]]["out"])
                row["hex_out"] = a
                row["ok"] = row["ok"] and a == hold["level"]
            note(label, row)
        drv.drive(gin, 0)

    if hold:
        drv.hold_level(hold["fixture"], hold["level"], False)

    for a in v.get("analog", []):
        fname = a.get("fixture") or sorted(layer.analog)[0]
        f = layer.analog[fname]
        gh, gr = _group(a.get("hold_ports")), _group(a.get("also_read"))
        bits = a.get("hold_bits") or 0
        if gh:
            drv.drive(gh, bits)
            sim.settle(settle)
        label = a.get("label") or ("analog %s" % fname)
        for level in sorted(f["levels"]):
            drv.hold_level(fname, level, True)
            sim.settle(settle)
            inj, out = sim.power(*f["inj"]), sim.power(*f["out"])
            row = {"level": level, "injected": inj, "hex_out": out,
                   "ok": out == level and inj == level}
            if gr:
                row["bits"] = drv.read(gr)
                row["ok"] = row["ok"] and row["bits"] == bits
            for p in _group(a.get("read_power")):
                # STRENGTH at the far end, not a bit: an analog value survives
                # a route only if the number itself arrives
                got = sim.power(*layer.reads[p][0])
                row["ss_" + p] = got
                row["ok"] = row["ok"] and got == level
            note(label, row)
        drv.hold_level(fname, None, False)
        if gh:
            drv.drive(gh, 0)

    res["ok"] = res["total"] > 0 and res["passed"] == res["total"]
    log.append("verify %d/%d (%s)" % (
        res["passed"], res["total"],
        ", ".join("%s %d/%d" % (k, s["passed"], s["total"])
                  for k, s in res["sections"].items())))
    return res


# ---------------------------------------------------------------------------
# the runner


def run(scn, outdir):
    log = []
    t0 = time.perf_counter()
    layer = Layer(scn["id"])
    for fx in scn.get("fixtures", []):
        layer.fixture(fx)
    for p in scn["ports"]:
        layer.port(p)
    for o in scn.get("obstacles", []):
        layer.obstacle(o)

    d, reports, solve_s = route(scn, layer)
    routed = [b["name"] for b in scn["buses"]
              if reports[b["name"]]["state"] == "routed"]
    all_routed = len(routed) == len(scn["buses"])
    per_bus = {name: bus_metrics(d, name) for name in routed}

    result = {
        "id": scn["id"], "title": scn["title"], "question": scn["question"],
        "expect": scn.get("expect", "solved"),
        "notes": scn.get("notes", ""),
        "scenario": scn,
        "buses": reports,
        "per_bus": per_bus,
        "cost_vector": cost_vector(per_bus),
        "solve_seconds": round(solve_s, 4),
        "loose_blocks": len(layer.cells),
        "fixtures": dict(
            [(k, {"kind": v["kind"], "provenance": v["provenance"]})
             for k, v in layer.analog.items()]
            + [(k, v) for k, v in layer.forms.items()]),
        "solver_produced": scn.get("solver_produced", True),
        "solved": False, "check": None, "verification": None,
        "artifact": None, "log": log,
    }

    if not all_routed:
        result["blocked_by"] = "; ".join(
            "%s -> %s%s" % (k, v["state"], (": " + v["error"]) if v["error"]
                            else "")
            for k, v in reports.items() if v["state"] != "routed")
        result["wall_seconds"] = round(time.perf_counter() - t0, 3)
        return result

    try:
        rep = d.check(strict=False)
        result["check"] = {"clean": bool(getattr(rep, "clean", False)),
                           "repr": repr(rep),
                           "drc": [str(x) for x in
                                   list(getattr(rep, "drc", []))[:12]],
                           "rules": [str(x) for x in
                                     list(getattr(rep, "rules", []))[:12]]}
    except Exception as exc:
        result["check"] = {"clean": False, "repr": "check raised: %s" % exc}

    flat = d.bake(scn.get("bake_budget", 4000))
    os.makedirs(os.path.join(outdir, "artifacts"), exist_ok=True)
    art = os.path.join(outdir, "artifacts", scn["id"] + ".schem")
    flat.save(art)
    result["artifact"] = os.path.relpath(art, outdir)
    result["artifact_blocks"] = len(json.loads(
        n.Schematic.open(art).get_non_air_blocks_json()))

    try:
        result["verification"] = verify(scn, layer, flat, log)
    except Exception as exc:
        import traceback
        result["verification"] = {
            "ok": False, "passed": 0, "total": 0,
            "error": "%s: %s" % (type(exc).__name__, exc),
            "trace": traceback.format_exc()[-1200:]}

    v = result["verification"] or {}
    result["solved"] = bool(all_routed and v.get("ok"))
    if not result["solved"]:
        result["blocked_by"] = ("routed, but verification failed: %s"
                               % (v.get("error")
                                  or "%d/%d cases" % (v.get("passed", 0),
                                                      v.get("total", 0))))

    # A fixture-only entry can never be SOLVED, however well it simulates: the
    # geometry is not the solver's.  The whole card exists to say so.
    if scn.get("solver_produced") is False:
        result["form_verified"] = bool(v.get("ok"))
        result["solved"] = False
        result["blocked_by"] = scn.get("blocked_by") or (
            "the form is verified in simulation here, but the solver cannot "
            "select it: this geometry came from its own constructor, not from "
            "route_bus")

    # A configurability entry is only SOLVED if the knob actually did something.
    ce = config_effect(scn, per_bus, outdir)
    if ce:
        result["config_effect"] = ce
        if ce.get("expect_change") and ce.get("geometry_identical"):
            result["solved"] = False
            result["blocked_by"] = (
                "the bus routes and verifies, but the CONFIGURATION was "
                "ignored: geometry is identical to %s" % ce["baseline"])
    result["wall_seconds"] = round(time.perf_counter() - t0, 3)
    return result
