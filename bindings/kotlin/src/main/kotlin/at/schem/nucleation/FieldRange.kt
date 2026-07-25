package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldRangeLib: Library {
}

internal class FieldRangeNative: Structure(), Structure.ByValue {
    @JvmField
    internal var min: Float = 0.0F;
    @JvmField
    internal var max: Float = 0.0F;

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("min", "max")
    }
}




internal class OptionFieldRangeNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: FieldRangeNative = FieldRangeNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): FieldRangeNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: FieldRangeNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: FieldRangeNative): OptionFieldRangeNative {
            return OptionFieldRangeNative(value, 1)
        }

        internal fun none(): OptionFieldRangeNative {
            return OptionFieldRangeNative(FieldRangeNative(), 0)
        }
    }

}

/** The closed interval a field's values are analytically proven to lie in.
*/
class FieldRange (var min: Float, var max: Float) {
    companion object {

        internal val libClass: Class<FieldRangeLib> = FieldRangeLib::class.java
        internal val lib: FieldRangeLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(FieldRangeNative::class.java).toLong()

        internal fun fromNative(nativeStruct: FieldRangeNative): FieldRange {
            val min: Float = nativeStruct.min
            val max: Float = nativeStruct.max

            return FieldRange(min, max)
        }

    }
}
