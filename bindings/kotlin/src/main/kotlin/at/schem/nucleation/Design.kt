package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface DesignLib: Library {
    fun Design_destroy(handle: Pointer)
    fun Design_create(name: Slice): ResultPointerInt
    fun Design_for_schematic(name: Slice, base: Pointer): ResultPointerInt
    fun Design_add_cell(handle: Pointer, name: Slice, cell: Pointer, write: Pointer): ResultUnitInt
    fun Design_place(handle: Pointer, name: Slice, cell: Slice, x: Int, y: Int, z: Int, rotY: Int): ResultUnitInt
    fun Design_declare_input(handle: Pointer, name: Slice, ax: Int, ay: Int, az: Int, sx: Int, sy: Int, sz: Int, width: FFIUint8, ty: Slice): ResultUnitInt
    fun Design_declare_output(handle: Pointer, name: Slice, ax: Int, ay: Int, az: Int, sx: Int, sy: Int, sz: Int, width: FFIUint8, ty: Slice): ResultUnitInt
    fun Design_route_bus(handle: Pointer, name: Slice, driver: Slice, sinksJson: Slice, gatesJson: Slice, styleJson: Slice, write: Pointer): ResultUnitInt
    fun Design_bus_state(handle: Pointer, name: Slice, write: Pointer): ResultUnitInt
    fun Design_rip(handle: Pointer, name: Slice): ResultUnitInt
    fun Design_flatten(handle: Pointer): ResultPointerInt
    fun Design_check(handle: Pointer, write: Pointer): ResultUnitInt
    fun Design_bake(handle: Pointer, budget: FFIUint32): ResultPointerInt
}
/** A composition document: loose blocks + cell instances + bus layers
*over a shared coordinate space.
*/
class Design internal constructor (
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

    private class DesignCleaner(val handle: Pointer, val lib: DesignLib) : Runnable {
        override fun run() {
            lib.Design_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Design.DesignCleaner(handle, Design.lib));
    }

    companion object {
        internal val libClass: Class<DesignLib> = DesignLib::class.java
        internal val lib: DesignLib = Native.load("nucleation", libClass)
        @JvmStatic

        /** An empty design.
        */
        fun create(name: String): Result<Design> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

            val returnVal = lib.Design_create(nameSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Design(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
        @JvmStatic

        /** A design whose loose block layer is a copy of `base` (endpoint
        *hardware placed with raw `set_block`).
        */
        fun forSchematic(name: String, base: Schematic): Result<Design> {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

            val returnVal = lib.Design_for_schematic(nameSliceMemory.slice, base.handle);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Design(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                nameSliceMemory.close()
            }
        }
    }

    /** Register a library cell; its contract is resolved from the
    *schematic (embedded metadata first, Insign signs as fallback)
    *and registration fails loudly when no source defines one.
    *Writes resolution warnings as a JSON array of strings.
    */
    fun addCell(name: String, cell: Schematic): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_add_cell(handle, nameSliceMemory.slice, cell.handle, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }

    /** Place an instance layer referencing a library cell. `rot_y` is
    *in degrees, a multiple of 90.
    */
    fun place(name: String, cell: String, x: Int, y: Int, z: Int, rotY: Int): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val cellSliceMemory = PrimitiveArrayTools.borrowUtf8(cell)

        val returnVal = lib.Design_place(handle, nameSliceMemory.slice, cellSliceMemory.slice, x, y, z, rotY);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            cellSliceMemory.close()
        }
    }

    /** Declare a drivable input port: anchor = bit-0 connection cell,
    *step to the next bit, `width` bits of `ty` (`"uint"` or
    *`"bool"`). The hardware is scanned (adjacent lever per bit) and
    *validated loudly.
    */
    fun declareInput(name: String, ax: Int, ay: Int, az: Int, sx: Int, sy: Int, sz: Int, width: UByte, ty: String): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val tySliceMemory = PrimitiveArrayTools.borrowUtf8(ty)

        val returnVal = lib.Design_declare_input(handle, nameSliceMemory.slice, ax, ay, az, sx, sy, sz, FFIUint8(width), tySliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            tySliceMemory.close()
        }
    }

    /** Declare a readable output port (adjacent lamp per bit); same
    *shape as `declare_input`.
    */
    fun declareOutput(name: String, ax: Int, ay: Int, az: Int, sx: Int, sy: Int, sz: Int, width: UByte, ty: String): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val tySliceMemory = PrimitiveArrayTools.borrowUtf8(ty)

        val returnVal = lib.Design_declare_output(handle, nameSliceMemory.slice, ax, ay, az, sx, sy, sz, FFIUint8(width), tySliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            tySliceMemory.close()
        }
    }

    /** Declare AND realize a bus. `sinks_json` is a JSON array of port
    *names; `gates_json` an array of `{"name", "anchor": [x,y,z],
    *"step": [x,y,z]}` (pass `[]` for none); `style_json` an object
    *with optional `bus_block` / `transparent_block`. Declaration
    *errors are error returns; geometric unroutability is the
    *written STATE: `"routed"` or `"failed: reason"` — realization
    *is atomic, never half-routed.
    */
    fun routeBus(name: String, driver: String, sinksJson: String, gatesJson: String, styleJson: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val driverSliceMemory = PrimitiveArrayTools.borrowUtf8(driver)
        val sinksJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(sinksJson)
        val gatesJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(gatesJson)
        val styleJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(styleJson)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_route_bus(handle, nameSliceMemory.slice, driverSliceMemory.slice, sinksJsonSliceMemory.slice, gatesJsonSliceMemory.slice, styleJsonSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            driverSliceMemory.close()
            sinksJsonSliceMemory.close()
            gatesJsonSliceMemory.close()
            styleJsonSliceMemory.close()
        }
    }

    /** The lifecycle state of a bus: `"intended"`, `"routed"` or
    *`"failed: reason"`.
    */
    fun busState(name: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_bus_state(handle, nameSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }

    /** Rip a bus: clear its fragment, back to `intended`.
    */
    fun rip(name: String): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

        val returnVal = lib.Design_rip(handle, nameSliceMemory.slice);
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

    /** Collapse the layer stack into ONE self-describing schematic:
    *named regions per layer (`inst:x`, `bus:y`) and the merged
    *contract embedded in the metadata — itself placeable as a cell.
    */
    fun flatten(): Result<Schematic> {

        val returnVal = lib.Design_flatten(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Schematic(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** DRC + LVS over the flattened artifact. Writes `{"clean",
    *"drc": [...], "lvs": {...}, "buses": {...}}`.
    */
    fun check(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_check(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Settle the flattened artifact in the vanilla-accurate tick
    *engine and return it with every settled state written back and
    *`InitialState::Baked` stamped into the embedded contract (needs
    *the `mc-tick` feature, else errors).
    */
    fun bake(budget: UInt): Result<Schematic> {

        val returnVal = lib.Design_bake(handle, FFIUint32(budget));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = Schematic(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
