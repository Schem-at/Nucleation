// Builds the JVM artifact from the Diplomat-generated Kotlin sources in src/.
// The generated code loads the `nucleation` native library through JNA; CI copies the
// per-platform cdylibs into src/main/resources/<os>-<arch>/ before `gradle jar` so JNA
// can extract them from the JAR at runtime (JNA's default classpath loading layout).
plugins {
    kotlin("jvm") version "2.0.20"
    `java-library`
}

group = "at.schem"
version = (findProperty("nucleationVersion") as String?) ?: "0.3.0"

repositories {
    mavenCentral()
}

dependencies {
    api("net.java.dev.jna:jna:5.14.0")
}

kotlin {
    jvmToolchain(21)
}

// Smoke program (smoke/Smoke.kt) lives outside src/ because
// tools/gen-bindings.sh wipes bindings/kotlin/src wholesale on regeneration.
val smoke: SourceSet by sourceSets.creating {
    java.setSrcDirs(emptyList<String>())
    resources.setSrcDirs(emptyList<String>())
    kotlin.setSrcDirs(listOf("smoke"))
}

dependencies {
    "smokeImplementation"(sourceSets.main.get().output)
    "smokeImplementation"("net.java.dev.jna:jna:5.14.0")
}

// `gradle smokeRun` builds the jar, then runs the smoke program with the jar
// first on the classpath, so classes AND the bundled native library resolve
// from the packaged artifact — proving the jar is self-contained on the
// current platform (JNA extracts the cdylib from the jar's resources).
tasks.register<JavaExec>("smokeRun") {
    group = "verification"
    description = "Runs smoke/Smoke.kt against the assembled jar."
    dependsOn(tasks.jar)
    classpath = files(tasks.jar) + smoke.output + configurations["smokeRuntimeClasspath"]
    mainClass.set("SmokeKt")
}
