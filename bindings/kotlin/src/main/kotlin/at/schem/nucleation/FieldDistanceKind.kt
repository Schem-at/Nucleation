package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldDistanceKindLib: Library {
}
/** Distance semantics of a [Field].
*/
enum class FieldDistanceKind {
    Exact,
    LowerBound,
    Estimate,
    Implicit;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<FieldDistanceKindLib> = FieldDistanceKindLib::class.java
        internal val lib: FieldDistanceKindLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): FieldDistanceKind {
            return FieldDistanceKind.entries[native]
        }

        fun default(): FieldDistanceKind {
            return Exact
        }
    }
}
