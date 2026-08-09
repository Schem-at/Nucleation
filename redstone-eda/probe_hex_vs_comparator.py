"""probe_hex_vs_comparator -- head-to-head over a MATCHED z-span.

The claim under test (the author's own words): the hex repeater bus is "wider and
less easy to fit but way faster than alternating comparators dusts and blocks".

Both carriers move an ANALOG 0..15 value, are driven by the same lever+attenuator
source, and are measured between the same two reference points:

    INJECT   the cell the source comparator drives (analog value v lands here)
    READ     the cell the carrier's last device drives (analog value read here)

span = number of z-blocks between INJECT and READ.  Latency = game ticks from the
source lever flip to READ showing the correct value, minus the 2 gt the shared
source comparator costs, so both carriers are charged only for themselves.

Groups:
  C0  which comparator-chain link is lossless (solid block / 1 dust / 2 dust)
  C1  comparator chain: latency vs span  -> gt per block
  C2  hex bus: latency vs span (1 and 2 ping-pong stages)
  C3  the table: matched spans, latency, cross-section, block cost
  C4  pulse-width filtering of each carrier

Run:  python3 probe_hex_vs_comparator.py
"""
import sys

import rs
from probe_hex_transmit import (LEVER, PASS, Rig, check, comparator, note, trace)


# --------------------------------------------------------------- the reference
class CompChain:
    """The 'alternating comparators / dusts / blocks' analog line.

    link  the cells between two comparators: "B" one solid block, "D" one dust,
          "DD" two dust.  Each repetition is len(link)+1 z-cells and one
          comparator.
    Runs along -z at x=0, y=1, on a solid floor at y=0.
    """

    ZSRC = 3          # attenuator z
    ZINJ = 1          # the source comparator's output cell == INJECT

    def __init__(self, link="B", span=16, value=15, name=None):
        self.link, self.span, self.value = link, span, value
        b = rs.Build(name or "cchain_%s_%d" % (link, span))
        # shared source: attenuator dust run + one lever injector for `value`
        for x in range(0, 15):
            b.dust(x, 1, self.ZSRC)
        b.stone(15 - value, 0, self.ZSRC + 1)
        b.put(15 - value, 1, self.ZSRC + 1, LEVER)
        self.lever = (15 - value, 1, self.ZSRC + 1)
        b.stone(0, 0, self.ZSRC - 1)
        b.put(0, 1, self.ZSRC - 1, comparator("south"))   # reads ZSRC, drives ZINJ
        # the chain
        z = self.ZINJ
        self.ncomp = 0
        zend = self.ZINJ - span
        while z > zend:
            for ch in link:
                if z <= zend:
                    break
                if ch == "B":
                    b.stone(0, 1, z)
                    b.stone(0, 0, z)
                else:
                    b.dust(0, 1, z)
                z -= 1
            if z <= zend:
                break
            b.stone(0, 0, z)
            b.put(0, 1, z, comparator("south"))
            self.ncomp += 1
            z -= 1
        # READ cell: the last comparator drives z (already decremented past it)
        self.read = (0, 1, z)
        b.dust(0, 1, z)
        self.actual_span = self.ZINJ - z
        self.inj = (0, 1, self.ZINJ)
        # the INJECT cell may be a solid block (link="B"), which has no power=
        # property, so the "value going in" is read off the attenuator instead.
        self.src = (0, 1, self.ZSRC)
        self.b = b
        self.s = b.sim()
        self.L = rs.Levers(self.s, [self.lever])
        # carrier-only block cost: everything at z <= ZINJ (excludes the shared
        # source comparator and attenuator)
        self.cells = len([p for p in b.cells if p[2] <= self.ZINJ])

    def on(self):
        self.L.set([1])

    def read_v(self):
        return self.s.power(*self.read)


def latency(rig_obj, lever, read, want, budget=90):
    """gt from the lever flip to `read` showing `want`, minus the 2 gt of the
    shared source comparator."""
    s = rig_obj.s
    s.use(*lever)
    tr = trace(s, {"r": read}, budget, stop_stable=99)["r"]
    at = next((i for i, x in enumerate(tr) if x == want), None)
    return (None if at is None else at - 2), tr


# ------------------------------------------------------------------- C0 link
def c0_links():
    print("C0  which comparator-chain link is lossless?")
    for link in ("B", "D", "DD", "BD"):
        rows = []
        for v in (3, 8, 15):
            c = CompChain(link=link, span=8, value=v)
            c.on()
            rows.append((v, c.s.power(*c.src), c.read_v(), c.ncomp))
        note("C0.1 link=%-2s span=8 -> (v, on the attenuator, read, #comparators)"
             % link, rows)
        if link in ("B", "D"):
            check("C0.2 link=%s (pitch 2) is LOSSLESS" % link,
                  [(vi, vr) for _, vi, vr, _ in rows],
                  [(v, v) for v in (3, 8, 15)])
        else:
            check("C0.3 link=%s (pitch %d) LOSES ss" % (link, len(link) + 1),
                  all(vr < vi for _, vi, vr, _ in rows), True)
            note("C0.4 link=%s loss per stage" % link,
                 [(vi, vr) for _, vi, vr, _ in rows])
    print()


# ------------------------------------------------------------- C1 chain speed
def c1_chain_speed():
    print("C1  comparator chain: latency vs span")
    rows = []
    for span in (4, 8, 16, 32):
        c = CompChain(link="B", span=span, value=8)
        lat, tr = latency(c, c.lever, c.read, 8)
        rows.append((c.actual_span, c.ncomp, lat, c.cells))
        note("C1.1 span=%2d  comparators=%2d  latency=%s gt  blocks=%d"
             % (c.actual_span, c.ncomp, lat, c.cells))
    check("C1.2 latency == 2 gt per comparator == 1 gt per block of span",
          [(sp, lat) for sp, nc, lat, _ in rows],
          [(sp, sp) for sp, nc, lat, _ in rows])
    return rows


# --------------------------------------------------------------- C2 hex speed
class HexBus:
    """n ping-pong hex stages sharing one 3-wide envelope.

    Stage k's INPUT lane, comb and OUTPUT lane alternate between x=0 and x=2; the
    repeater comb always sits in the middle column x=1, so the cross-section
    stays 3 x 2 no matter how many stages are chained.  Each stage is 15 comb
    rungs plus one output comparator == 16 z-cells, and re-injects the analog
    value at full strength.
    """

    def __init__(self, nstages=1, value=8):
        b = rs.Build("hexbus_%d" % nstages)
        self.value = value
        ZTOP = 16
        # shared source: attenuator + one injector
        for x in range(0, 15):
            b.dust(x, 1, ZTOP + 2)
        b.stone(15 - value, 0, ZTOP + 3)
        b.put(15 - value, 1, ZTOP + 3, LEVER)
        self.lever = (15 - value, 1, ZTOP + 3)
        b.stone(0, 0, ZTOP + 1)
        b.put(0, 1, ZTOP + 1, comparator("south"))
        self.inj = (0, ZTOP)          # (x, z) of the first comb top
        ztop = ZTOP
        xin = 0
        for k in range(nstages):
            xout = 2 - xin
            face = "west" if xin == 0 else "east"
            for z in range(ztop - 14, ztop + 1):
                b.dust(xin, 1, z)
                b.stone(1, 0, z)
                b.put(1, 1, z, rs.repeater(face, 1))
                b.dust(xout, 1, z)
            zc = ztop - 15
            b.stone(xout, 0, zc)
            b.put(xout, 1, zc, comparator("south"))
            b.dust(xout, 1, zc - 1)
            self.read = (xout, 1, zc - 1)
            ztop = zc - 1
            xin = xout
        self.span = ZTOP - (ztop)          # comb top -> final readout
        self.b = b
        # carrier-only cost: everything at z <= ZTOP
        self.cells = len([p for p in b.cells if p[2] <= ZTOP])
        self.devices = nstages * 15
        self.s = b.sim()
        self.L = rs.Levers(self.s, [self.lever])


def c2_hex_speed():
    print("C2  hex bus: latency vs span")
    out = {}
    for nst in (1, 2, 3):
        h = HexBus(nst, 8)
        lat, tr = latency(h, h.lever, h.read, 8, 80)
        note("C2.1 %d stage(s): span=%2d latency=%s gt blocks=%d trace=%s"
             % (nst, h.span, lat, h.cells, tr[:16]))
        check("C2.2 %d stage(s): %d blocks of z for %d gt"
              % (nst, h.span, 4 * nst), (h.span, lat), (16 * nst, 4 * nst))
        check("C2.3 %d stage(s): value intact" % nst, h.s.power(*h.read), 8)
        out[nst] = (h.span, lat, h.cells, h.devices + 2 * nst)
    return out


# ----------------------------------------------------------------- C3 table
def c3_table(hexres):
    print("C3  the table -- matched spans, both carriers analog-lossless")
    print("  | carrier | span (z) | latency (gt) | gt/block | width | height "
          "| devices | blocks | blocks/span |")
    print("  |---|---|---|---|---|---|---|---|---|")
    chain = {}
    for nst, (span, hlat, hcells, hdev) in sorted(hexres.items()):
        c = CompChain(link="B", span=span, value=8)
        clat, _ = latency(c, c.lever, c.read, 8, 120)
        chain[span] = (clat, c.ncomp, c.cells)
        print("  | comparator+block chain | %d | %s | %.3f | 1 | 2 | %d | %d "
              "| %.1f |" % (span, clat, clat / span, c.ncomp, c.cells,
                            c.cells / span))
        print("  | hex repeater comb x%d | %d | %s | %.3f | 3 | 2 | %d | %d "
              "| %.1f |" % (nst, span, hlat, hlat / span, hdev, hcells,
                            hcells / span))
    for nst, (span, hlat, hcells, hdev) in sorted(hexres.items()):
        clat, cnc, ccells = chain[span]
        note("C3.1 span=%2d: hex %d gt vs chain %d gt -> %.1fx faster, "
             "%.1fx the blocks, 3x the width"
             % (span, hlat, clat, clat / hlat, hcells / ccells))
        check("C3.2 span=%d: the hex bus IS faster" % span, hlat < clat, True)
        check("C3.3 span=%d: the hex bus IS bulkier" % span,
              hcells > ccells, True)
    print()
    return chain


# ------------------------------------------------------- C3b binary reference
class BinaryLine:
    """The speed limit: dust refreshed by a repeater every 15 cells.

    NOT analog -- the repeater normalises to 15 -- so it is the upper bound the
    hex bus is trying to approach while staying value-preserving.
    """

    def __init__(self, span=32, value=15, pitch=15):
        b = rs.Build("binline_%d_%d_%d" % (span, value, pitch))
        for x in range(0, 15):
            b.dust(x, 1, 3)
        b.stone(15 - value, 0, 4)
        b.put(15 - value, 1, 4, LEVER)
        self.lever = (15 - value, 1, 4)
        b.stone(0, 0, 2)
        b.put(0, 1, 2, comparator("south"))      # same source as the others
        z = 1
        self.ndev = 0
        zend = 1 - span
        while z > zend:
            for _ in range(pitch):
                if z <= zend:
                    break
                b.dust(0, 1, z)
                z -= 1
            if z <= zend:
                break
            b.stone(0, 0, z)
            b.put(0, 1, z, rs.repeater("south", 1))   # input from +z, out to -z
            self.ndev += 1
            z -= 1
        b.dust(0, 1, z)
        self.read = (0, 1, z)
        self.span = 1 - z
        self.cells = len([p for p in b.cells if p[2] <= 1])
        self.b = b
        self.s = b.sim()
        self.L = rs.Levers(self.s, [self.lever])


def c3b_binary():
    print("C3b binary dust+repeater line -- the speed limit, but NOT analog")
    for span in (16, 32, 48):
        bl = BinaryLine(span=span, value=15, pitch=15)
        lat, tr = latency(bl, bl.lever, bl.read, 15, 80)
        note("C3b.1 span=%2d latency=%s gt (%.3f gt/block) repeaters=%d "
             "blocks=%d" % (bl.span, lat, (lat or 0) / bl.span, bl.ndev,
                            bl.cells))
        check("C3b.2 span=%d: a binary line beats the hex bus per block"
              % span, lat < span * 0.25, True)
    # and it destroys the analog value: feed it 8, read 15
    bl = BinaryLine(span=7, value=8, pitch=6)   # read the cell right after rep 1
    bl.L.set([1])
    check("C3b.3 a repeater NORMALISES: feed it 8, read 15",
          (bl.s.power(0, 1, 1), bl.s.power(*bl.read)), (8, 15))
    print()


# --------------------------------------------------------------- C4 pulses
def c4_pulses():
    print("C4  pulse-width filtering")
    c = CompChain(link="B", span=16, value=15)
    c.on()
    c.s.use(*c.lever)
    c.s.sim.step()
    c.s.use(*c.lever)
    tr = trace(c.s, {"r": c.read}, 40, stop_stable=99)["r"]
    note("C4.1 comparator chain, 1-gt LOW pulse at READ", tr[:14])
    check("C4.2 comparator chain also swallows a 1-gt gap",
          all(x == 15 for x in tr), True)
    print()


def main():
    c0_links()
    c1_chain_speed()
    hexres = c2_hex_speed()
    c3_table(hexres)
    c3b_binary()
    c4_pulses()
    print("%d/%d checks passed" % (sum(PASS), len(PASS)))
    return 0 if all(PASS) else 1


if __name__ == "__main__":
    sys.exit(main())
