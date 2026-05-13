# nucleation-jvm

JNI bindings for [Nucleation](https://github.com/Schem-at/Nucleation), a
high-performance Minecraft schematic parser and utility library written in
Rust. Provides a Java / Kotlin / JVM-language consumer surface with **near-
native performance** (no reflection cost) and an **idiomatic, AutoCloseable**
API.

- **Group / artifact**: `com.github.schemat:nucleation`
- **Minimum JDK**: 21
- **Platforms bundled in the fat JAR**: `linux-x64`, `linux-arm64`,
  `macos-x64`, `macos-arm64`, `windows-x64`
- **License**: AGPL-3.0-only (inherits from the upstream library)

## Quick start

```java
import com.github.schemat.nucleation.*;
import java.nio.file.Files;
import java.nio.file.Path;

public class Demo {
    public static void main(String[] args) throws Exception {
        byte[] bytes = Files.readAllBytes(Path.of(args[0]));
        try (Schematic s = Schematic.fromLitematic(bytes)) {
            System.out.println("Name: " + s.name());
            System.out.println("Dimensions: " + s.dimensions());
            System.out.println("Block count: " + s.blockCount());

            // Iterate every non-air block (one JNI call, in-memory iteration after).
            for (Block b : s) {
                System.out.println(b);
            }

            // Mutate, then export to another format.
            s.fillCuboid(0, 0, 0, 5, 5, 5, "minecraft:stone");
            byte[] schem = s.toSchematic();
            Files.write(Path.of("out.schem"), schem);
        }
    }
}
```

### Building with shapes

```java
try (Schematic schem = new Schematic("dome");
     Shape sphere = Shape.sphere(0, 64, 0, 16.0);
     Shape floor  = Shape.cuboid(-16, 63, -16, 16, 63, 16);
     Shape shell  = Shape.difference(sphere, floor);
     Brush brush  = Brush.solid("minecraft:glass")) {

    BuildingTool.fill(schem, shell, brush);
    Files.write(Path.of("dome.litematic"), schem.toLitematic());
}
```

### Export GLB meshes

```java
import com.github.schemat.nucleation.*;
import java.nio.file.Files;
import java.nio.file.Path;

try (Schematic s = Schematic.fromLitematic(Files.readAllBytes(Path.of("build.litematic")));
     ResourcePack pack = ResourcePack.fromZipPath(Path.of("resources.zip"));
     MeshConfig cfg = new MeshConfig().greedyMeshing(true).atlasMaxSize(2048);
     MeshResult mesh = s.mesh(pack, cfg)) {
    Files.write(Path.of("out.glb"), mesh.glbData());
    System.out.printf("Mesh: %d verts, %d tris, bounds=%s%n",
            mesh.vertexCount(), mesh.triangleCount(), java.util.Arrays.toString(mesh.bounds()));
}
```

For multi-region schematics, use `meshByRegion(pack)` which returns a
`MultiMeshResult` you can iterate. Check `Nucleation.hasMeshing()` before
calling if you want to handle cdylibs built without the feature.

### Redstone simulation (feature-gated)

```java
if (!Nucleation.hasSimulation()) {
    throw new IllegalStateException("This build was not compiled with simulation");
}
try (Schematic s = Schematic.fromLitematic(Files.readAllBytes(Path.of("circuit.litematic")));
     MchprsWorld world = new MchprsWorld(s)) {
    world.tick(20); // 20 game ticks (1 second)
    try (Schematic after = world.getSchematic()) {
        Files.write(Path.of("after.litematic"), after.toLitematic());
    }
}
```

## Consuming the JAR

### Gradle (local)

For now (GitHub Releases distribution):

```kotlin
dependencies {
    implementation(files("libs/nucleation-0.2.8.jar"))
}
```

Drop the fat JAR into `libs/`. The runtime loader detects the host platform
and extracts the matching cdylib from the JAR on first use.

### From a mod

The library is platform-agnostic — it does not depend on Fabric, Forge, or
NeoForge. To use it from inside a mod, drop the JAR into the mod's
`build/libs/` dependencies, and consider declaring a class-loader-isolated
extraction path if multiple mods might bundle different Nucleation versions:

```java
System.setProperty(
    "nucleation.native.dir",
    "/path/to/mods/nucleation/" + Nucleation.version()
);
```

## API surface

| Class            | Mirrors            | Notes                                  |
|------------------|--------------------|----------------------------------------|
| `Schematic`      | `PySchematic`      | `AutoCloseable`, `Iterable<Block>`    |
| `BlockState`     | `PyBlockState`     | Immutable; `withProperty` returns new |
| `Block`          | (record)           | Decoded value from iteration          |
| `Dimensions`     | (record)           | width × height × length               |
| `Shape`          | `PyShape`          | 12 primitive + 4 composite ops        |
| `Brush`          | `PyBrush`          | `solid`, `color`                       |
| `BuildingTool`   | `PyBuildingTool`   | Static `fill` / `rstack` helpers      |
| `SchematicBuilder` | `PySchematicBuilder` | Fluent ASCII-art builder         |
| `MchprsWorld`    | `PyMchprsWorld`    | Feature-gated redstone sim            |
| `ResourcePack`   | `PyResourcePack`   | Textures + models for meshing         |
| `MeshConfig`     | `PyMeshConfig`     | Mesh-build parameters (fluent)        |
| `MeshResult`     | `PyMeshResult`     | `glbData()`, counts, bounds, atlas    |
| `MultiMeshResult`| `PyMultiMeshResult`| Iterable per-region mesh results      |
| `Nucleation`     | (entry point)      | `version()`, `hasSimulation()`, `hasMeshing()` |

API parity with `nucleation-py` is verified by
`tools/check_jvm_parity.rs` and enforced in CI.

## Build from source

### Rust cdylib only

```bash
cargo build --release --manifest-path nucleation-jvm/Cargo.toml
# Produces nucleation-jvm/target/release/libnucleation_jvm.{so,dylib,dll}
```

### Full fat JAR (host platform only)

```bash
cd nucleation-jvm/jvm
./gradlew jar
# Produces build/libs/nucleation-<version>.jar containing only the cdylib
# for whatever OS+arch you're currently on.
```

This is fine for local development on a single machine, but if you intend
to run the JAR somewhere else (e.g. a Docker container with a different
OS or arch than your host), you need a multi-platform JAR — see below.

### Multi-platform fat JAR — three options

#### 1. Use the CI-built JAR (recommended)

CI builds and uploads a JAR with all 5 platforms bundled on every push.
Tagged releases attach the JAR to GitHub Releases. This is the artifact
to use in production.

```bash
gh release download v0.2.8 -p '*.jar' -R Schem-at/Nucleation
# Or: download from https://github.com/Schem-at/Nucleation/releases
```

#### 2. Build all platforms locally with `crossJar` (Docker-backed)

Install [`cross`](https://github.com/cross-rs/cross) once:

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

Then from anywhere in the repo:

```bash
cd nucleation-jvm/jvm
./gradlew crossJar
# Builds all 5 cdylibs (uses Docker for non-host targets via `cross`),
# stages them into resources/native/<platform>/, then assembles the JAR.
```

Requires Docker to be running. Adds about 30 s per non-host target on
the first build, much less after caching.

#### 3. Build just the platform(s) you need

If you only need, say, `linux-arm64` for a Docker target on Apple Silicon:

```bash
cd nucleation-jvm
./build-cross.sh linux-arm64
cd jvm && ./gradlew jar
# The JAR now contains the macOS-arm64 host cdylib (from earlier builds)
# PLUS the freshly cross-compiled linux-arm64 one.
```

Or pass multiple targets:

```bash
./build-cross.sh linux-x64 linux-arm64 windows-x64
```

#### 4. Runtime override (no rebuild)

If you can't or don't want to rebuild the JAR, point at an external cdylib:

```bash
java -Dnucleation.native.path=/abs/path/to/libnucleation_jvm.so -jar your-app.jar
```

or a directory layout:

```bash
java -Dnucleation.native.dir=/abs/path/with/native -jar your-app.jar
# expects /abs/path/with/native/<platform>/<libfile>
```

### Running tests

```bash
cd nucleation-jvm/jvm
./gradlew test
```

## How loading works

1. First reference to any `com.github.schemat.nucleation.*` class triggers
   the static initialiser of `NucleationNative`.
2. `NativeLoader.loadOnce()` detects the host platform, extracts the
   matching cdylib from `/native/<platform>/` inside the JAR to a
   per-version-per-content-hash temp directory, then calls
   `System.load(...)`.
3. `JNI_OnLoad` runs once on first load, calling `RegisterNatives` for
   every export. After that, all method calls cross the JNI boundary in
   the usual sub-microsecond fashion.

Override knobs:

| System property              | Effect                                              |
|------------------------------|-----------------------------------------------------|
| `nucleation.native.path`     | Absolute path to a cdylib to `System.load` directly |
| `nucleation.native.dir`      | Directory containing `<platform>/libnucleation_jvm.*` |
| `nucleation.native.debug`    | `true` to log loader steps to stderr               |

## License

AGPL-3.0-only. The upstream rights-holder is Schem-at; if you need to
ship a closed-source mod or distribute under a different license, contact
<nano@schem.at> about dual-licensing.
