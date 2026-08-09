"""Shared helpers for the README examples.

Only the boring parts live here (yosys invocation, endpoint hardware) so each
example file stays as short as the README snippet that quotes it.
"""
from __future__ import annotations

import os
import subprocess
import tempfile

_HERE = os.path.dirname(os.path.abspath(__file__))          # redstone-eda/docs/examples
EDA = os.path.dirname(os.path.dirname(_HERE))               # redstone-eda
ROOT = os.path.dirname(EDA)                                 # repo root
SHOWCASE = os.path.join(EDA, "showcase")
HDL_DATA = os.path.join(ROOT, "crates", "nucleation-hdl", "tests", "data")
# Scratch output: never write examples' artifacts into the repo. TMPDIR is not
# trusted here (it can point at the checkout), so anchor to a real temp root.
_TMP = "/tmp" if os.path.isdir("/tmp") else tempfile.gettempdir()
OUT = tempfile.mkdtemp(prefix="eda-examples-", dir=_TMP)

# Blocks the endpoint banks are built from.
STONE = "minecraft:stone"
DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]"
LAMP = "minecraft:redstone_lamp[lit=false]"
LEVER = "minecraft:lever[face=floor,facing=north,powered=false]"
STEP = (0, 2, 0)


def synth(verilog: str, top: str, sequential: bool = False) -> str:
    """Verilog -> BLIF with the verified pipeline's exact yosys recipe."""
    blif = os.path.join(OUT, top + ".blif")
    recipe = "synth -lut 4; " + ("dffunmap; " if sequential else "") + \
             "write_blif %s" % blif
    subprocess.run(["yosys", "-q", "-p", recipe, verilog], check=True)
    return blif


def lever_bank(s, x, z, dx, dz, n=8):
    """A column of n drive levers, each with its dust cell; -> bit-0 anchor."""
    for i in range(n):
        y = 2 + 2 * i
        s.set_block_from_string(x, y - 1, z, STONE)
        s.set_block_from_string(x, y, z, LEVER)
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE)
        s.set_block_from_string(x + dx, y, z + dz, DUST)
    return (x + dx, 2, z + dz)


def lamp_bank(s, x, z, n=8):
    """A column of n readable lamps, each with its dust cell; -> bit-0 anchor."""
    for i in range(n):
        y = 2 + 2 * i
        s.set_block_from_string(x, y - 1, z, LAMP)
        s.set_block_from_string(x, y, z, DUST)
    return (x, 2, z)
