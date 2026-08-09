"""Compile SEQUENTIAL Verilog (a 4-bit counter) and clock it in the simulator."""
import os

import nucleation as n
from _common import HDL_DATA, synth

blif = open(synth(os.path.join(HDL_DATA, "counter4.v"), "counter4",
                  sequential=True)).read()          # yosys: + dffunmap

cell = n.Hdl.compile_blif(blif, "counter4", True)   # DFF bank + clock spine
cell.set_cell_contract_json(n.Hdl.compile_blif_contract(blif, "counter4"))

ex = n.design.Executor.for_schematic(cell)          # `clk` is a typed Boolean port
seen = []
for _ in range(6):
    seen.append(ex["q"])                            # init is baked by construction
    ex["clk"] = True;  ex.settle(400)               # rising edge
    ex["clk"] = False; ex.settle(400)
assert seen == [0, 1, 2, 3, 4, 5], seen
print("counter4 q =", seen, " OK 02_verilog_seq")
