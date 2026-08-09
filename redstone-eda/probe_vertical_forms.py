"""Probe the VERTICAL transport forms a bus router needs, and the slope rule.

`probe_vert.py` established the torch ladder as a primitive (2 y per torch,
inverting, 1x1).  This probes the forms a *bus* is built from -- how densely
each one packs, whether it goes DOWN as well as UP, and what its ports cost --
plus the horizontal level-change slope.  Templates: `vforms.py`.
Rows + rankings: `notes-vertical-transport.md`.  Tiling: `probe_spiral_tiling.py`.

Groups:

  L. TORCH LADDER, as a bus
     L1 rise is 2 y per torch and the cap reads 15 (refresh, not decay),
     L2 parity: an ODD torch count inverts, an EVEN one does not,
     L3 delay: measured game ticks per torch,
     L4 8 ladders at x-PITCH 1 with ports on ALTERNATING sides: all 256 bit
        patterns, zero crosstalk -- 1 xz cell per bit,
     L5 the same array with ports on the SAME side is BROKEN, and not by a
        leak: the pitch-1 entry dusts merge into one net whose connections
        turn it into a T, and a T does not point into the bases (POINTING
        LAW), so every ladder reads a constant.  This is the trap: the array
        looks quiet, and it is quiet -- dead.
     L6 x-pitch 2 with same-side ports is clean (the conservative form).

  G. TRANSPARENT TOWER (the 2x1 all-glass zigzag)
     G1 it climbs 1 y per cell in 2 xz cells, -1 ss per y,
     G2 all-SOLID supports kill it after one cell: every support of this form
        is the CUT cell of the dust below it, so transparency is MANDATORY,
     G3 it is a DIODE -- driven from the top, nothing comes out at the bottom,
     G4 two towers at z-pitch 1 LEAK (the glass keeps cross-net diagonals
        alive too); pitch 2 is clean.

  S. RING RISER (the spiral staircase), single bit
     S1 it climbs, S2 it DESCENDS -- the only form here that does,
     S3 -1 ss per y in both directions, so 14 y per source,
     S4 a repeater on a flat cell of the ring refreshes it for an unbounded
        descent, at the cost of 1 cell of rise and 2 gt.

  D. WHY DOWN IS ASYMMETRIC
     D1 a torch strongly powers the block ABOVE it and nothing below, so no
        torch form descends at all,
     D2 a dust weak-powers the block BELOW it, and weak power is invisible to
        dust -- so a dust cannot hand a signal down through a block either,
     D3 ...but a REPEATER reads that weakly powered block, which is the one
        active way down: 1 y per repeater station.

  H. HORIZONTAL SLOPE, and the "every second block is transparent" rule
     H1 a 1:1 staircase on all-SOLID supports conducts both ways,
     H2 the same staircase on all-GLASS supports conducts UP only (diode),
     H3 two lines 2 y apart on a half slope with ALL-GLASS supports MERGE,
     H4 the same pair with ALTERNATING supports is isolated in all 4 patterns.

Run: ~/eda-venv/bin/python probe_vertical_forms.py
"""
import itertools
import rs
import audit
import vforms as vf
from rs import DUST, STONE, LEVER_OFF, repeater
from vforms import GLASS

VERDICTS = []


def V(ok, text, extra=""):
    VERDICTS.append((bool(ok), text, extra))


# ---------------------------------------------------------------- harness
class Rig:
    """One isolated build + sim, with levers and named probe cells."""

    def __init__(self, name):
        self.b = rs.Build(name)
        self.levs = []
        self.p = {}

    def lever(self, cell, d=(0, -1), n=1):
        """A lever `n` dust cells away from `cell`, running out along `d`."""
        x, y, z = cell
        dx, dz = d
        for k in range(1, n + 1):
            self.b.force(x + dx * k, y - 1, z + dz * k, STONE)
            self.b.force(x + dx * k, y, z + dz * k, DUST)
        lc = (x + dx * (n + 1), y, z + dz * (n + 1))
        self.b.force(lc[0], lc[1] - 1, lc[2], STONE)
        self.b.force(*lc, LEVER_OFF)
        self.levs.append(lc)
        return lc

    def probe(self, name, cell):
        self.p[name] = cell
        return cell

    def start(self, settle=400):
        self.floating = sum(len(v) for v in audit.audit(self.b.cells).values())
        self.sim = self.b.sim(settle=settle)
        self.lv = rs.Levers(self.sim, self.levs)
        return self

    def set(self, bits):
        self.lv.set(list(bits))
        return self

    def ss(self, name):
        return self.sim.power(*self.p[name])

    def sweep(self, names, patterns=None):
        """{pattern: {name: ss}} over `patterns` (default: the full matrix)."""
        n = len(self.levs)
        pats = patterns if patterns is not None else list(
            itertools.product((0, 1), repeat=n))
        out = {}
        for bits in pats:
            self.set(bits)
            out[tuple(bits)] = {k: self.ss(k) for k in names}
        return out


def crosstalk(res, nbits, key=lambda i: "out%d" % i):
    """(min ss when driven, max ss when NOT driven) per bit."""
    lo = [None] * nbits
    hi = [0] * nbits
    for bits, vals in res.items():
        for i in range(nbits):
            v = vals[key(i)]
            if bits[i]:
                lo[i] = v if lo[i] is None else min(lo[i], v)
            else:
                hi[i] = max(hi[i], v)
    return lo, hi


# ================================================================= L. ladder
def ladder_rig(nbits, pitch, alt_sides, torches):
    r = Rig("lad_%d_%d_%s" % (nbits, pitch, alt_sides))
    for i in range(nbits):
        x = i * pitch
        side = -1 if (not alt_sides or i % 2 == 0) else +1
        e, o, cap = vf.torch_ladder(r.b, x, 0, 2, torches, port_side=side)
        # carry each exit 2 further cells out so a leak has to cross real dust
        for k in (2, 3):
            r.b.force(x, o[1] - 1, k * side, STONE)
            r.b.force(x, o[1], k * side, DUST)
        r.probe("out%d" % i, (x, o[1], 3 * side))
        r.probe("in%d" % i, e)
        r.lever(e, d=(0, side), n=2)
    return r


# L1/L2/L3 -- one ladder, both parities
for T in (3, 4):
    r = Rig("lad_parity_%d" % T)
    e, o, cap = vf.torch_ladder(r.b, 0, 0, 2, T)
    r.probe("cap", cap)
    r.probe("out", o)
    r.lever(e, d=(0, -1), n=2)
    r.start()
    r.set([0])
    o_off = r.ss("out")
    r.set([1])
    o_on = r.ss("out")
    rise = cap[1] - 2
    V(rise == 2 * T, "L1 torch ladder rise is 2 y per torch (%d torches -> %d y)"
      % (T, rise))
    V(max(o_off, o_on) == 15,
      "L1 the cap REFRESHES to 15 -- a ladder has no ss budget (%d torches)" % T,
      "off=%d on=%d" % (o_off, o_on))
    if T % 2:
        V(o_off == 15 and o_on == 0,
          "L2 an ODD torch count (%d) INVERTS" % T, "off=%d on=%d" % (o_off, o_on))
    else:
        V(o_off == 0 and o_on == 15,
          "L2 an EVEN torch count (%d) is non-inverting" % T,
          "off=%d on=%d" % (o_off, o_on))
    if T == 4:
        t0 = r.sim.sim.tick_count()
        r.set([0])
        dt = r.sim.sim.tick_count() - t0
        V(dt <= 2 * T + 2,
          "L3 a %d-torch ladder settles in %d gt (<= 2 gt per torch)" % (T, dt),
          "%.1f gt per y of rise" % (dt / float(2 * T)))

# L4 -- 8 ladders at pitch 1, alternating ports, ALL 256 patterns
r = ladder_rig(8, 1, True, 4).start()
names = ["out%d" % i for i in range(8)] + ["in%d" % i for i in range(8)]
res = r.sweep(names)
lo, hi = crosstalk(res, 8)
ilo, ihi = crosstalk(res, 8, key=lambda i: "in%d" % i)
LAD8 = r
V(all(v > 0 for v in lo) and all(v == 0 for v in hi)
  and all(v == 0 for v in ihi) and r.floating == 0,
  "L4 8 torch ladders at x-PITCH 1 with alternating-side ports: 256/256 "
  "patterns, zero crosstalk at the exits AND at every entry -- 1 xz cell "
  "per bit per y of rise",
  "driven=%s (15 at the port, -1 per probe cell) exit-leak=%s entry-leak=%s "
  "floating=%d" % (set(lo), set(hi), set(ihi), r.floating))

# L5 -- the same at pitch 1 with SAME-side ports: dead, not leaky
r = ladder_rig(4, 1, False, 4).start()
res = r.sweep(["out%d" % i for i in range(4)])
lo, hi = crosstalk(res, 4)
V(not all(v > 0 for v in lo),
  "L5 pitch-1 ladders with SAME-side ports are DEAD, not leaky: the merged "
  "entry dust is a T and a T does not point into the bases (POINTING LAW)",
  "driven=%s leak=%s" % (lo, hi))

# L6 -- pitch 2, same side
r = ladder_rig(4, 2, False, 4).start()
res = r.sweep(["out%d" % i for i in range(4)])
lo, hi = crosstalk(res, 4)
V(all(v > 0 for v in lo) and all(v == 0 for v in hi) and r.floating == 0,
  "L6 x-pitch 2 with same-side ports is clean (16/16) -- 2 xz cells per bit",
  "driven=%s leak=%s" % (set(lo), set(hi)))


# ============================================================== G. glass tower
N = 12
for sup, name in ((GLASS, "glass"), (vf.SLAB_TOP, "top slab"), (STONE, "solid")):
    r = Rig("tower_%s" % name.replace(" ", "_"))
    cs = vf.glass_tower(r.b, 1, 0, 4, N, support=sup)
    for i, c in enumerate(cs):
        r.probe("c%d" % i, c)
    r.lever(cs[0], d=(-1, 0), n=1)
    r.start()
    r.set([1])
    prof = [r.ss("c%d" % i) for i in range(N)]
    if sup is STONE:
        V(prof[1] == 0,
          "G2 an ALL-SOLID 2x1 zigzag is dead after one cell: every support of "
          "this form is the CUT cell of the dust below it, so transparency is "
          "MANDATORY", "profile=%s" % prof)
    else:
        V(all(v > 0 for v in prof) and prof[0] - prof[-1] == N - 1,
          "G1 a %s tower climbs 1 y per cell in 2 xz cells at -1 ss per y "
          "(%d y on one source)" % (name, N),
          "profile=%s" % prof)

# G3 -- the tower is a diode: drive it from the TOP
r = Rig("tower_down")
cs = vf.glass_tower(r.b, 1, 0, 4, N)
r.probe("bottom", cs[0])
r.probe("top", cs[-1])
r.lever(cs[-1], d=(-1, 0) if cs[-1][0] == 2 else (+1, 0), n=1)
r.start()
r.set([1])
V(r.ss("top") > 0 and r.ss("bottom") == 0,
  "G3 the transparent tower is a DIODE: driven from the top, the bottom is "
  "dark (every upper dust sits on a non-conductor)",
  "top=%d bottom=%d" % (r.ss("top"), r.ss("bottom")))

# G4 -- tower z-pitch
for zp in (1, 2):
    for dy in (0, 1):
        r = Rig("tower_pitch%d_%d" % (zp, dy))
        A = vf.glass_tower(r.b, 1, 0, 4, N)
        B = vf.glass_tower(r.b, 1, zp, 4 + dy, N)
        r.probe("out0", A[-1])
        r.probe("out1", B[-1])
        r.lever(A[0], d=(-1, 0))
        r.lever(B[0], d=(-1, 0))
        r.start()
        res = r.sweep(["out0", "out1"])
        lo, hi = crosstalk(res, 2)
        clean = all(v and v > 0 for v in lo) and all(v == 0 for v in hi)
        V(clean == (zp >= 2),
          "G4 two transparent towers at z-pitch %d, dy=%d: %s" %
          (zp, dy, "isolated" if zp >= 2 else "LEAK (glass keeps the cross-net "
           "diagonals alive too)"),
          "driven=%s leak=%s" % (lo, hi))


# =============================================================== S. ring riser
# S0 -- the SMALLEST riser that descends: a 2x2 ring, one bit only.
for lbl, drive_top in (("up", False), ("down", True)):
    r = Rig("ring22_%s" % lbl)
    cs = vf.ring_riser(r.b, vf.ring(2, 2), 0, 0, 4, 10)
    r.probe("near", cs[-1] if drive_top else cs[0])
    r.probe("far", cs[0] if drive_top else cs[-1])
    src = cs[-1] if drive_top else cs[0]
    r.lever(src, d=vf.ring_outward(src, 0, 0, 2, 2))
    r.start()
    r.set([1])
    V(r.ss("far") > 0,
      "S0 a 2x2 ring riser carries ONE bit %s in a 4-cell footprint -- the "
      "smallest descending form; period 4 leaves no room for a second bit, "
      "which needs offset >= 3" % lbl,
      "near=%d far=%d" % (r.ss("near"), r.ss("far")))

RG = vf.ring(3, 3)
NS = 12
# S1 up
r = Rig("ring_up")
cs = vf.ring_riser(r.b, RG, 0, 0, 4, NS)
for i, c in enumerate(cs):
    r.probe("c%d" % i, c)
r.lever(cs[0], d=vf.ring_outward(cs[0], 0, 0, 3, 3))
r.start()
r.set([1])
up = [r.ss("c%d" % i) for i in range(NS)]
V(all(v > 0 for v in up) and up[0] - up[-1] == NS - 1,
  "S1 a 3x3 ring riser climbs 1 y per cell on ALL-SOLID supports, -1 ss per y",
  "profile=%s" % up)

# S2 down -- same geometry, driven from the top
r = Rig("ring_down")
cs = vf.ring_riser(r.b, RG, 0, 0, 4, NS)
for i, c in enumerate(cs):
    r.probe("c%d" % i, c)
r.lever(cs[-1], d=vf.ring_outward(cs[-1], 0, 0, 3, 3))
r.start()
r.set([1])
dn = [r.ss("c%d" % i) for i in range(NS)]
V(all(v > 0 for v in dn) and dn[-1] - dn[0] == NS - 1,
  "S2 the SAME ring riser DESCENDS: every cut cell above a dust is air and "
  "every upper dust sits on a conductor, so the diode law passes downhill",
  "profile=%s" % dn)
V(up[0] - up[-1] == dn[-1] - dn[0] == NS - 1,
  "S3 -1 ss per y in BOTH directions -> 14 y of rise or fall per source")

# S4 -- a repeater on a flat cell of the ring refreshes it
r = Rig("ring_repeat")
cs = vf.ring_riser(r.b, RG, 0, 0, 4, 10)
# extend with a flat repeater station on the ring's own path, then keep going
sx, sy, sz = cs[-1]
nxt = RG[(10) % len(RG)]
# the station: a flat cell pair at the top of the climb
r.b.force(sx, sy - 1, sz + 1, STONE)
r.b.force(sx, sy, sz + 1, repeater("north"))       # input from -z, flows +z
r.b.force(sx, sy - 1, sz + 2, STONE)
r.b.force(sx, sy, sz + 2, DUST)
r.probe("refreshed", (sx, sy, sz + 2))
r.probe("pre", cs[-1])
r.lever(cs[0], d=vf.ring_outward(cs[0], 0, 0, 3, 3))
r.start()
r.set([1])
V(r.ss("refreshed") == 15 and r.ss("pre") < 15,
  "S4 a repeater on a flat cell of the ring refreshes it to 15 for an "
  "unbounded climb/descent (cost: 1 cell of rise, 2 gt)",
  "pre=%d post=%d" % (r.ss("pre"), r.ss("refreshed")))


# ======================================================= D. down is asymmetric
r = Rig("asym")
# D1: a lit torch's neighbourhood -- dust above vs dust below
r.b.force(0, 3, 0, STONE)
r.b.force(0, 4, 0, rs.TORCH)
r.b.force(0, 5, 0, STONE)
r.b.force(0, 6, 0, DUST)
r.probe("above_torch", (0, 6, 0))
r.b.force(0, 2, 0, STONE)          # under the torch's attachment
r.b.force(0, 1, 0, STONE)
r.b.force(1, 2, 0, DUST)           # dust on the block UNDER the attachment
r.b.force(1, 1, 0, STONE)
r.probe("below_torch", (1, 2, 0))
# D2/D3: dust weak-powers the block BELOW it; dust cannot read it, a repeater can
r.b.force(10, 5, 0, STONE)
r.b.force(10, 6, 0, DUST)          # driven dust
r.b.force(11, 5, 0, STONE)         # block beside/below: reads nothing
r.b.force(10, 4, 0, DUST)          # dust one level under the powered block
r.b.force(10, 3, 0, STONE)
r.probe("dust_under_block", (10, 4, 0))
r.b.force(11, 4, 0, repeater("west"))    # back faces -X = the weak block above? no:
r.b.force(11, 3, 0, STONE)
r.probe("rep_beside", (11, 4, 0))
# the real D3: the block the dust SITS ON is weakly powered; a repeater whose
# back is that block fires.
r.b.force(9, 5, 0, repeater("east"))     # input from +X == the weak block (10,5,0)
r.b.force(9, 4, 0, STONE)
r.b.force(8, 5, 0, DUST)
r.b.force(8, 4, 0, STONE)
r.probe("rep_out", (8, 5, 0))
r.lever((10, 6, 0), d=(0, -1), n=2)
r.start()
r.set([1])
V(r.ss("above_torch") == 15 and r.ss("below_torch") <= 0,
  "D1 a torch powers the block ABOVE it and nothing beneath -- no torch form "
  "descends, which is why UP has an active carrier and DOWN has none",
  "above=%d below=%d" % (r.ss("above_torch"), r.ss("below_torch")))
V(r.ss("dust_under_block") == 0,
  "D2 a dust weak-powers the block it sits on, and dust CANNOT read weak "
  "power -- so a signal cannot be handed straight down through a block",
  "ss=%d" % r.ss("dust_under_block"))
V(r.ss("rep_out") == 15,
  "D3 ...but a repeater whose BACK is that weakly powered block fires: the "
  "one active way down is 1 y per repeater station",
  "ss=%d" % r.ss("rep_out"))


# ================================================================== H. slopes
# H1/H2 -- a 1:1 staircase, all-solid vs all-glass, both directions
def stair(name, sup, drive_top):
    r = Rig(name)
    n = 12
    cs = [(i, 4 + i, 0) for i in range(n)]
    for (x, y, z) in cs:
        r.b.force(x, y - 1, z, sup)
        r.b.force(x, y, z, DUST)
    r.probe("bottom", cs[0])
    r.probe("top", cs[-1])
    r.lever(cs[-1] if drive_top else cs[0], d=(+1, 0) if drive_top else (-1, 0))
    r.start()
    r.set([1])
    return r.ss("bottom"), r.ss("top")


bu, tu = stair("stair_solid_up", STONE, False)
bd, td = stair("stair_solid_dn", STONE, True)
V(tu > 0 and bd > 0,
  "H1 a 1:1 staircase on ALL-SOLID supports conducts BOTH ways (1 y per x "
  "cell, -1 ss per y)", "up top=%d, down bottom=%d" % (tu, bd))
gu, gt_ = stair("stair_glass_up", GLASS, False)
gb, gtd = stair("stair_glass_dn", GLASS, True)
V(gt_ > 0 and gb == 0,
  "H2 the same staircase on ALL-GLASS supports is a DIODE: up only",
  "up top=%d, down bottom=%d" % (gt_, gb))

# H3/H4 -- two lines 2 y apart on a half slope
for alt in (False, True):
    r = Rig("halfslope_%s" % ("alt" if alt else "allglass"))
    n = 10
    A = vf.half_slope(r.b, 0, 0, 6, n, alternate=alt)
    B = vf.half_slope(r.b, 0, 0, 8, n, alternate=alt)
    r.probe("out0", A[-1])
    r.probe("out1", B[-1])
    r.lever(A[0], d=(-1, 0))
    r.lever(B[0], d=(-1, 0))
    r.start()
    res = r.sweep(["out0", "out1"])
    lo, hi = crosstalk(res, 2)
    clean = all(v and v > 0 for v in lo) and all(v == 0 for v in hi)
    if alt:
        V(clean and r.floating == 0,
          "H4 two lines 2 y apart on a half slope with ALTERNATING supports "
          "(transparent exactly where a dust is the lower end of a step) are "
          "isolated in all 4 patterns", "driven=%s leak=%s" % (lo, hi))
    else:
        V(not clean,
          "H3 the same pair with ALL-TRANSPARENT supports MERGES: the foreign "
          "diagonal from the lower line up to the upper line survives",
          "driven=%s leak=%s" % (lo, hi))


# H5 -- why the slope has to be a HALF slope for a stacked bus
full = vf.lower_end_of_step(vf.slope_levels(10, rise_every=1))
half = vf.lower_end_of_step(vf.slope_levels(10, rise_every=2))
V(all(full[:-1]) and not all(half[:-1]),
  "H5 on a 1:1 slope EVERY dust is the lower end of a step, so the "
  "alternation rule degenerates to all-transparent -- which H3 measures as a "
  "merge.  A half slope is the STEEPEST slope that can carry a second line 2 y "
  "above it: half its dusts are flat cells whose cap may be a conductor",
  "1:1 lower-ends=%s  half lower-ends=%s" % (full, half))
r = Rig("fullslope_2line")
A = vf.half_slope(r.b, 0, 0, 6, 8, alternate=True, rise_every=1)
B = vf.half_slope(r.b, 0, 0, 8, 8, alternate=True, rise_every=1)
r.probe("out0", A[-1])
r.probe("out1", B[-1])
r.lever(A[0], d=(-1, 0))
r.lever(B[0], d=(-1, 0))
r.start()
lo, hi = crosstalk(r.sweep(["out0", "out1"]), 2)
V(any(v > 0 for v in hi) or any(not v for v in lo),
  "H5 ...and measured: two lines 2 y apart on a 1:1 slope cannot be separated "
  "by any support choice the rule allows", "driven=%s leak=%s" % (lo, hi))


# ================================================== M. measured density table
# The numbers `notes-vertical-transport.md` tabulates, computed from the rigs
# rather than by hand.  "per y" counts only the cells a form consumes for each
# extra y of rise (its column cost); "total" is the whole bbox of an 8-bit
# riser including its ports, which are paid once at each end.
def bbox(cells):
    xs = [c[0] for c in cells]
    ys = [c[1] for c in cells]
    zs = [c[2] for c in cells]
    return (max(xs) - min(xs) + 1, max(ys) - min(ys) + 1, max(zs) - min(zs) + 1)


def footprint(cells):
    return len({(c[0], c[2]) for c in cells})


def per_y(cells):
    """Blocks the form spends on ONE mid-height y level -- its marginal cost
    per y of rise.  Taken at the median level so entry/exit ports do not
    pollute it."""
    ys = sorted({c[1] for c in cells})
    y = ys[len(ys) // 2]
    return sum(1 for c in cells if c[1] == y)


def row(name, nbits, cells, reserved, note):
    """reserved = the xz area a router must claim for the form, which for a
    form with a required neighbour pitch is wider than the cells it fills."""
    return (name, nbits, bbox(cells), per_y(cells), reserved, note)


rows = []
# the tower body only -- ports are paid once at each end, not per y of rise
b = rs.Build("m_lad8")
vf.ladder_bus(b, 0, 0, 2, 8, 4)
rows.append(row("torch_ladder x8, x-pitch 1", 8, b.cells, 8 * 3,
                "2 y/torch, inverting, ~1 gt/y, refresh to 15, UP only"))

b = rs.Build("m_tower8")
for i in range(8):
    vf.glass_tower(b, 1, 2 * i, 4, 12)          # required z-pitch 2 (G4)
rows.append(row("glass_tower x8, z-pitch 2", 8, b.cells, 2 * 16,
                "1 y/cell, UP only, 0 gt, -1 ss/y"))

b = rs.Build("m_ring8")
vf.ring_bus(b, 11, 3, 0, 0, 4, 12)
rows.append(row("ring_riser 11x3, 8 bits", 8, b.cells, 11 * 3,
                "1 y/cell, BOTH ways, 0 gt, -1 ss/y"))

b = rs.Build("m_ring53")
vf.ring_bus(b, 5, 3, 0, 0, 4, 12)
rows.append(row("ring_riser 5x3, 4 bits", 4, b.cells, 5 * 3,
                "1 y/cell, BOTH ways, 0 gt, -1 ss/y"))

b = rs.Build("m_ring33")
vf.ring_bus(b, 3, 3, 0, 0, 4, 12)
rows.append(row("ring_riser 3x3, 2 bits (180)", 2, b.cells, 3 * 3,
                "1 y/cell, BOTH ways, 0 gt, -1 ss/y"))

print("--- measured density ---")
print("%-30s %5s %-11s %10s %10s" %
      ("form", "bits", "bbox", "blocks/y/bit", "xz-claim/bit"))
for (name, nbits, bb, py, res, note) in rows:
    print("%-30s %5d %2dx%2dx%-5d %10.3f %10.3f   %s"
          % (name, nbits, bb[0], bb[1], bb[2], py / float(nbits),
             res / float(nbits), note))
print("(blocks/y/bit: blocks spent on one mid-height y level -- the marginal "
      "cost per y of\n rise.  xz-claim/bit: the xz area a router must reserve, "
      "including each form's\n required neighbour pitch and, for the ladder "
      "row, the two port lanes that are paid\n once at each end rather than "
      "per y of rise.)")
print()

# =================================================================== report
bad = 0
for ok, text, extra in VERDICTS:
    print("%s %s%s" % ("PASS" if ok else "FAIL", text,
                       ("   [%s]" % extra) if extra else ""))
    bad += 0 if ok else 1
print("probe_vertical_forms: %d/%d" % (len(VERDICTS) - bad, len(VERDICTS)))
raise SystemExit(1 if bad else 0)
