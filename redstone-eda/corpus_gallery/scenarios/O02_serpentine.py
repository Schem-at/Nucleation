"""CONGESTION: two staggered walls that leave no monotone path.

Wall A blocks z<=10 at x=6, so the bundle must be at z>=11 when it passes x=6.
Wall B blocks z>=8 at x=12, so it must be back at z<=7 by x=12.  There is no
route that only ever turns one way -- the bundle has to serpentine, in
lockstep, eight bits wide.
"""

N = 8

SCENARIO = {
    "id": "O02_serpentine",
    "title": "Congestion: staggered walls force a serpentine",
    "question": ("Two walls with disjoint gaps.  Does the router find the "
                 "S-shaped corridor, and what does the detour cost in delay "
                 "and skew?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "vertical",
         "anchor": [1, 2, 4], "width": N, "ty": "uint"},
        {"name": "a_out", "dir": "out", "form": "vertical",
         "anchor": [18, 2, 4], "width": N, "ty": "uint"},
    ],
    "obstacles": [
        {"min": [6, 0, 0], "max": [6, 20, 10],
         "block": "minecraft:polished_andesite"},
        {"min": [12, 0, 8], "max": [12, 20, 20],
         "block": "minecraft:polished_diorite"},
    ],
    "buses": [
        {"name": "bus_a", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:lime_concrete",
                   "transparent_block": "minecraft:lime_stained_glass"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "label": "bundle conducts through the S"},
        ],
    },
    "render": {"yaw": 150, "pitch": 30, "zoom": 1.75},
    "expect": "solved",
    "notes": ("This geometry is reused by the C-series configurability "
              "triptych, so the three configs are compared on the same "
              "problem."),
}
