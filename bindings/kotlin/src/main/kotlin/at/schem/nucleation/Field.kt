package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldLib: Library {
    fun Field_destroy(handle: Pointer)
    fun Field_constant(value: Float): ResultPointerInt
    fun Field_coord_x(): Pointer
    fun Field_coord_y(): Pointer
    fun Field_coord_z(): Pointer
    fun Field_from_schematic(schematic: Pointer, pad: FFIUint32): ResultPointerInt
    fun Field_from_glb_bytes(bytes: Slice, maxDim: FFIUint32, pad: FFIUint32): ResultPointerInt
    fun Field_from_obj_str(objText: Slice, maxDim: FFIUint32, pad: FFIUint32): ResultPointerInt
    fun Field_sphere(radius: Float): ResultPointerInt
    fun Field_box_shape(halfX: Float, halfY: Float, halfZ: Float, rounding: Float): ResultPointerInt
    fun Field_ellipsoid(radiusX: Float, radiusY: Float, radiusZ: Float): ResultPointerInt
    fun Field_torus(majorRadius: Float, minorRadius: Float): ResultPointerInt
    fun Field_capped_torus(majorRadius: Float, minorRadius: Float, capAngleDegrees: Float): ResultPointerInt
    fun Field_link(majorRadius: Float, minorRadius: Float, halfLength: Float): ResultPointerInt
    fun Field_capsule(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, radius: Float): ResultPointerInt
    fun Field_round_cone(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, r1: Float, r2: Float): ResultPointerInt
    fun Field_solid_angle(radius: Float, angleDegrees: Float): ResultPointerInt
    fun Field_cut_sphere(radius: Float, height: Float): ResultPointerInt
    fun Field_cut_hollow_sphere(radius: Float, height: Float, thickness: Float): ResultPointerInt
    fun Field_capped_cylinder(radius: Float, halfHeight: Float): ResultPointerInt
    fun Field_infinite_cylinder(radius: Float): ResultPointerInt
    fun Field_capped_cone(halfHeight: Float, bottomRadius: Float, topRadius: Float): ResultPointerInt
    fun Field_plane(normalX: Float, normalY: Float, normalZ: Float, offset: Float): ResultPointerInt
    fun Field_octahedron(size: Float): ResultPointerInt
    fun Field_hex_prism(radius: Float, halfHeight: Float): ResultPointerInt
    fun Field_super_prism(halfX: Float, halfY: Float, halfZ: Float, exponent: Float): ResultPointerInt
    fun Field_box_frame(halfX: Float, halfY: Float, halfZ: Float, thickness: Float): ResultPointerInt
    fun Field_infinite_cone(angleDegrees: Float): ResultPointerInt
    fun Field_square_pyramid(halfBase: Float, height: Float): ResultPointerInt
    fun Field_heightmap(baseY: Float, height: Pointer): ResultPointerInt
    fun Field_value_noise2(frequency: Float, seed: Int): ResultPointerInt
    fun Field_value_noise3(frequency: Float, seed: Int): ResultPointerInt
    fun Field_perlin2(frequency: Float, seed: Int): ResultPointerInt
    fun Field_perlin3(frequency: Float, seed: Int): ResultPointerInt
    fun Field_fbm2(basis: Int, frequency: Float, seed: Int, octaves: FFIUint32, lacunarity: Float, gain: Float, ridged: Boolean): ResultPointerInt
    fun Field_fbm3(basis: Int, frequency: Float, seed: Int, octaves: FFIUint32, lacunarity: Float, gain: Float, ridged: Boolean): ResultPointerInt
    fun Field_value_fbm3(frequency: Float, seed: Int, octaves: FFIUint32): ResultPointerInt
    fun Field_perlin_fbm3(frequency: Float, seed: Int, octaves: FFIUint32): ResultPointerInt
    fun Field_ridged_fbm3(frequency: Float, seed: Int, octaves: FFIUint32): ResultPointerInt
    fun Field_perlin_fbm2(frequency: Float, seed: Int, octaves: FFIUint32): ResultPointerInt
    fun Field_cells(frequency: Float, seed: Int, jitter: Float, mode: Int, threshold: Float): ResultPointerInt
    fun Field_neg(handle: Pointer): Pointer
    fun Field_abs(handle: Pointer): Pointer
    fun Field_sqrt(handle: Pointer): Pointer
    fun Field_sin(handle: Pointer): Pointer
    fun Field_cos(handle: Pointer): Pointer
    fun Field_saturate(handle: Pointer): Pointer
    fun Field_add(handle: Pointer, other: Pointer): Pointer
    fun Field_sub(handle: Pointer, other: Pointer): Pointer
    fun Field_mul(handle: Pointer, other: Pointer): Pointer
    fun Field_div(handle: Pointer, other: Pointer): Pointer
    fun Field_min_with(handle: Pointer, other: Pointer): Pointer
    fun Field_max_with(handle: Pointer, other: Pointer): Pointer
    fun Field_add_const(handle: Pointer, value: Float): ResultPointerInt
    fun Field_mul_const(handle: Pointer, value: Float): ResultPointerInt
    fun Field_clamp(handle: Pointer, lo: Float, hi: Float): ResultPointerInt
    fun Field_smoothstep(handle: Pointer, edge0: Float, edge1: Float): ResultPointerInt
    fun Field_step(handle: Pointer, edge: Float): ResultPointerInt
    fun Field_remap(handle: Pointer, inLo: Float, inHi: Float, outLo: Float, outHi: Float, clamp: Boolean): ResultPointerInt
    fun Field_lerp_with(handle: Pointer, other: Pointer, t: Pointer): Pointer
    fun Field_select(cond: Pointer, thenField: Pointer, elseField: Pointer): Pointer
    fun Field_union_with(handle: Pointer, other: Pointer): Pointer
    fun Field_intersection_with(handle: Pointer, other: Pointer): Pointer
    fun Field_subtract(handle: Pointer, other: Pointer): Pointer
    fun Field_xor_with(handle: Pointer, other: Pointer): Pointer
    fun Field_smooth_union(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Field_smooth_subtract(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Field_smooth_intersection(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Field_chamfer_union(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Field_chamfer_subtract(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Field_chamfer_intersection(handle: Pointer, other: Pointer, radius: Float): ResultPointerInt
    fun Field_steps_union(handle: Pointer, other: Pointer, radius: Float, steps: Float): ResultPointerInt
    fun Field_rounded(handle: Pointer, radius: Float): ResultPointerInt
    fun Field_shell(handle: Pointer, thickness: Float): ResultPointerInt
    fun Field_elongate(handle: Pointer, halfX: Float, halfY: Float, halfZ: Float): ResultPointerInt
    fun Field_displace(handle: Pointer, offset: Pointer, amplitude: Float): ResultPointerInt
    fun Field_displace_noise(handle: Pointer, amplitude: Float, frequency: Float, seed: Int, octaves: FFIUint32): ResultPointerInt
    fun Field_translate(handle: Pointer, x: Float, y: Float, z: Float): ResultPointerInt
    fun Field_rotate(handle: Pointer, xDegrees: Float, yDegrees: Float, zDegrees: Float): ResultPointerInt
    fun Field_scale(handle: Pointer, factor: Float): ResultPointerInt
    fun Field_scale_xyz(handle: Pointer, sx: Float, sy: Float, sz: Float): ResultPointerInt
    fun Field_mirror(handle: Pointer, axis: Int): Pointer
    fun Field_mirror_plane(handle: Pointer, nx: Float, ny: Float, nz: Float, offset: Float): ResultPointerInt
    fun Field_repeat_infinite(handle: Pointer, spacingX: Float, spacingY: Float, spacingZ: Float): ResultPointerInt
    fun Field_repeat_counted(handle: Pointer, spacingX: Float, spacingY: Float, spacingZ: Float, countX: FFIUint32, countY: FFIUint32, countZ: FFIUint32): ResultPointerInt
    fun Field_repeat_polar(handle: Pointer, count: FFIUint32): ResultPointerInt
    fun Field_repeat_points(handle: Pointer, offsets: Slice): ResultPointerInt
    fun Field_twist(handle: Pointer, amount: Float): ResultPointerInt
    fun Field_bend(handle: Pointer, amount: Float): ResultPointerInt
    fun Field_warp(handle: Pointer, warpX: Pointer, warpY: Pointer, warpZ: Pointer, amplitude: Float): ResultPointerInt
    fun Field_warp_noise(handle: Pointer, amplitude: Float, frequency: Float, seed: Int, octaves: FFIUint32): ResultPointerInt
    fun Field_tag(handle: Pointer, materialId: FFIUint16): ResultPointerInt
    fun Field_bounded(handle: Pointer, minX: Float, minY: Float, minZ: Float, maxX: Float, maxY: Float, maxZ: Float): ResultPointerInt
    fun Field_expr(source: Slice): ResultPointerInt
    fun Field_eval_at(handle: Pointer, x: Float, y: Float, z: Float): Float
    fun Field_sample_tagged(handle: Pointer, x: Float, y: Float, z: Float): FieldSampleNative
    fun Field_normal(handle: Pointer, x: Float, y: Float, z: Float, epsilon: Float): ResultFieldNormalNativeInt
    fun Field_bounds(handle: Pointer): ResultFieldBoundsNativeInt
    fun Field_output_range(handle: Pointer): ResultFieldRangeNativeInt
    fun Field_distance_kind(handle: Pointer): Int
    fun Field_to_shape(handle: Pointer): ResultPointerInt
    fun Field_to_shape_bounded(handle: Pointer, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun Field_from_json(json: Slice): ResultPointerInt
    fun Field_to_json(handle: Pointer, write: Pointer): ResultUnitInt
}
/** Unified immutable spatial field / SDF DAG node.
*
*Primitives, CSG (hard, smooth, chamfer, stepped), transforms, domain
*warps, procedural noise, arbitrary math, material tags, and sandboxed
*infix expressions all compose freely and share storage across consumers.
*/
class Field internal constructor (
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

    private class FieldCleaner(val handle: Pointer, val lib: FieldLib) : Runnable {
        override fun run() {
            lib.Field_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Field.FieldCleaner(handle, Field.lib));
    }

    companion object {
        internal val libClass: Class<FieldLib> = FieldLib::class.java
        internal val lib: FieldLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun constant(value: Float): Result<Field> {

            val returnVal = lib.Field_constant(value);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun coordX(): Field {

            val returnVal = lib.Field_coord_x();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun coordY(): Field {

            val returnVal = lib.Field_coord_y();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun coordZ(): Field {

            val returnVal = lib.Field_coord_z();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        /** Convert any existing Minecraft schematic into a smooth, composable
        *3D signed distance field via exact 3D Euclidean Distance Transform.
        */
        fun fromSchematic(schematic: Schematic, pad: UInt): Result<Field> {

            val returnVal = lib.Field_from_schematic(schematic.handle, FFIUint32(pad));
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        /** Convert binary glTF (`.glb`) bytes into a composable 3D signed
        *distance field fitted to `max_dim` blocks along its longest axis.
        */
        fun fromGlbBytes(bytes: UByteArray, maxDim: UInt, pad: UInt): Result<Field> {
            val bytesSliceMemory = PrimitiveArrayTools.borrow(bytes)

            val returnVal = lib.Field_from_glb_bytes(bytesSliceMemory.slice, FFIUint32(maxDim), FFIUint32(pad));
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Field(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                bytesSliceMemory.close()
            }
        }
        @JvmStatic

        /** Convert a Wavefront `.obj` string into a composable 3D signed
        *distance field fitted to `max_dim` blocks along its longest axis.
        */
        fun fromObjStr(objText: String, maxDim: UInt, pad: UInt): Result<Field> {
            val objTextSliceMemory = PrimitiveArrayTools.borrowUtf8(objText)

            val returnVal = lib.Field_from_obj_str(objTextSliceMemory.slice, FFIUint32(maxDim), FFIUint32(pad));
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Field(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                objTextSliceMemory.close()
            }
        }
        @JvmStatic

        fun sphere(radius: Float): Result<Field> {

            val returnVal = lib.Field_sphere(radius);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun boxShape(halfX: Float, halfY: Float, halfZ: Float, rounding: Float): Result<Field> {

            val returnVal = lib.Field_box_shape(halfX, halfY, halfZ, rounding);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun ellipsoid(radiusX: Float, radiusY: Float, radiusZ: Float): Result<Field> {

            val returnVal = lib.Field_ellipsoid(radiusX, radiusY, radiusZ);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun torus(majorRadius: Float, minorRadius: Float): Result<Field> {

            val returnVal = lib.Field_torus(majorRadius, minorRadius);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun cappedTorus(majorRadius: Float, minorRadius: Float, capAngleDegrees: Float): Result<Field> {

            val returnVal = lib.Field_capped_torus(majorRadius, minorRadius, capAngleDegrees);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun link(majorRadius: Float, minorRadius: Float, halfLength: Float): Result<Field> {

            val returnVal = lib.Field_link(majorRadius, minorRadius, halfLength);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun capsule(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, radius: Float): Result<Field> {

            val returnVal = lib.Field_capsule(ax, ay, az, bx, by, bz, radius);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun roundCone(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, r1: Float, r2: Float): Result<Field> {

            val returnVal = lib.Field_round_cone(ax, ay, az, bx, by, bz, r1, r2);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun solidAngle(radius: Float, angleDegrees: Float): Result<Field> {

            val returnVal = lib.Field_solid_angle(radius, angleDegrees);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun cutSphere(radius: Float, height: Float): Result<Field> {

            val returnVal = lib.Field_cut_sphere(radius, height);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun cutHollowSphere(radius: Float, height: Float, thickness: Float): Result<Field> {

            val returnVal = lib.Field_cut_hollow_sphere(radius, height, thickness);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun cappedCylinder(radius: Float, halfHeight: Float): Result<Field> {

            val returnVal = lib.Field_capped_cylinder(radius, halfHeight);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun infiniteCylinder(radius: Float): Result<Field> {

            val returnVal = lib.Field_infinite_cylinder(radius);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun cappedCone(halfHeight: Float, bottomRadius: Float, topRadius: Float): Result<Field> {

            val returnVal = lib.Field_capped_cone(halfHeight, bottomRadius, topRadius);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun plane(normalX: Float, normalY: Float, normalZ: Float, offset: Float): Result<Field> {

            val returnVal = lib.Field_plane(normalX, normalY, normalZ, offset);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun octahedron(size: Float): Result<Field> {

            val returnVal = lib.Field_octahedron(size);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun hexPrism(radius: Float, halfHeight: Float): Result<Field> {

            val returnVal = lib.Field_hex_prism(radius, halfHeight);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun superPrism(halfX: Float, halfY: Float, halfZ: Float, exponent: Float): Result<Field> {

            val returnVal = lib.Field_super_prism(halfX, halfY, halfZ, exponent);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun boxFrame(halfX: Float, halfY: Float, halfZ: Float, thickness: Float): Result<Field> {

            val returnVal = lib.Field_box_frame(halfX, halfY, halfZ, thickness);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun infiniteCone(angleDegrees: Float): Result<Field> {

            val returnVal = lib.Field_infinite_cone(angleDegrees);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun squarePyramid(halfBase: Float, height: Float): Result<Field> {

            val returnVal = lib.Field_square_pyramid(halfBase, height);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        /** Heightmap terrain surface: solid where `y <= base_y + height(x, 0, z)`.
        */
        fun heightmap(baseY: Float, height: Field): Result<Field> {

            val returnVal = lib.Field_heightmap(baseY, height.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun valueNoise2(frequency: Float, seed: Int): Result<Field> {

            val returnVal = lib.Field_value_noise2(frequency, seed);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun valueNoise3(frequency: Float, seed: Int): Result<Field> {

            val returnVal = lib.Field_value_noise3(frequency, seed);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun perlin2(frequency: Float, seed: Int): Result<Field> {

            val returnVal = lib.Field_perlin2(frequency, seed);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun perlin3(frequency: Float, seed: Int): Result<Field> {

            val returnVal = lib.Field_perlin3(frequency, seed);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun fbm2(basis: FieldNoiseBasis, frequency: Float, seed: Int, octaves: UInt, lacunarity: Float, gain: Float, ridged: Boolean): Result<Field> {

            val returnVal = lib.Field_fbm2(basis.toNative(), frequency, seed, FFIUint32(octaves), lacunarity, gain, ridged);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun fbm3(basis: FieldNoiseBasis, frequency: Float, seed: Int, octaves: UInt, lacunarity: Float, gain: Float, ridged: Boolean): Result<Field> {

            val returnVal = lib.Field_fbm3(basis.toNative(), frequency, seed, FFIUint32(octaves), lacunarity, gain, ridged);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun valueFbm3(frequency: Float, seed: Int, octaves: UInt): Result<Field> {

            val returnVal = lib.Field_value_fbm3(frequency, seed, FFIUint32(octaves));
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun perlinFbm3(frequency: Float, seed: Int, octaves: UInt): Result<Field> {

            val returnVal = lib.Field_perlin_fbm3(frequency, seed, FFIUint32(octaves));
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun ridgedFbm3(frequency: Float, seed: Int, octaves: UInt): Result<Field> {

            val returnVal = lib.Field_ridged_fbm3(frequency, seed, FFIUint32(octaves));
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun perlinFbm2(frequency: Float, seed: Int, octaves: UInt): Result<Field> {

            val returnVal = lib.Field_perlin_fbm2(frequency, seed, FFIUint32(octaves));
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun cells(frequency: Float, seed: Int, jitter: Float, mode: FieldCellMode, threshold: Float): Result<Field> {

            val returnVal = lib.Field_cells(frequency, seed, jitter, mode.toNative(), threshold);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun select(cond: Field, thenField: Field, elseField: Field): Field {

            val returnVal = lib.Field_select(cond.handle, thenField.handle, elseField.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        /** Parse a sandboxed infix expression (supports `x, y, z, p`, math,
        *noise, primitives, `let`, `if/else`, and bounded `for` loops).
        */
        fun expr(source: String): Result<Field> {
            val sourceSliceMemory = PrimitiveArrayTools.borrowUtf8(source)

            val returnVal = lib.Field_expr(sourceSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Field(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sourceSliceMemory.close()
            }
        }
        @JvmStatic

        fun fromJson(json: String): Result<Field> {
            val jsonSliceMemory = PrimitiveArrayTools.borrowUtf8(json)

            val returnVal = lib.Field_from_json(jsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Field(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                jsonSliceMemory.close()
            }
        }
    }

    fun neg(): Field {

        val returnVal = lib.Field_neg(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun abs(): Field {

        val returnVal = lib.Field_abs(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun sqrt(): Field {

        val returnVal = lib.Field_sqrt(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun sin(): Field {

        val returnVal = lib.Field_sin(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun cos(): Field {

        val returnVal = lib.Field_cos(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun saturate(): Field {

        val returnVal = lib.Field_saturate(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun add(other: Field): Field {

        val returnVal = lib.Field_add(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun sub(other: Field): Field {

        val returnVal = lib.Field_sub(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun mul(other: Field): Field {

        val returnVal = lib.Field_mul(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun div(other: Field): Field {

        val returnVal = lib.Field_div(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun minWith(other: Field): Field {

        val returnVal = lib.Field_min_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun maxWith(other: Field): Field {

        val returnVal = lib.Field_max_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun addConst(value: Float): Result<Field> {

        val returnVal = lib.Field_add_const(handle, value);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun mulConst(value: Float): Result<Field> {

        val returnVal = lib.Field_mul_const(handle, value);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun clamp(lo: Float, hi: Float): Result<Field> {

        val returnVal = lib.Field_clamp(handle, lo, hi);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun smoothstep(edge0: Float, edge1: Float): Result<Field> {

        val returnVal = lib.Field_smoothstep(handle, edge0, edge1);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun step(edge: Float): Result<Field> {

        val returnVal = lib.Field_step(handle, edge);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun remap(inLo: Float, inHi: Float, outLo: Float, outHi: Float, clamp: Boolean): Result<Field> {

        val returnVal = lib.Field_remap(handle, inLo, inHi, outLo, outHi, clamp);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun lerpWith(other: Field, t: Field): Field {

        val returnVal = lib.Field_lerp_with(handle, other.handle, t.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun unionWith(other: Field): Field {

        val returnVal = lib.Field_union_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun intersectionWith(other: Field): Field {

        val returnVal = lib.Field_intersection_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun subtract(other: Field): Field {

        val returnVal = lib.Field_subtract(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun xorWith(other: Field): Field {

        val returnVal = lib.Field_xor_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun smoothUnion(other: Field, radius: Float): Result<Field> {

        val returnVal = lib.Field_smooth_union(handle, other.handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun smoothSubtract(other: Field, radius: Float): Result<Field> {

        val returnVal = lib.Field_smooth_subtract(handle, other.handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun smoothIntersection(other: Field, radius: Float): Result<Field> {

        val returnVal = lib.Field_smooth_intersection(handle, other.handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun chamferUnion(other: Field, radius: Float): Result<Field> {

        val returnVal = lib.Field_chamfer_union(handle, other.handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun chamferSubtract(other: Field, radius: Float): Result<Field> {

        val returnVal = lib.Field_chamfer_subtract(handle, other.handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun chamferIntersection(other: Field, radius: Float): Result<Field> {

        val returnVal = lib.Field_chamfer_intersection(handle, other.handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun stepsUnion(other: Field, radius: Float, steps: Float): Result<Field> {

        val returnVal = lib.Field_steps_union(handle, other.handle, radius, steps);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun rounded(radius: Float): Result<Field> {

        val returnVal = lib.Field_rounded(handle, radius);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun shell(thickness: Float): Result<Field> {

        val returnVal = lib.Field_shell(handle, thickness);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun elongate(halfX: Float, halfY: Float, halfZ: Float): Result<Field> {

        val returnVal = lib.Field_elongate(handle, halfX, halfY, halfZ);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun displace(offset: Field, amplitude: Float): Result<Field> {

        val returnVal = lib.Field_displace(handle, offset.handle, amplitude);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun displaceNoise(amplitude: Float, frequency: Float, seed: Int, octaves: UInt): Result<Field> {

        val returnVal = lib.Field_displace_noise(handle, amplitude, frequency, seed, FFIUint32(octaves));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun translate(x: Float, y: Float, z: Float): Result<Field> {

        val returnVal = lib.Field_translate(handle, x, y, z);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun rotate(xDegrees: Float, yDegrees: Float, zDegrees: Float): Result<Field> {

        val returnVal = lib.Field_rotate(handle, xDegrees, yDegrees, zDegrees);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun scale(factor: Float): Result<Field> {

        val returnVal = lib.Field_scale(handle, factor);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun scaleXyz(sx: Float, sy: Float, sz: Float): Result<Field> {

        val returnVal = lib.Field_scale_xyz(handle, sx, sy, sz);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun mirror(axis: FieldAxis): Field {

        val returnVal = lib.Field_mirror(handle, axis.toNative());
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Field(handle, selfEdges, true)
        return returnOpaque
    }

    fun mirrorPlane(nx: Float, ny: Float, nz: Float, offset: Float): Result<Field> {

        val returnVal = lib.Field_mirror_plane(handle, nx, ny, nz, offset);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun repeatInfinite(spacingX: Float, spacingY: Float, spacingZ: Float): Result<Field> {

        val returnVal = lib.Field_repeat_infinite(handle, spacingX, spacingY, spacingZ);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun repeatCounted(spacingX: Float, spacingY: Float, spacingZ: Float, countX: UInt, countY: UInt, countZ: UInt): Result<Field> {

        val returnVal = lib.Field_repeat_counted(handle, spacingX, spacingY, spacingZ, FFIUint32(countX), FFIUint32(countY), FFIUint32(countZ));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun repeatPolar(count: UInt): Result<Field> {

        val returnVal = lib.Field_repeat_polar(handle, FFIUint32(count));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun repeatPoints(offsets: FloatArray): Result<Field> {
        val offsetsSliceMemory = PrimitiveArrayTools.borrow(offsets)

        val returnVal = lib.Field_repeat_points(handle, offsetsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            offsetsSliceMemory.close()
        }
    }

    fun twist(amount: Float): Result<Field> {

        val returnVal = lib.Field_twist(handle, amount);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun bend(amount: Float): Result<Field> {

        val returnVal = lib.Field_bend(handle, amount);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun warp(warpX: Field, warpY: Field, warpZ: Field, amplitude: Float): Result<Field> {

        val returnVal = lib.Field_warp(handle, warpX.handle, warpY.handle, warpZ.handle, amplitude);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun warpNoise(amplitude: Float, frequency: Float, seed: Int, octaves: UInt): Result<Field> {

        val returnVal = lib.Field_warp_noise(handle, amplitude, frequency, seed, FFIUint32(octaves));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun tag(materialId: UShort): Result<Field> {

        val returnVal = lib.Field_tag(handle, FFIUint16(materialId));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun bounded(minX: Float, minY: Float, minZ: Float, maxX: Float, maxY: Float, maxZ: Float): Result<Field> {

        val returnVal = lib.Field_bounded(handle, minX, minY, minZ, maxX, maxY, maxZ);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Field(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun evalAt(x: Float, y: Float, z: Float): Float {

        val returnVal = lib.Field_eval_at(handle, x, y, z);
        return (returnVal)
    }

    fun sampleTagged(x: Float, y: Float, z: Float): FieldSample {

        val returnVal = lib.Field_sample_tagged(handle, x, y, z);
        val returnStruct = FieldSample.fromNative(returnVal)
        return returnStruct
    }

    fun normal(x: Float, y: Float, z: Float, epsilon: Float): Result<FieldNormal> {

        val returnVal = lib.Field_normal(handle, x, y, z, epsilon);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = FieldNormal.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun bounds(): Result<FieldBounds> {

        val returnVal = lib.Field_bounds(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = FieldBounds.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun outputRange(): Result<FieldRange> {

        val returnVal = lib.Field_output_range(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = FieldRange.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun distanceKind(): FieldDistanceKind {

        val returnVal = lib.Field_distance_kind(handle);
        return (FieldDistanceKind.fromNative(returnVal))
    }

    fun toShape(): Result<Shape> {

        val returnVal = lib.Field_to_shape(handle);
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

        val returnVal = lib.Field_to_shape_bounded(handle, minX, minY, minZ, maxX, maxY, maxZ);
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
        val returnVal = lib.Field_to_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
