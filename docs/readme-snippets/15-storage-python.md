# Pluggable storage (Python)

```python
import tempfile
from nucleation import Schematic, Store, StoreIo

s = Schematic.create("castle")
s.fill_cuboid(0, 0, 0, 4, 4, 4, "minecraft:stone")

# StoreIo moves whole schematics by URI (format inferred from the path):
d = tempfile.mkdtemp()
StoreIo.save(s, "file://" + d + "/castle.schem", "")
print("StoreIo round-trip blocks:", StoreIo.open("file://" + d + "/castle.schem").block_count())

# Store is a raw key-value store over the same backends:
store = Store.open("mem://")            # also file:// · s3:// · redis:// · postgres://
store.put("meta/version", b"3")
print("get_b64:", store.get_b64("meta/version"))
print("exists:", store.exists("meta/version"), "| list:", store.list("meta/"))
```

Output:

```text
StoreIo round-trip blocks: 125
get_b64: Mw==
exists: True | list: ["meta/version"]
```

_Environment: CPython 3.14.6 + nucleation 0.3.10 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- `get_b64` returns base64 (Mw== decodes to b"3"). Non-mem backends (s3://, redis://, postgres://) need their cargo features enabled in the build. Store also has put_if_absent, delete, and list_paginated. -->
