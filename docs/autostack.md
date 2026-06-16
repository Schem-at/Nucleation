# Auto-stack — periodicity detection & resize

Detect the repeating structure(s) in a build and resize them by re-stamping the
fundamental domain a different number of times along its lattice vector. A 4-bit
adder becomes 8-bit; a 32×32 screen becomes 64×64; an N-module bore head becomes
2N.

The design, maths, and worked examples are in the companion document
**`auto-stack.tex`** (build with `tectonic auto-stack.tex`). This page is the API
reference.

Enable with the **`autostack`** Cargo feature (pure-voxel; no extra dependencies).

```toml
nucleation = { version = "*", features = ["autostack"] }
```

---

## Concepts

`detect_structures` ranks the repeating structures in a build by **region
coverage** — how much of the build is locally periodic under a candidate period
— so it is robust to surrounding non-periodic infrastructure (a screen is found
even when a decoder breaks the global run). Each result is a `Structure`:

| field | meaning |
|---|---|
| `mode` | `"1d"` or `"2d"` |
| `vectors` | one period vector `[x,y,z]` for 1D, two for 2D |
| `coverage` | fraction of the build (0–1) explained by this period |
| `region_min` / `region_max` | bounding box of the periodic region |
| `cell_min` / `cell_max` | bounding box of one representative unit cell |
| `label` | e.g. `"2D array · Z×Y · 92% of build"` |

Resizing tiles the unit cell while keeping the boundary (head/tail for 1D;
nine-slice corners/edges/interior for 2D). Resizing to the original unit count
reproduces the input exactly.

---

## Rust

```rust
use nucleation::{UniversalSchematic, autostack};

let s = /* a UniversalSchematic */;

// 1. detect
let structs = autostack::detect_structures(&s);       // Vec<Structure>, ranked
let json    = autostack::detect_structures_json(&s);  // same, as JSON

// 2. resize
let bigger = autostack::resize(&s, &structs[0], &[8]).unwrap();          // 1D → 8 cells
let screen = autostack::resize(&s, &structs[0], &[16, 16]).unwrap();     // 2D → 16×16

// or explicitly, by vector:
let a = autostack::resize_1d(&s, [2, -2, 0], 12).unwrap();               // diagonal
let b = autostack::resize_2d(&s, [0, 2, 0], [0, 0, 2], 16, 16).unwrap();
```

## Python

```python
from nucleation import Schematic

s = Schematic.open("adder.schem")

structs = s.detect_structures()      # list of dicts (mode, vectors, coverage, …)
top = structs[0]
print(top["label"])                  # "1D run · Y · 96% of build"

# 1D / diagonal — pass the period vector + new cell count
bigger = s.autostack_resize_1d(*top["vectors"][0], 8)

# 2D — two period vectors + (n1, n2)
v1, v2 = top["vectors"]              # when top["mode"] == "2d"
screen = s.autostack_resize_2d(*v1, *v2, 16, 16)

bigger.save("adder_8bit.schem")
```

## JavaScript / WASM

```js
import { SchematicWrapper } from "nucleation";

const s = new SchematicWrapper();
s.from_data(bytes);

const structs = s.detectStructures();          // array of { mode, vectors, coverage, … }
const top = structs[0];

const bigger = s.autostackResize1d(...top.vectors[0], 8);
const screen = s.autostackResize2d(...top.vectors[0], ...top.vectors[1], 16, 16);
```

## C / FFI

```c
// JSON array of structures; free with free_string()
char* json = schematic_detect_structures(schematic);

// resize → new schematic (free with schematic_free); NULL on error
SchematicWrapper* bigger = schematic_autostack_resize_1d(schematic, 0, 2, 0, /*units*/8);
SchematicWrapper* screen = schematic_autostack_resize_2d(
    schematic, 0,2,0,  0,0,2,  /*n1*/16, /*n2*/16);
```

## PHP

```php
$s = Nucleation\Schematic::open("adder.schem");
$structs = json_decode($s->detect_structures(), true);
$v = $structs[0]["vectors"][0];
$bigger = $s->autostack_resize_1d($v[0], $v[1], $v[2], 8);
$bigger->save("adder_8bit.schem");
```

The same logical methods exist across Python, WASM, FFI, and PHP
(`detect_structures`, `autostack_resize_1d`, `autostack_resize_2d`); the
Python/WASM/FFI trio is checked by `tools/check_api_parity.rs`.

## Shipping & feature flags

`autostack` is wired into the feature graph so it ships with every binding
automatically — **no build script or CI change is needed**:

```toml
default = ["store-fs", "autostack"]
ffi     = ["autostack"]
python  = [..., "autostack"]
wasm    = [..., "autostack"]
php     = ["ext-php-rs", "autostack"]
```

So the stock autobuild commands (`maturin build --features python,simulation,rendering`,
`wasm-pack build --features wasm,simulation,meshing`, `cargo build --features ffi,meshing`)
all include it. The JVM binding (`nucleation-jvm`) is a **separate crate** and
does not yet expose autostack.

---

## Graph-based diagonal detection (feature: `simulation`)

The pure-voxel `detect_structures` only proposes **axis-aligned** periods. A
**diagonal** datapath (e.g. a diagonal carry-chain adder) repeats along a
non-axis vector that voxel self-overlap can't easily find. `detect_structures_graph`
recovers that vector from the **redstone logic graph**: 3-round Weisfeiler-Lehman
node labels → most-common same-label displacement (gcd-reduced to a primitive,
possibly diagonal direction) → smallest translation-invariant period. The result
is an ordinary `Structure`, so it feeds the same `resize_1d` (which already
handles diagonal vectors).

```rust
let diag = autostack::detect_structures_graph(&s);   // Vec<Structure>, diagonal-capable
```
```python
diag = s.detect_structures_graph()                   # list of dicts (same shape)
```
```js
const diag = s.detectStructuresGraph();              // array of objects; [] for non-redstone
```
```c
char* json = schematic_detect_structures_graph(schematic);  // JSON array; free with free_string
```

It requires the `simulation` feature (graph extraction runs the redpiler) and
returns an empty result for non-redstone builds. A typical client merges it with
the voxel detector and dedups by vector (the browser resizer does exactly this).

## Scope

The voxel core (region-coverage detection + 1D/diagonal/2D resize) is validated
byte-for-byte against the reference prototype (`experiments/voxel-map/`) on real
builds — see `tests/autostack_test.rs` and the `autostack::tests` /
`autostack::graph_detect::tests` unit tests.

Still prototype-only (not yet in the core): near-periodic / signal-booster
resize, simulation-based functional verification, and support-aware single-cell
extraction. They layer on behind the `simulation` / `meshing` features as future
work.
