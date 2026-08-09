"""Typed execution: drive a saved artifact by port name and word value."""
import os

import nucleation as n
from _common import SHOWCASE

cell = n.Schematic.open(os.path.join(SHOWCASE, "bus_cross8_design.schem"))
ex = n.design.Executor.for_schematic(cell)     # contract read from the .schem metadata

ex["a_in"] = 0x55                              # ints convert to typed port Values
ex["b_in"] = 0xAA
ex.settle()                                    # run to quiescence in mc-tick
print("a_out = 0x%02X, b_out = 0x%02X" % (ex["a_out"], ex["b_out"]))
assert (ex["a_out"], ex["b_out"]) == (0x55, 0xAA)
print("OK 05_typed_exec")
