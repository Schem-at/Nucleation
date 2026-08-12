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
store = Store.open("mem://")           # also file:// · ssh:// · s3:// · redis:// · postgres://
store.put("meta/version", b"3")
store.get_b64("meta/version")          # "Mw=="
store.list("meta/")                    # ["meta/version"]
```

## Remote filesystem workers

With the `store-ssh` feature, a native worker can use a filesystem on another
machine without teaching the compute pipeline about that machine:

```text
ssh://harrison@100.71.144.24/Volumes/dock_m2/library
```

`SshStore` uses the system OpenSSH client, so keys, agents, host verification,
Tailscale addresses, `ProxyJump`, and other policy remain in `~/.ssh/config`.
It uses atomic temporary-file replacement and multiplexes connections. Keys
are always relative to the configured root and traversal is rejected.

For large extraction jobs, the compiled `segment_world` example does the hot
Anvil/segmentation work and writes schematics plus per-build provenance through
`Store`. `examples/distributed_world_extract.py` is the optional Python control
plane: it creates deterministic, resumable shards and invokes that Rust worker.
Every shard receives the same global partition definition, so build IDs do not
depend on the number of machines or shard size.

The input side uses the same abstraction. `StoreRegionTiles` treats a Store
prefix containing Anvil `region/r.X.Z.mca` objects as a random-access tile
source. Rectangle filtering happens before `get`, and one MCA is buffered at a
time. This is the preferred compute/storage split: expand a compressed backup
once on the storage node, then let workers fetch only the regions assigned to
their shards rather than copying or loading the complete world.
