package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface TypedCircuitExecutorLib: Library {
    fun TypedCircuitExecutor_destroy(handle: Pointer)
    fun TypedCircuitExecutor_from_layout(world: Pointer, layout: Pointer): ResultPointerInt
    fun TypedCircuitExecutor_from_layout_with_options(world: Pointer, layout: Pointer, optimize: Boolean, ioOnly: Boolean): ResultPointerInt
    fun TypedCircuitExecutor_from_insign(schematic: Pointer): ResultPointerInt
    fun TypedCircuitExecutor_from_insign_with_options(schematic: Pointer, optimize: Boolean, ioOnly: Boolean): ResultPointerInt
    fun TypedCircuitExecutor_set_state_mode(handle: Pointer, mode: Slice): ResultUnitInt
    fun TypedCircuitExecutor_reset(handle: Pointer): ResultUnitInt
    fun TypedCircuitExecutor_tick(handle: Pointer, ticks: FFIUint32): Unit
    fun TypedCircuitExecutor_flush(handle: Pointer): Unit
    fun TypedCircuitExecutor_set_input(handle: Pointer, name: Slice, value: Pointer): ResultUnitInt
    fun TypedCircuitExecutor_read_output(handle: Pointer, name: Slice): ResultPointerInt
    fun TypedCircuitExecutor_execute(handle: Pointer, inputsJson: Slice, mode: Pointer, write: Pointer): ResultUnitInt
    fun TypedCircuitExecutor_input_names_json(handle: Pointer, write: Pointer): Unit
    fun TypedCircuitExecutor_output_names_json(handle: Pointer, write: Pointer): Unit
    fun TypedCircuitExecutor_layout_info_json(handle: Pointer, write: Pointer): Unit
    fun TypedCircuitExecutor_sync_to_schematic(handle: Pointer): Pointer
}
/** A typed circuit executor. Wraps
*[crate::simulation::typed_executor::TypedCircuitExecutor].
*/
class TypedCircuitExecutor internal constructor (
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

    private class TypedCircuitExecutorCleaner(val handle: Pointer, val lib: TypedCircuitExecutorLib) : Runnable {
        override fun run() {
            lib.TypedCircuitExecutor_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, TypedCircuitExecutor.TypedCircuitExecutorCleaner(handle, TypedCircuitExecutor.lib));
    }

    companion object {
        internal val libClass: Class<TypedCircuitExecutorLib> = TypedCircuitExecutorLib::class.java
        internal val lib: TypedCircuitExecutorLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Create from a world and layout. Builds a fresh world from the
        *world's schematic (matches the old ABI, which cloned internally).
        */
        fun fromLayout(world: MchprsWorld, layout: IoLayout): Result<TypedCircuitExecutor> {
            
            val returnVal = lib.TypedCircuitExecutor_from_layout(world.handle, layout.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = TypedCircuitExecutor(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** Create from a world and layout with simulation options.
        */
        fun fromLayoutWithOptions(world: MchprsWorld, layout: IoLayout, optimize: Boolean, ioOnly: Boolean): Result<TypedCircuitExecutor> {
            
            val returnVal = lib.TypedCircuitExecutor_from_layout_with_options(world.handle, layout.handle, optimize, ioOnly);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = TypedCircuitExecutor(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** Create from Insign annotations in a schematic.
        */
        fun fromInsign(schematic: Schematic): Result<TypedCircuitExecutor> {
            
            val returnVal = lib.TypedCircuitExecutor_from_insign(schematic.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = TypedCircuitExecutor(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** Create from Insign annotations with options.
        */
        fun fromInsignWithOptions(schematic: Schematic, optimize: Boolean, ioOnly: Boolean): Result<TypedCircuitExecutor> {
            
            val returnVal = lib.TypedCircuitExecutor_from_insign_with_options(schematic.handle, optimize, ioOnly);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = TypedCircuitExecutor(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }
    
    /** Set the state mode ("stateless" | "stateful" | "manual").
    */
    fun setStateMode(mode: String): Result<Unit> {
        val modeSliceMemory = PrimitiveArrayTools.borrowUtf8(mode)
        
        val returnVal = lib.TypedCircuitExecutor_set_state_mode(handle, modeSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            modeSliceMemory.close()
        }
    }
    
    /** Reset the executor to its initial state.
    */
    fun reset(): Result<Unit> {
        
        val returnVal = lib.TypedCircuitExecutor_reset(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Advance the simulation by `ticks` ticks.
    */
    fun tick(ticks: UInt): Unit {
        
        val returnVal = lib.TypedCircuitExecutor_tick(handle, FFIUint32(ticks));
        
    }
    
    /** Flush pending changes.
    */
    fun flush(): Unit {
        
        val returnVal = lib.TypedCircuitExecutor_flush(handle);
        
    }
    
    /** Set a single input value.
    */
    fun setInput(name: String, value: Value): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        
        val returnVal = lib.TypedCircuitExecutor_set_input(handle, nameSliceMemory.slice, value.handle);
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
    
    /** Read a single output value.
    */
    fun readOutput(name: String): Result<Value> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        
        val returnVal = lib.TypedCircuitExecutor_read_output(handle, nameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = Value(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }
    
    /** Execute the circuit. `inputs_json` is a JSON object like
    *`{"input_name": {"type": "u32", "value": 42}}`; writes a JSON
    *object with `outputs`, `ticks_elapsed` and `condition_met`.
    */
    fun execute(inputsJson: String, mode: ExecutionMode): Result<String> {
        val inputsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(inputsJson)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TypedCircuitExecutor_execute(handle, inputsJsonSliceMemory.slice, mode.handle, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            inputsJsonSliceMemory.close()
        }
    }
    
    /** Input names as a JSON array string.
    */
    fun inputNamesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TypedCircuitExecutor_input_names_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Output names as a JSON array string.
    */
    fun outputNamesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TypedCircuitExecutor_output_names_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Layout info as a JSON object string
    *(old ABI: `typed_executor_get_layout_info`).
    */
    fun layoutInfoJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TypedCircuitExecutor_layout_info_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Sync the simulation state and return the schematic (cloned).
    */
    fun syncToSchematic(): Schematic {
        
        val returnVal = lib.TypedCircuitExecutor_sync_to_schematic(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal 
        val returnOpaque = Schematic(handle, selfEdges, true)
        return returnOpaque
    }

}