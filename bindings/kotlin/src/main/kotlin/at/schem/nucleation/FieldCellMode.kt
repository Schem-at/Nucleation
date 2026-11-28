package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldCellModeLib: Library {
}
/** Cellular / Worley noise output mode.
*/
enum class FieldCellMode {
    F1,
    F2,
    F2MinusF1,
    CellValue;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<FieldCellModeLib> = FieldCellModeLib::class.java
        internal val lib: FieldCellModeLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): FieldCellMode {
            return FieldCellMode.entries[native]
        }

        fun default(): FieldCellMode {
            return F1
        }
    }
}
