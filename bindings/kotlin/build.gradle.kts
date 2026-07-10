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

tasks.jar {
    from("src/main/resources")
}
