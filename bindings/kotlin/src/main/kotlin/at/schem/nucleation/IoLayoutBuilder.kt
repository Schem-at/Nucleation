package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface IoLayoutBuilderLib: Library {
    fun IoLayoutBuilder_destroy(handle: Pointer)
    fun IoLayoutBuilder_create(): Pointer
    fun IoLayoutBuilder_add_input(handle: Pointer, name: Slice, ioType: Pointer, layout: Pointer, positions: Slice): ResultUnitInt
    fun IoLayoutBuilder_add_output(handle: Pointer, name: Slice, ioType: Pointer, layout: Pointer, positions: Slice): ResultUnitInt
    fun IoLayoutBuilder_add_input_auto(handle: Pointer, name: Slice, ioType: Pointer, positions: Slice): ResultUnitInt
    fun IoLayoutBuilder_add_output_auto(handle: Pointer, name: Slice, ioType: Pointer, positions: Slice): ResultUnitInt
    fun IoLayoutBuilder_add_input_from_region(handle: Pointer, name: Slice, ioType: Pointer, layout: Pointer, regionPositions: Slice): ResultUnitInt
    fun IoLayoutBuilder_add_input_from_region_auto(handle: Pointer, name: Slice, ioType: Pointer, regionPositions: Slice): ResultUnitInt
    fun IoLayoutBuilder_add_output_from_region(handle: Pointer, name: Slice, ioType: Pointer, layout: Pointer, regionPositions: Slice): ResultUnitInt
    fun IoLayoutBuilder_add_output_from_region_auto(handle: Pointer, name: Slice, ioType: Pointer, regionPositions: Slice): ResultUnitInt
    fun IoLayoutBuilder_build(handle: Pointer): ResultPointerInt
}
/** Builder for an [IoLayout]. `build` consumes it (PORTING rule 11).
*/
class IoLayoutBuilder internal constructor (
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

    private class IoLayoutBuilderCleaner(val handle: Pointer, val lib: IoLayoutBuilderLib) : Runnable {
        override fun run() {
            lib.IoLayoutBuilder_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, IoLayoutBuilder.IoLayoutBuilderCleaner(handle, IoLayoutBuilder.lib));
    }

    companion object {
        internal val libClass: Class<IoLayoutBuilderLib> = IoLayoutBuilderLib::class.java
        internal val lib: IoLayoutBuilderLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Create an empty layout builder.
        */
        fun create(): IoLayoutBuilder {
            
            val returnVal = lib.IoLayoutBuilder_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = IoLayoutBuilder(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** Add an input. `positions` is flat `[x,y,z, x,y,z, ...]`.
    */
    fun addInput(name: String, ioType: IoType, layout: LayoutFunction, positions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val positionsSliceMemory = PrimitiveArrayTools.borrow(positions)
        
        val returnVal = lib.IoLayoutBuilder_add_input(handle, nameSliceMemory.slice, ioType.handle, layout.handle, positionsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            positionsSliceMemory.close()
        }
    }
    
    /** Add an output. `positions` is flat `[x,y,z, x,y,z, ...]`.
    */
    fun addOutput(name: String, ioType: IoType, layout: LayoutFunction, positions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val positionsSliceMemory = PrimitiveArrayTools.borrow(positions)
        
        val returnVal = lib.IoLayoutBuilder_add_output(handle, nameSliceMemory.slice, ioType.handle, layout.handle, positionsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            positionsSliceMemory.close()
        }
    }
    
    /** Add an input with automatic layout inference.
    */
    fun addInputAuto(name: String, ioType: IoType, positions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val positionsSliceMemory = PrimitiveArrayTools.borrow(positions)
        
        val returnVal = lib.IoLayoutBuilder_add_input_auto(handle, nameSliceMemory.slice, ioType.handle, positionsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            positionsSliceMemory.close()
        }
    }
    
    /** Add an output with automatic layout inference.
    */
    fun addOutputAuto(name: String, ioType: IoType, positions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val positionsSliceMemory = PrimitiveArrayTools.borrow(positions)
        
        val returnVal = lib.IoLayoutBuilder_add_output_auto(handle, nameSliceMemory.slice, ioType.handle, positionsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            positionsSliceMemory.close()
        }
    }
    
    /** Add an input from a region given as flat block positions (sorted by
    *the default YXZ strategy, matching the old region semantics).
    */
    fun addInputFromRegion(name: String, ioType: IoType, layout: LayoutFunction, regionPositions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.IoLayoutBuilder_add_input_from_region(handle, nameSliceMemory.slice, ioType.handle, layout.handle, regionPositionsSliceMemory.slice);
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
    
    /** Add an input from a region (flat positions) with automatic layout
    *inference.
    */
    fun addInputFromRegionAuto(name: String, ioType: IoType, regionPositions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.IoLayoutBuilder_add_input_from_region_auto(handle, nameSliceMemory.slice, ioType.handle, regionPositionsSliceMemory.slice);
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
    
    /** Add an output from a region given as flat block positions.
    */
    fun addOutputFromRegion(name: String, ioType: IoType, layout: LayoutFunction, regionPositions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.IoLayoutBuilder_add_output_from_region(handle, nameSliceMemory.slice, ioType.handle, layout.handle, regionPositionsSliceMemory.slice);
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
    
    /** Add an output from a region (flat positions) with automatic layout
    *inference.
    */
    fun addOutputFromRegionAuto(name: String, ioType: IoType, regionPositions: IntArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val regionPositionsSliceMemory = PrimitiveArrayTools.borrow(regionPositions)
        
        val returnVal = lib.IoLayoutBuilder_add_output_from_region_auto(handle, nameSliceMemory.slice, ioType.handle, regionPositionsSliceMemory.slice);
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
    
    /** Build the [IoLayout]. Consumes the builder (a second call returns
    *`AlreadyConsumed`).
    */
    fun build(): Result<IoLayout> {
        
        val returnVal = lib.IoLayoutBuilder_build(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal 
            val returnOpaque = IoLayout(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}