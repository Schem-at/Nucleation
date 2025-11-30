#!/usr/bin/env python3
"""
Comprehensive Python tests for Nucleation.
Tests feature parity with WASM bindings.

Run with: cd tests && python python_test.py
Or from root: maturin develop --features simulation,python && python tests/python_test.py
"""

import sys
import os

# Add the built module to path if running from development
try:
    from nucleation import (
        Schematic,
        BlockState,
        DefinitionRegion,
        SchematicBuilder,
        # Simulation classes (optional)
    )

    SIMULATION_AVAILABLE = False
    try:
        from nucleation import (
            MchprsWorld,
            Value,
            IoType,
            LayoutFunction,
            ExecutionMode,
            IoLayoutBuilder,
            TypedCircuitExecutor,
            CircuitBuilder,
        )

        SIMULATION_AVAILABLE = True
    except ImportError:
        print(
            "Note: Simulation features not available (simulation feature not enabled)"
        )
except ImportError as e:
    print(f"Error: Could not import nucleation module: {e}")
    print("Make sure to build with: maturin develop --features simulation,python")
    sys.exit(1)


def test_passed(name):
    print(f"✅ {name}")


def test_failed(name, error):
    print(f"❌ {name}: {error}")


# =============================================================================
# Core Schematic Tests
# =============================================================================


def test_schematic_creation():
    """Test basic schematic creation and operations."""
    schematic = Schematic("Test")
    assert schematic is not None
    assert schematic.block_count == 0
    test_passed("Schematic Creation")


def test_block_operations():
    """Test block set/get operations."""
    schematic = Schematic("Block Test")

    # Set simple block
    schematic.set_block(0, 0, 0, "minecraft:stone")
    block = schematic.get_block(0, 0, 0)
    assert block is not None
    assert block.name == "minecraft:stone"

    # Set block with bracket notation
    schematic.set_block_from_string(
        1, 0, 0, "minecraft:lever[facing=east,powered=true]"
    )
    block = schematic.get_block(1, 0, 0)
    assert block is not None
    assert block.name == "minecraft:lever"
    assert block.properties.get("powered") == "true"

    test_passed("Block Operations")


def test_dimensions():
    """Test dimension calculations."""
    schematic = Schematic("Dimensions Test")
    schematic.set_block(0, 0, 0, "minecraft:stone")
    schematic.set_block(5, 5, 5, "minecraft:stone")

    dims = schematic.dimensions
    assert dims[0] >= 6  # Width >= 6
    assert dims[1] >= 6  # Height >= 6
    assert dims[2] >= 6  # Length >= 6

    test_passed("Dimensions")


# =============================================================================
# DefinitionRegion Tests (NEW - Feature Parity with WASM)
# =============================================================================


def test_definition_region_creation():
    """Test basic DefinitionRegion creation."""
    region = DefinitionRegion()
    assert region.is_empty()
    assert region.volume() == 0

    region.add_bounds((0, 0, 0), (2, 0, 0))
    assert not region.is_empty()
    assert region.volume() == 3

    test_passed("DefinitionRegion Creation")


def test_definition_region_from_bounds():
    """Test DefinitionRegion.from_bounds."""
    region = DefinitionRegion.from_bounds((0, 0, 0), (2, 2, 2))
    assert region.volume() == 27  # 3x3x3
    test_passed("DefinitionRegion from_bounds")


def test_definition_region_boolean_operations():
    """Test boolean operations (subtract, intersect, union)."""
    # Create regions
    region_a = DefinitionRegion.from_bounds((0, 0, 0), (5, 0, 0))  # 6 points
    region_b = DefinitionRegion.from_bounds((3, 0, 0), (7, 0, 0))  # 5 points

    # Test subtract (mutating)
    a_copy = DefinitionRegion.from_bounds((0, 0, 0), (5, 0, 0))
    a_copy.subtract(region_b)
    assert a_copy.volume() == 3  # [0, 1, 2]

    # Test intersect (mutating)
    a_copy2 = DefinitionRegion.from_bounds((0, 0, 0), (5, 0, 0))
    a_copy2.intersect(region_b)
    assert a_copy2.volume() == 3  # [3, 4, 5]

    # Test union (immutable)
    union_region = region_a.union(region_b)
    assert union_region.volume() == 8  # [0-7]

    # Test immutable variants
    subtracted = region_a.subtracted(region_b)
    assert subtracted.volume() == 3

    intersected = region_a.intersected(region_b)
    assert intersected.volume() == 3

    test_passed("DefinitionRegion Boolean Operations")


def test_definition_region_geometric_ops():
    """Test geometric transformations (shift, expand, contract)."""
    region = DefinitionRegion.from_bounds((0, 0, 0), (2, 2, 2))

    # Test shift
    region.shift(10, 20, 30)
    bounds = region.get_bounds()
    assert bounds["min"] == (10, 20, 30)
    assert bounds["max"] == (12, 22, 32)

    # Test expand
    region2 = DefinitionRegion.from_bounds((5, 5, 5), (10, 10, 10))
    region2.expand(2, 2, 2)
    bounds2 = region2.get_bounds()
    assert bounds2["min"] == (3, 3, 3)
    assert bounds2["max"] == (12, 12, 12)

    # Test contract
    region2.contract(2)
    bounds3 = region2.get_bounds()
    assert bounds3["min"] == (5, 5, 5)
    assert bounds3["max"] == (10, 10, 10)

    test_passed("DefinitionRegion Geometric Operations")


def test_definition_region_connectivity():
    """Test connectivity analysis (is_contiguous, connected_components)."""
    # Contiguous L-shape
    region = DefinitionRegion()
    region.add_bounds((0, 0, 0), (2, 0, 0))
    region.add_bounds((0, 0, 0), (0, 2, 0))
    assert region.is_contiguous()
    assert region.connected_components() == 1

    # Disconnected regions
    disconnected = DefinitionRegion()
    disconnected.add_point(0, 0, 0)
    disconnected.add_point(10, 10, 10)  # Far away
    assert not disconnected.is_contiguous()
    assert disconnected.connected_components() == 2

    test_passed("DefinitionRegion Connectivity")


def test_definition_region_positions():
    """Test position iteration and sorting."""
    region = DefinitionRegion.from_bounds((0, 0, 0), (1, 1, 1))

    positions = region.positions()
    assert len(positions) == 8  # 2x2x2

    # Test sorted positions (deterministic order)
    sorted_positions = region.positions_sorted()
    assert len(sorted_positions) == 8
    # Should be sorted by Y, then X, then Z
    assert sorted_positions[0] == (0, 0, 0)

    test_passed("DefinitionRegion Positions")


def test_definition_region_contains():
    """Test contains method."""
    region = DefinitionRegion.from_bounds((0, 0, 0), (5, 5, 5))

    assert region.contains(0, 0, 0)
    assert region.contains(3, 3, 3)
    assert region.contains(5, 5, 5)
    assert not region.contains(6, 6, 6)
    assert not region.contains(-1, 0, 0)

    test_passed("DefinitionRegion Contains")


# =============================================================================
# DefinitionRegion Renderer Support Tests (NEW)
# =============================================================================


def test_definition_region_from_bounding_boxes():
    """Test from_bounding_boxes factory method."""
    boxes = [((0, 0, 0), (2, 2, 2)), ((5, 5, 5), (7, 7, 7))]
    region = DefinitionRegion.from_bounding_boxes(boxes)

    assert region.box_count() == 2
    assert region.contains(1, 1, 1)
    assert region.contains(6, 6, 6)
    assert not region.contains(3, 3, 3)

    test_passed("DefinitionRegion from_bounding_boxes")


def test_definition_region_from_positions():
    """Test from_positions factory method."""
    positions = [(0, 0, 0), (1, 0, 0), (2, 0, 0)]
    region = DefinitionRegion.from_positions(positions)

    assert region.volume() == 3
    assert region.contains(0, 0, 0)
    assert region.contains(1, 0, 0)
    assert region.contains(2, 0, 0)

    test_passed("DefinitionRegion from_positions")


def test_definition_region_box_access():
    """Test box access methods for rendering."""
    boxes = [((0, 0, 0), (2, 2, 2)), ((5, 5, 5), (7, 7, 7))]
    region = DefinitionRegion.from_bounding_boxes(boxes)

    # Test box_count
    assert region.box_count() == 2

    # Test get_box
    box0 = region.get_box(0)
    assert box0 == ((0, 0, 0), (2, 2, 2))

    box1 = region.get_box(1)
    assert box1 == ((5, 5, 5), (7, 7, 7))

    assert region.get_box(2) is None

    # Test get_boxes
    all_boxes = region.get_boxes()
    assert len(all_boxes) == 2
    assert all_boxes[0] == ((0, 0, 0), (2, 2, 2))
    assert all_boxes[1] == ((5, 5, 5), (7, 7, 7))

    test_passed("DefinitionRegion Box Access")


def test_definition_region_metadata():
    """Test metadata access methods."""
    region = DefinitionRegion.from_bounds((0, 0, 0), (5, 5, 5))

    # Test set/get metadata
    region.set_metadata("color", "red")
    region.set_metadata("label", "Input A")

    assert region.get_metadata("color") == "red"
    assert region.get_metadata("label") == "Input A"
    assert region.get_metadata("nonexistent") is None

    # Test with_metadata chaining
    region2 = (
        DefinitionRegion.from_bounds((0, 0, 0), (1, 1, 1))
        .with_metadata("type", "input")
        .with_metadata("index", "0")
    )
    assert region2.get_metadata("type") == "input"
    assert region2.get_metadata("index") == "0"

    # Test get_all_metadata
    all_meta = region.get_all_metadata()
    assert "color" in all_meta
    assert all_meta["color"] == "red"

    # Test metadata_keys
    keys = region.metadata_keys()
    assert "color" in keys
    assert "label" in keys

    test_passed("DefinitionRegion Metadata")


def test_definition_region_dimensions():
    """Test dimensions method."""
    region = DefinitionRegion.from_bounds((0, 0, 0), (9, 4, 2))
    dims = region.dimensions()
    assert dims == (10, 5, 3)  # inclusive bounds

    # Empty region
    empty = DefinitionRegion()
    assert empty.dimensions() == (0, 0, 0)

    test_passed("DefinitionRegion Dimensions")


def test_definition_region_center():
    """Test center methods."""
    region = DefinitionRegion.from_bounds((0, 0, 0), (10, 10, 10))

    # Integer center
    center = region.center()
    assert center == (5, 5, 5)

    # Float center
    center_f32 = region.center_f32()
    assert center_f32 is not None
    assert abs(center_f32[0] - 5.5) < 0.1
    assert abs(center_f32[1] - 5.5) < 0.1
    assert abs(center_f32[2] - 5.5) < 0.1

    # Empty region
    empty = DefinitionRegion()
    assert empty.center() is None
    assert empty.center_f32() is None

    test_passed("DefinitionRegion Center")


def test_definition_region_intersects_bounds():
    """Test intersects_bounds for frustum culling."""
    region = DefinitionRegion.from_bounds((0, 0, 0), (10, 10, 10))

    # Intersecting
    assert region.intersects_bounds((5, 5, 5), (15, 15, 15))
    assert region.intersects_bounds((-5, -5, -5), (5, 5, 5))
    assert region.intersects_bounds((5, 5, 5), (6, 6, 6))  # Inside

    # Not intersecting
    assert not region.intersects_bounds((20, 20, 20), (30, 30, 30))
    assert not region.intersects_bounds((-10, -10, -10), (-1, -1, -1))

    test_passed("DefinitionRegion Intersects Bounds")


def test_definition_region_immutable_transforms():
    """Test immutable transformation methods."""
    original = DefinitionRegion.from_bounds((0, 0, 0), (5, 5, 5))

    # Test shifted
    shifted = original.shifted(10, 20, 30)
    orig_bounds = original.get_bounds()
    shifted_bounds = shifted.get_bounds()

    assert orig_bounds["min"] == (0, 0, 0)
    assert shifted_bounds["min"] == (10, 20, 30)

    # Test expanded
    expanded = original.expanded(2, 2, 2)
    exp_bounds = expanded.get_bounds()
    assert orig_bounds["min"] == (0, 0, 0)  # Original unchanged
    assert exp_bounds["min"] == (-2, -2, -2)
    assert exp_bounds["max"] == (7, 7, 7)

    # Test contracted
    contracted = expanded.contracted(2)
    cont_bounds = contracted.get_bounds()
    assert cont_bounds["min"] == (0, 0, 0)
    assert cont_bounds["max"] == (5, 5, 5)

    test_passed("DefinitionRegion Immutable Transforms")


def test_definition_region_copy():
    """Test copy method and Python copy protocol."""
    import copy

    original = DefinitionRegion.from_bounds((0, 0, 0), (5, 5, 5))
    original.set_metadata("name", "test")

    # Test explicit copy
    copied = original.copy()
    copied.shift(100, 100, 100)

    orig_bounds = original.get_bounds()
    copy_bounds = copied.get_bounds()
    assert orig_bounds["min"] == (0, 0, 0)
    assert copy_bounds["min"] == (100, 100, 100)

    # Test Python copy protocol
    original2 = DefinitionRegion.from_bounds((0, 0, 0), (3, 3, 3))
    shallow = copy.copy(original2)
    deep = copy.deepcopy(original2)

    shallow.shift(10, 10, 10)
    deep.shift(20, 20, 20)

    orig2_bounds = original2.get_bounds()
    assert orig2_bounds["min"] == (0, 0, 0)

    test_passed("DefinitionRegion Copy")


# =============================================================================
# Simulation Tests (require simulation feature)
# =============================================================================


def test_simulation_basic():
    """Test basic simulation functionality."""
    if not SIMULATION_AVAILABLE:
        print("⏭️ Skipping simulation tests (feature not enabled)")
        return

    schematic = Schematic("Sim Test")
    schematic.set_block(0, 0, 0, "minecraft:gray_concrete")
    schematic.set_block(
        0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]"
    )
    for x in range(1, 5):
        schematic.set_block(x, 1, 0, "minecraft:redstone_wire[power=0]")
    schematic.set_block(5, 1, 0, "minecraft:redstone_lamp[lit=false]")

    world = schematic.create_simulation_world()

    # Initial state
    assert not world.is_lit(5, 1, 0)

    # Toggle lever
    world.on_use_block(0, 1, 0)
    world.tick(5)
    world.flush()

    assert world.is_lit(5, 1, 0)

    test_passed("Simulation Basic")


def test_typed_executor():
    """Test TypedCircuitExecutor functionality."""
    if not SIMULATION_AVAILABLE:
        return

    schematic = Schematic("Executor Test")
    schematic.set_block(0, 0, 0, "minecraft:gray_concrete")
    schematic.set_block(
        0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]"
    )
    for x in range(1, 5):
        schematic.set_block(x, 1, 0, "minecraft:redstone_wire[power=0]")
    schematic.set_block(5, 1, 0, "minecraft:redstone_lamp[lit=false]")

    # Build layout
    world = schematic.create_simulation_world()
    builder = IoLayoutBuilder()
    builder.add_input_auto("in", IoType.boolean(), [(0, 1, 0)])
    builder.add_output_auto("out", IoType.boolean(), [(5, 1, 0)])
    layout = builder.build()

    executor = TypedCircuitExecutor.from_layout(world, layout)

    # Execute
    result = executor.execute({"in": True}, ExecutionMode.fixed_ticks(100))

    assert "outputs" in result
    assert "out" in result["outputs"]

    test_passed("TypedCircuitExecutor")


def test_manual_tick_control():
    """Test manual tick control (tick, flush, set_input, read_output)."""
    if not SIMULATION_AVAILABLE:
        return

    schematic = Schematic("Manual Tick Test")
    schematic.set_block(0, 0, 0, "minecraft:gray_concrete")
    schematic.set_block(
        0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]"
    )
    for x in range(1, 5):
        schematic.set_block(x, 1, 0, "minecraft:redstone_wire[power=0]")
    schematic.set_block(5, 1, 0, "minecraft:redstone_lamp[lit=false]")

    world = schematic.create_simulation_world()
    builder = IoLayoutBuilder()
    builder.add_input_auto("lever", IoType.boolean(), [(0, 1, 0)])
    builder.add_output_auto("lamp", IoType.boolean(), [(5, 1, 0)])
    layout = builder.build()

    executor = TypedCircuitExecutor.from_layout(world, layout)
    executor.set_state_mode("manual")

    # Test input_names and output_names
    assert "lever" in executor.input_names()
    assert "lamp" in executor.output_names()

    # Set input and tick manually
    executor.set_input("lever", Value.bool(True))
    executor.tick(5)
    executor.flush()

    output = executor.read_output("lamp")
    assert output == True

    test_passed("Manual Tick Control")


def test_circuit_builder():
    """Test CircuitBuilder fluent API."""
    if not SIMULATION_AVAILABLE:
        return

    schematic = Schematic("Builder Test")
    schematic.set_block(0, 0, 0, "minecraft:gray_concrete")
    schematic.set_block(
        0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]"
    )
    for x in range(1, 5):
        schematic.set_block(x, 1, 0, "minecraft:redstone_wire[power=0]")
    schematic.set_block(5, 1, 0, "minecraft:redstone_lamp[lit=false]")

    input_region = DefinitionRegion()
    input_region.add_point(0, 1, 0)

    output_region = DefinitionRegion()
    output_region.add_point(5, 1, 0)

    builder = CircuitBuilder(schematic)
    builder.with_input_auto("in", IoType.boolean(), input_region)
    builder.with_output_auto("out", IoType.boolean(), output_region)

    assert builder.input_count() == 1
    assert builder.output_count() == 1
    assert "in" in builder.input_names()
    assert "out" in builder.output_names()

    executor = builder.build()
    result = executor.execute({"in": True}, ExecutionMode.fixed_ticks(100))

    assert result["outputs"]["out"] == True

    test_passed("CircuitBuilder Flow")


def test_io_layout_builder_regions():
    """Test IoLayoutBuilder with region-based methods."""
    if not SIMULATION_AVAILABLE:
        return

    schematic = Schematic("Region Test")
    schematic.set_block(0, 0, 0, "minecraft:gray_concrete")
    schematic.set_block(
        0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]"
    )
    for x in range(1, 5):
        schematic.set_block(x, 1, 0, "minecraft:redstone_wire[power=0]")
    schematic.set_block(5, 1, 0, "minecraft:redstone_lamp[lit=false]")

    region = DefinitionRegion()
    region.add_point(0, 1, 0)

    world = schematic.create_simulation_world()
    builder = IoLayoutBuilder()
    builder.add_input_from_region_auto("region_in", IoType.boolean(), region)
    layout = builder.build()

    assert "region_in" in layout.input_names()

    test_passed("IoLayoutBuilder Region Methods")


def test_get_layout_info():
    """Test get_layout_info for debugging."""
    if not SIMULATION_AVAILABLE:
        return

    schematic = Schematic("Layout Info Test")
    schematic.set_block(0, 0, 0, "minecraft:gray_concrete")
    schematic.set_block(
        0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]"
    )
    for x in range(1, 5):
        schematic.set_block(x, 1, 0, "minecraft:redstone_wire[power=0]")
    schematic.set_block(5, 1, 0, "minecraft:redstone_lamp[lit=false]")

    world = schematic.create_simulation_world()
    builder = IoLayoutBuilder()
    builder.add_input_auto("lever", IoType.boolean(), [(0, 1, 0)])
    builder.add_output_auto("lamp", IoType.boolean(), [(5, 1, 0)])
    layout = builder.build()

    executor = TypedCircuitExecutor.from_layout(world, layout)
    layout_info = executor.get_layout_info()

    assert "inputs" in layout_info
    assert "outputs" in layout_info
    assert "lever" in layout_info["inputs"]
    assert "lamp" in layout_info["outputs"]
    assert layout_info["inputs"]["lever"]["bit_count"] == 1

    test_passed("Get Layout Info")


# =============================================================================
# SortStrategy Tests (NEW)
# =============================================================================


def test_sort_strategy_creation():
    """Test SortStrategy factory methods."""
    if not SIMULATION_AVAILABLE:
        return

    from nucleation import SortStrategy

    # Test all factory methods
    yxz = SortStrategy.yxz()
    xyz = SortStrategy.xyz()
    zyx = SortStrategy.zyx()
    y_desc = SortStrategy.y_desc_xz()
    x_desc = SortStrategy.x_desc_yz()
    z_desc = SortStrategy.z_desc_yx()
    descending = SortStrategy.descending()
    preserve = SortStrategy.preserve()
    reverse = SortStrategy.reverse()
    dist = SortStrategy.distance_from(5, 5, 5)
    dist_desc = SortStrategy.distance_from_desc(5, 5, 5)

    # Verify names
    assert yxz.name == "y_x_z"
    assert xyz.name == "x_y_z"
    assert zyx.name == "z_y_x"
    assert y_desc.name == "y_desc_x_z"
    assert preserve.name == "preserve"
    assert reverse.name == "reverse"
    assert dist.name == "distance_from"

    test_passed("SortStrategy Creation")


def test_sort_strategy_from_string():
    """Test SortStrategy.from_string parsing."""
    if not SIMULATION_AVAILABLE:
        return

    from nucleation import SortStrategy

    # Valid strings
    assert SortStrategy.from_string("yxz").name == "y_x_z"
    assert SortStrategy.from_string("y_x_z").name == "y_x_z"
    assert SortStrategy.from_string("y").name == "y_x_z"
    assert SortStrategy.from_string("xyz").name == "x_y_z"
    assert SortStrategy.from_string("y_desc").name == "y_desc_x_z"
    assert SortStrategy.from_string("preserve").name == "preserve"
    assert SortStrategy.from_string("boxOrder").name == "preserve"
    assert SortStrategy.from_string("reverse").name == "reverse"

    # Invalid string should raise
    try:
        SortStrategy.from_string("invalid")
        assert False, "Should have raised ValueError"
    except ValueError:
        pass

    test_passed("SortStrategy from_string")


def test_circuit_builder_with_sort_strategy():
    """Test CircuitBuilder with custom sort strategies."""
    if not SIMULATION_AVAILABLE:
        return

    from nucleation import SortStrategy

    # Create schematic with 8 positions across 2 Y levels
    schematic = Schematic("Sort Strategy Test")
    for y in [1, 2]:
        for x in range(4):
            schematic.set_block(x, 0, 0, "minecraft:stone")
            schematic.set_block(x, y, 0, "minecraft:redstone_wire[power=0]")

    # Create region spanning two Y levels
    region = DefinitionRegion()
    region.add_bounds((0, 1, 0), (3, 1, 0))  # Y=1, 4 blocks
    region.add_bounds((0, 2, 0), (3, 2, 0))  # Y=2, 4 blocks

    # Test with default sort (YXZ)
    builder = CircuitBuilder(schematic)
    builder.with_input_auto("default", IoType.unsigned_int(8), region)
    assert builder.input_count() == 1
    test_passed("CircuitBuilder default sort")

    # Test with Y descending sort
    builder2 = CircuitBuilder(schematic)
    builder2.with_input_auto_sorted(
        "y_desc", IoType.unsigned_int(8), region, SortStrategy.y_desc_xz()
    )
    assert builder2.input_count() == 1
    test_passed("CircuitBuilder Y descending sort")

    # Test with preserve (box order)
    builder3 = CircuitBuilder(schematic)
    builder3.with_input_auto_sorted(
        "preserve", IoType.unsigned_int(8), region, SortStrategy.preserve()
    )
    assert builder3.input_count() == 1
    test_passed("CircuitBuilder preserve sort")

    # Test with distance-based sort
    builder4 = CircuitBuilder(schematic)
    builder4.with_input_auto_sorted(
        "distance",
        IoType.unsigned_int(8),
        region,
        SortStrategy.distance_from(2, 1, 0),
    )
    assert builder4.input_count() == 1
    test_passed("CircuitBuilder distance sort")

    # Test output with sort strategy
    builder5 = CircuitBuilder(schematic)
    builder5.with_output_auto_sorted(
        "out", IoType.unsigned_int(8), region, SortStrategy.xyz()
    )
    assert builder5.output_count() == 1
    test_passed("CircuitBuilder output with sort")


# =============================================================================
# Run All Tests
# =============================================================================


def run_all_tests():
    print("=" * 60)
    print("Running Nucleation Python Tests")
    print("=" * 60)
    print()

    # Core tests
    print("--- Core Schematic Tests ---")
    test_schematic_creation()
    test_block_operations()
    test_dimensions()
    print()

    # DefinitionRegion tests
    print("--- DefinitionRegion Tests ---")
    test_definition_region_creation()
    test_definition_region_from_bounds()
    test_definition_region_boolean_operations()
    test_definition_region_geometric_ops()
    test_definition_region_connectivity()
    test_definition_region_positions()
    test_definition_region_contains()
    print()

    # DefinitionRegion Renderer Support tests (NEW)
    print("--- DefinitionRegion Renderer Support Tests ---")
    test_definition_region_from_bounding_boxes()
    test_definition_region_from_positions()
    test_definition_region_box_access()
    test_definition_region_metadata()
    test_definition_region_dimensions()
    test_definition_region_center()
    test_definition_region_intersects_bounds()
    test_definition_region_immutable_transforms()
    test_definition_region_copy()
    print()

    # Simulation tests
    print("--- Simulation Tests ---")
    test_simulation_basic()
    test_typed_executor()
    test_manual_tick_control()
    test_circuit_builder()
    test_io_layout_builder_regions()
    test_get_layout_info()
    print()

    # SortStrategy tests (NEW)
    print("--- SortStrategy Tests ---")
    test_sort_strategy_creation()
    test_sort_strategy_from_string()
    test_circuit_builder_with_sort_strategy()
    print()

    print("=" * 60)
    print("All tests completed!")
    print("=" * 60)


if __name__ == "__main__":
    run_all_tests()
