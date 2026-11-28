package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldAxisLib: Library {
}
/** Axis selector for mirror operations.
*/
enum class FieldAxis {
    X,
    Y,
    Z;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<FieldAxisLib> = FieldAxisLib::class.java
        internal val lib: FieldAxisLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): FieldAxis {
            return FieldAxis.entries[native]
        }

        fun default(): FieldAxis {
            return X
        }
    }
}
