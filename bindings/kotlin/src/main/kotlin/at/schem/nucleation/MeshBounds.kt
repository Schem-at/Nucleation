package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MeshBoundsLib: Library {
}

internal class MeshBoundsNative: Structure(), Structure.ByValue {
    @JvmField
    internal var minX: Float = 0.0F;
    @JvmField
    internal var minY: Float = 0.0F;
    @JvmField
    internal var minZ: Float = 0.0F;
    @JvmField
    internal var maxX: Float = 0.0F;
    @JvmField
    internal var maxY: Float = 0.0F;
    @JvmField
    internal var maxZ: Float = 0.0F;

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("minX", "minY", "minZ", "maxX", "maxY", "maxZ")
    }
}




internal class OptionMeshBoundsNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: MeshBoundsNative = MeshBoundsNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): MeshBoundsNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: MeshBoundsNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: MeshBoundsNative): OptionMeshBoundsNative {
            return OptionMeshBoundsNative(value, 1)
        }

        internal fun none(): OptionMeshBoundsNative {
            return OptionMeshBoundsNative(MeshBoundsNative(), 0)
        }
    }

}

/** Axis-aligned bounding box of a mesh result.
*/
class MeshBounds (var minX: Float, var minY: Float, var minZ: Float, var maxX: Float, var maxY: Float, var maxZ: Float) {
    companion object {

        internal val libClass: Class<MeshBoundsLib> = MeshBoundsLib::class.java
        internal val lib: MeshBoundsLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(MeshBoundsNative::class.java).toLong()

        internal fun fromNative(nativeStruct: MeshBoundsNative): MeshBounds {
            val minX: Float = nativeStruct.minX
            val minY: Float = nativeStruct.minY
            val minZ: Float = nativeStruct.minZ
            val maxX: Float = nativeStruct.maxX
            val maxY: Float = nativeStruct.maxY
            val maxZ: Float = nativeStruct.maxZ

            return MeshBounds(minX, minY, minZ, maxX, maxY, maxZ)
        }

    }
    internal fun toNative(): MeshBoundsNative {
        var native = MeshBoundsNative()
        native.minX = this.minX
        native.minY = this.minY
        native.minZ = this.minZ
        native.maxX = this.maxX
        native.maxY = this.maxY
        native.maxZ = this.maxZ
        return native
    }

}