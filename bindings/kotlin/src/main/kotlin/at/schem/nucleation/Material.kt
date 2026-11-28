package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MaterialLib: Library {
    fun Material_destroy(handle: Pointer)
    fun Material_solid(block: Slice): ResultPointerInt
    fun Material_skip(): Pointer
    fun Material_color(r: FFIUint8, g: FFIUint8, b: FFIUint8, palette: Pointer): Pointer
    fun Material_from_brush(brush: Pointer): Pointer
    fun Material_channel_gradient(channel: Int, stops: Slice, colors: Slice, lo: Float, hi: Float, palette: Pointer, space: Int): ResultPointerInt
    fun Material_field_gradient(field: Pointer, stops: Slice, colors: Slice, lo: Float, hi: Float, palette: Pointer, space: Int): ResultPointerInt
    fun Material_channel_ramp(channel: Int, lo: Float, hi: Float, blocksJson: Slice, dither: Boolean): ResultPointerInt
    fun Material_field_ramp(field: Pointer, lo: Float, hi: Float, blocksJson: Slice, dither: Boolean): ResultPointerInt
    fun Material_weighted(blocksJson: Slice, weights: Slice, seed: Int): ResultPointerInt
    fun Material_shaded(r: FFIUint8, g: FFIUint8, b: FFIUint8, lx: Float, ly: Float, lz: Float, ambient: Float, palette: Pointer): ResultPointerInt
    fun Material_spotlight(px: Float, py: Float, pz: Float, dx: Float, dy: Float, dz: Float, coneAngleDeg: Float, r: FFIUint8, g: FFIUint8, b: FFIUint8, ambient: Float, palette: Pointer): ResultPointerInt
}
/** Composable block shader for [VoxelPipeline].
*/
class Material internal constructor (
    internal val handle: Pointer,
    // These ensure that anything that is borrowed is kept alive and not cleaned
    // up by the garbage collector.
    internal val selfEdges: List<Any>,
    internal var owned: Boolean,
)  {

    init {
        if (this.owned) {
            this.registerCleaner()
        }
    }

    private class MaterialCleaner(val handle: Pointer, val lib: MaterialLib) : Runnable {
        override fun run() {
            lib.Material_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Material.MaterialCleaner(handle, Material.lib));
    }

    companion object {
        internal val libClass: Class<MaterialLib> = MaterialLib::class.java
        internal val lib: MaterialLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun solid(block: String): Result<Material> {
            val blockSliceMemory = PrimitiveArrayTools.borrowUtf8(block)

            val returnVal = lib.Material_solid(blockSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Material(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                blockSliceMemory.close()
            }
        }
        @JvmStatic

        fun skip(): Material {

            val returnVal = lib.Material_skip();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = Material(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun color(r: UByte, g: UByte, b: UByte, palette: Palette): Material {

            val returnVal = lib.Material_color(FFIUint8(r), FFIUint8(g), FFIUint8(b), palette.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = Material(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun fromBrush(brush: Brush): Material {

            val returnVal = lib.Material_from_brush(brush.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = Material(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun channelGradient(channel: FieldChannel, stops: FloatArray, colors: UByteArray, lo: Float, hi: Float, palette: Palette, space: InterpolationSpace): Result<Material> {
            val stopsSliceMemory = PrimitiveArrayTools.borrow(stops)
            val colorsSliceMemory = PrimitiveArrayTools.borrow(colors)

            val returnVal = lib.Material_channel_gradient(channel.toNative(), stopsSliceMemory.slice, colorsSliceMemory.slice, lo, hi, palette.handle, space.toNative());
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Material(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                stopsSliceMemory.close()
                colorsSliceMemory.close()
            }
        }
        @JvmStatic

        fun fieldGradient(field: Field, stops: FloatArray, colors: UByteArray, lo: Float, hi: Float, palette: Palette, space: InterpolationSpace): Result<Material> {
            val stopsSliceMemory = PrimitiveArrayTools.borrow(stops)
            val colorsSliceMemory = PrimitiveArrayTools.borrow(colors)

            val returnVal = lib.Material_field_gradient(field.handle, stopsSliceMemory.slice, colorsSliceMemory.slice, lo, hi, palette.handle, space.toNative());
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Material(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                stopsSliceMemory.close()
                colorsSliceMemory.close()
            }
        }
        @JvmStatic

        fun channelRamp(channel: FieldChannel, lo: Float, hi: Float, blocksJson: String, dither: Boolean): Result<Material> {
            val blocksJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(blocksJson)

            val returnVal = lib.Material_channel_ramp(channel.toNative(), lo, hi, blocksJsonSliceMemory.slice, dither);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Material(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                blocksJsonSliceMemory.close()
            }
        }
        @JvmStatic

        fun fieldRamp(field: Field, lo: Float, hi: Float, blocksJson: String, dither: Boolean): Result<Material> {
            val blocksJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(blocksJson)

            val returnVal = lib.Material_field_ramp(field.handle, lo, hi, blocksJsonSliceMemory.slice, dither);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Material(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                blocksJsonSliceMemory.close()
            }
        }
        @JvmStatic

        fun weighted(blocksJson: String, weights: FloatArray, seed: Int): Result<Material> {
            val blocksJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(blocksJson)
            val weightsSliceMemory = PrimitiveArrayTools.borrow(weights)

            val returnVal = lib.Material_weighted(blocksJsonSliceMemory.slice, weightsSliceMemory.slice, seed);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Material(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                blocksJsonSliceMemory.close()
                weightsSliceMemory.close()
            }
        }
        @JvmStatic

        fun shaded(r: UByte, g: UByte, b: UByte, lx: Float, ly: Float, lz: Float, ambient: Float, palette: Palette): Result<Material> {

            val returnVal = lib.Material_shaded(FFIUint8(r), FFIUint8(g), FFIUint8(b), lx, ly, lz, ambient, palette.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Material(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun spotlight(px: Float, py: Float, pz: Float, dx: Float, dy: Float, dz: Float, coneAngleDeg: Float, r: UByte, g: UByte, b: UByte, ambient: Float, palette: Palette): Result<Material> {

            val returnVal = lib.Material_spotlight(px, py, pz, dx, dy, dz, coneAngleDeg, FFIUint8(r), FFIUint8(g), FFIUint8(b), ambient, palette.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Material(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }

}
