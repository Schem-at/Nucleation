# The vanilla oracle

Runs structure-based tests inside **real Minecraft**, headless, so the Rust tick
engine (`crates/mc-tick`) can be validated against the game rather than against a
reading of its source.

```sh
tools/gametest/run.sh                    # run every test
tools/gametest/run.sh --tests nucleation # filter
```

Output is a JUnit XML report at `tools/gametest/work/report.xml`.

## Why this is far simpler than expected

The plan assumed a Fabric mod with Mixins, Loom, and Yarn mappings. None of that
is needed, because of one discovery:

**Minecraft 26.2 ships its server jar unobfuscated.** Real names —
`ServerLevel.tick`, `runBlockEvents`, `ObjectLinkedOpenHashSet blockEvents`. That
is why Yarn has *zero* builds for 26.2 and why the version manifest no longer
carries `server_mappings` (1.21.4 and 1.20.4 still do).

So the whole oracle is `javac` plus the server jar's own bundled classpath. No
Gradle, no Loom, no Yarn, no Mixins, no mod loader.

**No EULA is required**, because we never boot a public server: Mojang's own
`GameTestServer` runs in-process for testing.

## How it works

1. `run.sh` downloads the 26.2 server jar (cached in `work/`), extracts the inner
   `server-26.2.jar` and its 39 bundled libraries, and builds a classpath.
2. Structures authored as **`.snbt`** in `pack/data/nucleation/structure/` are
   converted to the binary `.nbt` that datapacks require, using **the game's own
   `TagParser`/`NbtIo`** — so the conversion cannot disagree with the game.
3. `GameTestMainUtil.runGameTestServer` boots `GameTestServer`, loads the datapack,
   places each structure, and runs it.

`SharedConstants.tryDetectVersion()` must be called before
`runGameTestServer` — it calls `Bootstrap.bootStrap()` but not that, and without it
`DataFixers` dies at class-init with "Game version not set".

## Writing a test

A test is a **structure plus a JSON file**, and needs no Java at all.

`pack/data/nucleation/test_instance/<name>.json`:

```json
{
  "type": "minecraft:block_based",
  "environment": "nucleation:default",
  "structure": "nucleation:<name>",
  "max_ticks": 40,
  "setup_ticks": 0,
  "required": true
}
```

`pack/data/nucleation/structure/<name>.snbt` contains the contraption plus test
blocks. `minecraft:test_block` has four modes (`TestBlockMode`):

| mode | meaning |
|---|---|
| `start` | powered when the test begins — wire this into your circuit's input |
| `accept` | powering it passes the test |
| `fail` | powering it fails the test |
| `log` | powering it logs |

So a redstone test *is* a redstone circuit: `start` → your contraption → `accept`.
That is also why this doubles as a bug-report format — a user can build a failing
contraption in-game, export the structure, and hand over something that runs.

## Facts worth keeping

- `pack_format` for 26.2 is **107** (`version.json` → `pack_version.data_major`).
  A wrong value makes the pack load silently as nothing.
- `DataVersion` for 26.2 is **4903** (`version.json` → `world_version`).
- `--packs <dir>` expects a directory *containing* pack directories, not a pack
  itself. Point it at a pack and it copies nothing, the tests never register, and
  the run still reports success from vanilla's own `always_pass` — which is why
  `run.sh` asserts that the expected tests actually appeared in the report.
- Datapack structures must be binary `.nbt`; vanilla ships zero `.snbt` in its jar.
  SNBT loading exists but only via `DirectoryTemplateSource(loadAsText = true)`,
  which reads from world directories rather than datapacks.

## Guarding against a green run that tested nothing

The most dangerous failure here is a harness that reports success while running no
tests. It happened during development: with a malformed `--packs` layout the run
printed "All 1 required tests passed" — entirely from vanilla's `always_pass`.

Two defences:

1. `run.sh` fails if a test named in the pack is missing from the report.
2. Every test in this pack is verified against a **negative control** — the
   contraption is deliberately broken and the test confirmed to *fail*. A test that
   cannot fail is worthless. `torch_inverts` was validated this way: with the wire,
   pass; with the wire removed, "Didn't succeed or fail within 40 ticks".
