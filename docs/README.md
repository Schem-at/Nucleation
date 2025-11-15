# Nucleation Documentation

Complete documentation for the Nucleation schematic engine.

## Choose Your Language

Select your language for complete, language-specific documentation:

### 📦 [Rust Documentation](rust/)
Complete API reference for Rust, including:
- Core types (UniversalSchematic, BlockState, Region)
- Loading/saving schematics
- Block and region operations
- Block entities and NBT
- SchematicBuilder with ASCII art
- Redstone simulation (MCHPRS)
- TypedCircuitExecutor

### 🌐 [JavaScript/TypeScript Documentation](javascript/)
Complete API reference for JavaScript/TypeScript (WASM), including:
- Installation and setup (Node.js, browser, CDN)
- SchematicWrapper API
- Block operations and iteration
- Chunk strategies
- SchematicBuilder
- Real-time simulation
- TypedCircuitExecutor

### 🐍 [Python Documentation](python/)
Complete API reference for Python, including:
- Installation via pip
- Schematic class API
- Block and region operations
- Loading/saving files
- SchematicBuilder
- Simulation support
- TypedCircuitExecutor

## Shared Guides

These guides apply to all languages:

### Core Features
- **[SchematicBuilder Guide](shared/guide/schematic-builder.md)** - Build schematics with ASCII art and compositional design
- **[TypedCircuitExecutor Guide](shared/guide/typed-executor.md)** - High-level circuit simulation with typed IO
- **[Insign IO Integration](insign-io-integration.md)** - Auto-create TypedCircuitExecutor from sign annotations
- **[Unicode Palette Reference](shared/unicode-palette.md)** - Visual circuit design characters

## Quick Comparison

| Feature | Rust | JavaScript | Python |
|---------|------|------------|--------|
| Load/Save Schematics | ✅ | ✅ | ✅ |
| Block Operations | ✅ | ✅ | ✅ |
| Region Operations | ✅ | ✅ | ✅ |
| Block Entities | ✅ | ✅ | ✅ |
| SchematicBuilder | ✅ | ✅ | ✅ |
| Unicode Palettes | ✅ | ✅ | ✅ |
| Compositional Design | ✅ | ✅ | ✅ |
| CLI Tool | ✅ | ❌ | ❌ |
| Redstone Simulation | ✅ | ✅ | ⚠️ |
| TypedCircuitExecutor | ✅ | ✅ | ⚠️ |
| Custom IO Signals | ✅ | ✅ | ⚠️ |

**Legend:**
- ✅ Full support with complete documentation
- ⚠️ Supported but needs integration testing
- ❌ Not available

## Format Support

All languages support the same formats:

- ✅ **Litematic** (`.litematic`) - Full read/write support
- ✅ **Sponge Schematic v2** (`.schem`) - Full read/write support
- ✅ **WorldEdit Schematic** (`.schematic`) - Full read/write support
- ✅ **Structure NBT** (`.nbt`) - Read support
- ✅ **JSON export** - Write support (debugging)

## Installation

### Rust
```bash
cargo add nucleation
```

### JavaScript/TypeScript
```bash
npm install nucleation
```

### Python
```bash
pip install nucleation
```

## Quick Start Examples

### Rust
```rust
use nucleation::UniversalSchematic;

let mut schematic = UniversalSchematic::new("my_schematic".to_string());
schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));

let bytes = nucleation::litematic::to_litematic(&schematic)?;
std::fs::write("output.litematic", bytes)?;
```

### JavaScript
```typescript
import init, { SchematicWrapper } from "nucleation";
await init();

const schematic = new SchematicWrapper();
schematic.set_block(0, 0, 0, "minecraft:stone");

const bytes = schematic.to_litematic();
// Save or download bytes...
```

### Python
```python
from nucleation import Schematic

schematic = Schematic("my_schematic")
schematic.set_block(0, 0, 0, "minecraft:stone")

with open("output.litematic", "wb") as f:
    f.write(schematic.to_litematic())
```

## Feature Highlights

### SchematicBuilder
Build circuits with ASCII art and Unicode characters:
```rust
let circuit = SchematicBuilder::new()
    .from_template(r#"
        # Base layer
        ccc
        
        # Logic layer
        ─→─
        "#)
    .build()?;
```

### Compositional Design
Build complex circuits from smaller components:
```rust
let four_bit_adder = SchematicBuilder::new()
    .map_schematic('F', full_adder)  // Use schematic as palette entry
    .layers(&[&["FFFF"]])            // Stack 4 full-adders
    .build()?;
```

### Redstone Simulation
Simulate circuits in real-time:
```rust
let world = schematic.create_simulation_world()?;
world.on_use_block(0, 1, 0)?;  // Toggle lever
world.tick(10)?;
let is_lit = world.is_lit(5, 1, 0)?;
```

### TypedCircuitExecutor
High-level API with typed inputs/outputs:
```rust
let result = executor.execute(
    inputs,  // HashMap<String, Value>
    ExecutionMode::FixedTicks { ticks: 100 }
)?;
```

## Contributing

See the main repository [CONTRIBUTING.md](../CONTRIBUTING.md) for development setup and guidelines.

## License

Licensed under the **GNU AGPL-3.0-only**. See [LICENSE](../LICENSE) for details.
