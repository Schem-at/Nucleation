"""Genlib mapping onto comparator cells: yosys `abc -genlib` -> placed rows.

The PLA fabric (hdl/hdl2redstone.py) pays for generality: dual rails, QM
covers, collector rails.  This is the cell-mapped alternative: a small
library of VERIFIED flat cells (torch INV, comparator-subtract AND2/XOR2,
repeater-merge OR2, and their inverted tails), a genlib whose areas/delays
are the MEASURED block counts / structural rt of those cells, yosys+abc
technology mapping, levelized row placement, and the A* maze router for the
inter-cell nets.  Every produced schematic is exhaustively sim-verified
against a pure-Python evaluation of the mapped netlist before it may be
saved.

Cell physics (all probed earlier, see cells.py / pivot_tiles.md):
  * comparator subtract:  out = max(back - side, 0); binary AND = a - NOT b,
    XOR = (a-b) OR (b-a) (the cells.py half-adder, minus its carry branch);
  * torch NOT: dead-end dust runway points into the attachment block
    (POINTING LAW) so the block is weak-powered and the torch inverts;
  * repeater OR: repeaters are diodes, so two repeater outputs may share a
    dust join without back-driving their sources;
  * torches / comparator sides are INVISIBLE to nets.py (dust-only model):
    every such cell carries an explicit keepout so the router cannot lay a
    foreign dust in electrical reach (`Router.strong` is the enforcement
    hook -- same mechanism as station exit blocks).

Usage:
  ~/eda-venv/bin/python genlib_map.py --cells          # verify the library
  ~/eda-venv/bin/python genlib_map.py --design seg7    # map+place+verify
  ~/eda-venv/bin/python genlib_map.py --design cmp4 --out showcase/genlib_cmp4.schem
"""
import argparse
import os
import re
import subprocess
import sys

import rs
import cells
import nets
import audit
import router as router_mod

HERE = os.path.dirname(os.path.abspath(__file__))
BUILD_DIR = os.path.join(HERE, "genlib_build")

KEEP = "#keepout"          # Router.strong label: blocks foreign AND own dust


# ---------------------------------------------------------------------------
# cell fragments
# ---------------------------------------------------------------------------

class LibCell:
    def __init__(self, frag, delay_rt, keepout, inputs, name):
        self.frag = frag                  # cells.Fragment: ports a,b?,out
        self.delay_rt = delay_rt          # worst input->out structural rt
        self.keepout = keepout            # local cells to strong-mark
        self.inputs = inputs              # ordered input pin names
        self.name = name
        self.area = len(frag.cells)
        xs = [p[0] for p in frag.cells]
        zs = [p[2] for p in frag.cells]
        self.w = max(xs) - min(xs) + 1
        self.d = max(zs) - min(zs) + 1


def _dust(b, labels, x, y, z, lab):
    b.stone(x, y - 1, z)
    b.put(x, y, z, rs.DUST)
    labels[(x, y, z)] = lab


def _wall_torch_east(b, x, y, z):
    """Torch at (x,y,z) hanging off the block at (x-1,y,z), verified in rs."""
    b.put(x, y, z, rs.wall_torch("east"))


def _in_rep(b, labels, z, sig):
    """Port dust + repeater: the wire delivers ANY ss>=1, the repeater
    restores a hard 15 -- comparator cells are analog and die on weak
    arrivals (seg7's n54 arrived at ss3 and vanished inside the runway)."""
    _dust(b, labels, 0, 1, z, sig)
    b.stone(1, 0, z)
    b.put(1, 1, z, rs.repeater("west"))


def build_inv():
    """a -> repeater -> block -> torch -> out.  2 rt."""
    b = rs.Build("inv_cell")
    labels = {}
    _in_rep(b, labels, 0, "a")
    b.stone(2, 1, 0, "inv")
    _wall_torch_east(b, 3, 1, 0)
    _dust(b, labels, 4, 1, 0, "out")
    f = cells.Fragment()
    f.cells, f.labels = dict(b.cells), labels
    f.ports = {"a": (0, 1, 0), "out": (4, 1, 0)}
    keep = [(3, 1, 0)]                      # torch: powers ANY adjacent dust
    return LibCell(f, {"a": 2}, keep, ["a"], "INV")


def build_and2():
    """out = a - NOT(b): torch inverter on b feeds a subtract comparator's
    side; back input is a.  a:2rt, b:3rt."""
    b = rs.Build("and2_cell")
    labels = {}
    _in_rep(b, labels, 0, "a")
    _dust(b, labels, 2, 1, 0, "a")
    b.stone(3, 0, 0)
    b.put(3, 1, 0, cells.COMP % ("west", "subtract"))
    _dust(b, labels, 4, 1, 0, "out")
    # b branch: repeater -> block -> torch -> side dust north of the torch
    _in_rep(b, labels, 2, "b")
    b.stone(2, 1, 2, "inv")
    _wall_torch_east(b, 3, 1, 2)
    b.stone(3, 0, 1)
    b.put(3, 1, 1, rs.DUST)
    labels[(3, 1, 1)] = "notb"
    f = cells.Fragment()
    f.cells, f.labels = dict(b.cells), labels
    f.ports = {"a": (0, 1, 0), "b": (0, 1, 2), "out": (4, 1, 0)}
    keep = [(3, 1, 2),                      # torch
            (3, 1, 0),                      # comparator (side reach)
            (2, 1, -1), (2, 1, 1)]          # a-aimer flanks (dust into back)
    return LibCell(f, {"a": 2, "b": 3}, keep, ["a", "b"], "AND2")


def build_or2():
    """Repeater-isolated dust join.  1 rt."""
    b = rs.Build("or2_cell")
    labels = {}
    for z, sig in ((0, "a"), (2, "b")):
        _in_rep(b, labels, z, sig)
        _dust(b, labels, 2, 1, z, "out")
    _dust(b, labels, 2, 1, 1, "out")
    _dust(b, labels, 3, 1, 1, "out")
    f = cells.Fragment()
    f.cells, f.labels = dict(b.cells), labels
    f.ports = {"a": (0, 1, 0), "b": (0, 1, 2), "out": (3, 1, 1)}
    keep = [(3, 1, 0), (3, 1, 2)]           # join flanks: foreign dust there
    #                                         joins the out net diagonally
    return LibCell(f, {"a": 1, "b": 1}, keep, ["a", "b"], "OR2")


def build_nand2():
    """AND2 with a torch tail.  a:3rt, b:4rt."""
    a2 = build_and2()
    b = rs.Build("nand2_cell")
    labels = dict(a2.frag.labels)
    b.cells.update(a2.frag.cells)
    for p, blk in a2.frag.cells.items():
        b.s.set_block_from_string(*p, blk)
    labels[(4, 1, 0)] = "andv"              # comparator out is now internal
    b.stone(5, 1, 0, "inv")
    _wall_torch_east(b, 6, 1, 0)
    _dust(b, labels, 7, 1, 0, "out")
    f = cells.Fragment()
    f.cells, f.labels = dict(b.cells), labels
    f.ports = {"a": (0, 1, 0), "b": (0, 1, 2), "out": (7, 1, 0)}
    keep = a2.keepout + [(6, 1, 0), (4, 1, 1), (4, 1, -1)]
    return LibCell(f, {"a": 3, "b": 4}, keep, ["a", "b"], "NAND2")


def build_nor2():
    """OR2 with a torch tail.  2 rt."""
    o2 = build_or2()
    b = rs.Build("nor2_cell")
    labels = dict(o2.frag.labels)
    b.cells.update(o2.frag.cells)
    for p, blk in o2.frag.cells.items():
        b.s.set_block_from_string(*p, blk)
    for p in ((2, 1, 0), (2, 1, 1), (2, 1, 2), (3, 1, 1)):
        labels[p] = "orv"                   # join is now internal
    b.stone(4, 1, 1, "inv")
    _wall_torch_east(b, 5, 1, 1)
    _dust(b, labels, 6, 1, 1, "out")
    f = cells.Fragment()
    f.cells, f.labels = dict(b.cells), labels
    f.ports = {"a": (0, 1, 0), "b": (0, 1, 2), "out": (6, 1, 1)}
    keep = o2.keepout + [(5, 1, 1), (3, 1, 0), (3, 1, 2)]
    return LibCell(f, {"a": 2, "b": 2}, keep, ["a", "b"], "NOR2")


# C3 carry branch of the half adder, dropped to make a pure XOR2 (the tap
# repeater at (8,1,5) reads the xor line without shaping it, so removal is
# electrically inert; (6..7,1,6) were only C3's back feed).
_XOR_DROP = [(8, 1, 6), (8, 0, 6), (9, 1, 6), (9, 0, 6),
             (8, 1, 5), (8, 0, 5), (6, 1, 6), (6, 0, 6), (7, 1, 6), (7, 0, 6)]


def build_xor2():
    """cells.build_half_adder() minus the carry branch, shifted +2x behind
    fresh port repeaters (the HA was verified with lever-strength inputs;
    the repeaters recreate exactly that condition).  Worst path 4 rt."""
    ha = cells.build_half_adder()
    b = rs.Build("xor2_cell")
    labels = {}
    for p, blk in ha.cells.items():
        if p in _XOR_DROP:
            continue
        b.put(p[0] + 2, p[1], p[2], blk)
    for p, l in ha.labels.items():
        if p in _XOR_DROP:
            continue
        # the xor merge line IS the output net: relabel so composition
        # renames it to the mapped net (ports/labels must agree)
        labels[(p[0] + 2, p[1], p[2])] = "out" if l == "xor" else l
    for z, sig in ((0, "a"), (4, "b")):
        _in_rep(b, labels, z, sig)
    f = cells.Fragment()
    f.cells, f.labels = dict(b.cells), labels
    f.ports = {"a": (0, 1, 0), "b": (0, 1, 4), "out": (11, 1, 4)}
    keep = [(5, 1, 0), (7, 1, 4),              # comparators C1 / C2
            (4, 1, -1), (4, 1, 1),             # a-runway flanks before C1
            (6, 1, 3)]                         # b-runway flank before C2
    return LibCell(f, {"a": 4, "b": 3}, keep, ["a", "b"], "XOR2")


def build_xnor2():
    """XOR2 with a torch tail.  5 rt."""
    x2 = build_xor2()
    b = rs.Build("xnor2_cell")
    labels = dict(x2.frag.labels)
    b.cells.update(x2.frag.cells)
    for p, blk in x2.frag.cells.items():
        b.s.set_block_from_string(*p, blk)
    for p, l in list(labels.items()):
        if l == "out":
            labels[p] = "xorv"          # xor merge line is internal here
    b.stone(12, 1, 4, "inv")
    _wall_torch_east(b, 13, 1, 4)
    _dust(b, labels, 14, 1, 4, "out")
    f = cells.Fragment()
    f.cells, f.labels = dict(b.cells), labels
    f.ports = {"a": (0, 1, 0), "b": (0, 1, 4), "out": (14, 1, 4)}
    keep = x2.keepout + [(13, 1, 4), (11, 1, 3), (11, 1, 5)]
    return LibCell(f, {"a": 5, "b": 4}, keep, ["a", "b"], "XNOR2")


_BUILDERS = [build_inv, build_and2, build_or2, build_nand2, build_nor2,
             build_xor2, build_xnor2]

_EVAL = {
    "INV": lambda a: 1 - a,
    "AND2": lambda a, b: a & b,
    "OR2": lambda a, b: a | b,
    "NAND2": lambda a, b: 1 - (a & b),
    "NOR2": lambda a, b: 1 - (a | b),
    "XOR2": lambda a, b: a ^ b,
    "XNOR2": lambda a, b: 1 - (a ^ b),
}


def build_library():
    return {c.name: c for c in (f() for f in _BUILDERS)}


def verify_library(lib):
    ok = True
    for name, cell in sorted(lib.items()):
        fn = _EVAL[name]
        if len(cell.inputs) == 1:
            truth = [(a, fn(a)) for a in (0, 1)]
        else:
            truth = [(a, b, fn(a, b)) for a in (0, 1) for b in (0, 1)]
        ok = cells.verify_fragment(cell.frag, cell.inputs, ["out"],
                                   truth, name) and ok
        print("   %s: %d blocks, %dx%d footprint, %s rt"
              % (name, cell.area, cell.w, cell.d, cell.delay_rt))
    return ok


# ---------------------------------------------------------------------------
# genlib + yosys
# ---------------------------------------------------------------------------

_FORMULA = {
    "INV": ("O=!a;", "INV"),
    "AND2": ("O=a*b;", "NONINV"),
    "OR2": ("O=a+b;", "NONINV"),
    "NAND2": ("O=!(a*b);", "INV"),
    "NOR2": ("O=!(a+b);", "INV"),
    "XOR2": ("O=a*!b+!a*b;", "UNKNOWN"),
    "XNOR2": ("O=!(a*!b+!a*b);", "UNKNOWN"),
}


def write_genlib(lib, path):
    lines = []
    for name, cell in sorted(lib.items()):
        formula, phase = _FORMULA[name]
        d = max(cell.delay_rt.values())
        lines.append("GATE %-6s %3d  %-16s PIN * %s 1 999 %.1f 0.0 %.1f 0.0"
                     % (name, cell.area, formula, phase, d, d))
    # abc SEGFAULTS without a buffer and both constants in the library.
    # BUF never reaches the netlist (opt_clean absorbs it); constants are
    # rejected by parse_blif if a design ever produces one.
    lines.append("GATE BUF     0  O=a;             "
                 "PIN * NONINV 1 999 0.0 0.0 0.0 0.0")
    lines.append("GATE ZERO    0  O=CONST0;")
    lines.append("GATE ONE     0  O=CONST1;")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def run_yosys(verilog, genlib, blif):
    script = ("read_verilog %s; synth -noabc -flatten; abc -genlib %s; "
              "opt_clean; write_blif -gates %s" % (verilog, genlib, blif))
    r = subprocess.run(["yosys", "-q", "-p", script],
                       capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(r.stdout + r.stderr)
        raise RuntimeError("yosys failed")


class Netlist:
    def __init__(self):
        self.inputs, self.outputs, self.gates = [], [], []


def parse_blif(path):
    """Parse the mapped BLIF: .gate lines only (plus header)."""
    nl = Netlist()
    with open(path) as f:
        text = f.read().replace("\\\n", " ")
    names_stub = None
    for line in text.splitlines():
        line = line.strip()
        if line.startswith(".inputs"):
            nl.inputs = line.split()[1:]
        elif line.startswith(".outputs"):
            nl.outputs = line.split()[1:]
        elif line.startswith(".gate"):
            parts = line.split()
            kind = parts[1]
            pins = dict(p.split("=", 1) for p in parts[2:])
            nl.gates.append((kind, pins))
            names_stub = None
        elif line.startswith(".names"):
            names_stub = line.split()[1:]
            if names_stub not in (["$false"], ["$true"], ["$undef"]):
                raise RuntimeError("unmapped .names in %s: %s" % (path, line))
        elif line and not line.startswith((".", "#")) and names_stub:
            if names_stub == ["$true"] and line == "1":
                continue          # the constant-one stub's single table row
            raise RuntimeError("constant table rows unsupported: %s" % line)
    used = set()
    for _k, pins in nl.gates:
        used.update(pins.values())
    for bad in ("$false", "$true", "$undef"):
        if bad in used:
            raise RuntimeError("netlist uses constant %s" % bad)
    return nl


def eval_netlist(nl, order, pi_bits):
    """Reference evaluation: net name -> 0/1."""
    val = dict(pi_bits)
    for kind, pins in order:
        args = [val[pins[p]] for p in ("a", "b") if p in pins]
        val[pins["O"]] = _EVAL[kind](*args)
    return val


def topo_gates(nl):
    ready = set(nl.inputs)
    pending = list(nl.gates)
    order = []
    while pending:
        rest = []
        for g in pending:
            kind, pins = g
            if all(pins[p] in ready for p in ("a", "b") if p in pins):
                order.append(g)
                ready.add(pins["O"])
            else:
                rest.append(g)
        if len(rest) == len(pending):
            raise RuntimeError("combinational loop in mapped netlist")
        pending = rest
    return order


# ---------------------------------------------------------------------------
# placement + routing
# ---------------------------------------------------------------------------

CLEAR_Z = 5            # rows between stacked cells: keepout flanks + halo
PI_X = 0
PI_PITCH = 6
COL_CHANNEL = 14       # routing channel width between columns
ROUTE_Y_MAX = 6        # two extra routing layers for flyovers


def place(nl, lib, order):
    """Levelized placement: column per logic level, barycentric z order."""
    level = {n: 0 for n in nl.inputs}
    for kind, pins in order:
        level[pins["O"]] = 1 + max(level[pins[p]] for p in ("a", "b")
                                   if p in pins)
    inst_level = [max(level[pins[p]] for p in ("a", "b") if p in pins) + 1
                  for kind, pins in order]
    ncols = max(inst_level)
    colw = [0] * (ncols + 1)
    for (kind, _pins), lv in zip(order, inst_level):
        colw[lv] = max(colw[lv], lib[kind].w)
    colx = [0] * (ncols + 1)
    # column 1 must clear the PI runways AND the 5-deep approach corridors:
    # a corridor mouth sits 5 cells west of a port, and dust shorts at
    # distance 1 (a column-1 mouth at x=3 landed beside d[2]'s runway)
    x = PI_X + 12
    for lv in range(1, ncols + 1):
        colx[lv] = x
        x += colw[lv] + COL_CHANNEL

    pi_pos = {n: (PI_X + 2, 1, i * PI_PITCH) for i, n in enumerate(nl.inputs)}
    src_z = {n: p[2] for n, p in pi_pos.items()}

    placements = [None] * len(order)
    for lv in range(1, ncols + 1):
        col = [(i, order[i]) for i in range(len(order)) if inst_level[i] == lv]
        # barycentre of fanin source rows keeps nets short and uncrossed
        def bary(item):
            _i, (kind, pins) = item
            zs = [src_z[pins[p]] for p in ("a", "b") if p in pins]
            return sum(zs) / len(zs)
        col.sort(key=bary)
        z = 0
        for i, (kind, pins) in col:
            placements[i] = (colx[lv], 0, z)
            out_p = lib[kind].frag.ports["out"]
            src_z[pins["O"]] = z + out_p[2]
            z += lib[kind].d + CLEAR_Z
    return placements, pi_pos


class CountingRouter(router_mod.Router):
    """Router that reports the path and repeaters/torches added per route."""

    # margin 3 instead of 2: the worst laid dust must still light an
    # EXISTING corridor-mouth dust one cell on (emit cannot refresh cells
    # it does not lay)
    REFRESH = 13

    def __init__(self, *a, **kw):
        super().__init__(*a, **kw)
        self.veto = set()
        self.cellbox = {}      # pos -> frozenset(labels allowed here): hard
        #                        per-cell obstruction region.  Foreign nets
        #                        may not route inside a cell's claimed box
        #                        AT ALL -- ends the pocket-sealing arms race
        #                        (soft costs and stubs only bent it).

    def dust_ok(self, p, label, friendly=None):
        if p in self.veto:
            return False
        allowed = self.cellbox.get(p)
        if allowed is not None and label not in allowed:
            return False
        return super().dust_ok(p, label, friendly)

    def move_ok(self, p, q, label, friendly=None):
        """Ban own-net grazing: a NEW dust cell may touch own-net dust only
        through its path predecessor or at the destination aperture.  A
        branch that touches its trunk on both sides of a repeater closes a
        self-sustaining repeater ring (it latches; nets.check is blind to
        it because every cell is the same net)."""
        if q in self.b.cells:
            return True                     # trunk reuse walks ALONG the net
        ok = (friendly or set()) | {label}
        import nets as _nets
        self.b.cells[q] = rs.DUST
        try:
            for nb in _nets.neighbours(self.b.cells, q):
                if nb == p or nb in self._dsts:
                    continue
                if self.labels.get(nb) in ok:
                    return False            # side contact with own net
        finally:
            del self.b.cells[q]
        return True

    def _find_checked(self, src, dst, label, friendly):
        """find(), rejecting self-conflicting paths: a path that crosses
        directly above its own earlier segment lays the flyover's support
        ON the lower dust (emit is sequential and cannot know the future)."""
        self.veto = set()
        for _attempt in range(6):
            path = self.find(src, dst, label, friendly)
            if path is None:
                path = self.find(src, dst, label, friendly,
                                 max_iter=10_000_000)
            if path is None:
                return None
            cells_ = {p for p, _mv in path}
            dsts = {dst} if isinstance(dst, tuple) else set(dst)
            # (a) support collision: a flyover's support lands ON a lower
            #     path cell; (b) cap collision: a flyover's support lands
            #     directly ABOVE a lower path/dst dust, cutting the
            #     diagonal the path itself needs (seg7's g39.b mouth)
            bad = [p for p in cells_ if (p[0], p[1] + 1, p[2]) in cells_]
            bad += [p for p in cells_
                    if (p[0], p[1] - 2, p[2]) in cells_
                    or (p[0], p[1] - 2, p[2]) in dsts]
            if not bad:
                self.veto = set()
                return path
            self.veto |= set(bad)
        self.veto = set()
        return None

    def explain(self, p, label):
        """Why can this cell not take dust for `label`?  (debug aid)"""
        import nets as _nets
        x, y, z = p
        ok = {label}
        if p in self.b.cells:
            return "occ:%s/%s" % (self.b.cells[p].split("[")[0][10:],
                                  self.labels.get(p))
        for nb in ((x + 1, y, z), (x - 1, y, z), (x, y, z + 1), (x, y, z - 1),
                   (x, y + 1, z), (x, y - 1, z)):
            if nb in self.strong and self.strong[nb] not in ok:
                return "strong@%s=%s" % (nb, self.strong[nb])
        sup = (x, y - 1, z)
        s = self.b.cells.get(sup)
        if s is not None and not self.b.solid_at(*sup):
            return "badsup"
        self.b.cells[p] = rs.DUST
        try:
            for q in _nets.neighbours(self.b.cells, p):
                if self.labels.get(q) is not None \
                        and self.labels.get(q) not in ok:
                    return "touch:%s@%s" % (self.labels.get(q), q)
        finally:
            del self.b.cells[p]
        return "OK"

    def route(self, src, dst, label, friendly=None):
        path = self._find_checked(src, dst, label, friendly)
        if path is None:
            dsts = [dst] if isinstance(dst, tuple) else list(dst)
            for d in dsts:
                dx0, dy0, dz0 = d
                print("-- no path to %s; surroundings:" % (d,))
                for ddz in (-2, -1, 0, 1, 2):
                    for ddx in (-3, -2, -1, 0, 1):
                        q = (dx0 + ddx, dy0, dz0 + ddz)
                        w = self.explain(q, label)
                        if w != "OK":
                            print("   %s %s" % (q, w))
            raise RuntimeError("router: no path for %s: %s -> %s"
                               % (label, src, dst))

        def active(v):
            return sum(1 for blk in v if "repeater" in blk or "torch" in blk)
        before = active(self.b.cells.values())
        self.emit(path, label, friendly)
        self.last_rt = active(self.b.cells.values()) - before
        self.last_path = path
        return len(path)


def compose(name, nl, lib, order, layout=None):
    """Stamp + route the mapped netlist.  `layout` overrides the levelized
    placer with explicit ((placements, pi_pos)) -- the annealer's entry."""
    placements, pi_pos = layout if layout is not None \
        else place(nl, lib, order)
    b = rs.Build("genlib_" + name)
    labels = {}
    lever_at = {}
    lever_marks = []
    for n, (x, y, z) in pi_pos.items():
        b.stone(x - 2, y - 1, z)
        b.force(x - 2, y, z, rs.LEVER_OFF)
        for dx in (-1, 0):
            b.stone(x + dx, y - 1, z)
            b.put(x + dx, y, z, rs.DUST)
            labels[(x + dx, y, z)] = n
        lever_at[n] = (x - 2, y, z)
        # an ON lever powers EVERY adjacent dust -- it is a strong source
        # nets.py cannot see (d[3]'s wire once picked up 10 from a
        # neighbouring PI's lever).  Claim its neighbourhood for its net.
        lever_marks.append(((x - 2, y, z), n))

    keep_marks = []
    inst_ports = []
    conn = {}              # net -> dust cells PROVEN connected to the driver
    sink_cells = {}        # (inst, pin) -> that sink's runway+stub cells
    r_ss_seed = []         # (pos, ss) recorded during stamping
    for n, (x, y, z) in pi_pos.items():
        conn[n] = [(x - 1, y, z), (x, y, z)]
    for i, (kind, pins) in enumerate(order):
        cell = lib[kind]
        dx, dy, dz = placements[i]

        def rn(sig, pins=pins, i=i):
            if sig == "out":
                return pins["O"]
            if sig in pins:
                return pins[sig]
            return "g%d.%s" % (i, sig)
        ports = cell.frag.stamp(b, labels, dx, dy, dz, rename=rn)
        inst_ports.append(ports)
        for (kx, ky, kz) in cell.keepout:
            keep_marks.append((kx + dx, ky + dy, kz + dz))
            # a torch strong-powers the block above it: a route support laid
            # there would re-emit 15 into ITS neighbours -- claim the column
            keep_marks.append((kx + dx, ky + dy + 1, kz + dz))
        # Reserved approach stubs: one dust west of every input port, one
        # east of the output port.  A stub carries the net label, so no
        # foreign trunk may come within electrical reach of the only entry
        # to a port pocket (the seal that killed the first seg7 attempt).
        for pin in ("a", "b"):
            if pin not in pins:
                continue
            px, py, pz = ports[pin]
            # A reserved, SELF-REFRESHING approach corridor: mouth dust ->
            # repeater -> 3 dust -> port.  1-cell stubs let a sibling port's
            # approach fence the pocket; a route may legally deliver ss1 at
            # the mouth, so the repeater must read the mouth DIRECTLY; and a
            # 5-deep corridor pushes the contested mouth into open channel.
            mouth = (px - 5, py, pz)
            for k in range(1, 6):
                b.stone(px - k, py - 1, pz)
            b.put(*mouth, rs.DUST)
            labels[mouth] = pins[pin]
            b.put(px - 4, py, pz, rs.repeater("west"))
            corridor = []
            for k in (3, 2, 1):
                s = (px - k, py, pz)
                b.put(*s, rs.DUST)
                labels[s] = pins[pin]
                r_ss_seed.append((s, 15 - (3 - k)))
                corridor.append(s)
            cellset = [mouth] + corridor + [ports[pin]]
            for p, lab in cell.frag.labels.items():
                if lab == pin:
                    cellset.append((p[0] + dx, p[1] + dy, p[2] + dz))
            sink_cells[(i, pin)] = cellset
        ox, oy, oz = ports["out"]
        ostubs = []
        for k in (1, 2):
            s = (ox + k, oy, oz)
            b.stone(ox + k, oy - 1, oz)
            b.put(*s, rs.DUST)
            labels[s] = pins["O"]
            ostubs.append(s)
        conn[pins["O"]] = ostubs + [
            (p[0] + dx, p[1] + dy, p[2] + dz)
            for p, lab in cell.frag.labels.items() if lab == "out"]

    (x0, y0, z0), (x1, y1, z1) = b.bounds()
    r = CountingRouter(b, labels,
                       bounds=(x0 - 2, x1 + 6, 1, ROUTE_Y_MAX, z0 - 4, z1 + 6))
    for p in keep_marks:
        r.strong.setdefault(p, KEEP)
    for p, n in lever_marks:
        r.strong.setdefault(p, n)
    # hard cell obstruction boxes: the stamped fragment + its corridors and
    # out stubs, plus margin, claimed for the instance's own nets only
    for i, (kind, pins) in enumerate(order):
        cell = lib[kind]
        dx, dy, dz = placements[i]
        xs = [p[0] for p in cell.frag.cells]
        zs = [p[2] for p in cell.frag.cells]
        own = frozenset(pins.values())
        for bx in range(dx + min(xs) - 6, dx + max(xs) + 4):
            for bz in range(dz + min(zs) - 2, dz + max(zs) + 3):
                for by in (1, 2, 3):
                    q = (bx, by, bz)
                    prev = r.cellbox.get(q)
                    r.cellbox[q] = own if prev is None else (prev | own)
    for n, (x, y, z) in pi_pos.items():
        for bx in range(x - 3, x + 2):
            for bz in range(z - 2, z + 3):
                for by in (1, 2, 3):
                    q = (bx, by, bz)
                    prev = r.cellbox.get(q)
                    one = frozenset([n])
                    r.cellbox[q] = one if prev is None else (prev | one)
    # seed trunk strengths: PI runways sit beside their lever (15), a cell's
    # out line is within a few dust of its comparator/torch/repeater source
    # (the deepest is XOR2's merge, ~6 cells) -- conservative values so the
    # branch filter and emit's resumed budget stay honest
    for n, (x, y, z) in pi_pos.items():
        r.ss[(x - 1, y, z)] = 13
        r.ss[(x, y, z)] = 12
    for net, cs in conn.items():
        if net in pi_pos:
            continue
        for p in cs:
            r.ss.setdefault(p, 9)
    for p, v in r_ss_seed:
        r.ss[p] = v
    # soft pocket halo: a foreign trunk that merely PASSES a port approach is
    # fine, but parking beside one seals it (d[3] did exactly that to a
    # column-2 OR2).  Charge foreign nets for every cell near a stub.
    for (i, pin), cs in sink_cells.items():
        sxp, syp, szp = cs[0]
        net = order[i][1][pin]
        for ddx in (-3, -2, -1, 0, 1):
            for ddz in (-2, -1, 0, 1, 2):
                for ddy in (0, 1):
                    r.soft.setdefault((sxp + ddx, syp + ddy, szp + ddz),
                                      (net, 8))

    # net -> driver port position (the probe cell for POs)
    driver = dict(pi_pos)
    for i, (kind, pins) in enumerate(order):
        driver[pins["O"]] = inst_ports[i]["out"]

    # Sinks per net, routed NET-MAJOR in driver-topo order (PIs first): each
    # net lays one clean trunk while space is empty rather than sprawling
    # around everyone else's later branches.
    net_sinks = {}
    for i, (kind, pins) in enumerate(order):
        for pin in ("a", "b"):
            if pin in pins:
                net_sinks.setdefault(pins[pin], []).append((i, pin))
    net_order = [n for n in nl.inputs if n in net_sinks]
    net_order += [pins["O"] for _k, pins in order if pins["O"] in net_sinks]

    # Route each sink FROM the net's connected component (any trunk dust may
    # source a branch; a sealed driver port stops mattering), TO the sink's
    # stub-or-port.  Count refresh repeaters per route for the wire STA.
    wire_rt = {}           # (net, inst) -> repeaters+torches on that route
    wire_path = {}         # (net, inst) -> emitted path cells (debug/STA)
    for net in net_order:
        d0 = driver[net]
        sinks = sorted(net_sinks[net],
                       key=lambda ip: abs(sink_cells[ip][0][0] - d0[0])
                       + abs(sink_cells[ip][0][2] - d0[2]))
        for (i, pin) in sinks:
            sink = sink_cells[(i, pin)]
            # branch only from fresh-enough trunk dust: a corner laid off an
            # ss1 cell is dead before it reaches its first refresh
            srcs = [p for p in conn[net] if r.ss.get(p, 0) >= 4]
            r.route(srcs, sink[:1], net)
            wire_rt[(net, i)] = r.last_rt
            wire_path[(net, i)] = [p for p, _mv in r.last_path]
            conn[net].extend(p for p, _mv in r.last_path
                             if labels.get(p) == net)
            conn[net].extend(sink)
    return b, labels, inst_ports, driver, lever_at, wire_rt, wire_path


def structural_sta(nl, lib, order, wire_rt):
    """Arrival per net in rt: cell delay + per-route refresh delays."""
    arrival = {n: 0 for n in nl.inputs}
    for i, (kind, pins) in enumerate(order):
        cell = lib[kind]
        arr = 0
        for pin in ("a", "b"):
            if pin not in pins:
                continue
            net = pins[pin]
            arr = max(arr, arrival[net] + wire_rt.get((net, i), 0)
                      + cell.delay_rt[pin])
        arrival[pins["O"]] = arr
    worst = max(nl.outputs, key=lambda n: arrival[n])
    return arrival, worst


# ---------------------------------------------------------------------------
# verification + measurement
# ---------------------------------------------------------------------------

def verify_design(name, nl, lib, order, b, labels, driver, lever_at):
    problems = audit.audit(b.cells)
    shorts = nets.check(b.cells, labels)
    for kind, items in problems.items():
        if items:
            print("STRUCTURAL %s x%d e.g. %s" % (kind, len(items), items[0]))
    print("net check: %d shorts" % len(shorts))
    if any(problems.values()) or shorts:
        for s in shorts[:6]:
            print("   ", s)
        return None
    sim = b.sim()
    lv = rs.Levers(sim, [lever_at[n] for n in nl.inputs])
    npi = len(nl.inputs)
    bad = 0
    for case in range(1 << npi):
        bits = [(case >> k) & 1 for k in range(npi)]
        lv.set(bits)
        want = eval_netlist(nl, order, dict(zip(nl.inputs, bits)))
        got = {po: int(sim.on(*driver[po])) for po in nl.outputs}
        wrong = [po for po in nl.outputs if got[po] != want[po]]
        if wrong:
            bad += 1
            if bad <= 8:
                print("   WRONG case %s: %s"
                      % (bits, [(po, got[po], want[po]) for po in wrong]))
    total = 1 << npi
    print("%s exhaustive: %d/%d correct" % (name, total - bad, total))
    return sim if bad == 0 else None


def measure(tag, b):
    (x0, y0, z0), (x1, y1, z1) = b.bounds()
    dims = (x1 - x0 + 1, y1 - y0 + 1, z1 - z0 + 1)
    vol = dims[0] * dims[1] * dims[2]
    print("%s: %d blocks, %dx%dx%d (vol %d, fill %.0f%%)"
          % (tag, len(b.cells), dims[0], dims[1], dims[2], vol,
             100.0 * len(b.cells) / vol))
    return len(b.cells), dims


def run_design(name, out_path):
    os.makedirs(BUILD_DIR, exist_ok=True)
    lib = build_library()
    genlib = os.path.join(BUILD_DIR, "cells.genlib")
    write_genlib(lib, genlib)
    verilog = os.path.join(HERE, "hdl", name + ".v")
    blif = os.path.join(BUILD_DIR, name + "_map.blif")
    run_yosys(verilog, genlib, blif)
    nl = parse_blif(blif)
    order = topo_gates(nl)
    from collections import Counter
    hist = Counter(k for k, _ in order)
    cell_area = sum(lib[k].area for k, _ in order)
    print("%s mapped: %d gates %s, cell area %d blocks"
          % (name, len(order), dict(sorted(hist.items())), cell_area))

    b, labels, inst_ports, driver, lever_at, wire_rt, wire_path = \
        compose(name, nl, lib, order)
    arrival, worst = structural_sta(nl, lib, order, wire_rt)
    print("STA: critical path -> %s = %d rt" % (worst, arrival[worst]))
    blocks, dims = measure("genlib_" + name, b)

    sim = verify_design(name, nl, lib, order, b, labels, driver, lever_at)
    if sim is None:
        return 1
    if out_path:
        lv = rs.Levers(sim, [lever_at[n] for n in nl.inputs])
        lv.set([0] * len(nl.inputs))
        import build_ppa as bp
        print("baked %d states" % bp.bake(b, sim))
        b.s.save_to_file(out_path)
        print("saved", out_path)
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cells", action="store_true")
    ap.add_argument("--design", choices=["seg7", "cmp4", "popcnt4"])
    ap.add_argument("--out", default="")
    args = ap.parse_args()
    if args.cells:
        lib = build_library()
        return 0 if verify_library(lib) else 1
    if args.design:
        return run_design(args.design, args.out)
    ap.print_help()
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
