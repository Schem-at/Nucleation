package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SdfLib: Library {
    fun Sdf_destroy(handle: Pointer)
    fun Sdf_sphere(radius: Float): ResultPointerInt
    fun Sdf_box_shape(halfX: Float, halfY: Float, halfZ: Float, rounding: Float): ResultPointerInt
    fun Sdf_ellipsoid(radiusX: Float, radiusY: Float, radiusZ: Float): ResultPointerInt
    fun Sdf_torus(majorRadius: Float, minorRadius: Float): ResultPointerInt
    fun Sdf_capsule(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, radius: Float): ResultPointerInt
    fun Sdf_capped_cylinder(radius: Float, halfHeight: Float): ResultPointerInt
    fun Sdf_capped_cone(halfHeight: Float, bottomRadius: Float, topRadius: Float): ResultPointerInt
    fun Sdf_plane(normalX: Float, normalY: Float, normalZ: Float, offset: Float): ResultPointerInt
    fun Sdf_octahedron(size: Float): ResultPointerInt
    fun Sdf_hex_prism(radius: Float, halfHeight: Float): ResultPointerInt
    fun Sdf_super_prism(halfX: Float, halfY: Float, halfZ: Float, exponent: Float): ResultPointerInt
    fun Sdf_cells(frequency: Float, seed: Int, jitter: Float, mode: Int, threshold: Float): ResultPointerInt
    fun Sdf_union_with(handle: Pointer, other: Pointer): Pointer
    fun Sdf_intersection_with(handle: Pointer, other: Pointer): Pointer
    fun Sdf_subtract(handle: Pointer, other: Pointer): Pointer
    fun Sdf_smooth_union(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Sdf_smooth_subtract(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Sdf_smooth_intersection(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Sdf_rounded(handle: Pointer, radius: Float): ResultPointerInt
    fun Sdf_shell(handle: Pointer, thickness: Float): ResultPointerInt
    fun Sdf_translate(handle: Pointer, x: Float, y: Float, z: Float): ResultPointerInt
    fun Sdf_rotate(handle: Pointer, xDegrees: Float, yDegrees: Float, zDegrees: Float): ResultPointerInt
    fun Sdf_scale(handle: Pointer, factor: Float): ResultPointerInt
    fun Sdf_mirror(handle: Pointer, axis: Int): Pointer
    fun Sdf_repeat_infinite(handle: Pointer, spacingX: Float, spacingY: Float, spacingZ: Float): ResultPointerInt
    fun Sdf_repeat_counted(handle: Pointer, spacingX: Float, spacingY: Float, spacingZ: Float, countX: FFIUint32, countY: FFIUint32, countZ: FFIUint32): ResultPointerInt
    fun Sdf_displace(handle: Pointer, amplitude: Float, frequency: Float, seed: Int, octaves: FFIUint32): ResultPointerInt
    fun Sdf_warp(handle: Pointer, amplitude: Float, frequency: Float, seed: Int): ResultPointerInt
    fun Sdf_eval_at(handle: Pointer, x: Float, y: Float, z: Float): Float
    fun Sdf_normal(handle: Pointer, x: Float, y: Float, z: Float, epsilon: Float): ResultSdfNormalNativeInt
    fun Sdf_bounds(handle: Pointer): ResultSdfBoundsNativeInt
    fun Sdf_to_shape(handle: Pointer): ResultPointerInt
    fun Sdf_to_shape_bounded(handle: Pointer, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun Sdf_from_json_string(json: Slice): ResultPointerInt
    fun Sdf_to_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun Sdf_schematic_from_sdf_auto(sdfJson: Slice, rulesJson: Slice): ResultPointerInt
    fun Sdf_schematic_from_sdf(sdfJson: Slice, rulesJson: Slice, hasBounds: Boolean, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun Sdf_eval(sdfJson: Slice, x: Float, y: Float, z: Float): ResultFloatInt
}
/** An immutable, composable signed-distance-field expression graph.
*
*Primitive constructors and every combinator return a new graph, so values
*can be shared safely between Flow nodes and across Kotlin/Java, JavaScript,
*and Python bindings. JSON is retained only for explicit import/export and
*the legacy sampling helpers.
*/
class Sdf internal constructor (
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

    private class SdfCleaner(val handle: Pointer, val lib: SdfLib) : Runnable {
        override fun run() {
            lib.Sdf_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Sdf.SdfCleaner(handle, Sdf.lib));
    }

    companion object {
        internal val libClass: Class<SdfLib> = SdfLib::class.java
        internal val lib: SdfLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun sphere(radius: Float): Result<Sdf> {

            val returnVal = lib.Sdf_sphere(radius);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        /** Axis-aligned rounded box, centered at the origin.
        */
        fun boxShape(halfX: Float, halfY: Float, halfZ: Float, rounding: Float): Result<Sdf> {

            val returnVal = lib.Sdf_box_shape(halfX, halfY, halfZ, rounding);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun ellipsoid(radiusX: Float, radiusY: Float, radiusZ: Float): Result<Sdf> {

            val returnVal = lib.Sdf_ellipsoid(radiusX, radiusY, radiusZ);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun torus(majorRadius: Float, minorRadius: Float): Result<Sdf> {

            val returnVal = lib.Sdf_torus(majorRadius, minorRadius);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun capsule(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, radius: Float): Result<Sdf> {

            val returnVal = lib.Sdf_capsule(ax, ay, az, bx, by, bz, radius);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun cappedCylinder(radius: Float, halfHeight: Float): Result<Sdf> {

            val returnVal = lib.Sdf_capped_cylinder(radius, halfHeight);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun cappedCone(halfHeight: Float, bottomRadius: Float, topRadius: Float): Result<Sdf> {

            val returnVal = lib.Sdf_capped_cone(halfHeight, bottomRadius, topRadius);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun plane(normalX: Float, normalY: Float, normalZ: Float, offset: Float): Result<Sdf> {

            val returnVal = lib.Sdf_plane(normalX, normalY, normalZ, offset);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun octahedron(size: Float): Result<Sdf> {

            val returnVal = lib.Sdf_octahedron(size);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun hexPrism(radius: Float, halfHeight: Float): Result<Sdf> {

            val returnVal = lib.Sdf_hex_prism(radius, halfHeight);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun superPrism(halfX: Float, halfY: Float, halfZ: Float, exponent: Float): Result<Sdf> {

            val returnVal = lib.Sdf_super_prism(halfX, halfY, halfZ, exponent);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun cells(frequency: Float, seed: Int, jitter: Float, mode: SdfCellMode, threshold: Float): Result<Sdf> {

            val returnVal = lib.Sdf_cells(frequency, seed, jitter, mode.toNative(), threshold);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Sdf(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun fromJsonString(json: String): Result<Sdf> {
            val jsonSliceMemory = PrimitiveArrayTools.borrowUtf8(json)

            val returnVal = lib.Sdf_from_json_string(jsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Sdf(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                jsonSliceMemory.close()
            }
        }
        @JvmStatic

        /** Legacy JSON-first terrain helper. Prefer typed constructors and
        *`to_shape()` with `BuildingTool.fill()` for new code.
        */
        fun schematicFromSdfAuto(sdfJson: String, rulesJson: String): Result<Schematic> {
            val sdfJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(sdfJson)
            val rulesJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(rulesJson)

            val returnVal = lib.Sdf_schematic_from_sdf_auto(sdfJsonSliceMemory.slice, rulesJsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sdfJsonSliceMemory.close()
                rulesJsonSliceMemory.close()
            }
        }
        @JvmStatic

        /** Legacy JSON-first terrain helper with optional explicit bounds.
        */
        fun schematicFromSdf(sdfJson: String, rulesJson: String, hasBounds: Boolean, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Schematic> {
            val sdfJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(sdfJson)
            val rulesJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(rulesJson)

            val returnVal = lib.Sdf_schematic_from_sdf(sdfJsonSliceMemory.slice, rulesJsonSliceMemory.slice, hasBounds, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sdfJsonSliceMemory.close()
                rulesJsonSliceMemory.close()
            }
        }
        @JvmStatic

        /** Legacy JSON-first evaluator. Prefer `Sdf.from_json_string(...).eval_at(...)`.
        */
        fun eval(sdfJson: String, x: Float, y: Float, z: Float): Result<Float> {
            val sdfJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(sdfJson)

            val returnVal = lib.Sdf_eval(sdfJsonSliceMemory.slice, x, y, z);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return (nativeOkVal).ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sdfJsonSliceMemory.close()
            }
        }
    }

    fun unionWith(other: Sdf): Sdf {

        val returnVal = lib.Sdf_union_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Sdf(handle, selfEdges, true)
        return returnOpaque
    }

    fun intersectionWith(other: Sdf): Sdf {

        val returnVal = lib.Sdf_intersection_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Sdf(handle, selfEdges, true)
        return returnOpaque
    }

    fun subtract(other: Sdf): Sdf {

        val returnVal = lib.Sdf_subtract(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Sdf(handle, selfEdges, true)
        return returnOpaque
    }

    fun smoothUnion(other: Sdf, radius: Float): Result<Sdf> {

        val returnVal = lib.Sdf_smooth_union(handle, other.handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun smoothSubtract(other: Sdf, radius: Float): Result<Sdf> {

        val returnVal = lib.Sdf_smooth_subtract(handle, other.handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun smoothIntersection(other: Sdf, radius: Float): Result<Sdf> {

        val returnVal = lib.Sdf_smooth_intersection(handle, other.handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun rounded(radius: Float): Result<Sdf> {

        val returnVal = lib.Sdf_rounded(handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun shell(thickness: Float): Result<Sdf> {

        val returnVal = lib.Sdf_shell(handle, thickness);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun translate(x: Float, y: Float, z: Float): Result<Sdf> {

        val returnVal = lib.Sdf_translate(handle, x, y, z);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun rotate(xDegrees: Float, yDegrees: Float, zDegrees: Float): Result<Sdf> {

        val returnVal = lib.Sdf_rotate(handle, xDegrees, yDegrees, zDegrees);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun scale(factor: Float): Result<Sdf> {

        val returnVal = lib.Sdf_scale(handle, factor);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun mirror(axis: SdfAxis): Sdf {

        val returnVal = lib.Sdf_mirror(handle, axis.toNative());
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Sdf(handle, selfEdges, true)
        return returnOpaque
    }

    fun repeatInfinite(spacingX: Float, spacingY: Float, spacingZ: Float): Result<Sdf> {

        val returnVal = lib.Sdf_repeat_infinite(handle, spacingX, spacingY, spacingZ);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun repeatCounted(spacingX: Float, spacingY: Float, spacingZ: Float, countX: UInt, countY: UInt, countZ: UInt): Result<Sdf> {

        val returnVal = lib.Sdf_repeat_counted(handle, spacingX, spacingY, spacingZ, FFIUint32(countX), FFIUint32(countY), FFIUint32(countZ));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun displace(amplitude: Float, frequency: Float, seed: Int, octaves: UInt): Result<Sdf> {

        val returnVal = lib.Sdf_displace(handle, amplitude, frequency, seed, FFIUint32(octaves));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun warp(amplitude: Float, frequency: Float, seed: Int): Result<Sdf> {

        val returnVal = lib.Sdf_warp(handle, amplitude, frequency, seed);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun evalAt(x: Float, y: Float, z: Float): Float {

        val returnVal = lib.Sdf_eval_at(handle, x, y, z);
        return (returnVal)
    }

    fun normal(x: Float, y: Float, z: Float, epsilon: Float): Result<SdfNormal> {

        val returnVal = lib.Sdf_normal(handle, x, y, z, epsilon);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = SdfNormal.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun bounds(): Result<SdfBounds> {

        val returnVal = lib.Sdf_bounds(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = SdfBounds.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun toShape(): Result<Shape> {

        val returnVal = lib.Sdf_to_shape(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun toShapeBounded(minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Shape> {

        val returnVal = lib.Sdf_to_shape_bounded(handle, minX, minY, minZ, maxX, maxY, maxZ);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun toJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Sdf_to_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
