package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface WorldChunkViewLib: Library {
    fun WorldChunkView_destroy(handle: Pointer)
    fun WorldChunkView_create(cx: Int, cz: Int): Pointer
    fun WorldChunkView_cx(handle: Pointer): Int
    fun WorldChunkView_cz(handle: Pointer): Int
    fun WorldChunkView_to_schematic(handle: Pointer): Pointer
    fun WorldChunkView_set_block(handle: Pointer, x: Int, y: Int, z: Int, blockName: Slice): ResultUnitInt
    fun WorldChunkView_set_biome(handle: Pointer, biomeName: Slice): ResultUnitInt
    fun WorldChunkView_biome_palette_json(handle: Pointer, write: Pointer): ResultUnitInt
}
/** A single decoded chunk (or a from-scratch chunk under construction).
*/
class WorldChunkView internal constructor (
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

    private class WorldChunkViewCleaner(val handle: Pointer, val lib: WorldChunkViewLib) : Runnable {
        override fun run() {
            lib.WorldChunkView_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, WorldChunkView.WorldChunkViewCleaner(handle, WorldChunkView.lib));
    }

    companion object {
        internal val libClass: Class<WorldChunkViewLib> = WorldChunkViewLib::class.java
        internal val lib: WorldChunkViewLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Create an empty chunk view at the given chunk coordinates — the
        *starting point for generating worlds from scratch. Sections are
        *created on demand by `set_block`. Serialized with
        *`status = "minecraft:full"` (Minecraft will not regenerate over it)
        *and the default data version.
        */
        fun create(cx: Int, cz: Int): WorldChunkView {
            
            val returnVal = lib.WorldChunkView_create(cx, cz);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = WorldChunkView(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** The chunk X coordinate (in chunk units).
    */
    fun cx(): Int {
        
        val returnVal = lib.WorldChunkView_cx(handle);
        return (returnVal)
    }
    
    /** The chunk Z coordinate (in chunk units).
    */
    fun cz(): Int {
        
        val returnVal = lib.WorldChunkView_cz(handle);
        return (returnVal)
    }
    
    /** Convert the chunk view to a standalone schematic.
    */
    fun toSchematic(): Schematic {
        
        val returnVal = lib.WorldChunkView_to_schematic(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Schematic(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** Set a block at absolute world coordinates inside this chunk view.
    *`block_name` must be a valid Minecraft block identifier (e.g.
    *`minecraft:stone`). Errors with `InvalidArgument` if (x, z) is outside
    *this chunk's column.
    */
    fun setBlock(x: Int, y: Int, z: Int, blockName: String): Result<Unit> {
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        
        val returnVal = lib.WorldChunkView_set_block(handle, x, y, z, blockNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockNameSliceMemory.close()
        }
    }
    
    /** Overwrite the biome of every currently-present section of the chunk
    *view with `biome_name` (e.g. `minecraft:desert`). Sections are created
    *lazily by `set_block`, so call this AFTER placing blocks.
    */
    fun setBiome(biomeName: String): Result<Unit> {
        val biomeNameSliceMemory = PrimitiveArrayTools.borrowUtf8(biomeName)
        
        val returnVal = lib.WorldChunkView_set_biome(handle, biomeNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            biomeNameSliceMemory.close()
        }
    }
    
    /** Deduped union of all sections' biome palette entries, in order of
    *first appearance, written as a JSON array string (`[]` if no section
    *carries biome data).
    */
    fun biomePaletteJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.WorldChunkView_biome_palette_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}