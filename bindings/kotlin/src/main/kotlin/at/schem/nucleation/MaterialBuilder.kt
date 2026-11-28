package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MaterialBuilderLib: Library {
    fun MaterialBuilder_destroy(handle: Pointer)
    fun MaterialBuilder_create_layered(): Pointer
    fun MaterialBuilder_create_tag_matcher(blendSeams: Boolean, seed: Int): Pointer
    fun MaterialBuilder_add_layer(handle: Pointer, condition: Pointer, material: Pointer): ResultUnitInt
    fun MaterialBuilder_add_tag_case(handle: Pointer, materialId: FFIUint16, material: Pointer): ResultUnitInt
    fun MaterialBuilder_set_default(handle: Pointer, material: Pointer): Unit
    fun MaterialBuilder_build(handle: Pointer): Pointer
}
/** Builder for layered conditional materials and CSG tag-dispatched materials.
*/
class MaterialBuilder internal constructor (
    internal val handle: Pointer,
    // These ensure that anything that is borrowed is kept alive and not cleaned
    // up by the garbage collector.
    internal val selfEdges: List<Any>,
    internal var owned: Boolean,
)  {

    init {
        if (this.owned) {
            this.registerCleaner()
        }
    }

    private class MaterialBuilderCleaner(val handle: Pointer, val lib: MaterialBuilderLib) : Runnable {
        override fun run() {
            lib.MaterialBuilder_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, MaterialBuilder.MaterialBuilderCleaner(handle, MaterialBuilder.lib));
    }

    companion object {
        internal val libClass: Class<MaterialBuilderLib> = MaterialBuilderLib::class.java
        internal val lib: MaterialBuilderLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun createLayered(): MaterialBuilder {

            val returnVal = lib.MaterialBuilder_create_layered();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = MaterialBuilder(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun createTagMatcher(blendSeams: Boolean, seed: Int): MaterialBuilder {

            val returnVal = lib.MaterialBuilder_create_tag_matcher(blendSeams, seed);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = MaterialBuilder(handle, selfEdges, true)
            return returnOpaque
        }
    }

    fun addLayer(condition: MaterialCondition, material: Material): Result<Unit> {

        val returnVal = lib.MaterialBuilder_add_layer(handle, condition.handle, material.handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun addTagCase(materialId: UShort, material: Material): Result<Unit> {

        val returnVal = lib.MaterialBuilder_add_tag_case(handle, FFIUint16(materialId), material.handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun setDefault(material: Material): Unit {

        val returnVal = lib.MaterialBuilder_set_default(handle, material.handle);

    }

    fun build(): Material {

        val returnVal = lib.MaterialBuilder_build(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = Material(handle, selfEdges, true)
        return returnOpaque
    }

}
