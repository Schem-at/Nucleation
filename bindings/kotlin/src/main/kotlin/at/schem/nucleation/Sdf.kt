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
    fun Sdf_capped_torus(majorRadius: Float, minorRadius: Float, capAngleDegrees: Float): ResultPointerInt
    fun Sdf_link(majorRadius: Float, minorRadius: Float, halfLength: Float): ResultPointerInt
    fun Sdf_capsule(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, radius: Float): ResultPointerInt
    fun Sdf_round_cone(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, r1: Float, r2: Float): ResultPointerInt
    fun Sdf_solid_angle(radius: Float, angleDegrees: Float): ResultPointerInt
    fun Sdf_cut_sphere(radius: Float, height: Float): ResultPointerInt
    fun Sdf_cut_hollow_sphere(radius: Float, height: Float, thickness: Float): ResultPointerInt
    fun Sdf_capped_cylinder(radius: Float, halfHeight: Float): ResultPointerInt
    fun Sdf_infinite_cylinder(radius: Float): ResultPointerInt
    fun Sdf_capped_cone(halfHeight: Float, bottomRadius: Float, topRadius: Float): ResultPointerInt
    fun Sdf_plane(normalX: Float, normalY: Float, normalZ: Float, offset: Float): ResultPointerInt
    fun Sdf_octahedron(size: Float): ResultPointerInt
    fun Sdf_hex_prism(radius: Float, halfHeight: Float): ResultPointerInt
    fun Sdf_super_prism(halfX: Float, halfY: Float, halfZ: Float, exponent: Float): ResultPointerInt
    fun Sdf_box_frame(halfX: Float, halfY: Float, halfZ: Float, thickness: Float): ResultPointerInt
    fun Sdf_infinite_cone(angleDegrees: Float): ResultPointerInt
    fun Sdf_square_pyramid(halfBase: Float, height: Float): ResultPointerInt
    fun Sdf_cells(frequency: Float, seed: Int, jitter: Float, mode: Int, threshold: Float): ResultPointerInt
    fun Sdf_union_with(handle: Pointer, other: Pointer): Pointer
    fun Sdf_intersection_with(handle: Pointer, other: Pointer): Pointer
    fun Sdf_subtract(handle: Pointer, other: Pointer): Pointer
    fun Sdf_smooth_union(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Sdf_smooth_subtract(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Sdf_smooth_intersection(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Sdf_rounded(handle: Pointer, radius: Float): ResultPointerInt
    fun Sdf_shell(handle: Pointer, thickness: Float): ResultPointerInt
    fun Sdf_xor_with(handle: Pointer, other: Pointer): Pointer
    fun Sdf_elongate(handle: Pointer, halfX: Float, halfY: Float, halfZ: Float): ResultPointerInt
    fun Sdf_translate(handle: Pointer, x: Float, y: Float, z: Float): ResultPointerInt
    fun Sdf_rotate(handle: Pointer, xDegrees: Float, yDegrees: Float, zDegrees: Float): ResultPointerInt
    fun Sdf_scale(handle: Pointer, factor: Float): ResultPointerInt
    fun Sdf_mirror(handle: Pointer, axis: Int): Pointer
    fun Sdf_twist(handle: Pointer, amount: Float): ResultPointerInt
    fun Sdf_bend(handle: Pointer, amount: Float): ResultPointerInt
    fun Sdf_repeat_infinite(handle: Pointer, spacingX: Float, spacingY: Float, spacingZ: Float): ResultPointerInt
    fun Sdf_repeat_counted(handle: Pointer, spacingX: Float, spacingY: Float, spacingZ: Float, countX: FFIUint32, countY: FFIUint32, countZ: FFIUint32): ResultPointerInt
    fun Sdf_repeat_points(handle: Pointer, offsets: Slice): ResultPointerInt
    fun Sdf_displace(handle: Pointer, amplitude: Float, frequency: Float, seed: Int, octaves: FFIUint32): ResultPointerInt
    fun Sdf_offset_by_field(handle: Pointer, field: Pointer, amplitude: Float): ResultPointerInt
    fun Sdf_warp(handle: Pointer, amplitude: Float, frequency: Float, seed: Int): ResultPointerInt
    fun Sdf_eval_at(handle: Pointer, x: Float, y: Float, z: Float): Float
    fun Sdf_normal(handle: Pointer, x: Float, y: Float, z: Float, epsilon: Float): ResultSdfNormalNativeInt
    fun Sdf_bounds(handle: Pointer): ResultSdfBoundsNativeInt
    fun Sdf_to_shape(handle: Pointer): ResultPointerInt
    fun Sdf_to_shape_bounded(handle: Pointer, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun Sdf_from_json_string(json: Slice): ResultPointerInt
    fun Sdf_to_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun Sdf_from_program(program: Pointer): Pointer
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

        /** Torus ring cut down to an arc. `cap_angle_degrees` is the half-aperture
        *in `(0, 180]`, measured from +X and mirrored across X; `180` is a full
        *torus.
        */
        fun cappedTorus(majorRadius: Float, minorRadius: Float, capAngleDegrees: Float): Result<Sdf> {

            val returnVal = lib.Sdf_capped_torus(majorRadius, minorRadius, capAngleDegrees);
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

        /** Chain-link shape: a torus stretched along Z by `half_length` and
        *capped by two half-tori. `half_length: 0` is a plain torus.
        */
        fun link(majorRadius: Float, minorRadius: Float, halfLength: Float): Result<Sdf> {

            val returnVal = lib.Sdf_link(majorRadius, minorRadius, halfLength);
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

        /** Convex hull of two spheres: a capsule with a linear taper between
        *`r1` (at `a`) and `r2` (at `b`) instead of one constant radius.
        */
        fun roundCone(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, r1: Float, r2: Float): Result<Sdf> {

            val returnVal = lib.Sdf_round_cone(ax, ay, az, bx, by, bz, r1, r2);
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

        /** Sphere of `radius` intersected with an infinite cone of
        *half-aperture `angle_degrees` (in `(0, 180)`) from the +Y axis,
        *apex at the origin.
        */
        fun solidAngle(radius: Float, angleDegrees: Float): Result<Sdf> {

            val returnVal = lib.Sdf_solid_angle(radius, angleDegrees);
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

        /** Sphere cut by the plane `y = height`, keeping the cap above it
        *(a dome). `height` must be strictly between `-radius` and
        *`radius`.
        */
        fun cutSphere(radius: Float, height: Float): Result<Sdf> {

            val returnVal = lib.Sdf_cut_sphere(radius, height);
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

        /** Open (hollow) shell of `cut_sphere`'s dome: just the spherical
        *cap surface, offset by `thickness`, with no flat floor.
        */
        fun cutHollowSphere(radius: Float, height: Float, thickness: Float): Result<Sdf> {

            val returnVal = lib.Sdf_cut_hollow_sphere(radius, height, thickness);
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

        /** Exact Y-axis cylinder with infinite extent. Sampling requires explicit bounds.
        */
        fun infiniteCylinder(radius: Float): Result<Sdf> {

            val returnVal = lib.Sdf_infinite_cylinder(radius);
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

        /** Hollow wireframe box: only the 12 edge beams are solid.
        */
        fun boxFrame(halfX: Float, halfY: Float, halfZ: Float, thickness: Float): Result<Sdf> {

            val returnVal = lib.Sdf_box_frame(halfX, halfY, halfZ, thickness);
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

        /** Exact but unbounded Y-axis infinite cone: apex at the origin,
        *single nappe opening along +Y, half-aperture `angle_degrees`
        *strictly in `(0, 90)`.
        */
        fun infiniteCone(angleDegrees: Float): Result<Sdf> {

            val returnVal = lib.Sdf_infinite_cone(angleDegrees);
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

        /** Square-base pyramid, vertically centered: base (half-extent
        *`half_base`) at `y = -height/2`, apex at `y = height/2`. `height`
        *must be at least the smallest positive normal `f32`.
        */
        fun squarePyramid(halfBase: Float, height: Float): Result<Sdf> {

            val returnVal = lib.Sdf_square_pyramid(halfBase, height);
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

        /** Wrap a validated [FieldProgram] as an `Sdf` graph (cloning it,
        *with its own explicit bounds and distance-kind metadata), so it
        *composes with every other combinator.
        */
        fun fromProgram(program: FieldProgram): Sdf {

            val returnVal = lib.Sdf_from_program(program.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = Sdf(handle, selfEdges, true)
            return returnOpaque
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

    /** Symmetric difference (XOR): solid where exactly one of `self`/
    *`other` is solid.
    */
    fun xorWith(other: Sdf): Sdf {

        val returnVal = lib.Sdf_xor_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Sdf(handle, selfEdges, true)
        return returnOpaque
    }

    /** Stretches this graph with IQ's origin-centered `opElongate` fold.
    *Exactness requires a suitable origin-centered, reflection-symmetric
    *child; off-center/asymmetric children are mirrored and produce only
    *an estimate. Half-lengths must be finite and non-negative, with at
    *least one strictly positive.
    */
    fun elongate(halfX: Float, halfY: Float, halfZ: Float): Result<Sdf> {

        val returnVal = lib.Sdf_elongate(handle, halfX, halfY, halfZ);
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

    /** Twists this graph about the Y axis by `amount` radians per unit
    *Y (IQ's `opTwist`). *Distorted*: not guaranteed exact even when
    *`self` is.
    */
    fun twist(amount: Float): Result<Sdf> {

        val returnVal = lib.Sdf_twist(handle, amount);
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

    /** Cheaply bends this graph by `amount` radians per unit X (IQ's
    *`opCheapBend`). *Distorted*: not guaranteed exact even when
    *`self` is.
    */
    fun bend(amount: Float): Result<Sdf> {

        val returnVal = lib.Sdf_bend(handle, amount);
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

    /** Finite rigid instances of this graph at arbitrary XYZ offsets.
    *`offsets` is flat `[x0, y0, z0, x1, y1, z1, ...]` and may contain
    *at most 4096 points.
    */
    fun repeatPoints(offsets: FloatArray): Result<Sdf> {
        val offsetsSliceMemory = PrimitiveArrayTools.borrow(offsets)

        val returnVal = lib.Sdf_repeat_points(handle, offsetsSliceMemory.slice);
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
            offsetsSliceMemory.close()
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

    /** Offset this surface by a reusable scalar field. The resulting zero
    *set is generally an approximate field, not an exact distance field.
    */
    fun offsetByField(field: Field3, amplitude: Float): Result<Sdf> {

        val returnVal = lib.Sdf_offset_by_field(handle, field.handle, amplitude);
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

    /** Conservative finite bounds, or `NotFound` for an unbounded graph
    *(a bare `plane` or `infinite_cylinder` has no finite extent).
    */
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
