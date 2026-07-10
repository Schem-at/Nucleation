plugins {
    `java-library`
    `maven-publish`
}

group = "com.github.schemat"
version = providers.gradleProperty("nucleationVersion").orElse("0.2.18").get()

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
    withJavadocJar()
    withSourcesJar()
}

repositories {
    mavenCentral()
}

dependencies {
    testImplementation(platform("org.junit:junit-bom:5.10.2"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

// Where the local-development build expects the host cdylib to live so tests
// can run without the fat-JAR resource layout. CI overrides this by populating
// src/main/resources/native/<os>-<arch>/ directly.
val hostNativeDir: Directory = layout.projectDirectory.dir("../target/release")

val collectNatives by tasks.registering(Copy::class) {
    val dest = layout.buildDirectory.dir("native-staging/native")
    into(dest)

    // Host platform: copy whatever release artifact exists into the right
    // platform-suffixed subdir so the loader can find it during tests.
    val osName = System.getProperty("os.name").lowercase()
    val osArch = System.getProperty("os.arch").lowercase()
    val platform = when {
        osName.contains("mac") && osArch.contains("aarch64") -> "macos-arm64"
        osName.contains("mac") -> "macos-x64"
        osName.contains("linux") && osArch.contains("aarch64") -> "linux-arm64"
        osName.contains("linux") -> "linux-x64"
        osName.contains("windows") -> "windows-x64"
        else -> "unknown"
    }
    val ext = when {
        osName.contains("mac") -> "dylib"
        osName.contains("windows") -> "dll"
        else -> "so"
    }
    val prefix = if (osName.contains("windows")) "" else "lib"

    from(hostNativeDir) {
        include("${prefix}nucleation_jvm.$ext")
        into(platform)
    }

    // Note: pre-staged binaries under src/main/resources/native/<platform>/
    // are already on the default resources path, so we do NOT re-copy them
    // here. Doing so would produce duplicate entries in the JAR when CI
    // populates that directory from the build-jvm-cdylib matrix. Gradle 9+
    // treats duplicates as a hard error.
}

tasks.processResources {
    dependsOn(collectNatives)
    from(layout.buildDirectory.dir("native-staging"))
    // Safety net: if the host cargo target ever overlaps a pre-staged cdylib
    // for the same platform (e.g. someone runs cargo build on a linux-x64
    // CI host while linux-x64 is also in src/main/resources), keep the
    // first-encountered binary and silently drop the duplicate.
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
}

tasks.test {
    useJUnitPlatform()
    dependsOn(collectNatives)
    // Make the staged natives visible at runtime via the classpath JAR.
    systemProperty("nucleation.native.debug", "true")
    testLogging {
        events("passed", "failed", "skipped")
        showStandardStreams = true
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }
}

tasks.jar {
    archiveBaseName.set("nucleation")
    manifest {
        attributes(
            "Implementation-Title" to "Nucleation",
            "Implementation-Version" to project.version,
            "Implementation-Vendor" to "Schem-at"
        )
    }
}

/**
 * Build cdylibs for all 5 supported platforms via the cross-build script,
 * then assemble a true fat JAR containing every native binary. Requires
 * `cross` (https://github.com/cross-rs/cross) and a running Docker daemon
 * for the non-host targets.
 *
 * Usage:
 *   ./gradlew crossJar                          # all 5 platforms
 *   ./gradlew crossJar -PcrossTargets=linux-arm64        # subset
 *   ./gradlew crossJar -PcrossTargets="linux-x64 linux-arm64"
 *
 * The fat JAR ends up at build/libs/nucleation-<version>.jar with
 * native/<platform>/<libnucleation_jvm>.<ext> for every requested target.
 */
val crossBuild by tasks.registering(Exec::class) {
    group = "build"
    description = "Cross-compile cdylibs for every supported platform"
    workingDir = file("..")
    val targets = providers.gradleProperty("crossTargets").orNull
    val args = mutableListOf<String>("./build-cross.sh")
    if (!targets.isNullOrBlank()) {
        args.addAll(targets.split(Regex("\\s+")))
    }
    commandLine = args
}

val crossJar by tasks.registering {
    group = "build"
    description = "Cross-compile every platform's cdylib then assemble the fat JAR"
    dependsOn(crossBuild)
    finalizedBy(tasks.jar)
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            artifactId = "nucleation"
            from(components["java"])
            pom {
                name.set("Nucleation")
                description.set("JVM bindings for the Nucleation Minecraft schematic library")
                url.set("https://github.com/Schem-at/Nucleation")
                licenses {
                    license {
                        name.set("MIT")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                }
            }
        }
    }
}
