// 4-bit hex digit -> 7-segment decoder, seg[0]=a .. seg[6]=g, active high.
module seg7(input [3:0] d, output reg [6:0] seg);
  always @* begin
    case (d)
      4'h0: seg = 7'h3F;
      4'h1: seg = 7'h06;
      4'h2: seg = 7'h5B;
      4'h3: seg = 7'h4F;
      4'h4: seg = 7'h66;
      4'h5: seg = 7'h6D;
      4'h6: seg = 7'h7D;
      4'h7: seg = 7'h07;
      4'h8: seg = 7'h7F;
      4'h9: seg = 7'h6F;
      4'hA: seg = 7'h77;
      4'hB: seg = 7'h7C;
      4'hC: seg = 7'h39;
      4'hD: seg = 7'h5E;
      4'hE: seg = 7'h79;
      default: seg = 7'h71;   // F
    endcase
  end
endmodule
