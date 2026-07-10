package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface NucleationErrorLib: Library {
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
    NotFound;

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
}
class NucleationErrorError internal constructor(internal val value: NucleationError): Exception("Rust error result for NucleationError") {
    override fun toString(): String {
        return "NucleationError error with value " + value
    }

    fun getValue(): NucleationError = value
}
