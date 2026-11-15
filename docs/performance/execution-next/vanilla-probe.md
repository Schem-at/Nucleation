# Fresh Minecraft 1.21.11 probe

The isolated driver is [`TraceCapture12111.java`](../../../tools/gametest/src/TraceCapture12111.java).
It uses the original **Minecraft 1.21.11 GameTestServer**, resolving exact obfuscated
signatures from Mojang's mappings. It does not use the 26.2 runtime, a client,
Fabric, MC-Inspector, a decompiler or a remapped/patched game jar. This is a new
capture route; the original hipster Fabric OracleMod has not been recovered.

All four captures (two positive fixtures and their negative controls) **ran on
harrison-work using Temurin 21.0.13+9-LTS** on 15 November 2026. Each contains
40 captured ticks and asserts zero players throughout. Positive activity and
negative-control checks passed. The [saved captures](../../../crates/mc-tick/tests/corpus/vanilla12111_execution_next/)
and [source/capture hash manifest](vanilla-provenance.json) preserve this evidence.

The comparator fixture transferred items at ticks 1, 9 and 17. Its actual block
entity output rose from 0 to 1 at tick 3 and returned to 0 at tick 19. The locked
control kept all three items in the barrel and output zero for all 40 ticks.

The QC piston extended at tick 1; its moving blocks settled at tick 3. It remained
extended after the diagonal power block was removed at tick 10. The explicit
notification at tick 12 began retraction, completed at tick 14. The unpowered
control never extended. These are newly observed 1.21.11 tick-end results.

Driver source SHA-256: `25a18d114a5581607a7f276e1bb67c8df8c759f7a8e5ac2b6f42aa84561754f5`.

## Recovered runtime and verification

Read-only recovery location on the Mac:

```
/Users/harrison/RustroverProjects/Nucleation/tools/gametest/work-1213/1.21.11/
```

It contains `server.jar`, `server.txt`, and extracted `inner/` libraries/server.
The filename is from an earlier version-bisect investigation; its embedded
`version.json` identifies Minecraft 1.21.11, data version **4671**, Java **21**.

On 15 November 2026, the following local hashes matched the current official
[version manifest](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json)
and [1.21.11 metadata](https://piston-meta.mojang.com/v1/packages/812ee7fdfdf648bb872f7a1dca33718de4e786b4/1.21.11.json):

| Artifact | Hash |
| --- | --- |
| [Server bundle](https://piston-data.mojang.com/v1/objects/64bb6d763bed0a9f1d632ec347938594144943ed/server.jar), 56,327,581 bytes | SHA-1 `64bb6d763bed0a9f1d632ec347938594144943ed` |
| [Server mappings](https://piston-data.mojang.com/v1/objects/5621e9253f05fd57872bbe7f8ddf5f9a7d525955/server.txt), 8,678,606 bytes | SHA-1 `5621e9253f05fd57872bbe7f8ddf5f9a7d525955` |
| Extracted server jar | SHA-256 `ec47239a8de246335e1d54f6ac319bd35641778eb4b6a6da06372840d02fcebc` |

The bundle ZIP CRC and all 40 extracted server/library hashes also matched its
`META-INF/versions.list` and `META-INF/libraries.list`. The driver checks both the
mapping hash and the hash of the server jar from which its classes actually load.

For an isolated Java 21 installation, the official
[Temurin 21.0.13+9 release](https://github.com/adoptium/temurin21-binaries/releases/tag/jdk-21.0.13%2B9)
provides the [Linux x64 JDK archive](https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.13%2B9/OpenJDK21U-jdk_x64_linux_hotspot_21.0.13_9.tar.gz).
The publisher's [checksum](https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.13%2B9/OpenJDK21U-jdk_x64_linux_hotspot_21.0.13_9.tar.gz.sha256.txt)
is SHA-256 `d7f5d5cf865a2902b70ff549b97586a7d3f234e016f041ab447ea740c7d66d1c`
(207,858,352 bytes). Use its absolute `bin/java` and `bin/javac`; no global Java
configuration change is required.

## Compile and capture on harrison-work

The reusable [Python runner](../../../tools/gametest/run_capture12111.py) verifies
the server bundle and mapping hashes, ZIP CRC, all extracted library/server
checksums, the pinned Java version and every generated capture. It creates four
fresh worlds, preserves logs on failure, enforces per-process timeouts and writes
command/source/runtime/capture metadata. It downloads nothing and changes no
global configuration:

```sh
python3 tools/gametest/run_capture12111.py \
  --runtime /home/harrison/nucleation-perf/execution-next/vanilla/runtime \
  --jdk /absolute/path/to/jdk-21.0.13+9 \
  --jdk-archive /absolute/path/to/OpenJDK21U-jdk_x64_linux_hotspot_21.0.13_9.tar.gz \
  --output /absolute/new-or-empty/capture-run
```

`--jdk-archive` is optional; when supplied, its publisher checksum is verified too.
The Java driver also rechecks the mapping and loaded server hashes. The first
successful captures used the same driver through the parent's equivalent
temporary runner. The reusable runner then completed a second four-capture run
on harrison-work, verifying the JDK archive, bundle CRC and all 40 extracted hashes.
Every initial/tick snapshot, fixture and action matched the first run exactly;
chunk-readiness warmup counts may differ. [Reproduction metadata and logs](validation/vanilla/reproduction/)
record the executed commands and hashes.

Example paths after transferring the verified runtime; use a fresh universe for
every invocation because the driver refuses to overwrite an existing one:

```sh
ORACLE_RUNTIME=/home/harrison/nucleation-perf/execution-next/vanilla/runtime
ORACLE_OUT=/home/harrison/nucleation-perf/execution-next/vanilla
ORACLE_JDK=/absolute/path/to/jdk-21.0.13+9
mkdir -p "$ORACLE_OUT/classes"
"$ORACLE_JDK/bin/javac" -d "$ORACLE_OUT/classes" tools/gametest/src/TraceCapture12111.java
ORACLE_CP="$ORACLE_OUT/classes:$ORACLE_RUNTIME/inner/META-INF/versions/1.21.11/server-1.21.11.jar"
while IFS= read -r jar; do ORACLE_CP="$ORACLE_CP:$jar"; done < <(find "$ORACLE_RUNTIME/inner/META-INF/libraries" -name '*.jar' | sort)
"$ORACLE_JDK/bin/java" -Xmx2G -cp "$ORACLE_CP" org.nucleation.oracle.TraceCapture12111 \
  --mappings "$ORACLE_RUNTIME/server.txt" \
  --universe "$ORACLE_OUT/universe-comparator" \
  --out "$ORACLE_OUT/comparator_drain.json" --case comparator_drain --ticks 40
```

Repeat with `--case piston_qc` and distinct universe/output names. Repeat both
with `--negative-control`, again using distinct paths. Never time Rust benchmarks
concurrently with these server runs.

The probe uses its own named package. Minecraft's obfuscated classes occupy the
default package and are signed; placing the probe there would cause a signer
mismatch. Keep the original server signatures intact.

## What the fixtures and snapshots mean

These fixtures are authored from native 1.21.11 block descriptors and block-entity
NBT. They are **not** 26.2 schematics with a rewritten save version. Initial states
are installed directly into chunk sections, avoiding placement callbacks, and
block entities are loaded separately. This deliberately matches the engine's
`InWorld` loading mode. Initial states and the barrel's three occupied items are
asserted before any captured tick. The world RNG is seeded to zero at that point.

The driver tickets every observed chunk, waits for entity-ticking readiness,
then captures exactly 40 further ticks. It asserts zero players at every snapshot.
Each capture includes fixture size, native data version, local block descriptors,
block-entity NBT, explicit action records, Java version, server/mapping hashes,
initial state and every tick-end block, comparator-output and inventory snapshot. Coordinates are local
to world origin `(0,100,0)`. Scan order is y, z, x, matching dense engine storage.

- **Comparator drain:** a hopper pulls three redstone items from a barrel. The
  comparator must turn on and later return to zero. Final counts must be barrel
  zero, hopper three. Its negative control locks the hopper and requires every
  tick to retain barrel three, hopper zero.
- **Piston QC:** a redstone block diagonally above/behind an east-facing piston
  powers its quasi-connectivity input. Before tick 1, notify neighbours of the
  air above the piston. Before tick 10, remove the power block through ordinary
  `setBlock(...,3)` behaviour. The piston must stay extended through ticks 10–11.
  Before tick 12, notify above it again; the piston must retract. The negative
  control omits the power block and must never extend.

For Rust comparison, build the emitted fixture with `SettleMode::InWorld`, set
RNG seed zero, and apply each action **before stepping its numbered tick**. The
initial snapshot is tick zero; the first `step` produces snapshot one. Compare
all captured non-air states, comparator outputs and occupied slots, including ticks with no changes.
Use the emitted native version 4671 for these new fixtures; do not modify the
hipster fixture's historical save version 2730.

This probe captures tick-end behaviour, comparator outputs and inventories only. It does not claim a
complete within-tick vanilla notification trace or restore the old Fabric probe.
