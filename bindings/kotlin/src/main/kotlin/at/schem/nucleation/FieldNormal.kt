package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldNormalLib: Library {
}

internal class FieldNormalNative: Structure(), Structure.ByValue {
    @JvmField
    internal var x: Float = 0.0F;
    @JvmField
    internal var y: Float = 0.0F;
    @JvmField
    internal var z: Float = 0.0F;

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("x", "y", "z")
    }
}




internal class OptionFieldNormalNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: FieldNormalNative = FieldNormalNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): FieldNormalNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: FieldNormalNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: FieldNormalNative): OptionFieldNormalNative {
            return OptionFieldNormalNative(value, 1)
        }

        internal fun none(): OptionFieldNormalNative {
            return OptionFieldNormalNative(FieldNormalNative(), 0)
        }
    }

}

/** Unit surface normal estimated from the field gradient.
*/
class FieldNormal (var x: Float, var y: Float, var z: Float) {
    companion object {

        internal val libClass: Class<FieldNormalLib> = FieldNormalLib::class.java
        internal val lib: FieldNormalLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(FieldNormalNative::class.java).toLong()

        internal fun fromNative(nativeStruct: FieldNormalNative): FieldNormal {
            val x: Float = nativeStruct.x
            val y: Float = nativeStruct.y
            val z: Float = nativeStruct.z

            return FieldNormal(x, y, z)
        }

    }
}
