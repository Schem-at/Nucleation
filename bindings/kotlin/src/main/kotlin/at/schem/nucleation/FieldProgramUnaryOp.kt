package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldProgramUnaryOpLib: Library {
}
/** Unary field-program operations (see `crate::sdf::UnaryOp`).
*/
enum class FieldProgramUnaryOp {
    Neg,
    Abs,
    Sqrt,
    Log,
    Sin,
    Cos,
    Acos,
    VecX,
    VecY,
    VecZ,
    Length,
    Normalize;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<FieldProgramUnaryOpLib> = FieldProgramUnaryOpLib::class.java
        internal val lib: FieldProgramUnaryOpLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): FieldProgramUnaryOp {
            return FieldProgramUnaryOp.entries[native]
        }

        fun default(): FieldProgramUnaryOp {
            return Neg
        }
    }
}
