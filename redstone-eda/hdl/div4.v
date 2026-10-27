// 4-bit unsigned divider: q = a / b, r = a % b.
// Division by zero follows RISC-V: q = all ones, r = a.
module div4(input [3:0] a, input [3:0] b, output [3:0] q, output [3:0] r);
  assign q = (b == 0) ? 4'hF : a / b;
  assign r = (b == 0) ? a : a % b;
endmodule
