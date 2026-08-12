# Embedded schematic provenance

Nucleation schematics can carry a typed `SchematicProvenance` record describing
where their content came from. This is the common contract for extraction,
cataloguing, world generation, conversion, and language bindings; callers do
not need a format-specific metadata convention.

## Contract

The current `schema_version` is `1`. Coordinates are absolute Minecraft block
coordinates. Bounding boxes are inclusive, and `origin` is the world coordinate
that corresponds to schematic-local `(0, 0, 0)`.

```json
{
  "schema_version": 1,
  "source_id": "map:creative-mc26",
  "world_name": "Creative",
  "map_name": "Creative-MC26.1.2-DH",
  "dimension": "minecraft:overworld",
  "snapshot_id": "Creative-MC26.1.2-DH.tar.zst",
  "world_bbox": { "min": [5, 62, 1105], "max": [17, 85, 1114] },
  "origin": [5, 62, 1105],
  "partition_id": "plot:1:11",
  "stable_build_id": "abc123",
  "extracted_at": 1786400000,
  "config_hash": "…",
  "profile_hash": "…",
  "attributes": {
    "nucleation:partition_owner": "ExamplePlayer",
    "nucleation:partition_trusted": "BuilderOne,BuilderTwo",
    "nucleation:partition_catalog_hash": "…"
  }
}
```

`source_id` is the required stable identity. Paths belong in a namespaced
attribute when they are useful diagnostics; moving an archive must not change
the identity of every extracted build. Extension keys must contain `:` so two
producers cannot accidentally claim the same unqualified name.

Spatial extractors use `partition_id` as the stable join key for a plot, claim,
parcel, or other caller-defined region. Optional scalar catalogue fields are
stored under `nucleation:partition_<field>`; common examples are `owner`,
`trusted`, `members`, `alias`, and `flags`. The accompanying
`nucleation:partition_catalog_hash` identifies the exact ownership/claim
snapshot used by the run. These remain attributes rather than fixed top-level
fields because ownership systems differ, while the convention is shared by all
Nucleation formats and language bindings through `SchematicProvenance`.

## Storage and round trips

- Native Nucleation serialization stores the typed object in `Metadata`.
- Sponge `.schem` v2/v3 stores canonical JSON at
  `Metadata.NucleationProvenance`.
- `.litematic` uses the same key and JSON inside its `Metadata` compound.
- Formats without an extensible metadata location can be converted through
  Nucleation, but that target format cannot promise embedded preservation.

The separate extraction JSONL manifest is intentionally retained: embedded
metadata makes each schematic self-describing, while the manifest provides a
queryable catalog without opening every schematic.

## APIs

Rust uses `schematic.metadata.provenance: Option<SchematicProvenance>`.
Generated language bindings expose `provenanceJson()`,
`setProvenanceJson(json)`, and `clearProvenance()` on `Schematic` (spelling is
adapted to each language).

Validation rejects unsupported schema versions, empty `source_id` values,
reversed bounds, and unnamespaced extension keys.
