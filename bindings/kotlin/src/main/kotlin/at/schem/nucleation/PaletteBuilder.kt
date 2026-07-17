package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface PaletteBuilderLib: Library {
    fun PaletteBuilder_destroy(handle: Pointer)
    fun PaletteBuilder_create(): Pointer
    fun PaletteBuilder_exclude_falling(handle: Pointer): ResultUnitInt
    fun PaletteBuilder_exclude_tile_entities(handle: Pointer): ResultUnitInt
    fun PaletteBuilder_full_blocks_only(handle: Pointer): ResultUnitInt
    fun PaletteBuilder_exclude_needs_support(handle: Pointer): ResultUnitInt
    fun PaletteBuilder_exclude_transparent(handle: Pointer): ResultUnitInt
    fun PaletteBuilder_exclude_light_sources(handle: Pointer): ResultUnitInt
    fun PaletteBuilder_survival_only(handle: Pointer): ResultUnitInt
    fun PaletteBuilder_exclude_keyword(handle: Pointer, keyword: Slice): ResultUnitInt
    fun PaletteBuilder_include_keyword(handle: Pointer, keyword: Slice): ResultUnitInt
    fun PaletteBuilder_build(handle: Pointer): ResultPointerInt
}
/** Filter-driven palette construction (wraps
*[crate::building::PaletteBuilder], which fronts blockpedia's
*`BlockFilter`). Call flag methods, then `build` — the builder is
*consumed; further calls error with `AlreadyConsumed`.
*/
class PaletteBuilder internal constructor (
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

    private class PaletteBuilderCleaner(val handle: Pointer, val lib: PaletteBuilderLib) : Runnable {
        override fun run() {
            lib.PaletteBuilder_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, PaletteBuilder.PaletteBuilderCleaner(handle, PaletteBuilder.lib));
    }

    companion object {
        internal val libClass: Class<PaletteBuilderLib> = PaletteBuilderLib::class.java
        internal val lib: PaletteBuilderLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** A builder matching every colored block (no filters yet).
        */
        fun create(): PaletteBuilder {
            
            val returnVal = lib.PaletteBuilder_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = PaletteBuilder(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** Exclude gravity-affected blocks (sand, gravel, ...).
    */
    fun excludeFalling(): Result<Unit> {
        
        val returnVal = lib.PaletteBuilder_exclude_falling(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Exclude blocks with block entities (chests, furnaces, ...).
    */
    fun excludeTileEntities(): Result<Unit> {
        
        val returnVal = lib.PaletteBuilder_exclude_tile_entities(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Keep only full cube blocks (no stairs, slabs, fences, ...).
    */
    fun fullBlocksOnly(): Result<Unit> {
        
        val returnVal = lib.PaletteBuilder_full_blocks_only(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Exclude blocks that need supporting blocks (torches, rails, ...).
    */
    fun excludeNeedsSupport(): Result<Unit> {
        
        val returnVal = lib.PaletteBuilder_exclude_needs_support(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Exclude transparent/translucent blocks (glass, leaves, ...).
    */
    fun excludeTransparent(): Result<Unit> {
        
        val returnVal = lib.PaletteBuilder_exclude_transparent(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Exclude light-emitting blocks (glowstone, lanterns, ...).
    */
    fun excludeLightSources(): Result<Unit> {
        
        val returnVal = lib.PaletteBuilder_exclude_light_sources(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Keep only blocks obtainable in survival.
    */
    fun survivalOnly(): Result<Unit> {
        
        val returnVal = lib.PaletteBuilder_survival_only(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Exclude blocks whose id contains `keyword`.
    */
    fun excludeKeyword(keyword: String): Result<Unit> {
        val keywordSliceMemory = PrimitiveArrayTools.borrowUtf8(keyword)
        
        val returnVal = lib.PaletteBuilder_exclude_keyword(handle, keywordSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keywordSliceMemory.close()
        }
    }
    
    /** Keep only blocks whose id contains `keyword` (repeatable; matches
    *any of the included keywords).
    */
    fun includeKeyword(keyword: String): Result<Unit> {
        val keywordSliceMemory = PrimitiveArrayTools.borrowUtf8(keyword)
        
        val returnVal = lib.PaletteBuilder_include_keyword(handle, keywordSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keywordSliceMemory.close()
        }
    }
    
    /** Build the palette; consumes the builder.
    */
    fun build(): Result<Palette> {
        
        val returnVal = lib.PaletteBuilder_build(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal 
            val returnOpaque = Palette(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}