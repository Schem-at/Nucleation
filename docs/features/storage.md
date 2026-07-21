# Pluggable storage

## Pluggable storage


A library of builds: any schematic saves and loads through one URI, across
memory, filesystem, S3, Redis, and Postgres backends:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/storage-gallery.png" width="820" alt="A shaded sphere, a rainbow torus, an oak tree, and a sandstone pyramid: four saved schematics">

Two layers: `StoreIo` moves whole schematics, `Store` is a raw key-value store
over the same backends.

```python
# Whole schematics, by URI (format inferred from the path, or defaulted):
StoreIo.save(castle, "file:///data/castle.schem", "")
castle = StoreIo.open("file:///data/castle.schem")

# Or raw key-value over any backend:
store = Store.open("mem://")           # also file:// · s3:// · redis:// · postgres://
store.put("meta/version", b"3")
store.get_b64("meta/version")          # "Mw=="
store.list("meta/")                    # ["meta/version"]
```
