import json

import pytest

from nucleation import Schematic


def test_region_lifecycle_and_strict_named_placement():
    schematic = Schematic.create("regions")
    schematic.create_region("wing")
    assert schematic.has_region("wing")

    schematic.set_block_in_region(
        "wing",
        4,
        5,
        6,
        "minecraft:chest[facing=east]{CustomName:'\"Supplies\"'}",
    )
    state = schematic.get_block_in_region("wing", 4, 5, 6)
    assert state.name() == "minecraft:chest"
    assert json.loads(state.properties_json())["facing"] == "east"
    assert "Supplies" in schematic.get_block_entity_json_in_region("wing", 4, 5, 6)

    schematic.translate_region("wing", 3, -2, 4)
    translated = json.loads(
        schematic.get_block_entity_json_in_region("wing", 7, 3, 10)
    )
    assert translated["position"] == [7, 3, 10]

    schematic.set_block_in_region("wing", 7, 3, 10, "minecraft:stone")
    with pytest.raises(Exception):
        schematic.get_block_entity_json_in_region("wing", 7, 3, 10)

    schematic.rename_region("wing", "east_wing")
    assert not schematic.has_region("wing")
    assert schematic.has_region("east_wing")
    schematic.remove_region("east_wing")
    assert not schematic.has_region("east_wing")

    with pytest.raises(Exception):
        schematic.remove_region("Main")


def test_transform_scopes_validation_and_deep_clone():
    schematic = Schematic.create("transforms")
    schematic.set_block(0, 0, 0, "minecraft:quartz_block")
    assert json.loads(schematic.region_bounding_box_json("Main")) == [0, 0, 0, 0, 0, 0]
    assert "minecraft:quartz_block" in json.loads(schematic.region_palette_json("Main"))
    schematic.set_block_in_region("wing", 2, 0, 0, "minecraft:stone")
    schematic.set_block_in_region(
        "wing", 3, 0, 0, "minecraft:oak_stairs[facing=east]"
    )

    with pytest.raises(Exception):
        schematic.rotate_y(45)
    assert schematic.get_block_name(0, 0, 0) == "minecraft:quartz_block"

    schematic.rotate_region_y("wing", 90)
    state = schematic.get_block_in_region("wing", 2, 0, 1)
    assert json.loads(state.properties_json())["facing"] == "south"
    assert schematic.get_block_name(0, 0, 0) == "minecraft:quartz_block"

    clone = schematic.deep_clone()
    clone.translate_schematic(10, 0, 0)
    assert schematic.get_block_name(0, 0, 0) == "minecraft:quartz_block"
    assert clone.get_block_name(10, 0, 0) == "minecraft:quartz_block"


def test_stamp_exclusions_and_named_region_selection():
    source = Schematic.create("source")
    source.set_block(0, 0, 0, "minecraft:stone")
    source.set_block(1, 0, 0, "minecraft:gold_block")
    source.set_block_in_region("tower", 4, 0, 7, "minecraft:diamond_block")

    destination = Schematic.create("destination")
    destination.set_block(10, 0, 0, "minecraft:copper_block")
    destination.stamp_box(
        source,
        0,
        0,
        0,
        1,
        0,
        0,
        10,
        0,
        0,
        '["minecraft:stone"]',
    )
    assert destination.get_block_name(10, 0, 0) == "minecraft:copper_block"
    assert destination.get_block_name(11, 0, 0) == "minecraft:gold_block"

    destination.stamp_region(source, "tower", 20, 0, 0, "[]")
    assert destination.get_block_name(20, 0, 0) == "minecraft:diamond_block"

    with pytest.raises(Exception):
        destination.stamp_region(source, "missing", 30, 0, 0, "[]")
