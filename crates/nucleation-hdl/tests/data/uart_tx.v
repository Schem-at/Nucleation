// 8N1 UART serializer with a divide-by-2 baud divider (stretch example).
// One frame: start bit (0), 8 data bits LSB first, stop bit (1); each held
// for DIV clock cycles. tx idles high.
module uart_tx(clk, start, data, tx, busy);
  input clk, start;
  input [7:0] data;
  output tx, busy;
  parameter DIV = 2;
  reg divc;
  reg [3:0] bitc;
  reg [9:0] sh;
  reg active;
  initial begin divc = 0; bitc = 0; sh = 10'b1111111111; active = 0; end
  assign tx = sh[0];
  assign busy = active;
  always @(posedge clk) begin
    if (!active) begin
      if (start) begin
        sh <= {1'b1, data, 1'b0};
        bitc <= 0;
        divc <= 0;
        active <= 1;
      end
    end else begin
      if (divc == DIV - 1) begin
        divc <= 0;
        sh <= {1'b1, sh[9:1]};
        if (bitc == 9) active <= 0;
        else bitc <= bitc + 4'd1;
      end else divc <= divc + 1'd1;
    end
  end
endmodule
