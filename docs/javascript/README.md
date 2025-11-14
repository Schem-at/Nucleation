# Nucleation - JavaScript/TypeScript Documentation

Complete API reference and guide for using Nucleation in JavaScript and TypeScript (via WebAssembly).

## Quick Start

```bash
npm install nucleation
```

```typescript
import init, { SchematicWrapper } from "nucleation";

// Initialize WASM module
await init();

// Create schematic
const schematic = new SchematicWrapper();
schematic.set_block(0, 0, 0, "minecraft:stone");

// Save as litematic
const bytes = schematic.to_litematic();
// Download or save bytes...
```

## Table of Contents

1. [Installation & Setup](#installation--setup)
2. [Core API](#core-api)
3. [Loading and Saving](#loading-and-saving)
4. [Block Operations](#block-operations)
5. [Region Operations](#region-operations)
6. [Block Entities](#block-entities)
7. [SchematicBuilder](#schematicbuilder)
8. [Simulation](#simulation)
9. [TypedCircuitExecutor](#typedcircuitexecutor)

## Installation & Setup

### Node.js / Bundlers

```bash
npm install nucleation
```

```typescript
import init, { SchematicWrapper } from "nucleation";

// Auto-detects environment and loads WASM
await init();

const schematic = new SchematicWrapper();
```

### Browser via CDN

```html
<script type="module">
  import init, { SchematicWrapper } from
    "https://cdn.jsdelivr.net/npm/nucleation@latest/nucleation-cdn-loader.js";

  await init();  // Automatically resolves WASM path
  const schematic = new SchematicWrapper();
</script>
```

### Manual WASM Loading

```typescript
import init, { SchematicWrapper } from "nucleation";

const wasmBytes = await fetch("/path/to/nucleation_bg.wasm")
  .then(r => r.arrayBuffer());

await init(wasmBytes);
```

## Core API

### SchematicWrapper

Main class for working with schematics.

```typescript
class SchematicWrapper {
  constructor();  // Creates empty schematic named "Default"
  
  // Loading/Saving
  from_data(bytes: Uint8Array): void;
  from_litematic(bytes: Uint8Array): void;
  from_schematic(bytes: Uint8Array): void;
  to_litematic(): Uint8Array;
  to_schematic(): Uint8Array;
  
  // Block operations
  set_block(x: number, y: number, z: number, blockName: string): void;
  set_block_with_properties(x: number, y: number, z: number, 
                            blockName: string, properties: object): void;
  set_block_from_string(x: number, y: number, z: number, 
                       blockString: string): void;
  get_block(x: number, y: number, z: number): string | null;
  get_block_with_properties(x: number, y: number, z: number): BlockStateWrapper | null;
  
  // Block entities
  get_block_entity(x: number, y: number, z: number): object | null;
  get_all_block_entities(): Array<object>;
  
  // Region operations
  copy_region(sourceRegion: string, minX: number, minY: number, minZ: number,
             maxX: number, maxY: number, maxZ: number,
             targetX: number, targetY: number, targetZ: number,
             excludedBlocks: string[]): void;
  
  // Information
  get_dimensions(): [number, number, number];
  get_block_count(): number;
  get_volume(): number;
  get_region_names(): string[];
  debug_info(): string;
  print_schematic(): string;
  
  // Iteration
  blocks(): Array<{x: number, y: number, z: number, name: string, properties: object}>;
  chunks(width: number, height: number, length: number): Array<any>;
  chunks_with_strategy(width: number, height: number, length: number,
                      strategy: string, cx?: number, cy?: number, cz?: number): Array<any>;
  get_chunk_blocks(offsetX: number, offsetY: number, offsetZ: number,
                  width: number, height: number, length: number): Array<any>;
  
  // Simulation (requires simulation feature)
  create_simulation_world(): MchprsWorldWrapper;
  create_simulation_world_with_options(options: SimulationOptionsWrapper): MchprsWorldWrapper;
}
```

### BlockStateWrapper

Represents a block with properties.

```typescript
class BlockStateWrapper {
  constructor(name: string);
  
  with_property(key: string, value: string): void;  // Mutates in place
  name(): string;
  properties(): object;
}
```

## Loading and Saving

### Load from File

```typescript
// Browser
const fileInput = document.querySelector('input[type="file"]');
fileInput.addEventListener('change', async (e) => {
  const file = e.target.files[0];
  const bytes = new Uint8Array(await file.arrayBuffer());
  
  const schematic = new SchematicWrapper();
  schematic.from_data(bytes);  // Auto-detects format
  
  console.log(schematic.get_dimensions());
});

// Node.js
import { readFileSync } from 'fs';

const bytes = new Uint8Array(readFileSync('input.litematic'));
const schematic = new SchematicWrapper();
schematic.from_litematic(bytes);
```

### Save to File

```typescript
// Browser - Download
const bytes = schematic.to_litematic();
const blob = new Blob([bytes], { type: 'application/octet-stream' });
const url = URL.createObjectURL(blob);

const a = document.createElement('a');
a.href = url;
a.download = 'output.litematic';
a.click();

URL.revokeObjectURL(url);

// Node.js
import { writeFileSync } from 'fs';

const bytes = schematic.to_litematic();
writeFileSync('output.litematic', bytes);
```

## Block Operations

### Setting Blocks

```typescript
// Simple block
schematic.set_block(0, 0, 0, "minecraft:stone");

// Block with properties (object)
schematic.set_block_with_properties(0, 1, 0, "minecraft:lever", {
  facing: "east",
  powered: "false"
});

// Block from string (bracket notation)
schematic.set_block_from_string(
  1, 1, 0,
  "minecraft:redstone_wire[power=15,north=side,south=side]"
);

// Using BlockStateWrapper
const block = new BlockStateWrapper("minecraft:repeater");
block.with_property("facing", "east");
block.with_property("delay", "2");
// Note: BlockStateWrapper is mainly for reading, use set_block_with_properties for setting
```

### Getting Blocks

```typescript
// Get block name only
const blockName = schematic.get_block(0, 0, 0);
if (blockName) {
  console.log(`Block: ${blockName}`);
}

// Get block with properties
const blockState = schematic.get_block_with_properties(0, 1, 0);
if (blockState) {
  console.log(`Block: ${blockState.name()}`);
  console.log(`Properties:`, blockState.properties());
}
```

### Iterating Blocks

```typescript
// Get all blocks
const allBlocks = schematic.blocks();
for (const block of allBlocks) {
  console.log(`(${block.x}, ${block.y}, ${block.z}) = ${block.name}`);
  console.log(`Properties:`, block.properties);
}

// Filter non-air blocks
const nonAirBlocks = allBlocks.filter(b => !b.name.includes('air'));

// Count block types
const blockCounts = new Map();
for (const block of allBlocks) {
  const count = blockCounts.get(block.name) || 0;
  blockCounts.set(block.name, count + 1);
}
```

### Chunk Iteration

```typescript
// Get chunks (bottom-up order)
const chunks = schematic.chunks(16, 16, 16);
for (const chunk of chunks) {
  console.log(`Chunk at (${chunk.offset_x}, ${chunk.offset_y}, ${chunk.offset_z})`);
  console.log(`Blocks:`, chunk.blocks);
}

// Get chunks with strategy
const strategies = [
  "distance_to_camera",
  "top_down",
  "bottom_up",
  "center_outward",
  "random"
];

const chunks = schematic.chunks_with_strategy(
  16, 16, 16,
  "distance_to_camera",
  0, 100, 0  // Camera position
);

// Get specific chunk
const chunkBlocks = schematic.get_chunk_blocks(0, 0, 0, 16, 16, 16);
```

## Region Operations

### Copying Regions

```typescript
// Copy a region
schematic.copy_region(
  "Main",           // Source region name
  0, 0, 0,         // Min coordinates
  10, 10, 10,      // Max coordinates
  20, 0, 0,        // Target position
  ["minecraft:air"] // Excluded blocks
);
```

### Working with Multiple Regions

```typescript
// Get all region names
const regions = schematic.get_region_names();
console.log(`Regions:`, regions);

// Get dimensions
const [width, height, depth] = schematic.get_dimensions();
console.log(`Size: ${width}x${height}x${depth}`);

// Get block/volume counts
console.log(`Blocks: ${schematic.get_block_count()}`);
console.log(`Volume: ${schematic.get_volume()}`);
```

## Block Entities

### Setting Block Entities

```typescript
// Set block with entity using string notation
schematic.set_block_from_string(
  0, 1, 0,
  'minecraft:barrel[facing=up]{signal=13}'
);

// Note: Direct block entity manipulation is limited in WASM
// Use bracket notation with {nbt} for most cases
```

### Getting Block Entities

```typescript
// Get single block entity
const entity = schematic.get_block_entity(0, 1, 0);
if (entity) {
  console.log(`Entity:`, entity);
  // Entity is a plain JavaScript object with NBT data
}

// Get all block entities
const allEntities = schematic.get_all_block_entities();
for (const entity of allEntities) {
  console.log(`Entity at (${entity.x}, ${entity.y}, ${entity.z}):`, entity.data);
}
```

## SchematicBuilder

Build schematics programmatically with ASCII art and compositional design.

See [SchematicBuilder Guide](../shared/guide/schematic-builder.md) for complete documentation.

### Quick Example

```typescript
import init, { SchematicBuilder } from "nucleation";
await init();

const circuit = SchematicBuilder.new()
  .from_template(`
    # Base layer
    ccc
    ccc
    
    # Logic layer
    ─→─
    │█│
  `)
  .build();
```

### Compositional Design

```typescript
// Build basic gates
const andGate = createAndGate();
const xorGate = createXorGate();

// Compose into larger circuit
const halfAdder = SchematicBuilder.new()
  .map_schematic('A', andGate)
  .map_schematic('X', xorGate)
  .layers([["AX"]])
  .build();
```

## Simulation

Simulate redstone circuits in real-time.

### Basic Simulation

```typescript
import init, { SchematicWrapper } from "nucleation";
await init();

const schematic = new SchematicWrapper();
// Build circuit...
schematic.set_block_from_string(0, 1, 0, "minecraft:lever[facing=north,powered=false]");
schematic.set_block_from_string(5, 1, 0, "minecraft:redstone_lamp[lit=false]");
// ... add redstone wiring ...

// Create simulation world
const world = schematic.create_simulation_world();

// Toggle lever
world.on_use_block(0, 1, 0);

// Run simulation
world.tick(10);
world.flush();

// Check if lamp is lit
const isLit = world.is_lit(5, 1, 0);
console.log(`Lamp is lit: ${isLit}`);
```

### Custom IO Simulation

```typescript
import { SimulationOptionsWrapper } from "nucleation";

// Configure custom IO positions
const options = new SimulationOptionsWrapper();
options.addCustomIo(0, 1, 0);  // Input position
options.addCustomIo(10, 1, 0); // Output position

const world = schematic.create_simulation_world_with_options(options);

// Inject custom signal strength (0-15)
world.setSignalStrength(0, 1, 0, 15);  // Max power
world.tick(5);
world.flush();

// Read signal strength
const outputSignal = world.getSignalStrength(10, 1, 0);
console.log(`Output signal: ${outputSignal}`);
```

### Batch Signal Operations

```typescript
// Set multiple signals at once
const positions = [[0, 1, 0], [0, 1, 2], [0, 1, 4]];
const strengths = [15, 0, 15];

for (let i = 0; i < positions.length; i++) {
  const [x, y, z] = positions[i];
  world.setSignalStrength(x, y, z, strengths[i]);
}

world.tick(10);
world.flush();

// Read multiple signals
const outputs = positions.map(([x, y, z]) => 
  world.getSignalStrength(x, y, z)
);
```

## TypedCircuitExecutor

High-level API for circuit simulation with typed inputs/outputs.

See [TypedCircuitExecutor Guide](../shared/guide/typed-executor.md) for complete documentation.

### Quick Example

```typescript
import { TypedCircuitExecutor, IoType, LayoutFunction, Value } from "nucleation";

// Define IO mappings
const inputs = new Map([
  ["a", {
    io_type: IoType.Bool,
    layout: LayoutFunction.OneToOne,
    positions: [[0, 1, 0]]
  }],
  ["b", {
    io_type: IoType.Bool,
    layout: LayoutFunction.OneToOne,
    positions: [[0, 1, 2]]
  }]
]);

const outputs = new Map([
  ["result", {
    io_type: IoType.Bool,
    layout: LayoutFunction.OneToOne,
    positions: [[10, 1, 0]]
  }]
]);

// Create executor
const executor = new TypedCircuitExecutor(world, inputs, outputs);

// Execute with typed values
const inputValues = new Map([
  ["a", Value.Bool(true)],
  ["b", Value.Bool(true)]
]);

const result = executor.execute(inputValues, {
  mode: "fixed_ticks",
  ticks: 100
});

// Get typed output
const output = result.outputs.get("result");
console.log(`Result: ${output}`);  // Value.Bool(true)
```

## TypeScript Types

```typescript
// Block state
interface BlockState {
  name: string;
  properties: Record<string, string>;
}

// Block with position
interface PositionedBlock {
  x: number;
  y: number;
  z: number;
  name: string;
  properties: Record<string, string>;
}

// Chunk
interface Chunk {
  offset_x: number;
  offset_y: number;
  offset_z: number;
  blocks: PositionedBlock[];
}

// Execution mode
type ExecutionMode = 
  | { mode: "fixed_ticks", ticks: number }
  | { mode: "until_condition", output: string, condition: any, timeout: number }
  | { mode: "until_stable", stable_ticks: number, timeout: number }
  | { mode: "until_change", timeout: number };
```

## Examples

### Download Schematic

```typescript
function downloadSchematic(schematic: SchematicWrapper, filename: string) {
  const bytes = schematic.to_litematic();
  const blob = new Blob([bytes], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  
  URL.revokeObjectURL(url);
}
```

### Upload Schematic

```typescript
async function uploadSchematic(file: File): Promise<SchematicWrapper> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  const schematic = new SchematicWrapper();
  schematic.from_data(bytes);
  return schematic;
}

// Usage
const input = document.querySelector('input[type="file"]');
input.addEventListener('change', async (e) => {
  const schematic = await uploadSchematic(e.target.files[0]);
  console.log(schematic.get_dimensions());
});
```

### Build and Test Circuit

```typescript
async function buildAndTestCircuit() {
  await init();
  
  const schematic = new SchematicWrapper();
  
  // Build AND gate
  schematic.set_block(0, 0, 0, "minecraft:stone");
  schematic.set_block_from_string(0, 1, 0, "minecraft:lever[facing=north,powered=false]");
  // ... build circuit ...
  
  // Test simulation
  const world = schematic.create_simulation_world();
  world.on_use_block(0, 1, 0);  // Toggle input
  world.tick(10);
  world.flush();
  
  const output = world.is_lit(10, 1, 0);
  console.log(`Test passed: ${output === true}`);
}
```

## See Also

- [SchematicBuilder Guide](../shared/guide/schematic-builder.md)
- [TypedCircuitExecutor Guide](../shared/guide/typed-executor.md)
- [Unicode Palette Reference](../shared/unicode-palette.md)
- [NPM Package](https://www.npmjs.com/package/nucleation)


