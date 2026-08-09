"""Compile combinational Verilog to a verified redstone schematic."""
import os

import nucleation as n
from _common import EDA, OUT, synth

blif = open(synth(os.path.join(EDA, "hdl", "seg7.v"), "seg7")).read()

cell = n.Hdl.compile_blif(blif, "seg7", True)          # bake=True -> settled at rest
cell.set_cell_contract_json(n.Hdl.compile_blif_contract(blif, "seg7"))
cell.save_to_file(os.path.join(OUT, "seg7.schem"))

ex = n.design.Executor.for_schematic(cell)             # typed I/O, no coordinates
ex["d"] = 0x7
ex.settle()
assert ex["seg"] == 0b0000111, bin(ex["seg"])          # digit 7 -> segments a, b, c
print("seg7(0x7) = %s  OK 01_verilog_comb" % bin(ex["seg"]))
