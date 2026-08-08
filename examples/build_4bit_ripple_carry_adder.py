#!/usr/bin/env python3
"""Build and optionally simulate a four-bit ripple-carry redstone adder.

The circuit is synthesized directly from a nine-NAND full-adder network.  No
SchematicBuilder templates or imported gate schematics are used: every block
is placed through the Nucleation Python API.

Run:
    python examples/build_4bit_ripple_carry_adder.py
    python examples/build_4bit_ripple_carry_adder.py --verify
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path

import nucleation


BITS = 4
SLICE_PITCH = 60
DEFAULT_OUTPUT = (
    Path(__file__).resolve().parents[1]
    / "artifacts"
    / "4bit_ripple_carry_adder.schem"
)

# mc-tick models gray concrete as a normal redstone conductor.  Keeping the
# support choice explicit also makes simulation and in-game behavior identical.
SUPPORT = "minecraft:gray_concrete"
SLICE_FLOORS = (
    "minecraft:blue_concrete",
    "minecraft:cyan_concrete",
    "minecraft:lime_concrete",
    "minecraft:yellow_concrete",
)
WIRE_EW = (
    "minecraft:redstone_wire[power=0,east=side,west=side,"
    "north=none,south=none]"
)
WIRE_NS = (
    "minecraft:redstone_wire[power=0,east=none,west=none,"
    "north=side,south=side]"
)
WIRE_JUNCTION = (
    "minecraft:redstone_wire[power=0,east=side,west=side,"
    "north=side,south=side]"
)
REPEATER_EAST = (
    "minecraft:repeater[facing=west,delay=1,locked=false,powered=false]"
)
REPEATER_WEST = (
    "minecraft:repeater[facing=east,delay=1,locked=false,powered=false]"
)
REPEATER_SOUTH = (
    "minecraft:repeater[facing=north,delay=1,locked=false,powered=false]"
)
REPEATER_NORTH = (
    "minecraft:repeater[facing=south,delay=1,locked=false,powered=false]"
)
TORCH_EAST = "minecraft:redstone_wall_torch[facing=east,lit=true]"


Position = tuple[int, int, int]


@dataclass(frozen=True)
class SlicePorts:
    a: Position
    b: Position
    cin: Position
    sum: Position
    cout: Position


class RedstoneLayout:
    """Small placement helpers backed only by Schematic.set_block."""

    def __init__(self, schematic: nucleation.Schematic) -> None:
        self.schematic = schematic

    def block(self, x: int, y: int, z: int, state: str) -> None:
        self.schematic.set_block(x, y, z, state)

    def supported_wire(self, x: int, y: int, z: int, state: str) -> None:
        self.block(x, y - 1, z, SUPPORT)
        self.block(x, y, z, state)

    def line_x(
        self,
        x1: int,
        x2: int,
        y: int,
        z: int,
        *,
        boost: bool = True,
    ) -> None:
        """Place a dust line, restoring eastbound runs every twelve blocks."""

        lo, hi = sorted((x1, x2))
        for x in range(lo, hi + 1):
            # Use an explicit junction state so later north/south branches do
            # not sever the east/west run when they overwrite a corner cell.
            self.supported_wire(x, y, z, WIRE_JUNCTION)

        if boost and x2 > x1:
            for x in range(x1 + 12, x2, 12):
                self.block(x, y, z, REPEATER_EAST)

    def line_z(
        self,
        x: int,
        y: int,
        z1: int,
        z2: int,
        *,
        boost: bool = True,
    ) -> None:
        """Place a north/south dust line with directional restoration."""

        lo, hi = sorted((z1, z2))
        for z in range(lo, hi + 1):
            self.supported_wire(x, y, z, WIRE_JUNCTION)

        if not boost:
            return
        if z2 > z1:
            for z in range(z1 + 12, z2, 12):
                self.block(x, y, z, REPEATER_SOUTH)
        else:
            for z in range(z1 - 12, z2, -12):
                self.block(x, y, z, REPEATER_NORTH)

    def junction(self, x: int, y: int, z: int) -> None:
        self.supported_wire(x, y, z, WIRE_JUNCTION)

    def nand(self, x: int, z: int, *, y: int = 1) -> Position:
        """Place a two-input NAND with west inputs and an east output.

        Inputs are ``(x, y, z)`` and ``(x, y, z + 4)``.  The output is
        ``(x + 6, y, z + 2)``.  Each input strongly powers its own inverter
        block.  The two wall-torch outputs are wire-ORed, implementing
        ``not(a and b)`` by De Morgan's law.
        """

        for input_z in (z, z + 4):
            for input_x in (x, x + 1):
                self.supported_wire(input_x, y, input_z, WIRE_EW)
            self.block(x + 2, y - 1, input_z, SUPPORT)
            self.block(x + 2, y, input_z, SUPPORT)
            self.block(x + 3, y, input_z, TORCH_EAST)

        for output_z in range(z, z + 5):
            self.supported_wire(x + 4, y, output_z, WIRE_NS)

        self.block(x + 5, y - 1, z + 2, SUPPORT)
        self.block(x + 5, y, z + 2, REPEATER_EAST)
        self.supported_wire(x + 6, y, z + 2, WIRE_EW)
        return (x + 6, y, z + 2)

def build_full_adder_slice(
    layout: RedstoneLayout,
    origin_x: int,
    *,
    floor_block: str,
) -> SlicePorts:
    """Place one folded six-stage, nine-NAND full-adder slice."""

    x = origin_x

    # A and B enter NAND 1.  Their bypass tracks provide fan-out to NANDs 2/3.
    a = (x - 5, 1, 10)
    b = (x - 5, 1, 14)
    layout.line_x(x - 5, x, 1, 10, boost=False)
    layout.line_x(x - 5, x, 1, 14, boost=False)
    layout.line_z(x - 3, 1, 10, 5, boost=False)
    layout.line_x(x - 3, x + 9, 1, 5, boost=False)
    layout.block(x + 2, 1, 5, REPEATER_EAST)
    layout.line_z(x - 2, 1, 14, 19, boost=False)
    layout.line_x(x - 2, x + 9, 1, 19, boost=False)
    layout.block(x + 2, 1, 19, REPEATER_EAST)
    n1 = layout.nand(x, 10)

    # NAND 2/3 and NAND 4 produce p = A xor B.
    layout.line_x(n1[0], x + 7, 1, n1[2], boost=False)
    layout.line_z(x + 7, 1, 12, 9, boost=False)
    layout.line_x(x + 7, x + 9, 1, 9, boost=False)
    layout.line_z(x + 7, 1, 12, 15, boost=False)
    layout.line_x(x + 7, x + 9, 1, 15, boost=False)
    n2 = layout.nand(x + 9, 5)
    n3 = layout.nand(x + 9, 15)

    layout.line_x(n2[0], x + 17, 1, n2[2], boost=False)
    layout.line_z(x + 17, 1, 7, 10, boost=False)
    layout.line_x(x + 17, x + 18, 1, 10, boost=False)
    layout.line_x(n3[0], x + 17, 1, n3[2], boost=False)
    layout.line_z(x + 17, 1, 17, 14, boost=False)
    layout.line_x(x + 17, x + 18, 1, 14, boost=False)
    p = layout.nand(x + 18, 10)

    # Carry-in has a long isolated backbone along the south edge.
    cin = (x - 5, 1, 32)
    layout.line_x(x - 5, x + 32, 1, 32)
    # A ripple arrives after a turn and may be below full strength; restore it
    # before the generic twelve-block spacing used by the backbone.
    layout.block(x, 1, 32, REPEATER_EAST)
    layout.line_z(x + 26, 1, 32, 14, boost=False)
    layout.block(x + 26, 1, 26, REPEATER_NORTH)
    layout.line_x(x + 26, x + 27, 1, 14, boost=False)

    # NAND 5 combines p and Cin.
    layout.line_x(p[0], x + 26, 1, p[2], boost=False)
    layout.line_z(x + 26, 1, 12, 10, boost=False)
    layout.line_x(x + 26, x + 27, 1, 10, boost=False)
    n4 = layout.nand(x + 27, 10)

    # Fan p and n4 into NAND 6/7.
    layout.line_x(p[0], x + 25, 1, p[2], boost=False)
    layout.line_z(x + 25, 1, 12, 5, boost=False)
    layout.line_x(x + 25, x + 36, 1, 5, boost=False)
    layout.block(x + 30, 1, 5, REPEATER_EAST)

    layout.line_x(n4[0], x + 34, 1, n4[2], boost=False)
    layout.line_z(x + 34, 1, 12, 9, boost=False)
    layout.line_x(x + 34, x + 36, 1, 9, boost=False)
    layout.line_z(x + 34, 1, 12, 15, boost=False)
    layout.line_x(x + 34, x + 36, 1, 15, boost=False)

    layout.line_z(x + 32, 1, 32, 19)
    layout.line_x(x + 32, x + 36, 1, 19, boost=False)
    n5 = layout.nand(x + 36, 5)
    n6 = layout.nand(x + 36, 15)

    # NAND 8 produces the sum bit.
    layout.line_x(n5[0], x + 44, 1, n5[2], boost=False)
    layout.line_z(x + 44, 1, 7, 10, boost=False)
    layout.line_x(x + 44, x + 45, 1, 10, boost=False)
    layout.line_x(n6[0], x + 43, 1, n6[2], boost=False)
    layout.line_z(x + 43, 1, 17, 14, boost=False)
    layout.line_x(x + 43, x + 45, 1, 14, boost=False)
    sum_port = layout.nand(x + 45, 10)

    # NAND 9 makes Cout = NAND(n1, n4).  n1 takes an overhead route so it
    # cannot couple to the input and propagate tracks below.
    # Rise beside n1, turn south while overhead, then descend directly into
    # NAND 9's first input.  Keeping the whole crossing above y=3 prevents n1
    # from back-powering the B and intermediate-signal tracks.
    for step in range(4):
        layout.supported_wire(
            n1[0],
            1 + step,
            n1[2] + step,
            WIRE_JUNCTION,
        )
    layout.line_z(n1[0], 4, n1[2] + 3, 25)
    layout.line_x(n1[0], x + 33, 4, 25)
    layout.block(n1[0] + 2, 4, 25, REPEATER_EAST)
    for step in range(4):
        layout.supported_wire(
            x + 33 + step,
            4 - step,
            25,
            WIRE_JUNCTION,
        )

    # n4 uses a still higher bridge; it crosses n1's y=4 bus without joining
    # it, then descends into NAND 9's second input.
    for step in range(7):
        layout.supported_wire(
            n4[0],
            1 + step,
            n4[2] + step,
            WIRE_JUNCTION,
        )
    layout.line_x(n4[0], x + 39, 7, n4[2] + 6, boost=False)
    layout.block(n4[0] + 4, 7, n4[2] + 6, REPEATER_EAST)
    layout.line_z(x + 39, 7, n4[2] + 6, 29, boost=False)
    layout.block(x + 39, 7, 23, REPEATER_SOUTH)
    for step in range(7):
        layout.supported_wire(
            x + 39,
            7 - step,
            29 + step,
            WIRE_JUNCTION,
        )
    layout.line_x(x + 39, x + 36, 1, 35, boost=False)
    layout.block(x + 38, 1, 35, REPEATER_WEST)
    layout.line_z(x + 36, 1, 35, 29, boost=False)
    layout.block(x + 36, 1, 33, REPEATER_NORTH)
    cout = layout.nand(x + 36, 25)

    # A colored strip identifies the bit slice without touching its circuitry.
    for floor_x in range(x - 6, x + 55):
        layout.block(floor_x, 0, 35, floor_block)

    return SlicePorts(a=a, b=b, cin=cin, sum=sum_port, cout=cout)


def add_io_hardware(
    layout: RedstoneLayout,
    ports: list[SlicePorts],
) -> tuple[list[Position], list[Position], Position, list[Position], Position]:
    """Add floor levers for inputs and lamps for outputs."""

    a_levers: list[Position] = []
    b_levers: list[Position] = []
    sum_lamps: list[Position] = []

    for bit, port in enumerate(ports):
        for source, collection in ((port.a, a_levers), (port.b, b_levers)):
            lever = (source[0] - 2, source[1], source[2])
            layout.block(lever[0], lever[1] - 1, lever[2], SUPPORT)
            layout.block(
                *lever,
                "minecraft:lever[face=floor,facing=east,powered=false]",
            )
            layout.line_x(lever[0] + 1, source[0], source[1], source[2], boost=False)
            collection.append(lever)

        lamp = (port.sum[0] + 2, port.sum[1], port.sum[2])
        layout.line_x(port.sum[0], lamp[0] - 1, port.sum[1], port.sum[2], boost=False)
        layout.block(*lamp, "minecraft:redstone_lamp[lit=false]")
        sum_lamps.append(lamp)

        # Bit-number marker behind the control strip.
        layout.block(port.a[0] - 2, 0, 35, SLICE_FLOORS[bit])

    cin_source = ports[0].cin
    cin_lever = (cin_source[0] - 2, cin_source[1], cin_source[2])
    layout.block(cin_lever[0], cin_lever[1] - 1, cin_lever[2], SUPPORT)
    layout.block(
        *cin_lever,
        "minecraft:lever[face=floor,facing=east,powered=false]",
    )
    layout.line_x(
        cin_lever[0] + 1,
        cin_source[0],
        cin_source[1],
        cin_source[2],
        boost=False,
    )

    cout_source = ports[-1].cout
    cout_lamp = (cout_source[0] + 2, cout_source[1], cout_source[2])
    layout.line_x(
        cout_source[0],
        cout_lamp[0] - 1,
        cout_source[1],
        cout_source[2],
        boost=False,
    )
    layout.block(*cout_lamp, "minecraft:redstone_lamp[lit=false]")
    return a_levers, b_levers, cin_lever, sum_lamps, cout_lamp


def build_adder() -> tuple[
    nucleation.Schematic,
    list[Position],
    list[Position],
    Position,
    list[Position],
    Position,
]:
    schematic = nucleation.Schematic.create("four_bit_ripple_carry_adder")
    schematic.set_author("OpenAI Codex")
    schematic.set_description(
        "Four-bit ripple-carry adder synthesized from nine-NAND full-adder slices"
    )
    layout = RedstoneLayout(schematic)

    ports = [
        build_full_adder_slice(
            layout,
            bit * SLICE_PITCH,
            floor_block=SLICE_FLOORS[bit],
        )
        for bit in range(BITS)
    ]

    # Ripple each carry output into the following slice's Cin backbone.
    for bit in range(BITS - 1):
        source = ports[bit].cout
        destination = ports[bit + 1].cin
        layout.line_x(source[0], destination[0], 1, source[2])
        layout.line_z(destination[0], 1, source[2], destination[2])
        layout.junction(*destination)

    a, b, cin, sums, cout = add_io_hardware(layout, ports)
    return schematic, a, b, cin, sums, cout


def verify_truth_table(
    schematic: nucleation.Schematic,
    a_ports: list[Position],
    b_ports: list[Position],
    cin_port: Position,
    sum_ports: list[Position],
    cout_port: Position,
) -> None:
    """Exhaustively check all 512 input combinations with Nucleation."""

    builder = nucleation.CircuitBuilder.create(schematic)
    boolean = nucleation.IoType.boolean()
    one = nucleation.LayoutFunction.one_to_one()

    for bit in range(BITS):
        builder.with_input(f"a{bit}", boolean, one, list(a_ports[bit]))
        builder.with_input(f"b{bit}", boolean, one, list(b_ports[bit]))
        builder.with_output(f"s{bit}", boolean, one, list(sum_ports[bit]))
    builder.with_input("cin", boolean, one, list(cin_port))
    builder.with_output("cout", boolean, one, list(cout_port))
    builder.validate()
    executor = builder.build()

    for a in range(1 << BITS):
        for b in range(1 << BITS):
            for cin in (0, 1):
                inputs = {"cin": bool(cin)}
                for bit in range(BITS):
                    inputs[f"a{bit}"] = bool(a & (1 << bit))
                    inputs[f"b{bit}"] = bool(b & (1 << bit))

                raw = executor.execute(
                    json.dumps(inputs),
                    nucleation.ExecutionMode.fixed_ticks(100),
                )
                outputs = json.loads(raw)["outputs"]
                actual = sum(
                    int(outputs[f"s{bit}"]["value"]) << bit
                    for bit in range(BITS)
                )
                actual |= int(outputs["cout"]["value"]) << BITS
                expected = a + b + cin
                if actual != expected:
                    raise AssertionError(
                        f"{a} + {b} + {cin}: expected {expected}, got {actual}"
                    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "output",
        nargs="?",
        type=Path,
        default=DEFAULT_OUTPUT,
        help=f"output Sponge schematic (default: {DEFAULT_OUTPUT})",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="simulate all 512 input combinations before saving",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    schematic, a, b, cin, sums, cout = build_adder()

    if args.verify:
        verify_truth_table(schematic, a, b, cin, sums, cout)
        print("Verified all 512 input combinations.")

    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    schematic.save_to_file(str(output))

    loaded = nucleation.Schematic.load_from_file(str(output))
    dimensions = loaded.tight_dimensions()
    print(f"Saved {output}")
    print(
        f"Blocks: {loaded.block_count()} | "
        f"Tight dimensions: {dimensions.x} x {dimensions.y} x {dimensions.z}"
    )


if __name__ == "__main__":
    main()
