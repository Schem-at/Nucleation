package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldProgramValueTypeLib: Library {
}
/** The type of a value on a [FieldProgramBuilder]'s stack or in a slot.
*/
enum class FieldProgramValueType {
    Scalar,
    Vec3,
    Bool;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<FieldProgramValueTypeLib> = FieldProgramValueTypeLib::class.java
        internal val lib: FieldProgramValueTypeLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): FieldProgramValueType {
            return FieldProgramValueType.entries[native]
        }

        fun default(): FieldProgramValueType {
            return Scalar
        }
    }
}
