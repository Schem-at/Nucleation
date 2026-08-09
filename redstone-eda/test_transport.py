"""Unit tests for the mechanism-level transport API (materials.py section 2b).

Every assertion here restates a verdict that probe_transport.py / the older
probes established IN SIMULATION -- these tests only check that the model
agrees with the sim, so a model edit that drifts from physics fails here
before it reaches a build.

Run:  ~/eda-venv/bin/python test_transport.py
      ~/eda-venv/bin/python -m pytest test_transport.py -q
"""
import materials as M
import rs
from rs import DUST, STONE

GLASS = M.GLASS
SLAB_TOP = M.SLAB_TOP
SLAB_BOT = M.SLAB_BOT


# ------------------------------------------------- the three split predicates
def test_the_three_rules_are_separate_cells():
    """Same predicate, three rules -- what used to be one function."""
    for b in (STONE, "minecraft:red_wool"):
        assert M.cuts_step(b) and M.gates_downhill(b) and M.carries_weak(b)
    for b in (GLASS, SLAB_TOP, SLAB_BOT, None):
        assert not M.cuts_step(b)
        assert not M.gates_downhill(b)
        assert not M.carries_weak(b)
    # sturdiness is an INDEPENDENT axis: glass supports but does not conduct,
    # a bottom slab does neither (probe_materials PROBED table)
    assert M.sturdy(GLASS) and not M.conducts(GLASS)
    assert M.sturdy(SLAB_TOP) and not M.conducts(SLAB_TOP)
    assert not M.sturdy(SLAB_BOT) and not M.conducts(SLAB_BOT)
    assert M.sturdy(STONE) and M.conducts(STONE)


def test_step_reads_matches_the_probed_matrix():
    """probe_transport.py group C, all 8 cells, verbatim."""
    want = {
        ("air", "solid", False): True,    # uphill,   clear cut
        ("air", "glass", False): True,
        ("solid", "solid", False): False,  # uphill,   cut cell conducts
        ("solid", "glass", False): False,
        ("air", "solid", True): True,     # downhill, clear cut + solid diode
        ("air", "glass", True): False,    # downhill, transparent diode
        ("solid", "solid", True): False,
        ("solid", "glass", True): False,
    }
    blocks = {"air": None, "solid": STONE, "glass": GLASS}
    for (cut, sup, downhill), exp in want.items():
        got = M.step_reads(blocks[cut], blocks[sup], downhill)
        assert got == exp, (cut, sup, downhill, got, exp)
    # and the two cells are genuinely independent: the diode cell only ever
    # matters downhill, the cut cell always matters
    assert M.step_reads(None, GLASS, False) != M.step_reads(None, GLASS, True)
    assert M.step_reads(STONE, STONE, False) is False


def test_wire_connects_is_the_vanilla_scan():
    #   flat run
    g = {(0, 1, 0): DUST, (1, 1, 0): DUST, (0, 0, 0): STONE, (1, 0, 0): STONE}
    assert M.wire_connects(g, (0, 1, 0), (1, 1, 0))
    #   no planar diagonal (probe_transport P3)
    g2 = dict(g); g2[(1, 1, 1)] = DUST
    assert not M.wire_connects(g2, (0, 1, 0), (1, 1, 1))
    #   1-y step: lower (0,1,0), upper (1,2,0) on a solid support
    up = {(0, 1, 0): DUST, (0, 0, 0): STONE, (1, 1, 0): STONE, (1, 2, 0): DUST}
    assert M.wire_connects(up, (0, 1, 0), (1, 2, 0))       # lower reads up
    assert M.wire_connects(up, (1, 2, 0), (0, 1, 0))       # upper reads down
    #   transparent diode cell: uphill only
    di = dict(up); di[(1, 1, 0)] = GLASS
    assert not M.wire_connects(di, (0, 1, 0), (1, 2, 0))   # no downhill read
    assert M.wire_connects(di, (1, 2, 0), (0, 1, 0))       # uphill survives
    #   conducting CUT cell above the lower dust: dead both ways
    cut = dict(up); cut[(0, 2, 0)] = STONE
    assert not M.wire_connects(cut, (0, 1, 0), (1, 2, 0))
    assert not M.wire_connects(cut, (1, 2, 0), (0, 1, 0))


# --------------------------------------------------------------- can_occupy
def test_can_occupy_checks_support_and_attachment():
    g = {(0, 0, 0): STONE, (1, 0, 0): GLASS, (2, 0, 0): SLAB_BOT}
    assert M.can_occupy("dust", (0, 1, 0), g)[0]
    assert M.can_occupy("dust", (1, 1, 0), g)[0]          # glass supports
    assert not M.can_occupy("dust", (2, 1, 0), g)[0]      # bottom slab does not
    assert not M.can_occupy("dust", (3, 1, 0), g)[0]      # nothing under it
    assert M.can_occupy("repeater", (0, 1, 0), g)[0]
    assert M.can_occupy("torch_floor", (0, 1, 0), g)[0]
    #   a strong block needs nothing under it -- it is not a placed carrier
    assert M.can_occupy("strong_block", (5, 9, 5), {})[0]
    assert M.can_occupy("redstone_block", (5, 9, 5), {})[0]
    #   occupied cells are refused
    assert not M.can_occupy("dust", (0, 0, 0), g)[0]


# ---------------------------------------------------------------- emission
def test_strong_block_lights_all_six_faces_and_chains_to_none():
    e = M.MECH["strong_block"].emission((0, 0, 0))
    assert set(e.values()) == {M.STRONG}
    assert set(e) == {(0, 0, 0)} | {M._add((0, 0, 0), d) for d in M.NB6}
    #   ...and nothing two cells out: strong power does not chain (S1).  The
    #   model encodes that by emitting only to the 6 neighbours, which is why
    #   two strong blocks side by side are non-interfering (S3).
    assert (2, 0, 0) not in e
    a = ("strong_block", (0, 0, 0), (1, 0, 0), "netA")
    b = ("strong_block", (1, 0, 0), (1, 0, 0), "netB")
    assert not M.interferes(a, b)[0]


def test_weak_block_never_reaches_dust_but_always_reaches_a_device():
    weak = ("weak_block", (0, 0, 0), (1, 0, 0), "netA")
    dust_above = ("dust", (0, 1, 0), (1, 0, 0), "netB")
    assert not M.interferes(weak, dust_above)[0]           # W1
    #   a repeater whose BACK is that block does read it
    rep = ("repeater", (1, 0, 0), (1, 0, 0), "netB")       # back at (0,0,0)
    hit, why = M.interferes(weak, rep)
    assert hit, why                                        # W2
    #   ...but the same repeater rotated so its back faces away does not
    rep_away = ("repeater", (1, 0, 0), (-1, 0, 0), "netB")
    assert not M.interferes(weak, rep_away)[0]


def test_repeater_reads_only_its_back():
    strong = ("strong_block", (0, 0, 0), (1, 0, 0), "netA")
    #   standing ON the strong block: no interference (S4)
    on_top = ("repeater", (0, 1, 0), (1, 0, 0), "netB")
    assert not M.interferes(strong, on_top)[0]
    #   beside it, back turned away: no interference (probe_station S)
    beside = ("repeater", (0, 0, 1), (0, 0, -1), "netB")   # back at (0,0,2)
    assert not M.interferes(strong, beside)[0]
    #   beside it with its back ON it: that IS how a station is entered
    entered = ("repeater", (0, 0, 1), (0, 0, 1), "netB")   # back at (0,0,0)
    assert M.interferes(strong, entered)[0]
    #   back TO it: interference, as it must be
    facing = ("repeater", (1, 0, 0), (1, 0, 0), "netB")
    assert M.interferes(strong, facing)[0]


def test_dust_does_not_power_the_block_above_it():
    """W3: the whole reason a lid over a live run can carry a foreign line."""
    e = M.dust_emission((0, 1, 0), pointing=((1, 0, 0), (-1, 0, 0)))
    assert e[(0, 0, 0)] == M.WEAK                 # the block below
    assert (0, 2, 0) not in e                     # the block above: untouched
    assert e[(1, 1, 0)] == M.WEAK and e[(-1, 1, 0)] == M.WEAK
    #   a dust that points along X does not weak-power the block on its Z side
    assert (0, 1, 1) not in e                     # the pointing law


def test_torch_powers_above_not_its_attachment():
    m = M.MECH["torch_floor"]
    e = m.emission((0, 1, 0))
    assert e[(0, 2, 0)] == M.STRONG                        # T1
    assert (0, 0, 0) not in e                              # T2
    assert m.inverts and m.delay_gt == 2                   # T3
    assert m.requires((0, 1, 0))["attach"][0] == (0, 0, 0)


def test_redstone_block_is_a_source_not_a_powered_block():
    e = M.MECH["redstone_block"].emission((0, 0, 0))
    assert set(e.values()) == {M.SOURCE} and len(e) == 7   # B1 (6 faces + self)
    #   a solid support beside it is inert, so a foreign dust may sit on it
    a = ("redstone_block", (0, 0, 0), (1, 0, 0), "netA")
    b = ("dust", (1, 1, 0), (1, 0, 0), "netB")             # on the neighbour
    assert not M.interferes(a, b)[0]                       # B2


def test_dust_dust_interference_needs_the_grid():
    g = {(0, 0, 0): STONE, (1, 0, 0): STONE, (3, 0, 0): STONE,
         (0, 1, 0): DUST, (1, 1, 0): DUST, (3, 1, 0): DUST}
    a = ("dust", (0, 1, 0), (1, 0, 0), "netA")
    b = ("dust", (1, 1, 0), (1, 0, 0), "netB")
    far = ("dust", (3, 1, 0), (1, 0, 0), "netB")
    assert M.interferes(a, b, g)[0]                        # P1: 1 apart merges
    assert not M.interferes(a, far, g)[0]                  # P2: 2+ apart is fine


# ------------------------------------------------- the two crossings, modelled
def test_classic_crossing_is_legal_in_the_model():
    """CROSSWIRE002: the Z-line's exit block is strong; the X-line's repeater
    stands on it and the X-line's dust stands on the exit block's NEIGHBOURS."""
    strong = ("strong_block", (2, 1, 2), (0, 0, -1), "Z")
    xrep = ("repeater", (2, 2, 2), (-1, 0, 0), "X")
    xdust_l = ("dust", (1, 2, 2), (-1, 0, 0), "X")
    xdust_r = ("dust", (3, 2, 2), (-1, 0, 0), "X")
    for other in (xrep, xdust_l, xdust_r):
        hit, why = M.interferes(strong, other)
        assert not hit, why
    #   the Z-line's own readout, one cell from the strong block, DOES light
    zdust = ("dust", (2, 1, 1), (0, 0, -1), "Z")
    assert M.interferes(strong, ("dust", (2, 1, 1), (0, 0, -1), "X"))[0]
    assert not M.interferes(strong, zdust)[0]              # same net


def test_instant_crossing_is_legal_in_the_model():
    """CROSSWIRE001 region A: the Z-line bumps to (3,3,3) on a solid support
    at (3,2,3) that sits directly over the X-line's dust at (3,1,3).  The
    support does double duty: it carries the bump AND cuts the two y-diagonals
    between the two nets."""
    g = {
        (3, 0, 3): STONE, (2, 0, 3): STONE, (4, 0, 3): STONE,
        (2, 1, 3): DUST, (3, 1, 3): DUST, (4, 1, 3): DUST,   # X-line
        (3, 2, 3): STONE,                                    # the bump support
        (3, 3, 3): DUST,                                     # the bump top
        (3, 1, 2): STONE, (3, 1, 4): STONE,
        (3, 2, 2): DUST, (3, 2, 4): DUST,                    # Z-line legs
    }
    #   the bump is legal and connected to its own legs
    assert M.can_occupy("dust", (3, 3, 3), {k: v for k, v in g.items()
                                            if k != (3, 3, 3)})[0]
    assert M.wire_connects(g, (3, 3, 3), (3, 2, 2))
    assert M.wire_connects(g, (3, 3, 3), (3, 2, 4))
    #   the two nets' y-diagonals are CUT by that same support cell
    assert not M.wire_connects(g, (3, 1, 3), (3, 2, 2))
    assert not M.wire_connects(g, (3, 1, 3), (3, 2, 4))
    #   remove the support and the crossing shorts -- the negative control
    leak = dict(g); leak.pop((3, 2, 3))
    assert M.wire_connects(leak, (3, 1, 3), (3, 2, 2))
    #   a TRANSPARENT support would carry the bump but not cut: also a short
    glassy = dict(g); glassy[(3, 2, 3)] = GLASS
    assert M.wire_connects(glassy, (3, 1, 3), (3, 2, 2))
    #   and the X-line dust under the support is untouched by it (W3)
    assert (3, 2, 3) not in M.dust_emission((3, 1, 3),
                                            pointing=((0, 0, 1), (0, 0, -1)))


def test_every_row_cites_a_probe():
    rows = M.mechanism_table()
    assert len(rows) == 11
    for m in rows:
        assert m.probe, m.name
        assert m.note, m.name


TESTS = [v for k, v in sorted(globals().items()) if k.startswith("test_")]

if __name__ == "__main__":
    bad = 0
    for t in TESTS:
        try:
            t()
            print("PASS %s" % t.__name__)
        except AssertionError as e:
            bad += 1
            print("FAIL %s: %s" % (t.__name__, e))
    print("test_transport: %d/%d" % (len(TESTS) - bad, len(TESTS)))
    raise SystemExit(1 if bad else 0)
