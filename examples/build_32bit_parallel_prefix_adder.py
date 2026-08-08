#!/usr/bin/env python3
"""Build a 32-bit Sklansky parallel-prefix adder with Nucleation.

The implementation is synthesized block-by-block from redstone NAND gates.  It
has five prefix levels, carry-in, 64 operand levers, 32 sum lamps, and a
carry-out lamp.  No SchematicBuilder template or imported gate schematic is
used.

Run:
    python examples/build_32bit_parallel_prefix_adder.py
    python examples/build_32bit_parallel_prefix_adder.py --verify
"""

from __future__ import annotations

import argparse
import json
import random
from dataclasses import dataclass
from pathlib import Path

import nucleation

from build_4bit_ripple_carry_adder import (
    REPEATER_EAST,
    REPEATER_NORTH,
    REPEATER_SOUTH,
    SUPPORT,
    WIRE_JUNCTION,
    RedstoneLayout,
)


BITS = 32
LANE_PITCH = 40
PREFIX_LEVELS = 5
PREFIX_FIRST_X = 49
PREFIX_STAGE_PITCH = 40
PREFIX_FINAL_X = PREFIX_FIRST_X + (PREFIX_LEVELS - 1) * PREFIX_STAGE_PITCH + 15
CARRY_CELL_X = PREFIX_FINAL_X + 36
XOR_CELL_X = CARRY_CELL_X + 25
DEFAULT_OUTPUT = (
    Path(__file__).resolve().parents[1]
    / "artifacts"
    / "32bit_sklansky_parallel_prefix_adder.schem"
)

CONTROL_COLORS = (
    "minecraft:blue_concrete",
    "minecraft:cyan_concrete",
    "minecraft:lime_concrete",
    "minecraft:yellow_concrete",
)

Position = tuple[int, int, int]


@dataclass(frozen=True)
class SignalPair:
    generate: Position
    propagate: Position


@dataclass(frozen=True)
class PgCell:
    a: Position
    b: Position
    signals: SignalPair


@dataclass(frozen=True)
class AdderPorts:
    a: list[Position]
    b: list[Position]
    cin: Position
    sums: list[Position]
    cout: Position


def stair_up_x(
    layout: RedstoneLayout,
    x: int,
    y: int,
    z: int,
    rise: int,
) -> Position:
    for step in range(rise + 1):
        layout.supported_wire(x + step, y + step, z, WIRE_JUNCTION)
    return (x + rise, y + rise, z)


def stair_down_x(
    layout: RedstoneLayout,
    x: int,
    y: int,
    z: int,
    drop: int,
) -> Position:
    for step in range(drop + 1):
        layout.supported_wire(x + step, y - step, z, WIRE_JUNCTION)
    return (x + drop, y - drop, z)


def stair_down_z(
    layout: RedstoneLayout,
    x: int,
    y: int,
    z: int,
    drop: int,
) -> Position:
    for step in range(drop + 1):
        layout.supported_wire(x, y - step, z + step, WIRE_JUNCTION)
    return (x, y - drop, z + drop)


def place_pg_cell(layout: RedstoneLayout, x: int, z: int) -> PgCell:
    """Create bit propagate and generate: P=A xor B, G=A and B."""

    a = (x - 5, 1, z + 10)
    b = (x - 5, 1, z + 14)
    layout.line_x(x - 5, x, 1, z + 10, boost=False)
    layout.line_x(x - 5, x, 1, z + 14, boost=False)

    # Preserve A/B for the second NAND stage.
    layout.line_z(x - 3, 1, z + 10, z + 5, boost=False)
    layout.line_x(x - 3, x + 9, 1, z + 5, boost=False)
    layout.block(x + 2, 1, z + 5, REPEATER_EAST)
    layout.line_z(x - 2, 1, z + 14, z + 19, boost=False)
    layout.line_x(x - 2, x + 9, 1, z + 19, boost=False)
    layout.block(x + 2, 1, z + 19, REPEATER_EAST)

    n1 = layout.nand(x, z + 10)
    layout.line_x(n1[0], x + 7, 1, n1[2], boost=False)
    layout.line_z(x + 7, 1, z + 12, z + 9, boost=False)
    layout.line_x(x + 7, x + 9, 1, z + 9, boost=False)
    layout.line_z(x + 7, 1, z + 12, z + 15, boost=False)
    layout.line_x(x + 7, x + 9, 1, z + 15, boost=False)

    n2 = layout.nand(x + 9, z + 5)
    n3 = layout.nand(x + 9, z + 15)
    layout.line_x(n2[0], x + 17, 1, n2[2], boost=False)
    layout.line_z(x + 17, 1, z + 7, z + 10, boost=False)
    layout.line_x(x + 17, x + 18, 1, z + 10, boost=False)
    layout.line_x(n3[0], x + 17, 1, n3[2], boost=False)
    layout.line_z(x + 17, 1, z + 17, z + 14, boost=False)
    layout.line_x(x + 17, x + 18, 1, z + 14, boost=False)
    propagate = layout.nand(x + 18, z + 10)

    # G = not(NAND(A,B)).  The n1 takeoff is lifted above the XOR wiring.
    generate = layout.nand(x + 18, z + 25)
    for step in range(4):
        layout.supported_wire(
            n1[0],
            1 + step,
            n1[2] + step,
            WIRE_JUNCTION,
        )
    layout.line_z(n1[0], 4, z + 15, z + 27, boost=False)
    layout.block(n1[0], 4, z + 20, REPEATER_SOUTH)
    layout.line_x(n1[0], x + 15, 4, z + 27, boost=False)
    layout.block(n1[0] + 4, 4, z + 27, REPEATER_EAST)
    stair_down_x(layout, x + 15, 4, z + 27, 3)
    layout.line_z(x + 18, 1, z + 25, z + 29, boost=False)

    return PgCell(
        a=a,
        b=b,
        signals=SignalPair(generate=generate, propagate=propagate),
    )


def place_black_cell(layout: RedstoneLayout, x: int, z: int) -> SignalPair:
    """Place G=Gh|(Ph&Gl), P=Ph&Pl using five NAND gates."""

    not_gh = layout.nand(x, z + 20)
    not_term = layout.nand(x, z + 30)
    not_p = layout.nand(x, z + 10)
    g_out = layout.nand(x + 9, z + 25)
    p_out = layout.nand(x + 9, z + 10)

    layout.line_x(not_gh[0], x + 8, 1, not_gh[2], boost=False)
    layout.line_z(x + 8, 1, z + 22, z + 25, boost=False)
    layout.line_x(x + 8, x + 9, 1, z + 25, boost=False)

    layout.line_x(not_term[0], x + 7, 1, not_term[2], boost=False)
    layout.line_z(x + 7, 1, z + 32, z + 29, boost=False)
    layout.line_x(x + 7, x + 9, 1, z + 29, boost=False)

    layout.line_x(not_p[0], x + 7, 1, not_p[2], boost=False)
    layout.line_z(x + 7, 1, z + 10, z + 14, boost=False)
    layout.line_x(x + 7, x + 9, 1, z + 10, boost=False)
    layout.line_x(x + 7, x + 9, 1, z + 14, boost=False)

    return SignalPair(generate=g_out, propagate=p_out)


def place_gray_cell(layout: RedstoneLayout, x: int, z: int) -> Position:
    """Place C=G|(P&Cin), with west ports P,Cin,G,G."""

    not_term = layout.nand(x, z)
    not_g = layout.nand(x, z + 10)
    carry = layout.nand(x + 9, z + 4)

    layout.line_x(not_term[0], x + 8, 1, not_term[2], boost=False)
    layout.line_z(x + 8, 1, z + 2, z + 4, boost=False)
    layout.line_x(x + 8, x + 9, 1, z + 4, boost=False)

    layout.line_x(not_g[0], x + 7, 1, not_g[2], boost=False)
    layout.line_z(x + 7, 1, z + 12, z + 8, boost=False)
    layout.line_x(x + 7, x + 9, 1, z + 8, boost=False)
    return carry


def place_xor_cell(layout: RedstoneLayout, x: int, z: int) -> Position:
    """Place a four-NAND XOR. Inputs enter at z+10 and z+14."""

    layout.line_x(x - 5, x, 1, z + 10, boost=False)
    layout.line_x(x - 5, x, 1, z + 14, boost=False)
    n1 = layout.nand(x, z + 10)
    layout.line_z(x - 3, 1, z + 10, z + 5, boost=False)
    layout.line_x(x - 3, x + 9, 1, z + 5, boost=False)
    layout.block(x - 2, 1, z + 5, REPEATER_EAST)
    layout.line_z(x - 2, 1, z + 14, z + 19, boost=False)
    layout.line_x(x - 2, x + 9, 1, z + 19, boost=False)
    layout.block(x - 1, 1, z + 19, REPEATER_EAST)

    layout.line_x(n1[0], x + 7, 1, n1[2], boost=False)
    layout.line_z(x + 7, 1, z + 12, z + 9, boost=False)
    layout.line_x(x + 7, x + 9, 1, z + 9, boost=False)
    layout.line_z(x + 7, 1, z + 12, z + 15, boost=False)
    layout.line_x(x + 7, x + 9, 1, z + 15, boost=False)

    n2 = layout.nand(x + 9, z + 5)
    n3 = layout.nand(x + 9, z + 15)
    layout.line_x(n2[0], x + 17, 1, n2[2], boost=False)
    layout.line_z(x + 17, 1, z + 7, z + 10, boost=False)
    layout.line_x(x + 17, x + 18, 1, z + 10, boost=False)
    layout.line_x(n3[0], x + 17, 1, n3[2], boost=False)
    layout.line_z(x + 17, 1, z + 17, z + 14, boost=False)
    layout.line_x(x + 17, x + 18, 1, z + 14, boost=False)
    return layout.nand(x + 18, z + 10)


def route_original_propagate(
    layout: RedstoneLayout,
    source: Position,
    lane_z: int,
) -> Position:
    """Preserve P_i above the prefix network for the final sum XOR."""

    stair_up_x(layout, source[0], source[1], source[2], 3)
    layout.line_z(source[0] + 3, 4, source[2], lane_z + 39)
    layout.line_x(source[0] + 3, XOR_CELL_X - 9, 4, lane_z + 39)
    return (XOR_CELL_X - 9, 4, lane_z + 39)


def route_prefix_stage(
    layout: RedstoneLayout,
    previous: list[SignalPair],
    stage: int,
) -> list[SignalPair]:
    """Route and place one fan-out-optimized Sklansky prefix level."""

    cell_x = PREFIX_FIRST_X + stage * PREFIX_STAGE_PITCH
    output_x = cell_x + 15
    half = 1 << stage
    span = half << 1
    updated: dict[int, int] = {}

    for base in range(0, BITS, span):
        low_bit = base + half - 1
        for bit in range(base + half, min(base + span, BITS)):
            updated[bit] = low_bit

    # Bits in the lower half of each Sklansky group pass straight through.
    for bit, pair in enumerate(previous):
        if bit in updated:
            continue
        lane_z = bit * LANE_PITCH
        layout.line_x(pair.generate[0], output_x, 1, lane_z + 27)
        layout.block(pair.generate[0] + 1, 1, lane_z + 27, REPEATER_EAST)
        layout.line_x(pair.propagate[0], output_x, 1, lane_z + 12)
        layout.block(pair.propagate[0] + 1, 1, lane_z + 12, REPEATER_EAST)

    # Local high generate/propagate inputs for each black cell.
    for bit in updated:
        lane_z = bit * LANE_PITCH
        pair = previous[bit]

        g_track_x = cell_x - 4
        layout.line_x(pair.generate[0], g_track_x, 1, lane_z + 27)
        layout.block(pair.generate[0] + 1, 1, lane_z + 27, REPEATER_EAST)
        layout.line_z(g_track_x, 1, lane_z + 27, lane_z + 20, boost=False)
        layout.block(g_track_x, 1, lane_z + 26, REPEATER_NORTH)
        layout.line_x(g_track_x, cell_x, 1, lane_z + 20, boost=False)
        layout.block(g_track_x + 1, 1, lane_z + 20, REPEATER_EAST)
        layout.line_x(g_track_x, cell_x, 1, lane_z + 24, boost=False)
        layout.block(g_track_x + 1, 1, lane_z + 24, REPEATER_EAST)

        stair_up_x(
            layout,
            pair.propagate[0],
            pair.propagate[1],
            pair.propagate[2],
            3,
        )
        p_track_x = cell_x - 3
        layout.line_x(pair.propagate[0] + 3, p_track_x, 4, lane_z + 12)
        layout.block(pair.propagate[0] + 4, 4, lane_z + 12, REPEATER_EAST)
        layout.line_z(p_track_x, 4, lane_z + 12, lane_z + 10, boost=False)
        layout.line_z(p_track_x, 4, lane_z + 12, lane_z + 30)
        layout.block(p_track_x, 4, lane_z + 13, REPEATER_SOUTH)
        stair_down_x(layout, p_track_x, 4, lane_z + 10, 3)
        stair_down_x(layout, p_track_x, 4, lane_z + 30, 3)

    # Each lower-half endpoint fans out once on isolated elevated buses.
    for base in range(0, BITS, span):
        low_bit = base + half - 1
        destinations = list(range(base + half, min(base + span, BITS)))
        if not destinations:
            continue

        low_pair = previous[low_bit]
        low_z = low_bit * LANE_PITCH
        last_z = destinations[-1] * LANE_PITCH

        g_bus_x = cell_x - 10
        g_stair_x = cell_x - 16
        layout.line_z(
            low_pair.generate[0],
            1,
            low_z + 27,
            low_z + 36,
            boost=False,
        )
        layout.block(low_pair.generate[0], 1, low_z + 28, REPEATER_SOUTH)
        layout.line_x(low_pair.generate[0], g_stair_x, 1, low_z + 36)
        layout.block(low_pair.generate[0] + 1, 1, low_z + 36, REPEATER_EAST)
        stair_up_x(layout, g_stair_x, 1, low_z + 36, 6)
        layout.line_z(g_bus_x, 7, low_z + 36, last_z + 34)
        layout.block(g_bus_x, 7, low_z + 37, REPEATER_SOUTH)

        p_bus_x = cell_x - 12
        p_stair_x = cell_x - 21
        layout.line_z(
            low_pair.propagate[0],
            1,
            low_z + 12,
            low_z + 16,
            boost=False,
        )
        layout.block(low_pair.propagate[0], 1, low_z + 13, REPEATER_SOUTH)
        layout.line_x(low_pair.propagate[0], p_stair_x, 1, low_z + 16)
        layout.block(low_pair.propagate[0] + 1, 1, low_z + 16, REPEATER_EAST)
        stair_up_x(layout, p_stair_x, 1, low_z + 16, 9)
        layout.line_z(p_bus_x, 10, low_z + 16, last_z + 14)
        layout.block(p_bus_x, 10, low_z + 17, REPEATER_SOUTH)

        for bit in destinations:
            lane_z = bit * LANE_PITCH

            layout.line_x(g_bus_x, cell_x - 6, 7, lane_z + 34, boost=False)
            layout.block(g_bus_x + 1, 7, lane_z + 34, REPEATER_EAST)
            stair_down_x(layout, cell_x - 6, 7, lane_z + 34, 6)

            p_branch_x = cell_x - 9
            layout.line_x(p_bus_x, p_branch_x, 10, lane_z + 14, boost=False)
            layout.block(p_bus_x + 1, 10, lane_z + 14, REPEATER_EAST)
            layout.line_z(
                p_branch_x,
                10,
                lane_z + 14,
                lane_z + 5,
                boost=False,
            )
            layout.block(p_branch_x, 10, lane_z + 13, REPEATER_NORTH)
            layout.line_x(p_branch_x, cell_x, 10, lane_z + 5, boost=False)
            layout.block(p_branch_x + 1, 10, lane_z + 5, REPEATER_EAST)
            layout.block(cell_x - 1, 10, lane_z + 5, REPEATER_EAST)
            stair_down_z(layout, cell_x, 10, lane_z + 5, 9)

    for bit in updated:
        place_black_cell(layout, cell_x, bit * LANE_PITCH)

    return [
        SignalPair(
            generate=(output_x, 1, bit * LANE_PITCH + 27),
            propagate=(output_x, 1, bit * LANE_PITCH + 12),
        )
        for bit in range(BITS)
    ]


def route_cin_bus(layout: RedstoneLayout) -> tuple[Position, list[Position]]:
    """Build a high global Cin spine and isolated drops for c[0..32]."""

    bus_x = CARRY_CELL_X - 15
    bus_z0 = -4
    lever = (PREFIX_FINAL_X + 1, 1, bus_z0)
    layout.block(lever[0], 0, lever[2], SUPPORT)
    layout.block(
        *lever,
        "minecraft:lever[face=floor,facing=east,powered=false]",
    )
    layout.line_x(lever[0] + 1, bus_x - 12, 1, bus_z0, boost=False)
    layout.block(bus_x - 14, 1, bus_z0, REPEATER_EAST)
    stair_up_x(layout, bus_x - 12, 1, bus_z0, 12)

    last_z = BITS * LANE_PITCH + 4
    layout.line_z(bus_x, 13, bus_z0, last_z, boost=False)
    # The offset never coincides with a z=40i+4 branch point.
    # The climb reaches the spine with only a little residual strength, so
    # restore it immediately and then every twelve blocks.  This -3 mod 12
    # offset never lands on a z=40i+4 branch point.
    for repeater_z in range(bus_z0 + 1, last_z, 12):
        layout.block(bus_x, 13, repeater_z, REPEATER_SOUTH)

    carry_inputs: list[Position] = []
    for carry_index in range(BITS + 1):
        branch_z = carry_index * LANE_PITCH + 4
        layout.line_x(bus_x, bus_x + 3, 13, branch_z, boost=False)
        layout.block(bus_x + 1, 13, branch_z, REPEATER_EAST)
        carry_inputs.append(
            stair_down_x(layout, bus_x + 3, 13, branch_z, 12)
        )
    return lever, carry_inputs


def route_prefix_to_gray(
    layout: RedstoneLayout,
    pair: SignalPair,
    carry_index: int,
) -> None:
    """Feed final prefix G/P into gray carry cell c[carry_index]."""

    lane_z = carry_index * LANE_PITCH

    g_track_x = CARRY_CELL_X - 4
    layout.line_x(pair.generate[0], g_track_x, 1, pair.generate[2])
    layout.block(pair.generate[0] + 1, 1, pair.generate[2], REPEATER_EAST)
    layout.line_z(g_track_x, 1, pair.generate[2], lane_z + 14)
    layout.block(g_track_x, 1, pair.generate[2] + 1, REPEATER_SOUTH)
    layout.line_x(g_track_x, CARRY_CELL_X, 1, lane_z + 10, boost=False)
    layout.block(g_track_x + 1, 1, lane_z + 10, REPEATER_EAST)
    layout.line_x(g_track_x, CARRY_CELL_X, 1, lane_z + 14, boost=False)
    layout.block(g_track_x + 1, 1, lane_z + 14, REPEATER_EAST)

    p_track_x = CARRY_CELL_X - 6
    stair_up_x(
        layout,
        pair.propagate[0],
        pair.propagate[1],
        pair.propagate[2],
        6,
    )
    layout.line_x(pair.propagate[0] + 6, p_track_x, 7, pair.propagate[2])
    layout.block(pair.propagate[0] + 7, 7, pair.propagate[2], REPEATER_EAST)
    descent_z = lane_z - 8
    layout.line_z(p_track_x, 7, pair.propagate[2], descent_z)
    layout.block(p_track_x, 7, pair.propagate[2] + 1, REPEATER_SOUTH)
    layout.line_x(p_track_x, CARRY_CELL_X, 7, descent_z, boost=False)
    layout.block(p_track_x + 1, 7, descent_z, REPEATER_EAST)
    stair_down_z(layout, CARRY_CELL_X, 7, descent_z, 6)
    layout.line_z(CARRY_CELL_X, 1, lane_z - 2, lane_z, boost=False)


def add_io_hardware(
    layout: RedstoneLayout,
    pg_cells: list[PgCell],
    original_p: list[Position],
    final_pairs: list[SignalPair],
) -> AdderPorts:
    """Add operand controls, carry gray cells, sum XORs, and lamps."""

    a_levers: list[Position] = []
    b_levers: list[Position] = []
    sum_lamps: list[Position] = []

    for bit, pg in enumerate(pg_cells):
        color = CONTROL_COLORS[(bit // 8) % len(CONTROL_COLORS)]
        for source, collection in ((pg.a, a_levers), (pg.b, b_levers)):
            lever = (source[0] - 2, source[1], source[2])
            layout.block(lever[0], lever[1] - 1, lever[2], color)
            layout.block(
                *lever,
                "minecraft:lever[face=floor,facing=east,powered=false]",
            )
            layout.line_x(
                lever[0] + 1,
                source[0],
                source[1],
                source[2],
                boost=False,
            )
            collection.append(lever)

    cin_lever, carry_inputs = route_cin_bus(layout)

    # c0 is Cin; c1..c32 come from gray cells fed by the final prefixes.
    layout.line_x(
        carry_inputs[0][0],
        CARRY_CELL_X + 15,
        1,
        carry_inputs[0][2],
    )
    layout.block(carry_inputs[0][0] + 1, 1, carry_inputs[0][2], REPEATER_EAST)
    layout.line_z(
        CARRY_CELL_X + 15,
        1,
        carry_inputs[0][2],
        6,
        boost=False,
    )
    carries: list[Position] = [(CARRY_CELL_X + 15, 1, 6)]

    for carry_index in range(1, BITS + 1):
        route_prefix_to_gray(
            layout,
            final_pairs[carry_index - 1],
            carry_index,
        )
        carry = place_gray_cell(
            layout,
            CARRY_CELL_X,
            carry_index * LANE_PITCH,
        )
        carries.append(carry)

    # S_i = P_i xor c_i.
    for bit in range(BITS):
        lane_z = bit * LANE_PITCH
        carry = carries[bit]
        layout.line_x(carry[0], XOR_CELL_X - 8, 1, carry[2], boost=False)
        layout.block(carry[0] + 1, 1, carry[2], REPEATER_EAST)
        layout.line_z(
            XOR_CELL_X - 8,
            1,
            carry[2],
            lane_z + 10,
            boost=False,
        )
        layout.line_x(
            XOR_CELL_X - 8,
            XOR_CELL_X - 5,
            1,
            lane_z + 10,
            boost=False,
        )

        p_track = original_p[bit]
        layout.line_z(p_track[0], 4, p_track[2], lane_z + 14)
        layout.block(p_track[0], 4, p_track[2] - 1, REPEATER_NORTH)
        stair_down_x(layout, p_track[0], 4, lane_z + 14, 3)
        layout.line_x(
            p_track[0] + 3,
            XOR_CELL_X - 5,
            1,
            lane_z + 14,
            boost=False,
        )

        sum_out = place_xor_cell(layout, XOR_CELL_X, lane_z)
        lamp = (sum_out[0] + 2, sum_out[1], sum_out[2])
        layout.line_x(sum_out[0], lamp[0] - 1, sum_out[1], sum_out[2], boost=False)
        layout.block(*lamp, "minecraft:redstone_lamp[lit=false]")
        sum_lamps.append(lamp)

    cout = carries[BITS]
    cout_lamp = (cout[0] + 2, cout[1], cout[2])
    layout.line_x(cout[0], cout_lamp[0] - 1, cout[1], cout[2], boost=False)
    layout.block(*cout_lamp, "minecraft:redstone_lamp[lit=false]")

    return AdderPorts(
        a=a_levers,
        b=b_levers,
        cin=cin_lever,
        sums=sum_lamps,
        cout=cout_lamp,
    )


def build_adder() -> tuple[nucleation.Schematic, AdderPorts]:
    schematic = nucleation.Schematic.create(
        "thirty_two_bit_sklansky_parallel_prefix_adder"
    )
    schematic.set_author("OpenAI Codex")
    schematic.set_description(
        "32-bit Sklansky parallel-prefix adder synthesized directly from "
        "redstone NAND gates"
    )
    layout = RedstoneLayout(schematic)

    pg_cells = [
        place_pg_cell(layout, 0, bit * LANE_PITCH)
        for bit in range(BITS)
    ]
    original_p = [
        route_original_propagate(
            layout,
            pg.signals.propagate,
            bit * LANE_PITCH,
        )
        for bit, pg in enumerate(pg_cells)
    ]

    pairs = [pg.signals for pg in pg_cells]
    for stage in range(PREFIX_LEVELS):
        pairs = route_prefix_stage(layout, pairs, stage)

    ports = add_io_hardware(layout, pg_cells, original_p, pairs)
    return schematic, ports


def make_executor(
    schematic: nucleation.Schematic,
    ports: AdderPorts,
) -> nucleation.CircuitExecutor:
    builder = nucleation.CircuitBuilder.create(schematic)
    boolean = nucleation.IoType.boolean()
    one = nucleation.LayoutFunction.one_to_one()

    for bit in range(BITS):
        builder.with_input(f"a{bit}", boolean, one, list(ports.a[bit]))
        builder.with_input(f"b{bit}", boolean, one, list(ports.b[bit]))
        builder.with_output(f"s{bit}", boolean, one, list(ports.sums[bit]))
    builder.with_input("cin", boolean, one, list(ports.cin))
    builder.with_output("cout", boolean, one, list(ports.cout))
    builder.validate()
    return builder.build()


def simulate_case(
    executor: nucleation.CircuitExecutor,
    a: int,
    b: int,
    cin: int,
) -> None:
    inputs = {"cin": bool(cin)}
    for bit in range(BITS):
        inputs[f"a{bit}"] = bool(a & (1 << bit))
        inputs[f"b{bit}"] = bool(b & (1 << bit))

    raw = executor.execute(
        json.dumps(inputs),
        nucleation.ExecutionMode.fixed_ticks(300),
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
            f"0x{a:08x} + 0x{b:08x} + {cin}: "
            f"expected 0x{expected:09x}, got 0x{actual:09x}"
        )


def verify_adder(schematic: nucleation.Schematic, ports: AdderPorts) -> int:
    """Check edge cases and deterministic random vectors in mc-tick."""

    mask = (1 << BITS) - 1
    vectors = [
        (0, 0, 0),
        (0, 0, 1),
        (mask, 0, 0),
        (mask, 0, 1),
        (mask, mask, 0),
        (mask, mask, 1),
        (0xAAAAAAAA, 0x55555555, 0),
        (0xAAAAAAAA, 0x55555555, 1),
        (0x7FFFFFFF, 1, 0),
        (0x80000000, 0x80000000, 0),
    ]
    rng = random.Random(0x32ADD)
    vectors.extend(
        (rng.getrandbits(BITS), rng.getrandbits(BITS), rng.getrandbits(1))
        for _ in range(6)
    )

    executor = make_executor(schematic, ports)
    for a, b, cin in vectors:
        simulate_case(executor, a, b, cin)
    return len(vectors)


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
        help="simulate edge cases and deterministic random vectors",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    schematic, ports = build_adder()

    if args.verify:
        cases = verify_adder(schematic, ports)
        print(f"Verified {cases} representative 32-bit additions.")

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
    print("Topology: 5-level Sklansky | 80 black cells | 32 gray cells")


if __name__ == "__main__":
    main()
