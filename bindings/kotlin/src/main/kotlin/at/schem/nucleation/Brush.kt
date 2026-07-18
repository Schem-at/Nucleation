package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface BrushLib: Library {
    fun Brush_destroy(handle: Pointer)
    fun Brush_solid(blockName: Slice): ResultPointerInt
    fun Brush_color(r: FFIUint8, g: FFIUint8, b: FFIUint8): Pointer
    fun Brush_linear_gradient(x1: Int, y1: Int, z1: Int, r1: FFIUint8, g1: FFIUint8, b1: FFIUint8, x2: Int, y2: Int, z2: Int, r2: FFIUint8, g2: FFIUint8, b2: FFIUint8, space: Int): Pointer
    fun Brush_shaded(r: FFIUint8, g: FFIUint8, b: FFIUint8, lx: Float, ly: Float, lz: Float): Pointer
    fun Brush_bilinear_gradient(ox: Int, oy: Int, oz: Int, ux: Int, uy: Int, uz: Int, vx: Int, vy: Int, vz: Int, r00: FFIUint8, g00: FFIUint8, b00: FFIUint8, r10: FFIUint8, g10: FFIUint8, b10: FFIUint8, r01: FFIUint8, g01: FFIUint8, b01: FFIUint8, r11: FFIUint8, g11: FFIUint8, b11: FFIUint8, space: Int): Pointer
    fun Brush_point_gradient(positions: Slice, colors: Slice, falloff: Float, space: Int): ResultPointerInt
    fun Brush_spotlight(px: Float, py: Float, pz: Float, dx: Float, dy: Float, dz: Float, coneAngleDeg: Float, r: FFIUint8, g: FFIUint8, b: FFIUint8): Pointer
    fun Brush_set_palette(handle: Pointer, palette: Pointer): Unit
    fun Brush_curve_gradient(stops: Slice, colors: Slice, space: Int): ResultPointerInt
}
/** Decides which block goes at each point of a filled shape. Wraps `BrushEnum`.
*/
class Brush internal constructor (
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

    private class BrushCleaner(val handle: Pointer, val lib: BrushLib) : Runnable {
        override fun run() {
            lib.Brush_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Brush.BrushCleaner(handle, Brush.lib));
    }

    companion object {
        internal val libClass: Class<BrushLib> = BrushLib::class.java
        internal val lib: BrushLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Every point becomes `block_name` (a block-state string).
        */
        fun solid(blockName: String): Result<Brush> {
            val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
            
            val returnVal = lib.Brush_solid(blockNameSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Brush(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                blockNameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Nearest-block-to-RGB-color brush.
        */
        fun color(r: UByte, g: UByte, b: UByte): Brush {
            
            val returnVal = lib.Brush_color(FFIUint8(r), FFIUint8(g), FFIUint8(b));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Brush(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Linear color gradient between two anchored points.
        */
        fun linearGradient(x1: Int, y1: Int, z1: Int, r1: UByte, g1: UByte, b1: UByte, x2: Int, y2: Int, z2: Int, r2: UByte, g2: UByte, b2: UByte, space: InterpolationSpace): Brush {
            
            val returnVal = lib.Brush_linear_gradient(x1, y1, z1, FFIUint8(r1), FFIUint8(g1), FFIUint8(b1), x2, y2, z2, FFIUint8(r2), FFIUint8(g2), FFIUint8(b2), space.toNative());
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Brush(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Base color shaded by surface normal against light direction
        *(`lx`, `ly`, `lz`).
        */
        fun shaded(r: UByte, g: UByte, b: UByte, lx: Float, ly: Float, lz: Float): Brush {
            
            val returnVal = lib.Brush_shaded(FFIUint8(r), FFIUint8(g), FFIUint8(b), lx, ly, lz);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Brush(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Bilinear gradient over the patch `origin + s*u + t*v` with corner colors
        *c00/c10/c01/c11.
        */
        fun bilinearGradient(ox: Int, oy: Int, oz: Int, ux: Int, uy: Int, uz: Int, vx: Int, vy: Int, vz: Int, r00: UByte, g00: UByte, b00: UByte, r10: UByte, g10: UByte, b10: UByte, r01: UByte, g01: UByte, b01: UByte, r11: UByte, g11: UByte, b11: UByte, space: InterpolationSpace): Brush {
            
            val returnVal = lib.Brush_bilinear_gradient(ox, oy, oz, ux, uy, uz, vx, vy, vz, FFIUint8(r00), FFIUint8(g00), FFIUint8(b00), FFIUint8(r10), FFIUint8(g10), FFIUint8(b10), FFIUint8(r01), FFIUint8(g01), FFIUint8(b01), FFIUint8(r11), FFIUint8(g11), FFIUint8(b11), space.toNative());
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Brush(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Inverse-distance-weighted gradient between colored anchor points.
        *`positions` is flat `[x0, y0, z0, x1, …]` and `colors` is flat
        *`[r0, g0, b0, r1, …]`; both must describe the same non-zero number of
        *points (`positions.len() == colors.len()`, a multiple of 3).
        */
        fun pointGradient(positions: IntArray, colors: UByteArray, falloff: Float, space: InterpolationSpace): Result<Brush> {
            val positionsSliceMemory = PrimitiveArrayTools.borrow(positions)
            val colorsSliceMemory = PrimitiveArrayTools.borrow(colors)
            
            val returnVal = lib.Brush_point_gradient(positionsSliceMemory.slice, colorsSliceMemory.slice, falloff, space.toNative());
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Brush(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                positionsSliceMemory.close()
                colorsSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Spotlight-lit base color (`r`, `g`, `b`): Lambert shading toward a
        *cone light at (`px`, `py`, `pz`) aimed along (`dx`, `dy`, `dz`).
        *Full intensity inside 0.7 × `cone_angle_deg`, smoothstep falloff
        *to zero at the cone edge; surfaces facing away or outside the cone
        *drop to a 4% ambient floor.
        */
        fun spotlight(px: Float, py: Float, pz: Float, dx: Float, dy: Float, dz: Float, coneAngleDeg: Float, r: UByte, g: UByte, b: UByte): Brush {
            
            val returnVal = lib.Brush_spotlight(px, py, pz, dx, dy, dz, coneAngleDeg, FFIUint8(r), FFIUint8(g), FFIUint8(b));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Brush(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Gradient along a parametric curve: `stops` holds the curve parameters in
        *`[0, 1]` and `colors` the matching flat RGB triples
        *(`colors.len() == stops.len() * 3`, `stops` non-empty).
        */
        fun curveGradient(stops: FloatArray, colors: UByteArray, space: InterpolationSpace): Result<Brush> {
            val stopsSliceMemory = PrimitiveArrayTools.borrow(stops)
            val colorsSliceMemory = PrimitiveArrayTools.borrow(colors)
            
            val returnVal = lib.Brush_curve_gradient(stopsSliceMemory.slice, colorsSliceMemory.slice, space.toNative());
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Brush(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                stopsSliceMemory.close()
                colorsSliceMemory.close()
            }
        }
    }
    
    /** Use `palette` for this brush's color→block snapping instead of the
    *default all-blocks palette. No-op for `solid` brushes, which place
    *a fixed block state. Set it before filling; the palette is shared,
    *not copied.
    */
    fun setPalette(palette: Palette): Unit {
        
        val returnVal = lib.Brush_set_palette(handle, palette.handle);
        
    }

}