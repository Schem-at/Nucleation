# SchematicBuilder Guide

The `SchematicBuilder` provides an intuitive way to create Minecraft schematics using ASCII art and compositional design patterns.

## Quick Start

```rust
use nucleation::SchematicBuilder;

// Simple example - 3x1x3 platform
let schematic = SchematicBuilder::new()
    .name("platform")
    .map('S', "minecraft:stone")
    .map('_', "minecraft:air")
    .layers(&[
        &[
            "SSS",
            "SSS",
            "SSS",
        ],
    ])
    .build()?;
```

## Features

### 1. Unicode Palettes (Default)

The standard Unicode palette is loaded by default, allowing you to use visual characters for redstone components:

```rust
let circuit = SchematicBuilder::new()  // Standard palette auto-loaded!
    .from_template(r#"
        # Base layer
        ccc
        ccc
        
        # Logic layer
        ─│─
        ─→─
        "#)
    .build()?;
```

**Common Unicode Characters:**
- `─`, `│`, `┼`, `╋` - Redstone wire
- `→`, `←`, `↑`, `↓` - Repeaters (1-tick)
- `▷`, `◁`, `△`, `▽` - Comparators (compare mode)
- `▶`, `◀`, `▲`, `▼` - Comparators (subtract mode)
- `█` - Redstone torch
- `c` - Concrete (gray)
- `·` - Air

See [Unicode Palette Reference](../api/unicode-palette.md) for the complete list.

### 2. Custom Palettes

Override or extend the standard palette:

```rust
let schematic = SchematicBuilder::new()
    .map('R', "minecraft:redstone_block")
    .map('G', "minecraft:gold_block")
    .layers(&[&["RGR"]])
    .build()?;
```

### 3. Compositional Design

Use entire schematics as palette entries to build hierarchically:

```rust
// Build a basic gate
let and_gate = SchematicBuilder::new()
    .name("and_gate")
    // ... define AND gate ...
    .build()?;

// Use it in a larger circuit
let half_adder = SchematicBuilder::new()
    .name("half_adder")
    .map_schematic('A', and_gate)  // Use schematic as palette entry!
    .map_schematic('X', xor_gate)
    .layers(&[&["AX"]])  // Place them side-by-side
    .build()?;
```

**Key Benefits:**
- Circuits tile based on **tight bounds** (no extra spacing)
- Air blocks are skipped (no overwriting)
- Works with any dimensions (symmetric or asymmetric)

### 4. Template Format

Use a structured template format with inline palette definitions:

```rust
let template = r#"
# Base layer
ccc
ccc

# Logic layer
─→─
│█│

[palette]
c = minecraft:gray_concrete
─ = minecraft:redstone_wire
→ = minecraft:repeater[facing=west,delay=1]
│ = minecraft:redstone_wire
█ = minecraft:redstone_torch
"#;

let schematic = SchematicBuilder::from_template(template)?.build()?;
```

### 5. Multi-Layer Schematics

Build 3D structures with multiple layers (Y-axis):

```rust
let tower = SchematicBuilder::new()
    .map('S', "minecraft:stone")
    .layers(&[
        &["SSS", "SSS", "SSS"],  // Layer 0 (bottom)
        &["S_S", "S_S", "S_S"],  // Layer 1 (middle)
        &["SSS", "SSS", "SSS"],  // Layer 2 (top)
    ])
    .build()?;
```

### 6. IO Markers

Mark input/output positions for circuit simulation:

```rust
let circuit = SchematicBuilder::new()
    .map('I', "minecraft:redstone_wire")
    .map('O', "minecraft:redstone_wire")
    .io_marker('I', IoType::Input, "input_a")
    .io_marker('O', IoType::Output, "output")
    .layers(&[&["I_O"]])
    .build()?;
```

### 7. Export to Template/JSON

Convert schematics back to templates for version control:

```rust
let builder = SchematicBuilder::new()
    .map('S', "minecraft:stone")
    .layers(&[&["SSS"]]);

// Export as template
let template = builder.to_template();

// Export as JSON (requires "serde" feature)
#[cfg(feature = "serde")]
let json = builder.to_json()?;
```

## CLI Tool

Build schematics from the command line:

```bash
# From stdin
cat circuit.txt | schematic-builder -o circuit.litematic

# From file
schematic-builder -i circuit.txt -o circuit.litematic

# Choose format
schematic-builder -i circuit.txt -o circuit.schem --format schem

# Without standard palette
schematic-builder -i blocks.txt -o blocks.litematic --no-palette
```

## Best Practices

### 1. Use Tight Bounds for Tiling

When stacking schematics, they tile based on tight bounds (actual blocks, not full bounding box):

```rust
// Each unit is 10x10x10 actual blocks
let unit = create_10x10x10_cube();

// Stack 4 units = 40 blocks wide (not 40 + padding)
let stacked = SchematicBuilder::new()
    .map_schematic('U', unit)
    .layers(&[&["UUUU"]])
    .build()?;
// Result: exactly 40 blocks wide ✅
```

### 2. Consistent Dimensions

When mixing schematics in a layer, ensure they have compatible dimensions:

```rust
// ✅ Good: All 10x10x10
let grid = SchematicBuilder::new()
    .map_schematic('A', cube_10x10x10)
    .map_schematic('B', cube_10x10x10)
    .layers(&[&["AB", "BA"]])
    .build()?;

// ❌ Bad: Mismatched sizes cause misalignment
let grid = SchematicBuilder::new()
    .map_schematic('A', cube_10x10x10)
    .map('_', "minecraft:air")  // Single block!
    .layers(&[&["A_A"]])  // Incorrect tiling
    .build()?;
```

### 3. Visual Circuit Design

Use Unicode characters for highly readable circuit designs:

```rust
let adder = SchematicBuilder::new()
    .from_template(r#"
        # Base
        ·····c····
        ··ccccc···
        cc··cccccc
        
        # Logic
        ·····│····
        ··│█←┤█···
        ──··├┴┴┴←─
        "#)
    .build()?;
```

## Examples

See [docs/examples/](../examples/) for complete working examples:
- [4-Bit Adder](../examples/4-bit-adder.md) - Hierarchical circuit composition
- [Checkerboard](../examples/checkerboard.md) - 2D tiling patterns
- [CLI Usage](../examples/cli-usage.md) - Command-line workflows

## API Reference

- [Rust API](../api/rust.md#schematicbuilder)
- [JavaScript API](../api/javascript.md#schematicbuilder)
- [Python API](../api/python.md#schematicbuilder)

