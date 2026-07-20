# Formats and I/O

## The basics


A `Schematic` is a named collection of blocks, plus block entities, entities,
and metadata, held in one or many named regions. Load one from any supported
format, edit it with plain coordinates and block strings, save it in any other:

```python
from nucleation import Schematic

cube = Schematic.load_from_file("simple_cube.litematic")   # any format, auto-detected
cube.dimensions()                                          # (3, 3, 3)

cube.set_block(1, 3, 1, "minecraft:glowstone")             # y=3: the region grows to fit
cube.get_block_name(1, 3, 1)                               # "minecraft:glowstone"

cube.save_to_file("cube.schem")                            # format from the extension
```

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/basics.png" width="380" alt="The cube from the snippet with its glowstone crown, rendered">
</div>

The same loop in JavaScript. The WASM build has no filesystem, so it is bytes
in, bytes out:

```js
import { Schematic } from "nucleation";
import { readFileSync, writeFileSync } from "node:fs";

const cube = Schematic.fromData(readFileSync("simple_cube.litematic"));
cube.setBlock(1, 3, 1, "minecraft:glowstone");
writeFileSync("simple_cube.schem", Buffer.from(cube.toSchematicB64(), "base64"));
```

Block-state strings with properties work anywhere a block is named, like
`"minecraft:lever[face=floor,facing=east]"`, and every block string a schematic
can contain round-trips. Later Python snippets assume `from nucleation import *`
and an existing schematic `s`; each has a fully runnable version with captured
output in [`docs/readme-snippets/`](../readme-snippets/).
