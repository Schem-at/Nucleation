package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SdfAxisLib: Library {
}
/** Axis used by mirror operations.
*/
enum class SdfAxis {
    X,
    Y,
    Z;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<SdfAxisLib> = SdfAxisLib::class.java
        internal val lib: SdfAxisLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): SdfAxis {
            return SdfAxis.entries[native]
        }

        fun default(): SdfAxis {
            return X
        }
    }
}
