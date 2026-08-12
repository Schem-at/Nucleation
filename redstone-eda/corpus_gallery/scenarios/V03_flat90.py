"""FLAT 90-degree CORNER: both endpoints are flat, but on different plan axes,
which is the third form mismatch `pivot_for()` knows about."""

N = 8

SCENARIO = {
    "id": "V03_flat90",
    "title": "Flat X -> flat Z: the in-plane 90-degree corner",
    "question": ("Both ends are flat 2-pitch, but one spreads its bits along "
                 "X and the other along Z.  Does the router take the corner "
                 "without swapping lane order?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "flat_x",
         "anchor": [2, 2, 2], "width": N, "ty": "uint", "feed": [0, 0, -1]},
        {"name": "a_out", "dir": "out", "form": "flat_z",
         "anchor": [40, 2, 6], "width": N, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_corner", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:red_concrete",
                   "transparent_block": "minecraft:red_stained_glass"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "label": "corner turned, lane order preserved"},
        ],
    },
    "render": {"yaw": 140, "pitch": 30, "zoom": 1.8},
    "expect": "solved",
    "notes": ("The corner needs room: with the sink 24 blocks out the same "
          "call fails with an internal plan conflict inside the form "
          "adapter -- see V03b."),
}
