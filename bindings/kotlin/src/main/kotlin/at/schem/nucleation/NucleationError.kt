package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface NucleationErrorLib: Library {
    fun NucleationError_detail(inner: Int, write: Pointer): Unit
}
/** Every fallible method in the bridge returns `Result<T, NucleationError>` —
*see `stencil/docs/nucleation-error.md` for how these variants were derived from
*the three error conventions the old hand-written `ffi` module mixed.
*/
enum class NucleationError {
    NullArgument,
    InvalidArgument,
    Parse,
    Serialize,
    Io,
    Lock,
    Store,
    Mesh,
    Render,
    Simulation,
    AlreadyConsumed,
    NotFound,
    Generation;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<NucleationErrorLib> = NucleationErrorLib::class.java
        internal val lib: NucleationErrorLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): NucleationError {
            return NucleationError.entries[native]
        }

        fun default(): NucleationError {
            return NullArgument
        }
    }

    /** Why the last failing bridge call on this thread failed, in words.
    *
    *The enum cannot carry a message across the FFI, so a caught error
    *is a bare variant — `InvalidArgument` — while the layer that
    *refused already knew it was "19.2M cells over the 8M cap". Modules
    *that know the story record it; this reads it back, so an exception
    *handler holding the error value can ask it for the words. Empty
    *when the last detail-carrying call succeeded.
    */
    fun detail(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.NucleationError_detail(this.toNative(), write);

        val returnString = DW.writeToString(write)
        return returnString
    }
}
class NucleationErrorError internal constructor(internal val value: NucleationError): Exception("Rust error result for NucleationError") {
    override fun toString(): String {
        return "NucleationError error with value " + value
    }

    fun getValue(): NucleationError = value
}
