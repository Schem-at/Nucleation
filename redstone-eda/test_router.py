"""Router smoke test: two nets, plane 0 -> plane 1 (+12y), past obstacles.

Verifies in mc-tick that routed paths conduct, don't short each other, and
that climbs are non-inverting.
"""
import rs
import nets
import router

b = rs.Build("test_router")
labels = {}

# plane 0: two levers with short dust tails
SRC = {}
for i, z in enumerate((0, 4)):
    b.stone(0, 0, z); b.force(0, 1, z, rs.LEVER_OFF)
    b.stone(1, 0, z)
    b.put(1, 1, z, rs.DUST)
    labels[(1, 1, z)] = "net%d" % i
    SRC["net%d" % i] = (1, 1, z)

# an obstacle slab between the planes, with a 6-wide gap at x 4..9
for x in range(-2, 16):
    for z in range(-2, 8):
        if not (4 <= x <= 9):
            b.stone(x, 7, z)

# plane 1: two target rails (dust runs) at y=13, labelled like compiled rails
DST, PROBE = {}, {}
for i, z in enumerate((1, 5)):
    for x in range(8, 14):
        b.stone(x, 12, z)
        b.put(x, 13, z, rs.DUST)
        labels[(x, 13, z)] = "net%d" % i
    DST["net%d" % i] = (8, 13, z)
    PROBE["net%d" % i] = (13, 13, z)

# net2: a long flat run that must trigger block-sandwich stations (the
# max-pitch refresh: block / repeater / block, probe_station-verified)
b.stone(0, 0, 12); b.force(0, 1, 12, rs.LEVER_OFF)
b.stone(1, 0, 12)
b.put(1, 1, 12, rs.DUST)
labels[(1, 1, 12)] = "net2"
SRC["net2"] = (1, 1, 12)
for x in range(46, 51):
    b.stone(x, 0, 12)
    b.put(x, 1, 12, rs.DUST)
    labels[(x, 1, 12)] = "net2"
DST["net2"] = (46, 1, 12)
PROBE["net2"] = (50, 1, 12)

r = router.Router(b, labels)
for name in ("net0", "net1"):
    n = r.route(SRC[name], DST[name], name)
    print("routed %s in %d cells" % (name, n))
n = r.route(SRC["net2"], DST["net2"], "net2")
print("routed net2 in %d cells with %d stations" % (n, r.stations))
assert r.stations >= 1, "long straight run emitted no block-sandwich station"

shorts = nets.check(b.cells, labels)
print("net check: %d shorts" % len(shorts))
for s in shorts[:4]:
    print("   ", s)

sim = b.sim()
lv = rs.Levers(sim, [(0, 1, 0), (0, 1, 4), (0, 1, 12)])
ok = True
for bits in ((0, 0, 0), (1, 0, 1), (0, 1, 0), (1, 1, 1), (0, 0, 1)):
    lv.set(bits)
    got = tuple(int(sim.on(*PROBE["net%d" % i])) for i in range(3))
    match = got == bits
    ok = ok and match
    print("levers", bits, "-> probes", got, "OK" if match else "MISMATCH")
print("ROUTER TEST", "PASS" if (ok and not shorts) else "FAIL")
raise SystemExit(0 if (ok and not shorts) else 1)
