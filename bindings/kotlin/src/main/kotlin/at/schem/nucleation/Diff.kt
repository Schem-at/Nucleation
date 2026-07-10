package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface DiffLib: Library {
    fun Diff_destroy(handle: Pointer)
    fun Diff_compute(a: Pointer, b: Pointer, preset: Slice): ResultPointerInt
    fun Diff_compute_with_opts(a: Pointer, b: Pointer, preset: Slice, costAdd: Int, costDelete: Int, costChange: Int, costSwap: Int, symmetry: Slice): ResultPointerInt
    fun Diff_from_json(json: Slice): ResultPointerInt
    fun Diff_distance(handle: Pointer): FFIUint64
    fun Diff_support(handle: Pointer): Float
    fun Diff_to_json(handle: Pointer, write: Pointer): Unit
    fun Diff_summary_json(handle: Pointer, write: Pointer): Unit
    fun Diff_added(handle: Pointer): Pointer
    fun Diff_removed(handle: Pointer): Pointer
    fun Diff_changed(handle: Pointer): Pointer
    fun Diff_swapped(handle: Pointer): Pointer
    fun Diff_markers(handle: Pointer): Pointer
    fun Diff_to_overlay_glb_b64(handle: Pointer, afterGlb: Slice, write: Pointer): ResultUnitInt
}
/** A computed diff between two schematics.
*/
class Diff internal constructor (
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

    private class DiffCleaner(val handle: Pointer, val lib: DiffLib) : Runnable {
        override fun run() {
            lib.Diff_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Diff.DiffCleaner(handle, Diff.lib));
    }

    companion object {
        internal val libClass: Class<DiffLib> = DiffLib::class.java
        internal val lib: DiffLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Diff two schematics with the given preset (default cost model).
        *Errors with `InvalidArgument` on an unknown preset.
        */
        fun compute(a: Schematic, b: Schematic, preset: String): Result<Diff> {
            val presetSliceMemory = PrimitiveArrayTools.borrowUtf8(preset)
            
            val returnVal = lib.Diff_compute(a.handle, b.handle, presetSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Diff(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                presetSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Diff two schematics with optional cost/symmetry overrides. Negative
        *cost ints mean "unset" (use the preset default); an empty `symmetry`
        *string means "unset". Errors with `InvalidArgument` on an unknown
        *preset or symmetry name.
        */
        fun computeWithOpts(a: Schematic, b: Schematic, preset: String, costAdd: Int, costDelete: Int, costChange: Int, costSwap: Int, symmetry: String): Result<Diff> {
            val presetSliceMemory = PrimitiveArrayTools.borrowUtf8(preset)
            val symmetrySliceMemory = PrimitiveArrayTools.borrowUtf8(symmetry)
            
            val returnVal = lib.Diff_compute_with_opts(a.handle, b.handle, presetSliceMemory.slice, costAdd, costDelete, costChange, costSwap, symmetrySliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Diff(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                presetSliceMemory.close()
                symmetrySliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Reconstruct a diff from its JSON representation.
        */
        fun fromJson(json: String): Result<Diff> {
            val jsonSliceMemory = PrimitiveArrayTools.borrowUtf8(json)
            
            val returnVal = lib.Diff_from_json(jsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Diff(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                jsonSliceMemory.close()
            }
        }
    }
    
    /** The edit distance of the diff.
    */
    fun distance(): ULong {
        
        val returnVal = lib.Diff_distance(handle);
        return (returnVal.toULong())
    }
    
    /** The support (alignment confidence) of the diff.
    */
    fun support(): Float {
        
        val returnVal = lib.Diff_support(handle);
        return (returnVal)
    }
    
    /** Serialize the diff to its full JSON representation.
    */
    fun toJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Diff_to_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Serialize the diff to its compact summary JSON.
    */
    fun summaryJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Diff_summary_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** A new schematic containing only the blocks added in this diff.
    */
    fun added(): Schematic {
        
        val returnVal = lib.Diff_added(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Schematic(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** A new schematic containing only the blocks removed in this diff.
    */
    fun removed(): Schematic {
        
        val returnVal = lib.Diff_removed(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Schematic(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** A new schematic containing only the blocks changed in this diff.
    */
    fun changed(): Schematic {
        
        val returnVal = lib.Diff_changed(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Schematic(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** A new schematic containing only the blocks swapped in this diff.
    */
    fun swapped(): Schematic {
        
        val returnVal = lib.Diff_swapped(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Schematic(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** A new schematic with marker blocks summarizing this diff.
    */
    fun markers(): Schematic {
        
        val returnVal = lib.Diff_markers(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Schematic(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** Render a diff overlay on top of an "after" GLB buffer, returning the
    *new GLB as base64 (PORTING rule 6). Requires the `meshing` feature;
    *errors with `Mesh` when compiled without it.
    */
    fun toOverlayGlbB64(afterGlb: UByteArray): Result<String> {
        val afterGlbSliceMemory = PrimitiveArrayTools.borrow(afterGlb)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Diff_to_overlay_glb_b64(handle, afterGlbSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            afterGlbSliceMemory.close()
        }
    }

}