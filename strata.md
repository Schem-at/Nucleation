# VVCF — Voxel Version Control Format

**Working name:** VVCF (codename *Strata*)
**Specification version:** 0.1 (draft)
**Status:** Draft for discussion — not yet stable
**Identifier:** `vvcf/0`
**License of this specification:** CC0 / public domain (recommended, so the format is freely implementable)

> A vendor-neutral, game-neutral format for **content-addressed version control of voxel builds**. VVCF defines how a voxel structure is canonically serialized, hashed, stored as an immutable object graph, diffed, merged, and synchronized — independent of any one engine, game, or service.
>
> Minecraft semantics live in a *profile* (Appendix A), not in the core. Nucleation and schemat.io are a *reference implementation* (Appendix B), not the standard.

---

## 1. Introduction

### 1.1 Rationale

Voxel builds are edited continuously, shared widely, and remixed. Today they live as opaque, lossy, non-deterministic files (gzipped NBT schematics, engine-specific blobs) with no shared notion of history, identity, or change. VVCF provides, for voxels, what Git provides for text:

- **Stable identity** — the same build always hashes to the same address, across tools, languages, and games.
- **History** — an immutable commit graph: who changed what, when, from what parent.
- **Change** — a well-defined diff between any two versions, renderable and mergeable.
- **Distribution** — content-addressed objects sync by copying what the other side lacks.

### 1.2 Design principles

1. **Content addressing first.** Every object is named by the cryptographic hash of its canonical bytes. Identity is intrinsic, not assigned.
2. **Determinism is the contract.** Two conformant encoders MUST produce byte-identical canonical bytes for the same logical content. Without this, addressing forks.
3. **Interchange ≠ object form.** Existing engine formats remain the interchange layer; VVCF is the internal canonical form, convertible to/from them.
4. **Game-neutral core, profiled specifics.** The core models *voxels, layers, metadata, history*. Game-specific identifiers and metadata schemas live in profiles.
5. **Layered conformance.** A tool may implement only storage, or storage+history, or also diff/sync. Each is a defined conformance class.
6. **Hash-agility and extensibility.** Object IDs are self-describing (multihash); unknown fields and object kinds degrade gracefully.

---

## 2. Terminology & conventions

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

- **Voxel** — a single cell of the integer lattice with a *voxel type* (and optional metadata).
- **Voxel type** — a namespaced identifier plus an ordered set of state properties (§4.2).
- **Layer** — one named, parallel grid of values over the same lattice (e.g. the primary voxel-type layer; optionally a biome layer) (§4.3).
- **Chunk** — a fixed-size cubic sub-region of the lattice; the unit of storage and dedup (§4 / §6.3).
- **Build / snapshot** — a complete voxel structure at one point in time.
- **Object** — an immutable, content-addressed unit: chunk, tree, commit, or tag.
- **Object ID (OID)** — the multihash of an object's canonical bytes (§6.2).
- **Repository** — an object store plus a set of refs (§7).
- **Ref** — a mutable named pointer to a commit (branch or tag) (§8.2).
- **Profile** — a normative, named binding that maps a specific game/engine's semantics onto the core model (Appendix A).
- **Normative / Informative** — sections marked *Informative* describe non-binding guidance.

---

## 3. Scope & conformance classes

VVCF defines four cumulative conformance classes. An implementation MUST declare which it supports.

| Class | Name | Requires |
|---|---|---|
| **C1** | Object Store | §4 data model, §6 canonical serialization, §7 repository, §13 security |
| **C2** | History | C1 + §8 commits/refs/branches/tags + fast-forward merge |
| **C3** | Diff | C2 + §9 diff representation (produce/consume) |
| **C4** | Sync | C2 + §11 synchronization |

Optional extensions (§10 identity/similarity, §8.3 three-way merge, §9.3 move detection) are independently declarable and do not gate any class.

A **profile** (e.g. Minecraft, Appendix A) is REQUIRED for two implementations to interoperate on actual voxel content, because the core does not define concrete voxel-type identifiers or metadata schemas.

---

## 4. Abstract data model

### 4.1 Coordinate space

A build occupies a subset of the 3D integer lattice ℤ³. Axes are labelled **X, Y, Z**. The geometric meaning of each axis (up, north, …) and the handedness are **profile-defined**. The canonical packing order (§6.3) is fixed by this specification and is independent of axis meaning.

### 4.2 Voxel type & palette

A **voxel type** is:

```
type := { id: string, state: { key: string -> value: string } }
```

- `id` is a namespaced identifier, RECOMMENDED form `namespace:name` (e.g. `minecraft:oak_stairs`).
- `state` is an unordered map of string→string properties (e.g. `facing -> east`). Profiles MAY define typed values; the core treats values as opaque strings.
- The canonical form of `state` orders keys bytewise (§6).

A **palette** is the ordered, deduplicated list of voxel types occurring in a chunk/layer. The empty/absence type (e.g. air) is represented by *absence* (sparse), not by a palette entry, unless a profile requires otherwise.

### 4.3 Layers

A build has one or more **named layers** over the same lattice. Exactly one layer is the **primary** voxel-type layer (`vvcf:voxel`). Profiles MAY define additional layers (e.g. `vvcf:biome`). Each layer has its own palette and index grid. This generalizes "blocks + biomes" without privileging either.

### 4.4 Voxel metadata

A voxel MAY carry **metadata**: an arbitrary structured value (the abstract equivalent of an NBT compound / a CBOR map). Metadata is attached by lattice position. Its schema is profile-defined; the core requires only that it be representable as canonical CBOR (§6.1) so it can be hashed deterministically.

### 4.5 Entities (OPTIONAL)

A build MAY contain **entities**: free agents with rational/continuous positions and structured metadata, not bound to a lattice cell. Entities are stored at build level in the core (they straddle chunk boundaries). A profile MAY bind them to sub-structures (see §12 / Appendix A).

### 4.6 Build / snapshot

A build is: a set of non-empty chunks (each carrying its layers and per-voxel metadata), plus an OPTIONAL entity set, plus structural metadata (bounds, placement, profile id, profile content-version).

### 4.7 Normalized frame

A build is stored in a **normalized frame**: translated so its minimum occupied corner is the origin. World/scene placement (origin offset and rotation) is recorded in commit metadata, NOT in the chunk bytes. This makes identical sub-volumes hash-equal regardless of placement and prevents relocation from churning the content.

---

## 5. Object model

### 5.1 Object kinds

VVCF defines four object kinds. All are immutable and content-addressed.

- **chunk** — one fixed-size cubic region: its layers (palettes + packed indices) and per-voxel metadata. The bulk of the data.
- **tree** — the structure of a build: a typed list of entries. In `vvcf/0` the only entry kind is `Chunk{ coord, oid }`. Reserved entry kinds (e.g. `SubBuild{ name, ref, placement }`) enable composition/nesting in future versions (§12). A tree MAY also reference an entities object.
- **commit** — a tree OID, zero or more parent commit OIDs (forming a DAG), and metadata (author, time, message, placement, profile, identity hints).
- **tag** — an OPTIONAL annotated, signable pointer to a commit (lightweight tags are refs, §8.2).

> Note: there is intentionally **no working-tree or index object**. VVCF versions *snapshots*; the live editing surface belongs to the host application.

### 5.2 Object identity

The OID of an object is the **multihash** (§6.2) of its canonical serialization (§6). References between objects (tree→chunk, commit→tree, commit→parent) are by OID. The object graph is therefore a Merkle DAG: any change to any chunk changes its OID, propagating new OIDs up the tree and commit.

---

## 6. Canonical serialization (NORMATIVE)

This is the heart of the standard: the byte-exact rules that make addressing reproducible.

### 6.1 Container: Deterministic CBOR

Objects are encoded as **CBOR** (RFC 8949) using **Core Deterministic Encoding** (RFC 8949 §4.2.1):

- integers use the shortest form;
- all lengths are definite;
- map keys are sorted by bytewise lexicographic order of their encoded form;
- no duplicate map keys; no indefinite-length items; no floating-point NaN payloads.

Additional VVCF rules:

- Map keys in core objects are short ASCII strings defined by this spec.
- Floating-point values are forbidden in core objects (entity positions, if present, use a profile-defined fixed-point or rational encoding to avoid float nondeterminism).
- Strings are UTF-8, NFC-normalized.
- Unknown keys MUST be preserved by storage-only tools and MUST NOT be reordered (deterministic encoding already fixes order).

> *Informative:* deterministic CBOR is chosen over inventing a wire format because it is language-neutral, widely implemented, and has a published canonical-encoding profile. Implementations MAY internally use other forms but MUST hash the deterministic-CBOR bytes.

### 6.2 Hashing & object IDs

- Default hash: **BLAKE3-256**.
- An OID is a **multihash**: `varint(hash-code) ‖ varint(length) ‖ digest`, enabling hash-agility. Conformant `vvcf/0` implementations MUST support BLAKE3-256 and MAY support additional codes.
- Object IDs are presented to users in a **multibase** text form (RECOMMENDED `base32`).
- An object is hashed as: `H( type-tag ‖ canonical-CBOR-body )`, where `type-tag` is a 1-byte object-kind discriminant. (Length is implicit in the CBOR framing.)
- *Informative:* this layout is compatible with IPLD-style content addressing; bridging to IPLD/CID is possible but not required.

### 6.3 Chunk encoding

A chunk is a fixed cubic edge length **E** (the *chunk size*), profile-defined (Minecraft: 16). For each layer:

- `palette`: array of canonical voxel-type entries, sorted by (`id`, then canonical `state`).
- `indices`: a bit-packed array of `E³` palette indices, bit width = `ceil(log2(max(1, |palette|)))`, traversal order **X fastest, then Z, then Y** (`i = x + E*(z + E*y)`), MSB-first packing, no inter-element padding.
- empty layer ⇒ omitted.

Per-voxel metadata: an array of `{ index: uint, value: <canonical CBOR> }`, sorted by `index`.

A chunk with no non-empty layers MUST NOT be created (builds are sparse).

### 6.4 Tree encoding

```
{ "v": "vvcf/0",
  "entries": [ { "k": 0, "coord": [cx,cy,cz], "oid": <bytes> }, ... ],   # k=0 ⇒ Chunk; sorted by coord
  "entities": <oid>?,     # optional
  "bounds": [ [minx,miny,minz], [maxx,maxy,maxz] ] }
```

Entries are sorted by `coord` (X, then Y, then Z). `k` is the entry-kind discriminant; readers MUST ignore entries with unknown `k` only insofar as a profile/version permits (storage-only tools preserve them).

### 6.5 Commit encoding

```
{ "tree": <oid>,
  "parents": [ <oid>, ... ],          # 0 = root, 1 = normal, ≥2 = merge
  "author": { "name": str, "id": str? },
  "time": int,                         # epoch seconds, UTC
  "message": str,
  "placement": { "t": [x,y,z], "r": <rigid-op> },   # scene placement; identity default
  "profile": { "id": str, "content_version": int }, # e.g. minecraft / data_version
  "identity": { "content": <oid>, "fingerprint": bytes?, "signature": bytes? },  # §10, optional fields
  "meta": { ... } }                    # profile/host extras
```

`placement.r` (rigid op) is drawn from the symmetry vocabulary in §9.2. Volatile fields (author/time/message) live ONLY here, never in chunk/tree bytes, so identical geometry dedups regardless of who committed it.

### 6.6 Refs

A ref is a small text record holding one OID (multibase), OPTIONALLY with a reflog. Refs are mutable and are NOT content-addressed.

### 6.7 Compression

Compression is a **storage/transport concern applied AFTER hashing**. The OID is computed over the *uncompressed* canonical bytes. Implementations SHOULD use a deterministic-friendly codec (e.g. zstd) but MUST NOT let compression affect identity. This deliberately avoids the gzip-nondeterminism that makes legacy schematic files un-addressable.

---

## 7. Repository model & layout

### 7.1 Object store interface (abstract)

A repository is any key→bytes store supporting: `get(key)`, `put(key, bytes)`, `exists(key)`, `list(prefix)`, `delete(key)`. Objects are keyed by OID; refs by name. Any backend satisfying this interface (filesystem, object storage, KV store, database) is conformant.

### 7.2 Filesystem profile (RECOMMENDED layout)

```
HEAD                      → "ref: refs/heads/main"
config                    → repository settings (default profile, equivalence, …)
refs/heads/<branch>       → <OID>
refs/tags/<tag>           → <OID>
objects/<ab>/<rest>       → compressed canonical object bytes (sharded by OID prefix)
```

Other backends (S3, Postgres, KV) MAY use an equivalent key scheme. The layout is not part of object identity.

---

## 8. History semantics

### 8.1 Commits & the DAG

Commits form a directed acyclic graph via `parents`. A commit's reachable set is itself plus all ancestors. *Ancestry* and *merge-base* (lowest common ancestors) are defined as in a standard DAG.

### 8.2 Refs, branches, tags

- A **branch** is a ref under `refs/heads/` that advances as commits are added.
- A **lightweight tag** is a ref under `refs/tags/` pointing at a fixed commit.
- An **annotated tag** is a `tag` object (signable) referenced by such a ref.
- `HEAD` names the current branch (host concept; OPTIONAL for non-interactive stores).

### 8.3 Merge & conflicts

- **Fast-forward (C2, REQUIRED):** if one side's tip is an ancestor of the other, merging is advancing the ref; no new commit content.
- **Three-way (OPTIONAL extension):** given `base = merge-base(ours, theirs)`, compute `diff(base, ours)` and `diff(base, theirs)` (§9):
  - changes affecting **disjoint** voxels compose automatically;
  - changes affecting the **same** voxel divergently are **conflicts**, reported as spatial regions (§9.1).
  A three-way merge produces a merge commit (≥2 parents). The representation of an unresolved conflict (annotation vs. side-by-side) is left to §9/profile and is currently an open point.

### 8.4 Concurrency

Ref updates MUST be **compare-and-swap**: `update_ref(name, expected_old, new)` fails if the current value ≠ `expected_old`. Objects (immutable, content-addressed) MAY be written eagerly and deduped; only the final ref CAS serializes concurrent writers. The core holds no locks.

---

## 9. Diff representation (C3)

VVCF standardizes the *result* of a diff so tools interoperate. The *algorithm* is informative.

### 9.1 Change-set model (NORMATIVE result)

A diff from build A to build B, under an *equivalence profile* (§9.2) and a chosen rigid transform `T` mapping A's frame onto B's, is:

```
{ "from": <oidA>, "to": <oidB>,
  "transform": { "t": [x,y,z], "r": <rigid-op> },
  "added":   [ { "pos":[x,y,z], "voxel": <type> }, ... ],          # in B only
  "removed": [ { "pos":[x,y,z], "voxel": <type> }, ... ],          # in A only
  "changed": [ { "pos":[x,y,z], "from": <type>, "to": <type> }, ... ],
  "swapped": [ { "pos":[x,y,z], "from": <type>, "to": <type> }, ... ], # palette-level substitution
  "palette_swaps": [ [ <fromTypeKey>, <toTypeKey> ], ... ],
  "distance": uint,            # cost-weighted edit distance
  "support": float }           # fraction of the larger build that aligned (0..1)
```

All position arrays MUST be sorted by `(x,y,z)` for canonical, reproducible output. A *region* is a connected component (6-connectivity) of changed positions, used for summaries and conflict reporting.

### 9.2 Rigid alignment & equivalence profiles

- **Rigid ops** are the discrete symmetries of the lattice (identity, the axis-aligned rotations, and OPTIONAL reflections). The applicable **symmetry group** is part of the equivalence profile.
- An **equivalence profile** decides what "the same voxel" means: e.g. exact (id+state identical), shape (occupancy only), or domain-specific (functional). Profiles are named; the Minecraft profile (Appendix A) defines a standard set.
- A diff selects the transform `T` (over the symmetry group plus a translation found by alignment) minimizing the cost-weighted distance. The cost model (per-op weights) MAY be overridden.

*Informative:* a practical alignment uses anchor voting plus FFT cross-correlation for the translation, iterating over the symmetry group; see the reference implementation.

### 9.3 Move / rotate detection (Informative, OPTIONAL)

A higher-quality diff MAY recognize that a contiguous sub-structure was relocated/rotated rather than deleted+added: cluster removed/added regions, match them by transform-invariant shape fingerprint, recover the per-component rigid op via alignment, and (for repeated identical sub-structures) solve a deterministic minimum-cost assignment. Such moves are presented as `move{component, transform}` entries. This is presentation/merge quality only; it MUST NOT affect object storage or identity.

---

## 10. Identity & similarity (OPTIONAL extension)

Three derived values, all recordable in commit `identity`:

- **content OID** (NORMATIVE for storage) — exact-bytes identity.
- **fingerprint** — an equivalence-class hash under a profile (rotation/translation/palette-invariant per the profile). Detects equivalent rebuilds; changes when content is edited. Format: a fixed-length digest defined per equivalence profile.
- **signature** — a coarse, comparable feature descriptor (e.g. dimensions + a typed-token histogram) supporting *similarity* search and nearest-neighbor indexing.

Matching an arbitrary build to existing history SHOULD cascade: content OID (exact) → fingerprint (equivalent) → signature-shortlisted similarity (edited descendant). The similarity rung is heuristic and SHOULD be confidence-gated. Indexing/searching these values is a host concern, not part of this spec.

---

## 11. Synchronization (C4)

A pull/push between two repositories is a Merkle-DAG set reconciliation:

1. **Negotiate refs:** exchange ref→OID maps.
2. **have/want:** the receiver walks wanted commits' reachable objects and requests those whose OIDs it lacks (the DAG bounds the walk: a present OID implies all its descendants are present).
3. **Bundle:** the sender serializes the requested objects into a **bundle** — a framed sequence of `(OID, compressed canonical bytes)` with a header `{ "v":"vvcf/0", "objects": uint }`. The receiver MUST verify each object's bytes hash to its claimed OID before storing (§13).
4. **Apply refs:** the receiver fast-forwards refs where possible; divergent refs require three-way merge (§8.3) and are otherwise rejected (non-fast-forward).

The transport (HTTP, etc.) is out of scope; the **bundle format** is normative so any two C4 tools interoperate.

---

## 12. Extensibility & spec versioning

- Objects carry the spec id (`vvcf/0`). Incompatible changes bump the major (`vvcf/1`).
- The **tree entry list is typed and open**: new entry kinds (notably `SubBuild{ name, ref, placement }` for nested/recursive build dependencies and Litematica-style multi-region builds) are additive. Flat `vvcf/0` builds continue to hash identically because their entries contain only `Chunk` kinds.
- **`placement` is a shared type** reused by commit placement, detected moves, and future sub-build placement — relocating a nested module becomes a single `placement` change rather than a mass delete+add.
- **Entities** migrate from build-level (core) to per-sub-build ownership when sub-builds exist; this is a relocation of the entities reference, not a reshape.
- Unknown OPTIONAL fields and object kinds MUST be preserved by storage-only tools.

---

## 13. Security considerations

- **Untrusted objects:** a receiver MUST verify that every object's canonical bytes hash to its claimed OID before trusting or storing it. Hash mismatch ⇒ reject.
- **Decompression safety:** implementations MUST bound decompressed size and nesting (zip-bomb / CBOR-bomb resistance); reject objects exceeding configured limits.
- **Resource limits:** chunk dimensions, palette sizes, and metadata sizes SHOULD be bounded per profile.
- **Hash agility:** OIDs are multihashes so a future migration away from a weakened hash is possible; tools SHOULD record which hash they used.
- **Signatures:** annotated tags and commits MAY be signed (detached signature over the OID); signature schemes are profile/host-defined.
- **Metadata trust:** voxel/entity metadata is arbitrary structured data from untrusted sources; consumers MUST treat it as data, never as executable input.

---

## 14. Media types (suggested, for registration)

- `application/vnd.vvcf.object` — a single canonical object.
- `application/vnd.vvcf.bundle` — a sync bundle (§11).
- `application/vnd.vvcf.diff+cbor` / `+json` — a serialized diff (§9).

---

## Appendix A — Minecraft profile (NORMATIVE for Minecraft tools)

- **Profile id:** `minecraft`. `content_version` = Minecraft `DataVersion`.
- **Axes:** Y up; X east; Z south (Minecraft convention). Canonical packing order remains X→Z→Y per §6.3.
- **Chunk size E = 16** (matches a section).
- **Voxel type:** `id` is the block id; `state` is the blockstate property map. Air (`minecraft:air`, `cave_air`, `void_air`) is *absence* (sparse), never a palette entry.
- **Metadata:** block-entity NBT, mapped to canonical CBOR by: NBT compound→CBOR map (keys sorted per §6.1), NBT list→CBOR array (element type pinned), NBT numeric types→tagged CBOR integers preserving width, NBT strings→UTF-8. The mapping MUST round-trip.
- **Layers:** `vvcf:voxel` = blocks; `vvcf:biome` = biome data (3D where present).
- **Entities:** Minecraft entities, positions as fixed-point (profile-defined scale) to avoid float nondeterminism.
- **Equivalence profiles:** `exact`, `shape`, `structural`, `redstone_computational` (alias `redstone`), `redstone_survival`, with the symmetry group and token rules defined by the profile.
- **Cross-version:** builds with different `DataVersion` are distinct objects (lossless). A data-fixer MAY be applied at read time for semantic comparison; it MUST NOT alter stored bytes.
- **Interchange:** `.schem`, `.litematic`, `.nbt`, `.mcstructure` import/export; multi-region `.litematic` maps onto future `SubBuild` entries (§12).

---

## Appendix B — Reference implementation (Informative)

**Nucleation** (Rust core with WASM/JS, Python, C-FFI, JVM, PHP bindings) implements the Minecraft profile and conformance classes C1–C4. **schemat.io** is a hosted service built on it. These validate the spec but do not define it; any conformant implementation interoperates with them at the object/bundle/diff level.

---

## Appendix C — Conformance checklist

- [ ] **C1** deterministic CBOR (§6.1); multihash OIDs, BLAKE3-256 (§6.2); chunk/tree encoding (§6.3–6.4); compression after hashing (§6.7); object-store interface (§7); hash verification on ingest (§13).
- [ ] **C2** commit DAG, refs, branches, tags (§8.1–8.2); fast-forward merge (§8.3); CAS ref updates (§8.4).
- [ ] **C3** canonical diff result, sorted positions (§9.1); rigid alignment + equivalence profiles (§9.2).
- [ ] **C4** have/want negotiation, bundle format with per-object hash verification (§11).
- [ ] **Profile** a declared profile (e.g. Appendix A) for content-level interop.
- [ ] **Optional** three-way merge (§8.3); move detection (§9.3); identity/similarity (§10).

---

## Open issues (pre-1.0)

1. Conflict representation in three-way merge (annotation vs. dual-tree).
2. Exact fingerprint digest format per equivalence profile.
3. Signature vector schema (shared with similarity search / classification).
4. Whether to formally bridge OIDs to IPLD CIDs.
5. Reflog format and whether it is in-scope.
6. Canonical fixed-point scale for entity positions (per profile vs. core default).