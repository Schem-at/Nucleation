package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface CircuitBuilderLib: Library {
    fun CircuitBuilder_destroy(handle: Pointer)
    fun CircuitBuilder_create(schematic: Pointer): Pointer
    fun CircuitBuilder_from_insign(schematic: Pointer): ResultPointerInt
    fun CircuitBuilder_with_input(handle: Pointer, name: Slice, ioType: Pointer, layout: Pointer, regionPositions: Slice): ResultUnitInt
    fun CircuitBuilder_with_input_sorted(handle: Pointer, name: Slice, ioType: Pointer, layout: Pointer, regionPositions: Slice, sort: Pointer): ResultUnitInt
    fun CircuitBuilder_with_input_auto(handle: Pointer, name: Slice, ioType: Pointer, regionPositions: Slice): ResultUnitInt
    fun CircuitBuilder_with_input_auto_sorted(handle: Pointer, name: Slice, ioType: Pointer, regionPositions: Slice, sort: Pointer): ResultUnitInt
    fun CircuitBuilder_with_output(handle: Pointer, name: Slice, ioType: Pointer, layout: Pointer, regionPositions: Slice): ResultUnitInt
    fun CircuitBuilder_with_output_sorted(handle: Pointer, name: Slice, ioType: Pointer, layout: Pointer, regionPositions: Slice, sort: Pointer): ResultUnitInt
    fun CircuitBuilder_with_output_auto(handle: Pointer, name: Slice, ioType: Pointer, regionPositions: Slice): ResultUnitInt
    fun CircuitBuilder_with_output_auto_sorted(handle: Pointer, name: Slice, ioType: Pointer, regionPositions: Slice, sort: Pointer): ResultUnitInt
    fun CircuitBuilder_with_options(handle: Pointer, optimize: Boolean, ioOnly: Boolean): ResultUnitInt
    fun CircuitBuilder_with_state_mode(handle: Pointer, mode: Slice): ResultUnitInt
    fun CircuitBuilder_validate(handle: Pointer): ResultUnitInt
    fun CircuitBuilder_build(handle: Pointer): ResultPointerInt
    fun CircuitBuilder_build_validated(handle: Pointer): ResultPointerInt
    fun CircuitBuilder_input_count(handle: Pointer): FFIUint32
    fun CircuitBuilder_output_count(handle: Pointer): FFIUint32
    fun CircuitBuilder_input_names_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun CircuitBuilder_output_names_json(handle: Pointer, write: Pointer): ResultUnitInt
}
/** High-level circuit builder. `build`/`build_validated` consume it
*(PORTING rule 11). Regions are given as flat `[x,y,z,...]` positions
*(see module notes).
*/
class CircuitBuilder internal constructor (
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

    private class CircuitBuilderCleaner(val handle: Pointer, val lib: CircuitBuilderLib) : Runnable {
        override fun run() {
            lib.CircuitBuilder_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, CircuitBuilder.CircuitBuilderCleaner(handle, CircuitBuilder.lib));
    }

    companion object {
        internal val libClass: Class<CircuitBuilderLib> = CircuitBuilderLib::class.java
        internal val lib: CircuitBuilderLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(schematic: Schematic): CircuitBuilder {
            
            val returnVal = lib.CircuitBuilder_create(schematic.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = CircuitBuilder(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Create a builder pre-populated from Insign annotations.
        */
        fun fromInsign(schematic: Schematic): Result<CircuitBuilder> {
            
            val returnVal = lib.CircuitBuilder_from_insign(schematic.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = CircuitBuilder(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }
    
    /** Add an input with full control.
    */
    fun withInput(name: String, ioType: IoType, layout: LayoutFunction, regionPositions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.CircuitBuilder_with_input(handle, nameSliceMemory.slice, ioType.handle, layout.handle, regionPositionsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            regionPositionsSliceMemory.close()
        }
    }
    
    /** Add an input with full control and a custom sort strategy.
    */
    fun withInputSorted(name: String, ioType: IoType, layout: LayoutFunction, regionPositions: IntArray, sort: SortStrategy): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.CircuitBuilder_with_input_sorted(handle, nameSliceMemory.slice, ioType.handle, layout.handle, regionPositionsSliceMemory.slice, sort.handle);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            regionPositionsSliceMemory.close()
        }
    }
    
    /** Add an input with automatic layout inference.
    */
    fun withInputAuto(name: String, ioType: IoType, regionPositions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.CircuitBuilder_with_input_auto(handle, nameSliceMemory.slice, ioType.handle, regionPositionsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            regionPositionsSliceMemory.close()
        }
    }
    
    /** Add an input with automatic layout inference and a custom sort.
    */
    fun withInputAutoSorted(name: String, ioType: IoType, regionPositions: IntArray, sort: SortStrategy): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.CircuitBuilder_with_input_auto_sorted(handle, nameSliceMemory.slice, ioType.handle, regionPositionsSliceMemory.slice, sort.handle);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            regionPositionsSliceMemory.close()
        }
    }
    
    /** Add an output with full control.
    */
    fun withOutput(name: String, ioType: IoType, layout: LayoutFunction, regionPositions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.CircuitBuilder_with_output(handle, nameSliceMemory.slice, ioType.handle, layout.handle, regionPositionsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            regionPositionsSliceMemory.close()
        }
    }
    
    /** Add an output with full control and a custom sort strategy.
    */
    fun withOutputSorted(name: String, ioType: IoType, layout: LayoutFunction, regionPositions: IntArray, sort: SortStrategy): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.CircuitBuilder_with_output_sorted(handle, nameSliceMemory.slice, ioType.handle, layout.handle, regionPositionsSliceMemory.slice, sort.handle);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            regionPositionsSliceMemory.close()
        }
    }
    
    /** Add an output with automatic layout inference.
    */
    fun withOutputAuto(name: String, ioType: IoType, regionPositions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.CircuitBuilder_with_output_auto(handle, nameSliceMemory.slice, ioType.handle, regionPositionsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            regionPositionsSliceMemory.close()
        }
    }
    
    /** Add an output with automatic layout inference and a custom sort.
    */
    fun withOutputAutoSorted(name: String, ioType: IoType, regionPositions: IntArray, sort: SortStrategy): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.CircuitBuilder_with_output_auto_sorted(handle, nameSliceMemory.slice, ioType.handle, regionPositionsSliceMemory.slice, sort.handle);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            regionPositionsSliceMemory.close()
        }
    }
    
    /** Set simulation options.
    */
    fun withOptions(optimize: Boolean, ioOnly: Boolean): Result<Unit> {
        
        val returnVal = lib.CircuitBuilder_with_options(handle, optimize, ioOnly);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Set the state mode ("stateless" | "stateful" | "manual").
    */
    fun withStateMode(mode: String): Result<Unit> {
        val modeSliceMemory = PrimitiveArrayTools.borrowUtf8(mode)
        
        val returnVal = lib.CircuitBuilder_with_state_mode(handle, modeSliceMemory.slice);
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
    
    /** Validate the configuration without consuming the builder.
    */
    fun validate(): Result<Unit> {
        
        val returnVal = lib.CircuitBuilder_validate(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Build the executor. Consumes the builder.
    */
    fun build(): Result<TypedCircuitExecutor> {
        
        val returnVal = lib.CircuitBuilder_build(handle);
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
    
    /** Build the executor with validation. Consumes the builder.
    */
    fun buildValidated(): Result<TypedCircuitExecutor> {
        
        val returnVal = lib.CircuitBuilder_build_validated(handle);
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
    
    fun inputCount(): UInt {
        
        val returnVal = lib.CircuitBuilder_input_count(handle);
        return (returnVal.toUInt())
    }
    
    fun outputCount(): UInt {
        
        val returnVal = lib.CircuitBuilder_output_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Input names as a JSON array string.
    */
    fun inputNamesJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.CircuitBuilder_input_names_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Output names as a JSON array string.
    */
    fun outputNamesJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.CircuitBuilder_output_names_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}