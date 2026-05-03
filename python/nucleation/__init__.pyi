"""Type stubs for the polished Nucleation API.

The compiled extension types are stubbed in ``_native.pyi`` and re-exported
here for backwards compatibility.
"""

from pathlib import Path
from typing import (
    Any,
    Iterable,
    Mapping,
    Optional,
    Tuple,
    Union,
    overload,
)

from . import _native as _native

# ----- Re-exported native classes -----------------------------------------

BlockState = _native.BlockState
DefinitionRegion = _native.DefinitionRegion
BuildingTool = _native.BuildingTool
Shape = _native.Shape
Brush = _native.Brush

ResourcePack = _native.ResourcePack
MeshConfig = _native.MeshConfig
MeshResult = _native.MeshResult
MultiMeshResult = _native.MultiMeshResult
ChunkMeshResult = _native.ChunkMeshResult
RawMeshExport = _native.RawMeshExport
TextureAtlas = _native.TextureAtlas
ItemModelConfig = _native.ItemModelConfig
ItemModelResult = _native.ItemModelResult
RenderConfig = _native.RenderConfig
MchprsWorld = _native.MchprsWorld

# ----- Module helpers ------------------------------------------------------

def debug_schematic(*args: Any, **kwargs: Any) -> Any: ...
def debug_json_schematic(*args: Any, **kwargs: Any) -> Any: ...
def load_schematic(*args: Any, **kwargs: Any) -> Any: ...
def save_schematic(*args: Any, **kwargs: Any) -> Any: ...

# ----- Type aliases --------------------------------------------------------

Coord = Tuple[int, int, int]
StateValue = Union[str, int, bool]
BlockLike = Union[str, "Block"]

# ----- Block ---------------------------------------------------------------

class Block:
    id: str
    state: Optional[Mapping[str, StateValue]]
    nbt: Optional[Mapping[str, Any]]

    def __init__(
        self,
        id: str,
        state: Optional[Mapping[str, StateValue]] = ...,
        nbt: Optional[Mapping[str, Any]] = ...,
    ) -> None: ...
    @classmethod
    def parse(cls, s: str) -> "Block": ...
    def with_state(self, **kwargs: StateValue) -> "Block": ...
    def with_nbt(self, **kwargs: Any) -> "Block": ...
    def to_string(self) -> str: ...

# ----- Minecraft helpers ---------------------------------------------------

def text(
    s: str,
    *,
    color: Optional[str] = ...,
    bold: Optional[bool] = ...,
    italic: Optional[bool] = ...,
    underlined: Optional[bool] = ...,
    strikethrough: Optional[bool] = ...,
    obfuscated: Optional[bool] = ...,
) -> str: ...

class Item:
    id: str
    count: int
    slot: Optional[int]
    components: Optional[Mapping[str, Any]]
    def __init__(
        self,
        id: str,
        count: int = ...,
        slot: Optional[int] = ...,
        components: Optional[Mapping[str, Any]] = ...,
    ) -> None: ...

def chest(
    items: Union[Iterable[Any], Mapping[int, Any]],
    *,
    name: Optional[str] = ...,
    lock: Optional[str] = ...,
    loot_table: Optional[str] = ...,
) -> dict: ...

def sign(
    lines: Optional[Iterable[Any]] = ...,
    *,
    back: Optional[Iterable[Any]] = ...,
    color: str = ...,
    glowing: bool = ...,
    waxed: bool = ...,
) -> dict: ...

# ----- Events --------------------------------------------------------------

class UseBlock:
    pos: Coord
    def __init__(self, pos: Coord) -> None: ...

class ButtonPress:
    pos: Coord
    def __init__(self, pos: Coord) -> None: ...

Event = Union[UseBlock, ButtonPress]

# ----- Cursor --------------------------------------------------------------

class Cursor:
    pos: Coord
    step: Coord
    def place(
        self,
        block: BlockLike,
        *,
        state: Optional[Mapping[str, StateValue]] = ...,
        nbt: Optional[Mapping[str, Any]] = ...,
        offset: Coord = ...,
    ) -> "Cursor": ...
    def advance(self, n: int = ...) -> "Cursor": ...
    def reset(self) -> "Cursor": ...

# ----- Schematic -----------------------------------------------------------

class Schematic:
    pack: Optional[ResourcePack]

    def __init__(
        self,
        name_or_path: str = ...,
        *,
        pack: Optional[ResourcePack] = ...,
    ) -> None: ...
    @classmethod
    def new(
        cls, name: str = ..., *, pack: Optional[ResourcePack] = ...
    ) -> "Schematic": ...
    @classmethod
    def open(
        cls,
        path: Union[str, Path],
        *,
        pack: Optional[ResourcePack] = ...,
    ) -> "Schematic": ...
    @classmethod
    def from_template(
        cls,
        template: str,
        *,
        name: str = ...,
        pack: Optional[ResourcePack] = ...,
    ) -> "Schematic": ...

    @overload
    def set_block(
        self, x: int, y: int, z: int, block: BlockLike
    ) -> "Schematic": ...
    @overload
    def set_block(
        self,
        pos: Coord,
        block: BlockLike,
        *,
        state: Optional[Mapping[str, StateValue]] = ...,
        nbt: Optional[Mapping[str, Any]] = ...,
    ) -> "Schematic": ...
    def map(
        self,
        char: str,
        block: BlockLike,
        *,
        state: Optional[Mapping[str, StateValue]] = ...,
        nbt: Optional[Mapping[str, Any]] = ...,
    ) -> "Schematic": ...
    def fill(
        self,
        region: Tuple[Coord, Coord],
        block: BlockLike,
    ) -> "Schematic": ...
    def cursor(
        self, *, origin: Coord = ..., step: Coord = ...
    ) -> Cursor: ...
    def with_pack(self, pack: ResourcePack) -> "Schematic": ...
    def copy(self) -> "Schematic": ...
    def simulate(
        self,
        *,
        ticks: int = ...,
        events: Optional[Iterable[Event]] = ...,
        reset: bool = ...,
    ) -> "Schematic": ...
    def invalidate_simulation(self) -> "Schematic": ...
    def save(
        self,
        path: Union[str, Path],
        *,
        format: Optional[str] = ...,
    ) -> None: ...
    def render(
        self,
        path: Union[str, Path],
        config: Optional[RenderConfig] = ...,
        *,
        pack: Optional[ResourcePack] = ...,
        width: int = ...,
        height: int = ...,
        yaw: float = ...,
        pitch: float = ...,
        zoom: float = ...,
        fov: float = ...,
    ) -> None: ...
    def render_to_file(
        self,
        pack: ResourcePack,
        path: Union[str, Path],
        config: RenderConfig,
    ) -> None: ...
    def export_mesh(
        self,
        path: Union[str, Path],
        *,
        pack: Optional[ResourcePack] = ...,
        config: Optional[MeshConfig] = ...,
    ) -> None: ...
    def __enter__(self) -> "Schematic": ...
    def __exit__(self, *exc: Any) -> None: ...
    @property
    def raw(self) -> _native.Schematic: ...

    # Forwarded native methods (selection — anything else also forwards via __getattr__).
    @property
    def name(self) -> Optional[str]: ...
    @name.setter
    def name(self, value: str) -> None: ...
    def get_block(self, x: int, y: int, z: int) -> Optional[BlockState]: ...
    def get_block_string(self, x: int, y: int, z: int) -> Optional[str]: ...
    def to_litematic(self) -> bytes: ...
    def to_schematic(self) -> bytes: ...
    def to_mcstructure(self) -> bytes: ...
    def from_litematic(self, data: bytes) -> None: ...
    def from_schematic(self, data: bytes) -> None: ...
    def from_mcstructure(self, data: bytes) -> None: ...
    def from_data(self, data: bytes) -> None: ...
    def create_simulation_world(self) -> MchprsWorld: ...
    def __getattr__(self, name: str) -> Any: ...

# ----- SchematicBuilder (deprecated) --------------------------------------

class SchematicBuilder:
    def __init__(self) -> None: ...
    def name(self, n: str) -> "SchematicBuilder": ...
    def from_template(self, t: str) -> "SchematicBuilder": ...
    def map(self, char: str, block: str) -> "SchematicBuilder": ...
    def layers(self, layers: Any) -> "SchematicBuilder": ...
    def build(self) -> Schematic: ...
