package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface TextureAtlasLib: Library {
    fun TextureAtlas_destroy(handle: Pointer)
    fun TextureAtlas_build_global(schematic: Pointer, pack: Pointer, config: Pointer): ResultPointerInt
    fun TextureAtlas_width(handle: Pointer): FFIUint32
    fun TextureAtlas_height(handle: Pointer): FFIUint32
    fun TextureAtlas_rgba_data_b64(handle: Pointer, write: Pointer): Unit
}
/** A packed texture atlas. Wraps [schematic_mesher::TextureAtlas].
*/
class TextureAtlas internal constructor (
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

    private class TextureAtlasCleaner(val handle: Pointer, val lib: TextureAtlasLib) : Runnable {
        override fun run() {
            lib.TextureAtlas_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, TextureAtlas.TextureAtlasCleaner(handle, TextureAtlas.lib));
    }

    companion object {
        internal val libClass: Class<TextureAtlasLib> = TextureAtlasLib::class.java
        internal val lib: TextureAtlasLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Build a single shared atlas from every unique block state in the
        *schematic (old ABI: `schematic_build_global_atlas`).
        */
        fun buildGlobal(schematic: Schematic, pack: ResourcePack, config: MeshConfig): Result<TextureAtlas> {
            
            val returnVal = lib.TextureAtlas_build_global(schematic.handle, pack.handle, config.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = TextureAtlas(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }
    
    fun width(): UInt {
        
        val returnVal = lib.TextureAtlas_width(handle);
        return (returnVal.toUInt())
    }
    
    fun height(): UInt {
        
        val returnVal = lib.TextureAtlas_height(handle);
        return (returnVal.toUInt())
    }
    
    /** Raw RGBA atlas pixels, base64-encoded.
    */
    fun rgbaDataB64(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TextureAtlas_rgba_data_b64(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }

}