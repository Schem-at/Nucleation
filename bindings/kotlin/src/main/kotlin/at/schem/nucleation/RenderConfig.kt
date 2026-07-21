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
    fun RenderConfig_set_sphere_fit(handle: Pointer, sphereFit: Boolean): Unit
    fun RenderConfig_set_fov(handle: Pointer, fov: Float): Unit
    fun RenderConfig_set_background(handle: Pointer, r: Float, g: Float, b: Float, a: Float): Unit
    fun RenderConfig_clear_background(handle: Pointer): Unit
    fun RenderConfig_set_grid(handle: Pointer, halfExtent: Int, spacing: Int, planeY: Float, showAxes: Boolean, red: Float, green: Float, blue: Float, alpha: Float): Unit
    fun RenderConfig_set_fitted_grid(handle: Pointer, margin: Int, spacing: Int, planeY: Float, showAxes: Boolean, red: Float, green: Float, blue: Float, alpha: Float): Unit
    fun RenderConfig_clear_grid(handle: Pointer): Unit
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
        
        /** Create a config with the given output size in pixels. Camera starts
        *at the defaults: yaw 45°, pitch 30°, zoom 1.0, fov 45°, perspective
        *projection, default sky background.
        */
        fun create(width: UInt, height: UInt): RenderConfig {
            
            val returnVal = lib.RenderConfig_create(FFIUint32(width), FFIUint32(height));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = RenderConfig(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** Set the camera yaw (horizontal orbit angle) in degrees. Default: 45.
    */
    fun setYaw(yaw: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_yaw(handle, yaw);
        
    }
    
    /** Set the camera pitch (downward tilt) in degrees. Default: 30.
    */
    fun setPitch(pitch: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_pitch(handle, pitch);
        
    }
    
    /** Set the zoom factor applied to the auto-fitted framing
    *(1.0 = frame the whole model; 2.0 = twice as close; 0.5 = twice
    *as far). Default: 1.0.
    */
    fun setZoom(zoom: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_zoom(handle, zoom);
        
    }
    
    /** Fit the camera to the model's bounding sphere instead of its
    *yaw-dependent silhouette. The sphere is rotation invariant, so
    *orbiting cameras (turntables) keep a constant distance instead
    *of pulsing as the silhouette changes. Frames slightly looser
    *than the default fit. Default: false.
    */
    fun setSphereFit(sphereFit: Boolean): Unit {
        
        val returnVal = lib.RenderConfig_set_sphere_fit(handle, sphereFit);
        
    }
    
    /** Set the vertical field of view in degrees (perspective projection
    *only). Default: 45.
    */
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
    
    /** Configure a one-block world grid. Models are centred on integer
    *schematic coordinates, so grid lines are placed on half-integer
    *block boundaries automatically.
    */
    fun setGrid(halfExtent: Int, spacing: Int, planeY: Float, showAxes: Boolean, red: Float, green: Float, blue: Float, alpha: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_grid(handle, halfExtent, spacing, planeY, showAxes, red, green, blue, alpha);
        
    }
    
    /** Configure a compact grid fitted to half-integer block boundaries.
    */
    fun setFittedGrid(margin: Int, spacing: Int, planeY: Float, showAxes: Boolean, red: Float, green: Float, blue: Float, alpha: Float): Unit {
        
        val returnVal = lib.RenderConfig_set_fitted_grid(handle, margin, spacing, planeY, showAxes, red, green, blue, alpha);
        
    }
    
    fun clearGrid(): Unit {
        
        val returnVal = lib.RenderConfig_clear_grid(handle);
        
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