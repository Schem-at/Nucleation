"""BIT-SLICING: one 8-bit word fans out to two 4-bit halves.

This is the width-adaptation path (`route_bus_adapted`), which is reachable
only through `Design.raw` -- the Python veneer wraps `route_bus` but not
`route_bus_adapted`, so a user following the idiomatic API cannot slice a bus
at all.  Recorded here because it works and should be surfaced.
"""

SCENARIO = {
    "id": "P03_slice_hi_lo",
    "title": "Bit-slicing: 8-bit word -> low nibble + high nibble",
    "question": ("Split one 8-bit driver into two 4-bit sinks: `lo` takes "
                 "bits 0-3 (lsb-aligned), `hi` takes bits 4-7 (shift -4). "
                 "Both need `truncate`, since each drops half the word."),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "vertical",
         "anchor": [1, 2, 4], "width": 8, "ty": "uint"},
        {"name": "lo_out", "dir": "out", "form": "vertical",
         "anchor": [14, 2, 4], "width": 4, "ty": "uint"},
        {"name": "hi_out", "dir": "out", "form": "vertical",
         "anchor": [14, 12, 4], "width": 4, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_lo", "driver": "a_in", "sinks": ["lo_out"],
         "adapted": {"align": 0, "shift": 0, "truncate": True},
         "style": {"bus_block": "minecraft:light_blue_concrete"}},
        {"name": "bus_hi", "driver": "a_in", "sinks": ["hi_out"],
         "adapted": {"align": 2, "shift": -4, "truncate": True},
         "style": {"bus_block": "minecraft:yellow_concrete"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "lo_out", "patterns": "walking+extremes",
             "map": {"kind": "identity"}, "label": "low nibble = word & 0xF"},
            {"in": "a_in", "out": "hi_out", "patterns": "walking+extremes",
             "map": {"kind": "shift", "by": -4},
             "label": "high nibble = word >> 4"},
        ],
    },
    "render": {"yaw": 145, "pitch": 28, "zoom": 1.85},
    "expect": "solved",
    "notes": ("`truncate=True` is required: without it the router REFUSES a "
              "lossy connection, which is the right default -- losing a word's "
              "high bits is not the router's call."),
}
