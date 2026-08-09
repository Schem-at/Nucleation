"""Shared plumbing for the demos: path setup + a sim helper.

The demos author geometry with rs.Build (which speaks full block-state
strings) and simulate with mc-tick.  from_schematic maps the schematic's
bounding-box minimum onto sim (0,0,0), and the engine sizes its world from
the ALLOCATED region -- so we round-trip through .schem to tighten bounds
(see rs.Build.sim for the original trap write-up).  Routed geometry lands in
the schematic but not in Build.cells, so the offset here comes from the
schematic's own bounding box, not from Build.bounds().
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))  # redstone-eda: rs, cells, ...

import nucleation as n  # noqa: E402
import rs  # noqa: E402


def content_min(schem):
    """Minimum corner of the blocks actually present.  bounding_box_json
    reports the ALLOCATED region, which is padded far beyond the content
    (and .schem save re-normalises to it) -- so scan the real blocks."""
    blocks = json.loads(schem.get_all_blocks_json())
    xs = [(b["x"], b["y"], b["z"]) for b in blocks if b["name"] != "minecraft:air"]
    return tuple(min(c[i] for c in xs) for i in range(3))


def simulate(schem, settle=400):
    """Tighten + boot a TickSimulation.  Returns (sim, (ox, oy, oz)):
    sim coords = build coords - offset."""
    ox, oy, oz = content_min(schem)
    tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_demo_tighten.schem")
    schem.save_to_file(tmp)
    tight = n.Schematic.open(tmp)
    sim = n.TickSimulation.from_schematic(
        tight, n.TickSettleMode.Placement, 0, 0, 0, rs.EXTRA_STATES)
    sim.run_until_quiescent(settle)
    return sim, (ox, oy, oz)


def power_at(sim, x, y, z):
    """Redstone wire power at a sim coordinate, or -1 if not a wire."""
    import re
    m = re.search(r"power=(\d+)", sim.get_block(x, y, z))
    return int(m.group(1)) if m else -1
