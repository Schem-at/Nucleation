// Smoke program run by CI (and `gradle smokeRun`) against the assembled jar:
// JNA has to find, extract, and load the bundled cdylib for the current
// platform, and the core schematic surface has to actually work — catching
// jars that package no (or a broken) native library.
//
// Lives outside src/ because tools/gen-bindings.sh wipes bindings/kotlin/src
// wholesale; this file is hand-maintained, like the gradle build scripts.
import at.schem.nucleation.Schematic
import at.schem.nucleation.AnimationEffect
import at.schem.nucleation.BuildAnimation

fun main() {
    val s = Schematic.create("smoke")
    s.setBlockFromString(
        0, 0, 0,
        "minecraft:chest[facing=west]{Items:[{Slot:0b,id:\"minecraft:diamond\",Count:64b}]}",
    ).getOrThrow()
    val name = s.getBlockName(0, 0, 0).getOrThrow()
    check(name == "minecraft:chest") { "expected minecraft:chest, got '$name'" }

    val animation = BuildAnimation.create("fluent")
    val effect = AnimationEffect.spinIn(600.0f, 1.0f)
    check(animation.withEffect(effect).setBlock(0, 0, 0, "minecraft:stone").getOrThrow() == 0u)
    check(animation.setBlock(1, 0, 0, "minecraft:dirt").getOrThrow() == 1u)
    check(animation.groupCount() == 2u)

    println("JVM smoke OK: schematic + fluent animation effect")
}
