"""At-speed drivers -- the missing half of every verification in this directory.

`rs.Levers.set()` flips ONE lever and then runs `run_until_quiescent` before
doing anything else.  That protocol answers exactly one question:

    given infinite time between input changes, is the combinational function
    correct?

It cannot answer the question a clocked datapath actually asks:

    given a new input word every N game ticks, is the output correct?

The two differ for three separate reasons, and this module separates them so a
failure can be attributed instead of merely observed:

  SIMULTANEITY.  A word bus changes all its bits in one tick.  Flipping bits
    one at a time and settling between them hides every transient that a
    parallel change injects (a ripple chain can latch onto one).
    -> `Protocol.QW` (word-parallel, still settled) isolates this.

  LATENCY.  A circuit with an arrival time of L game ticks cannot be correct
    when sampled before L.  This is a benign, predictable failure: the answer
    is late, not wrong, and it is repaired by waiting.
    -> compare the failing hold against `measure_latency()`.

  BURNOUT.  A redstone torch that turns off more than
    `MAX_RECENT_TOGGLES = 8` times inside `TORCH_BURNOUT_WINDOW = 60` game
    ticks goes stuck-until-update (`probe_torch_burnout.py`).  This is a
    malign failure: it survives settling, it corrupts state that the output
    port does not expose, and it makes the NEXT input change read wrong.
    -> `burst()`'s canary phase catches it, and nothing else in this
       directory can.

Every torch is subject to burnout and every NOT gate is a torch, so the PLA
architecture (`build_ppa.py`: tap torch + gate torch per column) is torches all
the way down.  See `notes-rate-limits.md`.

The metric that matters is NOT "does it settle correctly".  It is:

    at_speed  -- correct when sampled at the end of each hold, mid-traffic;
    canary    -- correct on FRESH, FULLY SETTLED vectors applied *after* the
                 traffic stops.  A circuit that fails the canary has been
                 damaged by the traffic: it is silently wrong, and no
                 settle-based probe can see it.
"""

# mc-tick's constants (crates/mc-tick .. components.rs), mirrored in vforms.
TORCH_BURNOUT_WINDOW = 60
TORCH_MAX_TURNOFFS = 8
# A torch toggles twice per full cycle and only the off-going half is charged,
# so the budget is one input change per WINDOW / (2 * MAX) = 3.75 gt -> 4 gt.
TORCH_MIN_HOLD_GT = 4


class RateDriver:
    """Drives a lever word with an explicit hold time instead of a settle.

    Deliberately API-compatible with `rs.Levers` for `set()`, so an existing
    verifier can be re-pointed at it, but the interesting entry points are
    `apply()` (word-parallel, fixed hold, NO settle) and `apply_settled()`.
    """

    def __init__(self, sim, positions):
        self.sim = sim
        self.positions = list(positions)
        self.state = [bool(sim.powered(*p)) for p in self.positions]
        self.uses = 0

    # ---------------------------------------------------------------- protocols

    def _write(self, bits):
        """Toggle every lever that disagrees with `bits`, all inside one tick."""
        for i, want in enumerate(bits):
            if self.state[i] != bool(want):
                self.sim.use(*self.positions[i])
                self.state[i] = bool(want)
                self.uses += 1

    def apply(self, bits, hold):
        """AT SPEED: change the whole word in one tick, then run `hold` gt."""
        self._write(bits)
        if hold:
            self.sim.sim.run(hold)

    def apply_settled(self, bits, budget=4000):
        """Word-parallel but quiescent (protocol QW): isolates simultaneity."""
        self._write(bits)
        return self.sim.settle(budget)

    def set(self, bits, settle=4000):
        """Protocol Q1 -- byte-for-byte what `rs.Levers.set` does, so an
        existing verifier's baseline can be reproduced through this object."""
        ok = True
        for i, want in enumerate(bits):
            if self.state[i] != bool(want):
                self.sim.use(*self.positions[i])
                self.state[i] = bool(want)
                self.uses += 1
                ok = self.sim.settle(settle) and ok
        return self.sim.settle(settle) and ok


def measure_latency(drv, vectors, budget=4000):
    """Worst-case quasi-static settle time, in game ticks, over `vectors`.

    This is the floor below which an at-speed failure is merely LATE.  Measured
    with the word-parallel protocol, because that is what the rate harness
    drives.
    """
    worst, worst_v = 0, None
    for v in vectors:
        t0 = drv.sim.sim.tick_count()
        drv.apply_settled(v, budget)
        dt = drv.sim.sim.tick_count() - t0
        if dt > worst:
            worst, worst_v = dt, v
    return worst, worst_v


def burst(drv, vectors, hold, read, expect, canaries=None, budget=4000):
    """Drive `vectors` back-to-back at `hold` gt each; then check for damage.

    Phases:
      1. TRAFFIC -- each vector is applied word-parallel and sampled at the end
         of its hold.  No settle anywhere.  Counts `at_speed`.
      2. DRAIN -- settle once, re-read the final vector.  If this is correct but
         phase 1 was not, the circuit is merely SLOW at this rate.
      3. CANARY -- the damage test, and the reason this harness exists.  Each
         canary is applied with a FULL settle, and the canaries are separated by
         more than a whole burnout window so that *the canary phase cannot burn
         anything out by itself*.  Getting this wrong is easy and it poisons the
         result: four settled flips of a torch inverter take about ten game
         ticks in total, which is over the budget on its own, so a naive canary
         phase reports damage at every rate.  A wrong canary here means the
         TRAFFIC left persistent damage -- the circuit is SILENTLY WRONG at this
         rate, exactly as `probe_torch_burnout` B3 describes (a stuck torch, a
         port that still reads plausibly, and a wrong answer on the next input
         change).

    `read()` -> observed value, `expect(v)` -> wanted value.
    """
    canaries = list(canaries if canaries is not None else vectors[:4])
    at_ok, at_bad = 0, []
    for v in vectors:
        drv.apply(v, hold)
        got, want = read(), expect(v)
        if got == want:
            at_ok += 1
        elif len(at_bad) < 4:
            at_bad.append((v, got, want))

    # DRAIN.  `settle` reports whether it actually reached quiescence inside the
    # budget; that distinction matters, because a wrong output with updates
    # still pending is only late, while a wrong output in a QUIESCENT world is
    # stuck -- the circuit is holding a value that contradicts its own inputs
    # and nothing is scheduled to fix it.  That is the burnout signature.
    drain_quiet = bool(drv.sim.settle(budget)) and drv.sim.sim.is_quiescent()
    last = vectors[-1]
    drain_ok = read() == expect(last)
    stuck_wrong = drain_quiet and not drain_ok

    # RESUME -- B3's test, and the one a user actually experiences: the traffic
    # stops and you immediately use the circuit again.  Exactly one input change,
    # fully settled, still inside the burnout window the traffic exhausted.  One
    # change cannot exhaust a budget by itself, so a wrong answer here is lost
    # data caused by the traffic.
    resume_v = next((v for v in vectors if expect(v) != expect(last)), None)
    if resume_v is None:
        resume_ok, resume_detail = True, None
    else:
        drv.apply_settled(resume_v, budget)
        got, want = read(), expect(resume_v)
        resume_ok, resume_detail = (got == want), (resume_v, got, want)

    # Refresh every torch's toggle budget before the first canary, so that a
    # failing canary can only be persistent damage and never the canary's own
    # doing.  Burnout is stuck-until-update, so waiting cannot mask real
    # damage -- B3 measured 3x the window changing nothing.
    cool = TORCH_BURNOUT_WINDOW + 10
    drv.sim.sim.run(cool)
    can_ok, can_bad, first_ok = 0, [], None
    for v in canaries:
        drv.apply_settled(v, budget)
        got, want = read(), expect(v)
        if first_ok is None:
            first_ok = (got == want)
        if got == want:
            can_ok += 1
        elif len(can_bad) < 4:
            can_bad.append((v, got, want))
        drv.sim.sim.run(cool)          # keep the canaries independent

    return {
        "hold": hold,
        "n": len(vectors),
        "at_speed": at_ok / float(len(vectors)),
        "at_speed_bad": at_bad,
        "drain_ok": drain_ok,
        "drain_quiet": drain_quiet,
        "stuck_wrong": stuck_wrong,
        "resume_ok": bool(resume_ok),
        "resume_detail": resume_detail,
        "canary": can_ok / float(len(canaries)),
        "canary_bad": can_bad,
        "first_canary_ok": bool(first_ok),
        "damaged": can_ok < len(canaries),
    }


def classify(rows, latency):
    """Turn a hold-sweep into a verdict.

    Returns (min_hold_correct_at_speed, min_hold_undamaged, mode) where `mode`
    is the failure the circuit exhibits just below the safe hold:

      "silently wrong" -- a hold exists at which the canary fails.  The worst
                          kind: settling does not repair it.
      "slow only"      -- at-speed samples miss but every canary passes and the
                          drain is correct: the answer is late, never wrong.
    """
    rows = sorted(rows, key=lambda r: r["hold"])

    def _dirty(r):
        return r["damaged"] or not r["resume_ok"] or r["stuck_wrong"]

    safe_speed = next((r["hold"] for r in rows
                       if r["at_speed"] >= 1.0
                       and all(q["at_speed"] >= 1.0 for q in rows
                               if q["hold"] > r["hold"])), None)
    safe_clean = next((r["hold"] for r in rows
                       if not _dirty(r)
                       and all(not _dirty(q) for q in rows
                               if q["hold"] > r["hold"])), None)
    if any(r["damaged"] for r in rows):
        mode = "silently wrong (persistent)"
    elif any(not r["resume_ok"] for r in rows):
        mode = "silently wrong (next op)"
    elif any(r["stuck_wrong"] for r in rows):
        mode = "silently wrong (stuck)"
    else:
        mode = "slow only"
    return safe_speed, safe_clean, mode, latency


HOLDS = (2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192)


def sweep(make_drv, vectors, read_of, expect, holds=HOLDS, refine=True):
    """Run `burst` at each hold and (optionally) bisect for the exact boundary.

    `make_drv()` must return a FRESH (sim, drv) pair, and `read_of(sim)` the
    observed output value for that sim.  Freshness is not optional: burnout is
    stuck-until-update, so a rig damaged at one hold would carry that damage
    into the next measurement and the sweep would report the first rate it
    happened to be tested at rather than the rate under test.

    Returns (rows, safe_speed, safe_clean, mode).
    """
    def _run(h):
        sim, drv = make_drv()
        return burst(drv, vectors, h, lambda s=sim: read_of(s), expect)

    rows = [_run(h) for h in holds]

    # Extend upward if nothing passed.  A fixed ladder silently reports "no safe
    # rate exists" for any circuit slower than its top rung -- mult_4x4 settles
    # in 264 gt, past the end of the default ladder -- and that would look like
    # a finding when it is only a badly chosen sweep.
    h = max(holds)
    while not any(r["at_speed"] >= 1.0 for r in rows) and h < 4096:
        h *= 2
        rows.append(_run(h))

    if refine:
        ok = [r["hold"] for r in rows if r["at_speed"] >= 1.0]
        if ok:
            lo = max([r["hold"] for r in rows if r["at_speed"] < 1.0
                      and r["hold"] < min(ok)] or [0])
            hi = min(ok)
            while hi - lo > 1:
                mid = (lo + hi) // 2
                r = _run(mid)
                rows.append(r)
                if r["at_speed"] >= 1.0:
                    hi = mid
                else:
                    lo = mid
    return (rows,) + classify(rows, None)[:3]


# ------------------------------------------------------------ vector sequences

def alternating(n_bits, pairs=16):
    """Maximum-activity sequence: every input bit flips on every vector.

    This is the sequence that stresses the toggle budget hardest, and the one a
    real bus produces when it carries a value and then its complement.
    """
    a = [i % 2 for i in range(n_bits)]
    bar = [1 - v for v in a]
    out = []
    for _ in range(pairs):
        out.append(list(a))
        out.append(list(bar))
    return out


def walk(n_bits, count=32, seed=7):
    """Pseudo-random word sequence -- average, not worst-case, activity."""
    import random
    rnd = random.Random(seed)
    return [[rnd.getrandbits(1) for _ in range(n_bits)] for _ in range(count)]


def one_hot_sweep(n_bits, reps=2):
    """Single-bit changes: the minimum-activity sequence, for contrast.  A
    circuit that fails even here is failing on latency, not on burnout."""
    out, cur = [], [0] * n_bits
    for _ in range(reps):
        for i in range(n_bits):
            cur = list(cur)
            cur[i] ^= 1
            out.append(list(cur))
    return out
