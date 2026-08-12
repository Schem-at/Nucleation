"""Two 8-bit buses that MUST cross: the baseline the rest of the corpus
builds on.  The crossing is never asked for -- it is implied by the endpoint
geometry, and the router discovers it."""

N = 8

SCENARIO = {
    "id": "X01_cross8",
    "title": "Two 8-bit buses crossing at 90 degrees",
    "question": ("Bus A runs +X at z=8, bus B runs +Z at x=8, both in the "
                 "dense 2y vertical form.  They cannot both go straight. "
                 "Does the router find the crossing without being told?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "vertical",
         "anchor": [1, 2, 8], "width": N, "ty": "uint"},
        {"name": "a_out", "dir": "out", "form": "vertical",
         "anchor": [16, 2, 8], "width": N, "ty": "uint"},
        {"name": "b_in", "dir": "in", "form": "vertical",
         "anchor": [8, 2, 1], "width": N, "ty": "uint", "feed": [0, 0, -1]},
        {"name": "b_out", "dir": "out", "form": "vertical",
         "anchor": [8, 2, 16], "width": N, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_a", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:lime_concrete"}},
        {"name": "bus_b", "driver": "b_in", "sinks": ["b_out"],
         "style": {"bus_block": "minecraft:cyan_concrete",
                   "transparent_block": "minecraft:cyan_stained_glass"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "label": "bus_a conducts",
             "patterns": "walking+extremes"},
            {"in": "b_in", "out": "b_out", "label": "bus_b conducts",
             "patterns": "walking+extremes"},
        ],
    },
    "render": {"yaw": 145, "pitch": 30, "zoom": 1.8},
    "expect": "solved",
    "notes": ("The reference case: `showcase/bus_cross8_design.schem` in the "
              "README is the same design, so a regression here means the "
              "harness is wrong, not the router."),
}
