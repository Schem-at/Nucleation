// 1-bit toggler with a NON-ZERO initial state: exercises init-by-construction
// (the DFF layer is baked at the declared Q=1).
module toggle1(clk, en, q);
  input clk, en;
  output reg q;
  initial q = 1;
  always @(posedge clk) q <= en ? ~q : q;
endmodule
