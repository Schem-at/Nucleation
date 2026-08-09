#!/usr/bin/env python3
"""Verilog -> yosys -> BLIF -> PLA stages -> sim-verified redstone .schem.

The whole flow reuses the verified PLA compiler in ../build_ppa.py; this file
only has to turn an arbitrary combinational BLIF into the stage-dict shape
that compiler already knows how to place, route, audit and simulate:

  yosys        read_verilog; synth -lut 4; write_blif   (.names truth tables)
  parse        fold constants, truth-table every .names node
  polarity     a literal used complemented becomes its own dual-rail value:
               for node f, `~f` is a SECOND PLA node computing the off-set
               cover (Quine-McCluskey minimised) over the same inputs --
               inverter torches exist only in the input stage, everything
               downstream is pure positive-rail AND/OR columns
  levelise     every prim gets level = 1 + max(level of inputs); buffer nodes
               are inserted so every net crosses exactly ONE stage boundary
               (all routes are next-stage hops on channels 0/1: no far-route
               corridor conflicts by construction)
  place        nodes are packed 2/slice (gi0 <=2 terms + gi1 1 term) where
               possible; rails only conduct west->east, so a node's slice is
               forced >= the slice of every producer it taps
  build        bp.PPA(make=...) -> audit + net-short check -> mc-tick sim,
               exhaustive vs a pure-Python eval of the same prim graph ->
               bake at rest -> .schem

Sequential (.latch, rising-edge, single clock) compiles through the RUST
pipeline only (--rust): nucleation.Hdl places a verified master-slave
repeater-lock DFF bank + clock spine + feedback corridors, and this file's
independent clocked model (raw BLIF truth tables + latch state) drives the
fixed-tick verification protocol.  The pure-Python geometry path stays
combinational-only; .subckt / .gate are rejected everywhere.
"""
import argparse
import itertools
import os
import random
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))

import build_ppa as bp
import rs
import audit
import nets


# ---- yosys ----------------------------------------------------------------
def run_yosys(verilog, top, blif):
    # dffunmap legalises enable/reset DFF cells ($_DFFE_PP_ ...) into plain
    # $_DFF_P_ + muxes (write_blif only emits .latch for plain DFFs); the
    # second abc maps those muxes back into LUTs.  No-ops combinationally.
    script = ("read_verilog %s; hierarchy -check -top %s; "
              "synth -top %s -lut 4; dffunmap; abc -lut 4; opt_clean; "
              "write_blif %s"
              % (verilog, top, top, blif))
    r = subprocess.run(["yosys", "-q", "-p", script],
                       capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(r.stdout[-2000:] + r.stderr[-2000:])
        raise RuntimeError("yosys failed on %s" % verilog)
    return blif


# ---- BLIF -----------------------------------------------------------------
class Unsupported(Exception):
    pass


def parse_blif(path):
    lines, pend = [], ""
    for raw in open(path):
        raw = raw.split("#", 1)[0].rstrip()
        if raw.endswith("\\"):
            pend += raw[:-1] + " "
            continue
        line = (pend + raw).strip()
        pend = ""
        if line:
            lines.append(line)

    inputs, outputs, raw_nodes, latches = [], [], {}, []
    cur = None
    for line in lines:
        tok = line.split()
        if tok[0] == ".model" or tok[0] == ".end":
            cur = None
        elif tok[0] == ".inputs":
            inputs += tok[1:]
            cur = None
        elif tok[0] == ".outputs":
            outputs += tok[1:]
            cur = None
        elif tok[0] == ".names":
            cur = tok[-1]
            raw_nodes[cur] = (tok[1:-1], [])
        elif tok[0] == ".latch":
            # .latch <input> <output> [<type> <control>] [<init>]
            a = tok[1:]
            if len(a) == 2:
                d, q, ttype, ctrl, init = a[0], a[1], None, None, "3"
            elif len(a) == 3:
                d, q, ttype, ctrl, init = a[0], a[1], None, None, a[2]
            elif len(a) == 4:
                d, q, ttype, ctrl, init = a[0], a[1], a[2], a[3], "3"
            elif len(a) == 5:
                d, q, ttype, ctrl, init = a[0], a[1], a[2], a[3], a[4]
            else:
                raise Unsupported(".latch expects 2-5 arguments")
            if ttype not in (None, "re"):
                raise Unsupported("only rising-edge latches: got %s" % ttype)
            latches.append(dict(d=d, q=q, control=ctrl,
                                init=1 if init == "1" else 0))
            cur = None
        elif tok[0] in (".subckt", ".gate", ".mlatch"):
            raise Unsupported("flat .names/.latch only: found %s" % tok[0])
        elif tok[0].startswith("."):
            cur = None                       # .default_input_arrival etc
        elif cur is not None:
            if len(tok) == 1:                # 0-input node: row is just the out bit
                raw_nodes[cur][1].append(("", tok[0]))
            else:
                raw_nodes[cur][1].append((tok[0], tok[1]))
    return inputs, outputs, raw_nodes, latches


def tt_of(ins, rows):
    """Truth table as a bitmask over 2**k input assignments; bit i of the
    assignment index is the value of ins[i]."""
    k = len(ins)
    if not rows:
        return 0, k
    outs = {o for _p, o in rows}
    assert len(outs) == 1, "mixed-polarity .names cover"
    onset = outs == {"1"}
    tt = 0
    for m in range(1 << k):
        for pat, _o in rows:
            ok = all(c == "-" or int(c) == ((m >> i) & 1)
                     for i, c in enumerate(pat))
            if ok:
                tt |= 1 << m
                break
    if not onset:
        tt = ((1 << (1 << k)) - 1) ^ tt
    return tt, k


def fold(inputs, raw_nodes):
    """Topo-order nodes, collapse duplicate inputs, propagate constants."""
    order, seen = [], {}

    def visit(n):
        if seen.get(n) == 2:
            return
        assert seen.get(n) != 1, "combinational loop through %s" % n
        seen[n] = 1
        for s in raw_nodes[n][0]:
            if s in raw_nodes:
                visit(s)
        seen[n] = 2
        order.append(n)

    for n in raw_nodes:
        visit(n)

    pis = set(inputs)
    consts, nodes = {}, {}
    for n in order:
        ins, rows = raw_nodes[n]
        for s in ins:
            if s not in pis and s not in raw_nodes:
                raise Unsupported("undriven net %s feeding %s" % (s, n))
        tt, k = tt_of(ins, rows)
        if len(set(ins)) < len(ins):         # collapse duplicate inputs
            uniq = list(dict.fromkeys(ins))
            pos = [uniq.index(s) for s in ins]
            ntt = 0
            for m in range(1 << len(uniq)):
                full = sum((((m >> pos[i]) & 1) << i) for i in range(k))
                if (tt >> full) & 1:
                    ntt |= 1 << m
            ins, tt, k = uniq, ntt, len(uniq)
        free = [i for i, s in enumerate(ins) if s not in consts]
        if len(free) < k:                    # restrict away constant inputs
            base = sum(consts[ins[i]] << i for i in range(k) if ins[i] in consts)
            ntt = 0
            for m in range(1 << len(free)):
                full = base + sum(((m >> j) & 1) << free[j] for j in range(len(free)))
                if (tt >> full) & 1:
                    ntt |= 1 << m
            ins = [ins[i] for i in free]
            tt, k = ntt, len(free)
        if k == 0:
            consts[n] = tt & 1
        elif tt == 0:
            consts[n] = 0
        elif tt == (1 << (1 << k)) - 1:
            consts[n] = 1
        else:
            nodes[n] = (ins, tt, k)
    return nodes, consts


# ---- Quine-McCluskey ------------------------------------------------------
def qm(minterms, k):
    """Minimal-ish SOP cover of `minterms` over k variables.
    Returns [[(bit, polarity), ...], ...]; deterministic."""
    cur = {(m, 0) for m in minterms}
    primes = set()
    while cur:
        nxt, merged = set(), set()
        by = {}
        for v, mask in cur:
            by.setdefault(mask, set()).add(v)
        for mask, vs in by.items():
            for v in vs:
                for b in range(k):
                    if mask & (1 << b) or (v >> b) & 1:
                        continue
                    if (v | (1 << b)) in vs:
                        nxt.add((v, mask | (1 << b)))
                        merged.add((v, mask))
                        merged.add((v | (1 << b), mask))
        primes |= cur - merged
        cur = nxt

    def covers(p, m):
        v, mask = p
        return (m & ~mask) == (v & ~mask)

    remaining, chosen = set(minterms), []
    plist = sorted(primes, key=lambda p: (p[1], p[0]))
    while remaining:
        best = max(plist, key=lambda p: (sum(covers(p, m) for m in remaining),
                                         bin(p[1]).count("1"), -p[0], -p[1]))
        chosen.append(best)
        remaining -= {m for m in remaining if covers(best, m)}
        plist.remove(best)
    return [[(b, (v >> b) & 1) for b in range(k) if not (mask & (1 << b))]
            for v, mask in chosen]


# ---- BLIF -> prim graph ---------------------------------------------------
MAX_TERMS = 3           # a lone gi=0 node owns COL_A+COL_B = 3 columns


class Compiler:
    def __init__(self, inputs, outputs, nodes, consts):
        self.pis, self.pos_ = inputs, outputs
        self.nodes, self.consts = nodes, consts
        self.base = self._names(list(inputs) + list(nodes) + list(consts))
        self.val_terms = {}                  # vid -> [[vid, ...], ...]
        self.level = {}
        self.memo = {}
        self.pi_slice = {}
        for i, net in enumerate(inputs):
            for pol in (0, 1):
                vid = self.vid(net, pol)
                self.level[vid] = 0
                self.pi_slice[vid] = i

    @staticmethod
    def _names(nets):
        used, base = set(), {}
        for net in nets:
            b = re.sub(r"[^A-Za-z0-9_]", "_", net).strip("_") or "n"
            if b[0].isdigit():
                b = "n" + b
            c, i = b, 2
            while c in used:
                c, i = "%s_%d" % (b, i), i + 1
            used.add(c)
            base[net] = c
        return base

    def vid(self, net, pol):
        return self.base[net] + ("" if pol else ".n")

    # -- value construction (dual rail, on demand) --------------------------
    def value(self, net, pol):
        key = (net, pol)
        if key in self.memo:
            return self.memo[key]
        if net in self.consts:
            r = ("const", self.consts[net] if pol else 1 - self.consts[net])
            self.memo[key] = r
            return r
        if net in self.pi_slice or (net in self.pis):
            r = self.vid(net, pol)
            self.memo[key] = r
            return r
        ins, tt, k = self.nodes[net]
        want = [m for m in range(1 << k) if ((tt >> m) & 1) == pol]
        cover = qm(want, k)
        vid = self.vid(net, pol)
        self.memo[key] = vid
        terms = []
        for term in cover:
            lits = sorted({self.value(ins[b], p) for b, p in term})
            assert all(isinstance(u, str) for u in lits)
            terms.append(lits)
        self.emit(vid, terms)
        return vid

    def emit(self, vid, terms, depth=0):
        if len(terms) <= MAX_TERMS:
            self.val_terms[vid] = terms
            return
        parts = []
        for i in range(0, len(terms), MAX_TERMS):
            pv = "%s.d%dp%d" % (vid, depth, i // MAX_TERMS)
            self.val_terms[pv] = terms[i:i + MAX_TERMS]
            parts.append(pv)
        self.emit(vid, [[p] for p in parts], depth + 1)

    # -- peephole: collapse buffers / double inversions ---------------------
    def peephole(self):
        """Alias away pure buffer nodes: v = OR-of-AND over a single literal
        is just that literal.  Only this provably-equivalent rewrite is done.

        This is where double inversions die: dual-rail construction turns a
        BLIF NOT into a buffer of the complement rail (the on-set of ~x as a
        function of x's rails IS x.n), so NOT(NOT(x)) chains arrive here as
        buffer chains and collapse to nothing.  It is also the polarity-aware
        selection: a consumer of a collapsed chain taps the already-available
        rail directly instead of re-inverting.  Each removed node saves a
        full PLA column (tap torch + gate torch = 2 rt) plus its rail.

        Must run BEFORE levelise(): the buffers levelise inserts enforce the
        single-hop routing discipline and are structural, not redundant.
        """
        self._alias = {}
        for v, terms in self.val_terms.items():
            if len(terms) == 1 and len(terms[0]) == 1:
                self._alias[v] = terms[0][0]
        for v in self._alias:
            del self.val_terms[v]
        for v, terms in self.val_terms.items():
            self.val_terms[v] = [sorted({self.resolve(u) for u in t})
                                 for t in terms]
        return len(self._alias)

    def resolve(self, v):
        """Follow buffer aliases to the surviving vid."""
        while v in getattr(self, "_alias", {}):
            v = self._alias[v]
        return v

    # -- levels + single-hop buffering --------------------------------------
    def levelise(self):
        def lv(v):
            if v not in self.level:
                self.level[v] = 1 + max(lv(u) for t in self.val_terms[v] for u in t)
            return self.level[v]

        for v in list(self.val_terms):
            lv(v)
        need = {}                            # vid -> highest level a buffer is needed at
        for v, terms in self.val_terms.items():
            for t in terms:
                for u in t:
                    if self.level[u] < self.level[v] - 1:
                        need[u] = max(need.get(u, 0), self.level[v] - 1)
        for u, top in sorted(need.items()):
            prev = u
            for l in range(self.level[u] + 1, top + 1):
                b = "%s.b%d" % (u, l)
                self.val_terms[b] = [[prev]]
                self.level[b] = l
                prev = b
        for v, terms in self.val_terms.items():
            self.val_terms[v] = [
                [u if self.level[u] == self.level[v] - 1 else
                 "%s.b%d" % (u, self.level[v] - 1) for u in t] for t in terms]

    # -- slices + stage dicts ------------------------------------------------
    def stages(self):
        slice_of = dict(self.pi_slice)
        by_level = {}
        for v in self.val_terms:
            by_level.setdefault(self.level[v], []).append(v)

        stages = [self.input_stage()]
        for L in range(1, max(by_level) + 1 if by_level else 1):
            todo = by_level.get(L, [])
            minreq = {v: max([slice_of[u] for t in self.val_terms[v] for u in t] or [0])
                      for v in todo}
            todo.sort(key=lambda v: (minreq[v], -len(self.val_terms[v]), v))
            nodes, cur = [], 0
            while todo:
                a = todo.pop(0)
                sl = max(minreq[a], cur)
                na = len(self.val_terms[a])
                partner = None
                if na <= 2:                  # a rides gi0, find a 1-term gi1
                    cands = [b for b in todo
                             if len(self.val_terms[b]) == 1 and minreq[b] <= sl]
                    if cands:
                        partner = max(cands, key=lambda b: (minreq[b], b))
                        todo.remove(partner)
                group = [(a, self.val_terms[a])]
                if partner:
                    group.append((partner, self.val_terms[partner]))
                for v, _t in group:
                    slice_of[v] = sl
                nodes.append((sl, group))
                cur = sl + 1
            stages.append(dict(name="lv%d" % L, nodes=nodes, local=False,
                               inverters=[], levers=[]))
        return stages

    def input_stage(self):
        nodes, invs, levers = [], [], []
        for i, net in enumerate(self.pis):
            p, n = self.vid(net, 1), self.vid(net, 0)
            rail, railn = p + ".lv", p + ".lvn"
            nodes.append((i, [(p, [[rail]]), (n, [[railn]])]))
            invs.append((i, rail, railn))
            levers.append((i, rail, 0))
        return dict(name="in", nodes=nodes, local=True,
                    inverters=invs, levers=levers)

    # -- reference model -----------------------------------------------------
    def eval(self, bits):
        val = {}
        for i, net in enumerate(self.pis):
            p = self.vid(net, 1)
            val[p], val[p + ".n"] = bits[i], 1 - bits[i]
            val[p + ".lv"], val[p + ".lvn"] = bits[i], 1 - bits[i]
        for v in sorted(self.val_terms, key=lambda v: (self.level[v], v)):
            val[v] = int(any(all(val[u] for u in t) for t in self.val_terms[v]))
        return val


# ---- independent clocked reference (raw truth tables + latch state) -------
def eval_raw(raw_nodes, env):
    """Evaluate the raw BLIF covers over env (PIs + latch outputs); shares
    no code with either compiler."""
    val = dict(env)
    progress = True
    while progress:
        progress = False
        for name, (ins, rows) in raw_nodes.items():
            if name in val or not all(i in val for i in ins):
                continue
            if not rows:
                v = 0
            else:
                onset = rows[0][1] == "1"
                hit = any(all(c == "-" or int(c) == val[ins[i]]
                              for i, c in enumerate(pat))
                          for pat, _o in rows)
                v = int(hit == onset)
            val[name] = v
            progress = True
    return val


# ---- Rust sequential path --------------------------------------------------
def run_rust_seq(args, blif_path, inputs, outputs, raw_nodes, latches):
    """Sequential parity: the Rust pipeline compiles the .latch design (DFF
    bank + clock spine + wrap corridors, initial state baked); this file's
    independent clocked model drives the fixed-tick protocol — set input
    levers, wait (measured) for the D ports, pulse the clock lever, compare
    every Q rail and output probe against the stepped model.  Never
    run_until_quiescent after the placement settle."""
    import json
    import nucleation as n

    text = open(blif_path).read()
    report = json.loads(n.Hdl.compile_blif_report(text, args.top))
    clock = report["clock"]
    real_pis = report["inputs"]              # .inputs minus the clock net
    print("rust: %d blocks, %d latches, clock %s (est min period %d gt)"
          % (report["blocks"], len(report["latches"]), clock["net"],
             clock["est_min_period_gt"]))
    b = n.Hdl.compile_blif(text, args.top, False)
    if args.no_sim:
        if args.out:
            b.save_to_file(args.out)
            print("saved (rust, unbaked)", args.out)
        return 0

    tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_hdl_rust_seq.schem")
    b.save_to_file(tmp)
    tight = n.Schematic.open(tmp)
    sim = n.TickSimulation.from_schematic(tight, n.TickSettleMode.Placement,
                                          0, 0, 0, rs.EXTRA_STATES)
    sim.run_until_quiescent(4000)            # reset-by-bake: at-rest settle
    s = rs.Sim(sim, tuple(report["bounds"][0]))
    print("rust: placed; at-rest quiescent:", s.sim.is_quiescent())
    probes = report["probes"]
    po_report = {o["name"]: o for o in report["outputs"]}
    lv = rs.Levers(s, [tuple(l["pos"]) for l in report["levers"]])
    clk_lever = tuple(clock["lever"])

    state = [l["init"] for l in latches]
    rl = report["latches"]

    def model(bits):
        env = {net: bits[i] for i, net in enumerate(real_pis)}
        for l, st in zip(latches, state):
            env[l["q"]] = st
        return eval_raw(raw_nodes, env)

    def q_and_pos_match(val):
        for l, rlat, st in zip(latches, rl, state):
            if int(s.on(*probes[rlat["q_rail"]])) != st:
                return False
        for po in outputs:
            ro = po_report[po]
            if "probe" in ro and int(s.on(*probes[ro["probe"]])) != val[po]:
                return False
        return True

    # init: the baked state must already be standing
    ok = True
    val0 = model([0] * len(real_pis))
    if not q_and_pos_match(val0):
        print("   INIT WRONG: baked state not standing after settle")
        ok = False

    rnd = random.Random(args.seed)
    steps = args.steps
    cap, high = 800, 40
    max_setup = max_edge = 0
    for si in range(steps):
        bits = [rnd.getrandbits(1) for _ in real_pis]
        for i, want in enumerate(bits):      # fixed-tick lever discipline
            if lv.state[i] != bool(want):
                s.use(*lv.positions[i])
                lv.state[i] = bool(want)
                run_gt(s, 2)
        val = model(bits)
        t = 0                                # measured input-settle margin
        while t < cap:
            good = all(("d_vid" not in rlat)  # const D is tied at the cell
                       or int(s.on(*probes[rlat["d_probe"]])) == val[l["d"]]
                       for l, rlat in zip(latches, rl))
            if good:
                break
            run_gt(s, 1)
            t += 1
        max_setup = max(max_setup, t)
        run_gt(s, 4)
        s.use(*clk_lever)                    # rising edge
        run_gt(s, high)
        s.use(*clk_lever)                    # falling edge
        state = [val[l["d"]] for l in latches]
        val2 = model(bits)
        t = 0                                # measured post-edge margin
        while t < cap and not q_and_pos_match(val2):
            run_gt(s, 1)
            t += 1
        max_edge = max(max_edge, t)
        run_gt(s, 4)
        if not q_and_pos_match(val2):
            print("   STEP %d WRONG (want Q=%s)" % (si, state))
            ok = False
            break
    if ok:
        print("PASS (rust seq) %s: init + %d clocked steps, measured "
              "setup %d gt, edge %d gt, min period %d gt"
              % (args.top, steps, max_setup, max_edge,
                 max_setup + high + max_edge))
    if args.out and ok:
        b.save_to_file(args.out)
        print("saved (rust, baked-by-construction)", args.out)
    return 0 if ok else 1


def run_gt(sim, ticks):
    sim.sim.run(ticks)


# ---- Rust parity path -----------------------------------------------------
def run_rust(args, blif_path, inputs, outputs, comp, po_val):
    """Parity proof for the built-in compiler: the SAME BLIF goes through the
    Rust pipeline (nucleation.Hdl -> crates/nucleation-hdl, a line-by-line
    port of this file + build_ppa.py), and the RUST-built geometry is driven
    in mc-tick against THIS file's reference model.  The Python compile path
    above stays as the reference implementation."""
    import json
    import nucleation as n

    text = open(blif_path).read()
    report = json.loads(n.Hdl.compile_blif_report(text, args.top))
    assert report["inputs"] == list(inputs), (report["inputs"], inputs)
    b = n.Hdl.compile_blif(text, args.top, False)
    print("rust: %d blocks, %d prims over %d levels (peephole -%d)"
          % (report["blocks"], report["prims"], report["levels"],
             report["peephole_removed"]))
    if args.no_sim:
        if args.out:
            b.save_to_file(args.out)
            print("saved (rust, unbaked)", args.out)
        return 0

    # tighten round-trip, exactly rs.Build.sim: the engine sizes its world
    # from the ALLOCATED region, and a .schem save normalises it
    tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_hdl_rust.schem")
    b.save_to_file(tmp)
    tight = n.Schematic.open(tmp)
    sim = n.TickSimulation.from_schematic(tight, n.TickSettleMode.Placement,
                                          0, 0, 0, rs.EXTRA_STATES)
    sim.run_until_quiescent(4000)
    s = rs.Sim(sim, tuple(report["bounds"][0]))
    print("rust: placed; quiescent:", s.sim.is_quiescent())
    # one lever per PI, emitted in .inputs order -- drive by order
    lv = rs.Levers(s, [tuple(l["pos"]) for l in report["levers"]])
    probes = report["probes"]
    po_report = {o["name"]: o for o in report["outputs"]}

    n_in = len(inputs)
    if args.cases or n_in > 12:
        rnd = random.Random(args.seed)
        m = args.cases or 512
        cases = [0, (1 << n_in) - 1] + [rnd.getrandbits(n_in) for _ in range(m)]
    else:
        cases = list(range(1 << n_in))
    print("rust: checking %d cases (%s)" % (len(cases),
          "exhaustive" if not args.cases and n_in <= 12 else "sampled"))

    bad = 0
    for c in cases:
        bits = [(c >> i) & 1 for i in range(n_in)]
        lv.set(bits)
        want = comp.eval(bits)
        for po in outputs:
            v = po_val[po]
            w = v[1] if isinstance(v, tuple) else want[v]
            ro = po_report[po]
            g = ro["const"] if "const" in ro else int(s.on(*probes[ro["probe"]]))
            if g != w:
                bad += 1
                if bad <= 5:
                    print("   RUST WRONG %s inputs=%s got %d want %d"
                          % (po, format(c, "0%db" % n_in), g, w))
                break
    print("rust outputs correct: %d/%d" % (len(cases) - bad, len(cases)))
    if bad:
        return 1
    print("PASS (rust) %s: %d/%d cases" % (args.top, len(cases), len(cases)))
    if args.out:
        n.Hdl.compile_blif(text, args.top, True).save_to_file(args.out)
        print("saved (rust, baked at rest)", args.out)
    return 0


# ---- driver ---------------------------------------------------------------
def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--verilog", help="Verilog source file")
    ap.add_argument("--top", required=True, help="top module name")
    ap.add_argument("--blif", help="use a pre-made BLIF instead of running yosys")
    ap.add_argument("--out", default="", help="save baked .schem here")
    ap.add_argument("--cases", type=int, default=0,
                    help="random cases; 0 = exhaustive when <=12 inputs")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--no-sim", action="store_true")
    ap.add_argument("--steps", type=int, default=24,
                    help="clocked steps for sequential verification")
    ap.add_argument("--rust", action="store_true",
                    help="compile with the built-in Rust pipeline "
                         "(nucleation.Hdl, crates/nucleation-hdl) and verify "
                         "ITS build in mc-tick against this file's Python "
                         "reference model")
    args = ap.parse_args()

    blif = args.blif
    if not blif:
        if not args.verilog:
            ap.error("need --verilog or --blif")
        blif = os.path.join(HERE, "build", args.top + ".blif")
        run_yosys(args.verilog, args.top, blif)
        print("yosys: %s -> %s" % (args.verilog, blif))

    inputs, outputs, raw_nodes, latches = parse_blif(blif)
    if latches:
        clocks = {l["control"] for l in latches}
        assert len(clocks) == 1 and None not in clocks, "single clock only"
        clock = clocks.pop()
        real_pis = [i for i in inputs if i != clock]
        print("blif: sequential — %d latches on %s, %d data inputs"
              % (len(latches), clock, len(real_pis)))
        if not args.rust:
            print("sequential designs compile through the Rust pipeline: "
                  "re-run with --rust (the pure-Python geometry path is "
                  "combinational-only)")
            return 2
        return run_rust_seq(args, blif, real_pis, outputs, raw_nodes, latches)
    nodes, consts = fold(inputs, raw_nodes)
    print("blif: %d inputs %s, %d outputs, %d nodes, %d const nets"
          % (len(inputs), inputs, len(outputs), len(nodes), len(consts)))

    comp = Compiler(inputs, outputs, nodes, consts)
    po_val = {}
    for po in outputs:
        po_val[po] = comp.value(po, 1)       # ('const', b) or a vid
    removed = comp.peephole()
    po_val = {po: (v if isinstance(v, tuple) else comp.resolve(v))
              for po, v in po_val.items()}
    print("peephole: collapsed %d buffer/double-inversion nodes" % removed)
    comp.levelise()
    if args.rust:
        return run_rust(args, blif, inputs, outputs, comp, po_val)
    stages = comp.stages()
    n_prims = len(comp.val_terms)
    print("prims: %d nodes over %d levels (incl. complements/buffers/splits)"
          % (n_prims, max(comp.level.values()) if comp.level else 0))

    ppa = bp.PPA(len(inputs), make=lambda _n: stages, name=args.top)
    b = ppa.build()
    (x0, y0, z0), (x1, y1, z1) = b.bounds()
    vol = (x1 - x0 + 1) * (y1 - y0 + 1) * (z1 - z0 + 1)
    print("%s: %d blocks, extent x %d..%d y %d..%d z %d..%d (%.2fM cells)"
          % (args.top, len(b.cells), x0, x1, y0, y1, z0, z1, vol / 1e6))

    problems = audit.audit(b.cells)
    shorts = nets.check(b.cells, ppa.labels)
    for kind, items in problems.items():
        if items:
            print("STRUCTURAL %s x%d e.g. %s" % (kind, len(items), items[0]))
    print("net check: %d shorted nets" % len(shorts))
    for la, lb, pa, pb in shorts[:8]:
        print("   SHORT %s <-> %s  at %s / %s" % (la, lb, pa, pb))
    if any(problems.values()) or shorts:
        return 1

    import timing
    arrival, _whence = timing.analyze(ppa)
    sinks = [v for v in po_val.values() if not isinstance(v, tuple)]
    if sinks:
        worst = max(sinks, key=lambda s: arrival.get(s, 0))
        print("STA: critical path -> %s = %d rt (%d gt)"
              % (worst, arrival[worst], 2 * arrival[worst]))
    if args.no_sim:
        return 0

    sim = b.sim(settle=4000)
    print("placed; quiescent:", sim.sim.is_quiescent())
    lever_names = [s for s, _p in ppa.levers]
    lv = rs.Levers(sim, [p for _s, p in ppa.levers])
    pi_of_lever = {comp.vid(net, 1) + ".lv": i for i, net in enumerate(comp.pis)}

    n = len(inputs)
    if args.cases or n > 12:
        rnd = random.Random(args.seed)
        m = args.cases or 512
        cases = [0, (1 << n) - 1] + [rnd.getrandbits(n) for _ in range(m)]
    else:
        cases = list(range(1 << n))
    print("checking %d cases (%s)" % (len(cases),
          "exhaustive" if not args.cases and n <= 12 else "sampled"))

    bad, sig_bad = 0, {}
    for c in cases:
        bits = [(c >> i) & 1 for i in range(n)]
        lv.set([bits[pi_of_lever[nm]] for nm in lever_names])
        want = comp.eval(bits)
        wrong_po = False
        for po in outputs:
            v = po_val[po]
            w = v[1] if isinstance(v, tuple) else want[v]
            g = w if isinstance(v, tuple) else int(sim.on(*ppa.probe[v]))
            if g != w:
                wrong_po = True
        for s, p in ppa.probe.items():
            if s in want and int(sim.on(*p)) != want[s]:
                sig_bad[s] = sig_bad.get(s, 0) + 1
        if wrong_po:
            bad += 1
            if bad <= 5:
                print("   WRONG inputs=%s" % format(c, "0%db" % n))
    print("outputs correct: %d/%d" % (len(cases) - bad, len(cases)))
    if sig_bad:
        print("signals that disagreed (%d):" % len(sig_bad))
        for s, cnt in sorted(sig_bad.items(), key=lambda kv: -kv[1])[:12]:
            print("   %-24s wrong in %d/%d" % (s, cnt, len(cases)))
    if bad or sig_bad:
        return 1
    print("PASS %s: %d/%d cases" % (args.top, len(cases), len(cases)))

    if args.out:
        lv.set([0] * n)
        print("baked %d states" % bp.bake(b, sim))
        b.s.save_to_file(args.out)
        print("saved", args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
