"""Probe the two named next suspects for rate-blindness, plus the hex comb.

`notes-vertical-transport.md` closes with an open question:

    "Are there other rate-dependent mechanisms we have never driven by rate?
     Repeater locking and comparator priming are both stateful and both
     currently verified only by settle.  Same rig would answer it."

This is that rig.  Both mechanisms are stateful, so both *could* hide the same
class of fault torch burnout hides: internal state that disagrees with the
inputs, a port that still reads plausibly, and a wrong answer on the next
operation.  The question is empirical, so it is measured, not argued.

Groups:
  L. REPEATER LOCKING (`seq_probe.build_probe` rig, and the D latch).  The
     latch our whole sequential library is built on.  Driven by rate, and
     checked for the one thing settling cannot reveal: a `locked` flag that
     disagrees with its locker after the traffic stops.
  D. THE DFF's ONE TORCH.  `seq_cells.build_dff` contains exactly one redstone
     torch -- the NOT(CLK) inverter feeding the slave lock -- so the clock path
     is torch-bearing and the published min period must be checked against the
     burnout floor, not just against latency.  Also: the existing min-period
     sweep is 10 cycles, which is shorter than a burnout window is wide at the
     periods it accepts, so a SUSTAINED run is a different experiment.
  C. COMPARATOR PRIMING.  A comparator stores an output strength and schedules
     its own tick, so it is as stateful as a repeater.  Driven by rate as a
     signal element, as a locking driver, and as an analog element.
  H. THE HEX ANALOG COMB (`notes-hex-transport.md`: pipeline depth 4 gt,
     minimum value separation 3 gt).  Both numbers come from ONE transition.
     Re-measured under sustained traffic, which is a different claim.

Run: ~/eda-venv/bin/python probe_lock_prime.py
"""
import ratedrive as rd
import rs
import seq_probe as sp
from seq_probe import run_gt, locked

VERDICTS = []


def V(ok, text, extra=""):
    VERDICTS.append((bool(ok), text, extra))


def transitions(sim, read, lever, period, flips):
    """Toggle `lever` every `period` gt, `flips` times, never settling.

    Returns transitions observed at `read` per input flip.  A healthy carrier
    scores 1.0.  This is `probe_torch_burnout.Traffic.drive`'s metric, lifted
    so it can be pointed at any mechanism.
    """
    seen, last = 0, read()
    for _ in range(flips):
        sim.use(*lever)
        for _ in range(period):
            sim.sim.run(1)
            now = read()
            if now != last:
                seen += 1
                last = now
    return seen / float(flips)


# ==================================================== L. repeater locking

def lock_rig():
    b, dlv, llv, rep, out = sp.build_probe("south")
    sim = b.sim()
    return sim, dlv, llv, rep, out


sim, DLV, LLV, REP, OUT = lock_rig()
V(locked(sim, *REP) is False and not sim.on(*OUT),
  "L0 rig sanity: the data repeater starts unlocked with output 0")

# --- L1  the locker driven by rate, data static ---------------------------
# A repeater has no toggle budget, so the lock flag should track at any rate
# above its own delay.  Measured, not assumed.
l1 = {}
for p in (1, 2, 3, 4, 6, 8):
    s, d, l, r, o = lock_rig()
    s.use(*d)
    run_gt(s, 10)                             # D=1, output tracking high
    l1[p] = transitions(s, lambda s=s, r=r: locked(s, *r) is True, l, p, 24)
V(all(v >= 0.9 for p, v in l1.items() if p >= 2),
  "L1 the LOCK FLAG tracks its locker 1:1 at every period >= 2 gt -- and 2 gt "
  "is the locking repeater's OWN delay, so this is the mechanism's latency "
  "floor and not a budget.  A repeater has no toggle budget to exhaust, so "
  "locking is not burnout-prone the way a torch is",
  "locked-flag transitions per lock flip, by period: %s"
  % {p: round(v, 2) for p, v in sorted(l1.items())})
V(l1[1] < 0.9,
  "L1 ...at 1 gt the locker is being asked to change faster than a delay-1 "
  "repeater can schedule, so half the lock changes are swallowed before they "
  "reach the flag.  Swallowed at the input, deterministically -- the failure "
  "is under-sampling, not damage",
  "period 1 tracks %.2f of lock flips" % l1[1])

# --- L2  the DANGEROUS test: does the flag desync from its locker? ---------
# The fault to look for is not a lost transition, it is a repeater left
# `locked=true` while its locker is unpowered (or the reverse).  That state
# survives settling and freezes the latch forever, which is exactly the shape
# of the torch fault.  Drive D and LOCK together at speed, then release the
# lock, settle, and demand that the latch resume tracking D.
l2 = {}
for p in (1, 2, 3, 4, 8, 12):
    s, d, l, r, o = lock_rig()
    dstate, lstate = False, False
    for i in range(24):                       # both inputs change every step
        s.use(*d); dstate = not dstate
        s.use(*l); lstate = not lstate
        s.sim.run(p)
    if lstate:                                # leave the lock RELEASED
        s.use(*l); lstate = False
    s.settle(400)
    flag = locked(s, *r)
    tracks = (s.on(*o) == dstate)
    # and it must keep tracking on a fresh change
    s.use(*d); dstate = not dstate
    s.settle(400)
    tracks = tracks and (s.on(*o) == dstate)
    l2[p] = (flag, tracks)
V(all(f is False for f, _t in l2.values()),
  "L2 after 24 simultaneous D+LOCK changes at periods down to 1 gt, releasing "
  "the lock always leaves `locked=false`: the flag is DERIVED from the "
  "locker's power every evaluation, so it cannot desync and there is no "
  "stuck-locked state to get stuck in",
  "(locked, tracks) after release, by period: %s" % dict(sorted(l2.items())))
V(all(t for _f, t in l2.values()),
  "L2 ...and the latch resumes tracking D correctly in every case, including "
  "on the next fresh input change.  REPEATER LOCKING IS NOT RATE-BLIND: it "
  "loses no state and needs no recovery, so the settle-based verdicts in "
  "`seq_probe.py` hold at any rate the latch is fast enough to follow")

# --- L3  ...but it does have a latency floor, and that floor is silent -----
# Not a burnout fault: the latch simply cannot capture a value it was not shown
# long enough.  Worth stating because it is the constraint that DOES bind.
l3 = {}
for p in (0, 1, 2, 3, 4, 6, 10):
    s, d, l, r, o = lock_rig()
    # show D=1 for p gt, then lock; the latch should hold 1
    s.use(*d)
    run_gt(s, p)
    s.use(*l)
    run_gt(s, 40)
    l3[p] = bool(s.on(*o))
V(all(l3.values()),
  "L3 the latch has NO setup requirement to violate: D and the lock toggled "
  "inside the SAME game tick still store the new value, at every setup from 0 "
  "to 10 gt.  The hypothesis under test -- that a word-parallel input change "
  "is a zero-setup event and might capture stale data -- is wrong for this "
  "latch, because the data repeater's own 2 gt of delay supplies the margin "
  "(and it is why `seq_cells.characterize` measured setup = 0 gt)",
  "captured correctly by D-setup: %s" % dict(sorted(l3.items())))
V(l3.get(0) is True,
  "L3 => repeater locking survives all three rate hazards we could name: no "
  "toggle budget (L1), no flag desync (L2), no setup violation (L3).  Its only "
  "limit is under-sampling at 1 gt, which is the delay-1 repeater's own floor. "
  "The latch is NOT where our rate blind spot lives")


# ============================================ D. the DFF's one torch

import seq_cells as sc

frag = sc.build_dff()
n_torch = sum(1 for blk in frag.cells.values() if "torch" in blk)
n_rep = sum(1 for blk in frag.cells.values() if "repeater" in blk)
V(n_torch == 1,
  "D1 the MS DFF contains exactly ONE redstone torch -- the NOT(CLK) wall "
  "torch that drives the slave lock -- and %d repeaters.  So the DFF's data "
  "path is torch-free but its CLOCK path is not, and the clock is the fastest-"
  "toggling net in any clocked design" % n_rep,
  "torches=%d repeaters=%d" % (n_torch, n_rep))

# The NOT(CLK) torch turns off once per clock cycle (it goes dark when CLK
# rises), so the budget is MAX_TURNOFFS per WINDOW -> one cycle per 7.5 gt.
BURN_FLOOR = 2 * rd.TORCH_MIN_HOLD_GT
V(rd.TORCH_BURNOUT_WINDOW / float(rd.TORCH_MAX_TURNOFFS) <= BURN_FLOOR,
  "D2 that torch turns off once per clock CYCLE, so the burnout floor on any "
  "clocked design's period is WINDOW/MAX_TURNOFFS = %.1f gt, i.e. >= %d gt.  "
  "This is a hard floor no amount of pipelining removes, and neither "
  "`seq_cells.py` nor `seq_counter.py` knew about it"
  % (rd.TORCH_BURNOUT_WINDOW / float(rd.TORCH_MAX_TURNOFFS), BURN_FLOOR))


def dff_run(period, cycles, seed=11):
    """Clock the DFF at `period` gt for `cycles`, alternating D, vs the model.

    Half the period is spent low and half high, matching
    `seq_cells.characterize`'s min-period sweep, so the numbers are comparable.
    """
    import random
    rnd = random.Random(seed)
    b, ports, D, CLK = sc.dff_harness(frag, "r_dff")
    s = b.sim()
    d, q, good, first_bad = False, False, 0, None
    for i in range(cycles):
        want = bool(rnd.getrandbits(1))
        if want != d:
            s.use(*D)
            d = want
        s.sim.run(max(1, period // 2))
        s.use(*CLK)                            # rising edge -> capture
        s.sim.run(max(1, period // 2))
        s.use(*CLK)                            # falling edge
        q = d
        if bool(s.on(*ports["q"])) == q:
            good += 1
        elif first_bad is None:
            first_bad = i
    # damage test: stop the clock, wait out a whole burnout window, then do one
    # clean slow cycle.  A wrong answer here is persistent damage.
    s.sim.run(rd.TORCH_BURNOUT_WINDOW + 20)
    s.settle(400)
    want = not d
    s.use(*D)
    run_gt(s, 12)
    sc.pulse(s, CLK, 12)
    clean_ok = bool(s.on(*ports["q"])) == want
    return good / float(cycles), first_bad, clean_ok


PUBLISHED_MIN_PERIOD = 20      # seq_README.md, from a 10-cycle sweep
d3 = {}
for p in (8, 12, 16, 20, 24, 32):
    d3[p] = dff_run(p, 40)
V(d3[PUBLISHED_MIN_PERIOD][0] >= 1.0,
  "D3 the published min period of %d gt SURVIVES a sustained run: 40 "
  "consecutive random-D cycles, %d gt of continuous clocking -- %.1fx the "
  "burnout window -- all captured correctly.  The 10-cycle sweep that "
  "produced the number was not measuring a transient"
  % (PUBLISHED_MIN_PERIOD, PUBLISHED_MIN_PERIOD * 40,
     PUBLISHED_MIN_PERIOD * 40 / float(rd.TORCH_BURNOUT_WINDOW)),
  "correct fraction by period: %s"
  % {p: round(v[0], 2) for p, v in sorted(d3.items())})
V(all(v[2] for v in d3.values()),
  "D3 ...and at every period tried, including ones far below the min period, a "
  "clean slow cycle after the traffic reads correctly.  The DFF does NOT "
  "accumulate persistent damage: below its min period it drops bits, it does "
  "not corrupt itself",
  "clean-cycle-after-traffic by period: %s"
  % {p: v[2] for p, v in sorted(d3.items())})
fails = sorted(p for p, v in d3.items() if v[0] < 1.0)
V(bool(fails) and max(fails) < PUBLISHED_MIN_PERIOD,
  "D3 below the min period the DFF captures the WRONG BIT and reports it as "
  "if it were right -- the output is a legal-looking 0 or 1, so a consumer "
  "cannot tell.  Failing periods: %s.  That is silent per-cycle data loss, "
  "which is why a clocked design needs a period budget and not just a "
  "functional test" % fails,
  "correct fraction by period: %s"
  % {p: round(v[0], 2) for p, v in sorted(d3.items())})
V(min(p for p, v in d3.items() if v[0] >= 1.0) > BURN_FLOOR,
  "D3 the DFF's binding constraint is LATENCY (%d gt), not burnout (%d gt): "
  "the loop clk->lock->Q is slower than the torch budget, so respecting the "
  "published min period keeps the clock torch inside its budget automatically. "
  "The burnout floor binds only for a design faster than %d gt"
  % (min(p for p, v in d3.items() if v[0] >= 1.0), BURN_FLOOR, BURN_FLOOR))


# ================================================ C. comparator priming

import cells

COMP = cells.COMP


def comp_rig(mode="subtract"):
    """back <- lever line; side idle; output dust east.  out = back - 0 = back."""
    b = rs.Build("prime_" + mode)
    for x in range(0, 4):
        b.stone(x, 0, 0)
        b.put(x, 1, 0, rs.LEVER_OFF if x == 0 else rs.DUST)
    b.stone(4, 0, 0)
    # facing names the side the INPUT comes from (same convention as
    # rs.repeater): the back line is west of the device, so facing=west.
    b.put(4, 1, 0, COMP % ("west", mode))
    for x in (5, 6):
        b.dust(x, 1, 0)
    return b.sim(), (0, 1, 0), (6, 1, 0)


c1 = {}
for mode in ("subtract", "compare"):
    for p in (1, 2, 3, 4, 6, 8):
        s, lc, out = comp_rig(mode)
        c1[(mode, p)] = transitions(s, lambda s=s, o=out: s.power(*o) > 0,
                                    lc, p, 24)
V(all(v >= 0.9 for k, v in c1.items() if k[1] >= 2),
  "C1 a comparator tracks its back input 1:1 at every period >= 2 gt in BOTH "
  "modes, for 24 back-to-back changes -- like a repeater and unlike a torch, "
  "it has no toggle budget, so COMPARATOR PRIMING IS NOT BURNOUT-PRONE",
  "transitions per flip: %s" % {"%s/%d" % k: round(v, 2)
                                for k, v in sorted(c1.items())})
V(all(v < 0.9 for k, v in c1.items() if k[1] <= 1),
  "C1 its floor is 2 gt -- its own scheduled-tick delay -- and only a 1-gt "
  "period is swallowed.  Note this is NOT the 3-gt figure "
  "`notes-hex-transport.md` publishes: 3 gt is a property of the 15-repeater "
  "COMB, not of the comparator, so the two numbers are about different things "
  "and neither generalises to the other")

# --- C2  the comparator AS A LOCKER, driven at rate ------------------------
c2 = {}
for p in (1, 2, 4, 8):
    b, d, l, r, o = sp.build_probe("south")
    b.force(2, 1, 1, COMP % ("south", "compare"))
    s = b.sim()
    dstate, lstate = False, False
    for _ in range(24):
        s.use(*d); dstate = not dstate
        s.use(*l); lstate = not lstate
        s.sim.run(p)
    if lstate:
        s.use(*l)
    s.settle(400)
    c2[p] = (locked(s, *r), s.on(*o) == dstate)
V(all(f is False and t for f, t in c2.values()),
  "C2 a comparator used as a LOCKING driver behaves exactly as the repeater "
  "locker does under the same traffic: no stuck lock, correct resumption at "
  "every period down to 1 gt.  The `seq_probe.probe_comparator_locker` verdict "
  "holds at rate", "(locked, tracks) by period: %s" % dict(sorted(c2.items())))

# --- C3  analog value integrity: is a stale strength ever left behind? -----
# A comparator stores a strength, so the worry is a value that survives the
# input that produced it.  Sweep an analog value at speed, then settle and
# demand the quasi-static answer.
def analog_rig():
    """Two lever injectors onto one dust run feeding a subtract comparator, so
    the back strength really changes value (not just on/off)."""
    b = rs.Build("prime_analog")
    for x in range(0, 12):
        b.stone(x, 0, 0)
        b.put(x, 1, 0, rs.DUST)
    b.stone(0, 0, 1); b.force(0, 1, 1, rs.LEVER_OFF)      # strong, near
    b.stone(6, 0, 1); b.force(6, 1, 1, rs.LEVER_OFF)      # weaker, far
    b.stone(12, 0, 0); b.put(12, 1, 0, COMP % ("west", "subtract"))
    b.stone(13, 0, 0); b.put(13, 1, 0, rs.DUST)
    return b.sim(), [(0, 1, 1), (6, 1, 1)], (13, 1, 0)


s, LVS, AOUT = analog_rig()
drv = rd.RateDriver(s, LVS)
truth = {}
for bits in ((0, 0), (1, 0), (0, 1), (1, 1)):
    drv.apply_settled(list(bits))
    truth[bits] = s.power(*AOUT)
c3 = {}
for p in (1, 2, 4, 8):
    s, LVS, AOUT = analog_rig()
    drv = rd.RateDriver(s, LVS)
    for i in range(24):                        # thrash both injectors
        drv.apply([i & 1, (i >> 1) & 1], p)
    ok = True
    for bits in ((1, 1), (0, 1), (1, 0), (0, 0)):
        drv.apply_settled(list(bits))
        ok = ok and s.power(*AOUT) == truth[bits]
    c3[p] = ok
V(all(c3.values()),
  "C3 after 24 analog value changes at periods down to 1 gt, every one of the "
  "four settled strengths matches the quasi-static truth table.  No stale "
  "'primed' strength survives the input that produced it, so comparator state "
  "needs no recovery protocol",
  "settled-truth match after traffic, by period: %s (truth %s)"
  % (dict(sorted(c3.items())), truth))


# ====================================== H. the hex comb under sustained traffic

import probe_hex_transmit as ph

# H1  reproduce the single-burst claim, so the sustained result is comparable.
h1 = {}
for w in (1, 2, 3, 4, 6):
    r = ph.Rig(values=(15,))
    r.on(1)
    r.s.use(*r.levers[0])                      # 15 -> 0
    r.s.sim.run(w)
    r.s.use(*r.levers[0])                      # 0 -> 15
    r.s.settle(200)
    seen = 0
    # count whether the dip ever reached the output at all
    rr = ph.Rig(values=(15,))
    rr.on(1)
    rr.s.use(*rr.levers[0])
    dip = 0
    for _ in range(w):
        rr.s.sim.run(1)
    rr.s.use(*rr.levers[0])
    for _ in range(26):
        rr.s.sim.run(1)
        if rr.s.power(*rr.out) == 0:
            dip += 1
    h1[w] = dip
V(h1[1] == 0 and h1[2] == 0 and h1[3] > 0,
  "H1 reproduced from `notes-hex-transport.md` H5.8/H5.9: a 1-gt or 2-gt gap "
  "is swallowed, a 3-gt gap gets through.  This is the single-burst "
  "measurement the published 'minimum value separation 3 gt' rests on",
  "gt of 0 seen at the OUTPUT by gap width: %s" % dict(sorted(h1.items())))

# H2  the same separation, SUSTAINED.  This is the new claim.
h2 = {}
for p in (1, 2, 3, 4, 6, 8):
    r = ph.Rig(values=(15,))
    r.on(1)
    r.s.settle(200)
    h2[p] = transitions(r.s, lambda r=r: r.s.power(*r.out) > 0,
                        r.levers[0], p, 32)
V(all(h2[p] >= 0.9 for p in h2 if p >= 3),
  "H2 the 3-gt separation HOLDS UNDER SUSTAINED TRAFFIC, not merely for one "
  "burst: 32 back-to-back injector changes at 3 gt each track 1:1 at the "
  "output.  The comb is 15 repeaters and 2 comparators and contains no torch, "
  "so it has no toggle budget to exhaust and its published pipeline numbers "
  "are rate-safe as stated",
  "output transitions per input change, by period: %s"
  % {p: round(v, 2) for p, v in sorted(h2.items())})
V(all(h2[p] < 0.9 for p in h2 if p <= 2),
  "H2 ...and sub-3-gt traffic fails the same way sustained as it does once: "
  "changes are swallowed at the first delay-1 repeater.  Deterministic input "
  "filtering, not device damage")

# H3  and no stale value is left behind after the fastest traffic.
r = ph.Rig(values=(15, 6))
r.on(1, 1)
base15 = r.s.power(*r.out)
r.on(0, 1)
base6 = r.s.power(*r.out)
r.on(1, 1)
for _ in range(40):                            # thrash the 15-injector at 1 gt
    r.s.use(*r.levers[0])
    r.s.sim.run(1)
r.s.sim.run(rd.TORCH_BURNOUT_WINDOW + 20)
r.s.settle(400)
# `Rig.on` drives through rs.Levers, whose cached lever states are now stale
# after 40 raw use_block calls -- resynchronise before asking for a value.
r.L.state = [bool(r.s.powered(*p)) for p in r.levers]
r.on(1, 1)
now15 = r.s.power(*r.out)
r.on(0, 1)
now6 = r.s.power(*r.out)
V(now6 == base6 and now15 == base15 and base15 == 15,
  "H3 after 40 injector changes at ONE game tick each -- far inside the "
  "swallowing regime -- the comb settles to the correct analog value again "
  "(%d for the 6-injector alone).  Over-driving a torch-free carrier loses "
  "values in flight but leaves no residue" % base6,
  "before %d/%d, after %d/%d" % (base15, base6, now15, now6))


bad = 0
for ok, text, extra in VERDICTS:
    print("%s %s%s" % ("PASS" if ok else "FAIL", text,
                       ("   [%s]" % extra) if extra else ""))
    bad += 0 if ok else 1
print("probe_lock_prime: %d/%d" % (len(VERDICTS) - bad, len(VERDICTS)))
raise SystemExit(1 if bad else 0)
