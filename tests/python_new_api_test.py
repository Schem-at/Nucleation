"""Tests for the polished Python API introduced in api_upgrade.md.

Run from the repo root after `maturin develop` (or `pip install -e .`):

    pytest tests/python_new_api_test.py -v
"""

from __future__ import annotations

import warnings
from pathlib import Path

import pytest

import nucleation
from nucleation import (
    Block,
    BlockState,
    ButtonPress,
    Cursor,
    Item,
    Schematic,
    SchematicBuilder,
    UseBlock,
    chest,
    sign,
    text,
)


# --------------------------------------------------------------------- Block


class TestBlock:
    def test_basic_construction(self):
        b = Block("minecraft:stone")
        assert b.id == "minecraft:stone"
        assert b.state is None
        assert b.nbt is None

    def test_parse_no_state(self):
        assert Block.parse("minecraft:stone") == Block("minecraft:stone")

    def test_parse_with_state(self):
        b = Block.parse("minecraft:repeater[delay=4,facing=east,powered=false]")
        assert b.id == "minecraft:repeater"
        assert b.state == {"delay": 4, "facing": "east", "powered": False}

    def test_parse_with_snbt(self):
        b = Block.parse("minecraft:jukebox[has_record=true]{signal:5}")
        assert b.id == "minecraft:jukebox"
        assert b.state == {"has_record": True}
        assert b.nbt == {"__snbt__": "signal:5"}

    def test_with_state_returns_new_instance(self):
        b = Block("minecraft:chest")
        b2 = b.with_state(facing="west")
        assert b is not b2
        assert b.state is None
        assert b2.state == {"facing": "west"}

    def test_to_string_roundtrip_state(self):
        b = Block("minecraft:repeater", state={"delay": 4, "facing": "east"})
        # Order is dict insertion order (deterministic in py3.7+).
        s = b.to_string()
        assert s.startswith("minecraft:repeater[")
        assert "delay=4" in s and "facing=east" in s

    def test_frozen(self):
        b = Block("minecraft:stone")
        with pytest.raises(Exception):
            b.id = "minecraft:dirt"  # frozen dataclass


# ----------------------------------------------------------------- Schematic


class TestSchematicConstruction:
    def test_new_classmethod(self):
        s = Schematic.new("test")
        assert s.name == "test"

    def test_legacy_constructor_blank(self):
        s = Schematic("plain-name")
        assert s.name == "plain-name"

    def test_legacy_constructor_with_extension_no_file(self):
        # Even when the extension matches but no file exists, we don't blow up.
        s = Schematic("does-not-exist.schem")
        assert s is not None

    def test_open_classmethod_roundtrip(self, tmp_path: Path):
        # Save then reopen.
        s = Schematic.new("rt").set_block(0, 0, 0, "minecraft:stone")
        f = tmp_path / "rt.schem"
        s.save(f)
        loaded = Schematic.open(f)
        assert loaded.get_block_string(0, 0, 0) == "minecraft:stone"


class TestSetBlock:
    def test_legacy_4arg_form(self):
        s = Schematic.new("a")
        out = s.set_block(0, 0, 0, "minecraft:stone")
        assert out is s
        assert s.get_block_string(0, 0, 0) == "minecraft:stone"

    def test_new_tuple_form(self):
        s = Schematic.new("b")
        s.set_block((1, 2, 3), "minecraft:dirt")
        assert s.get_block_string(1, 2, 3) == "minecraft:dirt"

    def test_with_state(self):
        s = Schematic.new("c")
        s.set_block((0, 0, 0), "minecraft:repeater", state={"delay": 4, "facing": "east"})
        b = s.get_block(0, 0, 0)
        # Stored state is reflected in the get_block_string form.
        bs = s.get_block_string(0, 0, 0)
        assert bs is not None and "minecraft:repeater" in bs
        assert "delay=4" in bs

    def test_block_object(self):
        s = Schematic.new("d")
        s.set_block((0, 0, 0), Block("minecraft:gold_block"))
        assert s.get_block_string(0, 0, 0) == "minecraft:gold_block"

    def test_chain_returns_self(self):
        s = Schematic.new("e")
        out = (
            s.set_block((0, 0, 0), "minecraft:stone")
            .set_block((1, 0, 0), "minecraft:dirt")
            .set_block((2, 0, 0), "minecraft:grass_block")
        )
        assert out is s


class TestCursor:
    def test_basic_advance(self):
        s = Schematic.new("c")
        cur = s.cursor(origin=(0, 0, 0), step=(3, 0, 0))
        cur.place("minecraft:stone").advance()
        assert cur.pos == (3, 0, 0)
        cur.place("minecraft:dirt")
        assert s.get_block_string(0, 0, 0) == "minecraft:stone"
        assert s.get_block_string(3, 0, 0) == "minecraft:dirt"

    def test_offset(self):
        s = Schematic.new("c2")
        cur = s.cursor(origin=(5, 0, 0))
        cur.place("minecraft:emerald_block", offset=(0, 1, 0))
        assert s.get_block_string(5, 1, 0) == "minecraft:emerald_block"

    def test_reset(self):
        s = Schematic.new("c3")
        cur = s.cursor(step=(2, 0, 0)).advance(3)
        assert cur.pos == (6, 0, 0)
        cur.reset()
        assert cur.pos == (0, 0, 0)


class TestSaveAndFormatInference:
    def test_save_litematic(self, tmp_path: Path):
        s = Schematic.new("save-lit").set_block((0, 0, 0), "minecraft:stone")
        out = tmp_path / "x.litematic"
        s.save(out)
        assert out.exists() and out.stat().st_size > 0

    def test_save_schem(self, tmp_path: Path):
        s = Schematic.new("save-sch").set_block((0, 0, 0), "minecraft:stone")
        out = tmp_path / "x.schem"
        s.save(out)
        assert out.exists() and out.stat().st_size > 0

    def test_save_unknown_extension_requires_format(self, tmp_path: Path):
        s = Schematic.new("save-x").set_block((0, 0, 0), "minecraft:stone")
        with pytest.raises(ValueError):
            s.save(tmp_path / "x.unknown")

    def test_save_explicit_format_overrides(self, tmp_path: Path):
        s = Schematic.new("save-y").set_block((0, 0, 0), "minecraft:stone")
        out = tmp_path / "no-ext"
        s.save(out, format="litematic")
        assert out.exists() and out.stat().st_size > 0


class TestContextManager:
    def test_with_block(self, tmp_path: Path):
        path = tmp_path / "ctx.schem"
        Schematic.new("ctx").set_block((0, 0, 0), "minecraft:stone").save(path)
        with Schematic.open(path) as s:
            assert s.get_block_string(0, 0, 0) == "minecraft:stone"


class TestCopy:
    def test_copy_is_independent(self):
        a = Schematic.new("orig").set_block((0, 0, 0), "minecraft:stone")
        b = a.copy()
        b.set_block((0, 0, 0), "minecraft:dirt")
        assert a.get_block_string(0, 0, 0) == "minecraft:stone"
        assert b.get_block_string(0, 0, 0) == "minecraft:dirt"


# ------------------------------------------------------------- Template path


class TestFromTemplate:
    def test_basic_template(self):
        template = "ab\ncd"
        s = Schematic.from_template(template, name="t")
        s.map("a", "minecraft:stone").map("b", "minecraft:dirt")
        s.map("c", "minecraft:gold_block").map("d", "minecraft:diamond_block")
        # Materializes lazily on next non-template call.
        assert s.raw is not None

    def test_template_with_state_kwarg(self):
        template = "l"
        s = Schematic.from_template(template, name="ts")
        # Should not blow up — state encoded into the mapped string.
        s.map("l", "minecraft:lever", state={"face": "floor", "facing": "north"})
        assert s.raw is not None


# ------------------------------------------------------------- Backcompat


class TestBackcompatLegacyMethods:
    def test_legacy_set_block_with_properties_still_works(self):
        s = Schematic.new("legacy")
        s.set_block_with_properties(
            0, 0, 0, "minecraft:repeater", {"delay": "4", "facing": "east"}
        )
        bs = s.get_block_string(0, 0, 0)
        assert bs is not None and "minecraft:repeater" in bs

    def test_legacy_fill_cuboid_still_works(self):
        s = Schematic.new("fc")
        s.fill_cuboid(0, 0, 0, 2, 0, 0, "minecraft:stone")
        for x in range(3):
            assert s.get_block_string(x, 0, 0) == "minecraft:stone"

    def test_legacy_to_litematic_via_passthrough(self):
        s = Schematic.new("p").set_block((0, 0, 0), "minecraft:stone")
        data = s.to_litematic()
        assert isinstance(data, (bytes, bytearray)) and len(data) > 0


class TestBackcompatSchematicBuilder:
    def test_emits_deprecation_warning(self):
        with warnings.catch_warnings(record=True) as caught:
            warnings.simplefilter("always")
            SchematicBuilder()
        assert any(issubclass(c.category, DeprecationWarning) for c in caught)

    def test_old_chain_still_builds(self):
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", DeprecationWarning)
            schem = (
                SchematicBuilder()
                .name("legacy-build")
                .from_template("ab")
                .map("a", "minecraft:stone")
                .map("b", "minecraft:dirt")
                .build()
            )
        assert isinstance(schem, Schematic)


# ------------------------------------------------------------- Simulation


class TestSimulate:
    def test_simulate_no_events_succeeds(self):
        # Even an empty simulation should run without crashing.
        s = Schematic.new("sim")
        s.set_block((0, 0, 0), "minecraft:stone")
        result = s.simulate(ticks=1)
        assert result is s

    def test_simulate_with_useblock_event(self):
        s = Schematic.new("sim2").set_block((0, 0, 0), "minecraft:stone")
        # The event passes through to the underlying world. We don't assert
        # game-state outcomes here — that's covered by the full circuit tests.
        s.simulate(ticks=2, events=[UseBlock((0, 1, 0))])

    def test_create_simulation_world_still_exposed(self):
        s = Schematic.new("sim3").set_block((0, 0, 0), "minecraft:stone")
        # Power-user path still works.
        world = s.create_simulation_world()
        world.tick(1)


# --------------------------------------------------------------- Re-exports


class TestReExports:
    def test_native_classes_visible(self):
        assert nucleation.BlockState is BlockState
        assert hasattr(nucleation, "RenderConfig")
        assert hasattr(nucleation, "MeshConfig")
        assert hasattr(nucleation, "ResourcePack")
        assert hasattr(nucleation, "MchprsWorld")

    def test_native_module_accessible(self):
        assert hasattr(nucleation, "_native")
        assert hasattr(nucleation._native, "Schematic")


# ----------------------------------------------------- Minecraft helpers


class TestText:
    def test_plain(self):
        assert text("Hello") == '{"text":"Hello"}'

    def test_with_formatting(self):
        s = text("Warn", color="red", bold=True)
        assert '"color":"red"' in s
        assert '"bold":true' in s
        assert '"text":"Warn"' in s

    def test_quotes_escaped(self):
        s = text('He said "hi"')
        # JSON encoding handles escaping correctly.
        assert '\\"' in s


class TestChest:
    def test_list_of_tuples(self):
        s = Schematic.new("ch1")
        s.set_block((0, 0, 0), "minecraft:chest",
                    state={"facing": "west"},
                    nbt=chest([("minecraft:diamond", 64), "minecraft:elytra"]))
        be = s.get_block_entity(0, 0, 0)
        items = sorted(be["nbt"]["Items"], key=lambda x: x["Slot"])
        assert items[0]["Slot"] == 0
        assert items[0]["id"] == "minecraft:diamond"
        assert items[1]["Slot"] == 1
        assert items[1]["id"] == "minecraft:elytra"

    def test_dict_with_gaps(self):
        s = Schematic.new("ch2")
        s.set_block((0, 0, 0), "minecraft:chest", state={"facing": "east"},
                    nbt=chest({0: ("minecraft:diamond", 64), 13: "minecraft:elytra"}))
        slots = sorted(it["Slot"] for it in s.get_block_entity(0, 0, 0)["nbt"]["Items"])
        assert slots == [0, 13]

    def test_item_dataclass(self):
        s = Schematic.new("ch3")
        s.set_block((0, 0, 0), "minecraft:chest", state={"facing": "north"},
                    nbt=chest([Item("minecraft:netherite_ingot", count=3)]))
        items = s.get_block_entity(0, 0, 0)["nbt"]["Items"]
        assert items[0]["id"] == "minecraft:netherite_ingot"
        # Count comes back lowercase 'count' from the modern parser; either is OK.
        cnt = items[0].get("Count") or items[0].get("count")
        assert cnt == 3

    def test_custom_name_wraps_plain_string_as_json(self):
        s = Schematic.new("ch4")
        s.set_block((0, 0, 0), "minecraft:chest", state={"facing": "south"},
                    nbt=chest([("minecraft:diamond", 1)], name="Loot Stash"))
        cn = s.get_block_entity(0, 0, 0)["nbt"]["CustomName"]
        assert "Loot Stash" in cn and "{" in cn  # JSON text component

    def test_block_instance_reuse(self):
        loot = Block("minecraft:chest",
                     state={"facing": "west"},
                     nbt=chest([("minecraft:diamond", 64)]))
        s = Schematic.new("ch5")
        for x in (0, 5, 10):
            s.set_block((x, 0, 0), loot)
        for x in (0, 5, 10):
            be = s.get_block_entity(x, 0, 0)
            assert be["nbt"]["Items"][0]["id"] == "minecraft:diamond"


class TestSign:
    def test_basic(self):
        s = Schematic.new("sg1")
        s.set_block((0, 0, 0), "minecraft:oak_sign", state={"rotation": 8},
                    nbt=sign(["Welcome", "to the", "Loot Room"]))
        be = s.get_block_entity(0, 0, 0)
        front = be["nbt"]["front_text"]
        # Stored as raw SNBT-ish string by the schematic parser; check substring.
        assert "Welcome" in str(front)
        assert "Loot Room" in str(front)

    def test_with_text_component(self):
        s = Schematic.new("sg2")
        s.set_block((0, 0, 0), "minecraft:oak_sign", state={"rotation": 0},
                    nbt=sign([text("LOOT", color="gold", bold=True), "Inside"]))
        front = str(s.get_block_entity(0, 0, 0)["nbt"]["front_text"])
        assert "gold" in front
        assert "bold" in front

    def test_waxed_glowing(self):
        s = Schematic.new("sg3")
        s.set_block((0, 0, 0), "minecraft:oak_sign", state={"rotation": 0},
                    nbt=sign(["Hi"], waxed=True, glowing=True))
        nbt = s.get_block_entity(0, 0, 0)["nbt"]
        # Stored as 1 / "1b" depending on writer; both indicate true.
        assert str(nbt.get("is_waxed", "")) in ("1", "True", "1b", "True")

    def test_at_most_four_lines(self):
        with pytest.raises(ValueError):
            sign(["a", "b", "c", "d", "e"])

    def test_empty_lines_padded(self):
        n = sign(["one"])
        assert len(n["front_text"]["messages"]) == 4
        assert n["back_text"]["messages"] == ['""', '""', '""', '""']
