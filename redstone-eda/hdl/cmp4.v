// 4-bit unsigned comparator.
module cmp4(input [3:0] a, input [3:0] b, output eq, output lt, output gt);
  assign eq = (a == b);
  assign lt = (a < b);
  assign gt = (a > b);
endmodule
