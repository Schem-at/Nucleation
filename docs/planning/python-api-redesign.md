# Nucleation Python API redesign

A proposal for cleaning up the Python API surface while staying fully backwards compatible with existing code.

## Goals

- Reduce friction in the most common patterns: setting blocks, building from templates, simulating, rendering.
- Replace stringly-typed blockstates and SNBT with structured Python types.
- Unify the split between `Schematic` (imperative) and `SchematicBuilder` (fluent) into a single chainable type.
- **Break nothing.** Every existing script should keep running unchanged.

## Friction points in the current API

1. **String concatenation for blockstates and NBT.** Building `"minecraft:jukebox[has_record=true]{signal=" + str(s) + "}"` is error-prone, hard to read, and offers no editor support.
2. **Two object types doing similar things.** `Schematic` is mutable and imperative, `SchematicBuilder` is fluent and immutable. New users don't know which to reach for.
3. **Simulation round-trip is awkward.** `create_simulation_world` → `on_use_block` → `tick` → `sync_to_schematic` → `get_schematic` is five calls for one logical operation.
4. **Resource pack is passed redundantly.** Every `render_to_file` and `to_mesh` call takes the pack as an argument, even though it's almost always the same pack.
5. **`RenderConfig` is verbose for one-offs.** Most renders only set 2-3 fields.
6. **`Schematic("test")` is ambiguous.** Is it a filename or a name? Depends on whether the string ends in a known extension.
7. **Loop boilerplate.** `for i in range(0, len(ss))` with manual `i*3` arithmetic is the un-Pythonic version of what most users actually want.

## The redesigned API

### One unified `Schematic` class

`SchematicBuilder` goes away as a separate concept. `Schematic` is mutable, chainable, and every builder method returns `self`.

```python
class Schematic:
    @classmethod
    def new(cls, name: str = "untitled", *, pack: ResourcePack | None = None) -> Schematic: ...

    @classmethod
    def open(cls, path: str | Path, *, pack: ResourcePack | None = None) -> Schematic: ...

    @classmethod
    def from_template(
        cls,
        template: str,
        *,
        name: str = "untitled",
        pack: ResourcePack | None = None,
    ) -> Schematic: ...

    def map(self, char: str, block: BlockLike, *, state=None, nbt=None) -> Self: ...
    def set_block(self, pos: Coord, block: BlockLike, *, state=None, nbt=None) -> Self: ...
    def fill(self, region: tuple[Coord, Coord], block: BlockLike) -> Self: ...
    def cursor(self, *, origin: Coord = (0, 0, 0), step: Coord = (1, 0, 0)) -> Cursor: ...
    def with_pack(self, pack: ResourcePack) -> Self: ...
    def copy(self) -> Schematic: ...

    def simulate(self, *, ticks: int = 1, events: list[Event] | None = None) -> Self: ...

    def save(self, path: str | Path, *, format: str | None = None) -> None: ...
    def render(self, path: str | Path, config: RenderConfig | None = None, **kwargs) -> None: ...
    def export_mesh(self, path: str | Path, *, pack: ResourcePack | None = None) -> None: ...

    def __enter__(self) -> Self: ...
    def __exit__(self, *args) -> None: ...
```

### Structured `Block` type

Replaces stringly-typed blockstates and SNBT.

```python
@dataclass(frozen=True)
class Block:
    id: str
    state: dict[str, str | int | bool] | None = None
    nbt: dict[str, object] | None = None

    @classmethod
    def parse(cls, s: str) -> Block:
        """Parse 'minecraft:jukebox[has_record=true]{signal=5}' format."""

    def with_state(self, **kwargs) -> Block: ...
    def with_nbt(self, **kwargs) -> Block: ...
```

### `Cursor` for sequential placement

Replaces manual coordinate arithmetic in loops.

```python
class Cursor:
    pos: Coord
    step: Coord

    def place(self, block: BlockLike, *, state=None, nbt=None, offset: Coord = (0, 0, 0)) -> Self: ...
    def advance(self, n: int = 1) -> Self: ...
    def reset(self) -> Self: ...
```

### Structured simulation events

```python
@dataclass
class UseBlock:
    pos: Coord

@dataclass
class ButtonPress:
    pos: Coord

Event = UseBlock | ButtonPress
```

## Before and after

### Single block placement

**Before**

```python
from nucleation import Schematic, ResourcePack, RenderConfig
schematic = Schematic("test")
schematic.set_block(0, 0, 0, "minecraft:soul_lantern")
schematic.render_to_file(
    ResourcePack.from_file("pack.zip"),
    "artifacts/test.png",
    RenderConfig(
        width=3840, height=2160,
        yaw=45.0, pitch=45.0, zoom=0.7, fov=45.0,
    ),
)
```

**After**

```python
from nucleation import Schematic, ResourcePack

pack = ResourcePack.from_file("pack.zip")

schem = Schematic.new(name="test", pack=pack)
schem.set_block((0, 0, 0), "minecraft:soul_lantern")
schem.render("artifacts/test.png", width=3840, height=2160, yaw=45, pitch=45, zoom=0.7, fov=45)
```

### `set_block` with state and NBT

**Before**

```python
schem.set_block(5, 2, 3, "minecraft:repeater[delay=4,facing=east,powered=false,locked=false]")
schem.set_block(6, 2, 3, "minecraft:chest[facing=west]{Items:[{Slot:0b,id:\"minecraft:diamond\",Count:64b}]}")
```

**After**

```python
schem.set_block((5, 2, 3), "minecraft:repeater", state={"delay": 4, "facing": "east"})
schem.set_block((6, 2, 3), "minecraft:chest",
    state={"facing": "west"},
    nbt={"Items": [{"Slot": 0, "id": "minecraft:diamond", "Count": 64}]},
)
```

The NBT case is where this wins biggest. Real Python dicts mean syntax errors instead of silent SNBT parse failures, and your editor can autocomplete keys.

### Loops with `Cursor`

**Before**

```python
from nucleation import Schematic
schematic = Schematic("jukeboxes.schem")

for i in range(0, len(ss)):
    schematic.set_block(i*3, 0, 0, "minecraft:jukebox[has_record=true]{signal=" + str(ss[i]-1) + "}")
    schematic.set_block(i*3, 1, 0, wools[ss[i]])

schematic.save("jukeboxes.litematic")
```

**After**

```python
from nucleation import Schematic

with Schematic.open("jukeboxes.schem") as schem:
    cursor = schem.cursor(step=(3, 0, 0))
    for s in ss:
        cursor.place("minecraft:jukebox", state={"has_record": True}, nbt={"signal": s - 1})
        cursor.place(wools[s], offset=(0, 1, 0))
        cursor.advance()

    schem.save("jukeboxes.litematic")
```

Gone: `range(len(...))`, string concatenation, `i*3` arithmetic. Lifecycle is explicit via `with`.

### Template + simulate + render

**Before**

```python
from nucleation import Schematic, SchematicBuilder, ResourcePack, RenderConfig

XOR = ""
with open("xor.txt", "r") as f:
    XOR = f.read()

schematic = (
    SchematicBuilder()
    .name("out")
    .from_template(XOR)
    .map("l", "minecraft:lever[face=floor,powered=false,facing=north]")
    .map("░", "minecraft:redstone_lamp[lit=false]")
    .map("x", "minecraft:target")
    .build()
)

world = schematic.create_simulation_world()
world.on_use_block(0, 1, 3)
world.tick(4)

world.sync_to_schematic()
schem = world.get_schematic()
schem.to_mesh(resource_pack).save("artifacts/out.glb")
schem.save("artifacts/out.schem")
schem.save("artifacts/out.litematic")
schem.render_to_file(
    resource_pack,
    "artifacts/out.png",
    RenderConfig(
        width=3840, height=2160,
        yaw=45.0, pitch=45.0, zoom=0.7, fov=45.0,
    ),
)
```

**After**

```python
from nucleation import Schematic, ResourcePack, UseBlock
from pathlib import Path

pack = ResourcePack.from_file("pack.zip")
template = Path("xor.txt").read_text()

schem = (
    Schematic.from_template(template, name="out", pack=pack)
    .map("l", "minecraft:lever", state={"face": "floor", "facing": "north"})
    .map("░", "minecraft:redstone_lamp")
    .map("x", "minecraft:target")
    .simulate(ticks=4, events=[UseBlock((0, 1, 3))])
)

schem.save("artifacts/out.schem")
schem.save("artifacts/out.litematic")
schem.export_mesh("artifacts/out.glb")
schem.render("artifacts/out.png", width=3840, height=2160, yaw=45, pitch=45, zoom=0.7, fov=45)
```

The `SchematicBuilder` round-trip and the `world` round-trip both collapse. Default blockstates (`powered=false`, `lit=false`) drop out because they're inferred from the registry. Eight lines of plumbing become one.

## Backwards compatibility

Every change is additive or polymorphic. Existing code keeps working without modification.

### `set_block` accepts both signatures

```python
def set_block(self, *args, block=None, state=None, nbt=None) -> Self:
    if len(args) == 4:                  # set_block(0, 0, 0, "minecraft:stone")
        x, y, z, blk = args
        pos = (x, y, z)
    elif len(args) == 2:                # set_block((0, 0, 0), "minecraft:stone")
        pos, blk = args
    elif len(args) == 1 and block is not None:
        pos, blk = args[0], block
    else:
        raise TypeError("set_block(x, y, z, block) or set_block(pos, block)")
    ...
```

### `Schematic(...)` constructor preserved

```python
class Schematic:
    def __init__(self, name_or_path: str = "untitled", *, pack=None):
        if name_or_path.endswith((".schem", ".litematic", ".nbt", ".schematic")):
            self._load(name_or_path)
        else:
            self._init_blank(name=name_or_path)
        self.pack = pack
```

`Schematic("jukeboxes.schem")` still loads. `Schematic("test")` still creates a blank. New code uses `Schematic.new()` and `Schematic.open()` for clarity.

### `SchematicBuilder` becomes a thin shim

```python
class SchematicBuilder:
    """Deprecated: chain methods on Schematic directly."""
    def __init__(self):
        warnings.warn(
            "SchematicBuilder is deprecated; chain methods on Schematic directly.",
            DeprecationWarning, stacklevel=2,
        )
        self._name = "untitled"
        self._template = None
        self._mappings = {}

    def name(self, n): self._name = n; return self
    def from_template(self, t): self._template = t; return self
    def map(self, char, block): self._mappings[char] = block; return self

    def build(self) -> Schematic:
        schem = Schematic.from_template(self._template, name=self._name)
        for char, block in self._mappings.items():
            schem.map(char, block)
        return schem
```

About 20 lines of glue. Old code keeps working.

### `.map()` accepts both old strings and new kwargs

```python
def map(self, char: str, block: BlockLike, *, state=None, nbt=None) -> Self:
    if isinstance(block, str) and ("[" in block or "{" in block):
        block = Block.parse(block)
    elif state or nbt:
        block = Block(block, state=state, nbt=nbt)
    ...
```

### `render_to_file` and `to_mesh` become shims

```python
def render(self, path, config=None, *, pack=None, **kwargs) -> None:
    if config is None:
        config = RenderConfig(**kwargs)
    pack = pack or self.pack
    self._render_impl(path, pack, config)

def render_to_file(self, pack, path, config) -> None:
    """Deprecated: use .render()."""
    warnings.warn("render_to_file is deprecated; use .render()", DeprecationWarning, stacklevel=2)
    self._render_impl(path, pack, config)
```

### Simulation gains a wrapper, keeps the world API

```python
def simulate(self, *, ticks=1, events=None) -> Self:
    world = self.create_simulation_world()
    for event in events or []:
        if isinstance(event, UseBlock):
            world.on_use_block(*event.pos)
    world.tick(ticks)
    world.sync_to_schematic()
    self._replace_blocks_from(world.get_schematic())
    return self
```

`create_simulation_world()`, `tick()`, `sync_to_schematic()`, `get_schematic()` all keep working unchanged. Power users with multi-stage simulation flows aren't affected.

## Rollout plan

| Version | What ships |
|---------|------------|
| **v0.x.0** | New API ships. Old API works without warnings. Docs and examples updated to new style. README has a "what's new" section. |
| **v0.x+1.0** | Old paths emit `DeprecationWarning`. Most users won't see this unless they run with `-W default`. |
| **v1.0.0** | Optionally remove the most awkward old paths (`SchematicBuilder`, `render_to_file`). Or keep them indefinitely — Python libraries often keep deprecated APIs forever (see `os.path` vs `pathlib`). |

A small day-one feature worth considering: a `nucleation.__future_api__` flag (or a `from nucleation.strict import ...` module) that fires deprecation warnings immediately, for early adopters who want strictness in their own codebases.

## Open questions

- **Telemetry on `SchematicBuilder` usage.** If it has zero external users, the deprecation theater is unnecessary — keep the shim quiet. If it's in third-party code, the warnings matter more.
- **Mutable vs. immutable `Schematic`.** Currently mutable (`set_block` modifies in place). The proposal keeps it that way and adds `.copy()` for users who want the immutable workflow. Worth confirming this matches what feels natural for the most common use cases.
- **`simulate()` event surface.** `events=[UseBlock((0,1,3))]` scales to more event types (`ButtonPress`, `LeverToggle`, `RedstonePulse`) without growing the method surface. The trade-off vs. shorter forms like `on_use=(0,1,3)` is verbosity now for extensibility later.