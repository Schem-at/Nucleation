"""Shared geometry for the configurability triptych.

The whole point of the C-series is that the PROBLEM is identical and only the
CONFIG differs, so the problem lives here exactly once.  Files starting with
`_` are not loaded as scenarios.
"""

N = 8


def serpentine(sid, title, question, style=None, rule=None, notes="",
               render=None):
    """The O02 congestion problem, ready for a different config."""
    bus = {"name": "bus_a", "driver": "a_in", "sinks": ["a_out"]}
    if style:
        bus["style"] = style
    if rule:
        bus["rule"] = rule
    return {
        "id": sid,
        "title": title,
        "question": question,
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
        "buses": [bus],
        "verify": {
            "words": [
                {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
                 "label": "bundle conducts under this config"},
            ],
        },
        "render": render or {"yaw": 150, "pitch": 30, "zoom": 1.75},
        "expect": "solved",
        "notes": notes,
        "_family": "configurability",
    }
