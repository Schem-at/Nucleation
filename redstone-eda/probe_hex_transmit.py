"""probe_hex_transmit -- characterise TRANSMIT002_hex_transmit_flat.schem.

A user-supplied build described as "a typical hex bus that uses repeaters to
transport the signal, it's wider and less easy to fit but way faster than
alternating comparators dusts and blocks".  Every number in
notes-hex-transport.md comes from a group here.

  H0  structure    tight bounds, histogram, per-lane slices of the corpus file
  H1  replay       reload the corpus file into mc-tick, compare against the
                   state the author saved (InWorld and Placement modes)
  H2  faithful     drive the REAL build by rewriting the barrel's item count and
                   sweeping the analog value 0..15
  H3  parametric   rebuild the mechanism from rs.py primitives with lever
                   injectors, so the value can change while the sim is live
  H4  latency      per-game-tick trace of a rising and a falling edge, with the
                   delay attributed to each stage
  H5  pipelining   two injectors -> real value-to-value transitions; minimum
                   pulse width; do two values coexist in the fabric?
  H6  calibration  comb length and tap position -> the level-shift law
  H7  chain        ping-pong two stages in a constant 3-wide envelope

Run:  python3 probe_hex_transmit.py
"""
import os
import re
import sys

import nucleation as n
import rs

CORPUS = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                      "corpus", "TRANSMIT002_hex_transmit_flat.schem")

DIRS = ("north", "south", "east", "west")

# rs.EXTRA_STATES interns levers/torches/lamps/repeaters(delay 1-2) only.  This
# build needs comparators, all four repeater delays, wool and a barrel; a state
# that is not interned before construction sits INERT in mc-tick, so extend the
# module constant for the whole process (rs.Build.sim() reads it).
_EXTRA = ";".join(
    ["minecraft:comparator[facing=%s,mode=%s,powered=%s]" % (d, m, p)
     for d in DIRS for m in ("compare", "subtract") for p in ("true", "false")]
    + ["minecraft:repeater[facing=%s,delay=%d,locked=%s,powered=%s]" % (d, dl, lk, pw)
       for d in DIRS for dl in (1, 2, 3, 4) for lk in ("true", "false")
       for pw in ("true", "false")]
    + ["minecraft:barrel[facing=up,open=false]",
       "minecraft:magenta_wool", "minecraft:lime_wool", "minecraft:orange_wool"])
rs.EXTRA_STATES = rs.EXTRA_STATES + ";" + _EXTRA

_POWER = re.compile(r"power=(\d+)")

PASS = []


def check(name, got, want):
    ok = got == want
    PASS.append(ok)
    print("  %s %-52s got %-30s want %s"
          % ("PASS" if ok else "FAIL", name, repr(got), repr(want)))
    return ok


def note(name, got=""):
    print("  ---- %-52s %s" % (name, got))


def comparator(reads_from, mode="compare"):
    """A comparator whose BACK (input) is `reads_from`; output is the opposite.

    Vanilla DiodeBlock reads the neighbour in the FACING direction, so `facing`
    names the side the input arrives from -- same convention as rs.repeater.
    Verified by H0.5/H1 against the author's own saved state.
    """
    return "minecraft:comparator[facing=%s,mode=%s,powered=false]" % (reads_from, mode)


LEVER = "minecraft:lever[face=floor,facing=north,powered=false]"


# ----------------------------------------------------------------- H0 structure
def h0_structure():
    print("H0  structure of the corpus file")
    s = n.Schematic.open(CORPUS)
    d, tmin, tmax = s.dimensions(), s.tight_bounds_min(), s.tight_bounds_max()
    check("H0.1 dimensions (x,y,z)", (d.x, d.y, d.z), (3, 2, 19))
    check("H0.2 tight bounds are the whole file",
          ((tmin.x, tmin.y, tmin.z), (tmax.x, tmax.y, tmax.z)),
          ((0, 0, 0), (2, 1, 18)))
    cells = {}
    for x in range(d.x):
        for y in range(d.y):
            for z in range(d.z):
                b = s.get_block_string(x, y, z)
                if b and "air" not in b:
                    cells[(x, y, z)] = b
    check("H0.3 non-air cell count", len(cells), 98)
    kinds = {}
    for b in cells.values():
        kinds[b.split("[")[0]] = kinds.get(b.split("[")[0], 0) + 1
    note("H0.4 palette", sorted(kinds.items(), key=lambda kv: -kv[1]))
    check("H0.5 y=0 is a solid floor under every powered cell",
          all(cells.get((x, 0, z), "").endswith("wool")
              for (x, y, z) in cells if y == 1), True)

    inl = [_POWER.search(cells[(0, 1, z)]).group(1) for z in range(2, 17)]
    outl = [_POWER.search(cells[(2, 1, z)]).group(1) for z in range(2, 17)]
    reps = [cells[(1, 1, z)] for z in range(2, 17)]
    check("H0.6 x=1 lane: 15 repeaters, all facing=west, all delay=1",
          (len(reps), {r.split("[")[0] for r in reps},
           {"facing=west" in r for r in reps}, {"delay=1" in r for r in reps}),
          (15, {"minecraft:repeater"}, {True}, {True}))
    check("H0.7 saved INPUT lane ss z=2..16", ",".join(inl),
          "0,0,0,0,0,0,0,0,0,0,0,0,1,2,3")
    check("H0.8 saved OUTPUT lane ss z=2..16", ",".join(outl),
          "3,4,5,6,7,8,9,10,11,12,13,14,15,15,15")
    check("H0.9 repeaters are powered exactly where INPUT lane ss>=1",
          [z for z in range(2, 17) if "powered=true" in cells[(1, 1, z)]],
          [z for z in range(2, 17)
           if int(_POWER.search(cells[(0, 1, z)]).group(1)) >= 1])
    check("H0.10 both end devices are comparators in COMPARE mode facing=south",
          (cells[(0, 1, 17)], cells[(2, 1, 1)]),
          ("minecraft:comparator[facing=south,mode=compare,powered=true]",
           "minecraft:comparator[facing=south,mode=compare,powered=true]"))
    be = s.get_all_block_entities_snbt_json()
    check("H0.11 source is a barrel; both comparators saved OutputSignal 3",
          ("minecraft:barrel" in be, be.count("OutputSignal:3")), (True, 2))
    note("H0.12 input marker floor (0,0,18) / output marker floor (2,0,0)",
         (cells[(0, 0, 18)], cells[(2, 0, 0)]))
    note("H0.13 cross-section", "3 wide (x) x 2 high (y incl. floor); 19 long (z)")
    print()


# -------------------------------------------------------------------- H1 replay
def h1_replay():
    print("H1  replay the corpus file in mc-tick")
    for mode_name, mode in (("InWorld", n.TickSettleMode.InWorld),
                            ("Placement", n.TickSettleMode.Placement)):
        s = n.Schematic.open(CORPUS)
        sim = n.TickSimulation.from_schematic(s, mode, 0, 0, 0, rs.EXTRA_STATES)
        settled = sim.run_until_quiescent(600)

        def ss(x, y, z):
            m = _POWER.search(sim.get_block(x, y, z))
            return int(m.group(1)) if m else -1
        check("H1.%s settles" % mode_name, settled, True)
        check("H1.%s INPUT lane ss z=2..16" % mode_name,
              [ss(0, 1, z) for z in range(2, 17)], [0] * 12 + [1, 2, 3])
        check("H1.%s OUTPUT lane ss z=2..16" % mode_name,
              [ss(2, 1, z) for z in range(2, 17)],
              [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 15, 15])
        check("H1.%s OUTPUT dust (2,1,0) == barrel value 3" % mode_name,
              ss(2, 1, 0), 3)
        note("H1.%s ticks to quiescence (0 == the saved state is self-"
             "consistent)" % mode_name, sim.tick_count())
    print()


# ------------------------------------------------------------------ H2 faithful
def _barrel_snbt(stacks):
    items = ",".join('{id:"minecraft:redstone",Slot:%dB,count:64}' % i
                     for i in range(stacks))
    return "{Items:[%s]}" % items


def h2_faithful_barrel_sweep():
    print("H2  drive the REAL build by rewriting the barrel's item count")
    seen = {}
    for stacks in range(0, 28):
        s = n.Schematic.open(CORPUS)
        s.set_block_entity(0, 1, 18, "minecraft:barrel", _barrel_snbt(stacks))
        sim = n.TickSimulation.from_schematic(
            s, n.TickSettleMode.Placement, 0, 0, 0, rs.EXTRA_STATES)
        sim.run_until_quiescent(600)

        def ss(x, y, z):
            m = _POWER.search(sim.get_block(x, y, z))
            return int(m.group(1)) if m else -1
        v_in = ss(0, 1, 16)      # what the input comparator injected
        v_out = ss(2, 1, 0)      # what came out the far end
        fired = [z for z in range(2, 17) if "powered=true" in sim.get_block(1, 1, z)]
        seen.setdefault(v_in, (v_out, len(fired), min(fired) if fired else None))
    check("H2.1 all 16 analog values reachable from barrel fill",
          sorted(seen), list(range(16)))
    check("H2.2 every analog value arrives UNCHANGED (v_out == v_in)",
          [(v, seen[v][0]) for v in sorted(seen)],
          [(v, v) for v in sorted(seen)])
    check("H2.3 unary decode: #repeaters that fire == the analog value",
          [(v, seen[v][1]) for v in sorted(seen)],
          [(v, v) for v in sorted(seen)])
    check("H2.4 lowest firing repeater sits at z = 17 - value",
          [(v, seen[v][2]) for v in sorted(seen) if v > 0],
          [(v, 17 - v) for v in sorted(seen) if v > 0])
    print()


# --------------------------------------------------------------------- the rig
class Rig:
    """Parametric reconstruction of the corpus mechanism.

    Identical lanes to the corpus file (INPUT dust x=0, repeater comb x=1
    facing=west, OUTPUT dust x=2, transport along -z), but the barrel is
    replaced by an attenuator dust run with LEVER INJECTORS beside it, so the
    analog value can be selected and changed while the sim is live.

    An injector for value v is a floor lever at (15-v, 1, ZTOP+3): it forces the
    attenuator cell (15-v, 1, ZTOP+2) to 15, which decays to exactly v by the
    time it reaches the input comparator's back at (0, 1, ZTOP+2).  Several
    injectors may be live at once; dust takes the max, so the transported value
    is max(values).
    """

    ZTOP = 16

    def __init__(self, values=(15,), comb_len=15, tap=None, name="hexrig"):
        self.comb_len = comb_len
        self.zlo = self.ZTOP - comb_len + 1
        self.tap = self.zlo if tap is None else tap
        self.values = list(values)
        b = rs.Build(name)
        for z in range(self.zlo, self.ZTOP + 1):
            b.dust(0, 1, z)
            b.stone(1, 0, z)
            b.put(1, 1, z, rs.repeater("west", 1))
            if z >= self.tap:          # OUTPUT lane exists from the tap upward
                b.dust(2, 1, z)
        za = self.ZTOP + 2
        b.stone(0, 0, self.ZTOP + 1)
        b.put(0, 1, self.ZTOP + 1, comparator("south"))
        for x in range(0, 15):
            b.dust(x, 1, za)
        self.levers = []
        for v in self.values:
            x = 15 - v
            b.stone(x, 0, za + 1)
            b.put(x, 1, za + 1, LEVER)
            self.levers.append((x, 1, za + 1))
        b.stone(2, 0, self.tap - 1)
        b.put(2, 1, self.tap - 1, comparator("south"))
        b.dust(2, 1, self.tap - 2)
        self.b = b
        self.inj = (0, 1, self.ZTOP)          # comb top = injection point
        self.tapc = (2, 1, self.tap)          # output comparator's back
        self.out = (2, 1, self.tap - 2)       # readout dust
        self.s = b.sim()
        self.L = rs.Levers(self.s, self.levers)

    def on(self, *bits):
        self.L.set(bits)

    def rungs_fired(self):
        return [z for z in range(self.zlo, self.ZTOP + 1)
                if self.s.powered(1, 1, z)]


def trace(s, points, budget=24, stop_stable=4):
    """Step one game tick at a time, recording ss at each point."""
    tr = {k: [] for k in points}
    for _ in range(budget):
        for k, p in points.items():
            tr[k].append(s.power(*p))
        vals = list(tr.values())[0]
        if len(vals) >= stop_stable and len(set(vals[-stop_stable:])) == 1 \
                and s.sim.is_quiescent():
            break
        s.sim.step()
    return tr


# ------------------------------------------------------------- H3 parametric
def h3_parametric_sweep():
    print("H3  parametric reconstruction: value-preservation sweep")
    rows = []
    for v in range(1, 16):
        r = Rig(values=(v,), name="hex_v%d" % v)
        r.on(1)
        rows.append((v, r.s.power(*r.inj), r.s.power(*r.out),
                     len(r.rungs_fired())))
    check("H3.1 injector for v puts exactly v on the comb top",
          [(v, vi) for v, vi, _, _ in rows], [(v, v) for v in range(1, 16)])
    check("H3.2 OUTPUT == INPUT for every analog value 1..15",
          [(vi, vo) for _, vi, vo, _ in rows],
          [(v, v) for v in range(1, 16)])
    check("H3.3 unary comb: #rungs fired == value",
          [(vi, nf) for _, vi, _, nf in rows],
          [(v, v) for v in range(1, 16)])
    for v, vi, vo, nf in rows:
        note("H3.4 v=%2d -> in=%2d out=%2d (rungs fired %2d)" % (v, vi, vo, nf))
    r = Rig(values=(8,))
    check("H3.5 injector OFF -> nothing transported", r.s.power(*r.out), 0)
    check("H3.6 ... and no rung fires", r.rungs_fired(), [])
    print()


# ---------------------------------------------------------------- H4 latency
def h4_latency():
    print("H4  latency, per game tick")
    for v in (1, 3, 8, 15):
        r = Rig(values=(v,))
        r.s.use(*r.levers[0])                    # rising edge, no settle
        tr = trace(r.s, {"out": r.out}, 20)["out"]
        first = next((i for i, x in enumerate(tr) if x > 0), None)
        right = next((i for i, x in enumerate(tr) if x == v), None)
        check("H4.1 v=%2d final OUTPUT value" % v, tr[-1], v)
        note("H4.2 v=%2d rise trace at OUTPUT" % v, tr[:10])
        check("H4.3 v=%2d gt from lever to correct OUTPUT value" % v, right, 6)
        r.s.use(*r.levers[0])                    # falling edge
        trd = trace(r.s, {"out": r.out}, 20)["out"]
        note("H4.4 v=%2d fall trace at OUTPUT" % v, trd[:10])
        check("H4.5 v=%2d gt from lever to OUTPUT clear" % v,
              next((i for i, x in enumerate(trd) if x == 0), None), 6)
    # attribute the delay to each stage
    r = Rig(values=(8,))
    r.s.use(*r.levers[0])
    pts = {"a source dust (0,1,18)": (0, 1, 18),
           "b comb top    (0,1,16)": (0, 1, 16),
           "c comb bottom (0,1, 9)": (0, 1, 9),
           "d out lane top(2,1,16)": (2, 1, 16),
           "e tap         (2,1, 2)": (2, 1, 2),
           "f OUTPUT      (2,1, 0)": r.out}
    tr = trace(r.s, pts, 14, stop_stable=99)
    for k in sorted(pts):
        note("H4.6 v=8 %s" % k, tr[k])
    arrive = {k: next((i for i, x in enumerate(tr[k]) if x > 0), None)
              for k in pts}
    check("H4.7 stage budget (gt): source=0, comb top=2 (input comparator), "
          "out lane=4 (repeater), tap=4 (dust is free), OUTPUT=6 (output "
          "comparator)",
          [arrive[k] for k in sorted(pts)], [0, 2, 2, 4, 4, 6])
    note("H4.8 TRANSPORT SECTION only (comb top -> tap, 14 blocks of z)",
         "%d gt" % (arrive["e tap         (2,1, 2)"]
                    - arrive["b comb top    (0,1,16)"]))
    print()


# ------------------------------------------------------------ H5 pipelining
def h5_pipelining():
    print("H5  pipelining and throughput")
    # two injectors -> real value-to-value transitions (dust takes the max)
    r = Rig(values=(15, 6))
    r.on(1, 1)
    check("H5.1 both injectors on -> max wins", r.s.power(*r.out), 15)
    r.s.use(*r.levers[0])                       # drop 15, leave 6
    tr = trace(r.s, {"out": r.out, "in": r.inj}, 20, stop_stable=99)
    note("H5.2 15->6 transition, INPUT lane", tr["in"][:10])
    note("H5.3 15->6 transition, OUTPUT", tr["out"][:10])
    check("H5.4 15->6 settles to 6", tr["out"][-1], 6)
    coexist = [i for i in range(len(tr["in"]))
               if tr["in"][i] != tr["out"][i] and tr["out"][i] > 0]
    check("H5.5 PIPELINED: ticks where the fabric holds the old value at the "
          "output while the new value is already on the input lane",
          len(coexist) > 0, True)
    note("H5.6 those ticks", coexist)

    # minimum pulse width the carrier will pass
    widths = {}
    for w in (1, 2, 3, 4, 5):
        rr = Rig(values=(15,))
        rr.on(1)
        rr.s.use(*rr.levers[0])                 # 15 -> 0
        for _ in range(w):
            rr.s.sim.step()
        rr.s.use(*rr.levers[0])                 # 0 -> 15
        tr2 = trace(rr.s, {"out": rr.out}, 26, stop_stable=99)["out"]
        dip = sum(1 for x in tr2 if x == 0)
        widths[w] = (dip, tr2[:14])
        note("H5.7 %d-gt LOW pulse -> %d gt of 0 seen at OUTPUT" % (w, dip),
             tr2[:12])
    check("H5.8 a 1-gt or 2-gt gap is SWALLOWED by the delay-1 repeater",
          (widths[1][0], widths[2][0]), (0, 0))
    check("H5.9 a >=3-gt gap gets through", widths[3][0] > 0, True)
    print()


# ----------------------------------------------------------- H6 calibration
def h6_calibration():
    print("H6  calibration: comb length and tap position")
    rows = []
    for comb in (4, 8, 11, 15):
        for v in (1, 3, 8, 12, 15):
            r = Rig(values=(v,), comb_len=comb)
            r.on(1)
            rows.append((comb, v, r.s.power(*r.inj), r.s.power(*r.out)))
    for comb, v, vi, vo in rows:
        note("H6.1 comb_len=%2d v=%2d -> in=%2d out=%2d (shift %+d)"
             % (comb, v, vi, vo, vo - vi))
    check("H6.2 comb_len == 15 is the LOSSLESS length",
          [(v, vo) for c, v, vi, vo in rows if c == 15],
          [(1, 1), (3, 3), (8, 8), (12, 12), (15, 15)])
    law = [(c, v, vo, min(15, v + 15 - c)) for c, v, vi, vo in rows]
    check("H6.3 level-shift law: out = min(15, v + (15 - comb_len))",
          [(c, v, vo) for c, v, vo, w in law],
          [(c, v, w) for c, v, vo, w in law])
    # tap position on a full-length comb
    trows = []
    for tap in (2, 4, 7):
        for v in (3, 8):
            r = Rig(values=(v,), comb_len=15, tap=tap)
            r.on(1)
            trows.append((tap, v, r.s.power(*r.out)))
    for tap, v, vo in trows:
        note("H6.4 tap z=%d v=%2d -> out=%2d (shift %+d)" % (tap, v, vo, vo - v))
    check("H6.5 tap law: out = min(15, v + (tap - 2))",
          [(t, v, vo) for t, v, vo in trows],
          [(t, v, min(15, v + t - 2)) for t, v, vo in trows])
    print()


# ----------------------------------------------------------------- H7 chain
def h7_chain():
    print("H7  chaining: ping-pong two stages inside a constant 3-wide envelope")
    b = rs.Build("hexchain")
    # stage 1: INPUT x=0, comb x=1 facing=west, OUTPUT x=2; z = 2..16
    for z in range(2, 17):
        b.dust(0, 1, z)
        b.stone(1, 0, z)
        b.put(1, 1, z, rs.repeater("west", 1))
        b.dust(2, 1, z)
    # source: attenuator + injector for v
    V = 9
    b.stone(0, 0, 17)
    b.put(0, 1, 17, comparator("south"))
    for x in range(0, 15):
        b.dust(x, 1, 18)
    b.stone(15 - V, 0, 19)
    b.put(15 - V, 1, 19, LEVER)
    lever = (15 - V, 1, 19)
    # stage 1 tap comparator at (2,1,1) -> re-injects v at (2,1,0)
    b.stone(2, 0, 1)
    b.put(2, 1, 1, comparator("south"))
    b.dust(2, 1, 0)
    # stage 2 reuses the SAME three x columns, mirrored: INPUT x=2,
    # comb x=1 facing=east (reads x=2, drives x=0), OUTPUT x=0; z = -14..0
    for z in range(-14, 1):
        b.dust(2, 1, z)
        b.stone(1, 0, z)
        b.put(1, 1, z, rs.repeater("east", 1))
        b.dust(0, 1, z)
    b.stone(0, 0, -15)
    b.put(0, 1, -15, comparator("south"))
    b.dust(0, 1, -16)
    s = b.sim()
    L = rs.Levers(s, [lever])
    L.set([1])
    v_stage1_top = s.power(0, 1, 16)
    v_stage1_out = s.power(2, 1, 0)
    v_stage2_out = s.power(0, 1, -16)
    note("H7.1 stage1 comb top / stage1 out (= stage2 comb top) / stage2 out",
         (v_stage1_top, v_stage1_out, v_stage2_out))
    check("H7.2 stage 1 is lossless", (v_stage1_top, v_stage1_out), (V, V))
    check("H7.3 stage 2 is lossless too -> the carrier CHAINS",
          v_stage2_out, V)
    note("H7.4 z span covered by 2 stages (comb top 16 -> final out -16)", 32)
    note("H7.5 x envelope for 2 stages (ping-pong reuses the columns)", 3)
    # latency of the two-stage chain
    s.use(*lever)
    s.settle()
    s.use(*lever)
    tr = trace(s, {"o": (0, 1, -16)}, 26, stop_stable=99)["o"]
    rise = next((i for i, x in enumerate(tr) if x == V), None)
    note("H7.6 2-stage rise trace at final OUTPUT", tr[:14])
    check("H7.7 2-stage end-to-end latency (gt)", rise, 10)
    note("H7.8 marginal cost of one extra stage",
         "%d gt for 16 more blocks of z" % (rise - 6))
    print()


# --------------------------------------------- H8 lid, crosstalk, lane pitch
def h8_envelope():
    print("H8  envelope: lid tolerance, neighbour pitch, crosstalk")
    # H8a a solid lid over the WHOLE bus
    r = Rig(values=(9,))
    for z in range(r.zlo, r.ZTOP + 1):
        for x in (0, 1, 2):
            r.b.stone(x, 2, z)
    r.s = r.b.sim()
    r.L = rs.Levers(r.s, r.levers)
    r.on(1)
    check("H8.1 a solid lid at y+1 over all three lanes changes nothing",
          r.s.power(*r.out), 9)

    # H8b a foreign dust line ON the lid, directly above the OUTPUT lane.
    # NOTE the foreign lever must be kept off the attenuator: a floor lever
    # STRONGLY powers its attachment block, and a strong block is read at 15 by
    # any dust on any of its six faces -- parking it at (2,2,18) fed 15 straight
    # into the attenuator below and read 13 at the output.  That is a real trap
    # for a planner (a lever's attachment block is a 15-source in 6 directions),
    # not a property of this carrier, so the rig puts the lever at z=ZTOP+3
    # where its attachment touches nothing.
    r2 = Rig(values=(9,))
    for z in range(r2.zlo, r2.ZTOP + 4):
        for x in (0, 1, 2):
            r2.b.stone(x, 2, z)
        r2.b.put(2, 3, z, rs.DUST)
    r2.b.force(2, 3, r2.ZTOP + 3, LEVER)
    foreign = (2, 3, r2.ZTOP + 3)
    r2.s = r2.b.sim()
    r2.L = rs.Levers(r2.s, r2.levers + [foreign])
    r2.L.set([1, 0])
    check("H8.2 a foreign dust line on the lid: bus unaffected",
          r2.s.power(*r2.out), 9)
    check("H8.3 ... and the bus does not power the foreign line (dust never "
          "powers the block above -- W3)",
          [r2.s.power(2, 3, z) for z in (r2.ZTOP, r2.ZTOP - 7, r2.zlo)],
          [0, 0, 0])
    r2.L.set([1, 1])
    check("H8.4 driving the foreign line does not disturb the bus",
          r2.s.power(*r2.out), 9)

    # H8c neighbour pitch: a second, foreign hex-bus dust lane beside x=2
    for gap in (1, 2):
        r3 = Rig(values=(9,))
        xf = 2 + gap
        for z in range(r3.zlo, r3.ZTOP + 1):
            r3.b.dust(xf, 1, z)
        r3.s = r3.b.sim()
        r3.L = rs.Levers(r3.s, r3.levers)
        r3.on(1)
        leak = max(r3.s.power(xf, 1, z) for z in range(r3.zlo, r3.ZTOP + 1))
        note("H8.5 foreign dust lane at gap=%d from the OUTPUT lane: max ss "
             "picked up = %d" % (gap, leak))
        if gap == 1:
            check("H8.6 gap 1 LEAKS (the OUTPUT lane runs hot at 15)",
                  leak > 0, True)
        else:
            check("H8.7 gap 2 is clean", leak, 0)
    print()


# ------------------------------------------------ H9 locking and repeater delay
def h9_locking_and_delay():
    print("H9  comb correctness: no locking; delay knob")
    r = Rig(values=(9,))
    r.on(1)
    locked = [z for z in range(r.zlo, r.ZTOP + 1)
              if "locked=true" in r.s.block(1, 1, z)]
    check("H9.1 no rung locks -- the rungs are side by side but all face the "
          "same way, so none points into another's side", locked, [])
    for delay in (1, 2, 3, 4):
        b = rs.Build("hexdelay%d" % delay)
        for z in range(2, 17):
            b.dust(0, 1, z)
            b.stone(1, 0, z)
            b.put(1, 1, z, rs.repeater("west", delay))
            b.dust(2, 1, z)
        b.stone(0, 0, 17)
        b.put(0, 1, 17, comparator("south"))
        for x in range(0, 15):
            b.dust(x, 1, 18)
        b.stone(15 - 9, 0, 19)
        b.put(15 - 9, 1, 19, LEVER)
        lever = (15 - 9, 1, 19)
        b.stone(2, 0, 1)
        b.put(2, 1, 1, comparator("south"))
        b.dust(2, 1, 0)
        s = b.sim()
        rs.Levers(s, [lever])
        s.use(*lever)
        tr = trace(s, {"o": (2, 1, 0)}, 26, stop_stable=99)["o"]
        at = next((i for i, x in enumerate(tr) if x == 9), None)
        note("H9.2 comb delay=%d -> end-to-end %s gt, value %s"
             % (delay, at, tr[-1]))
        check("H9.3 comb delay=%d stays lossless" % delay, tr[-1], 9)
        check("H9.4 comb delay=%d latency == 4 + 2*delay gt" % delay,
              at, 4 + 2 * delay)
    print()


def main():
    h0_structure()
    h1_replay()
    h2_faithful_barrel_sweep()
    h3_parametric_sweep()
    h4_latency()
    h5_pipelining()
    h6_calibration()
    h7_chain()
    h8_envelope()
    h9_locking_and_delay()
    print("%d/%d checks passed" % (sum(PASS), len(PASS)))
    return 0 if all(PASS) else 1


if __name__ == "__main__":
    sys.exit(main())
