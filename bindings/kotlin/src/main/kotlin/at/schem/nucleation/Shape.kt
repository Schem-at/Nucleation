package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ShapeLib: Library {
    fun Shape_destroy(handle: Pointer)
    fun Shape_tube_along(curve: Pointer, radius: Double): ResultPointerInt
    fun Shape_sphere(cx: Float, cy: Float, cz: Float, radius: Float): Pointer
    fun Shape_cuboid(minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Pointer
    fun Shape_polygon_prism(polygonJson: Slice, yMin: Int, yMax: Int): ResultPointerInt
    fun Shape_ellipsoid(cx: Int, cy: Int, cz: Int, rx: Float, ry: Float, rz: Float): Pointer
    fun Shape_cylinder(bx: Float, by: Float, bz: Float, ax: Float, ay: Float, az: Float, radius: Float, height: Float): Pointer
    fun Shape_cylinder_between(x1: Float, y1: Float, z1: Float, x2: Float, y2: Float, z2: Float, radius: Float): Pointer
    fun Shape_cone(ax: Float, ay: Float, az: Float, dx: Float, dy: Float, dz: Float, radius: Float, height: Float): Pointer
    fun Shape_torus(cx: Float, cy: Float, cz: Float, majorR: Float, minorR: Float, ax: Float, ay: Float, az: Float): Pointer
    fun Shape_pyramid(bx: Float, by: Float, bz: Float, halfW: Float, halfD: Float, height: Float, ax: Float, ay: Float, az: Float): Pointer
    fun Shape_disk(cx: Float, cy: Float, cz: Float, radius: Float, nx: Float, ny: Float, nz: Float, thickness: Float): Pointer
    fun Shape_plane(ox: Float, oy: Float, oz: Float, ux: Float, uy: Float, uz: Float, vx: Float, vy: Float, vz: Float, uExt: Float, vExt: Float, thickness: Float): Pointer
    fun Shape_triangle(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, cx: Float, cy: Float, cz: Float, thickness: Float): Pointer
    fun Shape_line(x1: Float, y1: Float, z1: Float, x2: Float, y2: Float, z2: Float, thickness: Float): Pointer
    fun Shape_bezier(controlPoints: Slice, thickness: Float, resolution: FFIUint32): ResultPointerInt
    fun Shape_sdf(sdfJson: Slice): ResultPointerInt
    fun Shape_sdf_bounded(sdfJson: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun Shape_hollow(handle: Pointer, thickness: FFIUint32): Pointer
    fun Shape_union_with(handle: Pointer, other: Pointer): Pointer
    fun Shape_intersection_with(handle: Pointer, other: Pointer): Pointer
    fun Shape_difference_with(handle: Pointer, other: Pointer): Pointer
}
/** A solid region of blocks: primitives (sphere, cuboid, …) and boolean
*combinations thereof. Wraps `ShapeEnum`.
*/
class Shape internal constructor (
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

    private class ShapeCleaner(val handle: Pointer, val lib: ShapeLib) : Runnable {
        override fun run() {
            lib.Shape_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Shape.ShapeCleaner(handle, Shape.lib));
    }

    companion object {
        internal val libClass: Class<ShapeLib> = ShapeLib::class.java
        internal val lib: ShapeLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Thicken a sampled 3D curve into a parametric tube with the given radius.
        *Inputs outside voxel-coordinate or bounded-work limits are rejected.
        */
        fun tubeAlong(curve: Curve3D, radius: Double): Result<Shape> {
            
            val returnVal = lib.Shape_tube_along(curve.handle, radius);
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
        @JvmStatic
        
        /** Sphere centered at (`cx`, `cy`, `cz`) (truncated to block coordinates,
        *matching the old `shape_sphere`).
        */
        fun sphere(cx: Float, cy: Float, cz: Float, radius: Float): Shape {
            
            val returnVal = lib.Shape_sphere(cx, cy, cz, radius);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Axis-aligned box spanning the two corners (inclusive).
        */
        fun cuboid(minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Shape {
            
            val returnVal = lib.Shape_cuboid(minX, minY, minZ, maxX, maxY, maxZ);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** A closed 2D polygon extruded between two Y levels (inclusive). The
        *footprint is `polygon_json`, a JSON array of `[x, z]` world-coordinate
        *pairs in order (winding does not matter; the ring closes implicitly);
        *any simple polygon works, concave ones included. This is the shape
        *behind extruded building footprints, lake outlines, and plot fills.
        *Errors with `Parse` on invalid JSON and `InvalidArgument` on non-UTF-8
        *or fewer than three vertices.
        */
        fun polygonPrism(polygonJson: String, yMin: Int, yMax: Int): Result<Shape> {
            val polygonJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(polygonJson)
            
            val returnVal = lib.Shape_polygon_prism(polygonJsonSliceMemory.slice, yMin, yMax);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Shape(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                polygonJsonSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Ellipsoid centered at (`cx`, `cy`, `cz`) with per-axis radii.
        */
        fun ellipsoid(cx: Int, cy: Int, cz: Int, rx: Float, ry: Float, rz: Float): Shape {
            
            val returnVal = lib.Shape_ellipsoid(cx, cy, cz, rx, ry, rz);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Cylinder from base point along an axis vector.
        */
        fun cylinder(bx: Float, by: Float, bz: Float, ax: Float, ay: Float, az: Float, radius: Float, height: Float): Shape {
            
            val returnVal = lib.Shape_cylinder(bx, by, bz, ax, ay, az, radius, height);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Cylinder spanning the segment between two points.
        */
        fun cylinderBetween(x1: Float, y1: Float, z1: Float, x2: Float, y2: Float, z2: Float, radius: Float): Shape {
            
            val returnVal = lib.Shape_cylinder_between(x1, y1, z1, x2, y2, z2, radius);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Cone with apex at (`ax`, `ay`, `az`) opening along direction (`dx`, `dy`, `dz`).
        */
        fun cone(ax: Float, ay: Float, az: Float, dx: Float, dy: Float, dz: Float, radius: Float, height: Float): Shape {
            
            val returnVal = lib.Shape_cone(ax, ay, az, dx, dy, dz, radius, height);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Torus centered at (`cx`, `cy`, `cz`) with the given major/minor radii and
        *axis (`ax`, `ay`, `az`).
        */
        fun torus(cx: Float, cy: Float, cz: Float, majorR: Float, minorR: Float, ax: Float, ay: Float, az: Float): Shape {
            
            val returnVal = lib.Shape_torus(cx, cy, cz, majorR, minorR, ax, ay, az);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Rectangular pyramid: base center, half-extents, height, up-axis.
        */
        fun pyramid(bx: Float, by: Float, bz: Float, halfW: Float, halfD: Float, height: Float, ax: Float, ay: Float, az: Float): Shape {
            
            val returnVal = lib.Shape_pyramid(bx, by, bz, halfW, halfD, height, ax, ay, az);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Flat disk: center, radius, plane normal, thickness.
        */
        fun disk(cx: Float, cy: Float, cz: Float, radius: Float, nx: Float, ny: Float, nz: Float, thickness: Float): Shape {
            
            val returnVal = lib.Shape_disk(cx, cy, cz, radius, nx, ny, nz, thickness);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Finite plane patch: origin, two spanning vectors `u`/`v`, extents along
        *each, thickness.
        */
        fun plane(ox: Float, oy: Float, oz: Float, ux: Float, uy: Float, uz: Float, vx: Float, vy: Float, vz: Float, uExt: Float, vExt: Float, thickness: Float): Shape {
            
            val returnVal = lib.Shape_plane(ox, oy, oz, ux, uy, uz, vx, vy, vz, uExt, vExt, thickness);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Filled triangle between three vertices, thickened by `thickness`.
        */
        fun triangle(ax: Float, ay: Float, az: Float, bx: Float, by: Float, bz: Float, cx: Float, cy: Float, cz: Float, thickness: Float): Shape {
            
            val returnVal = lib.Shape_triangle(ax, ay, az, bx, by, bz, cx, cy, cz, thickness);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Line segment between two points, thickened by `thickness`.
        */
        fun line(x1: Float, y1: Float, z1: Float, x2: Float, y2: Float, z2: Float, thickness: Float): Shape {
            
            val returnVal = lib.Shape_line(x1, y1, z1, x2, y2, z2, thickness);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Shape(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Bézier curve through `control_points` (flat `[x0, y0, z0, x1, y1, z1, …]`,
        *so the length must be a non-zero multiple of 3), thickened by `thickness`
        *and sampled at `resolution` steps.
        */
        fun bezier(controlPoints: FloatArray, thickness: Float, resolution: UInt): Result<Shape> {
            val controlPointsSliceMemory = PrimitiveArrayTools.borrow(controlPoints)
            
            val returnVal = lib.Shape_bezier(controlPointsSliceMemory.slice, thickness, FFIUint32(resolution));
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Shape(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                controlPointsSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Any SDF tree as a Shape: the same JSON the terrain sampler takes
        *(primitives, smooth booleans, noise — see the SDF guide) becomes
        *fillable with every brush, combinable with other shapes, and
        *usable in masked fills. Normals come from the field gradient, so
        *the shaded brush shades smooth blends smoothly. Errors with
        *`Parse` on invalid JSON and `InvalidArgument` for unbounded trees
        *(use `sdf_bounded`).
        */
        fun sdf(sdfJson: String): Result<Shape> {
            val sdfJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(sdfJson)
            
            val returnVal = lib.Shape_sdf(sdfJsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Shape(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sdfJsonSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Like `sdf`, with explicit sampling bounds (inclusive block
        *coordinates) for unbounded trees such as planes.
        */
        fun sdfBounded(sdfJson: String, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Shape> {
            val sdfJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(sdfJson)
            
            val returnVal = lib.Shape_sdf_bounded(sdfJsonSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Shape(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sdfJsonSliceMemory.close()
            }
        }
    }
    
    /** Hollowed-out copy of this shape with the given wall thickness (clones the
    *input, like the old `shape_hollow`).
    */
    fun hollow(thickness: UInt): Shape {
        
        val returnVal = lib.Shape_hollow(handle, FFIUint32(thickness));
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Shape(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** Boolean union of this shape and `other` (clones both inputs).
    */
    fun unionWith(other: Shape): Shape {
        
        val returnVal = lib.Shape_union_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Shape(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** Boolean intersection of this shape and `other` (clones both inputs).
    */
    fun intersectionWith(other: Shape): Shape {
        
        val returnVal = lib.Shape_intersection_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Shape(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** Boolean difference: this shape minus `other` (clones both inputs).
    */
    fun differenceWith(other: Shape): Shape {
        
        val returnVal = lib.Shape_difference_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Shape(handle, selfEdges, true)
        return returnOpaque
    }

}