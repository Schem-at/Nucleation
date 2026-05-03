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

    # --- Bug reproductions: world rebuild leaks redpiler constant-fold ---
    #
    # The polished `simulate()` rebuilds an MchprsWorld on every call (see
    # __init__.py: simulate() -> create_simulation_world). Building the world
    # invokes the redpiler's compile-time constant fold, which propagates any
    # currently-on signal through the graph BEFORE any ticks run. As a result,
    # `simulate(ticks=0)` with no events isn't a no-op: the schematic gets
    # overwritten with the post-fold state.
    #
    # These tests assert the property the API documents ("Run a redstone
    # simulation for ``ticks`` ticks, then sync results back") — namely that
    # zero ticks plus zero events leaves the world unchanged.

    @staticmethod
    def _build_repeater_chain(name: str, length: int) -> Schematic:
        """Lever (off) at x=-1, then `length` repeaters facing west, then a lamp.
        All on a stone base. No initial power: the lever is in `powered=false`."""
        s = Schematic.new(name)
        for x in range(-1, length + 1):
            s.set_block((x, 0, 0), "minecraft:stone")
        s.set_block(
            (-1, 1, 0),
            "minecraft:lever",
            state={"face": "floor", "facing": "east", "powered": False},
        )
        s.set_blocks(
            [(x, 1, 0) for x in range(length)],
            "minecraft:repeater",
            state={"facing": "west", "delay": 1},
        )
        s.set_block((length, 1, 0), "minecraft:redstone_lamp")
        return s

    @staticmethod
    def _powered_pattern(s: Schematic, length: int) -> list[str]:
        return [s.get_block(x, 1, 0).properties.get("powered", "?") for x in range(length)]

    def test_simulate_ticks_zero_no_events_is_noop_after_simulation(self):
        """After running a simulation, calling `simulate(ticks=0)` with no
        events must not mutate any block state. Reproduces the bug where the
        world rebuild applies redpiler constant-fold to every repeater."""
        length = 8
        s = self._build_repeater_chain("rebuild_bug", length)

        # First simulation: toggle the lever and run one tick. After this only
        # the first delay-1 repeater (closest to the lever) should be powered.
        s.simulate(events=[UseBlock((-1, 1, 0))])
        before = self._powered_pattern(s, length)

        # The contended call: zero ticks, zero events.
        s.simulate(ticks=0)
        after = self._powered_pattern(s, length)

        assert before == after, (
            "simulate(ticks=0) altered repeater states.\n"
            f"before: {before}\nafter:  {after}"
        )

    def test_simulate_ticks_zero_idempotent_on_active_circuit(self):
        """Two simulate(ticks=0) calls in a row must produce identical state."""
        length = 4
        s = self._build_repeater_chain("idempotent", length)

        s.simulate(events=[UseBlock((-1, 1, 0))])
        s.simulate(ticks=0)
        first = self._powered_pattern(s, length)
        s.simulate(ticks=0)
        second = self._powered_pattern(s, length)

        assert first == second, (
            "simulate(ticks=0) is not idempotent.\n"
            f"first:  {first}\nsecond: {second}"
        )

    def test_simulate_one_tick_only_advances_first_delay_stage(self):
        """After flipping the lever and running exactly one tick, only the
        first repeater (delay=1, closest to the lever) should be powered.
        The whole chain lighting up indicates the redpiler folded the
        steady-state signal at compile time."""
        length = 4
        s = self._build_repeater_chain("one_tick", length)

        s.simulate(events=[UseBlock((-1, 1, 0))])
        powered = [p == "true" for p in self._powered_pattern(s, length)]

        # Strict assertion: only the first repeater on at tick 1.
        assert powered == [True, False, False, False], (
            f"expected only stage 0 powered; got {powered}"
        )

    def test_simulate_advances_signal_one_stage_per_tick(self):
        """Calling simulate(ticks=1) repeatedly should advance the redstone
        wavefront one delay-1 repeater per call. If the world rebuild folds
        constants, the second call lights the entire chain at once."""
        length = 4
        s = self._build_repeater_chain("advance", length)

        # Toggle the lever and tick exactly once at a time.
        s.simulate(events=[UseBlock((-1, 1, 0))], ticks=1)
        progression = [self._powered_pattern(s, length)]
        for _ in range(length - 1):
            s.simulate(ticks=1)
            progression.append(self._powered_pattern(s, length))

        expected = [
            ["true",  "false", "false", "false"],
            ["true",  "true",  "false", "false"],
            ["true",  "true",  "true",  "false"],
            ["true",  "true",  "true",  "true"],
        ]
        assert progression == expected, (
            "redstone wavefront did not advance one repeater per tick.\n"
            f"got:\n  " + "\n  ".join(repr(p) for p in progression) +
            "\nexpected:\n  " + "\n  ".join(repr(p) for p in expected)
        )

    def test_simulate_reset_rebuilds_world_after_mutation(self):
        """``simulate(reset=True)`` must drop the cached world and rebuild
        from the current schematic, so post-``set_block`` mutations are
        respected on the next call."""
        length = 4
        s = self._build_repeater_chain("reset_kwarg", length)
        s.simulate(events=[UseBlock((-1, 1, 0))])  # primes the cached world

        # Mutate the layout. The cached world doesn't know about this yet.
        s.set_block((0, 1, 0), "minecraft:stone")
        assert s._sim_world is not None  # cache still alive

        # reset=True drops the cache and rebuilds from current self.
        s.simulate(reset=True, ticks=0, events=[])
        assert s._sim_world is not None  # rebuilt
        # The rebuild reads stone at (0, 1, 0) and the round-trip preserves
        # it; without reset, sync_to_schematic would overwrite the stone
        # with the cached world's repeater.
        assert s.get_block(0, 1, 0).name == "minecraft:stone", (
            f"reset=True should respect the set_block stone; got "
            f"{s.get_block(0, 1, 0).name}"
        )

    def test_simulate_without_reset_does_not_see_mutation(self):
        """Documents the inverse: without ``reset=True``, ``simulate()``
        keeps using the cached world and overwrites the user's mutation
        on sync_to_schematic. Users are expected to opt into
        ``reset=True`` (or call :py:meth:`invalidate_simulation`) when
        they've mutated the schematic."""
        length = 4
        s = self._build_repeater_chain("no_reset", length)
        s.simulate(events=[UseBlock((-1, 1, 0))])

        s.set_block((0, 1, 0), "minecraft:stone")
        s.simulate(ticks=1)  # NO reset — uses cached world
        assert s.get_block(0, 1, 0).name == "minecraft:repeater", (
            "stale cache should overwrite mutation; got "
            f"{s.get_block(0, 1, 0).name}"
        )

    def test_invalidate_simulation_returns_self_and_drops_world(self):
        s = Schematic.new("inv").set_block((0, 0, 0), "minecraft:stone")
        s.simulate(ticks=1)
        assert s._sim_world is not None
        assert s.invalidate_simulation() is s
        assert s._sim_world is None

    def test_simulate_zero_ticks_zero_events_on_inert_schematic(self):
        """A schematic with no active power source must not gain power from
        a simulate() round-trip."""
        s = Schematic.new("inert")
        s.set_block((0, 0, 0), "minecraft:stone")
        s.set_block((0, 1, 0), "minecraft:redstone_wire")
        before = s.get_block_string(0, 1, 0)

        s.simulate(ticks=0)
        after = s.get_block_string(0, 1, 0)

        assert before == after, (
            f"inert wire mutated by simulate(ticks=0): {before!r} -> {after!r}"
        )


# --------------------------------------------------------------- Re-exports


class TestErrorMessages:
    """The polished API should raise clear, actionable errors. These tests
    pin the message shape so refactors don't degrade DX."""

    def test_set_block_too_few_args(self):
        s = Schematic.new("err")
        with pytest.raises(TypeError, match="2 or 4 positional"):
            s.set_block(1)
        with pytest.raises(TypeError, match="2 or 4 positional"):
            s.set_block(1, 2, 3)

    def test_set_block_too_many_args(self):
        s = Schematic.new("err")
        with pytest.raises(TypeError, match="2 or 4 positional"):
            s.set_block(1, 2, 3, "minecraft:stone", 99)

    def test_set_block_two_arg_first_must_be_tuple(self):
        s = Schematic.new("err")
        with pytest.raises(TypeError, match="3-tuple of ints"):
            s.set_block(1, "minecraft:stone")  # 2 args, but first isn't a tuple

    def test_set_block_2tuple_pos_rejected(self):
        s = Schematic.new("err")
        with pytest.raises((TypeError, ValueError)):
            s.set_block((1, 2), "minecraft:stone")  # only 2 coords

    def test_set_block_block_must_be_str_or_block(self):
        s = Schematic.new("err")
        with pytest.raises(TypeError, match="block must be"):
            s.set_block(1, 2, 3, 42)

    def test_set_block_state_must_be_dict(self):
        s = Schematic.new("err")
        with pytest.raises(TypeError, match="state must be a dict"):
            s.set_block(0, 0, 0, "minecraft:stone", state=42)

    def test_set_block_nbt_must_be_dict(self):
        # Strings used to be silently accepted, hiding typos. Now rejected.
        s = Schematic.new("err")
        with pytest.raises(TypeError, match="nbt= must be a dict"):
            s.set_block((0, 0, 0), "minecraft:chest", nbt="raw_string")

    def test_set_block_nbt_dict_works(self):
        # Sanity: the corrected path still accepts a real dict.
        s = Schematic.new("ok")
        s.set_block((0, 0, 0), "minecraft:chest",
                    nbt={"Items": [{"Slot": 0, "id": "minecraft:diamond", "Count": 64}]})

    def test_set_block_nbt_snbt_escape_works(self):
        # __snbt__ escape hatch still allowed — it's a dict, not a string.
        s = Schematic.new("ok")
        s.set_block((0, 0, 0), "minecraft:jukebox",
                    nbt={"__snbt__": "signal:5"})

    def test_save_unknown_format_message(self):
        s = Schematic.new("err").set_block(0, 0, 0, "minecraft:stone")
        with pytest.raises(ValueError, match="Cannot infer save format"):
            s.save("/tmp/no_such_ext.unknown")

    def test_render_without_pack_clear_error(self):
        s = Schematic.new("err").set_block(0, 0, 0, "minecraft:stone")
        with pytest.raises(ValueError, match=r"render\(\) requires a ResourcePack"):
            s.render("/tmp/x.png")

    def test_export_mesh_unknown_extension_first(self):
        # Should report bad extension BEFORE complaining about missing pack.
        s = Schematic.new("err").set_block(0, 0, 0, "minecraft:stone")
        with pytest.raises(ValueError, match="unsupported extension"):
            s.export_mesh("/tmp/x.bad")

    def test_export_mesh_missing_pack_when_extension_ok(self):
        s = Schematic.new("err").set_block(0, 0, 0, "minecraft:stone")
        with pytest.raises(ValueError, match=r"export_mesh\(\) requires a ResourcePack"):
            s.export_mesh("/tmp/x.glb")

    def test_schematic_new_non_str_name(self):
        with pytest.raises(TypeError, match="name must be a string"):
            Schematic.new(123)

    def test_block_parse_non_str(self):
        with pytest.raises(TypeError, match="expects a string"):
            Block.parse(123)

    def test_sign_too_many_lines(self):
        with pytest.raises(ValueError, match="at most 4 lines"):
            sign(["a", "b", "c", "d", "e"])

    def test_sign_non_string_line_rejected(self):
        with pytest.raises(TypeError, match="Sign line must be"):
            sign([1, 2, 3])

    def test_chained_error_does_not_corrupt_state(self):
        # An error mid-chain should not leave the schematic in a weird state.
        s = Schematic.new("err")
        s.set_block(0, 0, 0, "minecraft:stone")
        with pytest.raises(TypeError):
            s.set_block(1, 2, 3, 42)  # bad block type
        # Schematic should still work after the error.
        s.set_block(0, 1, 0, "minecraft:dirt")
        assert s.get_block_string(0, 0, 0) == "minecraft:stone"
        assert s.get_block_string(0, 1, 0) == "minecraft:dirt"


class TestBatchPerf:
    """The set_blocks parse-once batch path must produce identical results to
    per-call set_block_from_string for complex blocks (state + NBT)."""

    def test_chest_batch_preserves_nbt(self):
        s = Schematic.new("batch")
        positions = [(i, 0, 0) for i in range(50)]
        chest_str = (
            'minecraft:chest[facing=west]'
            '{Items:[{Slot:0b,id:"minecraft:diamond",Count:64b}]}'
        )
        # Polished set_blocks is chainable; raw count is on s.raw.
        s.set_blocks(positions, chest_str)
        # Spot-check: middle and last chest got both state and NBT.
        for x in (0, 25, 49):
            be = s.get_block_entity(x, 0, 0)
            assert be is not None, f"missing block entity at x={x}"
            assert "Items" in be["nbt"], f"missing Items at x={x}"
            assert be["nbt"]["Items"][0]["id"] == "minecraft:diamond"

    def test_state_only_batch(self):
        s = Schematic.new("batch-state")
        positions = [(i, 0, 0) for i in range(20)]
        s.set_blocks(positions, "minecraft:repeater[delay=4,facing=east]")
        for x in (0, 10, 19):
            bs = s.get_block_string(x, 0, 0)
            assert "minecraft:repeater" in bs
            assert "delay=4" in bs

    def test_flat_positions_path(self):
        # Flat list of ints (numpy-friendly) takes the faster set_blocks_flat
        # path internally and produces the same result.
        s = Schematic.new("flat")
        flat = []
        for i in range(20):
            flat.extend([i, 0, 0])
        s.set_blocks(flat, "minecraft:stone")
        for x in (0, 10, 19):
            assert s.get_block_string(x, 0, 0) == "minecraft:stone"


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
