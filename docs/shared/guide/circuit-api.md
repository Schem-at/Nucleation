# Circuit API: Advanced IO and Builder Patterns

This guide covers the enhanced DefinitionRegion operations and the unified CircuitBuilder pattern for creating redstone circuit simulators.

## Table of Contents

- [DefinitionRegion Enhancements](#definitionregion-enhancements)
  - [Boolean Operations](#boolean-operations)
  - [Geometric Transformations](#geometric-transformations)
  - [Connectivity Analysis](#connectivity-analysis)
  - [Property Filtering](#property-filtering)
- [CircuitBuilder](#circuitbuilder)
  - [Basic Usage](#basic-usage)
  - [From Insign Annotations](#from-insign-annotations)
  - [Validation](#validation)
  - [State Modes](#state-modes)
- [Manual Tick Control](#manual-tick-control)

---

## DefinitionRegion Enhancements

`DefinitionRegion` now supports advanced operations for manipulating IO regions.

### Boolean Operations

Combine regions using set operations:

```javascript
// JavaScript/WASM
const regionA = new DefinitionRegionWrapper();
regionA.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(5, 0, 0));

const regionB = new DefinitionRegionWrapper();
regionB.addBounds(new BlockPosition(3, 0, 0), new BlockPosition(7, 0, 0));

// Subtract: Remove points in B from A
regionA.subtract(regionB); // Result: [0, 1, 2]

// Intersect: Keep only points in both A and B
regionA.intersect(regionB); // Result: [3, 4, 5]

// Union: Combine both regions (creates new region)
const unionRegion = regionA.union(regionB); // Result: [0-7]
```

```rust
// Rust
let mut region_a = DefinitionRegion::from_bounds((0, 0, 0), (5, 0, 0));
let region_b = DefinitionRegion::from_bounds((3, 0, 0), (7, 0, 0));

region_a.subtract(&region_b);
// or
region_a.intersect(&region_b);
// or
let union = region_a.union(&region_b);
```

### Geometric Transformations

Move and resize regions:

```javascript
// JavaScript/WASM
const region = new DefinitionRegionWrapper();
region.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(2, 2, 2));

// Shift by offset
region.shift(10, 20, 30); // min is now (10, 20, 30)

// Expand outward (negative contracts)
region.expand(2, 2, 2); // Grows by 2 in all directions

// Contract uniformly
region.contract(1); // Shrinks by 1 in all directions

// Get bounding box
const bounds = region.getBounds();
// { min: [x, y, z], max: [x, y, z] }
```

```rust
// Rust
let mut region = DefinitionRegion::from_bounds((5, 5, 5), (10, 10, 10));

region.shift(10, 0, 0);    // Translate
region.expand(2, 2, 2);    // Grow outward
region.contract(1);        // Shrink uniformly

if let Some(bbox) = region.get_bounds() {
    println!("Bounds: {:?} to {:?}", bbox.min, bbox.max);
}
```

### Connectivity Analysis

Check if regions form connected components:

```javascript
// JavaScript/WASM
const region = new DefinitionRegionWrapper();
region.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(3, 0, 0));
region.addBounds(new BlockPosition(3, 0, 0), new BlockPosition(3, 3, 0));

// Check if all points are connected (6-connectivity)
const isConnected = region.isContiguous(); // true for L-shape

// Count separate components
const components = region.connectedComponents(); // 1

// Two separate regions
const separate = new DefinitionRegionWrapper();
separate.addPoint(0, 0, 0);
separate.addPoint(10, 10, 10);
separate.connectedComponents(); // 2
```

```rust
// Rust
let region = DefinitionRegion::from_bounds((0, 0, 0), (5, 5, 5));

if region.is_contiguous() {
    println!("Region is connected!");
}

let components = region.connected_components();
println!("Found {} separate components", components);
```

### Property Filtering

Filter regions by block properties:

```javascript
// JavaScript/WASM
const schematic = new SchematicWrapper();
schematic.set_block_with_properties(0, 0, 0, "minecraft:redstone_lamp", {
	lit: "true",
});
schematic.set_block_with_properties(1, 0, 0, "minecraft:redstone_lamp", {
	lit: "false",
});
schematic.set_block_with_properties(2, 0, 0, "minecraft:redstone_lamp", {
	lit: "true",
});

const region = new DefinitionRegionWrapper();
region.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(2, 0, 0));

// Filter by block name (substring match)
const wireRegion = region.filterByBlock(schematic, "redstone_wire");

// Filter by specific properties
const litLamps = region.filterByProperties(schematic, { lit: "true" });
// Returns region with only positions where lamp is lit
```

```rust
// Rust
use std::collections::HashMap;

let region = DefinitionRegion::from_bounds((0, 0, 0), (10, 0, 10));

// Filter by block name
let wires = region.filter_by_block(&schematic, "redstone_wire");

// Filter by properties
let mut props = HashMap::new();
props.insert("lit".to_string(), "true".to_string());
let lit_lamps = region.filter_by_properties(&schematic, &props);

// Custom filter predicate
let powered = region.filter_by(&schematic, |block| {
    block.properties.get("powered").map_or(false, |v| v == "true")
});
```

---

## CircuitBuilder

The `CircuitBuilder` provides a fluent API for creating `TypedCircuitExecutor` instances.

### Basic Usage

```javascript
// JavaScript/WASM
const {
	CircuitBuilderWrapper,
	DefinitionRegionWrapper,
	IoTypeWrapper,
	LayoutFunctionWrapper,
	BlockPosition,
} = nucleation;

const schematic = new SchematicWrapper();
// ... set up your schematic ...

// Define input region
const inputRegion = new DefinitionRegionWrapper();
inputRegion.addBounds(new BlockPosition(0, 1, 0), new BlockPosition(7, 1, 0));

// Define output region
const outputRegion = new DefinitionRegionWrapper();
outputRegion.addBounds(
	new BlockPosition(0, 1, 10),
	new BlockPosition(7, 1, 10)
);

// Build executor
let builder = new CircuitBuilderWrapper(schematic);
builder = builder.withInputAuto(
	"data_in",
	IoTypeWrapper.unsignedInt(8),
	inputRegion
);
builder = builder.withOutputAuto(
	"data_out",
	IoTypeWrapper.unsignedInt(8),
	outputRegion
);

const executor = builder.build();
```

```rust
// Rust
use nucleation::simulation::circuit_builder::CircuitBuilder;
use nucleation::simulation::typed_executor::{IoType, LayoutFunction};
use nucleation::definition_region::DefinitionRegion;

let schematic = UniversalSchematic::new("circuit".to_string());
// ... set up schematic ...

let input_region = DefinitionRegion::from_bounds((0, 1, 0), (7, 1, 0));
let output_region = DefinitionRegion::from_bounds((0, 1, 10), (7, 1, 10));

let executor = CircuitBuilder::new(schematic)
    .with_input_auto("data_in", IoType::UnsignedInt { bits: 8 }, input_region)?
    .with_output_auto("data_out", IoType::UnsignedInt { bits: 8 }, output_region)?
    .build()?;
```

### From Insign Annotations

Create a builder pre-populated from Insign sign annotations:

```javascript
// JavaScript/WASM
const builder = CircuitBuilderWrapper.fromInsign(schematic);

// Optionally add more IO or modify settings
builder = builder.withStateMode("stateful");

const executor = builder.build();
```

```rust
// Rust
let builder = CircuitBuilder::from_insign(schematic)?;
let executor = builder.build()?;
```

### Validation

Validate circuit configuration before building:

```javascript
// JavaScript/WASM
const builder = new CircuitBuilderWrapper(schematic);
// ... add inputs/outputs ...

// Validate explicitly
builder.validate(); // Throws if invalid

// Or validate during build
const executor = builder.buildValidated(); // Validates first
```

```rust
// Rust
let builder = CircuitBuilder::new(schematic)
    .with_input(/* ... */)?
    .with_output(/* ... */)?;

// Check for validation errors
match builder.validate() {
    Ok(_) => println!("Configuration valid"),
    Err(e) => println!("Validation error: {}", e),
}

// Or use build_validated for one-step validation + build
let executor = builder.build_validated()?;
```

Validation checks for:

- At least one input and one output defined
- No overlapping IO regions
- No empty regions

### State Modes

Control how state is managed between executions:

```javascript
// JavaScript/WASM
builder = builder.withStateMode("stateless"); // Reset before each execute (default)
builder = builder.withStateMode("stateful"); // Preserve state between executes
builder = builder.withStateMode("manual"); // User controls reset
```

```rust
// Rust
use nucleation::simulation::typed_executor::StateMode;

let builder = CircuitBuilder::new(schematic)
    .with_state_mode(StateMode::Stateful)
    // ...
```

---

## Manual Tick Control

For fine-grained control over simulation timing:

```javascript
// JavaScript/WASM
const executor = /* create executor */;
executor.setStateMode("manual");

// Set inputs individually
const value = ValueWrapper.fromU32(42);
executor.setInput("data_in", value);
executor.flush();  // Propagate changes

// Tick manually
executor.tick(10);  // Advance 10 ticks
executor.flush();   // Ensure state is current

// Read outputs individually
const output = executor.readOutput("data_out");
console.log("Output:", output.toJs());

// Get all IO names
console.log("Inputs:", executor.inputNames());
console.log("Outputs:", executor.outputNames());
```

```rust
// Rust
let mut executor = /* create executor */;
executor.set_state_mode(StateMode::Manual);

// Set inputs
executor.set_input("data_in", &Value::U32(42))?;

// Manual tick
executor.tick(10);
executor.flush();

// Read outputs
let output = executor.read_output("data_out")?;
println!("Output: {:?}", output);

// Access underlying world for advanced operations
let world = executor.world_mut();
world.on_use_block(BlockPos::new(0, 1, 0));  // Toggle lever
```

---

## API Reference

### DefinitionRegion Methods

| Method                                   | Description                               |
| ---------------------------------------- | ----------------------------------------- |
| `subtract(other)`                        | Remove points in `other` from this region |
| `intersect(other)`                       | Keep only points present in both regions  |
| `union(other)`                           | Create new region combining both          |
| `shift(x, y, z)`                         | Translate all points by offset            |
| `expand(x, y, z)`                        | Grow boxes by amount in each direction    |
| `contract(amount)`                       | Shrink boxes uniformly                    |
| `get_bounds()`                           | Get overall bounding box                  |
| `is_contiguous()`                        | Check if all points are connected         |
| `connected_components()`                 | Count separate connected regions          |
| `filter_by_block(schematic, name)`       | Filter by block name                      |
| `filter_by_properties(schematic, props)` | Filter by block properties                |
| `is_empty()`                             | Check if region has no points             |
| `contains(x, y, z)`                      | Check if point is in region               |
| `volume()`                               | Get total number of points                |
| `positions()`                            | Get all positions as array                |
| `simplify()`                             | Merge adjacent boxes                      |

### CircuitBuilder Methods

| Method                                   | Description                          |
| ---------------------------------------- | ------------------------------------ |
| `new(schematic)`                         | Create builder from schematic        |
| `fromInsign(schematic)`                  | Create from Insign annotations       |
| `withInput(name, type, layout, region)`  | Add input with explicit layout       |
| `withInputAuto(name, type, region)`      | Add input with auto-inferred layout  |
| `withOutput(name, type, layout, region)` | Add output with explicit layout      |
| `withOutputAuto(name, type, region)`     | Add output with auto-inferred layout |
| `withOptions(options)`                   | Set simulation options               |
| `withStateMode(mode)`                    | Set state management mode            |
| `validate()`                             | Check configuration validity         |
| `build()`                                | Create the executor                  |
| `buildValidated()`                       | Validate and build                   |
| `inputCount()`                           | Get number of inputs                 |
| `outputCount()`                          | Get number of outputs                |
| `inputNames()`                           | Get input names                      |
| `outputNames()`                          | Get output names                     |

### TypedCircuitExecutor Methods (New)

| Method                   | Description                                  |
| ------------------------ | -------------------------------------------- |
| `tick(ticks)`            | Advance simulation by ticks (manual control) |
| `flush()`                | Propagate pending changes                    |
| `set_input(name, value)` | Set single input value                       |
| `read_output(name)`      | Read single output value                     |
| `input_names()`          | Get all input names                          |
| `output_names()`         | Get all output names                         |
