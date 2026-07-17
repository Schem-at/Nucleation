package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface BuildingToolLib: Library {
    fun BuildingTool_destroy(handle: Pointer)
    fun BuildingTool_fill(schematic: Pointer, shape: Pointer, brush: Pointer): Unit
    fun BuildingTool_rstack(schematic: Pointer, shape: Pointer, brush: Pointer, count: FFISizet, offsetX: Int, offsetY: Int, offsetZ: Int): Unit
    fun BuildingTool_fill_only_air(schematic: Pointer, shape: Pointer, brush: Pointer): Unit
    fun BuildingTool_fill_replacing(schematic: Pointer, shape: Pointer, brush: Pointer, targetsJson: Slice): ResultUnitInt
}
/** Namespace for the fill operations that combine a schematic, a shape and a
*brush (the old `buildingtool_*` free functions).
*/
class BuildingTool internal constructor (
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

    private class BuildingToolCleaner(val handle: Pointer, val lib: BuildingToolLib) : Runnable {
        override fun run() {
            lib.BuildingTool_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, BuildingTool.BuildingToolCleaner(handle, BuildingTool.lib));
    }

    companion object {
        internal val libClass: Class<BuildingToolLib> = BuildingToolLib::class.java
        internal val lib: BuildingToolLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Fill `shape` into `schematic` using `brush`.
        */
        fun fill(schematic: Schematic, shape: Shape, brush: Brush): Unit {
            
            val returnVal = lib.BuildingTool_fill(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, shape.handle, brush.handle);
            
        }
        @JvmStatic
        
        /** Fill `count` copies of `shape`, each offset by `offset * i`.
        */
        fun rstack(schematic: Schematic, shape: Shape, brush: Brush, count: ULong, offsetX: Int, offsetY: Int, offsetZ: Int): Unit {
            
            val returnVal = lib.BuildingTool_rstack(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, shape.handle, brush.handle, FFISizet(count), offsetX, offsetY, offsetZ);
            
        }
        @JvmStatic
        
        /** Masked fill that preserves everything already placed: `brush` is
        *only written where `schematic` currently has air (or nothing at
        *all), so existing structures inside `shape` survive untouched.
        */
        fun fillOnlyAir(schematic: Schematic, shape: Shape, brush: Brush): Unit {
            
            val returnVal = lib.BuildingTool_fill_only_air(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, shape.handle, brush.handle);
            
        }
        @JvmStatic
        
        /** Masked fill that only overwrites the listed blocks: `targets_json`
        *is a JSON array of block ids (e.g. `["minecraft:stone"]`, state
        *properties ignored) and every cell of `shape` whose current block
        *id is in the list is replaced by `brush` — everything else,
        *including air, is left alone.
        */
        fun fillReplacing(schematic: Schematic, shape: Shape, brush: Brush, targetsJson: String): Result<Unit> {
            val targetsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(targetsJson)
            
            val returnVal = lib.BuildingTool_fill_replacing(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, shape.handle, brush.handle, targetsJsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                targetsJsonSliceMemory.close()
            }
        }
    }

}