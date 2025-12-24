# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

# Nucleation

Nucleation is a high-performance Minecraft schematic engine written in Rust with bindings for WebAssembly (JavaScript/TypeScript), Python, and FFI (C/PHP).

## Commands

### Build
- **Rust (Core):** `cargo build --release`
- **Rust (with Simulation):** `cargo build --release --features simulation`
- **WASM:** `./build-wasm.sh` (Builds `pkg/` directory)
- **Python:** `maturin develop --features python,simulation` (Installs into current venv)
- **FFI (Shared Libraries):** `./build-ffi.sh`

### Test
- **Core Tests:** `cargo test`
- **Simulation Tests:** `cargo test --features simulation`
- **WASM Tests:** `node tests/node_wasm_test.js` or `./test-wasm.sh`
- **Specific Integration Test:** `cargo test --lib --features simulation typed_executor::insign_io`

### Code Quality
- **Format:** `cargo fmt`
- **Pre-Push Verification:** `./pre-push.sh` (Runs formatting, builds, tests, and version checks)

## Code Architecture

### Core Abstractions
- **`UniversalSchematic` (`src/universal_schematic.rs`):** The central struct representing a schematic, independent of the source format (Litematic, Schem, etc.). It holds block data, entities, and metadata.
- **`SchematicBuilder` (`src/schematic_builder/`):** A fluent API for procedurally generating schematics. It supports ASCII-art style templates and compositional design.
- **`Region` / `DefinitionRegion`:** key structures for handling voxel data and boolean operations on regions.

### Modules & Structure
- **`src/formats/`:** Contains parsers and serializers for specific file formats (`litematic`, `schematic`, `nbt`).
- **`src/simulation/`:** wrappers around the `mchprs` crates to provide redstone simulation capabilities. Enabled via the `simulation` feature.
- **`src/wasm/`:** Rust-to-WASM bindings using `wasm-bindgen`.
- **`src/python/`:** Python bindings using `pyo3`.
- **`src/ffi/` & `src/php.rs`:** C-compatible FFI and PHP-specific bindings.

### Simulation (MCHPRS)
The simulation feature relies on a forked version of MCHPRS (Minecraft High Performance Redstone Server) crates (`mchprs_world`, `mchprs_redstone`, etc.), specified as git dependencies in `Cargo.toml`. This engine allows for accurate redstone circuit simulation within schematics.

### Language Bindings
The project is designed to maintain feature parity across languages.
- **Rust:** The source of truth.
- **WASM:** Exposes `SchematicParser`, `SchematicBuilder`, and `SimulationWorld` to JS.
- **Python:** Exposes similar classes via the `nucleation` module.

### Development Notes
- **Version Consistency:** `Cargo.toml` and `pyproject.toml` versions must match. The `./pre-push.sh` script verifies this.
- **WASM Output:** The WASM build artifacts are generated in the `pkg/` directory.
