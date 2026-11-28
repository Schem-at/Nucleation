package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MeshBytesLib: Library {
    fun MeshBytes_destroy(handle: Pointer)
    fun MeshBytes_len(handle: Pointer): FFIUint32
    fun MeshBytes_copy_into(handle: Pointer, offset: FFIUint32, out: Slice): FFIUint32
}
/** Owned binary byte buffer returned by zero-Base64 binary export methods
*(e.g. [MeshResult::to_glb_bytes]). Query [Self::len] and copy straight
*into a caller-allocated byte buffer / `Uint8Array` via [Self::copy_into].
*/
class MeshBytes internal constructor (
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

    private class MeshBytesCleaner(val handle: Pointer, val lib: MeshBytesLib) : Runnable {
        override fun run() {
            lib.MeshBytes_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, MeshBytes.MeshBytesCleaner(handle, MeshBytes.lib));
    }

    companion object {
        internal val libClass: Class<MeshBytesLib> = MeshBytesLib::class.java
        internal val lib: MeshBytesLib = Native.load("nucleation", libClass)
    }

    /** Byte length of the encoded payload.
    */
    fun len(): UInt {

        val returnVal = lib.MeshBytes_len(handle);
        return (returnVal.toUInt())
    }

    /** Copy up to `out.len()` bytes starting at `offset` into `out`.
    *Returns the number of bytes written.
    */
    fun copyInto(offset: UInt, out: UByteArray): UInt {
        val outSliceMemory = PrimitiveArrayTools.borrow(out)

        val returnVal = lib.MeshBytes_copy_into(handle, FFIUint32(offset), outSliceMemory.slice);
        try {
            return (returnVal.toUInt())
        } finally {
            outSliceMemory.close()
        }
    }

}
