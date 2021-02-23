module top(
    input pin_sw,
    output pin_led
    );
    assign pin_led = ~pin_sw;
endmodule
