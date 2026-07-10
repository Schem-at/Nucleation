package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface RendererLib: Library {
    fun Renderer_destroy(handle: Pointer)
    fun Renderer_render_pixels_b64(schematic: Pointer, packZip: Slice, config: Pointer, write: Pointer): ResultUnitInt
    fun Renderer_render_png_b64(schematic: Pointer, packZip: Slice, config: Pointer, write: Pointer): ResultUnitInt
    fun Renderer_render_to_file(schematic: Pointer, packZip: Slice, config: Pointer, path: Slice): ResultUnitInt
}
/** Namespace type for the render entry points (PORTING rule 12).
*/
class Renderer internal constructor (
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

    private class RendererCleaner(val handle: Pointer, val lib: RendererLib) : Runnable {
        override fun run() {
            lib.Renderer_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Renderer.RendererCleaner(handle, Renderer.lib));
    }

    companion object {
        internal val libClass: Class<RendererLib> = RendererLib::class.java
        internal val lib: RendererLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Render a schematic to raw RGBA pixel bytes, written as base64
        *(PORTING rule 6). `pack_zip` is a resource-pack zip in memory.
        */
        fun renderPixelsB64(schematic: Schematic, packZip: UByteArray, config: RenderConfig): Result<String> {
            val packZipSliceMemory = PrimitiveArrayTools.borrow(packZip)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Renderer_render_pixels_b64(schematic.handle, packZipSliceMemory.slice, config.handle, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                packZipSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Render a schematic to PNG bytes, written as base64 (PORTING rule 6).
        *`pack_zip` is a resource-pack zip in memory.
        */
        fun renderPngB64(schematic: Schematic, packZip: UByteArray, config: RenderConfig): Result<String> {
            val packZipSliceMemory = PrimitiveArrayTools.borrow(packZip)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Renderer_render_png_b64(schematic.handle, packZipSliceMemory.slice, config.handle, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                packZipSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Render a schematic to a PNG file at `path`.
        */
        fun renderToFile(schematic: Schematic, packZip: UByteArray, config: RenderConfig, path: String): Result<Unit> {
            val packZipSliceMemory = PrimitiveArrayTools.borrow(packZip)
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
            
            val returnVal = lib.Renderer_render_to_file(schematic.handle, packZipSliceMemory.slice, config.handle, pathSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                packZipSliceMemory.close()
                pathSliceMemory.close()
            }
        }
    }

}