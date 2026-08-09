"""Compositor MVP: place verified cell fragments, connect by abutment or
routing, then run the full analysis stack (audit / alias-aware nets.check /
bridge DRC / bridge LVS) and hand back mc-tick verification hooks.

This is the Python prototype of ROUTING_CRATE_DESIGN.md's product #2:
"place existing circuits, auto-connect matching ports by abutment or routed
bus, then analyse and optimise".  API:

    c = Compositor("acc4")
    fa0 = c.add("fa0", fa_frag, (0, 0, 0))
    fa1 = c.add("fa1", fa_frag, (0, 0, 13))
    c.connect(fa0.ref("cout"), fa1.ref("cin"))   # adjacent -> abutment alias
    c.connect(a.ref("x"), b.ref("y"))            # far apart -> queued net
    c.route()          # python router (labels stay sim-checkable)
    c.route_bridge()   # or Rust route_all (negotiated congestion)
    c.check()          # audit + alias-aware nets.check
    sim = c.sim(); c.bake("out.schem"); c.drc(); c.lvs()
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))       # redstone-eda

import nucleation as n            # noqa: E402
import audit as audit_mod         # noqa: E402
import nets as nets_mod           # noqa: E402
import router as router_mod       # noqa: E402
import rs                         # noqa: E402
from seq_probe import bake_states, reload_sim, ticks_to_quiescent  # noqa: E402,F401


class Instance:
    """A placed fragment: absolute ports + a bbox hull for placement work."""

    def __init__(self, name, frag, at, ports):
        self.name, self.frag, self.at, self.ports = name, frag, at, ports

    def ref(self, port):
        return (self, port)

    def bbox(self):
        xs = [p[0] for p in self.frag.cells]
        ys = [p[1] for p in self.frag.cells]
        zs = [p[2] for p in self.frag.cells]
        dx, dy, dz = self.at
        return ((min(xs) + dx, min(ys) + dy, min(zs) + dz),
                (max(xs) + dx, max(ys) + dy, max(zs) + dz))


class Compositor:
    def __init__(self, name):
        self.b = rs.Build(name)
        self.labels = {}
        self.aliases = []
        self.insts = {}
        self.pending = []          # (src_pos, dst_pos, src_label, dst_label)

    # ---------------------------------------------------------- placement
    def add(self, name, frag, at, shared=()):
        """Stamp `frag` at `at`; labels get an `inst.` prefix except `shared`
        ones (e.g. a clock net that must stay one net across instances)."""
        rn = lambda s: s if s in shared else "%s.%s" % (name, s)
        ports = frag.stamp(self.b, self.labels, at[0], at[1], at[2], rename=rn)
        inst = Instance(name, frag, at, ports)
        self.insts[name] = inst
        return inst

    def _label_of(self, ref):
        inst, port = ref
        pos = inst.ports[port]
        return self.labels.get(pos) or "%s.%s" % (inst.name, port)

    def connect(self, a, b, src_off=None, dst_off=None):
        """Abutment when the two port cells touch; otherwise queue a net.
        `src_off`/`dst_off` shift the routing endpoints off the (occupied)
        port cells onto the adjacent AIR cell of the port's face -- the
        router lays dust into every path cell, so endpoints must be free;
        the laid endpoint dust connects to the port dust by adjacency."""
        pa, pb = a[0].ports[a[1]], b[0].ports[b[1]]
        la, lb = self._label_of(a), self._label_of(b)
        d = sum(abs(pa[i] - pb[i]) for i in range(3))
        if d <= 1:
            self.alias(la, lb)
            return "abut"
        if src_off:
            pa = tuple(pa[i] + src_off[i] for i in range(3))
        if dst_off:
            pb = tuple(pb[i] + dst_off[i] for i in range(3))
        self.pending.append((pa, pb, la, lb))
        return "net"

    def alias(self, la, lb):
        if la != lb:
            self.aliases.append((la, lb))

    # ------------------------------------------------- manual wiring escape
    def dust(self, x, y, z, net, role="route", floor=True):
        if floor:
            self.b.stone(x, y - 1, z, role)
        self.b.put(x, y, z, rs.DUST)
        self.labels[(x, y, z)] = net

    def rep(self, x, y, z, input_from):
        self.b.stone(x, y - 1, z, "gate")
        self.b.put(x, y, z, rs.repeater(input_from))

    def guard(self, x, y, z):
        """Cap / diagonal-severing lid block."""
        self.b.put(x, y, z, rs.PALETTE["lid"])

    # ------------------------------------------------------------- routing
    def route(self, bounds=None):
        """Python maze router: emitted cells keep labels (sim-checkable)."""
        r = router_mod.Router(self.b, self.labels, bounds=bounds)
        for pa, pb, la, lb in self.pending:
            r.route(pa, pb, la)
            self.alias(la, lb)
        self.pending = []

    def route_bridge(self, bounds=None, classes=None, budget=None,
                     congestion=None):
        """Rust route_all: negotiated congestion over one labelled workspace.
        Emitted geometry is copied back into build cells + labels."""
        nets_json = {"nets": [
            {"label": la, "src": list(pa), "dsts": [list(pb)]}
            for pa, pb, la, lb in self.pending]}
        if bounds:
            nets_json["bounds"] = [list(bounds[0]), list(bounds[1])]
        if classes:
            nets_json["classes"] = classes
        if budget:
            nets_json["budget"] = budget
        if congestion:
            nets_json["congestion"] = congestion
        out = n.Routing.route_all(self.b.s, json.dumps(nets_json))
        rep = json.loads(out)
        blocks = {(bl["x"], bl["y"], bl["z"]): bl for bl in
                  json.loads(self.b.s.get_all_blocks_json())
                  if bl["name"] != "minecraft:air"}
        for route in rep.get("routes", []):
            for cell in route.get("path", []):
                p = tuple(cell)
                if p not in self.b.cells and p in blocks:
                    bl = blocks[p]
                    props = bl.get("properties") or {}
                    if isinstance(props, dict):
                        items = sorted(props.items())
                    else:            # list of [key, value] pairs
                        items = sorted((kv[0], kv[1]) for kv in props)
                    ps = ",".join("%s=%s" % kv for kv in items)
                    self.b.cells[p] = bl["name"] + ("[%s]" % ps if ps else "")
                q = (p[0], p[1] - 1, p[2])          # router-laid support
                if q not in self.b.cells and q in blocks:
                    self.b.cells[q] = blocks[q]["name"]
                if "redstone_wire" in self.b.cells.get(p, ""):
                    self.labels.setdefault(p, route["label"])
        for pa, pb, la, lb in self.pending:
            self.alias(la, lb)
        self.pending = []
        return rep

    def rip(self, cells_list):
        """Remove previously routed cells (dust/diodes) -- rip-up for repair."""
        for p in cells_list:
            blk = self.b.cells.get(p, "")
            if "redstone_wire" in blk or "repeater" in blk or "torch" in blk:
                del self.b.cells[p]
                self.labels.pop(p, None)
                self.b.s.set_block_from_string(p[0], p[1], p[2], "minecraft:air")

    # ------------------------------------------------------------ analysis
    def check(self, verbose=True):
        problems = audit_mod.audit(self.b.cells)
        shorts = nets_mod.check(self.b.cells, self.labels, self.aliases)
        nprob = sum(len(v) for v in problems.values())
        if verbose:
            for kind, items in problems.items():
                if items:
                    print("   STRUCTURAL %s x%d e.g. %s" % (kind, len(items), items[0]))
            for s in shorts[:6]:
                print("   SHORT", s)
        return nprob == 0 and not shorts, problems, shorts

    def drc(self, check_decay=False):
        """Bridge DRC over the composed schematic -> list of violations."""
        return json.loads(n.Routing.drc(self.b.s, check_decay))

    def lvs(self):
        """Bridge LVS: intent netlist from labels (alias-aware roots) vs the
        conduction netlist extracted from the schematic."""
        root = {}

        def find(a):
            root.setdefault(a, a)
            while root[a] != a:
                root[a] = root[root[a]]
                a = root[a]
            return a

        for a, b in self.aliases:
            root[find(a)] = find(b)
        by_net = {}
        for pos, lab in self.labels.items():
            if "redstone_wire" in self.b.cells.get(pos, ""):
                by_net.setdefault(find(lab), []).append(list(pos))
        intent = {"nets": [{"name": k, "terminals": v}
                           for k, v in sorted(by_net.items())]}
        return json.loads(n.Routing.lvs(self.b.s, json.dumps(intent)))

    def sta(self, netlist):
        return json.loads(n.Routing.sta(self.b.s, json.dumps(netlist)))

    # -------------------------------------------------------- verification
    def sim(self, settle=400):
        return self.b.sim(settle=settle)

    def bake(self, sim, path=None):
        """Bake the settled state (FPGA-bitstream style initial state)."""
        baked = bake_states(self.b, sim)
        if path:
            baked.save_to_file(path)
        return baked
