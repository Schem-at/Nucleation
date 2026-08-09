"""HexAnalog as a BUS FORM: an 8-bit word carried as TWO analog wires.

Each nibble rides ONE wire as signal strength 0-15 (the verified E4
primitives, compositor/hexanalog.py): binary port -> encoder (staged
comparator subtraction) -> one-wire analog trunk -> 3-stage decoder ->
binary port.  Two such channels side by side ARE the 8-bit bus:

  * trunk: 28 cells (x=9..36), alternating [comparator][dust] (a comparator
    regenerates its rear value losslessly -- probed rig B) with ONE
    comparator-SANDWICH station mid-trunk (dust -> block -> comparator ->
    block -> dust preserves exact ss -- probed here, and probe_station
    C1/C2).  The sandwich is the analog analogue of the binary bus's
    block-sandwich repeater station: it lets an analog trunk pass through
    a solid wall / under a perpendicular line exactly like bus8's.
  * channels are fully independent: bands 60 z apart (the encoder's
    exact-strength decay lanes sprawl to z = +/-20; the trunks themselves
    are 1 block wide -- see FOOTPRINT below).

FOOTPRINT vs the dense binary form (bus8_run): the binary bus moves 8 bits
as 8 stacked wires (1 wide x 16 tall cross-section, 8 repeaters per refresh
station, 1 gt latency per station).  The HexAnalog form moves the same word
on TWO wires (cross-section 2 x [1x2] -- wires may run 2 z apart since
comparators must not see foreign side inputs); the cost moves into TIME and
PORTS: every trunk comparator adds 1 gt (13 comps + 1 station here vs 2
repeaters on a same-length binary run), and each end pays a ~40x40-cell
encoder/decoder.  Use it where cross-section is the scarce resource.

Verified here: mini-probe (subtract-mode sandwich exact for a non-trivial
ss), the E4 primitive probes, then the full 2-channel bus EXHAUSTIVELY:
all 256 bytes in Gray-code order (one lever flip per step), each checking
BOTH trunk exit strengths exactly AND all 8 decoded bits.
Saved BAKED at rest (showcase/hexanalog_bus8.schem) only on green.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)

import nucleation as n            # noqa: E402
import rs                         # noqa: E402
import nets                       # noqa: E402
from hexanalog import (COMP, build_decoder, build_encoder, build_trunk,
                       comp, dust, torch, probe)          # noqa: E402
from seq_probe import bake_states                          # noqa: E402

DZ = 60                           # z pitch between the two channel bands
P0X = 36                          # trunk exit / decoder origin


def probe_sandwich():
    """dust(ss11) -> block -> SUBTRACT comp -> block -> dust == 11 exactly
    (C1/C2 probed the compare-mode sandwich; the trunk uses subtract)."""
    b = rs.Build("p_sandwich")
    labels = {}
    torch(b, 0, 0)
    for x in range(1, 6):
        dust(b, labels, x, 0, "a")            # (5,1,0) reads 11
    b.put(6, 1, 0, rs.STONE)                  # entry block
    b.stone(7, 0, 0)
    b.put(7, 1, 0, COMP % "west")
    b.put(8, 1, 0, rs.STONE)                  # exit block
    dust(b, labels, 9, 0, "out")
    sim = b.sim()
    got = sim.power(9, 1, 0)
    ok = got == 11
    print("%s subtract-comparator sandwich preserves ss11 (got %d)"
          % ("PASS" if ok else "FAIL", got), flush=True)
    return ok


def build_channel(tag):
    """One nibble channel in its own Build.  Returns (cells, levers, bits)
    with levers = [b0..b3] positions, bits = decoded port positions."""
    b = rs.Build(tag)
    labels = {}
    levers = build_encoder(b, labels)         # encoder out S at (8,1,0)
    build_trunk(b, labels, 9, 21)             # comps 9,11..21; dust between
    dust(b, labels, 22, 0, "trunk")
    b.put(23, 1, 0, rs.PALETTE["route"])      # comparator-sandwich station
    b.stone(24, 0, 0)
    b.put(24, 1, 0, COMP % "west")
    b.put(25, 1, 0, rs.PALETTE["route"])
    dust(b, labels, 26, 0, "trunk")
    build_trunk(b, labels, 27, 35)            # comps 27,29..35
    dust(b, labels, P0X, 0, "trunk")          # P0 = trunk exit
    bits = build_decoder(b, labels, P0X)
    shorts = nets.check(b.cells, labels)
    assert not shorts, shorts[:4]
    return b.cells, [levers[i] for i in range(4)], bits


def main():
    if not probe_sandwich() or not probe():
        return False

    lo_cells, lo_levers, lo_bits = build_channel("nib_lo")
    hi_cells, hi_levers, hi_bits = build_channel("nib_hi")

    comb = rs.Build("hexanalog_bus8")
    for (x, y, z), blk in lo_cells.items():
        comb.put(x, y, z, blk)
    for (x, y, z), blk in hi_cells.items():
        comb.put(x, y, z + DZ, blk)

    sim = comb.sim(settle=800)
    shift = lambda p: (p[0], p[1], p[2] + DZ)
    lv = rs.Levers(sim, lo_levers + [shift(p) for p in hi_levers])
    hi_bits = [shift(p) for p in hi_bits]
    trunk_lo, trunk_hi = (P0X, 1, 0), (P0X, 1, DZ)

    patterns = [g ^ (g >> 1) for g in range(256)]     # Gray order: 1 flip/step
    good = total = 0
    for byte in patterns:
        vlo, vhi = byte & 0xF, byte >> 4
        lv.set([(byte >> i) & 1 for i in range(8)], settle=800)
        elo, ehi = sim.power(*trunk_lo), sim.power(*trunk_hi)
        got = (sum(int(sim.on(*lo_bits[i])) << i for i in range(4))
               | (sum(int(sim.on(*hi_bits[i])) << i for i in range(4)) << 4))
        ok = elo == vlo and ehi == vhi and got == byte
        if not ok:
            print("FAIL byte %02X trunk lo=%2d hi=%2d decoded %02X"
                  % (byte, elo, ehi, got), flush=True)
        good += 1 if ok else 0
        total += 1
    print("hexanalog_bus8: %d/%d bytes EXHAUSTIVE (Gray order; each = 2 "
          "exact trunk strengths + 8 decoded bits)" % (good, total))
    if good != total:
        return False

    lv.set([0] * 8, settle=800)
    assert sim.power(*trunk_lo) == 0 and sim.power(*trunk_hi) == 0
    out = os.path.join(os.path.dirname(HERE), "showcase",
                       "hexanalog_bus8.schem")
    baked = bake_states(comb, sim)
    baked.save_to_file(out)
    print("saved %s" % out)
    return True


if __name__ == "__main__":
    ok = main()
    print("hexanalog_bus8:", "ALL PASS" if ok else "FAILURES")
    raise SystemExit(0 if ok else 1)
