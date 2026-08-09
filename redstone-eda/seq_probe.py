"""Micro-probe: the repeater-lock mechanism in mc-tick.

A repeater whose SIDE is driven by a powered repeater/comparator shows
locked=true and freezes its output.  This probes, with fixed-tick stepping:

  1. lock registration from BOTH side orientations (locker to the south
     facing north-out, and locker to the north facing south-out);
  2. retention: lock-while-high holds Q=1 against D falling, lock-while-low
     holds Q=0 against D rising; release resumes tracking;
  3. comparator as the locking driver;
  4. baked-state survival: a locked repeater frozen in a state contradicting
     its input, saved to .schem and reloaded, in both InWorld and Placement
     settle modes ("paste_safe" probe).

Shared helpers (bake_states, reload_sim, run_gt) live here and are imported
by seq_cells / seq_register4 / seq_counter.
"""
import os
import re

import nucleation as n
import rs
import cells  # noqa: F401  (side effect: interns comparator EXTRA_STATES)

_LOCKED = re.compile(r"locked=(true|false)")

TMP = os.environ.get("TMPDIR", "/tmp")


def locked(sim, x, y, z):
    m = _LOCKED.search(sim.block(x, y, z))
    return None if m is None else m.group(1) == "true"


def run_gt(sim, ticks):
    """Fixed-tick stepping -- the clocked-verification primitive."""
    sim.sim.run(ticks)


def bake_states(build, sim):
    """Read every authored cell's SETTLED state back out of the sim into a
    fresh schematic: the baked-initial-state artifact (FPGA-bitstream style).
    """
    s = n.Schematic.create("baked")
    for (x, y, z) in build.cells:
        s.set_block_from_string(x, y, z, sim.block(x, y, z))
    return s


def reload_sim(schem, offset, mode, extra=None, path=None):
    """Save -> reopen (tight bounds) -> TickSimulation, WITHOUT settling.

    Returns (rs.Sim addressable in original build coords, raw TickSimulation).
    Caller decides how many ticks to run -- that is the whole point.
    """
    path = path or os.path.join(TMP, "_seq_reload.schem")
    schem.save_to_file(path)
    tight = n.Schematic.open(path)
    sim = n.TickSimulation.from_schematic(tight, mode, 0, 0, 0, rs.EXTRA_STATES)
    return rs.Sim(sim, offset), sim


def ticks_to_quiescent(raw, cap=100):
    """Step one game tick at a time until is_quiescent; -1 if cap exceeded."""
    for t in range(cap + 1):
        if raw.is_quiescent():
            return t
        raw.step()
    return -1


def build_probe(lock_side):
    """Data repeater at (2,1,0) facing=west (conducts +X), output dust x3..x4.
    Locker: repeater one step to lock_side ('south' -> locker at z=1 facing
    south = outputs north into the data rep's side; 'north' -> locker at z=-1
    facing north = outputs south).  D lever west, LOCK lever behind the locker.
    """
    b = rs.Build("probe_lock_" + lock_side)
    # D path (z0)
    b.stone(0, 0, 0)
    b.force(0, 1, 0, rs.LEVER_OFF)
    b.dust(1, 1, 0)
    b.stone(2, 0, 0)
    b.put(2, 1, 0, rs.repeater("west"))
    b.dust(3, 1, 0)
    b.dust(4, 1, 0)
    # lock path
    zs = 1 if lock_side == "south" else -1
    b.stone(2, 0, zs)
    b.put(2, 1, zs, rs.repeater("south" if zs == 1 else "north"))
    b.dust(2, 1, 2 * zs)
    b.stone(2, 0, 3 * zs)
    b.force(2, 1, 3 * zs, rs.LEVER_OFF)
    d_lever, lock_lever, rep, out = (0, 1, 0), (2, 1, 3 * zs), (2, 1, 0), (4, 1, 0)
    return b, d_lever, lock_lever, rep, out


def probe_orientation(lock_side):
    print("== locker on the %s side ==" % lock_side)
    b, dlv, llv, rep, out = build_probe(lock_side)
    sim = b.sim()
    ok = True

    def chk(cond, msg):
        nonlocal ok
        print("   %-52s %s" % (msg, "PASS" if cond else "FAIL"))
        ok = ok and cond

    # baseline tracking
    chk(not sim.on(*out), "unlocked, D=0 -> out 0")
    sim.use(*dlv); run_gt(sim, 10)
    chk(sim.on(*out), "unlocked, D=1 -> out 1")
    chk(locked(sim, *rep) is False, "locked=false while free")

    # lock while HIGH, then drop D
    sim.use(*llv); run_gt(sim, 10)
    chk(locked(sim, *rep) is True, "locker powered -> locked=true")
    sim.use(*dlv); run_gt(sim, 40)
    chk(sim.on(*out), "D dropped 40gt ago, out FROZEN high")
    chk(sim.powered(*rep) is True, "rep still powered=true (stored 1)")
    # release
    sim.use(*llv); run_gt(sim, 10)
    chk(locked(sim, *rep) is False, "release -> locked=false")
    chk(not sim.on(*out), "released -> resumes tracking (out 0)")

    # lock while LOW, then raise D
    sim.use(*llv); run_gt(sim, 10)
    chk(locked(sim, *rep) is True, "re-locked at D=0")
    sim.use(*dlv); run_gt(sim, 40)
    chk(not sim.on(*out), "D raised 40gt ago, out FROZEN low")
    sim.use(*llv); run_gt(sim, 10)
    chk(sim.on(*out), "release -> catches up (out 1)")
    print("   orientation %s: %s" % (lock_side, "PASS" if ok else "FAIL"))
    return ok


def probe_comparator_locker():
    print("== comparator as locking driver ==")
    b, dlv, llv, rep, out = build_probe("south")
    # swap the locking repeater for a comparator (same orientation)
    b.force(2, 1, 1, cells.COMP % ("south", "compare"))
    sim = b.sim()
    sim.use(*dlv)
    run_gt(sim, 10)
    ok = sim.on(*out)
    sim.use(*llv); run_gt(sim, 10)
    ok = ok and locked(sim, *rep) is True
    sim.use(*dlv); run_gt(sim, 40)
    ok = ok and sim.on(*out)
    print("   comparator locks + freezes: %s" % ("PASS" if ok else "FAIL"))
    return ok


def probe_baked_survival():
    """Freeze a contradictory state (rep stores 1, D lever now OFF), bake it,
    reload under InWorld and Placement, and see what survives."""
    print("== baked-state survival (save -> reload) ==")
    b, dlv, llv, rep, out = build_probe("south")
    sim = b.sim()
    sim.use(*dlv); run_gt(sim, 10)       # D=1 propagates
    sim.use(*llv); run_gt(sim, 10)       # lock while high
    sim.use(*dlv); run_gt(sim, 40)       # drop D; output stays frozen high
    assert sim.on(*out) and locked(sim, *rep)
    baked = bake_states(b, sim)
    off = b.bounds()[0]
    ok = True

    for mode_name in ("InWorld", "Placement"):
        mode = getattr(n.TickSettleMode, mode_name)
        s2, raw = reload_sim(baked, off, mode,
                             path=os.path.join(TMP, "_probe_baked.schem"))
        tq = ticks_to_quiescent(raw, cap=200)
        lk = locked(s2, *rep)
        pw = s2.powered(*rep)
        o = s2.on(*out)
        lever = s2.powered(*dlv)
        print("   %-10s quiescent in %3d gt | locked=%s powered=%s out=%s "
              "D-lever=%s" % (mode_name, tq, lk, pw, o, lever))
        if mode_name == "InWorld":
            good = tq == 0 and lk is True and pw is True and o
            print("   InWorld baked lock survives: %s" % ("PASS" if good else "FAIL"))
            ok = ok and good
        else:
            print("   Placement paste_safe(frozen-high vs D=0): %s"
                  % ("YES" if (lk is True and o) else "NO (recorded)"))
    return ok


if __name__ == "__main__":
    r = [probe_orientation("south"), probe_orientation("north"),
         probe_comparator_locker(), probe_baked_survival()]
    print("seq_probe:", "ALL PASS" if all(r) else "FAILURES (see above)")
