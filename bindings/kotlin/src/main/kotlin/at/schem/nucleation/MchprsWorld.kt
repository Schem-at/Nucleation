package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MchprsWorldLib: Library {
    fun MchprsWorld_destroy(handle: Pointer)
    fun MchprsWorld_create(schematic: Pointer): ResultPointerInt
    fun MchprsWorld_create_with_options(schematic: Pointer, optimize: Boolean, ioOnly: Boolean): ResultPointerInt
    fun MchprsWorld_create_with_custom_io(schematic: Pointer, optimize: Boolean, ioOnly: Boolean, customIoPositions: Slice): ResultPointerInt
    fun MchprsWorld_simulate_use_block(schematic: Pointer, ticks: FFIUint32, eventsXyz: Slice): ResultPointerInt
    fun MchprsWorld_tick(handle: Pointer, ticks: FFIUint32): Unit
    fun MchprsWorld_flush(handle: Pointer): Unit
    fun MchprsWorld_set_lever_power(handle: Pointer, x: Int, y: Int, z: Int, powered: Boolean): Unit
    fun MchprsWorld_get_lever_power(handle: Pointer, x: Int, y: Int, z: Int): Byte
    fun MchprsWorld_is_lit(handle: Pointer, x: Int, y: Int, z: Int): Byte
    fun MchprsWorld_set_signal_strength(handle: Pointer, x: Int, y: Int, z: Int, strength: FFIUint8): Unit
    fun MchprsWorld_get_signal_strength(handle: Pointer, x: Int, y: Int, z: Int): FFIUint8
    fun MchprsWorld_on_use_block(handle: Pointer, x: Int, y: Int, z: Int): Unit
    fun MchprsWorld_sync_to_schematic(handle: Pointer): Unit
    fun MchprsWorld_get_schematic(handle: Pointer): Pointer
    fun MchprsWorld_get_redstone_power(handle: Pointer, x: Int, y: Int, z: Int): FFIUint8
    fun MchprsWorld_check_custom_io_changes(handle: Pointer): Unit
    fun MchprsWorld_poll_custom_io_changes_json(handle: Pointer, write: Pointer): Unit
    fun MchprsWorld_peek_custom_io_changes_json(handle: Pointer, write: Pointer): Unit
    fun MchprsWorld_clear_custom_io_changes(handle: Pointer): Unit
    fun MchprsWorld_export_graph(handle: Pointer): ResultPointerInt
    fun MchprsWorld_export_graph_structural(handle: Pointer): ResultPointerInt
}
/** A running MCHPRS redstone simulation. Wraps [crate::simulation::MchprsWorld].
*/
class MchprsWorld internal constructor (
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

    private class MchprsWorldCleaner(val handle: Pointer, val lib: MchprsWorldLib) : Runnable {
        override fun run() {
            lib.MchprsWorld_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, MchprsWorld.MchprsWorldCleaner(handle, MchprsWorld.lib));
    }

    companion object {
        internal val libClass: Class<MchprsWorldLib> = MchprsWorldLib::class.java
        internal val lib: MchprsWorldLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Create a simulation world from a schematic with default options.
        */
        fun create(schematic: Schematic): Result<MchprsWorld> {
            
            val returnVal = lib.MchprsWorld_create(schematic.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = MchprsWorld(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** Create a simulation world with explicit options.
        */
        fun createWithOptions(schematic: Schematic, optimize: Boolean, ioOnly: Boolean): Result<MchprsWorld> {
            
            val returnVal = lib.MchprsWorld_create_with_options(schematic.handle, optimize, ioOnly);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = MchprsWorld(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** Create a simulation world with custom IO positions
        *(`custom_io_positions` is flat `[x,y,z, x,y,z, ...]`).
        */
        fun createWithCustomIo(schematic: Schematic, optimize: Boolean, ioOnly: Boolean, customIoPositions: IntArray): Result<MchprsWorld> {
            val customIoPositionsSliceMemory = PrimitiveArrayTools.borrow(customIoPositions)
            
            val returnVal = lib.MchprsWorld_create_with_custom_io(schematic.handle, optimize, ioOnly, customIoPositionsSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = MchprsWorld(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                customIoPositionsSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** One-shot convenience (old ABI: `schematic_simulate_use_block`):
        *build a world from `schematic`, fire an `on_use_block` event per
        *triple in `events_xyz`, run `ticks` ticks, and return the simulated
        *schematic. Unlike the old ABI (which mutated in place), this returns
        *a new `Schematic`.
        */
        fun simulateUseBlock(schematic: Schematic, ticks: UInt, eventsXyz: IntArray): Result<Schematic> {
            val eventsXyzSliceMemory = PrimitiveArrayTools.borrow(eventsXyz)
            
            val returnVal = lib.MchprsWorld_simulate_use_block(schematic.handle, FFIUint32(ticks), eventsXyzSliceMemory.slice);
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
                eventsXyzSliceMemory.close()
            }
        }
    }
    
    /** Advance the simulation by `ticks` ticks.
    */
    fun tick(ticks: UInt): Unit {
        
        val returnVal = lib.MchprsWorld_tick(handle, FFIUint32(ticks));
        
    }
    
    /** Flush pending changes from the compiler to the world.
    */
    fun flush(): Unit {
        
        val returnVal = lib.MchprsWorld_flush(handle);
        
    }
    
    /** Set the power state of a lever.
    */
    fun setLeverPower(x: Int, y: Int, z: Int, powered: Boolean): Unit {
        
        val returnVal = lib.MchprsWorld_set_lever_power(handle, x, y, z, powered);
        
    }
    
    /** Get the power state of a lever.
    */
    fun getLeverPower(x: Int, y: Int, z: Int): Boolean {
        
        val returnVal = lib.MchprsWorld_get_lever_power(handle, x, y, z);
        return (returnVal > 0)
    }
    
    /** Whether a redstone lamp is lit at the given position.
    */
    fun isLit(x: Int, y: Int, z: Int): Boolean {
        
        val returnVal = lib.MchprsWorld_is_lit(handle, x, y, z);
        return (returnVal > 0)
    }
    
    /** Set the signal strength (0-15) at a custom IO position.
    */
    fun setSignalStrength(x: Int, y: Int, z: Int, strength: UByte): Unit {
        
        val returnVal = lib.MchprsWorld_set_signal_strength(handle, x, y, z, FFIUint8(strength));
        
    }
    
    /** Get the signal strength (0-15) at a position.
    */
    fun getSignalStrength(x: Int, y: Int, z: Int): UByte {
        
        val returnVal = lib.MchprsWorld_get_signal_strength(handle, x, y, z);
        return (returnVal.toUByte())
    }
    
    /** Simulate a right-click on a block (typically a lever).
    */
    fun onUseBlock(x: Int, y: Int, z: Int): Unit {
        
        val returnVal = lib.MchprsWorld_on_use_block(handle, x, y, z);
        
    }
    
    /** Sync the simulation state back to the internal schematic.
    */
    fun syncToSchematic(): Unit {
        
        val returnVal = lib.MchprsWorld_sync_to_schematic(handle);
        
    }
    
    /** A clone of the world's schematic.
    */
    fun getSchematic(): Schematic {
        
        val returnVal = lib.MchprsWorld_get_schematic(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Schematic(handle, selfEdges, true)
        return returnOpaque
    }
    
    /** The redstone power level (0-15) at a position.
    */
    fun getRedstonePower(x: Int, y: Int, z: Int): UByte {
        
        val returnVal = lib.MchprsWorld_get_redstone_power(handle, x, y, z);
        return (returnVal.toUByte())
    }
    
    /** Check for custom IO changes since the last check. Call before
    *`poll_custom_io_changes_json`.
    */
    fun checkCustomIoChanges(): Unit {
        
        val returnVal = lib.MchprsWorld_check_custom_io_changes(handle);
        
    }
    
    /** Queued custom IO changes as a JSON array
    *(`[{"x":..,"y":..,"z":..,"old_power":..,"new_power":..}, ...]`),
    *clearing the queue.
    */
    fun pollCustomIoChangesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.MchprsWorld_poll_custom_io_changes_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Queued custom IO changes as JSON without clearing the queue.
    */
    fun peekCustomIoChangesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.MchprsWorld_peek_custom_io_changes_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Clear all queued custom IO changes.
    */
    fun clearCustomIoChanges(): Unit {
        
        val returnVal = lib.MchprsWorld_clear_custom_io_changes(handle);
        
    }
    
    /** Extract the compiled (post-optimization) redstone logic graph.
    */
    fun exportGraph(): Result<RedstoneGraph> {
        
        val returnVal = lib.MchprsWorld_export_graph(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal 
            val returnOpaque = RedstoneGraph(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Extract the structural (pre-fold, as-built) redstone logic graph.
    */
    fun exportGraphStructural(): Result<RedstoneGraph> {
        
        val returnVal = lib.MchprsWorld_export_graph_structural(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal 
            val returnOpaque = RedstoneGraph(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}