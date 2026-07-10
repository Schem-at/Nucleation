package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MeshJobLib: Library {
    fun MeshJob_destroy(handle: Pointer)
    fun MeshJob_start(schematic: Pointer, pack: Pointer, config: Pointer, chunkSize: Int, atlas: Pointer): Pointer
    fun MeshJob_poll_progress(handle: Pointer): MeshProgressNative
    fun MeshJob_take_result(handle: Pointer): ResultPointerInt
}
/** A chunk-meshing job running on a background thread. Replaces the old
*`schematic_mesh_chunks_with_atlas_progress` C callback: poll it from a
*timer loop with [MeshJob::poll_progress], then call
*[MeshJob::take_result] once (it blocks until the job finishes and
*consumes the job — a second call returns `AlreadyConsumed`).
*/
class MeshJob internal constructor (
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

    private class MeshJobCleaner(val handle: Pointer, val lib: MeshJobLib) : Runnable {
        override fun run() {
            lib.MeshJob_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, MeshJob.MeshJobCleaner(handle, MeshJob.lib));
    }

    companion object {
        internal val libClass: Class<MeshJobLib> = MeshJobLib::class.java
        internal val lib: MeshJobLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Kick off chunk meshing with a shared atlas on a background thread
        *and return immediately. Takes the same parameters as
        *[ChunkMeshResult::create_with_atlas].
        */
        fun start(schematic: Schematic, pack: ResourcePack, config: MeshConfig, chunkSize: Int, atlas: TextureAtlas): MeshJob {
            
            val returnVal = lib.MeshJob_start(schematic.handle, pack.handle, config.handle, chunkSize, atlas.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = MeshJob(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** Cheap, non-blocking progress snapshot. Call from a timer/poll loop.
    */
    fun pollProgress(): MeshProgress {
        
        val returnVal = lib.MeshJob_poll_progress(handle);
        val returnStruct = MeshProgress.fromNative(returnVal)
        return returnStruct
    }
    
    /** Block until the job finishes (if it hasn't already) and return the
    *result. Consumes the job: a second call returns `AlreadyConsumed`.
    */
    fun takeResult(): Result<ChunkMeshResult> {
        
        val returnVal = lib.MeshJob_take_result(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal 
            val returnOpaque = ChunkMeshResult(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}