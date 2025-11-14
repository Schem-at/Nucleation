# Unicode Palette Reference

The standard Unicode palette provides visual characters for redstone components, making circuit design intuitive and readable.

## Setup

The standard palette is **loaded by default** in `SchematicBuilder::new()`:

```rust
// Standard palette automatically loaded!
let circuit = SchematicBuilder::new()
    .from_template(r#"
        ─→─
        │█│
        "#)
    .build()?;
```

To start with an empty palette:

```rust
let builder = SchematicBuilder::empty()
    .map('X', "minecraft:stone")
    // ... custom mappings only
```

## Complete Character Reference

### Redstone Wire

| Character | Block State |
|-----------|-------------|
| `─` | `minecraft:redstone_wire` (horizontal) |
| `│` | `minecraft:redstone_wire` (vertical) |
| `┼` | `minecraft:redstone_wire` (cross) |
| `╋` | `minecraft:redstone_wire` (cross) |
| `├` | `minecraft:redstone_wire` (T-junction left) |
| `┤` | `minecraft:redstone_wire` (T-junction right) |
| `┬` | `minecraft:redstone_wire` (T-junction top) |
| `┴` | `minecraft:redstone_wire` (T-junction bottom) |
| `┐` | `minecraft:redstone_wire` (corner top-right) |
| `┘` | `minecraft:redstone_wire` (corner bottom-right) |
| `└` | `minecraft:redstone_wire` (corner bottom-left) |
| `┌` | `minecraft:redstone_wire` (corner top-left) |

### Repeaters (1-tick delay)

| Character | Direction | Block State |
|-----------|-----------|-------------|
| `→` | East | `minecraft:repeater[facing=west,delay=1]` |
| `←` | West | `minecraft:repeater[facing=east,delay=1]` |
| `↑` | North | `minecraft:repeater[facing=south,delay=1]` |
| `↓` | South | `minecraft:repeater[facing=north,delay=1]` |

**Note:** Arrow shows signal flow direction; `facing` property is opposite.

### Repeaters (2-tick delay)

| Character | Direction | Block State |
|-----------|-----------|-------------|
| `⇒` | East | `minecraft:repeater[facing=west,delay=2]` |
| `⇐` | West | `minecraft:repeater[facing=east,delay=2]` |
| `⇑` | North | `minecraft:repeater[facing=south,delay=2]` |
| `⇓` | South | `minecraft:repeater[facing=north,delay=2]` |

### Repeaters (3-tick delay)

| Character | Direction | Block State |
|-----------|-----------|-------------|
| `⟹` | East | `minecraft:repeater[facing=west,delay=3]` |
| `⟸` | West | `minecraft:repeater[facing=east,delay=3]` |
| `⟰` | North | `minecraft:repeater[facing=south,delay=3]` |
| `⟱` | South | `minecraft:repeater[facing=north,delay=3]` |

### Repeaters (4-tick delay)

| Character | Direction | Block State |
|-----------|-----------|-------------|
| `⤏` | East | `minecraft:repeater[facing=west,delay=4]` |
| `⤎` | West | `minecraft:repeater[facing=east,delay=4]` |
| `⤒` | North | `minecraft:repeater[facing=south,delay=4]` |
| `⤓` | South | `minecraft:repeater[facing=north,delay=4]` |

### Comparators (Compare Mode)

| Character | Direction | Block State |
|-----------|-----------|-------------|
| `▷` | East | `minecraft:comparator[facing=west,mode=compare]` |
| `◁` | West | `minecraft:comparator[facing=east,mode=compare]` |
| `△` | North | `minecraft:comparator[facing=south,mode=compare]` |
| `▽` | South | `minecraft:comparator[facing=north,mode=compare]` |

### Comparators (Subtract Mode)

| Character | Direction | Block State |
|-----------|-----------|-------------|
| `▶` | East | `minecraft:comparator[facing=west,mode=subtract]` |
| `◀` | West | `minecraft:comparator[facing=east,mode=subtract]` |
| `▲` | North | `minecraft:comparator[facing=south,mode=subtract]` |
| `▼` | South | `minecraft:comparator[facing=north,mode=subtract]` |

### Torches & Blocks

| Character | Block State |
|-----------|-------------|
| `█` | `minecraft:redstone_torch[lit=true]` |
| `▓` | `minecraft:redstone_wall_torch[facing=north,lit=true]` |
| `▒` | `minecraft:redstone_wall_torch[facing=south,lit=true]` |
| `░` | `minecraft:redstone_wall_torch[facing=east,lit=true]` |
| `c` | `minecraft:gray_concrete` |
| `C` | `minecraft:stone` |
| `·` | `minecraft:air` |
| `_` | `minecraft:air` |

## Palette Variants

### Standard Palette (Default)

Complete set with all characters above. Loaded automatically with `SchematicBuilder::new()`.

```rust
let circuit = SchematicBuilder::new()  // Standard palette loaded!
    .from_template(template)
    .build()?;
```

### Minimal Palette

Compact subset for simple circuits:

```rust
let circuit = SchematicBuilder::empty()
    .use_minimal_palette()
    .from_template(template)
    .build()?;
```

**Includes:**
- Basic wire: `─`, `│`, `┼`
- Repeaters (1-tick): `→`, `←`, `↑`, `↓`
- Comparators: `▷`, `◁`, `△`, `▽`
- Torch: `█`
- Blocks: `c`, `·`

### Compact Palette

ASCII-only characters for maximum compatibility:

```rust
let circuit = SchematicBuilder::empty()
    .use_compact_palette()
    .from_template(template)
    .build()?;
```

**Includes:**
- Wire: `-`, `|`, `+`
- Repeaters: `>`, `<`, `^`, `v`
- Comparators: `}`, `{`, `A`, `V`
- Torch: `T`
- Blocks: `#`, `.`

## Examples

### Half-Adder Circuit

```rust
let half_adder = SchematicBuilder::new()
    .from_template(r#"
        # Base layer
        ·····c····
        ··ccccc···
        cc··cccccc
        
        # Logic layer
        ·····│····
        ··│█←┤█···
        ──··├┴┴┴←─
        "#)
    .build()?;
```

### XOR Gate

```rust
let xor = SchematicBuilder::new()
    .from_template(r#"
        # Base
        ccc
        ccc
        ccc
        
        # Logic
        █─┐
        ├┼┤
        █─┘
        "#)
    .build()?;
```

### Clock Circuit

```rust
let clock = SchematicBuilder::new()
    .from_template(r#"
        # Base
        cccc
        cccc
        
        # Logic
        →─█─
        ─←─┘
        "#)
    .build()?;
```

## Custom Overrides

Override specific characters while keeping the standard palette:

```rust
let circuit = SchematicBuilder::new()  // Standard palette loaded
    .map('R', "minecraft:redstone_block")  // Override/add custom
    .map('G', "minecraft:gold_block")
    .from_template(r#"
        RGR
        ─→─
        "#)
    .build()?;
```

## CLI Usage

The CLI tool uses the standard palette by default:

```bash
# Standard palette automatically loaded
cat circuit.txt | schematic-builder -o circuit.litematic

# Disable for plain block characters
schematic-builder -i blocks.txt -o blocks.litematic --no-palette
```

## Tips

1. **Visual Design**: Use Unicode characters to make circuits look like their actual layout
2. **Version Control**: Templates with Unicode are human-readable in diffs
3. **Documentation**: Circuits are self-documenting when using visual characters
4. **Compatibility**: Use compact palette for ASCII-only environments
5. **Custom Mix**: Start with standard palette and override specific characters as needed

## See Also

- [SchematicBuilder Guide](../guide/schematic-builder.md)
- [CLI Tool Guide](../guide/cli-tool.md)
- [Examples](../examples/)

