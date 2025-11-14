# Nucleation - Python Documentation

Complete API reference and guide for using Nucleation in Python.

## Quick Start

```bash
pip install nucleation
```

```python
from nucleation import Schematic

# Create schematic
schematic = Schematic("my_schematic")
schematic.set_block(0, 0, 0, "minecraft:stone")

# Save as litematic
with open("output.litematic", "wb") as f:
    f.write(schematic.to_litematic())
```

## Table of Contents

1. [Installation](#installation)
2. [Core API](#core-api)
3. [Loading and Saving](#loading-and-saving)
4. [Block Operations](#block-operations)
5. [Region Operations](#region-operations)
6. [Block Entities](#block-entities)
7. [SchematicBuilder](#schematicbuilder)
8. [Simulation](#simulation)
9. [TypedCircuitExecutor](#typedcircuitexecutor)

## Installation

```bash
pip install nucleation
```

### From Source

```bash
git clone https://github.com/Schem-at/Nucleation
cd Nucleation
maturin develop --features python
```

## Core API

### Schematic

Main class for working with schematics.

```python
class Schematic:
    def __init__(self, name: str):
        """Create empty schematic"""
        
    # Loading/Saving
    def load_from_bytes(self, data: bytes) -> None:
        """Load from bytes (auto-detects format)"""
    def load_from_litematic(self, data: bytes) -> None:
        """Load from litematic format"""
    def load_from_schematic(self, data: bytes) -> None:
        """Load from WorldEdit schematic format"""
    def to_litematic(self) -> bytes:
        """Save as litematic format"""
    def to_schematic(self) -> bytes:
        """Save as WorldEdit schematic format"""
    
    # Block operations
    def set_block(self, x: int, y: int, z: int, block_name: str) -> None:
        """Set block at position"""
    def set_block_with_properties(self, x: int, y: int, z: int, 
                                  block_name: str, properties: dict) -> None:
        """Set block with properties"""
    def set_block_from_string(self, x: int, y: int, z: int, 
                             block_string: str) -> None:
        """Set block from bracket notation string"""
    def get_block(self, x: int, y: int, z: int) -> str | None:
        """Get block name at position"""
    def get_block_with_properties(self, x: int, y: int, z: int) -> dict | None:
        """Get block with properties"""
    
    # Block entities
    def get_block_entity(self, x: int, y: int, z: int) -> dict | None:
        """Get block entity at position"""
    def get_all_block_entities(self) -> list[dict]:
        """Get all block entities"""
    
    # Region operations
    def copy_region(self, source_region: str, 
                   min_x: int, min_y: int, min_z: int,
                   max_x: int, max_y: int, max_z: int,
                   target_x: int, target_y: int, target_z: int,
                   excluded_blocks: list[str]) -> None:
        """Copy region to new position"""
    
    # Information
    def get_dimensions(self) -> tuple[int, int, int]:
        """Get schematic dimensions (width, height, depth)"""
    def get_block_count(self) -> int:
        """Get number of non-air blocks"""
    def get_volume(self) -> int:
        """Get total volume"""
    def get_region_names(self) -> list[str]:
        """Get all region names"""
    def get_info(self) -> str:
        """Get debug information"""
    def print_schematic(self) -> str:
        """Get ASCII representation"""
    
    # Iteration
    def blocks(self) -> list[dict]:
        """Get all blocks with positions and properties"""
    def chunks(self, width: int, height: int, length: int) -> list[dict]:
        """Get chunks (bottom-up order)"""
    def chunks_with_strategy(self, width: int, height: int, length: int,
                            strategy: str, cx: int = 0, cy: int = 0, cz: int = 0) -> list[dict]:
        """Get chunks with ordering strategy"""
    def get_chunk_blocks(self, offset_x: int, offset_y: int, offset_z: int,
                        width: int, height: int, length: int) -> list[dict]:
        """Get blocks in specific chunk"""
    
    # Simulation (requires simulation feature)
    def create_simulation_world(self) -> SimulationWorld:
        """Create simulation world"""
    def create_simulation_world_with_options(self, options: SimulationOptions) -> SimulationWorld:
        """Create simulation world with custom options"""
```

### BlockState

Represents a block with properties.

```python
class BlockState:
    def __init__(self, name: str):
        """Create block state"""
    
    def with_property(self, key: str, value: str) -> None:
        """Add property (mutates in place)"""
    
    @property
    def name(self) -> str:
        """Get block name"""
    
    @property
    def properties(self) -> dict[str, str]:
        """Get all properties"""
```

## Loading and Saving

### Load from File

```python
from nucleation import Schematic

# Load litematic
with open("input.litematic", "rb") as f:
    data = f.read()

schematic = Schematic("loaded")
schematic.load_from_litematic(data)

# Auto-detect format
schematic.load_from_bytes(data)

print(schematic.get_dimensions())
```

### Save to File

```python
# Save as litematic
with open("output.litematic", "wb") as f:
    f.write(schematic.to_litematic())

# Save as WorldEdit schematic
with open("output.schem", "wb") as f:
    f.write(schematic.to_schematic())
```

## Block Operations

### Setting Blocks

```python
# Simple block
schematic.set_block(0, 0, 0, "minecraft:stone")

# Block with properties (dict)
schematic.set_block_with_properties(0, 1, 0, "minecraft:lever", {
    "facing": "east",
    "powered": "false"
})

# Block from string (bracket notation)
schematic.set_block_from_string(
    1, 1, 0,
    "minecraft:redstone_wire[power=15,north=side,south=side]"
)

# Using BlockState
from nucleation import BlockState

block = BlockState("minecraft:repeater")
block.with_property("facing", "east")
block.with_property("delay", "2")
# Note: BlockState is mainly for reading, use set_block_with_properties for setting
```

### Getting Blocks

```python
# Get block name only
block_name = schematic.get_block(0, 0, 0)
if block_name:
    print(f"Block: {block_name}")

# Get block with properties
block_data = schematic.get_block_with_properties(0, 1, 0)
if block_data:
    print(f"Block: {block_data['name']}")
    print(f"Properties: {block_data['properties']}")
```

### Iterating Blocks

```python
# Get all blocks
all_blocks = schematic.blocks()
for block in all_blocks:
    print(f"({block['x']}, {block['y']}, {block['z']}) = {block['name']}")
    print(f"Properties: {block['properties']}")

# Filter non-air blocks
non_air_blocks = [b for b in all_blocks if 'air' not in b['name']]

# Count block types
from collections import Counter

block_counts = Counter(b['name'] for b in all_blocks)
for block_name, count in block_counts.most_common():
    print(f"{block_name}: {count}")
```

### Chunk Iteration

```python
# Get chunks (bottom-up order)
chunks = schematic.chunks(16, 16, 16)
for chunk in chunks:
    print(f"Chunk at ({chunk['offset_x']}, {chunk['offset_y']}, {chunk['offset_z']})")
    print(f"Blocks: {len(chunk['blocks'])}")

# Get chunks with strategy
strategies = [
    "distance_to_camera",
    "top_down",
    "bottom_up",
    "center_outward",
    "random"
]

chunks = schematic.chunks_with_strategy(
    16, 16, 16,
    "distance_to_camera",
    0, 100, 0  # Camera position
)

# Get specific chunk
chunk_blocks = schematic.get_chunk_blocks(0, 0, 0, 16, 16, 16)
```

## Region Operations

### Copying Regions

```python
# Copy a region
schematic.copy_region(
    "Main",           # Source region name
    0, 0, 0,         # Min coordinates
    10, 10, 10,      # Max coordinates
    20, 0, 0,        # Target position
    ["minecraft:air"] # Excluded blocks
)
```

### Working with Multiple Regions

```python
# Get all region names
regions = schematic.get_region_names()
print(f"Regions: {regions}")

# Get dimensions
width, height, depth = schematic.get_dimensions()
print(f"Size: {width}x{height}x{depth}")

# Get block/volume counts
print(f"Blocks: {schematic.get_block_count()}")
print(f"Volume: {schematic.get_volume()}")
```

## Block Entities

### Setting Block Entities

```python
# Set block with entity using string notation
schematic.set_block_from_string(
    0, 1, 0,
    'minecraft:barrel[facing=up]{signal=13}'
)

# Note: Direct block entity manipulation is limited in Python bindings
# Use bracket notation with {nbt} for most cases
```

### Getting Block Entities

```python
# Get single block entity
entity = schematic.get_block_entity(0, 1, 0)
if entity:
    print(f"Entity: {entity}")
    # Entity is a dict with NBT data

# Get all block entities
all_entities = schematic.get_all_block_entities()
for entity in all_entities:
    print(f"Entity at ({entity['x']}, {entity['y']}, {entity['z']}): {entity['data']}")
```

## SchematicBuilder

Build schematics programmatically with ASCII art and compositional design.

See [SchematicBuilder Guide](../shared/guide/schematic-builder.md) for complete documentation.

### Quick Example

```python
from nucleation import SchematicBuilder

circuit = SchematicBuilder.new() \
    .from_template("""
        # Base layer
        ccc
        ccc
        
        # Logic layer
        ─→─
        │█│
    """) \
    .build()
```

### Compositional Design

```python
# Build basic gates
and_gate = create_and_gate()
xor_gate = create_xor_gate()

# Compose into larger circuit
half_adder = SchematicBuilder.new() \
    .map_schematic('A', and_gate) \
    .map_schematic('X', xor_gate) \
    .layers([["AX"]]) \
    .build()
```

## Simulation

Simulate redstone circuits in real-time.

### Basic Simulation

```python
from nucleation import Schematic

schematic = Schematic("circuit")
# Build circuit...
schematic.set_block_from_string(0, 1, 0, "minecraft:lever[facing=north,powered=false]")
schematic.set_block_from_string(5, 1, 0, "minecraft:redstone_lamp[lit=false]")
# ... add redstone wiring ...

# Create simulation world
world = schematic.create_simulation_world()

# Toggle lever
world.on_use_block(0, 1, 0)

# Run simulation
world.tick(10)
world.flush()

# Check if lamp is lit
is_lit = world.is_lit(5, 1, 0)
print(f"Lamp is lit: {is_lit}")
```

### Custom IO Simulation

```python
from nucleation import SimulationOptions

# Configure custom IO positions
options = SimulationOptions()
options.add_custom_io(0, 1, 0)   # Input position
options.add_custom_io(10, 1, 0)  # Output position

world = schematic.create_simulation_world_with_options(options)

# Inject custom signal strength (0-15)
world.set_signal_strength(0, 1, 0, 15)  # Max power
world.tick(5)
world.flush()

# Read signal strength
output_signal = world.get_signal_strength(10, 1, 0)
print(f"Output signal: {output_signal}")
```

### Batch Signal Operations

```python
# Set multiple signals
positions = [(0, 1, 0), (0, 1, 2), (0, 1, 4)]
strengths = [15, 0, 15]

for (x, y, z), strength in zip(positions, strengths):
    world.set_signal_strength(x, y, z, strength)

world.tick(10)
world.flush()

# Read multiple signals
outputs = [world.get_signal_strength(x, y, z) for x, y, z in positions]
```

## TypedCircuitExecutor

High-level API for circuit simulation with typed inputs/outputs.

See [TypedCircuitExecutor Guide](../shared/guide/typed-executor.md) for complete documentation.

### Quick Example

```python
from nucleation import TypedCircuitExecutor, IoType, LayoutFunction, Value

# Define IO mappings
inputs = {
    "a": {
        "io_type": IoType.Bool,
        "layout": LayoutFunction.OneToOne,
        "positions": [(0, 1, 0)]
    },
    "b": {
        "io_type": IoType.Bool,
        "layout": LayoutFunction.OneToOne,
        "positions": [(0, 1, 2)]
    }
}

outputs = {
    "result": {
        "io_type": IoType.Bool,
        "layout": LayoutFunction.OneToOne,
        "positions": [(10, 1, 0)]
    }
}

# Create executor
executor = TypedCircuitExecutor(world, inputs, outputs)

# Execute with typed values
input_values = {
    "a": Value.Bool(True),
    "b": Value.Bool(True)
}

result = executor.execute(input_values, {
    "mode": "fixed_ticks",
    "ticks": 100
})

# Get typed output
output = result["outputs"]["result"]
print(f"Result: {output}")  # Value.Bool(True)
```

## Type Hints

```python
from typing import TypedDict, Literal

class BlockDict(TypedDict):
    x: int
    y: int
    z: int
    name: str
    properties: dict[str, str]

class ChunkDict(TypedDict):
    offset_x: int
    offset_y: int
    offset_z: int
    blocks: list[BlockDict]

ChunkStrategy = Literal[
    "distance_to_camera",
    "top_down",
    "bottom_up",
    "center_outward",
    "random"
]

class ExecutionMode(TypedDict, total=False):
    mode: Literal["fixed_ticks", "until_condition", "until_stable", "until_change"]
    ticks: int
    output: str
    condition: any
    timeout: int
    stable_ticks: int
```

## Examples

### Load and Modify

```python
from nucleation import Schematic

# Load existing schematic
with open("input.litematic", "rb") as f:
    schematic = Schematic("modified")
    schematic.load_from_bytes(f.read())

# Modify blocks
for x in range(10):
    for z in range(10):
        schematic.set_block(x, 0, z, "minecraft:stone")

# Save modified version
with open("output.litematic", "wb") as f:
    f.write(schematic.to_litematic())
```

### Build and Test Circuit

```python
from nucleation import Schematic

def build_and_test_circuit():
    schematic = Schematic("test_circuit")
    
    # Build AND gate
    schematic.set_block(0, 0, 0, "minecraft:stone")
    schematic.set_block_from_string(0, 1, 0, "minecraft:lever[facing=north,powered=false]")
    # ... build circuit ...
    
    # Test simulation
    world = schematic.create_simulation_world()
    world.on_use_block(0, 1, 0)  # Toggle input
    world.tick(10)
    world.flush()
    
    output = world.is_lit(10, 1, 0)
    assert output == True, "Test failed!"
    print("Test passed!")

build_and_test_circuit()
```

### Batch Processing

```python
import os
from nucleation import Schematic

def convert_all_schematics(input_dir: str, output_dir: str):
    """Convert all .schematic files to .litematic"""
    os.makedirs(output_dir, exist_ok=True)
    
    for filename in os.listdir(input_dir):
        if not filename.endswith('.schematic'):
            continue
        
        input_path = os.path.join(input_dir, filename)
        output_path = os.path.join(output_dir, filename.replace('.schematic', '.litematic'))
        
        # Load
        with open(input_path, "rb") as f:
            schematic = Schematic(filename)
            schematic.load_from_schematic(f.read())
        
        # Save
        with open(output_path, "wb") as f:
            f.write(schematic.to_litematic())
        
        print(f"Converted: {filename}")

convert_all_schematics("input/", "output/")
```

## See Also

- [SchematicBuilder Guide](../shared/guide/schematic-builder.md)
- [TypedCircuitExecutor Guide](../shared/guide/typed-executor.md)
- [Unicode Palette Reference](../shared/unicode-palette.md)
- [PyPI Package](https://pypi.org/project/nucleation)


