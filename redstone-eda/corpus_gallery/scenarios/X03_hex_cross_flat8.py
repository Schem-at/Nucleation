"""FLAGSHIP (2/2): the same crossing, eight times over.

In the FLAT 2-pitch form the eight lanes are spread across z, so the bundle
does not cross the hex trunk once -- it crosses it eight times, at eight
different z, every one of them landing on a cell the trunk's floor or devices
already own.  This is the hard version of the flagship case.
"""

N = 8

SCENARIO = {
    "id": "X03_hex_cross_flat8",
    "title": "Eight flat lanes crossing a hex analog trunk (8 crossings)",
    "question": ("A flat 2-pitch 8-bit bundle runs +X at y=2; the hex trunk "
                 "runs -Z through x=10..12.  Every lane has to get past the "
                 "trunk independently.  Does the router solve all eight, and "
                 "does the analog value still arrive exact?"),
    "fixtures": [
        {"kind": "hex_trunk", "name": "hex0", "at": [10, 0, 0],
         "values": [1, 3, 7, 11, 15]},
    ],
    "ports": [
        {"name": "a_in", "dir": "in", "form": "flat_z",
         "anchor": [1, 2, 2], "width": N, "ty": "uint", "feed": [-1, 0, 0]},
        {"name": "a_out", "dir": "out", "form": "flat_z",
         "anchor": [20, 2, 2], "width": N, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_bin", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:magenta_concrete",
                   "transparent_block": "minecraft:magenta_stained_glass"}},
    ],
    "verify": {
        "hold_analog": {"fixture": "hex0", "level": 7},
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "label": "8 flat lanes intact while hex carries 7"},
        ],
        "analog": [
            {"fixture": "hex0", "hold_ports": ["a_in"], "hold_bits": 0x55,
             "also_read": ["a_out"],
             "label": "hex value exact while the bundle carries 0x55"},
        ],
    },
    "render": {"yaw": 140, "pitch": 32, "zoom": 1.75},
    "expect": "solved",
    "notes": ("Same fixture as X02.  If this one fails where X02 passes, the "
              "limit is per-lane crossing in the flat form, not the crossing "
              "idea."),
}
