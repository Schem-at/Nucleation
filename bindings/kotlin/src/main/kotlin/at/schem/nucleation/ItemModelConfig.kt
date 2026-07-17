package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ItemModelConfigLib: Library {
    fun ItemModelConfig_destroy(handle: Pointer)
    fun ItemModelConfig_create(modelName: Slice): ResultPointerInt
    fun ItemModelConfig_set_namespace(handle: Pointer, namespace: Slice): ResultUnitInt
    fun ItemModelConfig_set_center(handle: Pointer, center: Boolean): Unit
    fun ItemModelConfig_set_texture_resolution(handle: Pointer, resolution: FFIUint32): Unit
    fun ItemModelConfig_set_item(handle: Pointer, item: Slice): ResultUnitInt
    fun ItemModelConfig_set_custom_model_data(handle: Pointer, cmd: Slice): ResultUnitInt
    fun ItemModelConfig_set_scale(handle: Pointer, scale: Float): Unit
    fun ItemModelConfig_set_scale_xyz(handle: Pointer, sx: Float, sy: Float, sz: Float): Unit
    fun ItemModelConfig_set_scale_auto(handle: Pointer): Unit
}
/** Item model generation configuration. Wraps [crate::meshing::ItemModelConfig].
*/
class ItemModelConfig internal constructor (
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

    private class ItemModelConfigCleaner(val handle: Pointer, val lib: ItemModelConfigLib) : Runnable {
        override fun run() {
            lib.ItemModelConfig_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, ItemModelConfig.ItemModelConfigCleaner(handle, ItemModelConfig.lib));
    }

    companion object {
        internal val libClass: Class<ItemModelConfigLib> = ItemModelConfigLib::class.java
        internal val lib: ItemModelConfigLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Create a config for a model named `model_name` (used in resource-pack
        *file paths). Other options start at their defaults: namespace
        *"nucleation", centered, 16px texture resolution, item "paper",
        *custom model data "1", auto scale.
        */
        fun create(modelName: String): Result<ItemModelConfig> {
            val modelNameSliceMemory = PrimitiveArrayTools.borrowUtf8(modelName)
            
            val returnVal = lib.ItemModelConfig_create(modelNameSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = ItemModelConfig(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                modelNameSliceMemory.close()
            }
        }
    }
    
    /** Set the resource-pack namespace (default: "nucleation").
    */
    fun setNamespace(namespace: String): Result<Unit> {
        val namespaceSliceMemory = PrimitiveArrayTools.borrowUtf8(namespace)
        
        val returnVal = lib.ItemModelConfig_set_namespace(handle, namespaceSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            namespaceSliceMemory.close()
        }
    }
    
    /** Center the schematic within the model bounds (default: true).
    */
    fun setCenter(center: Boolean): Unit {
        
        val returnVal = lib.ItemModelConfig_set_center(handle, center);
        
    }
    
    /** Set the texture resolution in pixels per block face (default: 16).
    */
    fun setTextureResolution(resolution: UInt): Unit {
        
        val returnVal = lib.ItemModelConfig_set_texture_resolution(handle, FFIUint32(resolution));
        
    }
    
    /** Set the Minecraft item the model binds to (default: "paper").
    */
    fun setItem(item: String): Result<Unit> {
        val itemSliceMemory = PrimitiveArrayTools.borrowUtf8(item)
        
        val returnVal = lib.ItemModelConfig_set_item(handle, itemSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            itemSliceMemory.close()
        }
    }
    
    /** Set the custom-model-data string used to select this model in game
    *(default: "1").
    */
    fun setCustomModelData(cmd: String): Result<Unit> {
        val cmdSliceMemory = PrimitiveArrayTools.borrowUtf8(cmd)
        
        val returnVal = lib.ItemModelConfig_set_custom_model_data(handle, cmdSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            cmdSliceMemory.close()
        }
    }
    
    /** Uniform scale.
    */
    fun setScale(scale: Float): Unit {
        
        val returnVal = lib.ItemModelConfig_set_scale(handle, scale);
        
    }
    
    /** Per-axis scale.
    */
    fun setScaleXyz(sx: Float, sy: Float, sz: Float): Unit {
        
        val returnVal = lib.ItemModelConfig_set_scale_xyz(handle, sx, sy, sz);
        
    }
    
    /** Auto-fit scale.
    */
    fun setScaleAuto(): Unit {
        
        val returnVal = lib.ItemModelConfig_set_scale_auto(handle);
        
    }

}