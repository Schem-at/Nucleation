package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldSampleLib: Library {
}

internal class FieldSampleNative: Structure(), Structure.ByValue {
    @JvmField
    internal var distance: Float = 0.0F;
    @JvmField
    internal var materialId: FFIUint16 = FFIUint16();
    @JvmField
    internal var blendMaterialId: FFIUint16 = FFIUint16();
    @JvmField
    internal var blendWeight: Float = 0.0F;

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("distance", "materialId", "blendMaterialId", "blendWeight")
    }
}




internal class OptionFieldSampleNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: FieldSampleNative = FieldSampleNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): FieldSampleNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: FieldSampleNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: FieldSampleNative): OptionFieldSampleNative {
            return OptionFieldSampleNative(value, 1)
        }

        internal fun none(): OptionFieldSampleNative {
            return OptionFieldSampleNative(FieldSampleNative(), 0)
        }
    }

}

/** Distance plus propagated CSG material tag and smooth seam blend weight.
*/
class FieldSample (var distance: Float, var materialId: UShort, var blendMaterialId: UShort, var blendWeight: Float) {
    companion object {

        internal val libClass: Class<FieldSampleLib> = FieldSampleLib::class.java
        internal val lib: FieldSampleLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(FieldSampleNative::class.java).toLong()

        internal fun fromNative(nativeStruct: FieldSampleNative): FieldSample {
            val distance: Float = nativeStruct.distance
            val materialId: UShort = nativeStruct.materialId.toUShort()
            val blendMaterialId: UShort = nativeStruct.blendMaterialId.toUShort()
            val blendWeight: Float = nativeStruct.blendWeight

            return FieldSample(distance, materialId, blendMaterialId, blendWeight)
        }

    }
}
