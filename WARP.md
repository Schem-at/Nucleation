# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

Nucleation is a high-performance Minecraft schematic engine written in Rust with multi-platform support:
- **Core**: Rust library with memory-safe zero-copy deserialization
- **WASM**: JavaScript/TypeScript bindings for browsers and Node.js
- **Python**: Native Python bindings via PyO3/Maturin
- **FFI**: C-compatible interface for PHP, C, Go, and other languages

Key formats supported: `.litematic`, `.schematic` (multiple versions), `.nbt`

## Essential Commands

### Building

```bash
# Build Rust core (default features)
cargo build --release

# Build with specific features
cargo build --release --features ffi
cargo build --release --features python
cargo build --release --features wasm
cargo build --release --features simulation

# Combine features (e.g., WASM with simulation)
cargo build --release --features wasm,simulation

# Build WASM module (includes package.json setup)
./build-wasm.sh

# Build Python bindings locally
maturin develop --features python

# Build FFI libs (uses Docker)
./build-ffi.sh
```

### Testing

```bash
# Run Rust tests (default features only)
cargo test

# Run WASM tests (installs wasm-pack if needed)
./test-wasm.sh

# Run benchmarks
cargo bench --bench performance_test
```

### Linting & Formatting

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-features
```

### Publishing

The CI/CD pipeline (`ci.yml`) auto-publishes when `Cargo.toml` version changes on main/master:
- crates.io (Rust)
- npm (WASM)
- PyPI (Python)
- GitHub Releases (native libraries)

## Architecture

### Core Design

**UniversalSchematic** (`src/universal_schematic.rs`) is the central type that provides format-agnostic schematic representation:
- Holds one `default_region` (Region) and optional `other_regions` (HashMap)
- Contains `Metadata` for schematic-level info
- Uses a `block_state_cache` for performance optimization

**Region** (`src/region.rs`) represents a 3D voxel volume:
- Uses a **palette system** for memory efficiency: stores block indices, not full BlockStates
- `blocks: Vec<usize>` - flat array of palette indices
- `palette: Vec<BlockState>` - unique block types
- `palette_index: HashMap<BlockState, usize>` - fast palette lookups
- Auto-expands when blocks are set outside current bounds via `expand_to_fit()`
- Uses `BoundingBox` for coordinate transformations

**BlockState** (`src/block_state.rs`):
- Simple struct: `name` (String) + `properties` (HashMap)
- Implements Hash/Eq for use as HashMap keys

**Key utilities**:
- `BoundingBox` (`src/bounding_box.rs`): Efficient coordinate-to-index conversion
- `BlockPosition` (`src/block_position.rs`): 3D coordinate wrapper
- `BlockEntity` (`src/block_entity/mod.rs`): Tile entities with NBT data
- `Entity` (`src/entity.rs`): Non-block entities

### Format Parsers

Located in `src/formats/`:
- **litematic** (`litematic.rs`): Handles Litematica format (compressed NBT)
- **schematic** (`schematic.rs`): Handles WorldEdit/Sponge schematics (multiple versions)

Both provide:
- `is_<format>()` - Format detection
- `from_<format>()` - Parse to UniversalSchematic
- `to_<format>()` - Export from UniversalSchematic

### Multi-Platform Bindings

All bindings wrap `UniversalSchematic` and provide language-specific APIs:

**WASM** (`src/wasm.rs`):
- Uses `wasm-bindgen` for JS interop
- `SchematicWrapper` exposes methods returning JsValue
- Build script creates universal init wrapper for Node.js/browser compatibility

**Python** (`src/python.rs`):
- Uses `pyo3` for Python bindings
- `PySchematic` wraps UniversalSchematic
- Built via `maturin` (config in `pyproject.toml`)

**FFI** (`src/ffi.rs`):
- C-compatible types with `#[repr(C)]`
- Manual memory management functions (`free_*`)
- Opaque pointer pattern for Rust types

**PHP** (`src/php.rs`):
- Uses `ext-php-rs` for native PHP extensions
- Requires `--features php`

**Simulation** (`src/simulation/`):
- MCHPRS integration for redstone circuit simulation
- `mchprs_world.rs`: World wrapper with chunk management
- `truth_table.rs`: Automatic truth table generation
- WASM-compatible with special backend initialization
- Requires `--features simulation`

### Feature Flags

Features are mutually compatible but typically used separately:
- `wasm`: WebAssembly bindings
- `python`: Python bindings (requires pyo3)
- `ffi`: C FFI interface
- `php`: PHP extension interface
- `simulation`: Redstone circuit simulation via MCHPRS (can be combined with wasm/python/ffi)
- Default: No features (Rust library only)

### Performance Patterns

1. **Palette Caching**: Regions maintain `palette_index` HashMap to avoid duplicate BlockStates
2. **Block State Cache**: UniversalSchematic caches frequently-used BlockStates by string
3. **Zero-Copy**: Uses `quartz_nbt` for efficient NBT parsing
4. **Chunk Loading**: `ChunkLoadingStrategy` enum supports different loading patterns
5. **Lazy Iteration**: `LazyChunkIterator` (WASM) avoids loading all chunks at once

### Testing Structure

- `tests/integration_tests.rs`: Format conversion tests
- `tests/wasm_tests.rs`: WASM-specific tests
- `tests/node_wasm_test.js`: Node.js integration tests
- `benches/performance_test.rs`: Criterion benchmarks
- Test files in `tests/samples/` (litematics and schematics)
- Output goes to `tests/output/`

## Development Guidelines

### Adding New Block Operations

When adding operations to `UniversalSchematic`, ensure:
1. Region auto-expansion is handled (via `expand_to_fit`)
2. Palette updates use `get_or_insert_in_palette()`
3. BoundingBox is rebuilt when region changes
4. All platform bindings are updated (WASM, Python, FFI)

### Adding Format Support

New format parsers should:
1. Go in `src/formats/`
2. Export in `src/formats/mod.rs`
3. Follow pattern: `is_format()`, `from_format()`, `to_format()`
4. Convert to/from `UniversalSchematic`
5. Handle NBT data via `quartz_nbt`

### Working with NBT Data

- Use `utils::NbtValue` and `utils::NbtMap` wrappers
- Block entities and entities store NBT in their respective structs
- Use helper functions in `utils/nbt.rs` for common operations

### Cross-Platform Testing

Before releasing:
1. Run `cargo test` for Rust
2. Run `./test-wasm.sh` for WASM
3. Test Python with `maturin develop --features python && python tests/test.py` (if exists)
4. Verify CI passes on all platforms (Linux x64/ARM64, macOS x64/ARM64)

### Version Bumping

CI watches `Cargo.toml` version changes. To release:
1. Update version in `Cargo.toml`
2. Push to main/master
3. CI automatically publishes to all registries and creates GitHub release

## Common Patterns

### Setting Blocks with Properties

There are three ways to set blocks with properties:

```rust
// Method 1: Explicit BlockState (verbose but clear)
let mut lever = BlockState::new("minecraft:lever".to_string());
lever.properties.insert("facing".to_string(), "east".to_string());
lever.properties.insert("powered".to_string(), "false".to_string());
lever.properties.insert("face".to_string(), "floor".to_string());
schematic.set_block(0, 1, 0, lever);

// Method 2: Bracket notation (concise and readable)
schematic.set_block_str(0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]");

// Method 3: Simple blocks without properties
schematic.set_block_str(0, 0, 0, "minecraft:stone");
```

**Bracket Notation Format:**
- `"block_name[property1=value1,property2=value2]"`
- Useful for redstone circuits where properties matter
- WASM `set_block()` automatically supports this format
- See `examples/bracket_notation.rs` for more examples

### Creating a Schematic Programmatically

```rust
let mut schematic = UniversalSchematic::new("MySchematic".to_string());
schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
```

### Loading from File

```rust
let data = std::fs::read("example.litematic")?;
let schematic = litematic::from_litematic(&data)?;
```

### Converting Formats

```rust
let litematic_data = std::fs::read("input.litematic")?;
let schematic = litematic::from_litematic(&litematic_data)?;
let schem_data = schematic::to_schematic(&schematic)?;
std::fs::write("output.schem", schem_data)?;
```

### Working with Regions

```rust
// Set block in specific region
schematic.set_block_in_region("custom_region", x, y, z, block);

// Access region directly
if let Some(region) = schematic.get_region("Main") {
    let palette = &region.palette;
    let dimensions = region.get_dimensions();
}
```

## Important Implementation Details

- **Air Blocks**: Default palette index 0 is always `minecraft:air`
- **Coordinate System**: Standard Minecraft coordinates (Y-up)
- **Region Expansion**: Automatic when setting blocks outside bounds (maintains data)
- **Palette Index Rebuilding**: Required after deserialization (`rebuild_palette_index()`)
- **Memory Safety**: All FFI boundaries have explicit memory management functions
- **Thread Safety**: Core types are Send + Sync (with feature guards)
