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
    Schematic,
    SchematicBuilder,
    UseBlock,
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
