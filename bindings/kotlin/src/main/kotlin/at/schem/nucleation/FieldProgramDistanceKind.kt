package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldProgramDistanceKindLib: Library {
}
/** What kind of distance a field program's output represents.
*/
enum class FieldProgramDistanceKind {
    Exact,
    LowerBound,
    Estimate,
    Implicit;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<FieldProgramDistanceKindLib> = FieldProgramDistanceKindLib::class.java
        internal val lib: FieldProgramDistanceKindLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): FieldProgramDistanceKind {
            return FieldProgramDistanceKind.entries[native]
        }

        fun default(): FieldProgramDistanceKind {
            return Exact
        }
    }
}
