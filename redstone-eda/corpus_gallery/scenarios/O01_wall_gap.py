"""OBSTACLE: a solid pier sits exactly on the straight line between the two
ports, from the floor to above the top bit.  The bundle has to go round it in
z, in lockstep, and arrive with its bit order intact."""

N = 8

SCENARIO = {
    "id": "O01_wall_gap",
    "title": "Obstacle avoidance: a full-height pier on the direct path",
    "question": ("A 3-deep solid pier blocks x=8, z=7..9, y=0..20 -- the whole "
                 "straight path including every bit's support.  Does the "
                 "router detour the whole bundle and keep it coherent?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "vertical",
         "anchor": [1, 2, 8], "width": N, "ty": "uint"},
        {"name": "a_out", "dir": "out", "form": "vertical",
         "anchor": [16, 2, 8], "width": N, "ty": "uint"},
    ],
    "obstacles": [
        {"min": [8, 0, 7], "max": [8, 20, 9],
         "block": "minecraft:polished_andesite"},
    ],
    "buses": [
        {"name": "bus_a", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:lime_concrete",
                   "transparent_block": "minecraft:lime_stained_glass"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "label": "bundle conducts past the pier"},
        ],
    },
    "render": {"yaw": 145, "pitch": 28, "zoom": 1.8},
    "expect": "solved",
    "notes": ("The pier is loose-layer blocks, not a declared keepout: this "
              "tests that the router treats OCCUPIED cells as occupied."),
}
