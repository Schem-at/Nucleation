package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface BuildAnimationLib: Library {
    fun BuildAnimation_destroy(handle: Pointer)
    fun BuildAnimation_create(name: Slice): Pointer
    fun BuildAnimation_set_default_effect(handle: Pointer, effect: Pointer): Unit
    fun BuildAnimation_with_effect(handle: Pointer, effect: Pointer): Pointer
    fun BuildAnimation_set_step_ms(handle: Pointer, stepMs: Float): Unit
    fun BuildAnimation_set_stagger_total_ms(handle: Pointer, totalMs: Float): Unit
    fun BuildAnimation_clear_stagger(handle: Pointer): Unit
    fun BuildAnimation_set_stagger_offset_ms(handle: Pointer, offsetMs: Float): Unit
    fun BuildAnimation_set_loop_period_ms(handle: Pointer, periodMs: Float): ResultUnitInt
    fun BuildAnimation_clear_loop_period(handle: Pointer): Unit
    fun BuildAnimation_begin_group(handle: Pointer): ResultUnitInt
    fun BuildAnimation_begin_keyed_group(handle: Pointer, key: Float): ResultUnitInt
    fun BuildAnimation_end_group(handle: Pointer): ResultFFIUint32Int
    fun BuildAnimation_set_block(handle: Pointer, x: Int, y: Int, z: Int, block: Slice): ResultFFIUint32Int
    fun BuildAnimation_create_region(handle: Pointer, name: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultUnitInt
    fun BuildAnimation_set_block_in_region(handle: Pointer, region: Slice, x: Int, y: Int, z: Int, block: Slice): ResultFFIUint32Int
    fun BuildAnimation_translate(handle: Pointer, x: Int, y: Int, z: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_translate_region(handle: Pointer, region: Slice, x: Int, y: Int, z: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_translate_all(handle: Pointer, x: Int, y: Int, z: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_rotate_x(handle: Pointer, degrees: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_rotate_y(handle: Pointer, degrees: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_rotate_z(handle: Pointer, degrees: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_rotate_region_x(handle: Pointer, region: Slice, degrees: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_rotate_region_y(handle: Pointer, region: Slice, degrees: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_rotate_region_z(handle: Pointer, region: Slice, degrees: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_rotate_all_x(handle: Pointer, degrees: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_rotate_all_y(handle: Pointer, degrees: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_rotate_all_z(handle: Pointer, degrees: Int, durationMs: Float): ResultUnitInt
    fun BuildAnimation_flip_x(handle: Pointer, durationMs: Float): ResultUnitInt
    fun BuildAnimation_flip_y(handle: Pointer, durationMs: Float): ResultUnitInt
    fun BuildAnimation_flip_z(handle: Pointer, durationMs: Float): ResultUnitInt
    fun BuildAnimation_flip_region_x(handle: Pointer, region: Slice, durationMs: Float): ResultUnitInt
    fun BuildAnimation_flip_region_y(handle: Pointer, region: Slice, durationMs: Float): ResultUnitInt
    fun BuildAnimation_flip_region_z(handle: Pointer, region: Slice, durationMs: Float): ResultUnitInt
    fun BuildAnimation_flip_all_x(handle: Pointer, durationMs: Float): ResultUnitInt
    fun BuildAnimation_flip_all_y(handle: Pointer, durationMs: Float): ResultUnitInt
    fun BuildAnimation_flip_all_z(handle: Pointer, durationMs: Float): ResultUnitInt
    fun BuildAnimation_stamp_region(handle: Pointer, source: Pointer, region: Slice, x: Int, y: Int, z: Int, exclusions: Slice, durationMs: Float): ResultUnitInt
    fun BuildAnimation_stamp_box(handle: Pointer, source: Pointer, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int, x: Int, y: Int, z: Int, exclusions: Slice, durationMs: Float): ResultUnitInt
    fun BuildAnimation_set_operation_gizmos(handle: Pointer, enabled: Boolean): Unit
    fun BuildAnimation_operations_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun BuildAnimation_frame_json(handle: Pointer, timeMs: Float, write: Pointer): ResultUnitInt
    fun BuildAnimation_fill_along_parameter(handle: Pointer, shape: Pointer, brush: Pointer, groupCount: FFIUint32): ResultFFIUint32Int
    fun BuildAnimation_add_armor_stand(handle: Pointer, x: Double, y: Double, z: Double, yaw: Float, armorMaterial: Slice): ResultFFIUint32Int
    fun BuildAnimation_animate_camera(handle: Pointer, effect: Pointer, offsetMs: Float): Unit
    fun BuildAnimation_frame_count(handle: Pointer, fps: Double, holdMs: Float): FFIUint32
    fun BuildAnimation_render_gif(handle: Pointer, packZip: Slice, config: Pointer, path: Slice, fps: Double, holdMs: Float): ResultFFIUint32Int
    fun BuildAnimation_render_frames(handle: Pointer, packZip: Slice, config: Pointer, prefix: Slice, fps: Double, holdMs: Float): ResultFFIUint32Int
    fun BuildAnimation_save_to_file(handle: Pointer, path: Slice): ResultUnitInt
    fun BuildAnimation_group_count(handle: Pointer): FFIUint32
    fun BuildAnimation_duration_ms(handle: Pointer): Float
}
/** A schematic wrapper that records construction calls as animation targets.
*/
class BuildAnimation internal constructor (
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

    private class BuildAnimationCleaner(val handle: Pointer, val lib: BuildAnimationLib) : Runnable {
        override fun run() {
            lib.BuildAnimation_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, BuildAnimation.BuildAnimationCleaner(handle, BuildAnimation.lib));
    }

    companion object {
        internal val libClass: Class<BuildAnimationLib> = BuildAnimationLib::class.java
        internal val lib: BuildAnimationLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(name: String): BuildAnimation {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.BuildAnimation_create(nameSliceMemory.slice);
            try {
                val selfEdges: List<Any> = listOf()
                val handle = returnVal 
                val returnOpaque = BuildAnimation(handle, selfEdges, true)
                return returnOpaque
            } finally {
                nameSliceMemory.close()
            }
        }
    }
    
    fun setDefaultEffect(effect: AnimationEffect): Unit {
        
        val returnVal = lib.BuildAnimation_set_default_effect(handle, effect.handle);
        
    }
    
    /** Apply an effect to exactly the next recorded operation or explicit group.
    *The returned borrowed builder enables fluent calls in every generated binding.
    */
    fun withEffect(effect: AnimationEffect): BuildAnimation {
        // This lifetime edge depends on lifetimes: 'a
        val aEdges: MutableList<Any> = mutableListOf(this);
        
        val returnVal = lib.BuildAnimation_with_effect(handle, effect.handle);
        val selfEdges: List<Any> = listOf(this)
        val handle = returnVal 
        val returnOpaque = BuildAnimation(handle, selfEdges, false)
        return returnOpaque
    }
    
    fun setStepMs(stepMs: Float): Unit {
        
        val returnVal = lib.BuildAnimation_set_step_ms(handle, stepMs);
        
    }
    
    fun setStaggerTotalMs(totalMs: Float): Unit {
        
        val returnVal = lib.BuildAnimation_set_stagger_total_ms(handle, totalMs);
        
    }
    
    fun clearStagger(): Unit {
        
        val returnVal = lib.BuildAnimation_clear_stagger(handle);
        
    }
    
    /** Shift every construction group's start time. Negative offsets let a
    *repeating staggered effect cross the beginning of a loop capture.
    */
    fun setStaggerOffsetMs(offsetMs: Float): Unit {
        
        val returnVal = lib.BuildAnimation_set_stagger_offset_ms(handle, offsetMs);
        
    }
    
    /** Capture exactly one loop period, excluding the duplicate endpoint.
    *The rounded frame count evenly partitions the complete period.
    */
    fun setLoopPeriodMs(periodMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_set_loop_period_ms(handle, periodMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun clearLoopPeriod(): Unit {
        
        val returnVal = lib.BuildAnimation_clear_loop_period(handle);
        
    }
    
    fun beginGroup(): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_begin_group(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun beginKeyedGroup(key: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_begin_keyed_group(handle, key);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun endGroup(): Result<UInt> {
        
        val returnVal = lib.BuildAnimation_end_group(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal.toUInt()).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun setBlock(x: Int, y: Int, z: Int, block: String): Result<UInt> {
        val blockSliceMemory = PrimitiveArrayTools.borrowUtf8(block)
        
        val returnVal = lib.BuildAnimation_set_block(handle, x, y, z, blockSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal.toUInt()).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockSliceMemory.close()
        }
    }
    
    fun createRegion(name: String, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        
        val returnVal = lib.BuildAnimation_create_region(handle, nameSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
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
    
    fun setBlockInRegion(region: String, x: Int, y: Int, z: Int, block: String): Result<UInt> {
        val regionSliceMemory = PrimitiveArrayTools.borrowUtf8(region)
        val blockSliceMemory = PrimitiveArrayTools.borrowUtf8(block)
        
        val returnVal = lib.BuildAnimation_set_block_in_region(handle, regionSliceMemory.slice, x, y, z, blockSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal.toUInt()).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionSliceMemory.close()
            blockSliceMemory.close()
        }
    }
    
    fun translate(x: Int, y: Int, z: Int, durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_translate(handle, x, y, z, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun translateRegion(region: String, x: Int, y: Int, z: Int, durationMs: Float): Result<Unit> {
        val regionSliceMemory = PrimitiveArrayTools.borrowUtf8(region)
        
        val returnVal = lib.BuildAnimation_translate_region(handle, regionSliceMemory.slice, x, y, z, durationMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionSliceMemory.close()
        }
    }
    
    fun translateAll(x: Int, y: Int, z: Int, durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_translate_all(handle, x, y, z, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun rotateX(degrees: Int, durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_rotate_x(handle, degrees, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun rotateY(degrees: Int, durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_rotate_y(handle, degrees, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun rotateZ(degrees: Int, durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_rotate_z(handle, degrees, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun rotateRegionX(region: String, degrees: Int, durationMs: Float): Result<Unit> {
        val regionSliceMemory = PrimitiveArrayTools.borrowUtf8(region)
        
        val returnVal = lib.BuildAnimation_rotate_region_x(handle, regionSliceMemory.slice, degrees, durationMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionSliceMemory.close()
        }
    }
    
    fun rotateRegionY(region: String, degrees: Int, durationMs: Float): Result<Unit> {
        val regionSliceMemory = PrimitiveArrayTools.borrowUtf8(region)
        
        val returnVal = lib.BuildAnimation_rotate_region_y(handle, regionSliceMemory.slice, degrees, durationMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionSliceMemory.close()
        }
    }
    
    fun rotateRegionZ(region: String, degrees: Int, durationMs: Float): Result<Unit> {
        val regionSliceMemory = PrimitiveArrayTools.borrowUtf8(region)
        
        val returnVal = lib.BuildAnimation_rotate_region_z(handle, regionSliceMemory.slice, degrees, durationMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionSliceMemory.close()
        }
    }
    
    fun rotateAllX(degrees: Int, durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_rotate_all_x(handle, degrees, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun rotateAllY(degrees: Int, durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_rotate_all_y(handle, degrees, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun rotateAllZ(degrees: Int, durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_rotate_all_z(handle, degrees, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun flipX(durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_flip_x(handle, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun flipY(durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_flip_y(handle, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun flipZ(durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_flip_z(handle, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun flipRegionX(region: String, durationMs: Float): Result<Unit> {
        val regionSliceMemory = PrimitiveArrayTools.borrowUtf8(region)
        
        val returnVal = lib.BuildAnimation_flip_region_x(handle, regionSliceMemory.slice, durationMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionSliceMemory.close()
        }
    }
    
    fun flipRegionY(region: String, durationMs: Float): Result<Unit> {
        val regionSliceMemory = PrimitiveArrayTools.borrowUtf8(region)
        
        val returnVal = lib.BuildAnimation_flip_region_y(handle, regionSliceMemory.slice, durationMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionSliceMemory.close()
        }
    }
    
    fun flipRegionZ(region: String, durationMs: Float): Result<Unit> {
        val regionSliceMemory = PrimitiveArrayTools.borrowUtf8(region)
        
        val returnVal = lib.BuildAnimation_flip_region_z(handle, regionSliceMemory.slice, durationMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionSliceMemory.close()
        }
    }
    
    fun flipAllX(durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_flip_all_x(handle, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun flipAllY(durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_flip_all_y(handle, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun flipAllZ(durationMs: Float): Result<Unit> {
        
        val returnVal = lib.BuildAnimation_flip_all_z(handle, durationMs);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun stampRegion(source: Schematic, region: String, x: Int, y: Int, z: Int, exclusions: String, durationMs: Float): Result<Unit> {
        val regionSliceMemory = PrimitiveArrayTools.borrowUtf8(region)
        val exclusionsSliceMemory = PrimitiveArrayTools.borrowUtf8(exclusions)
        
        val returnVal = lib.BuildAnimation_stamp_region(handle, source.handle, regionSliceMemory.slice, x, y, z, exclusionsSliceMemory.slice, durationMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionSliceMemory.close()
            exclusionsSliceMemory.close()
        }
    }
    
    fun stampBox(source: Schematic, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int, x: Int, y: Int, z: Int, exclusions: String, durationMs: Float): Result<Unit> {
        val exclusionsSliceMemory = PrimitiveArrayTools.borrowUtf8(exclusions)
        
        val returnVal = lib.BuildAnimation_stamp_box(handle, source.handle, minX, minY, minZ, maxX, maxY, maxZ, x, y, z, exclusionsSliceMemory.slice, durationMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            exclusionsSliceMemory.close()
        }
    }
    
    fun setOperationGizmos(enabled: Boolean): Unit {
        
        val returnVal = lib.BuildAnimation_set_operation_gizmos(handle, enabled);
        
    }
    
    fun operationsJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.BuildAnimation_operations_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun frameJson(timeMs: Float): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.BuildAnimation_frame_json(handle, timeMs, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Fill a parametric shape and record its voxels as ordered groups in
    *the same transactional construction operation.
    */
    fun fillAlongParameter(shape: Shape, brush: Brush, groupCount: UInt): Result<UInt> {
        
        val returnVal = lib.BuildAnimation_fill_along_parameter(handle, shape.handle, brush.handle, FFIUint32(groupCount));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal.toUInt()).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun addArmorStand(x: Double, y: Double, z: Double, yaw: Float, armorMaterial: String): Result<UInt> {
        val armorMaterialSliceMemory = PrimitiveArrayTools.borrowUtf8(armorMaterial)
        
        val returnVal = lib.BuildAnimation_add_armor_stand(handle, x, y, z, yaw, armorMaterialSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal.toUInt()).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            armorMaterialSliceMemory.close()
        }
    }
    
    fun animateCamera(effect: AnimationEffect, offsetMs: Float): Unit {
        
        val returnVal = lib.BuildAnimation_animate_camera(handle, effect.handle, offsetMs);
        
    }
    
    fun frameCount(fps: Double, holdMs: Float): UInt {
        
        val returnVal = lib.BuildAnimation_frame_count(handle, fps, holdMs);
        return (returnVal.toUInt())
    }
    
    /** Render directly to a looping GIF. The renderer, meshes, timeline and
    *GIF encoder all live in the Rust core; no ffmpeg subprocess is needed.
    */
    fun renderGif(packZip: UByteArray, config: RenderConfig, path: String, fps: Double, holdMs: Float): Result<UInt> {
        val packZipSliceMemory = PrimitiveArrayTools.borrow(packZip)
        val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
        
        val returnVal = lib.BuildAnimation_render_gif(handle, packZipSliceMemory.slice, config.handle, pathSliceMemory.slice, fps, holdMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal.toUInt()).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            packZipSliceMemory.close()
            pathSliceMemory.close()
        }
    }
    
    /** Render numbered PNG frames (`prefix0000.png`, ...) for an external
    *compositor while using the exact same public timeline API.
    */
    fun renderFrames(packZip: UByteArray, config: RenderConfig, prefix: String, fps: Double, holdMs: Float): Result<UInt> {
        val packZipSliceMemory = PrimitiveArrayTools.borrow(packZip)
        val prefixSliceMemory = PrimitiveArrayTools.borrowUtf8(prefix)
        
        val returnVal = lib.BuildAnimation_render_frames(handle, packZipSliceMemory.slice, config.handle, prefixSliceMemory.slice, fps, holdMs);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal.toUInt()).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            packZipSliceMemory.close()
            prefixSliceMemory.close()
        }
    }
    
    fun saveToFile(path: String): Result<Unit> {
        val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
        
        val returnVal = lib.BuildAnimation_save_to_file(handle, pathSliceMemory.slice);
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
    
    fun groupCount(): UInt {
        
        val returnVal = lib.BuildAnimation_group_count(handle);
        return (returnVal.toUInt())
    }
    
    fun durationMs(): Float {
        
        val returnVal = lib.BuildAnimation_duration_ms(handle);
        return (returnVal)
    }

}