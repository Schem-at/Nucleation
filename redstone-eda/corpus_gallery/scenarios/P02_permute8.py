"""Arbitrary lane PERMUTATION, expressed the only way the current API can
express one: eight 1-bit buses with crossed endpoints.

Two things had to be right before this solved, and both are findings:

* ROOM.  Each net that changes level needs the verified level-shift tile, and
  the router says exactly how much straight run it wants ("the verified
  level-shift tile needs 23 cells of straight run ... but the pair only spans
  15 in x").  A 15-block gap cannot hold an 8-level shift, so the endpoints are
  46 apart here.
* ORDER.  Handing the nets over in declaration order routes 7 of 8 and then
  fails on the last one because an earlier, easier net has already taken the
  cells it needed.  Handing them over hardest-first (largest level change
  first) routes all 8.  See P02b for the same problem in the other order --
  the solver has no rip-up-and-retry, so net order is the caller's problem.
"""

# a_i drives b_{P[i]} -- a shuffle with no fixed points and two long jumps.
P = [3, 1, 4, 7, 0, 6, 2, 5]
INV = [P.index(j) for j in range(len(P))]      # sink bit j reads source INV[j]
XOUT, ZSPREAD = 46, 2

# hardest first: the biggest level change gets the corridor it needs
ORDER = sorted(range(8), key=lambda i: -abs(2 * P[i] - 2 * i))

SCENARIO = {
    "id": "P02_permute8",
    "title": "Arbitrary 8-lane permutation (eight 1-bit buses, hardest first)",
    "question": ("Route a_i -> b_%s.  Can eight independent 1-bit buses "
                 "realize an arbitrary shuffle through one shared volume "
                 "without shorting?" % (P,)),
    "ports": (
        [{"name": "a%d" % i, "dir": "in", "form": "vertical",
          "anchor": [1, 2 + 2 * i, 4], "width": 1, "ty": "uint"}
         for i in range(8)]
        + [{"name": "b%d" % j, "dir": "out", "form": "vertical",
            "anchor": [XOUT, 2 + 2 * j, 4 + ZSPREAD * j], "width": 1,
            "ty": "uint"} for j in range(8)]),
    "buses": [
        {"name": "net_a%d" % i, "driver": "a%d" % i, "sinks": ["b%d" % P[i]]}
        for i in ORDER],
    "verify": {
        "words": [
            {"in": ["a%d" % i for i in range(8)],
             "out": ["b%d" % j for j in range(8)],
             "patterns": "walking+extremes",
             "map": {"kind": "permute", "perm": INV},
             "label": "shuffle a_i -> b_P[i], all 8 lanes isolated"},
        ],
    },
    "render": {"yaw": 145, "pitch": 26, "zoom": 1.6},
    "expect": "solved",
    "notes": ("Walking ones is the isolation proof: if two nets shorted, one "
              "lever would light two lamps and the read word would not match "
              "the permutation.  Net order: %s." % (ORDER,)),
}
