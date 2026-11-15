# Dynamic schematic candidates

These four downloads were fetched, SHA-256 checked, decompressed and inspected as
NBT. **Status: static-only.** None has been run by Nucleation or the Minecraft
1.21.11 reference server in this investigation. A schematic's author's success
claim is not independent parity evidence.

The selection excludes the existing hipster, BB, flyer, 6x6 door, sorter,
transfer, cart and trapdoor workloads listed in
[`hipster-next/fixtures.md`](../hipster-next/fixtures.md). Pinned sources,
coordinates and proposed controls are in [`dynamic-sources.json`](dynamic-sources.json).

| Candidate | Dimensions; non-air blocks | Native DataVersion | Initial block entities / entities | Initial piston states |
| --- | --- | --- | --- | --- |
| SleeperPin interlocking 3x3 door | 9×13×5; 174 | 4671 | 16 / 0 | 19 sticky + 4 ordinary, all retracted |
| vktec accessible shulker loader | 1×6×7; 29 | 2586 | 12 / 0 | 3 sticky + 3 ordinary, all retracted |
| Kizu CUD mixed loader | 4×6×4; 81 | 3465 | 15 / 0 | 1 sticky + 1 ordinary, both retracted |
| viktig_sau plane | 27×6×33; 252 | 4556 | 0 / 0 | 1 sticky + 24 ordinary, all retracted |

No selected file contains a moving-piston block entity. All four have negative
saved region extents; the importer must honor those, rather than treating the
saved corner as the region's minimum. Coordinates below refer to the normalized
enclosing box, and must be checked after production import.

## First checks to implement

**Interlocking door — first priority.** The [author's original post](https://www.reddit.com/r/redstone/comments/1qs127k/3x3_door_with_interlocking_block_openclose/)
provides a [direct schematic](https://sleeperpin-minecraft.s3.us-east-2.amazonaws.com/3x3_interlocking_block_door.litematic)
and explicitly says the design is not spam-proof. The saved lever is `(6,1,4)`.
Use ordinary lever behavior, establish the complete cycle duration in vanilla,
then capture two safely spaced cycles. Require complete opening, complete
closing, and return of every mechanism block, not merely activity. The three
droppers contain seven redstone items across three occupied slots; capture their
transfers and all comparator outputs. There are also three crafters and copper
bulbs, making this more varied than a plain piston door. Its native DataVersion
already matches the reference's 4671. The author's location/orientation claim
can become a separate set of translated/rotated reference tests.

**Accessible loader — compact inventory rollover.** The [catalog entry](https://storagecatalog.org/archives/AL001-Bottom-Accessible-Shulker-Loader/)
credits vktec, declares Minecraft 1.16.5, and describes single hopper-speed
operation with reliable collection. The file's twelve block entities contain no
items. Seed empty replacement boxes in dispenser `(0,2,1)`, a nearly full box at
`(0,1,1)`, and the last ordinary stackable item in input hopper `(0,1,0)`.
First verify that route and exact setup on vanilla. A 1727+1 item case exercises
fullness detection, piston break, collection and replacement without waiting
through a full fresh fill. Require all 1728 items to survive inside the output
shulker item, an empty replacement box, and no duplication. Include an empty-input
control and a second rollover once the first is understood.

**CUD loader — partial boxes and inventory-only transitions.** The [catalog entry](https://storagecatalog.org/archives/OL004-1x-CUD-based-loader/)
credits Kizu and declares Minecraft 1.20.1. Saved author metadata says `auraa42`;
both are preserved. Feed a finite mixed batch through hopper `(0,3,2)` into the
box at `(1,3,2)`, then stop. The archive describes a fader timeout followed by
box replacement even when the box is only partly full. Measure that timeout on
vanilla, then compare bursts with shorter and longer gaps, plus a zero-input
control. The save already holds 76 empty shulker-box items across 76 occupied
slots; preserve this stock and the six comparator outputs. Water and packed ice
add an item-transport path beyond the simple loader. Item conservation must cover
the emitted boxes' nested contents, rather than counting only outer box items.

**Plane — branching slime/honey movement.** The [publisher's page](https://redstone-schematics.org/schematic/8)
attributes it to `viktig_sau` and warns to stop the tail to avoid breaking the
machine. The listing says version “any”; the inspected save records DataVersion
4556. There are initially no inventories or entities. A controlled front-face
update at tail observer `(25,2,16)`, which faces east, is a topology-derived
launch candidate, **not a verified or author-specified launch sequence**. Resolve
launch on vanilla before adding a benchmark. Reserve a fully ticking corridor;
compare the whole assembly after repeated strokes, requiring exact translation
at equivalent settled cycle boundaries, no stranded blocks, a tail-only stop and
restart. Capture in-flight states too. Its download token may expire; refresh the
link from the source page and retain the hash pin.

## Reference-harness requirements

Use the verified 1.21.11 runtime and Java runner described in
[`execution-next/vanilla-probe.md`](../execution-next/vanilla-probe.md). The current
driver constructs two native fixtures; importing arbitrary community structures
still needs an adapter. Preserve each input's native DataVersion during
block-entity/item datafixing. Then assert the initial normalized block states,
block entities and occupied slots before captured ticks, maintain zero players,
wait for ticking readiness, seed RNG, and record every explicit action before its
numbered tick. An idle control detects unintended placement activation.

**The present reference capture is insufficient to prove loader behavior.** It
records block states, comparator outputs and flat occupied inventory slots, but
does not fully capture nested item components or item entities. Extend it to
record shulker contents through break/dispense/collection and item-entity state,
with stable matching, before claiming independent parity for either loader.
Retain ordered engine differential traces separately; tick-end vanilla agreement
does not establish complete within-tick notification equivalence.

One other download, the [108-block Panternoster elevator](https://redstone-schematics.org/schematic/12),
has a clear short-pulse startup but was saved with a source moving-piston block
entity at progress `0.5`. It is excluded from these four clean candidates until
pending movement restoration can be checked. Silently dropping that NBT would
create an invalid test setup.

## Reproduce the static inspection

```sh
python3 tools/prepare_tick_dynamic_candidates.py
```

This uses Python's standard library; it does not compile or run Rust or Minecraft.
It downloads only the four pinned small schematics, checks exact byte counts and
SHA-256 hashes, decodes the palette to cross-check saved block counts, and writes
full initial-state inspection to the ignored directory
`tests/benchmark-fixtures/candidates/dynamic/`. Existing matching downloads are
reused. The source JSON and helper are tracked; third-party binaries are not.

No explicit schematic redistribution license was found on the inspected source
pages or archive root. Preserve creator attribution and original links. The
[Storage Catalog repository](https://github.com/Storage-Catalog/Archive) is an
archive mirror; its presence does not by itself establish a license. The plane's
site expressly retains creators' copyright. This catalog does not redistribute
their binary files.

## Broader archive discovery

The following source trees were inspected on 15 November 2026 for a separate
persistent local archive. Counts are file occurrences at the pinned revision,
not unique designs; duplicate formats and duplicate builds remain possible.
Sizes are the sum of matching Git tree blob sizes, before local inspection or
deduplication. This research did not download these collections or execute them.

The [Archive Downloader server registry](https://github.com/Llama-Collective/llama-archives/blob/main/mod_data.json),
referenced by the mod's own `ServerDictionary.java`, identifies Soontech,
Autocrafting and Wither alongside Storage Catalog and Logical Redstone as its
supported community archives. For those three, preserve each entry's
`README.md` and `data.json` alongside `Archive/**/attachments/` schematics;
exclude videos, images, ZIP files and complete worlds.

| Source and pinned revision | Matching schematic files | Compressed file bytes | Scope and qualification |
|---|---:|---:|---|
| [Soontech-Annals/SoonNowArchive](https://github.com/Soontech-Annals/SoonNowArchive/tree/d58447452e7a2df336a989c4e1c389f2ecea8f7f) | 55 | 935,922 | Encoded storage systems and modules; inventory contents and timing need preservation. |
| [XPBot1/Autocrafting-Archive](https://github.com/XPBot1/Autocrafting-Archive/tree/a060302478997a59b97b68555ee16901c627281c) | 41 | 213,071 | Crafter and compactor mechanisms; requires crafting and nested inventory checks. |
| [DuskScorpio/wither-archive](https://github.com/DuskScorpio/wither-archive/tree/074bbabfc2807ab8999f787c9954d4404ae1dba0) | 63 | 350,375 | Wither, block and tree farms; many need mob/plant mechanics beyond a redstone-only fixture. |
| [SMA slimestone archive](https://github.com/Gudu-Z/SlimestoneMilitaryArmamentsArchives/tree/fbdbb158f76e90c7cdda1092048544ec4340eb0b) | 46 | 416,850 | The SMA community's Java aircraft/mobile-machine archive. Download `public/投影文件/**/*.litematic`; keep `projections/**/*.md` and `articles/**/*.md`. Skip RAR archives. Several metadata records are intentionally incomplete. |
| [X-com/slimestone-AI](https://github.com/X-com/slimestone-AI/tree/b0096835b0a50d2f4ea41983c955fb0fa46ec889) | 63 | 40,905 | Tiny `.schematic` inputs under `flying machines/schematics*`; machine-learning research inputs include explicit `doesnt_loop` negative cases. Do not label the collection functional. |
| [CasualMinecraft/casual-builds](https://github.com/CasualMinecraft/casual-builds/tree/0901c0036f4ee2481d38cbac6b4d178a640a17bd) | 8 | 18,572 | Four author-channel builds, each in two formats. The autoloading super-smelter is the most relevant item/cart/furnace case. |

The Casual super-smelter's [author instructions](https://github.com/CasualMinecraft/casual-builds/blob/0901c0036f4ee2481d38cbac6b4d178a640a17bd/redstone/autoloading-super-smelter--casual/README.md)
state Java 1.21, no mods, and distinguish the intact Litematica export from the
WorldEdit export requiring rail corrections and added hopper minecarts. Keep
that distinction in import tests. Its repository allows personal use and
modification and requires credit for redistribution. SMA and X-com have MIT
repository licenses; retain attribution and do not infer additional rights to
third-party designs from the surrounding website or simulator license.

[vjeux/redstonepedia at `86a5e4843d8effe81bf7bf411a554cef8b58a1c9`](https://github.com/vjeux/redstonepedia/tree/86a5e4843d8effe81bf7bf411a554cef8b58a1c9)
has **no** `.litematic`, `.schem`, `.schematic`, `.nbt` or `.snbt` blobs in that
tree. It has useful compact custom text inputs instead:

| Selected path | Files | Bytes |
|---|---:|---:|
| `designs/piston-door-3x3/archive/**/*.txt` | 1,694 | 3,072,047 |
| `designs/circuits/**/archive/**/*.txt` | 534 | 1,025,933 |
| `crates/redsim/tests/fixtures/*.fixture` | 18 | 81,512 |

Its layouts encode origin, block-state palette, layers and named ports; fixture
scripts include actions and expected frames. The repository is MIT, but its
README targets **Minecraft 26.3**, and the designs include agent-search outputs.
Their stated verification cannot establish 1.21.11 parity. Archive only these
small source artifacts and documentation, then convert and run a fresh 1.21.11
oracle before accepting any as a functional benchmark. Avoid cloning its large
history or downloading recordings, screenshots and videos.
