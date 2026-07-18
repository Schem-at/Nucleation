package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface PaletteLib: Library {
    fun Palette_destroy(handle: Pointer)
    fun Palette_all(): Pointer
    fun Palette_solid(): Pointer
    fun Palette_structural(): Pointer
    fun Palette_decorative(): Pointer
    fun Palette_concrete(): Pointer
    fun Palette_wool(): Pointer
    fun Palette_terracotta(): Pointer
    fun Palette_grayscale(): Pointer
    fun Palette_wood(): Pointer
    fun Palette_sorted_by_lightness(handle: Pointer): Pointer
    fun Palette_ramp_ids_json(handle: Pointer, r1: FFIUint8, g1: FFIUint8, b1: FFIUint8, r2: FFIUint8, g2: FFIUint8, b2: FFIUint8, steps: FFIUint32, write: Pointer): ResultUnitInt
    fun Palette_gradient_ids_json(handle: Pointer, r1: FFIUint8, g1: FFIUint8, b1: FFIUint8, r2: FFIUint8, g2: FFIUint8, b2: FFIUint8, steps: FFIUint32, write: Pointer): ResultUnitInt
    fun Palette_from_block_ids(idsJson: Slice): ResultPointerInt
    fun Palette_len(handle: Pointer): FFISizet
    fun Palette_block_ids_json(handle: Pointer, write: Pointer): Unit
    fun Palette_closest_block(handle: Pointer, r: FFIUint8, g: FFIUint8, b: FFIUint8, write: Pointer): ResultUnitInt
}
/** A set of colored blocks that color/gradient brushes snap their computed
*colors to (nearest neighbor in Oklab space). Wraps an Arc'd
*[crate::building::BlockPalette]; sharing one palette across many
*brushes is cheap.
*/
class Palette internal constructor (
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

    private class PaletteCleaner(val handle: Pointer, val lib: PaletteLib) : Runnable {
        override fun run() {
            lib.Palette_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Palette.PaletteCleaner(handle, Palette.lib));
    }

    companion object {
        internal val libClass: Class<PaletteLib> = PaletteLib::class.java
        internal val lib: PaletteLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Every block blockpedia knows a color for (the default palette
        *brushes use when none is set).
        */
        fun all(): Palette {
            
            val returnVal = lib.Palette_all();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Only solid blocks: no transparency, gravity, tile entities, or
        *support requirements.
        */
        fun solid(): Palette {
            
            val returnVal = lib.Palette_solid();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Conservative structural set (full building blocks).
        */
        fun structural(): Palette {
            
            val returnVal = lib.Palette_structural();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Decorative set: allows stairs/slabs but no tile entities.
        */
        fun decorative(): Palette {
            
            val returnVal = lib.Palette_decorative();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** The 16 concrete colors (excludes concrete powder).
        */
        fun concrete(): Palette {
            
            val returnVal = lib.Palette_concrete();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** The 16 wool colors.
        */
        fun wool(): Palette {
            
            val returnVal = lib.Palette_wool();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Terracotta colors (excludes glazed variants).
        */
        fun terracotta(): Palette {
            
            val returnVal = lib.Palette_terracotta();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Genuinely gray blocks: opaque full cubes whose measured color
        *is near-neutral (low Oklab chroma) — judged from color data,
        *not names.
        */
        fun grayscale(): Palette {
            
            val returnVal = lib.Palette_grayscale();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** The planks family — a natural light→dark wood ramp.
        */
        fun wood(): Palette {
            
            val returnVal = lib.Palette_wood();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Custom palette from a JSON array of block ids, e.g.
        *`["minecraft:stone", "minecraft:oak_planks"]`. Ids blockpedia has
        *no color for are silently skipped — check `len` afterwards.
        */
        fun fromBlockIds(idsJson: String): Result<Palette> {
            val idsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(idsJson)
            
            val returnVal = lib.Palette_from_block_ids(idsJsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Palette(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                idsJsonSliceMemory.close()
            }
        }
    }
    
    /** A copy of this palette ordered by perceptual lightness (Oklab L,
    *dark → light). Combined with `block_ids_json`, gives a
    *ready-to-index ramp: `ids[i]` for intensity `i / (len - 1)`.
    */
    fun sortedByLightness(): Palette {
        
        val returnVal = lib.Palette_sorted_by_lightness(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Palette(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** JSON array of exactly `steps` DISTINCT block ids forming the
    *smoothest ramp this palette can make from (`r1`,`g1`,`b1`) to
    *(`r2`,`g2`,`b2`): targets are evenly spaced along the Oklab line
    *and blocks are chosen by a minimum-cost monotonic matching, so
    *off-hue blocks are penalized and no block repeats. Errors with
    *`InvalidArgument` when the palette has fewer than `steps` blocks,
    *`steps` is 0, or start equals end.
    */
    fun rampIdsJson(r1: UByte, g1: UByte, b1: UByte, r2: UByte, g2: UByte, b2: UByte, steps: UInt): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Palette_ramp_ids_json(handle, FFIUint8(r1), FFIUint8(g1), FFIUint8(b1), FFIUint8(r2), FFIUint8(g2), FFIUint8(b2), FFIUint32(steps), write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** JSON array of exactly `steps` block ids sampling the color
    *gradient from (`r1`,`g1`,`b1`) to (`r2`,`g2`,`b2`) in Oklab
    *space, each step snapped to this palette's closest block. Built
    *for value→block lookups (heatmaps, fractals): index the returned
    *list by `intensity * (steps - 1)`. Entries may repeat on coarse
    *palettes; errors with `NotFound` on an empty palette.
    */
    fun gradientIdsJson(r1: UByte, g1: UByte, b1: UByte, r2: UByte, g2: UByte, b2: UByte, steps: UInt): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Palette_gradient_ids_json(handle, FFIUint8(r1), FFIUint8(g1), FFIUint8(b1), FFIUint8(r2), FFIUint8(g2), FFIUint8(b2), FFIUint32(steps), write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Number of blocks in the palette.
    */
    fun len(): ULong {
        
        val returnVal = lib.Palette_len(handle);
        return (returnVal.toULong())
    }
    
    /** The palette's block ids as a JSON array string.
    */
    fun blockIdsJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Palette_block_ids_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** The palette block whose color is closest (Oklab distance) to the
    *given RGB. Errors with `NotFound` on an empty palette.
    */
    fun closestBlock(r: UByte, g: UByte, b: UByte): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Palette_closest_block(handle, FFIUint8(r), FFIUint8(g), FFIUint8(b), write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}