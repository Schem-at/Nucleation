package at.schem.nucleation

/**
 * Java-friendly, exception-based façade for [FieldProgramBuilder].
 *
 * Slot IDs and repeat counts use signed JVM [Int] values and are range checked
 * before conversion to Kotlin unsigned types. Mutating methods return this
 * builder so Java and Kotlin callers can use a fluent instruction stream.
 */
class FieldProgramDsl private constructor(private val raw: FieldProgramBuilder) {
    companion object {
        @JvmStatic fun create() = FieldProgramDsl(FieldProgramBuilder.create())
    }

    fun addSlot(type: FieldProgramValueType): Int =
        raw.addSlot(type).getOrThrow().toInt()

    fun pushConst(value: Float) = apply { raw.pushConstScalar(value).getOrThrow() }
    fun pushConst(x: Float, y: Float, z: Float) = apply {
        raw.pushConstVec3(x, y, z).getOrThrow()
    }
    fun pushConst(value: Boolean) = apply { raw.pushConstBool(value).getOrThrow() }
    fun pushPosition() = apply { raw.pushPos().getOrThrow() }
    fun load(slot: Int) = apply { raw.loadLocal(slotId(slot)).getOrThrow() }
    fun store(slot: Int) = apply { raw.storeLocal(slotId(slot)).getOrThrow() }
    fun pop() = apply { raw.pop().getOrThrow() }
    fun unary(op: FieldProgramUnaryOp) = apply { raw.unaryOp(op).getOrThrow() }
    fun binary(op: FieldProgramBinaryOp) = apply { raw.binaryOp(op).getOrThrow() }
    fun clamp() = apply { raw.clamp().getOrThrow() }
    fun select() = apply { raw.select().getOrThrow() }
    fun makeVec3() = apply { raw.makeVec3().getOrThrow() }
    fun breakIf() = apply { raw.breakIf().getOrThrow() }

    fun beginRepeat(count: Int) = apply {
        require(count > 0) { "repeat count must be positive" }
        raw.beginRepeat(count.toUInt()).getOrThrow()
    }
    fun endRepeat() = apply { raw.endRepeat().getOrThrow() }

    fun output(slot: Int) = apply { raw.setOutput(slotId(slot)).getOrThrow() }
    fun bounds(
        minX: Float, minY: Float, minZ: Float,
        maxX: Float, maxY: Float, maxZ: Float,
    ) = apply {
        raw.setBounds(minX, minY, minZ, maxX, maxY, maxZ).getOrThrow()
    }
    fun distanceKind(kind: FieldProgramDistanceKind) = apply {
        raw.setDistanceKind(kind).getOrThrow()
    }

    /** Validate and consume this builder. */
    fun build(): FieldProgram = raw.build().getOrThrow()

    private fun slotId(slot: Int): UShort {
        require(slot in 0..UShort.MAX_VALUE.toInt()) { "slot must fit an unsigned 16-bit ID" }
        return slot.toUShort()
    }
}
