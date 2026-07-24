package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SdfCellModeLib: Library {
}
/** Cellular/Worley field output.
*/
enum class SdfCellMode {
    F1,
    F2,
    F2MinusF1,
    CellValue;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<SdfCellModeLib> = SdfCellModeLib::class.java
        internal val lib: SdfCellModeLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): SdfCellMode {
            return SdfCellMode.entries[native]
        }

        fun default(): SdfCellMode {
            return F1
        }
    }
}
