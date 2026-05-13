# Java consumer example

Minimal Gradle project that depends on the locally-built Nucleation JAR
and exercises the public API. Mirrors `examples/haskell/`.

## Run

First build the JAR:

```bash
cd ../../nucleation-jvm/jvm
./gradlew jar
cd ../../examples/java
```

Then run the demo:

```bash
./gradlew run --args="../../test-schematics/some_file.litematic"
```
