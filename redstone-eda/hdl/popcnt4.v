// Population count of a 4-bit vector.
module popcnt4(input [3:0] d, output [2:0] cnt);
  assign cnt = d[0] + d[1] + d[2] + d[3];
endmodule
