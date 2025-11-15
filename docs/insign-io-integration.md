# Insign IO Integration for TypedCircuitExecutor

This document describes how to use Insign DSL annotations to automatically create `TypedCircuitExecutor` instances with properly configured IO layouts.

## Overview

Instead of manually defining IO layouts in code, you can annotate your Minecraft schematics with signs using the Insign DSL. The system will:

1. Extract all signs from the schematic
2. Compile the Insign DSL to identify `io.*` regions
3. Auto-extract redstone positions from those regions
4. Sort positions using distance-based or custom sorting strategies
5. Create a fully configured `TypedCircuitExecutor`

## Basic Usage

### 1. Annotate Your Schematic with Signs

Place signs in your schematic with Insign annotations:

```
Sign at (0, 64, 0):
@io.counter=rc([0,0,0],[0,7,0])
#io.counter:type="output"
#io.counter:data_type="unsigned"
```

This defines an 8-bit unsigned output called `counter` at positions (0,64,0) through (0,71,0).

### 2. Create Executor from Schematic

```rust
use nucleation::simulation::typed_executor::create_executor_from_insign;

let schematic = /* load your schematic */;
let mut executor = create_executor_from_insign(&schematic)?;

// Execute with typed inputs
let mut inputs = HashMap::new();
inputs.insert("input_a".to_string(), Value::U32(42));

let result = executor.execute(
    inputs,
    ExecutionMode::FixedTicks { ticks: 10 }
)?;
```

## Insign Syntax for IO

### Region Definition

```
@io.<name>=rc([x1,y1,z1],[x2,y2,z2])
```

- `io.<name>`: All regions starting with `io.` are treated as circuit IO
- `rc`: Relative coordinates (offset from sign position)
- `ac`: Absolute coordinates (world coordinates)

### Required Metadata

#### `type` - IO Direction
```
#io.<name>:type="input"   // Input to the circuit
#io.<name>:type="output"  // Output from the circuit
```

#### `data_type` - Data Type

**Boolean (1 bit)**:
```
#io.enable:data_type="bool"
```

**Unsigned Integer (arbitrary bits)**:
```
#io.counter:data_type="unsigned"      // Infers bit width from position count
#io.counter:data_type="unsigned:8"    // Explicit 8-bit (validates position count)
```

**Signed Integer (arbitrary bits)**:
```
#io.offset:data_type="signed"         // Infers bit width from position count
#io.offset:data_type="signed:12"      // Explicit 12-bit
```

**Float32**:
```
#io.temperature:data_type="float32"   // Must have exactly 32 positions
#io.temperature:data_type="f32"       // Alias
```

### Optional Metadata

#### `sort` - Position Sorting Strategy

Controls how redstone positions are ordered (affects bit ordering):

```
#io.<name>:sort="distance"    // Sort by distance from sign (default)
#io.<name>:sort="y_first"     // Sort by Y offset first, then XZ distance
#io.<name>:sort="x_first"     // Sort by X offset first
#io.<name>:sort="z_first"     // Sort by Z offset first
```

**Why sorting matters**: The order of positions determines which position corresponds to bit 0, bit 1, etc.

## Examples

### Example 1: Simple 8-bit Counter

```
Sign at (0, 64, 0):
@io.counter=rc([0,0,0],[0,7,0])
#io.counter:type="output"
#io.counter:data_type="unsigned"
```

- Extracts 8 redstone wire positions vertically
- Sorts by distance from sign (0,64,0)
- Creates an 8-bit unsigned output

### Example 2: Multi-bit Input with Explicit Validation

```
Sign at (10, 64, 10):
@io.input_a=rc([0,0,0],[3,0,0])
#io.input_a:type="input"
#io.input_a:data_type="unsigned:16"
#io.input_a:sort="x_first"
```

- Extracts 4 redstone positions horizontally (packed 4 bits each = 16 bits total)
- Validates that exactly 16 positions are found
- Sorts by X offset first

### Example 3: Multiple IO Regions

```
Sign 1 at (0, 64, 0):
@io.a=rc([0,0,0],[3,0,0])
#io.a:type="input"
#io.a:data_type="unsigned"

Sign 2 at (10, 64, 0):
@io.b=rc([0,0,0],[3,0,0])
#io.b:type="input"
#io.b:data_type="unsigned"

Sign 3 at (5, 64, 10):
@io.result=rc([0,0,0],[7,0,0])
#io.result:type="output"
#io.result:data_type="unsigned"
```

Creates an executor with two 16-bit inputs (`a`, `b`) and one 32-bit output (`result`).

### Example 4: Accumulating Regions Across Signs

```
Sign 1 at (0, 64, 0):
@io.bus=rc([0,0,0],[0,7,0])
#io.bus:type="input"
#io.bus:data_type="unsigned"

Sign 2 at (0, 72, 0):
@io.bus=rc([0,0,0],[0,7,0])
```

- Both signs contribute to the same `io.bus` region
- Total: 16 positions (16-bit input)
- Positions are sorted relative to the **first** sign that defined the region

## Rotation Invariance

The distance-based sorting ensures that rotating your schematic 180° produces the same bit ordering:

**Original**:
- Sign at (0, 64, 0)
- Wires at (0, 64-71, 0)
- Bit 0 = (0, 64, 0), Bit 7 = (0, 71, 0)

**After 180° Rotation**:
- Sign at (0, 64, 0) (rotated position)
- Wires at (0, 64-71, 0) (rotated positions)
- Bit 0 = (0, 64, 0), Bit 7 = (0, 71, 0) ✅ Same ordering!

This works because positions are sorted by their **relative offset** from the sign, not absolute coordinates.

## Valid Custom IO Blocks

The following blocks are recognized as valid custom IO:

- `minecraft:redstone_wire`
- `minecraft:repeater`
- `minecraft:comparator`
- `minecraft:redstone_torch`
- `minecraft:redstone_wall_torch`
- `minecraft:lever`
- `minecraft:stone_button`
- `minecraft:oak_button`
- `minecraft:redstone_lamp`

## API Reference

### `parse_io_layout_from_insign`

```rust
pub fn parse_io_layout_from_insign(
    input: &[([i32; 3], String)],
    schematic: &UniversalSchematic,
) -> Result<IoLayoutBuilder, InsignIoError>
```

Parses Insign DSL and creates an `IoLayoutBuilder`.

**Parameters**:
- `input`: Array of (sign_position, sign_text) tuples
- `schematic`: The schematic to extract redstone positions from

**Returns**: `IoLayoutBuilder` ready to be built into an `IoLayout`

### `create_executor_from_insign`

```rust
pub fn create_executor_from_insign(
    schematic: &UniversalSchematic,
) -> Result<TypedCircuitExecutor, InsignIoError>
```

One-shot function to create a fully configured executor from a schematic.

**Parameters**:
- `schematic`: The schematic with Insign annotations

**Returns**: Ready-to-use `TypedCircuitExecutor`

### `SortStrategy`

```rust
pub enum SortStrategy {
    Distance,  // Sort by Euclidean distance from sign (default)
    YFirst,    // Sort by Y offset, then XZ distance
    XFirst,    // Sort by X offset, then YZ distance
    ZFirst,    // Sort by Z offset, then XY distance
}
```

## Error Handling

```rust
pub enum InsignIoError {
    CompileError(String),                          // Insign DSL syntax error
    NoPositions(String),                           // No valid redstone found in region
    InvalidDataType(String, String),               // Unknown or malformed data type
    MissingMetadata(String, String),               // Required metadata missing
    InvalidMetadataValue(String, String, String),  // Invalid metadata value
    PositionCountMismatch(String, usize, usize),   // Position count doesn't match type
    LayoutBuildError(String),                      // IoLayout build failed
    SchematicError(String),                        // Schematic loading error
}
```

## Best Practices

1. **Use relative coordinates (`rc`)** for portability - your annotations work regardless of where the schematic is placed
2. **Validate bit widths** with explicit types (e.g., `unsigned:8`) to catch errors early
3. **Use descriptive names** for IO regions (e.g., `io.player_health` not `io.a`)
4. **Document your sort strategy** if using non-default sorting
5. **Test rotation** if your build needs to work in multiple orientations

## Limitations

- Only `io.*` regions are processed (other Insign regions are ignored)
- Float64 is not supported (only Float32)
- Complex types (Array, Matrix, Struct) are not yet supported via Insign
- Layout functions are auto-inferred (Packed4 for ≤4 positions, OneToOne otherwise)

## Future Enhancements

- Support for 2D matrix layouts via Insign
- Custom layout function specification
- Array and struct type definitions
- Integration with schematic-renderer for visual IO overlay

