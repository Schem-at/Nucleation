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
import at.schem.nucleation.SdfExpr
import at.schem.nucleation.FieldProgramBinaryOp
import at.schem.nucleation.FieldProgramDistanceKind
import at.schem.nucleation.FieldProgramDsl
import at.schem.nucleation.FieldProgramUnaryOp
import at.schem.nucleation.FieldProgramValueType

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

    val field = SdfExpr.sphere(4.0f)
        .smoothUnion(SdfExpr.sphere(3.0f).translate(4.0f, 0.0f, 0.0f), 1.0f)
        .displace(0.5f, 0.1f, 7)
    check(field.evalAt(0.0f, 0.0f, 0.0f) < 0.0f)
    field.toShape()

    val programDsl = FieldProgramDsl.create()
    val distance = programDsl.addSlot(FieldProgramValueType.Scalar)
    val program = programDsl
        .output(distance)
        .bounds(-4.0f, -4.0f, -4.0f, 4.0f, 4.0f, 4.0f)
        .distanceKind(FieldProgramDistanceKind.Exact)
        .pushPosition()
        .unary(FieldProgramUnaryOp.Length)
        .pushConst(2.0f)
        .binary(FieldProgramBinaryOp.Sub)
        .store(distance)
        .build()
    check(program.toJson().getOrThrow().contains("\"version\":1"))
    check(SdfExpr.fromProgram(program).evalAt(0.0f, 0.0f, 0.0f) < 0.0f)

    val iq = SdfExpr.squarePyramid(2.0f, 4.0f)
        .elongate(1.0f, 0.0f, 1.0f)
        .twist(0.1f)
        .bend(0.05f)
        .xorWith(SdfExpr.sphere(1.0f))
    check(iq.evalAt(4.0f, 0.0f, 0.0f).isFinite())
    check(SdfExpr.infiniteCone(45.0f).evalAt(0.0f, -1.0f, 0.0f) > 0.0f)

    val link = SdfExpr.link(halfLength = 7.0f, majorRadius = 3.0f, minorRadius = 0.5f)
    val linkTubeCenter = link.evalAt(3.0f, 0.0f, 7.0f)
    check(kotlin.math.abs(linkTubeCenter + 0.5f) < 1.0e-4f) {
        "SdfExpr.link argument order mismatch: distance=$linkTubeCenter"
    }

    println("JVM smoke OK: schematic + fluent animation effect + typed/program SDF")
}
