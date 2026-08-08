// 4-bit synchronous counter: the canonical stateful HDL smoke test.
// yosys: read_verilog; synth -lut 4; write_blif  (see gen_blif.sh)
module counter4(clk, q);
  input clk;
  output reg [3:0] q;
  initial q = 0;
  always @(posedge clk) q <= q + 4'd1;
endmodule
