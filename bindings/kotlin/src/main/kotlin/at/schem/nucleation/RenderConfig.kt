package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface RenderConfigLib: Library {
    fun RenderConfig_destroy(handle: Pointer)
    fun RenderConfig_create(width: FFIUint32, height: FFIUint32): Pointer
    fun RenderConfig_set_yaw(handle: Pointer, yaw: Float): Unit
    fun RenderConfig_set_pitch(handle: Pointer, pitch: Float): Unit
    fun RenderConfig_set_zoom(handle: Pointer, zoom: Float): Unit
    fun RenderConfig_set_fov(handle: Pointer, fov: Float): Unit
    fun RenderConfig_set_background(handle: Pointer, r: Float, g: Float, b: Float, a: Float): Unit
    fun RenderConfig_clear_background(handle: Pointer): Unit
    fun RenderConfig_set_orthographic(handle: Pointer, orthographic: Boolean): Unit
    fun RenderConfig_set_isometric(handle: Pointer): Unit
}
/** Camera / output configuration for rendering.
*/
class RenderConfig internal constructor (
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

    private class RenderConfigCleaner(val handle: Pointer, val lib: RenderConfigLib) : Runnable {
        override fun run() {
            lib.RenderConfig_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, RenderConfig.RenderConfigCleaner(handle, RenderConfig.lib));
    }

    companion object {
        internal val libClass: Class<RenderConfigLib> = RenderConfigLib::class.java
        internal val lib: RenderConfigLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(width: UInt, height: UInt): RenderConfig {
            
            val returnVal = lib.RenderConfig_create(FFIUint32(width), FFIUint32(height));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = RenderConfig(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    fun setYaw(yaw: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_yaw(handle, yaw);
        
    }
    
    fun setPitch(pitch: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_pitch(handle, pitch);
        
    }
    
    fun setZoom(zoom: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_zoom(handle, zoom);
        
    }
    
    fun setFov(fov: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_fov(handle, fov);
        
    }
    
    /** Set a solid RGBA clear color (linear 0.0–1.0). Alpha < 1.0 yields a
    *transparent PNG. Ignored when HDRI is enabled.
    */
    fun setBackground(r: Float, g: Float, b: Float, a: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_background(handle, r, g, b, a);
        
    }
    
    /** Clear the custom background — revert to default sky / HDRI.
    */
    fun clearBackground(): Unit {
        
        val returnVal = lib.RenderConfig_clear_background(handle);
        
    }
    
    /** Enable (`true`) or disable orthographic projection.
    */
    fun setOrthographic(orthographic: Boolean): Unit {
        
        val returnVal = lib.RenderConfig_set_orthographic(handle, orthographic);
        
    }
    
    /** Configure a true isometric view: orthographic at yaw 45° /
    *pitch ≈35.264° (preserves the current width/height).
    */
    fun setIsometric(): Unit {
        
        val returnVal = lib.RenderConfig_set_isometric(handle);
        
    }

}