"""Nucleation — high-performance Minecraft schematic parser.

This is the polished public API. The compiled extension lives at
``nucleation._native`` and remains accessible for power users.

The redesign described in api_upgrade.md is implemented here on top of the
native extension. Backwards compatibility is preserved: every previous call
pattern continues to work, deprecated paths emit a ``DeprecationWarning``.
"""

from __future__ import annotations

import warnings
from dataclasses import dataclass, replace
from functools import cached_property
from pathlib import Path
from typing import Any, Iterable, Mapping, Optional, Tuple, Union

from . import _native

# ---------------------------------------------------------------------------
# Re-exports for backwards compatibility.
# Anything that used to be importable as ``nucleation.X`` still is.
# ---------------------------------------------------------------------------

BlockState = _native.BlockState
DefinitionRegion = _native.DefinitionRegion
BuildingTool = _native.BuildingTool
Shape = _native.Shape
Brush = _native.Brush

# Module-level helper functions
debug_schematic = _native.debug_schematic
debug_json_schematic = _native.debug_json_schematic
load_schematic = _native.load_schematic
save_schematic = _native.save_schematic

# Optional feature classes — re-exported when their feature was compiled in.
for _name in (
    "ResourcePack",
    "MeshConfig",
    "MeshResult",
    "MultiMeshResult",
    "ChunkMeshResult",
    "RawMeshExport",
    "TextureAtlas",
    "ItemModelConfig",
    "ItemModelResult",
    "RenderConfig",
    "MchprsWorld",
    "Value",
    "IoType",
    "LayoutFunction",
    "OutputCondition",
    "ExecutionMode",
    "IoLayoutBuilder",
    "IoLayout",
    "TypedCircuitExecutor",
    "CircuitBuilder",
    "SortStrategy",
):
    if hasattr(_native, _name):
        globals()[_name] = getattr(_native, _name)
del _name


# ---------------------------------------------------------------------------
# Type aliases
# ---------------------------------------------------------------------------

Coord = Tuple[int, int, int]
StateValue = Union[str, int, bool]


# ---------------------------------------------------------------------------
# Block — structured representation of an id + state + nbt
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Block:
    """A Minecraft block identifier with optional state and NBT.

    Use :py:meth:`Block.parse` to parse strings of the form
    ``"minecraft:jukebox[has_record=true]{signal=5}"``. Use
    :py:meth:`with_state` / :py:meth:`with_nbt` to derive variants.
    """

    id: str
    state: Optional[Mapping[str, StateValue]] = None
    nbt: Optional[Mapping[str, Any]] = None

    @classmethod
    def parse(cls, s: str) -> "Block":
        """Parse ``id[k=v,...]{snbt}`` form into a structured Block.

        SNBT inside ``{}`` is preserved as a raw string under the key
        ``"__snbt__"`` so the native layer can re-emit it. Most users should
        prefer the structured kwargs form.
        """
        if not isinstance(s, str):
            raise TypeError(
                f"Block.parse() expects a string, got {type(s).__name__}"
            )
        rest = s.strip()
        snbt: Optional[str] = None
        if rest.endswith("}"):
            depth = 0
            cut = -1
            for i in range(len(rest) - 1, -1, -1):
                ch = rest[i]
                if ch == "}":
                    depth += 1
                elif ch == "{":
                    depth -= 1
                    if depth == 0:
                        cut = i
                        break
            if cut >= 0:
                snbt = rest[cut + 1 : -1]
                rest = rest[:cut]
        state: Optional[dict] = None
        if rest.endswith("]"):
            cut = rest.rfind("[")
            if cut >= 0:
                inner = rest[cut + 1 : -1]
                rest = rest[:cut]
                state = {}
                for kv in inner.split(","):
                    if not kv.strip():
                        continue
                    k, _, v = kv.partition("=")
                    state[k.strip()] = _coerce_state_value(v.strip())
        nbt = {"__snbt__": snbt} if snbt is not None else None
        return cls(id=rest, state=state, nbt=nbt)

    def with_state(self, **kwargs: StateValue) -> "Block":
        merged = dict(self.state or {})
        merged.update(kwargs)
        return replace(self, state=merged)

    def with_nbt(self, **kwargs: Any) -> "Block":
        merged = dict(self.nbt or {})
        merged.update(kwargs)
        return replace(self, nbt=merged)

    def to_string(self) -> str:
        """Serialize back to the ``id[k=v,...]{snbt}`` string form (cached)."""
        return self._payload

    @cached_property
    def _payload(self) -> str:
        """Memoized full ``id[state]{nbt}`` payload — computed once per Block.

        Frozen dataclass means the source data can't change, so it's safe to
        cache. Reusing a Block across many ``set_block`` calls drops the
        SNBT/state serialization to zero on every call after the first.
        """
        out = self.id
        if self.state:
            parts = ",".join(f"{k}={_state_to_str(v)}" for k, v in self.state.items())
            out += f"[{parts}]"
        if self.nbt:
            if "__snbt__" in self.nbt and len(self.nbt) == 1:
                out += "{" + str(self.nbt["__snbt__"]) + "}"
            else:
                out += "{" + _dict_to_snbt(self.nbt) + "}"
        return out

    @cached_property
    def _has_extras(self) -> bool:
        """True if this block has any state or nbt — used for hot-path routing."""
        return bool(self.state) or bool(self.nbt)


def _unpack_coord(method: str, args: Tuple[Any, ...]) -> Coord:
    """Accept ``(x, y, z)`` or a single ``(x, y, z)`` tuple."""
    if len(args) == 1 and isinstance(args[0], tuple) and len(args[0]) == 3:
        x, y, z = args[0]
    elif len(args) == 3:
        x, y, z = args
    else:
        raise TypeError(
            f"{method}() expected (x, y, z) or a single (x, y, z) tuple"
        )
    return int(x), int(y), int(z)


def _coerce_state_value(v: str) -> StateValue:
    if v == "true":
        return True
    if v == "false":
        return False
    if v.isdigit() or (v.startswith("-") and v[1:].isdigit()):
        return int(v)
    return v


def _state_to_str(v: StateValue) -> str:
    if isinstance(v, bool):
        return "true" if v else "false"
    return str(v)


def _dict_to_snbt(d: Mapping[str, Any]) -> str:
    """Minimal Python-dict → SNBT serializer.

    Handles strings, ints, bools, lists, and nested dicts. Sufficient for the
    common chest/sign/etc. cases. Power users can pass a pre-formatted SNBT
    string by setting ``nbt={"__snbt__": "<raw>"}``.
    """
    if "__snbt__" in d and len(d) == 1:
        return str(d["__snbt__"])
    parts = []
    for k, v in d.items():
        parts.append(f"{k}:{_value_to_snbt(v)}")
    return ",".join(parts)


def _value_to_snbt(v: Any) -> str:
    if isinstance(v, bool):
        return "1b" if v else "0b"
    if isinstance(v, int):
        return f"{v}"
    if isinstance(v, float):
        return f"{v}f"
    if isinstance(v, str):
        return '"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"'
    if isinstance(v, list):
        return "[" + ",".join(_value_to_snbt(x) for x in v) + "]"
    if isinstance(v, dict):
        return "{" + _dict_to_snbt(v) + "}"
    raise TypeError(f"Cannot encode {type(v).__name__} as SNBT")


BlockLike = Union[str, Block]


# ---------------------------------------------------------------------------
# Minecraft helpers — chest, sign, text, Item
#
# These produce ready-to-use ``nbt=`` dicts (or, for ``text``, JSON strings).
# They target the modern (1.20+) schemas. For pre-1.20 NBT layouts, build the
# dict yourself or use the ``__snbt__`` escape hatch in ``set_block``.
# ---------------------------------------------------------------------------


def text(
    s: str,
    *,
    color: Optional[str] = None,
    bold: Optional[bool] = None,
    italic: Optional[bool] = None,
    underlined: Optional[bool] = None,
    strikethrough: Optional[bool] = None,
    obfuscated: Optional[bool] = None,
) -> str:
    """Build a Minecraft JSON text-component string.

    >>> text("Hello")
    '{"text":"Hello"}'
    >>> text("Warn", color="red", bold=True)
    '{"text":"Warn","color":"red","bold":true}'
    """
    import json as _json

    payload: dict = {"text": s}
    if color is not None:
        payload["color"] = color
    for k, v in (
        ("bold", bold),
        ("italic", italic),
        ("underlined", underlined),
        ("strikethrough", strikethrough),
        ("obfuscated", obfuscated),
    ):
        if v is not None:
            payload[k] = v
    return _json.dumps(payload, separators=(",", ":"))


@dataclass(frozen=True)
class Item:
    """An item stack for inventories.

    ``components`` covers the 1.20.5+ data-components map (e.g.
    ``{"minecraft:enchantments": {"levels": {"minecraft:sharpness": 5}}}``).
    For pre-1.20.5 ``tag``-style NBT, set the matching key directly.
    """

    id: str
    count: int = 1
    slot: Optional[int] = None
    components: Optional[Mapping[str, Any]] = None


def _coerce_item(x: Any, default_slot: int) -> dict:
    """Normalize chest-input shapes into a single dict ready for SNBT."""
    if isinstance(x, Item):
        d: dict = {"Slot": x.slot if x.slot is not None else default_slot,
                   "id": x.id, "Count": x.count}
        if x.components:
            d["components"] = dict(x.components)
        return d
    if isinstance(x, str):
        return {"Slot": default_slot, "id": x, "Count": 1}
    if isinstance(x, tuple):
        if len(x) == 2:
            iid, count = x
            return {"Slot": default_slot, "id": iid, "Count": int(count)}
        if len(x) == 3:
            iid, count, components = x
            d = {"Slot": default_slot, "id": iid, "Count": int(count)}
            if components:
                d["components"] = dict(components)
            return d
    raise TypeError(
        f"Unsupported chest item shape: {x!r}. "
        "Use Item(...), 'minecraft:foo', or (id, count[, components])."
    )


def chest(
    items: Union[Iterable[Any], Mapping[int, Any]],
    *,
    name: Optional[str] = None,
    lock: Optional[str] = None,
    loot_table: Optional[str] = None,
) -> dict:
    """Build a chest NBT dict suitable for ``set_block(... nbt=chest(...))``.

    Accepted ``items`` shapes:
      - list / iterable of ``Item`` | ``"minecraft:foo"`` | ``("id", count)``
        | ``("id", count, components)`` — slots auto-assigned 0..N
      - dict ``{slot_int: item}`` — explicit slots
    """
    out: list = []
    if isinstance(items, Mapping):
        for slot, x in items.items():
            d = _coerce_item(x, slot)
            d["Slot"] = int(slot)  # explicit slot wins over Item.slot
            out.append(d)
    else:
        for i, x in enumerate(items):
            d = _coerce_item(x, i)
            out.append(d)

    nbt: dict = {"Items": out}
    if name is not None:
        nbt["CustomName"] = text(name) if not name.startswith("{") else name
    if lock is not None:
        nbt["Lock"] = lock
    if loot_table is not None:
        nbt["LootTable"] = loot_table
    return nbt


def _sign_messages(lines: Optional[Iterable[Any]]) -> list:
    msgs: list = []
    for line in (lines or []):
        if line is None or line == "":
            msgs.append('""')
        elif isinstance(line, str):
            # Already a JSON text component? leave it. Otherwise wrap.
            msgs.append(line if line.startswith("{") else text(line))
        else:
            raise TypeError(
                f"Sign line must be str (plain or JSON-component), got {type(line).__name__}"
            )
    while len(msgs) < 4:
        msgs.append('""')
    if len(msgs) > 4:
        raise ValueError("A sign has at most 4 lines per side")
    return msgs


def sign(
    lines: Optional[Iterable[Any]] = None,
    *,
    back: Optional[Iterable[Any]] = None,
    color: str = "black",
    glowing: bool = False,
    waxed: bool = False,
) -> dict:
    """Build a modern (1.20+) sign NBT dict.

    Each line may be a plain string (auto-wrapped via ``text(...)``) or an
    already-built JSON text-component string from ``text(...)``.
    """
    return {
        "front_text": {
            "messages": _sign_messages(lines),
            "color": color,
            "has_glowing_text": glowing,
        },
        "back_text": {
            "messages": _sign_messages(back),
            "color": color,
            "has_glowing_text": glowing,
        },
        "is_waxed": waxed,
    }


# ---------------------------------------------------------------------------
# Simulation events
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class UseBlock:
    """Right-click / use event at a position."""

    pos: Coord


@dataclass(frozen=True)
class ButtonPress:
    """Button-press event at a position. Currently lowered to ``UseBlock``."""

    pos: Coord


@dataclass(frozen=True)
class LeverState:
    """Force a lever to a specific powered state (idempotent — no toggle).

    Use this when you want "lever ON" / "lever OFF" semantics rather than
    the right-click / :class:`UseBlock` toggle.
    """

    pos: Coord
    state: bool = True


Event = Union[UseBlock, ButtonPress, LeverState]


# ---------------------------------------------------------------------------
# Cursor — sequential placement helper
# ---------------------------------------------------------------------------


class Cursor:
    """Move-and-place helper for sequential placement loops.

    Created via :py:meth:`Schematic.cursor`. ``advance(n)`` moves by ``n*step``;
    ``place(...)`` places at the current pos plus an optional offset.
    """

    __slots__ = ("_schem", "pos", "step", "_origin")

    def __init__(self, schem: "Schematic", origin: Coord, step: Coord):
        self._schem = schem
        self._origin = origin
        self.pos: Coord = origin
        self.step: Coord = step

    def place(
        self,
        block: BlockLike,
        *,
        state: Optional[Mapping[str, StateValue]] = None,
        nbt: Optional[Mapping[str, Any]] = None,
        offset: Coord = (0, 0, 0),
    ) -> "Cursor":
        target = (
            self.pos[0] + offset[0],
            self.pos[1] + offset[1],
            self.pos[2] + offset[2],
        )
        self._schem.set_block(target, block, state=state, nbt=nbt)
        return self

    def advance(self, n: int = 1) -> "Cursor":
        self.pos = (
            self.pos[0] + self.step[0] * n,
            self.pos[1] + self.step[1] * n,
            self.pos[2] + self.step[2] * n,
        )
        return self

    def reset(self) -> "Cursor":
        self.pos = self._origin
        return self


# ---------------------------------------------------------------------------
# Schematic — the unified facade
# ---------------------------------------------------------------------------


_LOAD_EXTENSIONS = (".schem", ".litematic", ".nbt", ".schematic", ".mcstructure")


def _load_into(native_schem: Any, path: Union[str, Path]) -> None:
    p = Path(path)
    data = p.read_bytes()
    suffix = p.suffix.lower()
    if suffix == ".litematic":
        native_schem.from_litematic(data)
    elif suffix in (".schem", ".schematic"):
        native_schem.from_schematic(data)
    elif suffix == ".mcstructure":
        native_schem.from_mcstructure(data)
    else:
        native_schem.from_data(data)


def _save_to(native_schem: Any, path: Union[str, Path], fmt: Optional[str]) -> None:
    p = Path(path)
    suffix = (fmt or p.suffix.lstrip(".")).lower()
    if suffix == "litematic":
        data = native_schem.to_litematic()
    elif suffix in ("schem", "schematic"):
        data = native_schem.to_schematic()
    elif suffix == "mcstructure":
        data = native_schem.to_mcstructure()
    else:
        raise ValueError(
            f"Cannot infer save format from {path!r}; pass format='litematic'/'schem'/'mcstructure'"
        )
    p.write_bytes(data)


class Schematic(_native.Schematic):
    """A mutable, chainable Minecraft schematic.

    Subclass of the compiled extension class. Native methods (set_block,
    set_blocks, fill_cuboid, get_block, format I/O, etc.) are inherited
    directly — chaining preserves the polished class. Polished features
    (classmethods, save/render with format inference, simulate, cursor,
    copy, with_pack, ...) are defined here as Python methods.
    """

    __slots__ = (
        "pack",
        # Lazy template-builder state (only set by from_template).
        "_pending_builder",
        "_pending_name",
        # Cached redstone world; built lazily on first simulate() call and
        # reused across calls so the redpiler doesn't constant-fold a
        # mid-simulation snapshot back through the graph each time. Cleared
        # by invalidate_simulation() and recreated automatically when None.
        "_sim_world",
    )

    # ------------------------------------------------------------------ ctor

    def __new__(cls, name_or_path: str = "untitled", **kwargs: Any) -> "Schematic":
        # Native PyO3 #[new] only accepts `name`. Strip kwargs here so they
        # can flow through to __init__.
        return super().__new__(cls, name_or_path)

    def __init__(self, name_or_path: str = "untitled", *, pack: Optional[Any] = None) -> None:
        # Native __new__(name_or_path) already ran. Set polished attrs and
        # detect path-vs-name for backwards-compat constructor.
        self.pack = pack
        self._sim_world = None
        if name_or_path.lower().endswith(_LOAD_EXTENSIONS) and Path(name_or_path).exists():
            _load_into(self, name_or_path)
            self.name = Path(name_or_path).stem

    # ------------------------------------------------------------ classmethod

    @classmethod
    def new(cls, name: str = "untitled", *, pack: Optional[Any] = None) -> "Schematic":
        """Create a blank schematic."""
        if not isinstance(name, str):
            raise TypeError(
                f"Schematic.new() name must be a string, got {type(name).__name__}"
            )
        return cls(name, pack=pack)

    @classmethod
    def open(cls, path: Union[str, Path], *, pack: Optional[Any] = None) -> "Schematic":
        """Load a schematic file. Format is inferred from the extension."""
        s = cls(Path(path).stem, pack=pack)
        _load_into(s, path)
        return s

    @classmethod
    def from_template(
        cls,
        template: str,
        *,
        name: str = "untitled",
        pack: Optional[Any] = None,
    ) -> "Schematic":
        """Build from an ASCII-art template (see ``SchematicBuilder.from_template``).

        Map characters via ``.map(char, block)`` before any mutation. The
        template is materialized lazily on first non-map call.
        """
        s = cls(name, pack=pack)
        s._pending_builder = _native.SchematicBuilder.from_template(template)
        s._pending_builder.name(name)
        s._pending_name = name
        return s

    # --------------------------------------------------------------- internals

    def _ensure_built(self) -> None:
        """Materialize a pending template-builder into self via byte transfer."""
        pending = getattr(self, "_pending_builder", None)
        if pending is not None:
            built = pending.build()  # native Schematic
            self.from_data(bytes(built.to_litematic()))
            del self._pending_builder

    @property
    def raw(self) -> "Schematic":
        """Compatibility shim. ``self`` is already the native class — return it."""
        self._ensure_built()
        return self

    # ------------------------------------------------------------- mutation API

    def map(
        self,
        char: str,
        block: BlockLike,
        *,
        state: Optional[Mapping[str, StateValue]] = None,
        nbt: Optional[Mapping[str, Any]] = None,
    ) -> "Schematic":
        """Map a template character to a block. Chainable."""
        if isinstance(block, Block):
            payload = block.to_string()
        elif state is not None or nbt is not None:
            payload = Block(id=block, state=dict(state or {}), nbt=dict(nbt) if nbt else None).to_string()
        else:
            payload = block

        pending = getattr(self, "_pending_builder", None)
        if pending is not None:
            pending.map(char, payload)
        else:
            # Materialized schematics don't have a meaningful .map(); the
            # user is calling it post-build. We can fall back to issuing
            # set_block calls only if we know the layout — which we don't.
            raise RuntimeError(
                "map() is only valid on Schematic.from_template(); "
                "for placed blocks call set_block()."
            )
        return self

    def set_blocks(
        self,
        positions: Any,
        block: BlockLike,
        *,
        state: Optional[Mapping[str, StateValue]] = None,
        nbt: Optional[Mapping[str, Any]] = None,
    ) -> "Schematic":
        """Place the same block at many positions.

        Accepts ``positions`` as:
          - list of ``(x, y, z)`` tuples
          - flat list / numpy array of ints ``[x0, y0, z0, x1, y1, z1, ...]``

        The flat form is faster for very large batches because PyO3 doesn't
        unwrap each tuple at the boundary. For uniform regions, prefer
        :py:meth:`fill`. For repeated NBT-bearing tile entities (chests,
        signs, ...) the parse-once batch path applies the template to all
        positions in one native call.
        """
        if isinstance(block, Block):
            payload = block.to_string()
        elif state is not None or nbt is not None:
            payload = Block(id=block, state=state, nbt=nbt).to_string()
        else:
            payload = block

        self._ensure_built()

        # Flat-array dispatch: faster boundary crossing.
        if positions is not None and len(positions) > 0:
            first = positions[0] if not hasattr(positions, "dtype") else positions[0]
            if isinstance(first, (int,)) or (
                hasattr(positions, "dtype") and getattr(positions, "ndim", 1) == 1
            ):
                flat = (
                    list(positions)
                    if not isinstance(positions, list)
                    else positions
                )
                super().set_blocks_flat(flat, payload)
                return self

        # Tuple-list path.
        super().set_blocks(list(positions), payload)
        return self

    def fill(self, region: Tuple[Coord, Coord], block: BlockLike) -> "Schematic":
        """Fill a cuboid region (inclusive) with the given block."""
        (x1, y1, z1), (x2, y2, z2) = region
        block_id = block.to_string() if isinstance(block, Block) else block
        self._ensure_built()
        self.fill_cuboid(x1, y1, z1, x2, y2, z2, block_id)
        return self

    # ----------------------------------------------------- simulation queries

    def _get_or_build_sim_world(self) -> Any:
        """Internal: return the cached MchprsWorld, building it on demand."""
        self._ensure_built()
        if self._sim_world is None:
            self._sim_world = self.create_simulation_world()
        return self._sim_world

    def is_lit(self, *args: Any) -> bool:
        """``True`` if the redstone lamp at this position is lit.

        Reads from the cached simulation world — no schematic round-trip.
        """
        x, y, z = _unpack_coord("is_lit", args)
        return self._get_or_build_sim_world().is_lit(x, y, z)

    def is_powered(self, *args: Any) -> bool:
        """``True`` if anything at this position is currently powered.

        Unified power check: returns ``True`` when the position is a lit
        redstone lamp, a powered lever or button, or carries a non-zero
        redstone signal (wire, repeater, comparator, etc.).
        """
        x, y, z = _unpack_coord("is_powered", args)
        w = self._get_or_build_sim_world()
        return (
            w.get_lever_power(x, y, z)
            or w.is_lit(x, y, z)
            or w.get_redstone_power(x, y, z) > 0
        )

    def signal_strength(self, *args: Any) -> int:
        """Redstone signal strength (0–15) at this position."""
        x, y, z = _unpack_coord("signal_strength", args)
        return self._get_or_build_sim_world().get_redstone_power(x, y, z)

    # ---------------------------------------------------------------- get_block

    def get_block(self, *args: Any) -> Optional[BlockState]:
        """Read a block. Accepts ``(x, y, z)`` or a single ``(x, y, z)`` tuple
        for symmetry with ``set_block``."""
        x, y, z = _unpack_coord("get_block", args)
        return _native.Schematic.get_block(self, x, y, z)

    def get_block_string(self, *args: Any) -> Optional[str]:
        """String form of :py:meth:`get_block`. Same coord conventions."""
        x, y, z = _unpack_coord("get_block_string", args)
        return _native.Schematic.get_block_string(self, x, y, z)

    def cursor(
        self, *, origin: Coord = (0, 0, 0), step: Coord = (1, 0, 0)
    ) -> Cursor:
        """Create a placement cursor anchored at ``origin`` stepping by ``step``."""
        self._ensure_built()
        return Cursor(self, origin, step)

    def with_pack(self, pack: Any) -> "Schematic":
        """Bind a resource pack so later ``render()``/``export_mesh()`` need not pass one."""
        self.pack = pack
        return self

    def copy(self) -> "Schematic":
        """Return an independent copy of this schematic."""
        self._ensure_built()
        new = type(self)(self.name or "untitled", pack=self.pack)
        new.from_data(bytes(self.to_litematic()))
        return new

    # --------------------------------------------------------------- simulate

    def simulate(
        self,
        *,
        ticks: int = 1,
        events: Optional[Iterable[Event]] = None,
        reset: bool = False,
        sync: bool = True,
    ) -> "Schematic":
        """Run a redstone simulation for ``ticks`` ticks, then sync results back.

        The underlying ``MchprsWorld`` is built on the first call and reused
        on subsequent calls so the redstone wavefront advances correctly
        across multiple ``simulate()`` invocations.

        Parameters
        ----------
        ticks
            How many redstone ticks to advance. ``0`` with no events is a
            no-op and returns ``self`` untouched.
        events
            Iterable of :class:`UseBlock`, :class:`ButtonPress`, or
            :class:`LeverState` events applied before ticking.
        reset
            If ``True``, drop any cached simulation world and rebuild it
            from the current schematic before running. Equivalent to calling
            :py:meth:`invalidate_simulation` immediately before
            ``simulate()``. Use this after mutating the schematic
            (``set_block``, ``fill``, ...) between calls; otherwise the
            cached world reflects the pre-mutation circuit.
        sync
            If ``True`` (default), copy the simulated state back into the
            schematic via the byte round-trip — slower, but ``save()`` /
            ``render()`` see the updated block states.
            If ``False``, skip the round-trip; query state through the
            polished accessors instead (``is_lit`` / ``is_powered`` /
            ``signal_strength``), which read directly from the cached
            simulation world. Much faster for tight tick loops.

        Notes
        -----
        Rebuilding the world is destructive: the redpiler runs a
        compile-time constant fold that propagates any active signals
        through the graph immediately. That's why we cache the world
        across calls, and why ``reset=True`` should be used deliberately —
        a mid-simulation reset will collapse the wavefront to its steady
        state.
        """
        self._ensure_built()
        # Materialize so we can both validate events and short-circuit
        # cleanly when there's nothing to do.
        event_list = list(events or ())
        if ticks <= 0 and not event_list and not reset:
            # Building an MchprsWorld is destructive: the redpiler runs a
            # compile-time constant fold and `sync_to_schematic` writes the
            # canonicalized state back. With nothing to simulate, that's
            # silent state mutation — return self untouched.
            return self

        if reset:
            self._sim_world = None

        world = self._sim_world
        if world is None:
            world = self.create_simulation_world()
            self._sim_world = world

        for ev in event_list:
            if isinstance(ev, (UseBlock, ButtonPress)):
                x, y, z = ev.pos
                world.on_use_block(x, y, z)
            elif isinstance(ev, LeverState):
                x, y, z = ev.pos
                world.set_lever_power(x, y, z, bool(ev.state))
            else:
                raise TypeError(
                    f"Unsupported event {type(ev).__name__}; "
                    f"use UseBlock, ButtonPress, or LeverState"
                )
        world.tick(ticks)
        # Flush the redpiler's pending state so the polished accessors
        # (is_lit / is_powered / signal_strength) return current values
        # whether or not the user opts into the schematic round-trip.
        world.flush()
        if sync:
            world.sync_to_schematic()
            # Pull the simulated state back into self without consuming the
            # world — keep it alive so the next simulate() call resumes from
            # the live tick state instead of a re-folded snapshot.
            synced = world.get_schematic()
            self.from_data(bytes(synced.to_litematic()))
        # When sync=False the schematic is not updated; query state via the
        # cached world (is_lit / is_powered / signal_strength).
        return self

    def invalidate_simulation(self) -> "Schematic":
        """Drop the cached simulation world.

        Call this after manually mutating the schematic (``set_block``,
        ``fill``, ...) when you intend to call ``simulate()`` again — the
        next call will rebuild a fresh world from the current state.
        """
        self._sim_world = None
        return self

    # ----------------------------------------------------------------- save

    def save(self, path: Union[str, Path], *, format: Optional[str] = None) -> None:
        """Save to a file. Format inferred from extension unless overridden."""
        self._ensure_built()
        _save_to(self, path, format)

    # ----------------------------------------------------------------- render

    def render(
        self,
        path: Union[str, Path],
        config: Optional[Any] = None,
        *,
        pack: Optional[Any] = None,
        **kwargs: Any,
    ) -> None:
        """Render to an image file.

        ``config`` may be a ``RenderConfig`` instance, or you can pass kwargs
        (``width``, ``height``, ``yaw``, ``pitch``, ``zoom``, ``fov``).
        ``pack`` defaults to the pack bound via ``with_pack`` or the constructor.
        """
        self._ensure_built()
        pack = pack or self.pack
        if pack is None:
            raise ValueError(
                "render() requires a ResourcePack — pass pack= or bind via with_pack()"
            )
        if config is None:
            config = _make_render_config(**kwargs)
        elif kwargs:
            raise TypeError("render() takes either config= or kwargs, not both")
        # render_to_file is inherited from native.
        super().render_to_file(pack, str(path), config)

    # ------------------------------------------------------------- export_mesh

    def export_mesh(
        self,
        path: Union[str, Path],
        *,
        pack: Optional[Any] = None,
        config: Optional[Any] = None,
    ) -> None:
        """Generate a mesh and write it to ``path``.

        Output format inferred from extension (``.glb`` or ``.nucm``).
        """
        # Validate extension first so the user gets a precise error
        # before any heavy mesh work or pack-loading.
        p = Path(path)
        suffix = p.suffix.lower()
        if suffix not in (".glb", ".nucm"):
            raise ValueError(
                f"export_mesh: unsupported extension {p.suffix!r}; use .glb or .nucm"
            )

        self._ensure_built()
        pack = pack or self.pack
        if pack is None:
            raise ValueError(
                "export_mesh() requires a ResourcePack — pass pack= or bind via with_pack()"
            )
        # to_mesh is inherited from native.
        result = super().to_mesh(pack, config) if config is not None else super().to_mesh(pack)
        data = result.glb_data() if suffix == ".glb" else result.nucm_data()
        p.write_bytes(bytes(data))

    # ------------------------------------------------------------- ctx manager

    def __enter__(self) -> "Schematic":
        self._ensure_built()
        return self

    def __exit__(self, *exc: Any) -> None:
        # No resources to release; native handle is GC'd with the wrapper.
        pass

    # --------------------------------------------------------------- dunder

    def __repr__(self) -> str:
        if hasattr(self, "_pending_builder"):
            return "<Schematic (template-pending)>"
        try:
            return f"<Schematic name={self.name!r}>"
        except Exception:
            return "<Schematic>"


def _make_render_config(**kwargs: Any) -> Any:
    if not hasattr(_native, "RenderConfig"):
        raise RuntimeError(
            "RenderConfig is not available; the wheel was built without the rendering feature"
        )
    target = kwargs.pop("target", None)
    if target is not None:
        if not (isinstance(target, tuple) and len(target) == 3):
            raise TypeError(
                "render(): target must be an (x, y, z) tuple"
            )
        target = (float(target[0]), float(target[1]), float(target[2]))
    cfg = _native.RenderConfig(
        kwargs.pop("width", 1920),
        kwargs.pop("height", 1080),
        float(kwargs.pop("yaw", 45.0)),
        float(kwargs.pop("pitch", 45.0)),
        float(kwargs.pop("zoom", 1.0)),
        float(kwargs.pop("fov", 45.0)),
        target,
    )
    if kwargs:
        raise TypeError(f"render(): unexpected kwargs {list(kwargs)}")
    return cfg


# ---------------------------------------------------------------------------
# SchematicBuilder — deprecated shim (about 20 lines)
# ---------------------------------------------------------------------------


class SchematicBuilder:
    """Deprecated. Use ``Schematic.from_template`` and chain methods directly."""

    def __init__(self) -> None:
        warnings.warn(
            "SchematicBuilder is deprecated; chain methods on Schematic directly "
            "(see Schematic.from_template, Schematic.new).",
            DeprecationWarning,
            stacklevel=2,
        )
        self._native = _native.SchematicBuilder()

    def name(self, n: str) -> "SchematicBuilder":
        self._native.name(n)
        return self

    def from_template(self, t: str) -> "SchematicBuilder":
        self._native = _native.SchematicBuilder.from_template(t)
        return self

    def map(self, char: str, block: str) -> "SchematicBuilder":
        self._native.map(char, block)
        return self

    def layers(self, layers: Any) -> "SchematicBuilder":
        self._native.layers(layers)
        return self

    def build(self) -> Schematic:
        built = self._native.build()  # _native.Schematic
        s = Schematic(built.name or "untitled")
        s.from_data(bytes(built.to_litematic()))
        return s


__all__ = [
    "Block",
    "BlockLike",
    "BlockState",
    "BuildingTool",
    "Brush",
    "ButtonPress",
    "LeverState",
    "Coord",
    "Cursor",
    "DefinitionRegion",
    "Event",
    "Item",
    "Schematic",
    "SchematicBuilder",
    "Shape",
    "UseBlock",
    "_native",
    "chest",
    "debug_json_schematic",
    "debug_schematic",
    "load_schematic",
    "save_schematic",
    "sign",
    "text",
]

# Add feature-gated names if compiled in.
for _opt in (
    "ResourcePack",
    "MeshConfig",
    "MeshResult",
    "MultiMeshResult",
    "ChunkMeshResult",
    "RawMeshExport",
    "TextureAtlas",
    "ItemModelConfig",
    "ItemModelResult",
    "RenderConfig",
    "MchprsWorld",
    "Value",
    "IoType",
    "LayoutFunction",
    "OutputCondition",
    "ExecutionMode",
    "IoLayoutBuilder",
    "IoLayout",
    "TypedCircuitExecutor",
    "CircuitBuilder",
    "SortStrategy",
):
    if _opt in globals():
        __all__.append(_opt)
del _opt
