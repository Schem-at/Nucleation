# Bindings and languages

## One API, seven languages


Every binding is generated from one annotated-Rust source of truth
([`src/bridge/`](../../src/bridge/)) via
[Diplomat](https://github.com/rust-diplomat/diplomat), then committed,
regenerated, and diffed in CI so they can never drift:

| Language | Package | Errors | Naming |
| --- | --- | --- | --- |
| Rust | `nucleation` crate (native API) | `Result` | `snake_case` |
| JavaScript | `npm install nucleation` | exceptions | `setBlock` |
| Python | `pip install nucleation` | exceptions | `set_block` |
| Kotlin/JVM | Release JAR (JNA, 5 platforms bundled) | `kotlin.Result` | `setBlock` |
| PHP | Release archive (`php/` + FFI) | `DiplomatError` | `setBlock` |
| C | Release archive (`include/` + library) | result structs | `Schematic_set_block` |
| C++ | Header-only over the C ABI | `diplomat::result` | `set_block` |

What ships where:

| Channel | Surface |
| --- | --- |
| npm | full surface; WASM includes simulation + meshing (no GPU rendering) |
| PyPI | full surface, including simulation, meshing, rendering, scripting |
| Release archives + JAR | full surface, native, 5 platforms |
| crates.io | full surface except `simulation`* |

\* MCHPRS isn't on crates.io; for simulation in Rust, use
`nucleation = { git = "https://github.com/Schem-at/Nucleation", features = ["simulation"] }`.

<details>
<summary><b>Kotlin, PHP, and C quickstarts</b></summary>

```kotlin
import at.schem.nucleation.*

val schematic = Schematic.create("demo")
schematic.setBlock(1, 2, 3, "minecraft:stone").getOrThrow()
println(schematic.getBlockName(1, 2, 3).getOrThrow()) // "minecraft:stone"
```

```php
<?php
require "php/index.php";
use Stencil\Lib;
use Stencil\Schematic;

Lib::init("/path/to/libnucleation.so");
$schematic = Schematic::create("demo");
$schematic->setBlock(1, 2, 3, "minecraft:stone");
echo $schematic->getBlockName(1, 2, 3); // "minecraft:stone"
```

```c
#include "Schematic.h"

int main(void) {
    DiplomatStringView name = {"demo", 4};
    Schematic *s = Schematic_create(name);
    DiplomatStringView stone = {"minecraft:stone", 15};
    Schematic_set_block(s, 1, 2, 3, stone);
    Schematic_destroy(s);
    return 0;
}
```

</details>
