# Nucleation WASM (JavaScript/TypeScript) API Reference

Nucleation provides first-class WebAssembly bindings for use in browsers and Node.js. All types are exported via `wasm-bindgen` and follow JavaScript naming conventions (camelCase methods, PascalCase types).

---

## Table of Contents

- [Core](#core)
  - [SchematicWrapper](#schematicwrapper)
  - [BlockStateWrapper](#blockstatewrapper)
  - [SchematicBuilderWrapper](#schematicbuilderwrapper)
  - [DefinitionRegionWrapper](#definitionregionwrapper)
  - [PaletteManager](#palettemanager)
  - [LazyChunkIterator](#lazychunkiterator)
- [Building](#building)
  - [ShapeWrapper](#shapewrapper)
  - [BrushWrapper](#brushwrapper)
  - [WasmBuildingTool](#wasmbuildingtool)
- [Simulation](#simulation-feature-gated) *(feature: `simulation`)*
  - [SimulationOptionsWrapper](#simulationoptionswrapper)
  - [MchprsWorldWrapper](#mchprsworldwrapper)
  - [CircuitBuilderWrapper](#circuitbuilderwrapper)
  - [TypedCircuitExecutorWrapper](#typedcircuitexecutorwrapper)
  - [IoLayoutBuilderWrapper](#iolayoutbuilderwrapper)
  - [IoLayoutWrapper](#iolayoutwrapper)
  - [ValueWrapper](#valuewrapper)
  - [IoTypeWrapper](#iotypewrapper)
  - [LayoutFunctionWrapper](#layoutfunctionwrapper)
  - [ExecutionModeWrapper](#executionmodewrapper)
  - [OutputConditionWrapper](#outputconditionwrapper)
  - [SortStrategyWrapper](#sortstrategywrapper)
  - [StateModeConstants](#statemodeconstants)
- [Meshing](#meshing-feature-gated) *(feature: `meshing`)*
  - [ResourcePackWrapper](#resourcepackwrapper)
  - [MeshConfigWrapper](#meshconfigwrapper)
  - [MeshOutputWrapper](#meshoutputwrapper)
  - [MultiMeshResultWrapper](#multimeshresultwrapper)
  - [ChunkMeshResultWrapper](#chunkmeshresultwrapper)
  - [ChunkMeshIteratorWrapper](#chunkmeshiteratorwrapper)
  - [TextureAtlasWrapper](#textureatlaswrapper)
  - [RawMeshExportWrapper](#rawmeshexportwrapper)
- [Module Functions](#module-functions)

---

## Core

### SchematicWrapper

The primary class for creating, loading, editing, and exporting Minecraft schematics. Wraps the internal `UniversalSchematic` and exposes the full API surface to JavaScript.

#### Constructor

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new() → SchematicWrapper` | Creates a new empty schematic. |

#### Format I/O

Load schematics from binary data in any supported format. All `from*` methods mutate the existing instance.

| Method | Signature | Description |
|--------|-----------|-------------|
| `fromData` | `fromData(data: Uint8Array) → void` | Auto-detect format and load. Supports Litematic, Sponge Schematic, and McStructure. |
| `fromLitematic` | `fromLitematic(data: Uint8Array) → void` | Load from `.litematic` format (Java Edition, Litematica mod). |
| `fromSchematic` | `fromSchematic(data: Uint8Array) → void` | Load from `.schematic` / `.schem` format (Sponge/WorldEdit). |
| `fromMcstructure` | `fromMcstructure(data: Uint8Array) → void` | Load from `.mcstructure` format (Bedrock Edition). |
| `fromMca` | `fromMca(data: Uint8Array) → void` | Load from a single MCA region file. |
| `fromMcaBounded` | `fromMcaBounded(data: Uint8Array, minX: number, minY: number, minZ: number, maxX: number, maxY: number, maxZ: number) → void` | Load MCA with coordinate bounds to limit loaded area. |
| `fromWorldZip` | `fromWorldZip(data: Uint8Array) → void` | Load from a zipped Minecraft world. |
| `fromWorldZipBounded` | `fromWorldZipBounded(data: Uint8Array, minX: number, minY: number, minZ: number, maxX: number, maxY: number, maxZ: number) → void` | Load zipped world with coordinate bounds. |
| `fromSnapshot` | `fromSnapshot(data: Uint8Array) → void` | Load from snapshot format (fast binary, `.nusn`). |

Export schematics to binary data.

| Method | Signature | Description |
|--------|-----------|-------------|
| `toLitematic` | `toLitematic() → Uint8Array` | Export to Litematic format. |
| `toSchematic` | `toSchematic() → Uint8Array` | Export to Sponge Schematic (default version). |
| `toSchematicVersion` | `toSchematicVersion(version: string) → Uint8Array` | Export to a specific Sponge Schematic version. |
| `toMcstructure` | `toMcstructure() → Uint8Array` | Export to Bedrock McStructure format. |
| `toWorld` | `toWorld(optionsJson?: string) → Map<string, Uint8Array>` | Export as Minecraft world files. Returns a map of file paths to byte content. |
| `toWorldZip` | `toWorldZip(optionsJson?: string) → Uint8Array` | Export as a zipped Minecraft world. |
| `toSnapshot` | `toSnapshot() → Uint8Array` | Export to snapshot format (fast binary, `.nusn`). |
| `saveAs` | `saveAs(format: string, version?: string, settings?: string) → Uint8Array` | Generic export to any registered format with optional version and JSON settings. |
| `getExportSettingsSchema` | `getExportSettingsSchema(format: string) → string \| undefined` | Get JSON schema for a format's export settings. |
| `getImportSettingsSchema` | `getImportSettingsSchema(format: string) → string \| undefined` | Get JSON schema for a format's import settings. |

**Static format discovery methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `getSupportedImportFormats` | `static getSupportedImportFormats() → string[]` | List all importable format names. |
| `getSupportedExportFormats` | `static getSupportedExportFormats() → string[]` | List all exportable format names. |
| `getFormatVersions` | `static getFormatVersions(format: string) → string[]` | List available versions for a format. |
| `getDefaultFormatVersion` | `static getDefaultFormatVersion(format: string) → string \| undefined` | Get the default version for a format. |
| `getAvailableSchematicVersions` | `static getAvailableSchematicVersions() → string[]` | List available Sponge Schematic versions. |

#### Block Operations

Set and get individual blocks. Block names follow the `minecraft:block_name` namespace format.

| Method | Signature | Description |
|--------|-----------|-------------|
| `setBlock` | `setBlock(x: number, y: number, z: number, blockName: string) → void` | Set a block by name (no properties). |
| `setBlockInRegion` | `setBlockInRegion(regionName: string, x: number, y: number, z: number, blockName: string) → boolean` | Set a block in a named region. |
| `setBlockFromString` | `setBlockFromString(x: number, y: number, z: number, blockString: string) → void` | Parse a full block string with properties and NBT, e.g. `"minecraft:chest[facing=north]{Items:[...]}"`. |
| `setBlockWithProperties` | `setBlockWithProperties(x: number, y: number, z: number, blockName: string, properties: object) → void` | Set a block with a JS object of properties, e.g. `{facing: "north", powered: "true"}`. |
| `setBlockWithNbt` | `setBlockWithNbt(x: number, y: number, z: number, blockName: string, nbtData: object) → void` | Set a block with block entity NBT data. |
| `getBlock` | `getBlock(x: number, y: number, z: number) → string \| undefined` | Get the block name at a position. |
| `getBlockString` | `getBlockString(x: number, y: number, z: number) → string \| undefined` | Get the full block string with properties. |
| `getBlockWithProperties` | `getBlockWithProperties(x: number, y: number, z: number) → BlockStateWrapper \| undefined` | Get a full BlockState wrapper with name and properties. |

#### Batch Block Operations

Efficient batch operations using flat integer arrays for positions (`[x0, y0, z0, x1, y1, z1, ...]`).

| Method | Signature | Description |
|--------|-----------|-------------|
| `setBlocks` | `setBlocks(positions: Int32Array, blockName: string) → number` | Set the same block at multiple positions. Returns count of blocks set. |
| `getBlocks` | `getBlocks(positions: Int32Array) → string[]` | Get block names at multiple positions. |
| `fillCuboid` | `fillCuboid(minX: number, minY: number, minZ: number, maxX: number, maxY: number, maxZ: number, blockState: string) → void` | Fill an axis-aligned cuboid with a single block. |
| `fillSphere` | `fillSphere(cx: number, cy: number, cz: number, radius: number, blockState: string) → void` | Fill a sphere with a single block. |

#### Region Copying

| Method | Signature | Description |
|--------|-----------|-------------|
| `copyRegion` | `copyRegion(fromSchematic: SchematicWrapper, minX: number, minY: number, minZ: number, maxX: number, maxY: number, maxZ: number, targetX: number, targetY: number, targetZ: number, excludedBlocks?: string[]) → void` | Copy a rectangular region from another schematic, optionally excluding specific block types. |

#### Block Entities

| Method | Signature | Description |
|--------|-----------|-------------|
| `getBlockEntity` | `getBlockEntity(x: number, y: number, z: number) → object \| undefined` | Get the block entity (tile entity) at a position. Returns `{id, position, nbt}`. |
| `getAllBlockEntities` | `getAllBlockEntities() → object[]` | Get all block entities in the schematic. |

#### Mobile Entities

| Method | Signature | Description |
|--------|-----------|-------------|
| `entityCount` | `entityCount() → number` | Get the number of mobile entities. |
| `getEntities` | `getEntities() → object[]` | Get all entities as `[{id, position: [x,y,z], nbt}, ...]`. |
| `addEntity` | `addEntity(id: string, x: number, y: number, z: number, nbtJson?: string) → void` | Add a mobile entity with optional NBT JSON. |
| `removeEntity` | `removeEntity(index: number) → boolean` | Remove an entity by its index. |

#### Transformations

Flip and rotate the entire schematic or a named region. Rotation values must be 90, 180, or 270 degrees.

| Method | Signature | Description |
|--------|-----------|-------------|
| `flipX` / `flipY` / `flipZ` | `flipX() → void` | Mirror the schematic along the given axis. |
| `rotateX` / `rotateY` / `rotateZ` | `rotateY(degrees: number) → void` | Rotate around the given axis (90/180/270). |
| `flipRegionX` / `flipRegionY` / `flipRegionZ` | `flipRegionX(regionName: string) → void` | Mirror a named region. |
| `rotateRegionX` / `rotateRegionY` / `rotateRegionZ` | `rotateRegionY(regionName: string, degrees: number) → void` | Rotate a named region. |

#### Metadata

| Method | Signature | Description |
|--------|-----------|-------------|
| `getName` / `setName` | `getName() → string \| undefined` | Schematic name. |
| `getAuthor` / `setAuthor` | `getAuthor() → string \| undefined` | Author name. |
| `getDescription` / `setDescription` | `getDescription() → string \| undefined` | Description text. |
| `getCreated` / `setCreated` | `getCreated() → number \| undefined` | Creation timestamp (milliseconds since epoch). |
| `getModified` / `setModified` | `getModified() → number \| undefined` | Modification timestamp. |
| `getLmVersion` / `setLmVersion` | `getLmVersion() → number \| undefined` | Litematic format version. |
| `getMcVersion` / `setMcVersion` | `getMcVersion() → number \| undefined` | Minecraft data version. |
| `getWeVersion` / `setWeVersion` | `getWeVersion() → number \| undefined` | WorldEdit format version. |

#### Dimensions & Bounds

| Method | Signature | Description |
|--------|-----------|-------------|
| `getDimensions` | `getDimensions() → number[]` | Tight dimensions `[width, height, length]` of actual content. |
| `getAllocatedDimensions` | `getAllocatedDimensions() → number[]` | Full allocated buffer dimensions. |
| `getTightDimensions` | `getTightDimensions() → number[]` | Alias for content-only dimensions. |
| `getTightBoundsMin` | `getTightBoundsMin() → number[] \| undefined` | Minimum corner `[x, y, z]` of content bounds. |
| `getTightBoundsMax` | `getTightBoundsMax() → number[] \| undefined` | Maximum corner `[x, y, z]` of content bounds. |
| `getBoundingBox` | `getBoundingBox() → {min: number[], max: number[]}` | Full bounding box. |
| `getRegionBoundingBox` | `getRegionBoundingBox(regionName: string) → {min: number[], max: number[]}` | Bounding box of a named region. |
| `getBlockCount` | `getBlockCount() → number` | Total non-air block count. |
| `getVolume` | `getVolume() → number` | Total bounding box volume. |
| `getRegionNames` | `getRegionNames() → string[]` | All region names in the schematic. |

#### Palette Access

| Method | Signature | Description |
|--------|-----------|-------------|
| `getPalette` | `getPalette() → object` | Merged palette of all regions. |
| `getDefaultRegionPalette` | `getDefaultRegionPalette() → object` | Palette of the default region. |
| `getPaletteFromRegion` | `getPaletteFromRegion(regionName: string) → object` | Palette of a specific region. |
| `getAllPalettes` | `getAllPalettes() → {default: string[], regions: object}` | All palettes organized by region. |

#### Block Iteration

| Method | Signature | Description |
|--------|-----------|-------------|
| `blocks` | `blocks() → Array<{x, y, z, name, properties}>` | Iterate all non-air blocks with full data. |
| `blocksIndices` | `blocksIndices() → Array` | All blocks as palette indices (compact). |
| `getOptimizationInfo` | `getOptimizationInfo() → {totalBlocks, nonAirBlocks, paletteSize, compressionRatio}` | Statistics about the schematic. |

#### Chunk Iteration

Divide the schematic into spatial chunks for streaming or LOD rendering.

| Method | Signature | Description |
|--------|-----------|-------------|
| `chunks` | `chunks(chunkWidth: number, chunkHeight: number, chunkLength: number) → Array` | Get all chunks with full block data. |
| `chunksWithStrategy` | `chunksWithStrategy(chunkWidth: number, chunkHeight: number, chunkLength: number, strategy: string, cameraX: number, cameraY: number, cameraZ: number) → Array` | Chunks ordered by a loading strategy. |
| `getChunkBlocks` | `getChunkBlocks(offsetX: number, offsetY: number, offsetZ: number, width: number, height: number, length: number) → Array` | Get blocks in a specific chunk region. |
| `chunksIndices` | `chunksIndices(chunkWidth: number, chunkHeight: number, chunkLength: number) → Array` | Chunks with palette indices (compact). |
| `chunksIndicesWithStrategy` | `chunksIndicesWithStrategy(chunkWidth: number, chunkHeight: number, chunkLength: number, strategy: string, cameraX: number, cameraY: number, cameraZ: number) → Array` | Indexed chunks with loading strategy. |
| `getChunkBlocksIndices` | `getChunkBlocksIndices(offsetX: number, offsetY: number, offsetZ: number, width: number, height: number, length: number) → Array` | Block indices in a chunk region. |
| `getChunkData` | `getChunkData(chunkX: number, chunkY: number, chunkZ: number, chunkWidth: number, chunkHeight: number, chunkLength: number) → object` | Optimized chunk data with blocks and entities. |
| `createLazyChunkIterator` | `createLazyChunkIterator(chunkWidth: number, chunkHeight: number, chunkLength: number, strategy: string, cameraX: number, cameraY: number, cameraZ: number) → LazyChunkIterator` | Create a lazy iterator that generates chunks on demand. |

**Loading strategies:** `"distance_to_camera"`, `"top_down"`, `"bottom_up"`, `"center_outward"`, `"random"`

#### Sign Text & Insign

| Method | Signature | Description |
|--------|-----------|-------------|
| `extractSigns` | `extractSigns() → Array<{pos: number[], text: string}>` | Extract all sign text from the schematic. |
| `compileInsign` | `compileInsign() → object` | Compile Insign annotations embedded in signs into structured metadata. |

#### Definition Regions

Definition regions are logical named volumes used for circuit I/O, spatial queries, and metadata.

| Method | Signature | Description |
|--------|-----------|-------------|
| `addDefinitionRegion` | `addDefinitionRegion(name: string, region: DefinitionRegionWrapper) → void` | Add a pre-built definition region. |
| `getDefinitionRegion` | `getDefinitionRegion(name: string) → DefinitionRegionWrapper` | Get a definition region by name. |
| `removeDefinitionRegion` | `removeDefinitionRegion(name: string) → boolean` | Remove a definition region. |
| `getDefinitionRegionNames` | `getDefinitionRegionNames() → string[]` | List all definition region names. |
| `createDefinitionRegion` | `createDefinitionRegion(name: string) → void` | Create an empty definition region. |
| `createDefinitionRegionFromPoint` | `createDefinitionRegionFromPoint(name: string, x: number, y: number, z: number) → void` | Create a single-point region. |
| `createDefinitionRegionFromBounds` | `createDefinitionRegionFromBounds(name: string, min: BlockPosition, max: BlockPosition) → void` | Create a region from a bounding box. |
| `createRegion` | `createRegion(name: string, min: [number, number, number], max: [number, number, number]) → DefinitionRegionWrapper` | Create and return a region. |
| `updateRegion` | `updateRegion(name: string, region: DefinitionRegionWrapper) → void` | Update an existing region. |
| `definitionRegionAddBounds` | `definitionRegionAddBounds(name: string, min: BlockPosition, max: BlockPosition) → void` | Add bounds to an existing region. |
| `definitionRegionAddPoint` | `definitionRegionAddPoint(name: string, x: number, y: number, z: number) → void` | Add a point to an existing region. |
| `definitionRegionSetMetadata` | `definitionRegionSetMetadata(name: string, key: string, value: string) → void` | Set metadata on a region. |
| `definitionRegionShift` | `definitionRegionShift(name: string, x: number, y: number, z: number) → void` | Shift a region by a delta. |

#### Debug

| Method | Signature | Description |
|--------|-----------|-------------|
| `printSchematic` | `printSchematic() → string` | Formatted text layout of the schematic. |
| `debugInfo` | `debugInfo() → string` | Structured debug information. |

---

### BlockStateWrapper

Represents a Minecraft block with its name and properties (e.g., `minecraft:oak_stairs[facing=north,half=bottom]`).

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new(name: string) → BlockStateWrapper` | Create a block state with no properties. |
| `withProperty` | `withProperty(key: string, value: string) → void` | Add a property (mutates in place). |
| `name` | `name() → string` | Get the block name. |
| `properties` | `properties() → object` | Get properties as a JS object `{key: value, ...}`. |

---

### SchematicBuilderWrapper

A fluent ASCII art builder for constructing schematics layer-by-layer using character-to-block mappings.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new() → SchematicBuilderWrapper` | Create a new builder. |
| `name` | `name(name: string) → SchematicBuilderWrapper` | Set the schematic name (chainable). |
| `map` | `map(ch: string, block: string) → SchematicBuilderWrapper` | Map a character to a block type (chainable). |
| `layers` | `layers(layers: string[][][]) → SchematicBuilderWrapper` | Set the 3D layer array. Each layer is a 2D array of strings. (chainable). |
| `build` | `build() → SchematicWrapper` | Build the final schematic. |
| `fromTemplate` | `static fromTemplate(template: string) → SchematicBuilderWrapper` | Create from a named template. |

---

### DefinitionRegionWrapper

A logical volume composed of one or more axis-aligned bounding boxes. Used to define circuit I/O regions, spatial selections, and metadata-bearing zones. Supports both mutating and immutable operations.

#### Constructors

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new() → DefinitionRegionWrapper` | Create an empty region. |
| `fromBounds` | `static fromBounds(min: BlockPosition, max: BlockPosition) → DefinitionRegionWrapper` | Create from a single bounding box. |
| `fromBoundingBoxes` | `static fromBoundingBoxes(boxes: Array<{min: number[], max: number[]}>) → DefinitionRegionWrapper` | Create from multiple bounding boxes. |
| `fromPositions` | `static fromPositions(positions: Array<number[]>) → DefinitionRegionWrapper` | Create from individual positions (auto-merged). |

#### Mutating Operations

These methods modify the region in place and return `this` for chaining.

| Method | Signature | Description |
|--------|-----------|-------------|
| `addBounds` | `addBounds(min: [number, number, number], max: [number, number, number]) → DefinitionRegionWrapper` | Add a bounding box. |
| `addPoint` | `addPoint(x: number, y: number, z: number) → DefinitionRegionWrapper` | Add a single point. |
| `addFilter` | `addFilter(filter: string) → DefinitionRegionWrapper` | Add a block filter. |
| `excludeBlock` | `excludeBlock(blockName: string) → DefinitionRegionWrapper` | Exclude positions containing a specific block. |
| `setMetadata` | `setMetadata(key: string, value: string) → DefinitionRegionWrapper` | Set a metadata key-value pair. |
| `setColor` | `setColor(color: number) → DefinitionRegionWrapper` | Set visualization color (ARGB). |
| `merge` | `merge(other: DefinitionRegionWrapper) → DefinitionRegionWrapper` | Merge another region into this one. |
| `shift` | `shift(x: number, y: number, z: number) → DefinitionRegionWrapper` | Translate the region. |
| `expand` | `expand(x: number, y: number, z: number) → DefinitionRegionWrapper` | Expand bounds by the given amounts. |
| `contract` | `contract(amount: number) → DefinitionRegionWrapper` | Contract bounds inward. |
| `simplify` | `simplify() → DefinitionRegionWrapper` | Merge overlapping bounding boxes. |
| `subtract` | `subtract(other: DefinitionRegionWrapper) → DefinitionRegionWrapper` | Remove points present in `other`. |
| `intersect` | `intersect(other: DefinitionRegionWrapper) → DefinitionRegionWrapper` | Keep only points in both regions. |
| `unionInto` | `unionInto(other: DefinitionRegionWrapper) → DefinitionRegionWrapper` | Union in place. |

#### Immutable Operations

These return new `DefinitionRegionWrapper` instances without modifying the original.

| Method | Signature | Description |
|--------|-----------|-------------|
| `union` | `union(other: DefinitionRegionWrapper) → DefinitionRegionWrapper` | Return the union. |
| `subtracted` | `subtracted(other: DefinitionRegionWrapper) → DefinitionRegionWrapper` | Return the difference. |
| `intersected` | `intersected(other: DefinitionRegionWrapper) → DefinitionRegionWrapper` | Return the intersection. |
| `shifted` | `shifted(x: number, y: number, z: number) → DefinitionRegionWrapper` | Return a shifted copy. |
| `expanded` | `expanded(x: number, y: number, z: number) → DefinitionRegionWrapper` | Return an expanded copy. |
| `contracted` | `contracted(amount: number) → DefinitionRegionWrapper` | Return a contracted copy. |
| `copy` / `clone` | `copy() → DefinitionRegionWrapper` | Deep copy. |

#### Querying

| Method | Signature | Description |
|--------|-----------|-------------|
| `isEmpty` | `isEmpty() → boolean` | Check if the region contains any points. |
| `contains` | `contains(x: number, y: number, z: number) → boolean` | Test point membership. |
| `volume` | `volume() → number` | Total voxel count. |
| `isContiguous` | `isContiguous() → boolean` | Whether all positions form a single connected component. |
| `connectedComponents` | `connectedComponents() → number` | Number of connected components. |
| `boxCount` | `boxCount() → number` | Number of bounding boxes. |
| `intersectsBounds` | `intersectsBounds(minX: number, minY: number, minZ: number, maxX: number, maxY: number, maxZ: number) → boolean` | Test intersection with a bounding box. |

#### Filtering

| Method | Signature | Description |
|--------|-----------|-------------|
| `filterByBlock` | `filterByBlock(schematic: SchematicWrapper, blockName: string) → DefinitionRegionWrapper` | Keep only positions containing a specific block. |
| `filterByProperties` | `filterByProperties(schematic: SchematicWrapper, properties: object) → DefinitionRegionWrapper` | Keep only positions matching block properties. |

#### Data Accessors

| Method | Signature | Description |
|--------|-----------|-------------|
| `positions` | `positions() → Array<number[]>` | All positions as `[[x,y,z], ...]`. |
| `positionsSorted` | `positionsSorted() → Array<number[]>` | Positions in deterministic Y, X, Z order (used for bit assignment). |
| `getBounds` | `getBounds() → {min: number[], max: number[]} \| null` | Overall bounding box. |
| `getBox` | `getBox(index: number) → object \| null` | Get a specific bounding box by index. |
| `getBoxes` | `getBoxes() → Array` | All bounding boxes. |
| `dimensions` | `dimensions() → number[]` | `[width, height, length]`. |
| `center` | `center() → number[] \| null` | Integer center point. |
| `centerF32` | `centerF32() → number[] \| null` | Float center point. |
| `getMetadata` | `getMetadata(key: string) → string \| null` | Get a metadata value. |
| `getAllMetadata` | `getAllMetadata() → object` | All metadata as `{key: value, ...}`. |
| `metadataKeys` | `metadataKeys() → string[]` | All metadata keys. |
| `getBlocks` | `getBlocks() → Array` | All block data within the region. |

---

### PaletteManager

Static utility for getting predefined Minecraft block palettes.

| Method | Signature | Description |
|--------|-----------|-------------|
| `getWoolBlocks` | `static getWoolBlocks() → string[]` | All 16 wool block names. |
| `getConcreteBlocks` | `static getConcreteBlocks() → string[]` | All 16 concrete block names. |
| `getTerracottaBlocks` | `static getTerracottaBlocks() → string[]` | All 16 terracotta block names. |
| `getPaletteByKeywords` | `static getPaletteByKeywords(keywords: string[]) → string[]` | Get blocks matching any of the given keywords. |

---

### LazyChunkIterator

An on-demand chunk iterator that generates mesh data one chunk at a time, reducing peak memory usage.

| Method | Signature | Description |
|--------|-----------|-------------|
| `next` | `next() → object \| null` | Get the next chunk, or null if exhausted. |
| `hasNext` | `hasNext() → boolean` | Whether more chunks remain. |
| `totalChunks` | `totalChunks() → number` | Total number of chunks. |
| `currentPosition` | `currentPosition() → number` | Current iterator position. |
| `reset` | `reset() → void` | Reset to the beginning. |
| `skipTo` | `skipTo(index: number) → void` | Jump to a specific index. |

---

## Building

### ShapeWrapper

Geometric shape primitives for use with the building tool.

| Method | Signature | Description |
|--------|-----------|-------------|
| `sphere` | `static sphere(cx: number, cy: number, cz: number, radius: number) → ShapeWrapper` | Create a sphere. |
| `cuboid` | `static cuboid(minX: number, minY: number, minZ: number, maxX: number, maxY: number, maxZ: number) → ShapeWrapper` | Create an axis-aligned cuboid. |

---

### BrushWrapper

Block-filling patterns for painting shapes. RGB color brushes automatically map to the closest Minecraft block.

| Method | Signature | Description |
|--------|-----------|-------------|
| `solid` | `static solid(blockState: string) → BrushWrapper` | A single solid block type. |
| `color` | `static color(r: number, g: number, b: number, paletteFilter?: string[]) → BrushWrapper` | Match the closest block to an RGB color. Optional palette filter limits the search space. |
| `linearGradient` | `static linearGradient(x1, y1, z1, r1, g1, b1, x2, y2, z2, r2, g2, b2, space?, paletteFilter?) → BrushWrapper` | Linear gradient between two colored points. `space`: 0=RGB (default), 1=Oklab. |
| `shaded` | `static shaded(r, g, b, lx, ly, lz, paletteFilter?) → BrushWrapper` | Lambertian shading with a directional light. |
| `bilinearGradient` | `static bilinearGradient(ox, oy, oz, ux, uy, uz, vx, vy, vz, r00, g00, b00, r10, g10, b10, r01, g01, b01, r11, g11, b11, space?, paletteFilter?) → BrushWrapper` | 4-corner quad gradient over two axes (origin + U axis + V axis). |
| `pointGradient` | `static pointGradient(positions: number[], colors: number[], falloff?, space?, paletteFilter?) → BrushWrapper` | Inverse distance weighted gradient from multiple colored points. `positions` is flat `[x,y,z,...]`, `colors` is flat `[r,g,b,...]`. Default falloff: 2.0. |

---

### WasmBuildingTool

Applies a brush to a shape within a schematic.

| Method | Signature | Description |
|--------|-----------|-------------|
| `fill` | `static fill(schematic: SchematicWrapper, shape: ShapeWrapper, brush: BrushWrapper) → void` | Fill the shape with the brush pattern in the schematic. |

---

## Simulation (feature-gated)

> Requires the `simulation` feature flag at compile time. Provides full MCHPRS-based redstone simulation.

### SimulationOptionsWrapper

Configuration for the simulation world.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new() → SimulationOptionsWrapper` | Create with defaults. |
| `optimize` | `get/set optimize: boolean` | Enable redstone graph optimization. |
| `io_only` | `get/set io_only: boolean` | Track only IO-relevant blocks. |
| `addCustomIo` | `addCustomIo(x: number, y: number, z: number) → void` | Register a custom IO position. |
| `clearCustomIo` | `clearCustomIo() → void` | Clear all custom IO positions. |

---

### MchprsWorldWrapper

A live redstone simulation world. Create from a schematic, toggle levers, advance ticks, and read output states.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new(schematic: SchematicWrapper) → MchprsWorldWrapper` | Create simulation from schematic. |
| `with_options` | `static with_options(schematic: SchematicWrapper, options: SimulationOptionsWrapper) → MchprsWorldWrapper` | Create with custom options. |
| `onUseBlock` | `onUseBlock(x: number, y: number, z: number) → void` | Right-click a block (toggle lever, press button). |
| `tick` | `tick(ticks: number) → void` | Advance the simulation by N ticks. |
| `flush` | `flush() → void` | Propagate pending redstone changes. |
| `isLit` | `isLit(x: number, y: number, z: number) → boolean` | Check if a redstone lamp is powered. |
| `getLeverPower` | `getLeverPower(x: number, y: number, z: number) → boolean` | Check if a lever is on. |
| `getRedstonePower` | `getRedstonePower(x: number, y: number, z: number) → number` | Get redstone power level (0-15). |
| `setSignalStrength` | `setSignalStrength(x: number, y: number, z: number, strength: number) → void` | Set custom IO signal strength (0-15). |
| `getSignalStrength` | `getSignalStrength(x: number, y: number, z: number) → number` | Get custom IO signal strength. |
| `checkCustomIoChanges` | `checkCustomIoChanges() → void` | Detect and queue IO state changes. |
| `pollCustomIoChanges` | `pollCustomIoChanges() → Array<{x, y, z, oldPower, newPower}>` | Get and clear queued IO changes. |
| `peekCustomIoChanges` | `peekCustomIoChanges() → Array<{x, y, z, oldPower, newPower}>` | Get IO changes without clearing. |
| `clearCustomIoChanges` | `clearCustomIoChanges() → void` | Clear the change queue. |
| `getTruthTable` | `getTruthTable() → object` | Generate a truth table for the circuit. |
| `syncToSchematic` | `syncToSchematic() → void` | Write simulation state back to the schematic. |
| `getSchematic` | `getSchematic() → SchematicWrapper` | Get a copy of the current schematic state. |
| `intoSchematic` | `intoSchematic() → SchematicWrapper` | Consume the world and return the schematic. |

---

### CircuitBuilderWrapper

Fluent builder for creating typed circuit executors. Defines inputs, outputs, and their data types.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new(schematic: SchematicWrapper) → CircuitBuilderWrapper` | Create from a schematic. |
| `fromInsign` | `static fromInsign(schematic: SchematicWrapper) → CircuitBuilderWrapper` | Create from Insign annotations in signs. |
| `withInput` | `withInput(name: string, ioType: IoTypeWrapper, layout: LayoutFunctionWrapper, region: DefinitionRegionWrapper) → CircuitBuilderWrapper` | Add a typed input. |
| `withInputSorted` | `withInputSorted(name: string, ioType: IoTypeWrapper, layout: LayoutFunctionWrapper, region: DefinitionRegionWrapper, sort: SortStrategyWrapper) → CircuitBuilderWrapper` | Add input with custom sort order. |
| `withInputAuto` | `withInputAuto(name: string, ioType: IoTypeWrapper, region: DefinitionRegionWrapper) → CircuitBuilderWrapper` | Add input with auto-inferred layout. |
| `withInputAutoSorted` | `withInputAutoSorted(name: string, ioType: IoTypeWrapper, region: DefinitionRegionWrapper, sort: SortStrategyWrapper) → CircuitBuilderWrapper` | Add input with auto layout and custom sort. |
| `withOutput` | `withOutput(name: string, ioType: IoTypeWrapper, layout: LayoutFunctionWrapper, region: DefinitionRegionWrapper) → CircuitBuilderWrapper` | Add a typed output. |
| `withOutputSorted` | `withOutputSorted(name: string, ioType: IoTypeWrapper, layout: LayoutFunctionWrapper, region: DefinitionRegionWrapper, sort: SortStrategyWrapper) → CircuitBuilderWrapper` | Add output with custom sort. |
| `withOutputAuto` | `withOutputAuto(name: string, ioType: IoTypeWrapper, region: DefinitionRegionWrapper) → CircuitBuilderWrapper` | Add output with auto layout. |
| `withOutputAutoSorted` | `withOutputAutoSorted(name: string, ioType: IoTypeWrapper, region: DefinitionRegionWrapper, sort: SortStrategyWrapper) → CircuitBuilderWrapper` | Add output with auto layout and custom sort. |
| `withOptions` | `withOptions(options: SimulationOptionsWrapper) → CircuitBuilderWrapper` | Set simulation options. |
| `withStateMode` | `withStateMode(mode: string) → CircuitBuilderWrapper` | Set state mode: `"stateless"`, `"stateful"`, or `"manual"`. |
| `validate` | `validate() → void` | Validate the builder configuration (throws on error). |
| `build` | `build() → TypedCircuitExecutorWrapper` | Build the executor. |
| `buildValidated` | `buildValidated() → TypedCircuitExecutorWrapper` | Validate and build. |
| `inputCount` / `outputCount` | `inputCount() → number` | Number of configured inputs/outputs. |
| `inputNames` / `outputNames` | `inputNames() → string[]` | Names of configured inputs/outputs. |

---

### TypedCircuitExecutorWrapper

Executes a redstone circuit with typed inputs and outputs. Supports automated and manual execution modes.

| Method | Signature | Description |
|--------|-----------|-------------|
| `fromLayout` | `static fromLayout(world: MchprsWorldWrapper, layout: IoLayoutWrapper) → TypedCircuitExecutorWrapper` | Create from a world and IO layout. |
| `fromLayoutWithOptions` | `static fromLayoutWithOptions(world: MchprsWorldWrapper, layout: IoLayoutWrapper, options: SimulationOptionsWrapper) → TypedCircuitExecutorWrapper` | Create with custom options. |
| `fromInsign` | `static fromInsign(schematic: SchematicWrapper) → TypedCircuitExecutorWrapper` | Create from Insign annotations. |
| `fromInsignWithOptions` | `static fromInsignWithOptions(schematic: SchematicWrapper, options: SimulationOptionsWrapper) → TypedCircuitExecutorWrapper` | Create from Insign with options. |
| `setStateMode` | `setStateMode(mode: string) → void` | Set mode: `"stateless"` (reset between runs), `"stateful"` (preserve state), `"manual"` (full control). |
| `reset` | `reset() → void` | Reset circuit state. |
| `execute` | `execute(inputs: object, mode: ExecutionModeWrapper) → {outputs: object, ticksElapsed: number, conditionMet: boolean}` | Run circuit with typed inputs and an execution mode. Returns typed outputs. |
| `run` | `run(inputs: object, limit: number, mode: string) → object` | Simplified execution. `mode`: `"fixed"` or `"stable"`. |
| `tick` | `tick(ticks: number) → void` | Manual: advance N ticks. |
| `flush` | `flush() → void` | Manual: propagate changes. |
| `setInput` | `setInput(name: string, value: ValueWrapper) → void` | Manual: set an input value. |
| `readOutput` | `readOutput(name: string) → ValueWrapper` | Manual: read an output value. |
| `inputNames` / `outputNames` | `inputNames() → string[]` | List input/output names. |
| `syncToSchematic` | `syncToSchematic() → SchematicWrapper` | Sync state and return schematic. |
| `getLayoutInfo` | `getLayoutInfo() → object` | Detailed layout with bit positions. |

---

### IoLayoutBuilderWrapper

Builder for creating IO layouts that map typed data to physical redstone positions.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new() → IoLayoutBuilderWrapper` | Create a new builder. |
| `addInput` | `addInput(name: string, ioType: IoTypeWrapper, layout: LayoutFunctionWrapper, positions: Array) → IoLayoutBuilderWrapper` | Add an input with explicit positions. |
| `addInputAuto` | `addInputAuto(name: string, ioType: IoTypeWrapper, positions: Array) → IoLayoutBuilderWrapper` | Add input with auto layout. |
| `addInputRegion` / `addInputRegionAuto` | `addInputRegion(name: string, ioType: IoTypeWrapper, layout: LayoutFunctionWrapper, min: BlockPosition, max: BlockPosition) → IoLayoutBuilderWrapper` | Add input from a bounding box region. |
| `addInputFromRegion` / `addInputFromRegionAuto` | `addInputFromRegion(name: string, ioType: IoTypeWrapper, layout: LayoutFunctionWrapper, region: DefinitionRegionWrapper) → IoLayoutBuilderWrapper` | Add input from a definition region. |
| `addOutput` / `addOutputAuto` / `addOutputRegion` / `addOutputRegionAuto` / `addOutputFromRegion` / `addOutputFromRegionAuto` | *(same patterns as input methods)* | Add outputs with the same API variants. |
| `build` | `build() → IoLayoutWrapper` | Finalize the layout. |

---

### IoLayoutWrapper

A finalized IO layout. Used with `TypedCircuitExecutorWrapper.fromLayout()`.

| Method | Signature | Description |
|--------|-----------|-------------|
| `inputNames` | `inputNames() → string[]` | List input names. |
| `outputNames` | `outputNames() → string[]` | List output names. |

---

### ValueWrapper

A typed value for circuit I/O. Supports unsigned integers, signed integers, floats, booleans, and strings.

| Method | Signature | Description |
|--------|-----------|-------------|
| `fromU32` | `static fromU32(value: number) → ValueWrapper` | Create unsigned 32-bit integer. |
| `fromI32` | `static fromI32(value: number) → ValueWrapper` | Create signed 32-bit integer. |
| `fromF32` | `static fromF32(value: number) → ValueWrapper` | Create 32-bit float. |
| `fromBool` | `static fromBool(value: boolean) → ValueWrapper` | Create boolean. |
| `fromString` | `static fromString(value: string) → ValueWrapper` | Create string. |
| `toJs` | `toJs() → any` | Convert to a native JS value. |
| `typeName` | `typeName() → string` | Get the type name (`"u32"`, `"i32"`, `"f32"`, `"bool"`, `"string"`). |

---

### IoTypeWrapper

Defines the data type for a circuit input or output.

| Method | Signature | Description |
|--------|-----------|-------------|
| `unsignedInt` | `static unsignedInt(bits: number) → IoTypeWrapper` | N-bit unsigned integer. |
| `signedInt` | `static signedInt(bits: number) → IoTypeWrapper` | N-bit signed integer. |
| `float32` | `static float32() → IoTypeWrapper` | 32-bit IEEE 754 float. |
| `boolean` | `static boolean() → IoTypeWrapper` | Single-bit boolean. |
| `ascii` | `static ascii(chars: number) → IoTypeWrapper` | ASCII string of N characters (7 bits per char). |

---

### LayoutFunctionWrapper

Defines how bits are mapped to physical redstone positions.

| Method | Signature | Description |
|--------|-----------|-------------|
| `oneToOne` | `static oneToOne() → LayoutFunctionWrapper` | 1 bit per position (default). |
| `packed4` | `static packed4() → LayoutFunctionWrapper` | 4 bits per position (signal strength 0-15). |
| `custom` | `static custom(mapping: number[]) → LayoutFunctionWrapper` | Custom bit-to-position mapping. |
| `rowMajor` | `static rowMajor(rows: number, cols: number, bitsPerElement: number) → LayoutFunctionWrapper` | Row-major 2D layout. |
| `columnMajor` | `static columnMajor(rows: number, cols: number, bitsPerElement: number) → LayoutFunctionWrapper` | Column-major 2D layout. |
| `scanline` | `static scanline(width: number, height: number, bitsPerPixel: number) → LayoutFunctionWrapper` | Image scanline layout. |

---

### ExecutionModeWrapper

Controls how the circuit executor runs.

| Method | Signature | Description |
|--------|-----------|-------------|
| `fixedTicks` | `static fixedTicks(ticks: number) → ExecutionModeWrapper` | Run for exactly N ticks. |
| `untilCondition` | `static untilCondition(outputName: string, condition: OutputConditionWrapper, maxTicks: number, checkInterval: number) → ExecutionModeWrapper` | Run until an output meets a condition. |
| `untilChange` | `static untilChange(maxTicks: number, checkInterval: number) → ExecutionModeWrapper` | Run until any output changes. |
| `untilStable` | `static untilStable(stableTicks: number, maxTicks: number) → ExecutionModeWrapper` | Run until outputs are stable for N ticks. |

---

### OutputConditionWrapper

A predicate for use with `ExecutionModeWrapper.untilCondition()`.

| Method | Signature | Description |
|--------|-----------|-------------|
| `equals` | `static equals(value: ValueWrapper) → OutputConditionWrapper` | Output equals value. |
| `notEquals` | `static notEquals(value: ValueWrapper) → OutputConditionWrapper` | Output does not equal value. |
| `greaterThan` | `static greaterThan(value: ValueWrapper) → OutputConditionWrapper` | Output is greater than value. |
| `lessThan` | `static lessThan(value: ValueWrapper) → OutputConditionWrapper` | Output is less than value. |
| `bitwiseAnd` | `static bitwiseAnd(mask: number) → OutputConditionWrapper` | Output AND mask is non-zero. |

---

### SortStrategyWrapper

Controls the order in which physical positions are assigned to bits in IO layouts.

| Method | Signature | Description |
|--------|-----------|-------------|
| `yxz` | `static yxz() → SortStrategyWrapper` | Sort by Y, then X, then Z (default). |
| `xyz` | `static xyz() → SortStrategyWrapper` | Sort by X, then Y, then Z. |
| `zyx` | `static zyx() → SortStrategyWrapper` | Sort by Z, then Y, then X. |
| `yDescXZ` | `static yDescXZ() → SortStrategyWrapper` | Y descending, then X, then Z. |
| `xDescYZ` | `static xDescYZ() → SortStrategyWrapper` | X descending, then Y, then Z. |
| `zDescYX` | `static zDescYX() → SortStrategyWrapper` | Z descending, then Y, then X. |
| `descending` | `static descending() → SortStrategyWrapper` | All axes descending. |
| `distanceFrom` | `static distanceFrom(x: number, y: number, z: number) → SortStrategyWrapper` | Sort by distance from a point (ascending). |
| `distanceFromDesc` | `static distanceFromDesc(x: number, y: number, z: number) → SortStrategyWrapper` | Sort by distance from a point (descending). |
| `preserve` | `static preserve() → SortStrategyWrapper` | Keep original order. |
| `reverse` | `static reverse() → SortStrategyWrapper` | Reverse original order. |
| `fromString` | `static fromString(s: string) → SortStrategyWrapper` | Parse from string. |
| `name` | `get name: string` | Get the strategy name. |

---

### StateModeConstants

String constants for state modes.

| Property | Value | Description |
|----------|-------|-------------|
| `STATELESS` | `"stateless"` | Reset between each execution. |
| `STATEFUL` | `"stateful"` | Preserve state between executions. |
| `MANUAL` | `"manual"` | Full manual control of ticks and I/O. |

---

## Meshing (feature-gated)

> Requires the `meshing` feature flag. Generates 3D mesh data from schematics using Minecraft resource packs.

### ResourcePackWrapper

Loads and provides access to Minecraft resource pack assets (blockstate definitions, models, textures).

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new(data: Uint8Array) → ResourcePackWrapper` | Load from ZIP bytes. |
| `blockstateCount` | `get blockstateCount: number` | Number of blockstate definitions. |
| `modelCount` | `get modelCount: number` | Number of block models. |
| `textureCount` | `get textureCount: number` | Number of textures. |
| `namespaces` | `get namespaces: string[]` | Resource pack namespaces. |
| `listBlockstates` | `listBlockstates() → string[]` | All blockstate names. |
| `listModels` | `listModels() → string[]` | All model names. |
| `listTextures` | `listTextures() → string[]` | All texture names. |
| `getBlockstateJson` | `getBlockstateJson(name: string) → string \| undefined` | Get raw blockstate JSON. |
| `getModelJson` | `getModelJson(name: string) → string \| undefined` | Get raw model JSON. |
| `getTextureInfo` | `getTextureInfo(name: string) → {width, height, isAnimated, frameCount}` | Get texture metadata. |
| `getTexturePixels` | `getTexturePixels(name: string) → Uint8Array \| undefined` | Get raw RGBA8 pixel data. |
| `addBlockstateJson` | `addBlockstateJson(name: string, json: string) → void` | Add/override a blockstate definition. |
| `addModelJson` | `addModelJson(name: string, json: string) → void` | Add/override a model definition. |
| `addTexture` | `addTexture(name: string, width: number, height: number, pixels: Uint8Array) → void` | Add a custom texture (RGBA8). |
| `getStats` | `getStats() → object` | Summary statistics. |

---

### MeshConfigWrapper

Configuration for mesh generation.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `new() → MeshConfigWrapper` | Create with sensible defaults. |
| `cullHiddenFaces` | `get/set cullHiddenFaces: boolean` | Remove faces between adjacent solid blocks (default: true). |
| `ambientOcclusion` | `get/set ambientOcclusion: boolean` | Enable AO darkening in corners (default: true). |
| `aoIntensity` | `get/set aoIntensity: number` | AO darkening strength, 0.0-1.0 (default: 0.4). |
| `biome` | `get/set biome: string \| undefined` | Biome name for tinting grass/water/foliage. |
| `atlasMaxSize` | `get/set atlasMaxSize: number` | Maximum texture atlas dimension in pixels (default: 4096). |
| `cullOccludedBlocks` | `get/set cullOccludedBlocks: boolean` | Skip fully enclosed blocks (default: true). |
| `greedyMeshing` | `get/set greedyMeshing: boolean` | Merge coplanar faces into larger quads (default: false). |

---

### MeshOutputWrapper

The result of meshing a schematic or chunk. Contains vertex data organized into three transparency layers (opaque, cutout, transparent) plus a shared texture atlas.

#### Vertex Data (per layer)

Each layer provides typed arrays suitable for direct GPU upload.

| Method | Returns | Description |
|--------|---------|-------------|
| `opaquePositions` | `Float32Array` | Vertex positions `[x,y,z,...]` for solid blocks. |
| `opaqueNormals` | `Float32Array` | Vertex normals `[nx,ny,nz,...]`. |
| `opaqueUvs` | `Float32Array` | Texture coordinates `[u,v,...]`. |
| `opaqueColors` | `Float32Array` | Vertex colors `[r,g,b,a,...]` (biome tinting, AO). |
| `opaqueIndices` | `Uint32Array` | Triangle indices. |
| `cutoutPositions` / `cutoutNormals` / `cutoutUvs` / `cutoutColors` / `cutoutIndices` | *(same types)* | Data for alpha-tested blocks (leaves, flowers). |
| `transparentPositions` / `transparentNormals` / `transparentUvs` / `transparentColors` / `transparentIndices` | *(same types)* | Data for translucent blocks (glass, water). |

#### Texture Atlas

| Method | Returns | Description |
|--------|---------|-------------|
| `atlasRgba` | `Uint8Array` | RGBA8 atlas pixel data. |
| `atlasWidth` | `number` | Atlas width in pixels. |
| `atlasHeight` | `number` | Atlas height in pixels. |

#### Export

| Method | Signature | Description |
|--------|-----------|-------------|
| `toGlb` | `toGlb() → Uint8Array` | Export as GLB (binary glTF). |
| `toNucm` | `toNucm() → Uint8Array` | Export as NUCM binary cache format. |
| `toUsdz` | `toUsdz() → Uint8Array` | Export as USDZ (Apple AR). |

#### Metadata

| Property | Type | Description |
|----------|------|-------------|
| `totalVertices` | `number` | Total vertex count across all layers. |
| `totalTriangles` | `number` | Total triangle count. |
| `hasTransparency` | `boolean` | Whether transparent layer has data. |
| `isEmpty` | `boolean` | Whether the mesh has no geometry. |
| `lodLevel` | `number` | Level-of-detail (0 = full detail). |
| `chunkCoord` | `number[] \| null` | Chunk coordinates `[cx, cy, cz]` if from chunk meshing. |
| `bounds` | `number[]` | Bounding box `[minX, minY, minZ, maxX, maxY, maxZ]`. |
| `vertexCount` / `triangleCount` | `number` | Backward-compatible aliases. |
| `glbData` | `Uint8Array` | Backward-compatible alias for `toGlb()`. |

---

### MultiMeshResultWrapper

Result of per-region mesh generation (`meshByRegion`). Contains one `MeshOutputWrapper` per named region.

| Method | Signature | Description |
|--------|-----------|-------------|
| `getRegionNames` | `getRegionNames() → string[]` | List region names. |
| `getMesh` | `getMesh(regionName: string) → MeshOutputWrapper \| undefined` | Get the mesh for a region. |
| `totalVertexCount` | `get totalVertexCount: number` | Combined vertex count. |
| `totalTriangleCount` | `get totalTriangleCount: number` | Combined triangle count. |
| `meshCount` | `get meshCount: number` | Number of region meshes. |

---

### ChunkMeshResultWrapper

Result of chunk-based mesh generation. Contains one `MeshOutputWrapper` per spatial chunk.

| Method | Signature | Description |
|--------|-----------|-------------|
| `getChunkCoordinates` | `getChunkCoordinates() → number[][]` | All chunk coordinates `[[cx,cy,cz], ...]`. |
| `getMesh` | `getMesh(cx: number, cy: number, cz: number) → MeshOutputWrapper \| undefined` | Get the mesh for a chunk. |
| `toNucm` | `toNucm() → Uint8Array` | Export all chunks as NUCM. |
| `totalVertexCount` | `get totalVertexCount: number` | Combined vertex count. |
| `totalTriangleCount` | `get totalTriangleCount: number` | Combined triangle count. |
| `chunkCount` | `get chunkCount: number` | Number of chunk meshes. |

---

### ChunkMeshIteratorWrapper

Streaming iterator for generating chunk meshes one at a time, with progress callbacks.

| Method | Signature | Description |
|--------|-----------|-------------|
| `advance` | `advance() → boolean` | Process the next chunk. Returns true if a chunk was produced. |
| `current` | `current() → MeshOutputWrapper \| undefined` | Get the most recently generated chunk mesh. |
| `currentCoord` | `currentCoord() → number[] \| null` | Get the current chunk coordinates. |
| `setProgressCallback` | `setProgressCallback(callback: Function) → void` | Set a progress callback: `{phase, chunksDone, chunksTotal, verticesSoFar, trianglesSoFar}`. |
| `chunkCount` | `get chunkCount: number` | Total number of chunks. |
| `hasSharedAtlas` | `get hasSharedAtlas: boolean` | Whether a shared atlas is available. |
| `sharedAtlas` | `sharedAtlas() → TextureAtlasWrapper \| undefined` | Get the shared texture atlas. |

---

### TextureAtlasWrapper

A pre-built texture atlas that can be shared across multiple chunk meshes for consistency.

| Method | Signature | Description |
|--------|-----------|-------------|
| `width` | `get width: number` | Atlas width in pixels. |
| `height` | `get height: number` | Atlas height in pixels. |
| `toBytes` | `toBytes() → Uint8Array` | Get RGBA8 pixel data. |

---

### RawMeshExportWrapper

Raw mesh data for custom rendering pipelines. Provides flat arrays without GPU-specific formatting.

| Method | Signature | Description |
|--------|-----------|-------------|
| `positionsFlat` | `positionsFlat() → number[]` | Vertex positions `[x,y,z,...]`. |
| `normalsFlat` | `normalsFlat() → number[]` | Vertex normals. |
| `uvsFlat` | `uvsFlat() → number[]` | Texture coordinates. |
| `colorsFlat` | `colorsFlat() → number[]` | Vertex colors. |
| `indices` | `indices() → number[]` | Triangle indices. |
| `textureRgba` | `textureRgba() → number[]` | Texture RGBA data. |
| `textureWidth` | `get textureWidth: number` | Texture width. |
| `textureHeight` | `get textureHeight: number` | Texture height. |
| `vertexCount` | `get vertexCount: number` | Total vertices. |
| `triangleCount` | `get triangleCount: number` | Total triangles. |

#### Meshing Methods on SchematicWrapper

These methods are available on `SchematicWrapper` when the `meshing` feature is enabled.

| Method | Signature | Description |
|--------|-----------|-------------|
| `toMesh` | `toMesh(pack: ResourcePackWrapper, config: MeshConfigWrapper) → MeshOutputWrapper` | Generate a single mesh for the entire schematic. |
| `meshByRegion` | `meshByRegion(pack: ResourcePackWrapper, config: MeshConfigWrapper) → MultiMeshResultWrapper` | Generate one mesh per named region. |
| `meshByChunk` | `meshByChunk(pack: ResourcePackWrapper, config: MeshConfigWrapper) → ChunkMeshResultWrapper` | Generate meshes by 16x16x16 chunks. |
| `meshByChunkSize` | `meshByChunkSize(pack: ResourcePackWrapper, config: MeshConfigWrapper, chunkSize: number) → ChunkMeshResultWrapper` | Generate meshes by custom chunk size. |
| `chunkMeshIterator` | `chunkMeshIterator(pack: ResourcePackWrapper, config: MeshConfigWrapper, chunkSize: number) → ChunkMeshIteratorWrapper` | Create a streaming chunk mesh iterator. |
| `buildGlobalAtlas` | `buildGlobalAtlas(pack: ResourcePackWrapper, config: MeshConfigWrapper) → TextureAtlasWrapper` | Pre-build a shared texture atlas. |
| `chunkMeshIteratorWithAtlas` | `chunkMeshIteratorWithAtlas(pack: ResourcePackWrapper, config: MeshConfigWrapper, chunkSize: number, atlas: TextureAtlasWrapper) → ChunkMeshIteratorWrapper` | Create iterator with a pre-built atlas. |
| `toUsdz` | `toUsdz(pack: ResourcePackWrapper, config: MeshConfigWrapper) → MeshOutputWrapper` | Generate mesh and export as USDZ. |
| `toRawMesh` | `toRawMesh(pack: ResourcePackWrapper, config: MeshConfigWrapper) → RawMeshExportWrapper` | Generate raw mesh data. |
| `registerMeshExporter` | `registerMeshExporter(pack: ResourcePackWrapper) → void` | Register mesh export as a `saveAs` format. |

---

## Module Functions

Top-level functions available at the module level.

| Function | Signature | Description |
|----------|-----------|-------------|
| `start` | `start() → void` | Initialize the WASM module (called automatically). |
| `debug_schematic` | `debug_schematic(schematic: SchematicWrapper) → string` | Formatted debug output. |
| `debug_json_schematic` | `debug_json_schematic(schematic: SchematicWrapper) → string` | JSON-formatted debug output. |
