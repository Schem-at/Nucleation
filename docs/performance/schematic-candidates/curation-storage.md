# Storage and crafting archive curation

The local catalog covers **374 schematic artifacts**: 333 from Storage Catalog and 41 from Autocrafting Archive. These include variants and repeated designs; there are 368 distinct SHA-256 values. All remain **unvalidated** in this curation. The 17 cases below have proposed finite tests, not successful engine runs or fresh vanilla captures.

The machine-readable catalog is `/Users/harrison/RustroverProjects/Nucleation-schematic-archive/curation/storage.json`. Each entry uses the exact `source` and `path` from its download manifest and retains the artifact SHA-256, Git blob ID, source commit, author metadata, declared version/dimensions, mechanics tags, and blockers. All 374 local files were rechecked against their manifest SHA-256 and Git blob IDs. That proves byte integrity, not circuit behavior.

## Sources and evidence

- [Storage-Catalog/Archive](https://github.com/Storage-Catalog/Archive/tree/183856344fe4fad1e47cbec1c298da1d262345b1), pinned to `183856344fe4fad1e47cbec1c298da1d262345b1`.
- [XPBot1/Autocrafting-Archive](https://github.com/XPBot1/Autocrafting-Archive/tree/a060302478997a59b97b68555ee16901c627281c), pinned to `a060302478997a59b97b68555ee16901c627281c`.

Tags come from source categories, names, and entry metadata. They are candidate mechanics, not a verified block palette. Complexity uses the source-declared bounding volume: compact at most 1,024 cells, medium at most 16,384, otherwise large. It does not estimate active blocks or runtime. One comment attachment has no corresponding top-level attachment metadata and therefore has no declared version or dimensions.

Upstream “Functional” and “Tested & Functional” labels are retained as attribution only. Five artifacts have an upstream “Broken” label and are quarantined from the positive shortlist. Seven declare a 26.x version; keep those separate from the Minecraft 1.21.11 reference. Even a declared “1.21.11” label does not establish the binary save DataVersion: every entry's `inspected_data_version` remains null in this metadata-only catalog.

Preserve per-entry authorship and license evidence. No blanket artifact redistribution license is asserted. The ACD002 and ACS005 entry notes explicitly link CC BY-SA 4.0; other selected entries need a separate license determination before redistribution. Third-party schematic bytes remain in the local archive, not this document.

## Proposed first cases

Paths and exact attachments are in the catalog. The codes below refer to Storage Catalog. Input/output coordinates still need mapping from each artifact. Proposed tick limits in the JSON are failure timeouts, not observed completion times or throughput promises.

| Code | Selected artifact / useful mechanism | Finite input and independent check |
|---|---|---|
| CC002 | Divide-by-N dropper counter, 8×6×1 | Configure N = 1, 2, 3, 8; deliver 2N+1 separated rail pulses. Toggle after N and 2N pulses; conserve counter items. |
| DT002 | Hopper-speed dropper elevator chart, 36×17×1 | Extract one labelled non-D1/D2 tower as a recorded derived fixture. Feed 65 stone; require all 65 at the top after drain, with a complete intermediate inventory ledger. |
| MF005 | Slot-reservation MIS, 11×8×8 | Preserve one-item reservations. Feed 16 stone, 16 dirt, and 8 unreserved items; check matching destination increments and account for overflow. |
| OIP001 | Three-way stack-type sorter, 5×4×2 | Retain the documented 38-item dropper prefill. Feed 8 stone, 8 ender pearls, and 8 shears; check the 64-, 16-, and 1-stack outputs and conservation. |
| OIP004 | Simple unstackable filter, 3×2×3 | Feed 4 stone and 4 shears. Check routing while retaining the source-documented one-item unstackable buffer in the accounting. |
| BU003 | Small box unloader, hopper-collection variant, 4×4×3 | Feed an empty box, a one-stone box, and a 65-stone box. Recover 66 stone and all three empty boxes. |
| OL005 | Basic 1× box loader, 5×3×3 | Supply two empty boxes and 1,729 stone. After drain, require one 1,728-stone output box and one stone in the next placed box. |
| FL003 | Valid 1.13-labelled filtered-loader attachment, 7×7×5 | Preserve/configure the SS3 filter. Supply two boxes, 1,728 stone, and 16 dirt; stone loads, dirt follows the mapped bypass, and all prefill/items remain accounted for. |
| FLS007 | Three-way box fill-level sorter, 11×4×3 | Feed boxes containing 0, 1, 1,727, and 1,728 stone. Check empty/partial/full routes and exact nested contents. |
| BS004 | Basic first-item box sorter, 12×9×4 | Configure two filters and feed six nonempty boxes: two matching each filter and two unmatched. Vary later slots; routing follows the first item while complete contents remain unchanged. |
| AL001 | Accessible moving loader, 1×6×7 | Supply two boxes and 1,729 stone to one slice; account for the full/partial boxes across movement. A later two-slice test must preserve the instructed alternating honey/slime. |

Source setup constraints matter. DT002 documents direction restrictions for D1/D2, so a different tower cannot validate those variants. MF005 requires leaving reservation items in their slots. BS004 requires explicitly programmed filter hoppers and a known overflow route. FL003 describes a rare multi-slice loss mechanism; a short isolated run cannot refute that warning. AL001's source reliability claim is not established by one successful finite run.

## Crafting cases to retain for capability work

These codes refer to Autocrafting Archive. They are useful future tests, but active crafting currently blocks engine validation.

| Code | Selected artifact / useful mechanism | Finite recipe check |
|---|---|---|
| ACC001 | Simple precision compactor, 2×5×5 | Configure nine-slot iron-block crafting. Test 27 ingots → 3 blocks, then 26 ingots → 2 blocks + 8 retained ingots, accounting for authored prefill. |
| ACC010 | Compactor suppressing double firing, 1×6×6 | Preserve the silent-dropper item and supply 18 ingots. Require two iron blocks, with no extra craft from the second-pulse mechanism. |
| ACD002 | On-demand decompactor, 1×4×4 | Supply three gold blocks, hold output full for 256 ticks, then free space. Require 27 ingots after drain and account for every intermediate item during backpressure. |
| ACS002 | Flint-and-steel / two-slot crafter, 3×4×3 | Use the documented left flint chest, right iron chest, and middle output hopper. Feed eight of each at a regular cadence and exercise the disable lever; expect eight tools after prefill accounting. |
| ACS004 | Minimal-prefill sea-lantern crafter loader, 11×7×9 | Retain prefill; feed 40 shards and 50 crystals for ten incremental sea lanterns. Include a withheld-ingredient control and count all four crafters and nested box outputs. |
| ACS005 | Iron-block-to-shears station, 3×5×2 | Set equal lower-crafter recipe ingredients before placing the observer next to single-dot dust, as instructed. Two blocks represent 18 ingots and nine incremental shears after complete prefill/leftover accounting. |

Several crafting attachments declare MC 1.20.4 while their entry compatibility says 1.21+. Inspect actual DataVersion and experimental feature flags before datafix; neither label permits silently treating the save as native 1.21.11. ACS002 explicitly excludes random/spam inputs. Its ordinary finite-input test must respect that condition.

## Current engine and oracle boundaries

- [Vanilla rule registration](../../../crates/mc-tick/src/vanilla.rs) currently registers a crafter only as an inert passive container when `triggered=false` and `crafting=false`. Disabled-slot routing and comparator output support do not implement recipe execution. Active crafter tests must remain blocked or fail explicitly.
- [Inventory helpers](../../../crates/mc-tick/src/inventory.rs) still use 64 in `fullness_sum`. `VanillaRules::max_stack` supplies a partial item-specific table to some query paths, so the limitation is not “all engine paths always assume 64.” Stack-limit-sensitive designs need item-by-item verification and a complete component-aware policy before broad claims.
- Shulker items carry nested inventory contents. A flat `(slot, item ID, count)` comparison cannot establish box preservation, routing correctness, or conservation. Count contents recursively, track box identity/content vectors, and retain component data through save loading and datafix.
- The current community oracle compares tick-end blocks, comparator outputs, and occupied flat inventory. It does not independently census runtime entities or compare item components, moving block-entity payloads, or intra-tick update delivery. Loaders, unloaders, water streams, and cart systems need the corresponding oracle extensions; engine-only item counts are not fresh vanilla parity evidence.
- Large storage halls, test item sets, chunk loaders, and player-dependent systems remain cataloged at lower priority. A chunk loader requires real server chunk/portal lifecycle; bounded headless simulation alone cannot prove that function.

For each promoted case, inspect the binary, map ports, record all initial inventory and prefill, preserve the source save version through datafix, and run a no-input control plus finite positive sequence. Capture a new Minecraft 1.21.11 reference where the required mechanics are represented. Compare every supported tick-end field and a complete item ledger, then benchmark only after those checks pass. Keep upstream bytes immutable and give extracted or configured fixtures separate identities.
