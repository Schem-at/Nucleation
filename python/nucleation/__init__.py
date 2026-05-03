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
        """Serialize back to the ``id[k=v,...]{snbt}`` string form."""
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
            del self._pending_builder
        return self._inner

    def __getattr__(self, name: str) -> Any:
        # Forward unknown attributes to the underlying native instance.
        # Avoids enumerating every legacy method (set_block_with_properties,
        # fill_cuboid, get_palette, format I/O, etc.).
        if name.startswith("_"):
            raise AttributeError(name)
        inner = self.__dict__.get("_inner")
        if inner is None:
            inner = self._ensure_built()
        return getattr(inner, name)

    @property
    def raw(self) -> Any:
        """The underlying ``_native.Schematic`` instance."""
        return self._ensure_built()

    # ------------------------------------------------------------- mutation API

    def set_block(self, *args: Any, **kwargs: Any) -> "Schematic":
        """Place a block.

        Accepted call patterns::

            set_block(x, y, z, "minecraft:stone")             # legacy
            set_block((x, y, z), "minecraft:stone")           # new
            set_block((x, y, z), "minecraft:repeater", state={"delay": 4})
            set_block((x, y, z), Block("minecraft:chest", nbt={...}))
        """
        state = kwargs.pop("state", None)
        nbt = kwargs.pop("nbt", None)
        if kwargs:
            raise TypeError(f"set_block(): unexpected kwargs {list(kwargs)}")

        if len(args) == 4:
            x, y, z, block = args
        elif len(args) == 2:
            pos, block = args
            x, y, z = pos
        else:
            raise TypeError(
                "set_block(x, y, z, block) or set_block((x, y, z), block, *, state, nbt)"
            )

        inner = self._ensure_built()
        if isinstance(block, Block):
            block_struct = block
        elif isinstance(block, str):
            if state is None and nbt is None and ("[" in block or "{" in block):
                block_struct = Block.parse(block)
            else:
                block_struct = Block(id=block, state=state, nbt=nbt)
        else:
            raise TypeError(f"block must be str or Block, got {type(block).__name__}")

        # Merge any extra kwargs over Block's own state/nbt.
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
            inner.set_block_from_string(x, y, z, payload)
        elif eff_state:
            props = {k: _state_to_str(v) for k, v in eff_state.items()}
            inner.set_block_with_properties(x, y, z, block_struct.id, props)
        else:
            inner.set_block(x, y, z, block_struct.id)
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
        inner = self.__dict__.get("_inner")
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
    "Schematic",
    "SchematicBuilder",
    "Shape",
    "UseBlock",
    "_native",
    "debug_json_schematic",
    "debug_schematic",
    "load_schematic",
    "save_schematic",
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
