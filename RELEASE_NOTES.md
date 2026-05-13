# Nucleation v0.2.10

Build script fix. v0.2.9's `assemble-jvm-jar` job failed at the
`processResources` step under Gradle 9:

    Entry native/linux-arm64/libnucleation_jvm.so is a duplicate but no
    duplicate handling strategy has been set.

Two compounding sources of the duplicate:

1. `collectNatives` was copying `src/main/resources/native/**/*.{so,
   dylib,dll}` into `build/native-staging/`. Those files were already
   on the default resources classpath, so they got bundled twice.
2. `processResources` had no `duplicatesStrategy` set, which under
   Gradle 9 (strict by default) fails the build instead of warning.

Fixed in `nucleation-jvm/jvm/build.gradle.kts`:
- Dropped the redundant `preStaged` from() in `collectNatives` — pre-
  staged cdylibs reach the JAR through the default resources path
  alone, no need to re-copy them.
- Added `duplicatesStrategy = DuplicatesStrategy.EXCLUDE` to
  `processResources` as a safety net in case the host cargo target and
  a pre-staged cdylib happen to overlap on the same platform.

No source / API changes since v0.2.7.

v0.2.8 retired (deprecated macos-13 runner).
v0.2.9 retired (Gradle 9 duplicate-resources failure).

See v0.2.7 release notes for the feature work.
