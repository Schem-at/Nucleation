package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SchematicRegionsLib: Library {
    fun SchematicRegions_destroy(handle: Pointer)
    fun SchematicRegions_add(schematic: Pointer, name: Slice, region: Pointer): ResultUnitInt
    fun SchematicRegions_update(schematic: Pointer, name: Slice, region: Pointer): ResultUnitInt
    fun SchematicRegions_get(schematic: Pointer, name: Slice): ResultPointerInt
    fun SchematicRegions_remove(schematic: Pointer, name: Slice): ResultUnitInt
    fun SchematicRegions_names_json(schematic: Pointer, write: Pointer): ResultUnitInt
    fun SchematicRegions_create(schematic: Pointer, name: Slice): ResultUnitInt
    fun SchematicRegions_create_from_point(schematic: Pointer, name: Slice, x: Int, y: Int, z: Int): ResultUnitInt
    fun SchematicRegions_create_from_bounds(schematic: Pointer, name: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultUnitInt
    fun SchematicRegions_create_region(schematic: Pointer, name: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun SchematicRegions_add_bounds_to(schematic: Pointer, name: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultUnitInt
    fun SchematicRegions_add_point_to(schematic: Pointer, name: Slice, x: Int, y: Int, z: Int): ResultUnitInt
    fun SchematicRegions_set_metadata_on(schematic: Pointer, name: Slice, key: Slice, value: Slice): ResultUnitInt
    fun SchematicRegions_shift_region(schematic: Pointer, name: Slice, dx: Int, dy: Int, dz: Int): ResultUnitInt
}
/** Namespace type for the schematic-attached definition-region operations
*(PORTING rule 12; the `Schematic` opaque lives in another module, so
*these are statics taking it explicitly, like `Autostack`).
*/
class SchematicRegions internal constructor (
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

    private class SchematicRegionsCleaner(val handle: Pointer, val lib: SchematicRegionsLib) : Runnable {
        override fun run() {
            lib.SchematicRegions_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, SchematicRegions.SchematicRegionsCleaner(handle, SchematicRegions.lib));
    }

    companion object {
        internal val libClass: Class<SchematicRegionsLib> = SchematicRegionsLib::class.java
        internal val lib: SchematicRegionsLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Insert (or overwrite) `region` under `name`.
        */
        fun add(schematic: Schematic, name: String, region: DefinitionRegion): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_add(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice, region.handle);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Overwrite the region stored under `name` (identical to `add` in the
        *old ABI too; kept as a separate method for 1:1 coverage of
        *`schematic_update_region`).
        */
        fun update(schematic: Schematic, name: String, region: DefinitionRegion): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_update(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice, region.handle);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** A copy of the region stored under `name`. Errors with `NotFound` when
        *absent.
        */
        fun get(schematic: Schematic, name: String): Result<DefinitionRegion> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_get(schematic.handle, nameSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = DefinitionRegion(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Remove the region stored under `name`. Errors with `NotFound` when
        *absent.
        */
        fun remove(schematic: Schematic, name: String): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_remove(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** The names of every definition region, written as a JSON array string.
        */
        fun namesJson(schematic: Schematic): Result<String> {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.SchematicRegions_names_json(schematic.handle, write);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** Create an empty region under `name`.
        */
        fun create(schematic: Schematic, name: String): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_create(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Create a single-point region under `name`.
        */
        fun createFromPoint(schematic: Schematic, name: String, x: Int, y: Int, z: Int): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_create_from_point(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice, x, y, z);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Create a single-box region under `name`.
        */
        fun createFromBounds(schematic: Schematic, name: String, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_create_from_bounds(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Create a single-box region under `name` and return a copy of it.
        */
        fun createRegion(schematic: Schematic, name: String, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<DefinitionRegion> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_create_region(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = DefinitionRegion(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Add a box to the region stored under `name`. Errors with `NotFound`
        *when absent.
        */
        fun addBoundsTo(schematic: Schematic, name: String, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_add_bounds_to(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Add a point to the region stored under `name`. Errors with `NotFound`
        *when absent.
        */
        fun addPointTo(schematic: Schematic, name: String, x: Int, y: Int, z: Int): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_add_point_to(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice, x, y, z);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Set a metadata entry on the region stored under `name`. Errors with
        *`NotFound` when absent.
        */
        fun setMetadataOn(schematic: Schematic, name: String, key: String, value: String): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
            val valueSliceMemory = PrimitiveArrayTools.borrowUtf8(value)
            
            val returnVal = lib.SchematicRegions_set_metadata_on(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice, keySliceMemory.slice, valueSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
                keySliceMemory.close()
                valueSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Shift the region stored under `name` by (`dx`, `dy`, `dz`). Errors
        *with `NotFound` when absent.
        */
        fun shiftRegion(schematic: Schematic, name: String, dx: Int, dy: Int, dz: Int): Result<Unit> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.SchematicRegions_shift_region(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice, dx, dy, dz);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
    }

}