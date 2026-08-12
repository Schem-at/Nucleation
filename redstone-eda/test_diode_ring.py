"""Guard: a route must never join the rail it drives.

The hazard, found the hard way.  `router.emit` refreshes a route at whatever
straight cell it reaches once the signal budget is spent, so moving the refresh
pitch moves every repeater in every route.  When `REFRESH` went 5 -> 14
(commit 5076520c) mult4's `m1B2` route was re-pathed straight along the rail it
was supposed to DRIVE.  `dust_ok` allowed it -- same net label, and own-net
reuse is normally fine -- but a compiled rail is fed THROUGH a repeater, so the
reused cells sat downstream of a diode the route then fed from the other end:

    rail driver -> rail cell -> route tail -> drive cell -> rail driver

A ring containing a repeater LATCHES at 15.  The net read high forever, no input
could clear it, and because the world was quiescent `run_until_quiescent` saw
nothing to do.  mult4's output bit 3 stuck at 1 and the multiplier scored 64/256
while every structural check (audit, net shorts, quiescence) stayed green.

Four sections, cheapest first, none of them needing the full 256-case proof.
1-3 pin down the router-specific fix; 4 covers the DEFECT CLASS -- `drc.py`
generalises the rule to any diode ring anywhere, formed by any builder, which is
the check that would have caught this without knowing anything about rails:

  1. PHYSICS -- hand-build the ring and simulate it, so the rule rests on a
     measurement.  A ring with a diode latches; the same ring without one does
     not.
  2. RULE    -- the router refuses to join a rail that sits behind a diode, and
     still permits own-net reuse of a diode-FREE rail (test_router needs that).
  3. DESIGN  -- mult4's real routed geometry has no such contact anywhere.
  4. RULE (general) -- `drc.repeater_cycles` flags a ring formed any way at all
     (bare dust, a self-loop on one component, a block-sandwich station) and
     leaves legal geometry alone (forward chains, torch loops).  Its regression
     proof strips `router.downstream_rail` to rebuild the PRE-FIX tree and
     confirms the rule reports the ring, on net `m1B2`, and nothing on the
     fixed tree.

    python test_diode_ring.py [--exhaustive]

`--exhaustive` additionally re-runs mult4's own 256/256 proof (minutes, not
seconds), which is the thing these cheap checks stand in for.
"""
import sys

import nets
import router
import rs

FAILS = []


def check(cond, what):
    print("%-4s %s" % ("PASS" if cond else "FAIL", what))
    if not cond:
        FAILS.append(what)


# --------------------------------------------------------------- 1. physics
def ring_build(with_diode):
    """A drive cell whose rail loops back into it, with or without the diode.

    Layout at y=1 (supports at y=0), rail running +X from the driver:

        (0,1,0) D  drive cell        (1,1,0) repeater facing=west -> +X
        (2,1,0) rail cell            return path (2,1,1) (1,1,1) (0,1,1) -> D
        (4,1,1) lever -> (3,1,1) injects into the return path
    """
    b = rs.Build("ring")
    for p in [(0, 1, 0), (2, 1, 0), (2, 1, 1), (1, 1, 1), (0, 1, 1), (3, 1, 1)]:
        b.dust(*p)
    if with_diode:
        b.stone(1, 0, 0)
        b.put(1, 1, 0, rs.repeater("west"))     # reads (0,1,0), drives (2,1,0)
    else:
        b.dust(1, 1, 0)                         # plain dust: no diode in the ring
    b.stone(4, 0, 1)
    b.force(4, 1, 1, rs.LEVER_OFF)
    return b


def test_physics():
    for with_diode in (True, False):
        b = ring_build(with_diode)
        sim = b.sim(settle=400)
        lv = rs.Levers(sim, [(4, 1, 1)])
        lv.set([1])
        hot = sim.power(2, 1, 0)
        lv.set([0])
        held = sim.power(2, 1, 0)
        tag = "with a diode" if with_diode else "without a diode"
        check(hot > 0, "ring %s energises while the lever is on (%d)" % (tag, hot))
        if with_diode:
            check(held > 0, "ring WITH a diode latches: lever off, rail still %d"
                            " -- this is the failure the rule prevents" % held)
        else:
            check(held == 0, "ring without a diode clears: lever off, rail %d"
                             % held)


# ------------------------------------------------------------------ 2. rule
def rail_scene(with_diode):
    """A compiled-style destination rail, optionally fed through a repeater.

    Returns (router, src, dst, rail_cells).
    """
    b, labels = rs.Build("rail"), {}
    b.stone(0, 0, 8)
    b.force(0, 1, 8, rs.LEVER_OFF)
    b.dust(1, 1, 8)
    labels[(1, 1, 8)] = "src"
    src = (1, 1, 8)

    drive = (8, 13, 1)
    b.dust(*drive)
    labels[drive] = "net"
    first_rail = 10 if with_diode else 9
    if with_diode:
        b.stone(9, 12, 1)
        b.put(9, 13, 1, rs.repeater("west"))    # reads the drive cell, feeds +X
    rail = []
    for x in range(first_rail, first_rail + 5):
        b.dust(x, 13, 1)
        labels[(x, 13, 1)] = "net"
        rail.append((x, 13, 1))
    r = router.Router(b, labels)
    # emulate find()'s preamble so dust_ok can be asked directly
    r._dsts = {drive}
    r._dst_blob = r.dest_blob({drive}, "net") | r.dest_blob({src}, "net")
    return r, src, drive, rail


def test_rule():
    r, src, drive, rail = rail_scene(with_diode=True)
    check(all(r.downstream_rail(q, "net") for q in rail),
          "a rail behind a diode is downstream: all %d cells refused" % len(rail))
    check(not r.downstream_rail(drive, "net"),
          "the drive cell itself stays reusable (it is the route's endpoint)")
    mid = rail[2]
    check(not r.dust_ok(mid, "net", friendly={"src", "net"}),
          "stepping INTO a downstream rail cell is refused")
    graze = (mid[0], mid[1], mid[2] + 1)
    check(not r.dust_ok(graze, "net", friendly={"src", "net"}),
          "merely GRAZING a downstream rail cell is refused too")

    r2, src2, drive2, rail2 = rail_scene(with_diode=False)
    check(not any(r2.downstream_rail(q, "net") for q in rail2),
          "a diode-FREE rail is one node with its drive cell: reuse still legal")
    mid2 = rail2[2]
    check(r2.dust_ok(mid2, "net", friendly={"src", "net"}),
          "own-net reuse of a diode-free rail still permitted (test_router)")
    check(not r2.downstream_rail(src2, "net"),
          "the route's own SOURCE is never treated as downstream rail")


# ---------------------------------------------------------------- 3. design
def test_mult4_design():
    import mult4
    d = mult4.build()
    cells, labels, prewired = d["b"].cells, d["labels"], d["prewired"]
    # the compiled rail structure, i.e. connectivity before any routing
    pre_cells = {p: cells[p] for p in prewired if p in cells}
    bad, downstream_total = [], 0
    for _sig, dst_sig, _src, dst in d["routed"]:
        # the endpoint's electrical node within the COMPILED geometry: the
        # closure stops at the rail's driver repeater, so everything beyond it
        # is the span this route drives and must keep clear of
        node, stack = {dst}, [dst]
        while stack:
            for q in nets.neighbours(pre_cells, stack.pop()):
                if q not in node and labels.get(q) == dst_sig:
                    node.add(q)
                    stack.append(q)
        driven = {p for p, lab in labels.items()
                  if lab == dst_sig and p in prewired and p not in node}
        downstream_total += len(driven)
        emitted = {p for p, lab in labels.items()
                   if lab == dst_sig and p not in prewired}
        for p in emitted:
            if p in driven or any(q in driven for q in nets.neighbours(cells, p)):
                bad.append((dst_sig, p))
    check(not bad,
          "no routed cell touches the rail its net drives (%d nets, %d driven "
          "rail cells in reach)" % (len(d["routed"]), downstream_total)
          + ("" if not bad else " -- offenders: %s" % bad[:4]))
    check(downstream_total > 0,
          "the check has teeth: %d driven rail cells exist to collide with"
          % downstream_total)
    check(len(cells) > 20000,
          "mult4 still builds (%d blocks, %d routed cells)"
          % (len(cells), d["routed_cells"]))


# -------------------------------------------------- 4. the general DRC rule
def test_drc_rule():
    """`drc.repeater_cycles` must flag the ring shape wherever it appears.

    Sections 1-3 are all about ONE way to form the ring: a router joining the
    rail it drives.  The rule below does not care how the ring was formed, so
    it is what actually closes the defect class -- and it is the check every
    builder now runs.
    """
    import drc

    # the very geometry section 1 MEASURED as latching
    check(len(drc.repeater_cycles(ring_build(True).cells)) == 1,
          "the ring section 1 measured as latching is flagged (exactly 1)")
    check(drc.repeater_cycles(ring_build(False).cells) == [],
          "the same ring with plain dust instead of the diode is not flagged")

    # a forward chain of diodes is not a cycle, however long
    b = rs.Build("chain")
    for k in range(4):
        b.dust(3 * k, 1, 0)
        b.stone(3 * k + 1, 0, 0)
        b.put(3 * k + 1, 1, 0, rs.repeater("west"))
        b.dust(3 * k + 2, 1, 0)
    check(drc.repeater_cycles(b.cells) == [],
          "a 4-deep forward chain of repeaters is not a cycle")

    # a diode whose input and output land in the SAME dust component: the mult4
    # shape reduced to its essence, and a self-loop the DFS must still catch
    b = rs.Build("selfloop")
    for p in [(0, 1, 0), (2, 1, 0), (2, 1, 1), (1, 1, 1), (0, 1, 1)]:
        b.dust(*p)
    b.stone(1, 0, 0)
    b.put(1, 1, 0, rs.repeater("west"))
    check(len(drc.repeater_cycles(b.cells)) == 1,
          "a diode bridging one dust component (self-loop) is flagged")

    # a ring closed through a block-sandwich station: the router emits these at
    # full pitch now, and the bare dust-adjacency rule cannot see them
    b = rs.Build("station")
    for p in [(0, 1, 0), (5, 1, 0), (5, 1, 1), (4, 1, 1), (3, 1, 1), (2, 1, 1),
              (1, 1, 1), (0, 1, 1)]:
        b.dust(*p)
    b.stone(1, 1, 0)                        # entry block (dust at (0,1,0) points in)
    b.stone(2, 0, 0)
    b.put(2, 1, 0, rs.repeater("west"))     # reads the entry block
    b.stone(3, 1, 0)                        # exit block: strongly powered, re-emits 15
    b.dust(4, 1, 0)
    check(len(drc.repeater_cycles(b.cells)) == 1,
          "a ring closed through a block/repeater/block station is flagged")

    # torches are NOT diodes here: an inverter ring oscillates, and the NOR
    # gadgets in rca_cells/seq_cells are built from torches on purpose
    b = rs.Build("torchring")
    for p in [(0, 1, 1), (2, 1, 1)]:
        b.dust(*p)
    b.stone(1, 0, 0)
    b.put(1, 1, 0, rs.TORCH)
    check(drc.repeater_cycles(b.cells) == [],
          "a torch loop is not reported (inverters oscillate; NOR gadgets are legal)")


def test_drc_catches_prefix_mult4():
    """The regression proof: on the PRE-FIX geometry the rule fires.

    `router.downstream_rail` is what refuses the bad contact today, so removing
    it reproduces the tree as it stood when mult4 scored 64/256.  The rule must
    report the ring there and stay silent on the fixed build -- otherwise it is
    not the check that would have caught this.
    """
    import drc
    import router

    keep = router.Router.downstream_rail
    try:
        router.Router.downstream_rail = lambda self, q, label: False
        import mult4
        pre = mult4.build()
        rings = drc.repeater_cycles(pre["b"].cells)
        labels = pre["labels"]
        nets_hit = sorted({labels.get(q)
                           for ring in rings for p in ring
                           for q in _diode_ends(pre["b"].cells, p)
                           if labels.get(q)})
        check(len(rings) == 1,
              "pre-fix mult4 (no downstream_rail guard): %d ring(s) reported, "
              "want 1" % len(rings))
        check(nets_hit == ["m1B2"],
              "the ring sits on the net the bug was reported against: %s"
              % (nets_hit or "none"))
    finally:
        router.Router.downstream_rail = keep

    import mult4
    post = mult4.build()          # guard restored: a fresh build of the real tree
    check(drc.repeater_cycles(post["b"].cells) == [],
          "the fixed mult4 build reports no ring")


def _diode_ends(cells, p):
    import drc
    io = drc.diode_io(p, cells[p])
    return io if io else ()


def main():
    test_physics()
    test_rule()
    test_mult4_design()
    test_drc_rule()
    test_drc_catches_prefix_mult4()
    if "--exhaustive" in sys.argv:
        import mult4
        print("\n--- exhaustive 256/256 (slow) ---")
        check(mult4.main() == 0, "mult4 exhaustive proof still 256/256")
    print("\ntest_diode_ring: %s" % ("PASS" if not FAILS else
                                     "FAIL (%d)" % len(FAILS)))
    return 1 if FAILS else 0


if __name__ == "__main__":
    raise SystemExit(main())
