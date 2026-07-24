package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldProgramBinaryOpLib: Library {
}
/** Binary field-program operations (see `crate::sdf::BinaryOp`). `Add`
*and `Sub` accept either two scalars or two vec3s.
*/
enum class FieldProgramBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    Pow,
    Atan2,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Dot,
    Cross,
    Scale;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<FieldProgramBinaryOpLib> = FieldProgramBinaryOpLib::class.java
        internal val lib: FieldProgramBinaryOpLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): FieldProgramBinaryOp {
            return FieldProgramBinaryOp.entries[native]
        }

        fun default(): FieldProgramBinaryOp {
            return Add
        }
    }
}
