package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SchematicLib: Library {
    fun Schematic_destroy(handle: Pointer)
    fun Schematic_create(name: Slice): Pointer
    fun Schematic_dimensions(handle: Pointer): DimensionsNative
    fun Schematic_set_block(handle: Pointer, x: Int, y: Int, z: Int, blockName: Slice): ResultByteInt
    fun Schematic_get_block_name(handle: Pointer, x: Int, y: Int, z: Int, write: Pointer): ResultUnitInt
    fun Schematic_save_to_file(handle: Pointer, path: Slice): ResultUnitInt
    fun Schematic_load_from_file(path: Slice): ResultPointerInt
    fun Schematic_from_data(data: Slice): ResultPointerInt
    fun Schematic_from_litematic(data: Slice): ResultPointerInt
    fun Schematic_to_litematic_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun Schematic_from_schematic(data: Slice): ResultPointerInt
    fun Schematic_to_schematic_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun Schematic_from_snapshot(data: Slice): ResultPointerInt
    fun Schematic_to_snapshot_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun Schematic_from_mcstructure(data: Slice): ResultPointerInt
    fun Schematic_to_mcstructure_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun Schematic_from_mca(data: Slice): ResultPointerInt
    fun Schematic_from_mca_bounded(data: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun Schematic_from_world_zip(data: Slice): ResultPointerInt
    fun Schematic_from_world_zip_bounded(data: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun Schematic_from_world_directory(path: Slice): ResultPointerInt
    fun Schematic_from_world_directory_bounded(path: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun Schematic_to_world_json(handle: Pointer, optionsJson: Slice, write: Pointer): ResultUnitInt
    fun Schematic_save_world(handle: Pointer, directory: Slice, optionsJson: Slice): ResultUnitInt
    fun Schematic_to_world_zip_b64(handle: Pointer, optionsJson: Slice, write: Pointer): ResultUnitInt
    fun Schematic_set_block_with_properties(handle: Pointer, x: Int, y: Int, z: Int, blockName: Slice, propertiesJson: Slice): ResultUnitInt
    fun Schematic_set_block_from_string(handle: Pointer, x: Int, y: Int, z: Int, blockString: Slice): ResultUnitInt
    fun Schematic_prepare_block(handle: Pointer, blockName: Slice): ResultIntInt
    fun Schematic_place(handle: Pointer, x: Int, y: Int, z: Int, paletteIndex: Int): ResultUnitInt
    fun Schematic_set_blocks(handle: Pointer, positions: Slice, blockName: Slice): ResultIntInt
    fun Schematic_get_blocks_json(handle: Pointer, positions: Slice, write: Pointer): ResultUnitInt
    fun Schematic_copy_region(handle: Pointer, source: Pointer, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int, targetX: Int, targetY: Int, targetZ: Int, excludedBlocksJson: Slice): ResultUnitInt
    fun Schematic_get_block_with_properties(handle: Pointer, x: Int, y: Int, z: Int): ResultPointerInt
    fun Schematic_get_block_string(handle: Pointer, x: Int, y: Int, z: Int, write: Pointer): ResultUnitInt
    fun Schematic_get_block_entity_json(handle: Pointer, x: Int, y: Int, z: Int, write: Pointer): ResultUnitInt
    fun Schematic_get_all_block_entities_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_entity_count(handle: Pointer): FFIUint32
    fun Schematic_get_entities_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_add_entity(handle: Pointer, id: Slice, x: Double, y: Double, z: Double, nbtJson: Slice): ResultUnitInt
    fun Schematic_remove_entity(handle: Pointer, index: FFIUint32): ResultUnitInt
    fun Schematic_canonical_data_version(): Int
    fun Schematic_convert_to_data_version(handle: Pointer, targetDataVersion: Int, sourceDataVersion: Int, write: Pointer): Unit
    fun Schematic_convert_to_version(handle: Pointer, targetDataVersion: Int, write: Pointer): Unit
    fun Schematic_source_data_version(handle: Pointer): Int
    fun Schematic_set_source_data_version(handle: Pointer, version: Int): Unit
    fun Schematic_to_litematic_for_version_json(handle: Pointer, targetDataVersion: Int, write: Pointer): ResultUnitInt
    fun Schematic_get_block_entity_snbt(handle: Pointer, x: Int, y: Int, z: Int, write: Pointer): ResultUnitInt
    fun Schematic_set_block_entity(handle: Pointer, x: Int, y: Int, z: Int, id: Slice, snbt: Slice): ResultUnitInt
    fun Schematic_remove_block_entity(handle: Pointer, x: Int, y: Int, z: Int): ResultUnitInt
    fun Schematic_get_all_block_entities_snbt_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_get_entities_snbt_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_add_entity_from_snbt(handle: Pointer, snbt: Slice): ResultUnitInt
    fun Schematic_get_all_blocks_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_get_chunk_blocks_json(handle: Pointer, offsetX: Int, offsetY: Int, offsetZ: Int, width: Int, height: Int, length: Int, write: Pointer): Unit
    fun Schematic_get_chunks_json(handle: Pointer, chunkWidth: Int, chunkHeight: Int, chunkLength: Int, write: Pointer): Unit
    fun Schematic_get_chunks_with_strategy_json(handle: Pointer, chunkWidth: Int, chunkHeight: Int, chunkLength: Int, strategy: Slice, cameraX: Float, cameraY: Float, cameraZ: Float, write: Pointer): Unit
    fun Schematic_block_count(handle: Pointer): Int
    fun Schematic_volume(handle: Pointer): Int
    fun Schematic_region_names_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_debug_info(handle: Pointer, write: Pointer): Unit
    fun Schematic_print_string(handle: Pointer, write: Pointer): Unit
    fun Schematic_print_schematic_string(handle: Pointer, write: Pointer): Unit
    fun Schematic_debug_string(handle: Pointer, write: Pointer): Unit
    fun Schematic_debug_json_string(handle: Pointer, write: Pointer): Unit
    fun Schematic_name(handle: Pointer, write: Pointer): ResultUnitInt
    fun Schematic_set_name(handle: Pointer, name: Slice): ResultUnitInt
    fun Schematic_author(handle: Pointer, write: Pointer): ResultUnitInt
    fun Schematic_set_author(handle: Pointer, author: Slice): ResultUnitInt
    fun Schematic_description(handle: Pointer, write: Pointer): ResultUnitInt
    fun Schematic_set_description(handle: Pointer, description: Slice): ResultUnitInt
    fun Schematic_created(handle: Pointer): Long
    fun Schematic_set_created(handle: Pointer, created: FFIUint64): Unit
    fun Schematic_modified(handle: Pointer): Long
    fun Schematic_set_modified(handle: Pointer, modified: FFIUint64): Unit
    fun Schematic_lm_version(handle: Pointer): Int
    fun Schematic_set_lm_version(handle: Pointer, version: Int): Unit
    fun Schematic_mc_version(handle: Pointer): Int
    fun Schematic_set_mc_version(handle: Pointer, version: Int): Unit
    fun Schematic_we_version(handle: Pointer): Int
    fun Schematic_set_we_version(handle: Pointer, version: Int): Unit
    fun Schematic_flip_x(handle: Pointer): Unit
    fun Schematic_flip_y(handle: Pointer): Unit
    fun Schematic_flip_z(handle: Pointer): Unit
    fun Schematic_rotate_x(handle: Pointer, degrees: Int): Unit
    fun Schematic_rotate_y(handle: Pointer, degrees: Int): Unit
    fun Schematic_rotate_z(handle: Pointer, degrees: Int): Unit
    fun Schematic_flip_region_x(handle: Pointer, regionName: Slice): ResultUnitInt
    fun Schematic_flip_region_y(handle: Pointer, regionName: Slice): ResultUnitInt
    fun Schematic_flip_region_z(handle: Pointer, regionName: Slice): ResultUnitInt
    fun Schematic_rotate_region_x(handle: Pointer, regionName: Slice, degrees: Int): ResultUnitInt
    fun Schematic_rotate_region_y(handle: Pointer, regionName: Slice, degrees: Int): ResultUnitInt
    fun Schematic_rotate_region_z(handle: Pointer, regionName: Slice, degrees: Int): ResultUnitInt
    fun Schematic_fill_cuboid(handle: Pointer, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int, blockName: Slice): ResultUnitInt
    fun Schematic_fill_sphere(handle: Pointer, cx: Float, cy: Float, cz: Float, radius: Float, blockName: Slice): ResultUnitInt
    fun Schematic_save_as_b64(handle: Pointer, format: Slice, version: Slice, settings: Slice, write: Pointer): ResultUnitInt
    fun Schematic_save_to_file_with_format(handle: Pointer, path: Slice, format: Slice, version: Slice): ResultUnitInt
    fun Schematic_to_schematic_version_b64(handle: Pointer, version: Slice, write: Pointer): ResultUnitInt
    fun Schematic_available_schematic_versions_json(write: Pointer): ResultUnitInt
    fun Schematic_set_block_with_nbt(handle: Pointer, x: Int, y: Int, z: Int, blockName: Slice, nbtJson: Slice): ResultUnitInt
    fun Schematic_set_block_in_region(handle: Pointer, regionName: Slice, x: Int, y: Int, z: Int, blockName: Slice): ResultUnitInt
    fun Schematic_bounding_box_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_region_bounding_box_json(handle: Pointer, regionName: Slice, write: Pointer): ResultUnitInt
    fun Schematic_palette_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_tight_dimensions(handle: Pointer): DimensionsNative
    fun Schematic_allocated_dimensions(handle: Pointer): DimensionsNative
    fun Schematic_extract_signs_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_compile_insign_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun Schematic_all_palettes_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_default_region_palette_json(handle: Pointer, write: Pointer): Unit
    fun Schematic_region_palette_json(handle: Pointer, regionName: Slice, write: Pointer): ResultUnitInt
    fun Schematic_tight_bounds_min(handle: Pointer): ResultBlockPosNativeInt
    fun Schematic_tight_bounds_max(handle: Pointer): ResultBlockPosNativeInt
}

class Schematic internal constructor (
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

    private class SchematicCleaner(val handle: Pointer, val lib: SchematicLib) : Runnable {
        override fun run() {
            lib.Schematic_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Schematic.SchematicCleaner(handle, Schematic.lib));
    }

    companion object {
        internal val libClass: Class<SchematicLib> = SchematicLib::class.java
        internal val lib: SchematicLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(name: String): Schematic {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.Schematic_create(nameSliceMemory.slice);
            try {
                val selfEdges: List<Any> = listOf()
                val handle = returnVal 
                val returnOpaque = Schematic(handle, selfEdges, true)
                return returnOpaque
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        fun loadFromFile(path: String): Result<Schematic> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
            
            val returnVal = lib.Schematic_load_from_file(pathSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                pathSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Build a schematic from raw byte data, auto-detecting the format.
        *Supports Litematic, Sponge Schematic, and McStructure (Bedrock) formats.
        *`Parse` if a format was detected but failed to parse, `InvalidArgument` if
        *no format was recognized.
        */
        fun fromData(data: UByteArray): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Schematic_from_data(dataSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Build a schematic from Litematic data.
        */
        fun fromLitematic(data: UByteArray): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Schematic_from_litematic(dataSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Build a schematic from classic `.schematic` data.
        */
        fun fromSchematic(data: UByteArray): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Schematic_from_schematic(dataSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Build a schematic from snapshot (fast binary) data.
        */
        fun fromSnapshot(data: UByteArray): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Schematic_from_snapshot(dataSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Build a schematic from McStructure (Bedrock) data.
        */
        fun fromMcstructure(data: UByteArray): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Schematic_from_mcstructure(dataSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Import from a single MCA region file.
        */
        fun fromMca(data: UByteArray): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Schematic_from_mca(dataSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Import from MCA with coordinate bounds.
        */
        fun fromMcaBounded(data: UByteArray, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Schematic_from_mca_bounded(dataSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Import from a zipped world folder.
        */
        fun fromWorldZip(data: UByteArray): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Schematic_from_world_zip(dataSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Import from zipped world with coordinate bounds.
        */
        fun fromWorldZipBounded(data: UByteArray, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Schematic_from_world_zip_bounded(dataSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Import from a Minecraft world directory path.
        */
        fun fromWorldDirectory(path: String): Result<Schematic> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
            
            val returnVal = lib.Schematic_from_world_directory(pathSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                pathSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Import from world directory with coordinate bounds.
        */
        fun fromWorldDirectoryBounded(path: String, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Schematic> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
            
            val returnVal = lib.Schematic_from_world_directory_bounded(pathSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                pathSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** The canonical in-memory data version (the forward-conversion target).
        */
        fun canonicalDataVersion(): Int {
            
            val returnVal = lib.Schematic_canonical_data_version();
            return (returnVal)
        }
        @JvmStatic
        
        /** The available Sponge schematic exporter versions, as a JSON array of
        *strings.
        */
        fun availableSchematicVersionsJson(): Result<String> {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Schematic_available_schematic_versions_json(write);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }
    
    fun dimensions(): Dimensions {
        
        val returnVal = lib.Schematic_dimensions(handle);
        val returnStruct = Dimensions.fromNative(returnVal)
        return returnStruct
    }
    
    /** Returns `true` if a block was placed (out-of-range coordinates extend the
    *schematic rather than erroring, matching `UniversalSchematic::set_block`).
    */
    fun setBlock(x: Int, y: Int, z: Int, blockName: String): Result<Boolean> {
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        
        val returnVal = lib.Schematic_set_block(handle, x, y, z, blockNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal > 0).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockNameSliceMemory.close()
        }
    }
    
    fun getBlockName(x: Int, y: Int, z: Int): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_block_name(handle, x, y, z, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun saveToFile(path: String): Result<Unit> {
        val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
        
        val returnVal = lib.Schematic_save_to_file(handle, pathSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            pathSliceMemory.close()
        }
    }
    
    /** The schematic as Litematic bytes, base64-encoded.
    */
    fun toLitematicB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_to_litematic_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The schematic as classic `.schematic` bytes, base64-encoded.
    */
    fun toSchematicB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_to_schematic_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The schematic as snapshot (fast binary) bytes, base64-encoded.
    */
    fun toSnapshotB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_to_snapshot_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The schematic as McStructure (Bedrock) bytes, base64-encoded.
    */
    fun toMcstructureB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_to_mcstructure_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Export the schematic as a Minecraft world: a JSON array of
    *`{"path": <relative file path>, "data_b64": <base64 bytes>}` entries
    *(the old `CFileMap`). `options_json` may be empty for defaults.
    */
    fun toWorldJson(optionsJson: String): Result<String> {
        val optionsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(optionsJson)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_to_world_json(handle, optionsJsonSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            optionsJsonSliceMemory.close()
        }
    }
    
    /** Export and write world files to a directory. `options_json` may be empty.
    */
    fun saveWorld(directory: String, optionsJson: String): Result<Unit> {
        val directorySliceMemory = PrimitiveArrayTools.borrowUtf8(directory)
        val optionsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(optionsJson)
        
        val returnVal = lib.Schematic_save_world(handle, directorySliceMemory.slice, optionsJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            directorySliceMemory.close()
            optionsJsonSliceMemory.close()
        }
    }
    
    /** Export the schematic as a zipped Minecraft world, base64-encoded.
    *`options_json` may be empty for defaults.
    */
    fun toWorldZipB64(optionsJson: String): Result<String> {
        val optionsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(optionsJson)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_to_world_zip_b64(handle, optionsJsonSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            optionsJsonSliceMemory.close()
        }
    }
    
    /** Set a block with properties given as a JSON object of string→string
    *(the old `CProperty` array).
    */
    fun setBlockWithProperties(x: Int, y: Int, z: Int, blockName: String, propertiesJson: String): Result<Unit> {
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        val propertiesJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(propertiesJson)
        
        val returnVal = lib.Schematic_set_block_with_properties(handle, x, y, z, blockNameSliceMemory.slice, propertiesJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockNameSliceMemory.close()
            propertiesJsonSliceMemory.close()
        }
    }
    
    /** Set a block from a full block string, e.g.
    *`minecraft:chest[facing=north]{Items:[...]}`.
    */
    fun setBlockFromString(x: Int, y: Int, z: Int, blockString: String): Result<Unit> {
        val blockStringSliceMemory = PrimitiveArrayTools.borrowUtf8(blockString)
        
        val returnVal = lib.Schematic_set_block_from_string(handle, x, y, z, blockStringSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockStringSliceMemory.close()
        }
    }
    
    /** Pre-resolve a plain block name to a palette index for use with `place`.
    *Pair them in hot loops with many unique block names to skip the per-call
    *name → palette lookup.
    */
    fun prepareBlock(blockName: String): Result<Int> {
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        
        val returnVal = lib.Schematic_prepare_block(handle, blockNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockNameSliceMemory.close()
        }
    }
    
    /** Place a block by pre-resolved palette index (from `prepare_block`).
    */
    fun place(x: Int, y: Int, z: Int, paletteIndex: Int): Result<Unit> {
        
        val returnVal = lib.Schematic_place(handle, x, y, z, paletteIndex);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Batch-set blocks at multiple positions to the same block (name, block
    *string with properties, or block string with NBT). `positions` is flat
    *`[x0,y0,z0, x1,y1,z1, ...]` (length must be a multiple of 3).
    *Returns the number of blocks set.
    */
    fun setBlocks(positions: IntArray, blockName: String): Result<Int> {
        val positionsSliceMemory = PrimitiveArrayTools.borrow(positions)
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        
        val returnVal = lib.Schematic_set_blocks(handle, positionsSliceMemory.slice, blockNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            positionsSliceMemory.close()
            blockNameSliceMemory.close()
        }
    }
    
    /** Batch-get block names at multiple positions. `positions` is flat
    *`[x0,y0,z0, ...]` (length must be a multiple of 3). Writes a JSON array,
    *one entry per position: the block name string, or `null` for
    *empty/out-of-bounds positions.
    */
    fun getBlocksJson(positions: IntArray): Result<String> {
        val positionsSliceMemory = PrimitiveArrayTools.borrow(positions)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_blocks_json(handle, positionsSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            positionsSliceMemory.close()
        }
    }
    
    /** Copy a region from `source` into this schematic. `excluded_blocks_json`
    *is a JSON array of block strings to skip (empty string or `[]` for none).
    */
    fun copyRegion(source: Schematic, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int, targetX: Int, targetY: Int, targetZ: Int, excludedBlocksJson: String): Result<Unit> {
        val excludedBlocksJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(excludedBlocksJson)
        
        val returnVal = lib.Schematic_copy_region(handle, source.handle, minX, minY, minZ, maxX, maxY, maxZ, targetX, targetY, targetZ, excludedBlocksJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            excludedBlocksJsonSliceMemory.close()
        }
    }
    
    /** The block at a position with its properties, as a `BlockState`.
    */
    fun getBlockWithProperties(x: Int, y: Int, z: Int): Result<BlockState> {
        
        val returnVal = lib.Schematic_get_block_with_properties(handle, x, y, z);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal 
            val returnOpaque = BlockState(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The full block string (name, properties, NBT) at a position.
    */
    fun getBlockString(x: Int, y: Int, z: Int): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_block_string(handle, x, y, z, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The block entity at a position as JSON
    *`{"id": ..., "position": [x,y,z], "nbt": {...}}` (the old `CBlockEntity`).
    */
    fun getBlockEntityJson(x: Int, y: Int, z: Int): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_block_entity_json(handle, x, y, z, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Every block entity as a JSON array of
    *`{"id": ..., "position": [x,y,z], "nbt": {...}}`.
    */
    fun getAllBlockEntitiesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_all_block_entities_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** The number of mobile entities (not block entities).
    */
    fun entityCount(): UInt {
        
        val returnVal = lib.Schematic_entity_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Every mobile entity as a JSON array of
    *`{"id": ..., "position": [x,y,z], "nbt": {...}}` (the old `CEntityArray`).
    */
    fun getEntitiesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_entities_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Add a mobile entity. `nbt_json` is a JSON object (may be empty).
    */
    fun addEntity(id: String, x: Double, y: Double, z: Double, nbtJson: String): Result<Unit> {
        val idSliceMemory = PrimitiveArrayTools.borrowUtf8(id)
        val nbtJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(nbtJson)
        
        val returnVal = lib.Schematic_add_entity(handle, idSliceMemory.slice, x, y, z, nbtJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            idSliceMemory.close()
            nbtJsonSliceMemory.close()
        }
    }
    
    /** Remove a mobile entity by index.
    */
    fun removeEntity(index: UInt): Result<Unit> {
        
        val returnVal = lib.Schematic_remove_entity(handle, FFIUint32(index));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Convert block/item/entity data between Minecraft data versions. Forward
    *(`target >= source`) is lossless; reverse is lossy. Writes a JSON loss
    *report (`[]` when lossless).
    */
    fun convertToDataVersion(targetDataVersion: Int, sourceDataVersion: Int): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_convert_to_data_version(handle, targetDataVersion, sourceDataVersion, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Convert to `target_data_version` using the schematic's captured source
    *version (else `mc_version`, else canonical) as origin, updating metadata
    *to the target. Writes a JSON loss report (`[]` when lossless).
    */
    fun convertToVersion(targetDataVersion: Int): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_convert_to_version(handle, targetDataVersion, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** The Minecraft data version of the file this schematic was loaded from, or
    *`-1` if none was captured (versionless / freshly built).
    */
    fun sourceDataVersion(): Int {
        
        val returnVal = lib.Schematic_source_data_version(handle);
        return (returnVal)
    }
    
    /** Override the source data version for formats that carry no Java data
    *version, so the converter knows what to convert *from*.
    */
    fun setSourceDataVersion(version: Int): Unit {
        
        val returnVal = lib.Schematic_set_source_data_version(handle, version);
        
    }
    
    /** Serialize a `.litematic` targeting a specific Minecraft data version. A
    *COPY is converted and the matching Version header written; the schematic
    *is left unchanged. Writes JSON
    *`{"data_b64": <base64 .litematic>, "loss": <loss report>}`.
    */
    fun toLitematicForVersionJson(targetDataVersion: Int): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_to_litematic_for_version_json(handle, targetDataVersion, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The block entity's NBT as a typed SNBT string. Round-trips losslessly.
    */
    fun getBlockEntitySnbt(x: Int, y: Int, z: Int): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_block_entity_snbt(handle, x, y, z, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Set (or replace) a block entity at a position from a typed SNBT string.
    */
    fun setBlockEntity(x: Int, y: Int, z: Int, id: String, snbt: String): Result<Unit> {
        val idSliceMemory = PrimitiveArrayTools.borrowUtf8(id)
        val snbtSliceMemory = PrimitiveArrayTools.borrowUtf8(snbt)
        
        val returnVal = lib.Schematic_set_block_entity(handle, x, y, z, idSliceMemory.slice, snbtSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            idSliceMemory.close()
            snbtSliceMemory.close()
        }
    }
    
    /** Remove the block entity at a position. `NotFound` if none was there.
    */
    fun removeBlockEntity(x: Int, y: Int, z: Int): Result<Unit> {
        
        val returnVal = lib.Schematic_remove_block_entity(handle, x, y, z);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Every block entity as a JSON array of `{id, position: [x,y,z], snbt}`.
    *The `snbt` is the inner data only (no `Id`/`Pos`).
    */
    fun getAllBlockEntitiesSnbtJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_all_block_entities_snbt_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Every mobile entity as a JSON array of typed SNBT strings (full compound
    *incl. `id`/`Pos`).
    */
    fun getEntitiesSnbtJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_entities_snbt_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Add a mobile entity from a full SNBT entity compound (must contain `id`
    *and `Pos`).
    */
    fun addEntityFromSnbt(snbt: String): Result<Unit> {
        val snbtSliceMemory = PrimitiveArrayTools.borrowUtf8(snbt)
        
        val returnVal = lib.Schematic_add_entity_from_snbt(handle, snbtSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            snbtSliceMemory.close()
        }
    }
    
    /** Every non-air block as a JSON array of
    *`{"x", "y", "z", "name", "properties"}` (the old `CBlockArray`).
    */
    fun getAllBlocksJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_all_blocks_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** All blocks within a sub-region (chunk) of the schematic, as the same
    *JSON array shape as `get_all_blocks_json`.
    */
    fun getChunkBlocksJson(offsetX: Int, offsetY: Int, offsetZ: Int, width: Int, height: Int, length: Int): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_chunk_blocks_json(handle, offsetX, offsetY, offsetZ, width, height, length, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Split the schematic into chunks (default bottom-up strategy). Writes a
    *JSON array of `{"chunk_x", "chunk_y", "chunk_z", "blocks": [...]}` where
    *blocks have the `get_all_blocks_json` shape (the old `CChunkArray`).
    */
    fun getChunksJson(chunkWidth: Int, chunkHeight: Int, chunkLength: Int): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_chunks_json(handle, chunkWidth, chunkHeight, chunkLength, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Split the schematic into chunks with a loading strategy: one of
    *`distance_to_camera`, `top_down`, `bottom_up`, `center_outward`,
    *`random` (anything else falls back to `bottom_up`). Camera coordinates
    *are only used by `distance_to_camera`. Same JSON shape as
    *`get_chunks_json`.
    */
    fun getChunksWithStrategyJson(chunkWidth: Int, chunkHeight: Int, chunkLength: Int, strategy: String, cameraX: Float, cameraY: Float, cameraZ: Float): String {
        val strategySliceMemory = PrimitiveArrayTools.borrowUtf8(strategy)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_get_chunks_with_strategy_json(handle, chunkWidth, chunkHeight, chunkLength, strategySliceMemory.slice, cameraX, cameraY, cameraZ, write);
        try {
            
            val returnString = DW.writeToString(write)
            return returnString
        } finally {
            strategySliceMemory.close()
        }
    }
    
    /** The total number of non-air blocks in the schematic.
    */
    fun blockCount(): Int {
        
        val returnVal = lib.Schematic_block_count(handle);
        return (returnVal)
    }
    
    /** The total volume of the schematic's bounding box.
    */
    fun volume(): Int {
        
        val returnVal = lib.Schematic_volume(handle);
        return (returnVal)
    }
    
    /** The names of all regions, as a JSON array of strings.
    */
    fun regionNamesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_region_names_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Basic debug info about the schematic (name + region count).
    */
    fun debugInfo(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_debug_info(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** A formatted schematic layout string (old `schematic_print`).
    */
    fun printString(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_print_string(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** A formatted schematic layout string (old `schematic_print_schematic`;
    *same output as `print_string`).
    */
    fun printSchematicString(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_print_schematic_string(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** A detailed debug string, including a visual layout (old `debug_schematic`).
    */
    fun debugString(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_debug_string(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** A detailed debug string with a JSON layout (old `debug_json_schematic`).
    */
    fun debugJsonString(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_debug_json_string(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** The schematic name. `NotFound` if not set.
    */
    fun name(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_name(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun setName(name: String): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        
        val returnVal = lib.Schematic_set_name(handle, nameSliceMemory.slice);
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
    
    /** The schematic author. `NotFound` if not set.
    */
    fun author(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_author(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun setAuthor(author: String): Result<Unit> {
        val authorSliceMemory = PrimitiveArrayTools.borrowUtf8(author)
        
        val returnVal = lib.Schematic_set_author(handle, authorSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            authorSliceMemory.close()
        }
    }
    
    /** The schematic description. `NotFound` if not set.
    */
    fun description(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_description(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun setDescription(description: String): Result<Unit> {
        val descriptionSliceMemory = PrimitiveArrayTools.borrowUtf8(description)
        
        val returnVal = lib.Schematic_set_description(handle, descriptionSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            descriptionSliceMemory.close()
        }
    }
    
    /** The creation timestamp (milliseconds since epoch), or `-1` if not set.
    */
    fun created(): Long {
        
        val returnVal = lib.Schematic_created(handle);
        return (returnVal)
    }
    
    /** Set the creation timestamp (milliseconds since epoch).
    */
    fun setCreated(created: ULong): Unit {
        
        val returnVal = lib.Schematic_set_created(handle, FFIUint64(created));
        
    }
    
    /** The modification timestamp (milliseconds since epoch), or `-1` if not set.
    */
    fun modified(): Long {
        
        val returnVal = lib.Schematic_modified(handle);
        return (returnVal)
    }
    
    /** Set the modification timestamp (milliseconds since epoch).
    */
    fun setModified(modified: ULong): Unit {
        
        val returnVal = lib.Schematic_set_modified(handle, FFIUint64(modified));
        
    }
    
    /** The Litematic format version, or `-1` if not set.
    */
    fun lmVersion(): Int {
        
        val returnVal = lib.Schematic_lm_version(handle);
        return (returnVal)
    }
    
    fun setLmVersion(version: Int): Unit {
        
        val returnVal = lib.Schematic_set_lm_version(handle, version);
        
    }
    
    /** The Minecraft data version, or `-1` if not set.
    */
    fun mcVersion(): Int {
        
        val returnVal = lib.Schematic_mc_version(handle);
        return (returnVal)
    }
    
    fun setMcVersion(version: Int): Unit {
        
        val returnVal = lib.Schematic_set_mc_version(handle, version);
        
    }
    
    /** The WorldEdit version, or `-1` if not set.
    */
    fun weVersion(): Int {
        
        val returnVal = lib.Schematic_we_version(handle);
        return (returnVal)
    }
    
    fun setWeVersion(version: Int): Unit {
        
        val returnVal = lib.Schematic_set_we_version(handle, version);
        
    }
    
    fun flipX(): Unit {
        
        val returnVal = lib.Schematic_flip_x(handle);
        
    }
    
    fun flipY(): Unit {
        
        val returnVal = lib.Schematic_flip_y(handle);
        
    }
    
    fun flipZ(): Unit {
        
        val returnVal = lib.Schematic_flip_z(handle);
        
    }
    
    fun rotateX(degrees: Int): Unit {
        
        val returnVal = lib.Schematic_rotate_x(handle, degrees);
        
    }
    
    fun rotateY(degrees: Int): Unit {
        
        val returnVal = lib.Schematic_rotate_y(handle, degrees);
        
    }
    
    fun rotateZ(degrees: Int): Unit {
        
        val returnVal = lib.Schematic_rotate_z(handle, degrees);
        
    }
    
    fun flipRegionX(regionName: String): Result<Unit> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        
        val returnVal = lib.Schematic_flip_region_x(handle, regionNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
        }
    }
    
    fun flipRegionY(regionName: String): Result<Unit> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        
        val returnVal = lib.Schematic_flip_region_y(handle, regionNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
        }
    }
    
    fun flipRegionZ(regionName: String): Result<Unit> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        
        val returnVal = lib.Schematic_flip_region_z(handle, regionNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
        }
    }
    
    fun rotateRegionX(regionName: String, degrees: Int): Result<Unit> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        
        val returnVal = lib.Schematic_rotate_region_x(handle, regionNameSliceMemory.slice, degrees);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
        }
    }
    
    fun rotateRegionY(regionName: String, degrees: Int): Result<Unit> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        
        val returnVal = lib.Schematic_rotate_region_y(handle, regionNameSliceMemory.slice, degrees);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
        }
    }
    
    fun rotateRegionZ(regionName: String, degrees: Int): Result<Unit> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        
        val returnVal = lib.Schematic_rotate_region_z(handle, regionNameSliceMemory.slice, degrees);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
        }
    }
    
    /** Fill a cuboid with a block.
    */
    fun fillCuboid(minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int, blockName: String): Result<Unit> {
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        
        val returnVal = lib.Schematic_fill_cuboid(handle, minX, minY, minZ, maxX, maxY, maxZ, blockNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockNameSliceMemory.close()
        }
    }
    
    /** Fill a sphere with a block.
    */
    fun fillSphere(cx: Float, cy: Float, cz: Float, radius: Float, blockName: String): Result<Unit> {
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        
        val returnVal = lib.Schematic_fill_sphere(handle, cx, cy, cz, radius, blockNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockNameSliceMemory.close()
        }
    }
    
    /** Serialize to a named format, base64-encoded. `version` and `settings`
    *may be empty strings for defaults.
    */
    fun saveAsB64(format: String, version: String, settings: String): Result<String> {
        val formatSliceMemory = PrimitiveArrayTools.borrowUtf8(format)
        val versionSliceMemory = PrimitiveArrayTools.borrowUtf8(version)
        val settingsSliceMemory = PrimitiveArrayTools.borrowUtf8(settings)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_save_as_b64(handle, formatSliceMemory.slice, versionSliceMemory.slice, settingsSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            formatSliceMemory.close()
            versionSliceMemory.close()
            settingsSliceMemory.close()
        }
    }
    
    /** Save to a file. If `format` is empty, the format is auto-detected from
    *the file extension; `version` may be empty for the default.
    */
    fun saveToFileWithFormat(path: String, format: String, version: String): Result<Unit> {
        val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
        val formatSliceMemory = PrimitiveArrayTools.borrowUtf8(format)
        val versionSliceMemory = PrimitiveArrayTools.borrowUtf8(version)
        
        val returnVal = lib.Schematic_save_to_file_with_format(handle, pathSliceMemory.slice, formatSliceMemory.slice, versionSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            pathSliceMemory.close()
            formatSliceMemory.close()
            versionSliceMemory.close()
        }
    }
    
    /** Serialize as a Sponge schematic targeting a specific format version,
    *base64-encoded.
    */
    fun toSchematicVersionB64(version: String): Result<String> {
        val versionSliceMemory = PrimitiveArrayTools.borrowUtf8(version)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_to_schematic_version_b64(handle, versionSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            versionSliceMemory.close()
        }
    }
    
    /** Set a block with NBT data given as a JSON object of string→string
    *(may be empty).
    */
    fun setBlockWithNbt(x: Int, y: Int, z: Int, blockName: String, nbtJson: String): Result<Unit> {
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        val nbtJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(nbtJson)
        
        val returnVal = lib.Schematic_set_block_with_nbt(handle, x, y, z, blockNameSliceMemory.slice, nbtJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockNameSliceMemory.close()
            nbtJsonSliceMemory.close()
        }
    }
    
    /** Set a block (by name) in a named region.
    */
    fun setBlockInRegion(regionName: String, x: Int, y: Int, z: Int, blockName: String): Result<Unit> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        
        val returnVal = lib.Schematic_set_block_in_region(handle, regionNameSliceMemory.slice, x, y, z, blockNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
            blockNameSliceMemory.close()
        }
    }
    
    /** The schematic bounding box as a JSON array
    *`[min_x, min_y, min_z, max_x, max_y, max_z]`.
    */
    fun boundingBoxJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_bounding_box_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** A named region's bounding box as a JSON array
    *`[min_x, min_y, min_z, max_x, max_y, max_z]`. `"default"`/`"Default"`
    *address the default region.
    */
    fun regionBoundingBoxJson(regionName: String): Result<String> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_region_bounding_box_json(handle, regionNameSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
        }
    }
    
    /** The merged-region palette block names, as a JSON array of strings.
    */
    fun paletteJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_palette_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** The tight (content) dimensions.
    */
    fun tightDimensions(): Dimensions {
        
        val returnVal = lib.Schematic_tight_dimensions(handle);
        val returnStruct = Dimensions.fromNative(returnVal)
        return returnStruct
    }
    
    /** The allocated dimensions (same as `dimensions`; named for parity with
    *the old `schematic_get_allocated_dimensions`).
    */
    fun allocatedDimensions(): Dimensions {
        
        val returnVal = lib.Schematic_allocated_dimensions(handle);
        val returnStruct = Dimensions.fromNative(returnVal)
        return returnStruct
    }
    
    /** Every sign in the schematic, as a JSON array of
    *`{"pos": [x,y,z], "text": [...]}`.
    */
    fun extractSignsJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_extract_signs_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Compile the schematic's insign annotations to JSON.
    */
    fun compileInsignJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_compile_insign_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Every region's palette, as a JSON object mapping region name → array of
    *block names (the default region under `"default"`).
    */
    fun allPalettesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_all_palettes_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** The default region's palette block names, as a JSON array of strings.
    */
    fun defaultRegionPaletteJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_default_region_palette_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** A named region's palette block names, as a JSON array of strings.
    *`"default"`/`"Default"` address the default region.
    */
    fun regionPaletteJson(regionName: String): Result<String> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Schematic_region_palette_json(handle, regionNameSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
        }
    }
    
    /** The minimum corner of the tight (content) bounds. `NotFound` when the
    *schematic has no content.
    */
    fun tightBoundsMin(): Result<BlockPos> {
        
        val returnVal = lib.Schematic_tight_bounds_min(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = BlockPos.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The maximum corner of the tight (content) bounds. `NotFound` when the
    *schematic has no content.
    */
    fun tightBoundsMax(): Result<BlockPos> {
        
        val returnVal = lib.Schematic_tight_bounds_max(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = BlockPos.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}