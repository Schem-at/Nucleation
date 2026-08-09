"""Showcase geometry for the two headline vertical-transport forms.

Self-verifying: each piece is simulated over its full lever matrix and only
saved if every bit conducts and no undriven bit sees any power ANYWHERE in its
cells.  Physics + rankings: `../notes-vertical-transport.md`.
Templates: `../vforms.py`.  Probes: `../probe_vertical_forms.py`,
`../probe_spiral_tiling.py`.

  vriser_ladder8.schem  8-bit torch-ladder riser, towers at x-PITCH 1 with
                        ports on alternating sides -- 1 block per y per bit,
                        the densest legal riser.  UP only, inverts per torch.
  vriser_ring53.schem   4-bit spiral-staircase riser on one 5x3 ring, bits at
                        path-offset 3 (the generalisation of "offset 180
                        degrees") -- the densest form that also goes DOWN.
"""
import itertools
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, ROOT)

import rs                      # noqa: E402
import audit                   # noqa: E402
import vforms as vf            # noqa: E402
from rs import DUST, STONE, LEVER_OFF, repeater   # noqa: E402


def check(name, b, levs, outs, nets):
    floating = sum(len(v) for v in audit.audit(b.cells).values())
    sim = b.sim()
    lv = rs.Levers(sim, levs)
    nb = len(levs)
    lo = [None] * nb
    hi = [0] * nb
    pats = list(itertools.product((0, 1), repeat=nb))
    for bits in pats:
        lv.set(list(bits))
        for j in range(nb):
            v = sim.power(*outs[j])
            if bits[j]:
                lo[j] = v if lo[j] is None else min(lo[j], v)
            else:
                hi[j] = max([v] + [sim.power(*c) for c in nets[j]] + [hi[j]])
    lv.set([0] * nb)
    ok = (all(v and v > 0 for v in lo) and all(v == 0 for v in hi)
          and floating == 0)
    print("%s %-22s %d bits, %d patterns, driven=%s leak=%s floating=%d"
          % ("PASS" if ok else "FAIL", name, nb, len(pats), lo, hi, floating))
    return ok


def ladder8():
    b = rs.Build("vriser_ladder8")
    levs, outs, nets = [], [], []
    ports = vf.ladder_bus(b, 0, 0, 2, 8, 4)
    for i, (entry, exit_) in enumerate(ports):
        side = -1 if i % 2 == 0 else +1
        # lever -> 2 dust -> entry
        for k in (1, 2):
            b.force(i, entry[1] - 1, entry[2] + k * side, STONE)
            b.force(i, entry[1], entry[2] + k * side, DUST)
        lc = (i, entry[1], entry[2] + 3 * side)
        b.force(lc[0], lc[1] - 1, lc[2], STONE)
        b.force(*lc, LEVER_OFF)
        levs.append(lc)
        # exit -> 2 dust readout
        cells = [exit_]
        for k in (1, 2):
            b.force(i, exit_[1] - 1, exit_[2] + k * side, STONE)
            b.force(i, exit_[1], exit_[2] + k * side, DUST)
            cells.append((i, exit_[1], exit_[2] + k * side))
        outs.append(cells[-1])
        nets.append(cells)
    return b, levs, outs, nets


def ring53():
    SX, SZ, Y0, N = 5, 3, 4, 10
    b = rs.Build("vriser_ring53")
    levs, outs, nets = [], [], []
    for p in vf.ring_bits(2 * (SX + SZ) - 4, 3):
        cs = vf.ring_riser(b, vf.ring(SX, SZ), 0, 0, Y0, N, p)
        nets.append(cs)
        d = vf.ring_outward(cs[0], 0, 0, SX, SZ)
        back = {(0, -1): "north", (0, 1): "south",
                (-1, 0): "west", (1, 0): "east"}[d]
        x, y, z = cs[0]
        b.force(x + d[0], y - 1, z + d[1], STONE)
        b.force(x + d[0], y, z + d[1], repeater(back))
        for k in (2, 3):
            b.force(x + d[0] * k, y - 1, z + d[1] * k, STONE)
            b.force(x + d[0] * k, y, z + d[1] * k, DUST)
        lc = (x + d[0] * 4, y, z + d[1] * 4)
        b.force(lc[0], lc[1] - 1, lc[2], STONE)
        b.force(*lc, LEVER_OFF)
        levs.append(lc)
        e = vf.ring_outward(cs[-1], 0, 0, SX, SZ)
        x, y, z = cs[-1]
        for k in (1, 2):
            b.force(x + e[0] * k, y - 1, z + e[1] * k, STONE)
            b.force(x + e[0] * k, y, z + e[1] * k, DUST)
        outs.append((x + e[0] * 2, y, z + e[1] * 2))
    return b, levs, outs, nets


bad = 0
for name, mk in (("vriser_ladder8", ladder8), ("vriser_ring53", ring53)):
    b, levs, outs, nets = mk()
    if check(name, b, levs, outs, nets):
        path = os.path.join(HERE, name + ".schem")
        b.s.save_to_file(path)
        print("     saved %s (%d blocks)" % (path, len(b.cells)))
    else:
        bad += 1
print("showcase_vertical: %d/2" % (2 - bad))
raise SystemExit(1 if bad else 0)
