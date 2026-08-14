# Bindings and languages

Seven language surfaces share the annotated Rust definitions in
[`src/bridge/`](https://github.com/Schem-at/Nucleation/tree/master/src/bridge).
[Diplomat](https://github.com/rust-diplomat/diplomat) generates the foreign
bindings. Generated files are committed, regenerated in CI, and checked for
drift.

## Packages and conventions

| Language | Package | Errors | Naming |
|---|---|---|---|
| Rust | `nucleation` crate | `Result` | `snake_case` |
| JavaScript | `npm install nucleation` | exceptions | `setBlock` |
| Python | `pip install nucleation` | exceptions | `set_block` |
| Kotlin/JVM | release JAR, JNA with five native platforms | `kotlin.Result` | `setBlock` |
| PHP | release archive with FFI library | `DiplomatError` | `setBlock` |
| C | release archive with headers and library | result structs | `Schematic_set_block` |
| C++ | headers over the C ABI | `diplomat::result` | `set_block` |

## Distribution limits

| Channel | Surface |
|---|---|
| npm | WASM with simulation and meshing; no GPU renderer and no local filesystem |
| PyPI | simulation, meshing, rendering, and scripting |
| Release archives and JAR | native surface for five platform targets |
| crates.io | all published features except `simulation` |

MCHPRS is not published on crates.io. Rust callers that need its redstone
backend use the Git dependency and enable `simulation`:

```toml
nucleation = { git = "https://github.com/Schem-at/Nucleation", features = ["simulation"] }
```

## Native targets

The release workflow builds these targets:

```text
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin
x86_64-pc-windows-msvc
wasm32-unknown-unknown
```

## Short examples

<details>
<summary>Kotlin, PHP, and C</summary>

```kotlin
import at.schem.nucleation.*

val schematic = Schematic.create("demo")
schematic.setBlock(1, 2, 3, "minecraft:stone").getOrThrow()
println(schematic.getBlockName(1, 2, 3).getOrThrow())
```

```php
<?php
require "php/index.php";
use Stencil\Lib;
use Stencil\Schematic;

Lib::init("/path/to/libnucleation.so");
$schematic = Schematic::create("demo");
$schematic->setBlock(1, 2, 3, "minecraft:stone");
echo $schematic->getBlockName(1, 2, 3);
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
