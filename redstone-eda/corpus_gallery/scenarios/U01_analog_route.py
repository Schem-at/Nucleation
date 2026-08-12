"""UNSOLVED BY CONSTRUCTION: ask the router to carry an ANALOG value.

The hex trunk's tap presents a signal strength -- a number from 1..15 on ONE
wire.  This scenario declares a port over that tap and asks the router to
deliver it 14 blocks away, then sweeps every level and reads the STRENGTH at
the far end (not a bit).

`IoType` (src/io_contract/io_type.rs) has UnsignedInt, SignedInt, Float32,
Boolean, Ascii, Array, Matrix, PixelBuffer, Struct, BitArray -- and nothing for
"a value carried in signal strength".  So the router necessarily treats this
lane as boolean and the number is destroyed by dust decay.  The sweep below
measures exactly how it is destroyed, which is the useful part.
"""

SCENARIO = {
    "id": "U01_analog_route",
    "title": "Routing an ANALOG value (signal strength) between two ports",
    "question": ("The hex tap carries 1..15 on one wire.  Can the router move "
                 "that VALUE 14 blocks, the way it moves a bit?"),
    "fixtures": [
        {"kind": "hex_trunk", "name": "hex0", "at": [0, 0, 0],
         "values": [1, 3, 7, 11, 15]},
    ],
    "ports": [
        # declared OVER the fixture's own tap dust: no hardware stamped
        {"name": "hex_tap", "dir": "in", "form": "vertical",
         "anchor": [2, 1, 0], "width": 1, "ty": "uint", "stamp": False},
        {"name": "far_out", "dir": "out", "form": "vertical",
         "anchor": [16, 1, 0], "width": 1, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_analog", "driver": "hex_tap", "sinks": ["far_out"],
         "style": {"bus_block": "minecraft:brown_concrete"}},
    ],
    "verify": {
        "analog": [
            {"fixture": "hex0", "read_power": ["far_out"],
             "label": "does the analog VALUE survive the routed lane?"},
        ],
    },
    "render": {"yaw": 145, "pitch": 26, "zoom": 1.9},
    "expect": "unsolved",
    "notes": ("TWO blocking assumptions, and the second one bites first.\n\n"
              "1. PORTS MUST BE BACKED BY HARDWARE THE ROUTER RECOGNISES. "
              "`declare_input` over the tap is refused with InvalidArgument "
              "before any routing happens.  Measured: a dust cell with a LEVER "
              "beside it declares as an input; a dust cell on a LAMP declares "
              "as an output; a bare dust tap on stone is refused in both "
              "directions.  So the router cannot be connected to another "
              "mechanism's output at all -- only to lever/lamp banks it "
              "understands.  For a tool meant to route between real builds, "
              "this is the sharpest edge in the whole corpus.\n\n"
              "2. NO ANALOG TYPE, NO ANALOG CARRIER.  Even given a port, every "
              "bus is bits-on-lanes: `IoType` has no signal-strength variant, "
              "and the router has no form that emits the value-preserving hex "
              "stage (mechanism 12 of notes-hex-transport.md), which is "
              "measured, verified and sitting in the corpus unused."),
}
