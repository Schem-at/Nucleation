package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ItemModelResultLib: Library {
    fun ItemModelResult_destroy(handle: Pointer)
    fun ItemModelResult_create(schematic: Pointer, pack: Pointer, config: Pointer): ResultPointerInt
    fun ItemModelResult_model_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun ItemModelResult_element_count(handle: Pointer): ResultFFIUint32Int
    fun ItemModelResult_texture_count(handle: Pointer): ResultFFIUint32Int
    fun ItemModelResult_plane_count(handle: Pointer): ResultFFIUint32Int
    fun ItemModelResult_dimensions(handle: Pointer): ResultDimensionsNativeInt
    fun ItemModelResult_scale(handle: Pointer): ResultItemScaleNativeInt
    fun ItemModelResult_to_resource_pack_zip_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun ItemModelResult_add_to_pack(handle: Pointer, builder: Pointer): ResultUnitInt
}
/** A generated item model. Wraps [crate::meshing::ItemModelResult].
*
*Holds `Option<...>` because [ItemModelResult::add_to_pack] moves the
*inner value into an [ItemModelPackBuilder] (PORTING rule 11); accessors
*return `AlreadyConsumed` afterwards.
*/
class ItemModelResult internal constructor (
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

    private class ItemModelResultCleaner(val handle: Pointer, val lib: ItemModelResultLib) : Runnable {
        override fun run() {
            lib.ItemModelResult_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, ItemModelResult.ItemModelResultCleaner(handle, ItemModelResult.lib));
    }

    companion object {
        internal val libClass: Class<ItemModelResultLib> = ItemModelResultLib::class.java
        internal val lib: ItemModelResultLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Generate an item model from a schematic (old ABI: `schematic_to_item_model`).
        */
        fun create(schematic: Schematic, pack: ResourcePack, config: ItemModelConfig): Result<ItemModelResult> {
            
            val returnVal = lib.ItemModelResult_create(schematic.handle, pack.handle, config.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = ItemModelResult(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }
    
    /** The Minecraft item model JSON.
    */
    fun modelJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ItemModelResult_model_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun elementCount(): Result<UInt> {
        
        val returnVal = lib.ItemModelResult_element_count(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal.toUInt()).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun textureCount(): Result<UInt> {
        
        val returnVal = lib.ItemModelResult_texture_count(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal.toUInt()).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun planeCount(): Result<UInt> {
        
        val returnVal = lib.ItemModelResult_plane_count(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal.toUInt()).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun dimensions(): Result<Dimensions> {
        
        val returnVal = lib.ItemModelResult_dimensions(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = Dimensions.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The applied model scale (old ABI: `itemmodel_result_scale`).
    */
    fun scale(): Result<ItemScale> {
        
        val returnVal = lib.ItemModelResult_scale(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = ItemScale.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** A single-model resource pack ZIP, base64-encoded.
    */
    fun toResourcePackZipB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ItemModelResult_to_resource_pack_zip_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Move this result into a pack builder. Consumes the result: further
    *accessor calls return `AlreadyConsumed`.
    */
    fun addToPack(builder: ItemModelPackBuilder): Result<Unit> {
        
        val returnVal = lib.ItemModelResult_add_to_pack(handle, builder.handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}