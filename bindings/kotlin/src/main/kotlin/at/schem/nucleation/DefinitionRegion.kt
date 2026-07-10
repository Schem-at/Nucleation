package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface DefinitionRegionLib: Library {
    fun DefinitionRegion_destroy(handle: Pointer)
    fun DefinitionRegion_create(): Pointer
    fun DefinitionRegion_from_bounds(minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Pointer
    fun DefinitionRegion_from_positions(positions: Slice): ResultPointerInt
    fun DefinitionRegion_from_bounding_boxes(boxes: Slice): ResultPointerInt
    fun DefinitionRegion_add_bounds(handle: Pointer, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Unit
    fun DefinitionRegion_add_point(handle: Pointer, x: Int, y: Int, z: Int): Unit
    fun DefinitionRegion_set_metadata(handle: Pointer, key: Slice, value: Slice): ResultUnitInt
    fun DefinitionRegion_get_metadata(handle: Pointer, key: Slice, write: Pointer): ResultUnitInt
    fun DefinitionRegion_all_metadata_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun DefinitionRegion_metadata_keys_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun DefinitionRegion_add_filter(handle: Pointer, filter: Slice): ResultUnitInt
    fun DefinitionRegion_is_empty(handle: Pointer): Byte
    fun DefinitionRegion_volume(handle: Pointer): FFIUint64
    fun DefinitionRegion_contains(handle: Pointer, x: Int, y: Int, z: Int): Byte
    fun DefinitionRegion_shift(handle: Pointer, dx: Int, dy: Int, dz: Int): Unit
    fun DefinitionRegion_expand(handle: Pointer, x: Int, y: Int, z: Int): Unit
    fun DefinitionRegion_contract(handle: Pointer, amount: Int): Unit
    fun DefinitionRegion_intersected(handle: Pointer, other: Pointer): Pointer
    fun DefinitionRegion_union_with(handle: Pointer, other: Pointer): Pointer
    fun DefinitionRegion_subtracted(handle: Pointer, other: Pointer): Pointer
    fun DefinitionRegion_merge(handle: Pointer, other: Pointer): Unit
    fun DefinitionRegion_union_into(handle: Pointer, other: Pointer): Unit
    fun DefinitionRegion_bounds(handle: Pointer): ResultRegionBoundsNativeInt
    fun DefinitionRegion_dimensions(handle: Pointer): DimensionsNative
    fun DefinitionRegion_center(handle: Pointer): ResultBlockPosNativeInt
    fun DefinitionRegion_center_f32_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun DefinitionRegion_positions_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun DefinitionRegion_positions_sorted_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun DefinitionRegion_box_count(handle: Pointer): FFIUint32
    fun DefinitionRegion_get_box(handle: Pointer, index: FFIUint32): ResultRegionBoundsNativeInt
    fun DefinitionRegion_boxes_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun DefinitionRegion_is_contiguous(handle: Pointer): Byte
    fun DefinitionRegion_connected_components(handle: Pointer): FFIUint32
    fun DefinitionRegion_simplify(handle: Pointer): Unit
    fun DefinitionRegion_filter_by_block(handle: Pointer, schematic: Pointer, blockName: Slice): ResultPointerInt
    fun DefinitionRegion_filter_by_properties(handle: Pointer, schematic: Pointer, propertiesJson: Slice): ResultPointerInt
    fun DefinitionRegion_exclude_block(handle: Pointer, schematic: Pointer, blockName: Slice): ResultUnitInt
    fun DefinitionRegion_intersects_bounds(handle: Pointer, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Byte
    fun DefinitionRegion_shifted(handle: Pointer, dx: Int, dy: Int, dz: Int): Pointer
    fun DefinitionRegion_expanded(handle: Pointer, x: Int, y: Int, z: Int): Pointer
    fun DefinitionRegion_contracted(handle: Pointer, amount: Int): Pointer
    fun DefinitionRegion_copy(handle: Pointer): Pointer
    fun DefinitionRegion_set_color(handle: Pointer, color: FFIUint32): Unit
    fun DefinitionRegion_blocks_json(handle: Pointer, schematic: Pointer, write: Pointer): ResultUnitInt
    fun DefinitionRegion_sync(handle: Pointer, schematic: Pointer, name: Slice): ResultUnitInt
}
/** A named sub-volume of a schematic: a union of boxes plus a metadata map.
*/
class DefinitionRegion internal constructor (
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

    private class DefinitionRegionCleaner(val handle: Pointer, val lib: DefinitionRegionLib) : Runnable {
        override fun run() {
            lib.DefinitionRegion_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, DefinitionRegion.DefinitionRegionCleaner(handle, DefinitionRegion.lib));
    }

    companion object {
        internal val libClass: Class<DefinitionRegionLib> = DefinitionRegionLib::class.java
        internal val lib: DefinitionRegionLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(): DefinitionRegion {
            
            val returnVal = lib.DefinitionRegion_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = DefinitionRegion(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun fromBounds(minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): DefinitionRegion {
            
            val returnVal = lib.DefinitionRegion_from_bounds(minX, minY, minZ, maxX, maxY, maxZ);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = DefinitionRegion(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Build a region from single-block positions crossing as flat `[i32]`
        *chunked in threes (PORTING rule 7). Errors with `InvalidArgument` if
        *the length is not a multiple of 3.
        */
        fun fromPositions(positions: IntArray): Result<DefinitionRegion> {
            val positionsSliceMemory = PrimitiveArrayTools.borrow(positions)
            
            val returnVal = lib.DefinitionRegion_from_positions(positionsSliceMemory.slice);
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
                positionsSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Build a region from bounding boxes crossing as flat `[i32]` chunked in
        *sixes (`min_x, min_y, min_z, max_x, max_y, max_z` per box). Errors
        *with `InvalidArgument` if the length is not a multiple of 6.
        */
        fun fromBoundingBoxes(boxes: IntArray): Result<DefinitionRegion> {
            val boxesSliceMemory = PrimitiveArrayTools.borrow(boxes)
            
            val returnVal = lib.DefinitionRegion_from_bounding_boxes(boxesSliceMemory.slice);
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
                boxesSliceMemory.close()
            }
        }
    }
    
    fun addBounds(minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Unit {
        
        val returnVal = lib.DefinitionRegion_add_bounds(handle, minX, minY, minZ, maxX, maxY, maxZ);
        
    }
    
    fun addPoint(x: Int, y: Int, z: Int): Unit {
        
        val returnVal = lib.DefinitionRegion_add_point(handle, x, y, z);
        
    }
    
    fun setMetadata(key: String, value: String): Result<Unit> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        val valueSliceMemory = PrimitiveArrayTools.borrowUtf8(value)
        
        val returnVal = lib.DefinitionRegion_set_metadata(handle, keySliceMemory.slice, valueSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
            valueSliceMemory.close()
        }
    }
    
    /** Errors with `NotFound` when the key is absent.
    */
    fun getMetadata(key: String): Result<String> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.DefinitionRegion_get_metadata(handle, keySliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
        }
    }
    
    /** The full metadata map, written as a JSON object string (the old ABI
    *returned an array of `"key=value"` strings).
    */
    fun allMetadataJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.DefinitionRegion_all_metadata_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The metadata keys, written as a JSON array string.
    */
    fun metadataKeysJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.DefinitionRegion_metadata_keys_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Store a filter expression in the region's metadata under the `filter`
    *key.
    */
    fun addFilter(filter: String): Result<Unit> {
        val filterSliceMemory = PrimitiveArrayTools.borrowUtf8(filter)
        
        val returnVal = lib.DefinitionRegion_add_filter(handle, filterSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            filterSliceMemory.close()
        }
    }
    
    fun isEmpty(): Boolean {
        
        val returnVal = lib.DefinitionRegion_is_empty(handle);
        return (returnVal > 0)
    }
    
    fun volume(): ULong {
        
        val returnVal = lib.DefinitionRegion_volume(handle);
        return (returnVal.toULong())
    }
    
    fun contains(x: Int, y: Int, z: Int): Boolean {
        
        val returnVal = lib.DefinitionRegion_contains(handle, x, y, z);
        return (returnVal > 0)
    }
    
    fun shift(dx: Int, dy: Int, dz: Int): Unit {
        
        val returnVal = lib.DefinitionRegion_shift(handle, dx, dy, dz);
        
    }
    
    fun expand(x: Int, y: Int, z: Int): Unit {
        
        val returnVal = lib.DefinitionRegion_expand(handle, x, y, z);
        
    }
    
    fun contract(amount: Int): Unit {
        
        val returnVal = lib.DefinitionRegion_contract(handle, amount);
        
    }
    
    /** A new region: the intersection of `self` and `other`.
    */
    fun intersected(other: DefinitionRegion): DefinitionRegion {
        
        val returnVal = lib.DefinitionRegion_intersected(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = DefinitionRegion(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** A new region: the union of `self` and `other`.
    */
    fun unionWith(other: DefinitionRegion): DefinitionRegion {
        
        val returnVal = lib.DefinitionRegion_union_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = DefinitionRegion(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** A new region: `self` minus `other`.
    */
    fun subtracted(other: DefinitionRegion): DefinitionRegion {
        
        val returnVal = lib.DefinitionRegion_subtracted(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = DefinitionRegion(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** Merge `other`'s boxes and metadata into `self`.
    */
    fun merge(other: DefinitionRegion): Unit {
        
        val returnVal = lib.DefinitionRegion_merge(handle, other.handle);
        
    }
    
    /** Union `other`'s boxes into `self` in place.
    */
    fun unionInto(other: DefinitionRegion): Unit {
        
        val returnVal = lib.DefinitionRegion_union_into(handle, other.handle);
        
    }
    
    /** The overall bounding box. Errors with `NotFound` when the region is
    *empty.
    */
    fun bounds(): Result<RegionBounds> {
        
        val returnVal = lib.DefinitionRegion_bounds(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = RegionBounds.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun dimensions(): Dimensions {
        
        val returnVal = lib.DefinitionRegion_dimensions(handle);
        val returnStruct = Dimensions.fromNative(returnVal)
        return returnStruct
    }
    
    /** The center block position. Errors with `NotFound` when the region is
    *empty.
    */
    fun center(): Result<BlockPos> {
        
        val returnVal = lib.DefinitionRegion_center(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = BlockPos.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The exact (fractional) center, written as a JSON `[x, y, z]` array of
    *floats. Errors with `NotFound` when the region is empty.
    */
    fun centerF32Json(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.DefinitionRegion_center_f32_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Every contained position, written as a flat JSON array of ints
    *(`[x0, y0, z0, x1, y1, z1, …]`), deduplicated, in box order.
    */
    fun positionsJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.DefinitionRegion_positions_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Every contained position in sorted (y, z, x) order, written as a flat
    *JSON array of ints.
    */
    fun positionsSortedJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.DefinitionRegion_positions_sorted_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The number of boxes making up this region.
    */
    fun boxCount(): UInt {
        
        val returnVal = lib.DefinitionRegion_box_count(handle);
        return (returnVal.toUInt())
    }
    
    /** The box at `index`. Errors with `NotFound` when out of range.
    */
    fun getBox(index: UInt): Result<RegionBounds> {
        
        val returnVal = lib.DefinitionRegion_get_box(handle, FFIUint32(index));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = RegionBounds.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Every box, written as a flat JSON array of ints (six ints per box:
    *`min_x, min_y, min_z, max_x, max_y, max_z`).
    */
    fun boxesJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.DefinitionRegion_boxes_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun isContiguous(): Boolean {
        
        val returnVal = lib.DefinitionRegion_is_contiguous(handle);
        return (returnVal > 0)
    }
    
    fun connectedComponents(): UInt {
        
        val returnVal = lib.DefinitionRegion_connected_components(handle);
        return (returnVal.toUInt())
    }
    
    /** Merge overlapping/adjacent boxes into a minimal representation.
    */
    fun simplify(): Unit {
        
        val returnVal = lib.DefinitionRegion_simplify(handle);
        
    }
    
    /** A new region containing only the positions where `schematic` has a
    *block named `block_name`.
    */
    fun filterByBlock(schematic: Schematic, blockName: String): Result<DefinitionRegion> {
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        
        val returnVal = lib.DefinitionRegion_filter_by_block(handle, schematic.handle, blockNameSliceMemory.slice);
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
            blockNameSliceMemory.close()
        }
    }
    
    /** A new region containing only the positions where the block in
    *`schematic` matches every property in `properties_json` (a JSON
    *object of property name → value strings).
    */
    fun filterByProperties(schematic: Schematic, propertiesJson: String): Result<DefinitionRegion> {
        val propertiesJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(propertiesJson)
        
        val returnVal = lib.DefinitionRegion_filter_by_properties(handle, schematic.handle, propertiesJsonSliceMemory.slice);
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
            propertiesJsonSliceMemory.close()
        }
    }
    
    /** Remove every position where `schematic` has a block named
    *`block_name` (in place).
    */
    fun excludeBlock(schematic: Schematic, blockName: String): Result<Unit> {
        val blockNameSliceMemory = PrimitiveArrayTools.borrowUtf8(blockName)
        
        val returnVal = lib.DefinitionRegion_exclude_block(handle, schematic.handle, blockNameSliceMemory.slice);
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
    
    fun intersectsBounds(minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Boolean {
        
        val returnVal = lib.DefinitionRegion_intersects_bounds(handle, minX, minY, minZ, maxX, maxY, maxZ);
        return (returnVal > 0)
    }
    
    /** A new region shifted by (`dx`, `dy`, `dz`).
    */
    fun shifted(dx: Int, dy: Int, dz: Int): DefinitionRegion {
        
        val returnVal = lib.DefinitionRegion_shifted(handle, dx, dy, dz);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = DefinitionRegion(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** A new region expanded by (`x`, `y`, `z`) on each axis.
    */
    fun expanded(x: Int, y: Int, z: Int): DefinitionRegion {
        
        val returnVal = lib.DefinitionRegion_expanded(handle, x, y, z);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = DefinitionRegion(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** A new region contracted by `amount` on every axis.
    */
    fun contracted(amount: Int): DefinitionRegion {
        
        val returnVal = lib.DefinitionRegion_contracted(handle, amount);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = DefinitionRegion(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** A deep copy of this region.
    */
    fun copy(): DefinitionRegion {
        
        val returnVal = lib.DefinitionRegion_copy(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = DefinitionRegion(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** Store a display color (`0xRRGGBB`) in the region's metadata.
    */
    fun setColor(color: UInt): Unit {
        
        val returnVal = lib.DefinitionRegion_set_color(handle, FFIUint32(color));
        
    }
    
    /** The blocks of `schematic` inside this region, written as a JSON array
    *of `{"x", "y", "z", "name", "properties"}` objects (the old ABI
    *returned a `CBlockArray`).
    */
    fun blocksJson(schematic: Schematic): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.DefinitionRegion_blocks_json(handle, schematic.handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Write this region into `schematic`'s definition-region map under
    *`name` (insert or overwrite).
    */
    fun sync(schematic: Schematic, name: String): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        
        val returnVal = lib.DefinitionRegion_sync(handle, schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, nameSliceMemory.slice);
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