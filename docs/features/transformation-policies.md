# Transformation, normalization, and content policies

Nucleation processes untrusted or non-standard schematics through an explicit,
versioned transform plan:

```text
bounded decode -> inspect -> policy decision -> transform -> validate -> store
```

A plan is separate from serialization. It is applied to a clone and committed
only if no rule rejects it, so rejection leaves the source unchanged. Inspection
executes the same plan on a clone and returns the proposed findings without
committing it.

## Python

```python
from nucleation import Schematic
from nucleation import TransformPlan, apply_transform, inspect_transform

schematic = Schematic.open("untrusted.schem")
plan = TransformPlan.registry_safe()

preview = inspect_transform(schematic, plan)
if not preview.rejected and not preview.quarantined:
    report = apply_transform(schematic, plan)
    schematic.save("accepted.schem")
```

Decode untrusted input with explicit limits before applying a plan:

```python
from nucleation import DecodeLimits, decode_bounded

schematic = decode_bounded(payload, DecodeLimits(
    max_input_bytes=64 * 1024 * 1024,
    max_decompressed_bytes=256 * 1024 * 1024,
    max_volume=64_000_000,
))
```

The compiled cross-language surface accepts the same versioned JSON contract:

```python
report_json = schematic.inspect_transform_plan_json(plan.to_json())
report_json = schematic.apply_transform_plan_json(plan.to_json())
```

Policy rejection is represented by `report.rejected`; malformed plan JSON is an
API error. Reports use stable codes, paths, actions, and counts and never include
the removed or redacted value.

## Plan passes

Version 1 includes three core passes:

- `canonicalize_palette`: losslessly sort properties and palette entries,
  deduplicate equivalent states, remove unused entries, and reserve palette
  index zero for air.
- `remap_materials`: apply a named, versioned exact or color-family material
  profile.
- `content_policy`: recursively inspect and transform entities, block entities,
  nested items, text, NBT, identifiers, and UUID references.

Passes run in listed order.

## Transformation history

Successful plans append a content-addressed record to
`metadata.transformation_history`. The record contains the plan name and hash,
plan-schema version,
lossless/quarantine flags, the non-sensitive summary, and a `verification` map.
The core records plan validation, policy acceptance, and an actual transform-
twice idempotence check (`passed`, `failed`, or `not_applicable`). It deliberately has
no wall-clock timestamp, keeping deterministic output reproducible. Immediate
reapplication of the same plan is deduplicated. Source provenance is never
rewritten to record processing history.

Sponge `.schem`, Litematic, and Nucleation snapshot v3 persist the history.
Snapshot versions 1 and 2 remain readable.

## UUID policy

UUID policy is intentionally more detailed than a `strip_uuids` flag.

| Field | Choices and behavior |
|---|---|
| `mode` | `preserve`, `remove`, `regenerate_random`, or `regenerate_deterministic` |
| `representation` | Preserve the source shape or standardize to `int_array`, `string`, or `long_pair` |
| `scope` | Rewrite definitions only, definitions with references, or all recognized UUID-like keys |
| `salt` | Namespace for deterministic identities |
| `identity_basis` | `stable_path`, or `entity_location` for identities stable across entity reordering |
| `assign_missing` | Assign identities to top-level entities and nested passengers; requires a regeneration mode |
| `collision` | `warn`, `reject`, or knowingly `keep` duplicate definitions |
| `dangling` | `warn`, `remove`, `reject`, or `preserve` references without an in-schematic definition |
| `definition_keys` | Configure recognized UUID-definition field names |
| `reference_keys` | Configure owner, leash, trust, and other reference field names |

Deterministic identities derive from the selected stable identity and policy
salt, not repeatedly from the previous UUID. `stable_path` is canonical and
compact. `entity_location` uses region, entity type, and exact IEEE-754
position bits, so reordering entities does not change identities. Two entities
of the same type at the same exact position are surfaced through the configured
collision policy. The operation is therefore idempotent.
References are rewritten through the definition map so owners, passengers, and
other internal relationships keep the same identity. This includes direct
fields, nested owner/leash compounds, and UUID lists such as `Trusted`.
Unknown references are handled by `dangling`; they are never independently
randomized. A long-pair cannot be represented as one NBT-list element, so that
combination is reported and standardized to an int array for the list element.
The configurable definition and reference key sets let modded schemas opt in
without making every UUID-looking string an identity automatically.

```python
from nucleation import ContentPolicy, TransformPlan, UuidPolicy, apply_transform

uuid_policy = UuidPolicy(
    mode="regenerate_deterministic",
    representation="int_array",
    salt="schematio:ore:v1",
    identity_basis="entity_location",
    assign_missing=True,
    collision="reject",
    dangling="remove",
)

plan = TransformPlan.from_passes("ore-registry-v1", [{
    "type": "content_policy",
    "policy": ContentPolicy(uuids=uuid_policy).as_dict(),
}])
report = apply_transform(schematic, plan)
```

An `Owner` in trusted Nucleation extraction provenance is not Minecraft entity
NBT and is not traversed by these rules. Registries must still distinguish
verified provenance from arbitrary imported provenance before attribution.

## Text, NBT, items, and entities

Text rules can remove selected fields, redact configured words, warn or redact
on suspicious patterns, and enforce string-size limits. Pattern detection is a
policy signal rather than proof of malicious intent; warning or quarantine is
recommended for prompt-injection and code-like heuristics.

NBT policies limit depth, node counts, collection sizes, and independently
handle ordinary keys, executable/command fields, player/profile data, and
volatile state such as scheduled ticks. Item rules can clear inventories,
allow/deny item IDs, and cap inventory list sizes.
Block, entity, and block-entity policies support allowlists, denylists,
namespace rules, and `allow`, `warn`, `remove`, `reject`, or `quarantine`
decisions. Removing a block replaces it with air and also removes a block
entity orphaned at that coordinate. Entity policies additionally provide count
budgets globally, per region, and per 1,000 blocks, plus player exclusion.
Mobile entities and block entities remain
separate because item frames, minecarts, display entities, signs, barrels, and
command blocks have different preservation requirements.

The in-memory NBT budgets in a transform protect registry policy and output
shape. Bounded readers separately enforce input and decompressed bytes,
dimensions and checked volume, region/palette/entity counts, NBT depth, string
lengths, collection lengths, and total nodes before large allocations. Sponge,
Litematic, classic schematic, mcstructure, structure SNBT, and snapshot formats
implement this contract. Whole MCA and world-ZIP containers are deliberately
refused by the generic bounded reader; use the world-segment streaming API with
explicit world bounds so a complete world is never materialized implicitly.

## Material standards

Material profiles support exact mappings and family templates:

```python
from nucleation import MaterialProfile, TransformPlan

profile = MaterialProfile(
    name="concrete-standard-v1",
    safety="profile",
    mappings={"minecraft:stone": "minecraft:white_concrete"},
    family_mappings=[{
        "source": "minecraft:{color}_wool",
        "target": "minecraft:{color}_concrete",
    }],
)

plan = TransformPlan.from_passes("concrete-standard-v1", [{
    "type": "remap_materials",
    "profile": profile.as_dict(),
}])
```

Safety modes are `exact`, `behavior_preserving`, `profile`, and `aggressive`.
`exact` accepts only identical states. `behavior_preserving` classifies both
states using block data and conservative roles: kind, full-cube/collision,
transparency, emitted light, block-entity behavior, support needs, redstone
component status, piston class, and properties. It applies a mapping only when
those roles match. `profile` and `aggressive` apply explicitly requested convention maps
and report `material.behavior_not_proven`. Concrete, glass, slabs, movable
blocks, and support blocks are not interchangeable merely because they look
similar.

This role proof is intentionally conservative: unknown or modded blocks require
an explicit profile rather than being guessed compatible.

## Built-in presets and registry integration

- `canonical`: lossless deterministic palette normalization.
- `registry-safe-v1`: canonicalization, authored-text removal, suspicious-text
  warnings, removal of common ephemeral entities, an entity-count quarantine
  threshold, and deterministic UUID standardization.

`RegistryPipeline` is the first-class storage workflow. It streams a key from
any `Store` under `max_input_bytes`, bounded-decodes it, applies the plan
atomically, writes accepted/quarantined output as a deterministic Nucleation
snapshot, and persists a JSON audit report. Decode and policy failures route to
reject without writing a schematic.

`RegistryHookRule` is the scripting boundary. A rule examines only stable
summary counters and may escalate `accept` to `quarantine` or `reject`; it can
never override a rejection. Python and compiled extensions can emit this JSON
or consume the report out of process. Nucleation deliberately does not import
arbitrary callback code into the registry process.

```rust
use nucleation::{MemStore, RegistryPipeline, Store};

let input = MemStore::new();
let output = MemStore::new();
input.put("incoming/build.schem", &payload)?;
let result = RegistryPipeline::default()
    .ingest_store(&input, "incoming/build.schem", &output)?;
println!("{:?}: {}", result.route, result.report_key);
```

All policy, limit, hook, and report contracts remain serializable so Rust,
Python, JavaScript/WASM, Kotlin/JVM, PHP, and C/C++ share the same semantics.

## Required conformance tests

Every built-in policy must cover:

- dry-run and apply equivalence;
- atomic rejection;
- deterministic and idempotent application where the mode permits it;
- nested items, passengers, profiles, owners, and block entities;
- UUID definitions, references, collisions, dangling references, and all
  supported representations;
- transform-twice and format round trips;
- stable reports that do not echo private values;
- malformed, deeply nested, oversized, unknown, and modded fixtures;
- property-style mutation tests and parser fuzzing for untrusted data.

The checked-in `fuzz/` package contains `bounded_decode` and `policy_json`
`cargo-fuzz` targets. `tests/policy_conformance.rs` supplies deterministic CI
coverage for the same panic-safety and non-leak invariants.
