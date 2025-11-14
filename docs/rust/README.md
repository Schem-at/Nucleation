# Nucleation - Rust Documentation

Complete API reference and guide for using Nucleation in Rust.

## Quick Start

```bash
cargo add nucleation
```

```rust
use nucleation::UniversalSchematic;

let mut schematic = UniversalSchematic::new("my_schematic".to_string());
schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));

// Save as litematic
let bytes = nucleation::litematic::to_litematic(&schematic)?;
std::fs::write("output.litematic", bytes)?;
```

## Table of Contents

1. [Core Types](#core-types)
2. [Loading and Saving](#loading-and-saving)
3. [Block Operations](#block-operations)
4. [Region Operations](#region-operations)
5. [Block Entities](#block-entities)
6. [SchematicBuilder](#schematicbuilder)
7. [Simulation](#simulation)
8. [TypedCircuitExecutor](#typedcircuitexecutor)

## Core Types

### UniversalSchematic

The main data structure for working with schematics.

```rust
pub struct UniversalSchematic {
    pub default_region: Region,
    pub default_region_name: String,
    pub other_regions: HashMap<String, Region>,
    // ...
}
```

**Methods:**
- `new(name: String) -> Self` - Create empty schematic
- `get_block(x: i32, y: i32, z: i32) -> Option<&BlockState>` - Get block at position
- `set_block(x: i32, y: i32, z: i32, block: BlockState)` - Set block at position
- `set_block_from_string(x: i32, y: i32, z: i32, block_str: &str) -> Result<(), String>` - Parse and set block
- `get_dimensions() -> (i32, i32, i32)` - Get schematic dimensions
- `total_blocks() -> usize` - Count non-air blocks
- `iter_blocks() -> impl Iterator<Item = (BlockPosition, &BlockState)>` - Iterate all blocks

### BlockState

Represents a block with its properties.

```rust
pub struct BlockState {
    pub name: String,
    pub properties: HashMap<String, String>,
}
```

**Methods:**
- `new(name: String) -> Self` - Create block with no properties
- `with_property(mut self, key: String, value: String) -> Self` - Add property (builder pattern)
- `to_string() -> String` - Convert to bracket notation

**Example:**
```rust
let block = BlockState::new("minecraft:lever".to_string())
    .with_property("facing".to_string(), "east".to_string())
    .with_property("powered".to_string(), "false".to_string());
```

### Region

A 3D region containing blocks and block entities.

```rust
pub struct Region {
    pub name: String,
    pub position: (i32, i32, i32),
    pub size: (i32, i32, i32),
    pub blocks: Vec<usize>,  // Palette indices
    pub palette: Vec<BlockState>,
    pub block_entities: HashMap<(i32, i32, i32), BlockEntity>,
    // ...
}
```

## Loading and Saving

### Litematic Format

```rust
use nucleation::litematic;

// Load
let bytes = std::fs::read("input.litematic")?;
let schematic = litematic::from_litematic(&bytes)?;

// Save
let bytes = litematic::to_litematic(&schematic)?;
std::fs::write("output.litematic", bytes)?;
```

### Sponge Schematic Format

```rust
use nucleation::sponge_schematic;

// Load
let bytes = std::fs::read("input.schem")?;
let schematic = sponge_schematic::from_schematic(&bytes)?;

// Save
let bytes = schematic.to_schematic()?;
std::fs::write("output.schem", bytes)?;
```

### Auto-Detection

```rust
let bytes = std::fs::read("unknown.file")?;
let mut schematic = UniversalSchematic::new("loaded".to_string());
schematic.load_from_data(&bytes)?;  // Auto-detects format
```

## Block Operations

### Setting Blocks

```rust
// Simple block
schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));

// Block with properties
let lever = BlockState::new("minecraft:lever".to_string())
    .with_property("facing".to_string(), "east".to_string())
    .with_property("powered".to_string(), "false".to_string());
schematic.set_block(0, 1, 0, lever);

// From string (bracket notation)
schematic.set_block_from_string(
    1, 1, 0,
    "minecraft:redstone_wire[power=15,north=side,south=side]"
)?;
```

### Getting Blocks

```rust
// Get block
if let Some(block) = schematic.get_block(0, 0, 0) {
    println!("Block: {}", block.name);
    for (key, value) in &block.properties {
        println!("  {}: {}", key, value);
    }
}

// Check if position is in bounds
let (width, height, depth) = schematic.get_dimensions();
if x >= 0 && x < width && y >= 0 && y < height && z >= 0 && z < depth {
    // Safe to access
}
```

### Iterating Blocks

```rust
// Iterate all blocks
for (pos, block_state) in schematic.iter_blocks() {
    println!("({}, {}, {}) = {}", pos.x, pos.y, pos.z, block_state.name);
}

// Filter non-air blocks
for (pos, block_state) in schematic.iter_blocks() {
    if !block_state.name.contains("air") {
        // Process non-air block
    }
}

// Count block types
let block_types = schematic.count_block_types();
for (block_state, count) in block_types {
    println!("{}: {} blocks", block_state, count);
}
```

## Region Operations

### Copying Regions

```rust
use nucleation::BoundingBox;

// Define source region
let source_bounds = BoundingBox::new((0, 0, 0), (10, 10, 10));

// Copy to new position
let target_pos = (20, 0, 0);
let excluded_blocks = vec!["minecraft:air".to_string()];

schematic.copy_region(
    &source_bounds,
    target_pos,
    &excluded_blocks
)?;
```

### Working with Multiple Regions

```rust
// Access default region
let default_region = &schematic.default_region;
println!("Default region size: {:?}", default_region.size);

// Access other regions
for (name, region) in &schematic.other_regions {
    println!("Region '{}': {:?}", name, region.size);
}

// Get block from specific region
let block = schematic.get_block_from_region("custom_region", 0, 0, 0)?;
```

### Tight Bounds

Get the actual bounds of non-air blocks:

```rust
if let Some(tight_bounds) = schematic.default_region.get_tight_bounds() {
    let dimensions = tight_bounds.get_dimensions();
    println!("Actual content size: {:?}", dimensions);
    println!("Min: {:?}, Max: {:?}", tight_bounds.min, tight_bounds.max);
}
```

## Block Entities

### Setting Block Entities

```rust
use nucleation::{BlockEntity, BlockPosition};

// Create block entity
let mut block_entity = BlockEntity::new("minecraft:barrel".to_string());
block_entity.set_nbt_value("signal", 13);  // Custom signal strength

// Set in schematic
let pos = BlockPosition { x: 0, y: 1, z: 0 };
schematic.set_block_entity(pos, block_entity);
```

### Getting Block Entities

```rust
// Get single block entity
let pos = BlockPosition { x: 0, y: 1, z: 0 };
if let Some(entity) = schematic.get_block_entity(pos) {
    println!("Entity type: {}", entity.id);
    if let Some(signal) = entity.get_nbt_value("signal") {
        println!("Signal: {}", signal);
    }
}

// Get all block entities
for entity in schematic.default_region.get_block_entities_as_list() {
    println!("Entity at ({}, {}, {}): {}",
        entity.position.0, entity.position.1, entity.position.2, entity.id);
}
```

## SchematicBuilder

Build schematics programmatically with ASCII art and compositional design.

See [SchematicBuilder Guide](../shared/guide/schematic-builder.md) for complete documentation.

### Quick Example

```rust
use nucleation::SchematicBuilder;

let circuit = SchematicBuilder::new()
    .from_template(r#"
        # Base layer
        ccc
        ccc
        
        # Logic layer
        ─→─
        │█│
        "#)
    .build()?;
```

### Compositional Design

```rust
// Build basic gates
let and_gate = create_and_gate();
let xor_gate = create_xor_gate();

// Compose into larger circuit
let half_adder = SchematicBuilder::new()
    .map_schematic('A', and_gate)
    .map_schematic('X', xor_gate)
    .layers(&[&["AX"]])
    .build()?;
```

## Simulation

Simulate redstone circuits using the MCHPRS integration.

**Requires:** `simulation` feature flag

```toml
[dependencies]
nucleation = { version = "*", features = ["simulation"] }
```

### Basic Simulation

```rust
use nucleation::simulation::{MchprsWorld, SimulationOptions};

// Create simulation world
let world = MchprsWorld::new(schematic)?;

// Interact with blocks
world.on_use_block(0, 1, 0)?;  // Toggle lever

// Run simulation ticks
world.tick(10)?;
world.flush()?;

// Check block state
let is_powered = world.is_powered(5, 1, 0)?;
println!("Block is powered: {}", is_powered);
```

### Custom IO Positions

```rust
use mchprs_blocks::BlockPos;

// Define custom IO positions for signal injection/monitoring
let mut options = SimulationOptions::default();
options.custom_io.push(BlockPos::new(0, 1, 0));  // Input
options.custom_io.push(BlockPos::new(10, 1, 0)); // Output

let mut world = MchprsWorld::with_options(schematic, options)?;

// Inject signal strength
world.set_signal_strength(BlockPos::new(0, 1, 0), 15)?;
world.tick(5)?;
world.flush()?;

// Read signal strength
let strength = world.get_signal_strength(BlockPos::new(10, 1, 0))?;
println!("Output signal: {}", strength);
```

### Batch Operations

```rust
// Set multiple signals at once
let positions = vec![(0, 1, 0), (0, 1, 2), (0, 1, 4)];
let strengths = vec![15, 0, 15];
world.set_signals_batch(&positions, &strengths)?;

// Read multiple signals
let output_positions = vec![(10, 1, 0), (10, 1, 2)];
let signals = world.get_signals_batch(&output_positions);
```

## TypedCircuitExecutor

High-level API for circuit simulation with typed inputs/outputs.

See [TypedCircuitExecutor Guide](../shared/guide/typed-executor.md) for complete documentation.

### Quick Example

```rust
use nucleation::{TypedCircuitExecutor, IoType, LayoutFunction, Value, ExecutionMode};
use std::collections::HashMap;

// Define IO mappings
let mut inputs = HashMap::new();
inputs.insert("a".to_string(), IoMapping {
    io_type: IoType::Bool,
    layout: LayoutFunction::OneToOne,
    positions: vec![(0, 1, 0)],
});

let mut outputs = HashMap::new();
outputs.insert("result".to_string(), IoMapping {
    io_type: IoType::Bool,
    layout: LayoutFunction::OneToOne,
    positions: vec![(10, 1, 0)],
});

// Create executor
let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

// Execute with typed values
let mut input_values = HashMap::new();
input_values.insert("a".to_string(), Value::Bool(true));
input_values.insert("b".to_string(), Value::Bool(true));

let result = executor.execute(
    input_values,
    ExecutionMode::FixedTicks { ticks: 100 }
)?;

// Get typed output
let output = result.outputs.get("result").unwrap();
assert_eq!(*output, Value::Bool(true));
```

## Feature Flags

```toml
[dependencies]
nucleation = { version = "*", features = ["simulation", "serde"] }
```

| Feature | Description |
|---------|-------------|
| `simulation` | Enable MCHPRS redstone simulation |
| `serde` | Enable JSON serialization/deserialization |
| `python` | Enable Python bindings (for building) |
| `wasm` | Enable WASM bindings (for building) |
| `ffi` | Enable C FFI (for building) |

## Examples

See the [`examples/`](../../examples/) directory for complete working examples:

- `create_simple_litematic.rs` - Basic schematic creation
- `test_unicode_circuit.rs` - SchematicBuilder with Unicode palettes
- `build_adder.rs` - 4-bit adder with compositional design
- `custom_io_signals.rs` - Circuit simulation with custom IO

## See Also

- [SchematicBuilder Guide](../shared/guide/schematic-builder.md)
- [TypedCircuitExecutor Guide](../shared/guide/typed-executor.md)
- [Unicode Palette Reference](../shared/unicode-palette.md)


