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


Event = Union[UseBlock, ButtonPress]


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


class Schematic:
    """A mutable, chainable Minecraft schematic.

    Replaces the previous split between ``Schematic`` (imperative) and
    ``SchematicBuilder`` (fluent). Every mutating method returns ``self`` for
    chaining. The underlying compiled instance is available as ``.raw`` for
    power users.
    """

    __slots__ = (
        "_inner",
        "pack",
        # Cached bound methods of the compiled extension. Caching at __init__
        # eliminates ~50 ns of attribute lookup on the per-call hot path.
        "_fast_set_block",
        "_fast_set_block_with_properties",
        "_fast_set_block_from_string",
        "_fast_place",
        "_fast_prepare_block",
        "_fast_get_block",
        "_fast_get_blocks",
        # Per-instance name→palette-index cache so repeated set_block calls
        # with the same plain id skip even the FxHashMap palette lookup on
        # the native side. ~30 ns/call saved on the multi-id common case.
        "_name_idx_cache",
        # Lazy template-builder state (only set by from_template).
        "_pending_builder",
        "_pending_name",
    )

    # ------------------------------------------------------------------ ctor

    def __init__(
        self,
        name_or_path: str = "untitled",
        *,
        pack: Optional[Any] = None,
        _native_inner: Optional[Any] = None,
    ) -> None:
        if _native_inner is not None:
            self._inner = _native_inner
        else:
            inner = _native.Schematic(name_or_path)
            looks_like_path = name_or_path.lower().endswith(_LOAD_EXTENSIONS) and Path(
                name_or_path
            ).exists()
            if looks_like_path:
                _load_into(inner, name_or_path)
                inner.name = Path(name_or_path).stem
            self._inner = inner
        self.pack = pack
        # Cache hot-path bound methods so set_block doesn't pay the
        # __dict__/attribute-lookup cost on every call. ~50 ns saved per
        # placement.
        if self._inner is not None:
            self._fast_set_block = self._inner.set_block
            self._fast_set_block_with_properties = self._inner.set_block_with_properties
            self._fast_set_block_from_string = self._inner.set_block_from_string
            self._fast_place = self._inner.place
            self._fast_prepare_block = self._inner.prepare_block
            self._fast_get_block = self._inner.get_block
            self._fast_get_blocks = self._inner.get_blocks
            self._name_idx_cache = {}

    # ------------------------------------------------------------ classmethod

    @classmethod
    def new(cls, name: str = "untitled", *, pack: Optional[Any] = None) -> "Schematic":
        """Create a blank schematic. Unambiguous alternative to ``Schematic("name")``."""
        return cls(_native_inner=_native.Schematic(name), pack=pack)

    @classmethod
    def open(cls, path: Union[str, Path], *, pack: Optional[Any] = None) -> "Schematic":
        """Load a schematic file. Format is inferred from the extension."""
        inner = _native.Schematic(Path(path).stem)
        _load_into(inner, path)
        return cls(_native_inner=inner, pack=pack)

    @classmethod
    def from_template(
        cls,
        template: str,
        *,
        name: str = "untitled",
        pack: Optional[Any] = None,
    ) -> "Schematic":
        """Build from an ASCII-art template (see ``SchematicBuilder.from_template``)."""
        builder = _native.SchematicBuilder.from_template(template)
        builder.name(name)
        # Keep the builder alive on the wrapper so .map() can extend it before
        # the first build. We materialize lazily in _ensure_built().
        wrapper = cls(_native_inner=None, pack=pack)
        wrapper._inner = None  # type: ignore[assignment]
        wrapper._pending_builder = builder  # type: ignore[attr-defined]
        wrapper._pending_name = name  # type: ignore[attr-defined]
        return wrapper

    # --------------------------------------------------------------- internals

    def _ensure_built(self) -> Any:
        """Materialize a pending template-builder into a real native schematic."""
        pending = getattr(self, "_pending_builder", None)
        if pending is not None:
            self._inner = pending.build()
            self._rebind_fast()
            del self._pending_builder
        return self._inner

    def _rebind_fast(self) -> None:
        """Refresh the cached bound methods after `_inner` is replaced."""
        inner = self._inner
        if inner is not None:
            self._fast_set_block = inner.set_block
            self._fast_set_block_with_properties = inner.set_block_with_properties
            self._fast_set_block_from_string = inner.set_block_from_string
            self._fast_place = inner.place
            self._fast_prepare_block = inner.prepare_block
            self._name_idx_cache = {}

    def __getattr__(self, name: str) -> Any:
        # Forward unknown attributes to the underlying native instance.
        # Avoids enumerating every legacy method (set_block_with_properties,
        # fill_cuboid, get_palette, format I/O, etc.).
        if name.startswith("_"):
            raise AttributeError(name)
        # Use object.__getattribute__ since slots leaves no __dict__.
        try:
            inner = object.__getattribute__(self, "_inner")
        except AttributeError:
            inner = None
        if inner is None:
            inner = self._ensure_built()
        return getattr(inner, name)

    @property
    def raw(self) -> Any:
        """The underlying ``_native.Schematic`` instance."""
        return self._ensure_built()

    # ------------------------------------------------------------- mutation API

    def get_block(self, x: int, y: int, z: int) -> Any:
        """Fast block lookup at a position. Cached bound method on the
        underlying native instance — no __getattr__ overhead."""
        if self._inner is None:
            self._ensure_built()
        return self._fast_get_block(x, y, z)

    def get_blocks(self, positions: Any) -> Any:
        """Batch block lookup. Same fast-bound-method shortcut as
        :py:meth:`get_block`."""
        if self._inner is None:
            self._ensure_built()
        return self._fast_get_blocks(list(positions))

    def prepare_block(self, name: str) -> int:
        """Pre-resolve a plain block id to a palette index.

        Use ``prepare_block`` once per unique id in setup, then call
        :py:meth:`place` in the hot loop — it skips the per-call name
        lookup entirely and runs at the absolute Python ceiling for
        per-block placement.

        Example::

            stone_idx = schem.prepare_block("minecraft:stone")
            place = schem.place
            for x, y, z in positions:
                place(x, y, z, stone_idx)

        Throughput on this path is ~8 M/s (vs ~3 M/s for ``set_block``).
        Complex strings with ``[`` or ``{`` are not allowed here — use
        ``set_block`` for those.
        """
        if self._inner is None:
            self._ensure_built()
        return self._inner.prepare_block(name)

    @property
    def place(self):
        """Bound fast-path placement method.

        Pair with :py:meth:`prepare_block`. Cache the property once::

            place = schem.place
            for ... :
                place(x, y, z, idx)

        Internally this is the underlying native ``place`` — no Python
        wrapper between you and the array write.
        """
        if self._inner is None:
            self._ensure_built()
        return self._inner.place

    def set_block(
        self,
        *args: Any,
        state: Optional[Mapping[str, StateValue]] = None,
        nbt: Optional[Mapping[str, Any]] = None,
    ) -> "Schematic":
        """Place a block.

        Accepted call patterns::

            set_block(x, y, z, "minecraft:stone")             # legacy
            set_block((x, y, z), "minecraft:stone")           # new
            set_block((x, y, z), "minecraft:repeater", state={"delay": 4})
            set_block((x, y, z), Block("minecraft:chest", nbt={...}))

        Performance notes
        -----------------
        - Plain string ids take a fast path straight to native.
        - Reused ``Block`` instances cache their full payload, so repeated
          placements skip state/NBT serialization after the first call.
        - For uniform regions, prefer ``fill()`` / ``fill_cuboid`` (orders of
          magnitude faster than per-block calls).
        - For hot loops placing many independent blocks, batch via
          ``set_blocks(positions, "id")`` (20-30M/s) instead of per-call
          (~2M/s, capped by PyO3 boundary cost). If you must use per-call,
          cache the bound method::

              place = schem.raw.set_block
              for x, y, z, id in blocks:
                  place(x, y, z, id)

          This skips the polished wrapper and runs at ~3.5M/s.
        """
        # ----- argument shape (most-common form first) -----
        if len(args) == 2:
            (x, y, z), block = args[0], args[1]
        elif len(args) == 4:
            x, y, z, block = args
        else:
            raise TypeError(
                "set_block(x, y, z, block) or set_block((x, y, z), block, *, state, nbt)"
            )

        # Lazy-materialize a template-pending schematic on first mutation.
        if self._inner is None:
            self._ensure_built()

        # ----- ultra-fast path: plain string id, no kwargs -----
        # Use a per-instance name→palette-index cache so repeated calls
        # with the same id skip even the native palette lookup. Complex
        # strings (with [ or {) bypass the cache and go through the
        # full parser path.
        if state is None and nbt is None and block.__class__ is str:
            cache = self._name_idx_cache
            idx = cache.get(block)
            if idx is None:
                if "[" in block or "{" in block:
                    self._fast_set_block(x, y, z, block)
                    return self
                idx = self._fast_prepare_block(block)
                cache[block] = idx
            self._fast_place(x, y, z, idx)
            return self

        # ----- fast path: reused Block instance, no override kwargs -----
        if state is None and nbt is None and isinstance(block, Block):
            if not block._has_extras:
                self._fast_set_block(x, y, z, block.id)
            elif block.nbt:
                self._fast_set_block_from_string(x, y, z, block._payload)
            else:
                # State only → use the typed properties API.
                props = {k: _state_to_str(v) for k, v in block.state.items()}
                self._fast_set_block_with_properties(x, y, z, block.id, props)
            return self

        # ----- slow path: kwargs may augment a Block or a string -----
        if isinstance(block, Block):
            block_struct = block
        elif isinstance(block, str):
            if state is None and nbt is None and ("[" in block or "{" in block):
                block_struct = Block.parse(block)
            else:
                block_struct = Block(id=block, state=state, nbt=nbt)
        else:
            raise TypeError(f"block must be str or Block, got {type(block).__name__}")

        # If there are no override kwargs and nothing to merge, use the cache.
        if state is None and nbt is None:
            if not block_struct._has_extras:
                self._fast_set_block(x, y, z, block_struct.id)
            elif block_struct.nbt:
                self._fast_set_block_from_string(x, y, z, block_struct._payload)
            else:
                props = {k: _state_to_str(v) for k, v in block_struct.state.items()}
                self._fast_set_block_with_properties(x, y, z, block_struct.id, props)
            return self

        # Merge override kwargs over the Block's own state/nbt.
        eff_state = dict(block_struct.state or {})
        if state:
            eff_state.update(state)
        eff_nbt = dict(block_struct.nbt or {})
        if nbt:
            eff_nbt.update(nbt)

        if eff_nbt:
            snbt = _dict_to_snbt(eff_nbt)
            payload = block_struct.id
            if eff_state:
                parts = ",".join(f"{k}={_state_to_str(v)}" for k, v in eff_state.items())
                payload += f"[{parts}]"
            payload += "{" + snbt + "}"
            self._fast_set_block_from_string(x, y, z, payload)
        elif eff_state:
            props = {k: _state_to_str(v) for k, v in eff_state.items()}
            self._fast_set_block_with_properties(x, y, z, block_struct.id, props)
        else:
            self._fast_set_block(x, y, z, block_struct.id)
        return self

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

        inner = self._ensure_built()

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
                inner.set_blocks_flat(flat, payload)
                return self

        # Tuple-list path.
        inner.set_blocks(list(positions), payload)
        return self

    def fill(self, region: Tuple[Coord, Coord], block: BlockLike) -> "Schematic":
        """Fill a cuboid region (inclusive) with the given block."""
        (x1, y1, z1), (x2, y2, z2) = region
        if isinstance(block, Block):
            block_id = block.to_string()
        else:
            block_id = block
        inner = self._ensure_built()
        inner.fill_cuboid(x1, y1, z1, x2, y2, z2, block_id)
        return self

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
        inner = self._ensure_built()
        new_inner = _native.Schematic(inner.name or "untitled")
        new_inner.from_data(inner.to_litematic())
        return Schematic(_native_inner=new_inner, pack=self.pack)

    # --------------------------------------------------------------- simulate

    def simulate(
        self,
        *,
        ticks: int = 1,
        events: Optional[Iterable[Event]] = None,
    ) -> "Schematic":
        """Run a redstone simulation for ``ticks`` ticks, then sync results back.

        Collapses the create-world / on-use-block / tick / sync round-trip
        into a single call. Power users wanting multi-stage flows should
        keep using ``create_simulation_world()`` directly.
        """
        inner = self._ensure_built()
        world = inner.create_simulation_world()
        for ev in events or ():
            if isinstance(ev, (UseBlock, ButtonPress)):
                x, y, z = ev.pos
                world.on_use_block(x, y, z)
            else:
                raise TypeError(
                    f"Unsupported event {type(ev).__name__}; "
                    "use UseBlock or ButtonPress"
                )
        world.tick(ticks)
        world.sync_to_schematic()
        # The world syncs *into* its own schematic snapshot. Replace ours.
        self._inner = world.into_schematic()
        self._rebind_fast()
        return self

    # ----------------------------------------------------------------- save

    def save(self, path: Union[str, Path], *, format: Optional[str] = None) -> None:
        """Save to a file. Format inferred from extension unless overridden."""
        inner = self._ensure_built()
        _save_to(inner, path, format)

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
        inner = self._ensure_built()
        pack = pack or self.pack
        if pack is None:
            raise ValueError(
                "render() requires a ResourcePack — pass pack= or bind via with_pack()"
            )
        if config is None:
            config = _make_render_config(**kwargs)
        elif kwargs:
            raise TypeError(
                "render() takes either config= or kwargs, not both"
            )
        inner.render_to_file(pack, str(path), config)

    def render_to_file(self, pack: Any, path: Union[str, Path], config: Any) -> None:
        """Deprecated. Use ``render(path, config, pack=pack)``."""
        warnings.warn(
            "render_to_file is deprecated; use .render(path, config, pack=pack)",
            DeprecationWarning,
            stacklevel=2,
        )
        self._ensure_built().render_to_file(pack, str(path), config)

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
        inner = self._ensure_built()
        pack = pack or self.pack
        if pack is None:
            raise ValueError(
                "export_mesh() requires a ResourcePack — pass pack= or bind via with_pack()"
            )
        result = inner.to_mesh(pack, config) if config is not None else inner.to_mesh(pack)
        p = Path(path)
        suffix = p.suffix.lower()
        if suffix == ".glb":
            data = result.glb_data()
        elif suffix == ".nucm":
            data = result.nucm_data()
        else:
            raise ValueError(
                f"export_mesh: unsupported extension {p.suffix!r}; use .glb or .nucm"
            )
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
        try:
            inner = object.__getattribute__(self, "_inner")
        except AttributeError:
            inner = None
        if inner is None:
            return f"<Schematic (template-pending)>"
        try:
            return f"<Schematic name={inner.name!r}>"
        except Exception:
            return "<Schematic>"


def _make_render_config(**kwargs: Any) -> Any:
    if not hasattr(_native, "RenderConfig"):
        raise RuntimeError(
            "RenderConfig is not available; the wheel was built without the rendering feature"
        )
    cfg = _native.RenderConfig(
        kwargs.pop("width", 1920),
        kwargs.pop("height", 1080),
        float(kwargs.pop("yaw", 45.0)),
        float(kwargs.pop("pitch", 45.0)),
        float(kwargs.pop("zoom", 1.0)),
        float(kwargs.pop("fov", 45.0)),
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
        inner = self._native.build()
        return Schematic(_native_inner=inner)


__all__ = [
    "Block",
    "BlockLike",
    "BlockState",
    "BuildingTool",
    "Brush",
    "ButtonPress",
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
