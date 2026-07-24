package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SdfNormalLib: Library {
}

internal class SdfNormalNative: Structure(), Structure.ByValue {
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




internal class OptionSdfNormalNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: SdfNormalNative = SdfNormalNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): SdfNormalNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: SdfNormalNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: SdfNormalNative): OptionSdfNormalNative {
            return OptionSdfNormalNative(value, 1)
        }

        internal fun none(): OptionSdfNormalNative {
            return OptionSdfNormalNative(SdfNormalNative(), 0)
        }
    }

}

/** Unit surface normal estimated from the SDF gradient.
*/
class SdfNormal (var x: Float, var y: Float, var z: Float) {
    companion object {

        internal val libClass: Class<SdfNormalLib> = SdfNormalLib::class.java
        internal val lib: SdfNormalLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(SdfNormalNative::class.java).toLong()

        internal fun fromNative(nativeStruct: SdfNormalNative): SdfNormal {
            val x: Float = nativeStruct.x
            val y: Float = nativeStruct.y
            val z: Float = nativeStruct.z

            return SdfNormal(x, y, z)
        }

    }
}
