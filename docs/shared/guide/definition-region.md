# DefinitionRegion Guide

`DefinitionRegion` represents a logical 3D region defined by one or more bounding boxes. It's essential for defining circuit inputs/outputs, rendering highlights, and manipulating spatial data in Minecraft schematics.

## Table of Contents

1. [Creating Regions](#creating-regions)
2. [Accessing Region Data](#accessing-region-data)
3. [Geometric Transformations](#geometric-transformations)
4. [Boolean Operations](#boolean-operations)
11. [Serialization & Persistence](#serialization--persistence)
12. [Metadata](#metadata)
13. [Connectivity Analysis](#connectivity-analysis)
14. [Filtering with Schematics](#filtering-with-schematics)
15. [Sort Strategies for Circuit Execution](#sort-strategies-for-circuit-execution)
16. [Renderer Integration](#renderer-integration)
17. [Best Practices](#best-practices)

---

## Creating Regions

### Empty Region

```javascript
// JavaScript
const region = new DefinitionRegionWrapper();

// Or in Python
region = DefinitionRegion();
```

### From a Single Bounding Box

```javascript
// JavaScript
const region = DefinitionRegionWrapper.fromBounds(
	new BlockPosition(0, 0, 0),
	new BlockPosition(10, 5, 10)
);

// Python
region = DefinitionRegion.from_bounds((0, 0, 0), (10, 5, 10));
```

### From Multiple Bounding Boxes

Use when you have pre-defined, potentially disjoint boxes:

```javascript
// JavaScript - keeps boxes separate (no merging)
const boxes = [
	{ min: [0, 0, 0], max: [3, 0, 0] }, // Input bits 0-3
	{ min: [0, 0, 2], max: [3, 0, 2] }, // Input bits 4-7
];
const region = DefinitionRegionWrapper.fromBoundingBoxes(boxes);

// Python
boxes = [((0, 0, 0), (3, 0, 0)), ((0, 0, 2), (3, 0, 2))];
region = DefinitionRegion.from_bounding_boxes(boxes);
```

### From Individual Positions

Adjacent points are automatically merged into larger boxes:

```javascript
// JavaScript
const positions = [
	[0, 0, 0],
	[1, 0, 0],
	[2, 0, 0],
	[3, 0, 0],
];
const region = DefinitionRegionWrapper.fromPositions(positions);

// Python
positions = [(0, 0, 0), (1, 0, 0), (2, 0, 0), (3, 0, 0)];
region = DefinitionRegion.from_positions(positions);
```

### Building Incrementally

```javascript
// JavaScript
const region = new DefinitionRegionWrapper();
region.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(5, 0, 0));
region.addPoint(10, 0, 0); // Single point

// Python
region = DefinitionRegion();
region.add_bounds((0, 0, 0), (5, 0, 0));
region.add_point(10, 0, 0);
```

---

## Accessing Region Data

### Box-Level Access (Essential for Rendering)

```javascript
// JavaScript
const count = region.boxCount();           // Number of boxes
const box0 = region.getBox(0);             // { min: [x,y,z], max: [x,y,z] } or null
const allBoxes = region.getBoxes();        // Array of all boxes

// Python
count = region.box_count()
box0 = region.get_box(0)                   # ((min_x, min_y, min_z), (max_x, max_y, max_z)) or None
all_boxes = region.get_boxes()             # List of all boxes
```

### Position-Level Access

```javascript
// JavaScript
const positions = region.positions(); // All positions (unordered)
const sorted = region.positionsSorted(); // Sorted by Y, then X, then Z

// Python
positions = region.positions();
sorted_pos = region.positions_sorted();
```

### Geometry Information

```javascript
// JavaScript
const bounds = region.getBounds();          // { min: [x,y,z], max: [x,y,z] } or null
const dims = region.dimensions();           // [width, height, length]
const center = region.center();             // [x, y, z] integers, or null
const centerF = region.centerF32();         // [x, y, z] floats for rendering

const vol = region.volume();                // Total blocks
const empty = region.isEmpty();

// Python
bounds = region.get_bounds()                # { "min": (x,y,z), "max": (x,y,z) } or None
dims = region.dimensions()                  # (width, height, length)
center = region.center()                    # (x, y, z) or None
center_f = region.center_f32()              # (x, y, z) floats or None

vol = region.volume()
empty = region.is_empty()
```

### Point Queries

```javascript
// JavaScript
region.contains(5, 5, 5); // true/false

// Frustum culling: does region overlap this box?
region.intersectsBounds(minX, minY, minZ, maxX, maxY, maxZ);

// Python
region.contains(5, 5, 5);
region.intersects_bounds((minX, minY, minZ), (maxX, maxY, maxZ));
```

---

## Geometric Transformations

### Mutating Transformations

```javascript
// JavaScript
region.shift(10, 0, -5); // Move by offset
region.expand(1, 1, 1); // Grow in all directions
region.contract(1); // Shrink uniformly

// Python
region.shift(10, 0, -5);
region.expand(1, 1, 1);
region.contract(1);
```

### Immutable Transformations (Recommended for Rendering)

```javascript
// JavaScript - original unchanged
const moved = region.shifted(10, 0, -5);
const bigger = region.expanded(1, 1, 1);
const smaller = region.contracted(1);

// Python
moved = region.shifted(10, 0, -5);
bigger = region.expanded(1, 1, 1);
smaller = region.contracted(1);
```

### Copying

```javascript
// JavaScript
const copy = region.copy();   // or region.clone()

// Python
copy = region.copy()
# Also supports Python copy protocol
import copy
copy1 = copy.copy(region)
copy2 = copy.deepcopy(region)
```

---

## Boolean Operations

### Mutating Operations

```javascript
// JavaScript
regionA.subtract(regionB); // Remove B's points from A
regionA.intersect(regionB); // Keep only overlapping points
regionA.merge(regionB); // Add B's boxes to A
regionA.unionInto(regionB); // Union with simplification

// Python
region_a.subtract(region_b);
region_a.intersect(region_b);
region_a.merge(region_b);
region_a.union_into(region_b);
```

### Immutable Operations

```javascript
// JavaScript
const diff = regionA.subtracted(regionB);
const overlap = regionA.intersected(regionB);
const combined = regionA.union(regionB);

// Python
diff = region_a.subtracted(region_b);
overlap = region_a.intersected(region_b);
combined = region_a.union(region_b);
```

---

## Serialization & Persistence

Definition Regions can be serialized and embedded directly into schematic files. This allows you to define logical regions (like inputs, outputs, or documentation zones) that persist when the schematic is saved and loaded.

### Supported Formats

- **.litematic**: Stored in the `Metadata` compound tag.
- **.schem (V3)**: Stored in the `Metadata` compound tag.
- **.schem (V2)**: Legacy support via `Metadata` tag injection.

### How it Works

When you save a `UniversalSchematic`, its `definition_regions` map is serialized to JSON and stored in the schematic's metadata under the key `NucleationDefinitions`. When loading, this JSON is parsed back into full `DefinitionRegion` objects.

### Example

```rust
// Rust
let mut schematic = UniversalSchematic::new("MyCircuit".to_string());

// Create a region
let mut region = DefinitionRegion::new();
region.add_bounds((0, 0, 0), (5, 5, 5));
region.set_metadata("type", "input");

// Add to schematic
schematic.definition_regions.insert("MainInput".to_string(), region);

// Save to file (region data is embedded)
let data = schematic.to_schematic().unwrap();
std::fs::write("circuit.schem", data).unwrap();

// ... later ...

// Load from file (region data is restored)
let loaded = UniversalSchematic::from_schematic(&data).unwrap();
let loaded_region = loaded.definition_regions.get("MainInput").unwrap();
assert_eq!(loaded_region.get_metadata("type"), Some(&"input".to_string()));
```

This works seamlessly across all language bindings (Rust, JavaScript/WASM, Python).

---

Store arbitrary key-value data with regions:

```javascript
// JavaScript - chaining style
const region = DefinitionRegionWrapper.fromBounds(...)
  .setMetadata("color", "#ff0000")
  .setMetadata("label", "Input A");

// Read back
const color = region.getMetadata("color");  // "#ff0000" or null
const allMeta = region.getAllMetadata();    // { color: "#ff0000", label: "Input A" }
const keys = region.metadataKeys();         // ["color", "label"]

// Python
region = DefinitionRegion.from_bounds((0,0,0), (5,5,5))
    .with_metadata("color", "#ff0000")
    .with_metadata("label", "Input A")

# Or mutating style
region.set_metadata("color", "#ff0000")

color = region.get_metadata("color")
all_meta = region.get_all_metadata()
keys = region.metadata_keys()
```

---

## Connectivity Analysis

```javascript
// JavaScript
const isOneBlob = region.isContiguous(); // All blocks connected?
const numParts = region.connectedComponents(); // How many separate groups?

// Python
is_one_blob = region.is_contiguous();
num_parts = region.connected_components();
```

---

## Filtering with Schematics

```javascript
// JavaScript
// Keep only blocks matching name substring
const redstone = region.filterByBlock(schematic, "redstone");

// Keep only blocks with specific properties
const litLamps = region.filterByProperties(schematic, { lit: "true" });

// Python
redstone = region.filter_by_block(schematic, "redstone");
lit_lamps = region.filter_by_properties(schematic, { lit: "true" });
```

---

## Sort Strategies for Circuit Execution

When using `DefinitionRegion` for circuit IO, the order of positions determines bit assignment (position 0 = LSB). Use `SortStrategy` to control this ordering.

### Available Strategies

```javascript
// JavaScript
const { SortStrategyWrapper } = nucleation;

// Axis-first ascending
SortStrategyWrapper.yxz(); // Y first, then X, then Z (DEFAULT)
SortStrategyWrapper.xyz(); // X first, then Y, then Z
SortStrategyWrapper.zyx(); // Z first, then Y, then X

// Axis-first descending (primary axis descending, others ascending)
SortStrategyWrapper.yDescXZ(); // Y descending, then X, then Z
SortStrategyWrapper.xDescYZ(); // X descending, then Y, then Z
SortStrategyWrapper.zDescYX(); // Z descending, then Y, then X

// Fully descending
SortStrategyWrapper.descending(); // Y, X, Z all descending

// Distance-based (useful for radial layouts)
SortStrategyWrapper.distanceFrom(x, y, z); // Closest to (x,y,z) first
SortStrategyWrapper.distanceFromDesc(x, y, z); // Farthest from (x,y,z) first

// Order preservation
SortStrategyWrapper.preserve(); // Keep order from region construction
SortStrategyWrapper.reverse(); // Reverse the iteration order
```

```python
# Python
from nucleation import SortStrategy

# Axis-first ascending
SortStrategy.yxz()      # Y first, then X, then Z (DEFAULT)
SortStrategy.xyz()      # X first, then Y, then Z
SortStrategy.zyx()      # Z first, then Y, then X

# Axis-first descending
SortStrategy.y_desc_xz()  # Y descending, then X, then Z
SortStrategy.x_desc_yz()  # X descending, then Y, then Z
SortStrategy.z_desc_yx()  # Z descending, then Y, then X

# Fully descending
SortStrategy.descending()  # Y, X, Z all descending

# Distance-based
SortStrategy.distance_from(x, y, z)
SortStrategy.distance_from_desc(x, y, z)

# Order preservation
SortStrategy.preserve()
SortStrategy.reverse()
```

### Using with CircuitBuilder

```javascript
// JavaScript - Default sorting (YXZ)
let builder = new CircuitBuilderWrapper(schematic).withInputAuto(
	"data",
	IoTypeWrapper.unsignedInt(8),
	region
);

// Custom sorting (Y descending for top-to-bottom bit ordering)
builder = new CircuitBuilderWrapper(schematic).withInputAutoSorted(
	"data",
	IoTypeWrapper.unsignedInt(8),
	region,
	SortStrategyWrapper.yDescXZ()
);

// Preserve box order (LSB in first box, MSB in second)
const multiBoxRegion = DefinitionRegionWrapper.fromBoundingBoxes([
	{ min: [0, 0, 0], max: [3, 0, 0] }, // Bits 0-3
	{ min: [0, 0, 2], max: [3, 0, 2] }, // Bits 4-7
]);
builder = new CircuitBuilderWrapper(schematic).withInputAutoSorted(
	"data",
	IoTypeWrapper.unsignedInt(8),
	multiBoxRegion,
	SortStrategyWrapper.preserve()
);

// Distance-based (closest to sign position first)
const signPos = [5, 2, 0];
builder = new CircuitBuilderWrapper(schematic).withInputAutoSorted(
	"data",
	IoTypeWrapper.unsignedInt(8),
	region,
	SortStrategyWrapper.distanceFrom(...signPos)
);
```

```python
# Python - Default sorting (YXZ)
builder = CircuitBuilder(schematic)
builder.with_input_auto("data", IoType.unsigned_int(8), region)

# Custom sorting (Y descending)
builder = CircuitBuilder(schematic)
builder.with_input_auto_sorted(
    "data",
    IoType.unsigned_int(8),
    region,
    SortStrategy.y_desc_xz()
)

# Preserve box order
multi_box_region = DefinitionRegion.from_bounding_boxes([
    ((0, 0, 0), (3, 0, 0)),  # Bits 0-3
    ((0, 0, 2), (3, 0, 2)),  # Bits 4-7
])
builder = CircuitBuilder(schematic)
builder.with_input_auto_sorted(
    "data",
    IoType.unsigned_int(8),
    multi_box_region,
    SortStrategy.preserve()
)
```

### Parse from String

```javascript
// JavaScript
const strategy = SortStrategyWrapper.fromString("y_desc");
console.log(strategy.name); // "y_desc_x_z"
```

```python
# Python
strategy = SortStrategy.from_string("y_desc")
print(strategy.name)  # "y_desc_x_z"
```

Valid strings: `"yxz"`, `"xyz"`, `"zyx"`, `"y_desc"`, `"x_desc"`, `"z_desc"`, `"descending"`, `"preserve"`, `"boxOrder"`, `"reverse"`

---

## Renderer Integration

This section provides patterns for integrating `DefinitionRegion` with 3D renderers.

### Basic Box Rendering

The most efficient approach is to render each bounding box as a wireframe or translucent cube:

```javascript
// JavaScript / Three.js style pseudocode
function renderRegion(region, scene, color = 0xff0000, opacity = 0.3) {
	const boxes = region.getBoxes();

	for (const box of boxes) {
		const [minX, minY, minZ] = box.min;
		const [maxX, maxY, maxZ] = box.max;

		// Box dimensions (inclusive bounds, so +1)
		const width = maxX - minX + 1;
		const height = maxY - minY + 1;
		const depth = maxZ - minZ + 1;

		// Create box geometry
		const geometry = new THREE.BoxGeometry(width, height, depth);
		const material = new THREE.MeshBasicMaterial({
			color: color,
			transparent: true,
			opacity: opacity,
			wireframe: false,
		});

		const mesh = new THREE.Mesh(geometry, material);

		// Position at center of box
		mesh.position.set(minX + width / 2, minY + height / 2, minZ + depth / 2);

		scene.add(mesh);
	}
}
```

### Frustum Culling

Use `intersectsBounds` to skip regions outside the camera view:

```javascript
function renderVisibleRegions(regions, camera, scene) {
	// Get camera frustum bounds (simplified)
	const frustum = getFrustumBounds(camera);

	for (const region of regions) {
		// Quick rejection test
		if (
			!region.intersectsBounds(
				frustum.minX,
				frustum.minY,
				frustum.minZ,
				frustum.maxX,
				frustum.maxY,
				frustum.maxZ
			)
		) {
			continue; // Skip - not visible
		}

		renderRegion(region, scene);
	}
}
```

### Color-Coded IO Visualization

```javascript
function renderCircuitIO(executor, scene) {
	const layoutInfo = executor.getLayoutInfo();

	// Define color scheme
	const colors = {
		input: 0x00ff00, // Green for inputs
		output: 0xff0000, // Red for outputs
		active: 0xffff00, // Yellow for active/powered
	};

	// Render inputs
	for (const [name, info] of Object.entries(layoutInfo.inputs)) {
		const region = info.region;
		const color = info.isActive ? colors.active : colors.input;

		renderRegion(region, scene, color, 0.4);

		// Add label at center
		const center = region.centerF32();
		if (center) {
			addLabel(scene, name, center, colors.input);
		}
	}

	// Render outputs similarly
	for (const [name, info] of Object.entries(layoutInfo.outputs)) {
		const region = info.region;
		const color = info.isActive ? colors.active : colors.output;
		renderRegion(region, scene, color, 0.4);
	}
}
```

### Hover Detection

```javascript
function getRegionAtPoint(regions, x, y, z) {
	for (const [name, region] of Object.entries(regions)) {
		if (region.contains(x, y, z)) {
			return { name, region };
		}
	}
	return null;
}

// Usage with raycasting
function onMouseMove(event, camera, regions) {
	const raycaster = new THREE.Raycaster();
	raycaster.setFromCamera(mousePos, camera);

	// Get intersection point with schematic
	const intersect = getSchematicIntersection(raycaster);
	if (intersect) {
		const hit = getRegionAtPoint(
			regions,
			Math.floor(intersect.x),
			Math.floor(intersect.y),
			Math.floor(intersect.z)
		);

		if (hit) {
			showTooltip(`${hit.name}: ${hit.region.volume()} blocks`);
			highlightRegion(hit.region);
		}
	}
}
```

### Level of Detail (LOD)

For large regions, use the overall bounds at distance:

```javascript
function renderRegionLOD(region, camera, scene) {
	const center = region.centerF32();
	if (!center) return;

	const distance = camera.position.distanceTo(new THREE.Vector3(...center));
	const dims = region.dimensions();
	const maxDim = Math.max(...dims);

	// LOD thresholds
	if (distance > maxDim * 10) {
		// Far: just render overall bounds as single box
		const bounds = region.getBounds();
		renderSingleBox(bounds, scene, 0.1);
	} else if (distance > maxDim * 5) {
		// Medium: render boxes without individual positions
		renderBoxes(region.getBoxes(), scene, 0.3);
	} else {
		// Close: full detail with individual blocks if needed
		renderDetailedRegion(region, scene, 0.5);
	}
}
```

### Animation: Pulsing Highlight

```javascript
class RegionHighlight {
	constructor(region, color) {
		this.region = region;
		this.color = color;
		this.phase = 0;
		this.meshes = [];
	}

	build(scene) {
		const boxes = this.region.getBoxes();
		for (const box of boxes) {
			const mesh = createBoxMesh(box, this.color);
			this.meshes.push(mesh);
			scene.add(mesh);
		}
	}

	update(deltaTime) {
		this.phase += deltaTime * 2; // Pulse speed
		const opacity = 0.2 + Math.sin(this.phase) * 0.15;

		for (const mesh of this.meshes) {
			mesh.material.opacity = opacity;
		}
	}

	dispose(scene) {
		for (const mesh of this.meshes) {
			scene.remove(mesh);
			mesh.geometry.dispose();
			mesh.material.dispose();
		}
		this.meshes = [];
	}
}
```

---

## Best Practices

### 1. Use Immutable Methods for UI

When manipulating regions for display, prefer immutable methods to avoid unintended side effects:

```javascript
// Good - original preserved
const displayRegion = region.expanded(1, 1, 1); // Highlight border

// Risky - modifies original
region.expand(1, 1, 1);
```

### 2. Use `from_bounding_boxes` for Exact IO Layout

When you need precise control over bit ordering (e.g., for multi-bit inputs), use `from_bounding_boxes` which preserves box order:

```javascript
// Bits 0-3 in first box, bits 4-7 in second
const inputRegion = DefinitionRegionWrapper.fromBoundingBoxes([
	{ min: [0, 0, 0], max: [3, 0, 0] }, // LSB
	{ min: [0, 0, 2], max: [3, 0, 2] }, // MSB
]);
```

### 3. Store Render Metadata

Use metadata to store rendering preferences:

```javascript
const inputA = DefinitionRegionWrapper.fromBounds(...)
  .setMetadata("color", "#00ff00")
  .setMetadata("label", "A[7:0]")
  .setMetadata("group", "inputs");
```

### 4. Cache Box Data

If rendering frequently, cache the box data instead of calling `getBoxes()` every frame:

```javascript
class CachedRegionRenderer {
	constructor(region) {
		this.boxes = region.getBoxes(); // Cache once
		this.center = region.centerF32();
		this.bounds = region.getBounds();
	}
}
```

### 5. Use `intersects_bounds` for Culling

Before expensive operations, check if the region is relevant:

```javascript
if (region.intersectsBounds(...viewportBounds)) {
	// Only render if visible
	renderRegion(region);
}
```

### 6. Simplify After Boolean Operations

Boolean operations can create many small boxes. Call `simplify()` when done:

```javascript
let result = regionA.union(regionB);
result = result.subtracted(regionC);
result.simplify(); // Merge adjacent boxes
```
