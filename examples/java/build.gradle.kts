plugins {
    application
}

java {
    toolchain { languageVersion.set(JavaLanguageVersion.of(21)) }
}

repositories { mavenCentral() }

dependencies {
    implementation(files("../../nucleation-jvm/jvm/build/libs/nucleation-0.2.13.jar"))
}

application {
    mainClass.set("examples.NucleationDemo")
}
