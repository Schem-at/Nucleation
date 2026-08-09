// Moore FSM: "11" sequence detector. z rises after two consecutive x=1
// samples and stays up while x holds 1.
module fsm(clk, x, z);
  input clk, x;
  output z;
  reg [1:0] s;
  initial s = 0;
  always @(posedge clk) begin
    case (s)
      2'd0: s <= x ? 2'd1 : 2'd0;
      2'd1: s <= x ? 2'd2 : 2'd0;
      2'd2: s <= x ? 2'd2 : 2'd0;
      default: s <= 2'd0;
    endcase
  end
  assign z = (s == 2'd2);
endmodule
