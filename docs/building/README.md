# Building with Nucleation

Nucleation provides a powerful `building` module that allows you to procedurally generate structures using geometric shapes and advanced brushes.

## Overview

The building system is composed of three main components:

1.  **Shapes**: Define the geometry of the structure (e.g., Sphere, Cuboid).
2.  **Brushes**: Define what blocks fill the shape (e.g., Solid, Gradient, Shaded).
3.  **BuildingTool**: The interface that applies a brush to a shape on a schematic.

## Getting Started

First, ensure you have the necessary imports:

```rust
use nucleation::building::{BuildingTool, Sphere, Cuboid, SolidBrush, ColorBrush, LinearGradientBrush, ShadedBrush};
use nucleation::UniversalSchematic;
```

## Shapes

Shapes define *where* blocks will be placed. They implement the `Shape` trait which provides point iteration and surface normal calculation.

### Sphere

```rust
let sphere = Sphere::new(
    (0, 0, 0), // Center (x, y, z)
    10.0       // Radius
);
```

### Cuboid

```rust
let cuboid = Cuboid::new(
    (0, 0, 0),   // Min corner
    (10, 10, 10) // Max corner
);
```

## Brushes

Brushes define *what* blocks will be placed. They can use the position and surface normal to determine the block.

### SolidBrush

Places a single type of block.

```rust
let brush = SolidBrush::new(BlockState::new("minecraft:stone"));
```

### ColorBrush

Places the block that best matches a specific RGB color. It uses an internal palette of blocks mapped to their colors.

```rust
let brush = ColorBrush::new(255, 0, 0); // Red
```

### LinearGradientBrush

Interpolates between two colors linearly.

```rust
let brush = LinearGradientBrush::new(
    (0, 0, 0), (255, 0, 0),   // Start point and color (Red)
    (10, 0, 0), (0, 0, 255)   // End point and color (Blue)
);
```

You can also specify the interpolation color space (RGB or Oklab):

```rust
use nucleation::building::InterpolationSpace;

let brush = LinearGradientBrush::new(...)
    .with_space(InterpolationSpace::Oklab); // Smoother gradients
```

### MultiPointGradientBrush

Interpolates between multiple color stops.

```rust
let brush = MultiPointGradientBrush::new(
    (0, 0, 0),   // Start position
    (20, 0, 0),  // End position
    vec![
        (0.0, (255, 0, 0)),   // Red at 0%
        (0.5, (0, 255, 0)),   // Green at 50%
        (1.0, (0, 0, 255)),   // Blue at 100%
    ]
);
```

### ShadedBrush

Uses the surface normal to simulate lighting. It picks lighter or darker blocks from the palette based on the angle to the light source.

```rust
let brush = ShadedBrush::new(
    (255, 255, 255), // Base color (White)
    (0.0, 1.0, 0.0)  // Light direction (Top-down)
);
```

## Usage Example

```rust
let mut schematic = UniversalSchematic::new("my_build".to_string());
let mut tool = BuildingTool::new(&mut schematic);

// Create a shaded sphere
let sphere = Sphere::new((0, 0, 0), 10.0);
let brush = ShadedBrush::new((255, 255, 255), (0.0, 1.0, 0.0));

tool.fill(&sphere, &brush);
```
