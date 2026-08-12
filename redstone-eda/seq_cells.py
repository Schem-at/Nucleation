"""Sequential cells: transparent D latch + master-slave edge-triggered DFF.

Mechanism (probed in seq_probe.py): a repeater whose side is entered by a
powered repeater/comparator is locked=true and freezes.  Latch = data
repeater + side locker.  DFF = two latches in series with opposite clock
phases:

    master lock <- CLK        (master transparent while CLK low)
    slave  lock <- NOT CLK    (slave transparent while CLK high)

Rising edge: master locks (capturing D) BEFORE the slave opens (the NOT
torch + lock repeater add 2+2 gt), so there is no transparency window on
the capture edge -- a clean RISING-edge DFF.

Initial state is carried by the schematic (baked, deployed InWorld); RST is
an option, not a correctness need.  Verification uses fixed-tick stepping
(sim.sim.run(n)) with a clocked protocol -- never run_until_quiescent.

DFF cell: 13x4x7 (x 0..12, z -1..5, y 0..3), pitch 7 in z.
Ports: D (0,1,0) west, Q (12,1,0) east (buffered; a designed y3 bridge
       carries Q OVER the clock column, capping the clock dust -- legal,
       the column is a straight flat run), clk_in (10,1,-1) north,
       clk_out (10,1,5) south (chain by abutment, one refresh repeater
       per cell).
Boundary rows z=-1 and z=5 hold NOTHING but the x=10 clock column.
"""
import nucleation as n
import rs
import seq_probe as sp
from cells import Fragment
from seq_probe import run_gt, locked, bake_states, reload_sim, ticks_to_quiescent

CLK_X = 10          # clock chain column (north-south, like the FA carry chain)
DFF_PITCH = 7


# ---------------------------------------------------------------- D latch

def build_dlatch():
    """Transparent D latch: data repeater + side locker.
    Ports: D (0,1,0), EN (2,1,3) [EN=1 -> HOLD, EN=0 -> transparent], Q (4,1,0).
    """
    b = rs.Build("dlatch")
    labels = {}
    for x in (0, 1):
        b.dust(x, 1, 0)
        labels[(x, 1, 0)] = "d"
    b.stone(2, 0, 0)
    b.put(2, 1, 0, rs.repeater("west"))
    for x in (3, 4):
        b.dust(x, 1, 0)
        labels[(x, 1, 0)] = "q"
    b.stone(2, 0, 1)
    b.put(2, 1, 1, rs.repeater("south"))
    b.dust(2, 1, 2)
    labels[(2, 1, 2)] = "en"
    frag = Fragment()
    frag.cells, frag.labels = dict(b.cells), dict(labels)
    frag.ports = {"d": (0, 1, 0), "en": (2, 1, 2), "q": (4, 1, 0),
                  "rep": (2, 1, 0)}
    return frag


def verify_dlatch():
    print("== D latch (fixed-tick protocol) ==")
    frag = build_dlatch()
    b = rs.Build("v_dlatch")
    labels = {}
    ports = frag.stamp(b, labels, 0, 0, 0)
    # levers: D west, EN south of the en dust
    b.stone(-2, 0, 0); b.force(-2, 1, 0, rs.LEVER_OFF); b.dust(-1, 1, 0)
    b.stone(2, 0, 4); b.force(2, 1, 4, rs.LEVER_OFF); b.dust(2, 1, 3)
    sim = b.sim()
    ok = True

    def chk(c, m):
        nonlocal ok
        print("   %-46s %s" % (m, "PASS" if c else "FAIL"))
        ok = ok and c

    D, EN, Q = (-2, 1, 0), (2, 1, 4), ports["q"]
    chk(not sim.on(*Q), "EN=0 D=0: Q=0")
    sim.use(*D); run_gt(sim, 6)
    chk(sim.on(*Q), "transparent: D=1 -> Q=1")
    sim.use(*EN); run_gt(sim, 6)
    chk(locked(sim, *ports["rep"]) is True, "EN=1 -> locked")
    sim.use(*D); run_gt(sim, 30)
    chk(sim.on(*Q), "hold: D=0, Q stays 1")
    sim.use(*EN); run_gt(sim, 6)
    chk(not sim.on(*Q), "EN=0 -> transparent again, Q=0")
    print("   dlatch: %s" % ("PASS" if ok else "FAIL"))
    return ok


# ---------------------------------------------------------------- MS DFF

def build_dff():
    b = rs.Build("dff")
    labels = {}

    def dust(x, y, z, lab, role="plain"):
        b.stone(x, y - 1, z, role)
        b.put(x, y, z, rs.DUST)
        labels[(x, y, z)] = lab

    # data row z0: D -> master -> slave -> buffer -> Q
    dust(0, 1, 0, "d", "lane"); dust(1, 1, 0, "d", "lane")
    b.stone(2, 0, 0, "gate"); b.put(2, 1, 0, rs.repeater("west"))   # master
    dust(3, 1, 0, "m", "lane")
    b.stone(4, 0, 0, "gate"); b.put(4, 1, 0, rs.repeater("west"))   # slave
    dust(5, 1, 0, "s", "lane")
    b.stone(6, 0, 0, "gate"); b.put(6, 1, 0, rs.repeater("west"))   # Q buffer
    dust(7, 1, 0, "q", "lane")
    # lock repeaters (z1): outputs point north into the data repeaters' sides
    b.stone(2, 0, 1, "gate"); b.put(2, 1, 1, rs.repeater("south"))  # master lock
    b.stone(4, 0, 1, "gate"); b.put(4, 1, 1, rs.repeater("south"))  # slave lock
    # z2: clk feed for master lock (x2), nclk feed for slave lock (x4),
    # x3 stays EMPTY (clk/nclk clearance)
    dust(2, 1, 2, "clk", "route")
    dust(4, 1, 2, "nclk", "inv")
    # NOT(CLK): straight dead-end dust north into a block, wall torch west
    dust(6, 1, 3, "clk", "route")
    b.put(6, 1, 2, rs.PALETTE["inv"])                          # torch base
    b.put(5, 1, 2, rs.wall_torch("west"))                      # -> nclk
    labels[(5, 1, 2)] = "nclk"
    # clk column x2 (row z4 -> master lock feed)
    dust(2, 1, 3, "clk", "route")
    # clk row z4: x2..x10
    for x in range(2, 11):
        dust(x, 1, 4, "clk", "route")
    # clock chain column x10: north port, refresh repeater, south port
    dust(CLK_X, 1, -1, "clk", "route"); dust(CLK_X, 1, 0, "clk", "route")
    dust(CLK_X, 1, 1, "clk", "route")
    b.stone(CLK_X, 0, 2, "route"); b.put(CLK_X, 1, 2, rs.repeater("north"))
    dust(CLK_X, 1, 3, "clk", "route")
    dust(CLK_X, 1, 5, "clk", "route")
    # Q bridge: carry Q OVER the clock column on a y3 flyover.  The cap at
    # (10,2,0) sits directly on the clock dust -- legal, that run is straight
    # and flat (no diagonals to cut).  Guard blocks keep the descent from
    # diagonally touching the clock dust.
    b.stone(8, 1, 0, "tap");  dust(8, 2, 0, "q")
    b.stone(9, 2, 0, "tap");  dust(9, 3, 0, "q")
    b.stone(CLK_X, 2, 0, "tap"); dust(CLK_X, 3, 0, "q")        # cap + fly
    b.stone(11, 1, 0, "tap"); dust(11, 2, 0, "q")
    dust(12, 1, 0, "q", "lane")                                # Q port

    frag = Fragment()
    frag.cells, frag.labels = dict(b.cells), dict(labels)
    frag.ports = {"d": (0, 1, 0), "q": (12, 1, 0),
                  "clk_in": (CLK_X, 1, -1), "clk_out": (CLK_X, 1, 5),
                  "master": (2, 1, 0), "slave": (4, 1, 0)}
    frag.meta = {}       # timing + paste_safe recorded by characterize()
    return frag


def dff_harness(frag, name="v_dff"):
    """DFF + D lever (west) + CLK lever (north of clk_in).  Returns
    (build, ports, D lever, CLK lever)."""
    b = rs.Build(name)
    labels = {}
    ports = frag.stamp(b, labels, 0, 0, 0)
    b.stone(-2, 0, 0); b.force(-2, 1, 0, rs.LEVER_OFF); b.dust(-1, 1, 0)
    x, y, z = ports["clk_in"]
    b.stone(x, 0, z - 1); b.put(x, y, z - 1, rs.DUST)
    b.stone(x, 0, z - 2); b.force(x, y, z - 2, rs.LEVER_OFF)
    import nets, drc
    shorts = nets.check(b.cells, labels)
    assert not shorts, "shorts: %s" % shorts[:4]
    # The latch stores state in a LOCKED repeater's side, not in a conduction
    # ring, so a diode ring here would be an accident, not the mechanism.
    rings = drc.repeater_cycles(b.cells)
    assert not rings, "diode ring (latches at 15): %s" % rings[:2]
    return b, ports, (-2, 1, 0), (x, y, z - 2)


PULSE = 12           # generous clock half-period used for functional tests
SETUP = 10           # generous D-to-edge lead used for functional tests


def pulse(sim, clk, width=PULSE):
    sim.use(*clk); run_gt(sim, width)
    sim.use(*clk); run_gt(sim, width)


def verify_dff():
    print("== MS DFF (fixed-tick clocked protocol) ==")
    frag = build_dff()
    b, ports, D, CLK = dff_harness(frag)
    sim = b.sim()          # placement settle of the fresh build only
    ok = True

    def chk(c, m):
        nonlocal ok
        print("   %-52s %s" % (m, "PASS" if c else "FAIL"))
        ok = ok and c

    Q = ports["q"]
    chk(not sim.on(*Q), "init: Q=0")
    chk(locked(sim, *ports["slave"]) is True, "CLK=0: slave locked (nclk=1)")
    chk(locked(sim, *ports["master"]) is False, "CLK=0: master transparent")

    # capture 1
    sim.use(*D); run_gt(sim, SETUP)
    chk(not sim.on(*Q), "D=1 before edge: Q still 0")
    sim.use(*CLK); run_gt(sim, PULSE)            # rising edge
    chk(sim.on(*Q), "rising edge captures: Q=1")
    chk(locked(sim, *ports["master"]) is True, "CLK=1: master locked")
    sim.use(*D); run_gt(sim, 30)
    chk(sim.on(*Q), "D dropped while CLK=1: Q holds 1")
    sim.use(*CLK); run_gt(sim, PULSE)            # falling edge
    chk(sim.on(*Q), "falling edge: Q still 1 (no capture)")
    run_gt(sim, 30)
    chk(sim.on(*Q), "CLK=0, D=0 for 30gt: Q holds 1")
    # capture 0
    pulse(sim, CLK)
    chk(not sim.on(*Q), "next rising edge captures D=0: Q=0")
    # random-ish sequence vs model
    import random
    random.seed(7)
    q_model, d_now = 0, 0
    good = True
    for _ in range(24):
        d_new = random.randint(0, 1)
        if d_new != d_now:
            sim.use(*D); d_now = d_new
        run_gt(sim, SETUP)
        if random.random() < 0.7:
            pulse(sim, CLK)
            q_model = d_now
        else:
            run_gt(sim, 2 * PULSE)
        good = good and int(sim.on(*Q)) == q_model
    chk(good, "24-step random D/CLK sequence matches model")
    print("   dff functional: %s" % ("PASS" if ok else "FAIL"))
    return ok


def verify_dff_baked():
    """Baked initial state: settle at a known Q, bake, reload InWorld,
    confirm 0-tick quiescence + Q, then clock the RELOADED sim."""
    print("== DFF baked initial state ==")
    frag = build_dff()
    ok = True
    for q0 in (0, 1):
        b, ports, D, CLK = dff_harness(frag, "v_dff_bake%d" % q0)
        sim = b.sim()
        if q0:
            sim.use(*D); run_gt(sim, SETUP)
            pulse(sim, CLK)
            sim.use(*D); run_gt(sim, 20)         # D back to 0, Q=1 held
        baked = bake_states(b, sim)
        off = b.bounds()[0]
        s2, raw = reload_sim(baked, off, n.TickSettleMode.InWorld)
        tq = ticks_to_quiescent(raw)
        good = tq == 0 and int(s2.on(*ports["q"])) == q0
        # the reloaded register must still clock correctly: drive D to 1-q0
        if bool(s2.powered(*D)) != bool(1 - q0):
            s2.use(*D)
        run_gt(s2, SETUP)
        pulse(s2, CLK)
        good = good and int(s2.on(*ports["q"])) == (1 - q0)
        print("   baked Q=%d: quiescent in %d gt, Q ok, clocks after reload: %s"
              % (q0, tq, "PASS" if good else "FAIL"))
        ok = ok and good
        # paste_safe probe: same baked schematic under Placement settle
        s3, raw3 = reload_sim(baked, off, n.TickSettleMode.Placement)
        tq3 = ticks_to_quiescent(raw3, cap=200)
        ps = tq3 >= 0 and int(s3.on(*ports["q"])) == q0
        print("   baked Q=%d under Placement: quiescent %d gt, state kept: %s"
              % (q0, tq3, ps))
        frag.meta["paste_safe_q%d" % q0] = bool(ps)
    return ok


# ------------------------------------------------------- characterization

def characterize(frag):
    """Empirical timing, all in GAME ticks (2 gt = 1 redstone tick).
    mc-tick is deterministic, so these are exact."""
    print("== DFF characterization (game ticks) ==")
    meta = frag.meta

    def fresh(q_want_zero=True):
        b, ports, D, CLK = dff_harness(frag, "c_dff")
        sim = b.sim()
        return sim, ports, D, CLK

    # setup: raise D, wait k gt, raise CLK; min k with capture
    setup = None
    for k in range(0, 13):
        sim, ports, D, CLK = fresh()
        sim.use(*D); run_gt(sim, k)
        sim.use(*CLK); run_gt(sim, 40)
        if sim.on(*ports["q"]):
            setup = k
            break
    meta["setup_gt"] = setup

    # hold: raise D and CLK together is illegal timing; instead raise D with
    # full setup, raise CLK, then DROP D k gt after the edge; min k where the
    # captured 1 survives.
    hold = None
    for k in range(0, 13):
        sim, ports, D, CLK = fresh()
        sim.use(*D); run_gt(sim, 12)
        sim.use(*CLK); run_gt(sim, k)
        sim.use(*D); run_gt(sim, 40)
        if sim.on(*ports["q"]):
            hold = k
            break
    meta["hold_gt"] = hold

    # min CLK pulse width: D=1, CLK high for w, then low; capture must survive
    width = None
    for w in range(1, 21):
        sim, ports, D, CLK = fresh()
        sim.use(*D); run_gt(sim, 12)
        sim.use(*CLK); run_gt(sim, w)
        sim.use(*CLK); run_gt(sim, 40)
        if sim.on(*ports["q"]):
            width = w
            break
    meta["min_pulse_gt"] = width

    # clk->Q: D=1 settled, raise CLK, step until Q rises
    sim, ports, D, CLK = fresh()
    sim.use(*D); run_gt(sim, 12)
    sim.use(*CLK)
    c2q = None
    for t in range(1, 41):
        run_gt(sim, 1)
        if sim.on(*ports["q"]):
            c2q = t
            break
    meta["clk_to_q_gt"] = c2q

    # min period: alternate D every cycle at period P (half-period P/2),
    # 10 cycles all captured correctly
    minp = None
    for p in range(4, 41, 2):
        sim, ports, D, CLK = fresh()
        good, d = True, 0
        for i in range(10):
            want = 1 - d
            sim.use(*D); d = want
            run_gt(sim, p // 2)
            sim.use(*CLK); run_gt(sim, p // 2)   # rising edge + high phase
            sim.use(*CLK)                        # falling edge; next D follows
            if int(sim.on(*ports["q"])) != want:
                good = False
                break
        if good:
            minp = p
            break
    meta["min_period_gt"] = minp

    for k in ("setup_gt", "hold_gt", "min_pulse_gt", "clk_to_q_gt",
              "min_period_gt"):
        print("   %-14s %s" % (k, meta[k]))
    return meta


if __name__ == "__main__":
    r = [verify_dlatch(), verify_dff(), verify_dff_baked()]
    meta = characterize(build_dff())
    print("seq_cells:", "ALL PASS" if all(r) else "FAILURES")
