package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface VideoConfigLib: Library {
    fun VideoConfig_destroy(handle: Pointer)
    fun VideoConfig_prores_4444(fps: Double): ResultPointerInt
    fun VideoConfig_h264(fps: Double): ResultPointerInt
    fun VideoConfig_set_ffmpeg_path(handle: Pointer, path: Slice): ResultUnitInt
}
/** Native FFmpeg video output preset.
*/
class VideoConfig internal constructor (
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

    private class VideoConfigCleaner(val handle: Pointer, val lib: VideoConfigLib) : Runnable {
        override fun run() {
            lib.VideoConfig_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, VideoConfig.VideoConfigCleaner(handle, VideoConfig.lib));
    }

    companion object {
        internal val libClass: Class<VideoConfigLib> = VideoConfigLib::class.java
        internal val lib: VideoConfigLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Alpha-preserving ProRes 4444 in a MOV container.
        */
        fun prores4444(fps: Double): Result<VideoConfig> {
            
            val returnVal = lib.VideoConfig_prores_4444(fps);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = VideoConfig(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** H.264 in an MP4 or MOV container. H.264 does not preserve alpha.
        */
        fun h264(fps: Double): Result<VideoConfig> {
            
            val returnVal = lib.VideoConfig_h264(fps);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = VideoConfig(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }
    
    /** Override the FFmpeg executable. The default resolves `ffmpeg` on PATH.
    */
    fun setFfmpegPath(path: String): Result<Unit> {
        val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
        
        val returnVal = lib.VideoConfig_set_ffmpeg_path(handle, pathSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            pathSliceMemory.close()
        }
    }

}