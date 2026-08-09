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
    fun Design_route_bus_or(handle: Pointer, name: Slice, driversJson: Slice, sinksJson: Slice, gatesJson: Slice, styleJson: Slice, write: Pointer): ResultUnitInt
    fun Design_set_block(handle: Pointer, x: Int, y: Int, z: Int, block: Slice): ResultUnitInt
    fun Design_move_instance(handle: Pointer, name: Slice, x: Int, y: Int, z: Int, rotY: Int, write: Pointer): ResultUnitInt
    fun Design_remove_instance(handle: Pointer, name: Slice, write: Pointer): ResultUnitInt
    fun Design_reroute(handle: Pointer, name: Slice, write: Pointer): ResultUnitInt
    fun Design_remove_bus(handle: Pointer, name: Slice): ResultUnitInt
    fun Design_to_schem_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun Design_flatten_composite(handle: Pointer): ResultPointerInt
    fun Design_instance_ports(handle: Pointer, write: Pointer): ResultUnitInt
    fun Design_resolve_port(handle: Pointer, name: Slice, write: Pointer): ResultUnitInt
    fun Design_add_gate(handle: Pointer, bus: Slice, gate: Slice, x: Int, y: Int, z: Int, sx: Int, sy: Int, sz: Int, write: Pointer): ResultUnitInt
    fun Design_move_gate(handle: Pointer, bus: Slice, gate: Slice, x: Int, y: Int, z: Int, write: Pointer): ResultUnitInt
    fun Design_set_bus_rule(handle: Pointer, bus: Slice, ruleJson: Slice): ResultUnitInt
    fun Design_bus_skew(handle: Pointer, name: Slice, write: Pointer): ResultUnitInt
    fun Design_bus_state(handle: Pointer, name: Slice, write: Pointer): ResultUnitInt
    fun Design_rip(handle: Pointer, name: Slice): ResultUnitInt
    fun Design_flatten(handle: Pointer): ResultPointerInt
    fun Design_check(handle: Pointer, write: Pointer): ResultUnitInt
    fun Design_bake(handle: Pointer, budget: FFIUint32): ResultPointerInt
    fun Design_to_nucm_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun Design_from_nucm(data: Slice): ResultPointerInt
    fun Design_save_nucm(handle: Pointer, path: Slice): ResultUnitInt
    fun Design_load_nucm(path: Slice): ResultPointerInt
    fun Design_to_litematic_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun Design_from_litematic(data: Slice): ResultPointerInt
    fun Design_export_litematic(handle: Pointer, path: Slice): ResultUnitInt
    fun Design_import_litematic(path: Slice): ResultPointerInt
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
        @JvmStatic

        /** Reopen a `.nucm` design document from raw bytes. The reloaded
        *design is the same model mid-edit: rerouting works.
        */
        fun fromNucm(data: UByteArray): Result<Design> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)

            val returnVal = lib.Design_from_nucm(dataSliceMemory.slice);
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
                dataSliceMemory.close()
            }
        }
        @JvmStatic

        /** Load a `.nucm` project document from a file. Not available in
        *JS — read the bytes yourself and use `from_nucm`.
        */
        fun loadNucm(path: String): Result<Design> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)

            val returnVal = lib.Design_load_nucm(pathSliceMemory.slice);
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
                pathSliceMemory.close()
            }
        }
        @JvmStatic

        /** Import a layered `.litematic` (with a `NucleationDesign`
        *manifest) from raw bytes; a plain litematic errors loudly —
        *open those with `Schematic.from_litematic`.
        */
        fun fromLitematic(data: UByteArray): Result<Design> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)

            val returnVal = lib.Design_from_litematic(dataSliceMemory.slice);
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
                dataSliceMemory.close()
            }
        }
        @JvmStatic

        /** Import a layered `.litematic` from a file. Not available in JS
        *— read the bytes yourself and use `from_litematic`.
        */
        fun importLitematic(path: String): Result<Design> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)

            val returnVal = lib.Design_import_litematic(pathSliceMemory.slice);
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
                pathSliceMemory.close()
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

    /** Declare AND realize a wired-OR bus: `drivers_json` is a JSON
    *array of port names — multiple drivers are legal ONLY through
    *this explicit merge (`merge="or"`). Extra drivers join the
    *trunk as diode-isolated dust-merge branches; the LVS intent
    *stays ONE net per bit. Same shapes as `route_bus` otherwise.
    */
    fun routeBusOr(name: String, driversJson: String, sinksJson: String, gatesJson: String, styleJson: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val driversJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(driversJson)
        val sinksJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(sinksJson)
        val gatesJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(gatesJson)
        val styleJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(styleJson)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_route_bus_or(handle, nameSliceMemory.slice, driversJsonSliceMemory.slice, sinksJsonSliceMemory.slice, gatesJsonSliceMemory.slice, styleJsonSliceMemory.slice, write);
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
            driversJsonSliceMemory.close()
            sinksJsonSliceMemory.close()
            gatesJsonSliceMemory.close()
            styleJsonSliceMemory.close()
        }
    }

    /** Edit the loose block layer: plain `set_block` on the base
    *schematic (participates in occupancy and flatten).
    */
    fun setBlock(x: Int, y: Int, z: Int, block: String): Result<Unit> {
        val blockSliceMemory = PrimitiveArrayTools.borrowUtf8(block)

        val returnVal = lib.Design_set_block(handle, x, y, z, blockSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockSliceMemory.close()
        }
    }

    /** Drag an instance layer to a new position/rotation. The move
    *itself ALWAYS succeeds (the document's truth); the affected bus
    *set — fragments intersecting the old or new footprint +
    *influence halo, plus every already-failed bus — is ripped and
    *co-rerouted deterministically with bounded retry rounds.
    *Writes `{"rerouted": [...], "failed": {name: reason}}`.
    */
    fun moveInstance(name: String, x: Int, y: Int, z: Int, rotY: Int): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_move_instance(handle, nameSliceMemory.slice, x, y, z, rotY, write);
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

    /** Remove an instance layer. Buses that terminate on one of its
    *ports are DELETED (they lost an endpoint); buses that merely
    *crossed its space are ripped and co-rerouted. Writes
    *`{"removed_buses": [...], "rerouted": [...], "failed": {...}}`.
    */
    fun removeInstance(name: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_remove_instance(handle, nameSliceMemory.slice, write);
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

    /** Re-realize a bus from its stored declaration (the counterpart to
    *`rip`); writes the resulting bus state.
    */
    fun reroute(name: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_reroute(handle, nameSliceMemory.slice, write);
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

    /** Delete a bus outright — fragment AND declaration, freeing the
    *name. `rip` keeps the declaration so the bus can be rerouted.
    */
    fun removeBus(name: String): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

        val returnVal = lib.Design_remove_bus(handle, nameSliceMemory.slice);
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

    /** The flattened artifact as `.schem` bytes, base64. Unlike
    *`flatten()` + the schematic writer, this composites the layer
    *stack into ONE region first: `.schem` has no layers, and the
    *region merge drops named-layer cells that the loose layer's
    *bounding box shadows.
    */
    fun toSchemB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_to_schem_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** The flattened artifact composited into ONE region (see
    *`to_schem_b64`) — the shape an interchange export wants.
    */
    fun flattenComposite(): Result<Schematic> {

        val returnVal = lib.Design_flatten_composite(handle);
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

    /** Every routing endpoint the placed instances expose, as a JSON
    *array of `{name, instance, port, role, ty, width, hardware,
    *wires, step, routable, blocked}`. `name` is `{instance}.{port}`
    *— exactly what `route_bus` accepts; `role` is the CELL-facing
    *direction, so `"output"` drives a bus and `"input"` receives
    *one. A port whose bits have no dust connection cell (a lever
    *input, say) reports `routable: false` and why in `blocked`.
    */
    fun instancePorts(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_instance_ports(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Resolve one routing endpoint name — a declared design port or an
    *instance port `{instance}.{port}` — to the geometry a bus would
    *use: `{"name","anchor","step","width","direction","connectable"}`.
    *`direction` is DESIGN-facing (`"input"` drives buses).
    */
    fun resolvePort(name: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_resolve_port(handle, nameSliceMemory.slice, write);
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

    /** Add a gate to an existing bus (splitting the segment it lands
    *in) and re-realize it. Writes the resulting bus state.
    */
    fun addGate(bus: String, gate: String, x: Int, y: Int, z: Int, sx: Int, sy: Int, sz: Int): Result<String> {
        val busSliceMemory = PrimitiveArrayTools.borrowUtf8(bus)
        val gateSliceMemory = PrimitiveArrayTools.borrowUtf8(gate)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_add_gate(handle, busSliceMemory.slice, gateSliceMemory.slice, x, y, z, sx, sy, sz, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            busSliceMemory.close()
            gateSliceMemory.close()
        }
    }

    /** Drag a gate: the anchor moves unconditionally, then EXACTLY the
    *two adjacent segments are ripped and rerouted atomically. An
    *unroutable move leaves the bus `failed: reason` — visible,
    *never half-routed. Writes `{"state": "...",
    *"rerouted_segments": n}`.
    */
    fun moveGate(bus: String, gate: String, x: Int, y: Int, z: Int): Result<String> {
        val busSliceMemory = PrimitiveArrayTools.borrowUtf8(bus)
        val gateSliceMemory = PrimitiveArrayTools.borrowUtf8(gate)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_move_gate(handle, busSliceMemory.slice, gateSliceMemory.slice, x, y, z, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            busSliceMemory.close()
            gateSliceMemory.close()
        }
    }

    /** Attach a net-class discipline to a bus (JSON `NetClassRule`:
    *optional `max_len_rt` delay budget, `y_band` layer band, …);
    *`check()` enforces it.
    */
    fun setBusRule(bus: String, ruleJson: String): Result<Unit> {
        val busSliceMemory = PrimitiveArrayTools.borrowUtf8(bus)
        val ruleJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(ruleJson)

        val returnVal = lib.Design_set_bus_rule(handle, busSliceMemory.slice, ruleJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            busSliceMemory.close()
            ruleJsonSliceMemory.close()
        }
    }

    /** Per-bus skew from the routed fragment: writes
    *`{"per_bit_rt": [...], "skew_rt": n, "max_rt": n}`.
    */
    fun busSkew(name: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_bus_skew(handle, nameSliceMemory.slice, write);
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

    /** Serialize the FULL design document to `.nucm` project-tier
    *bytes (magic `NUCM`): cells deduped by content hash, instance
    *transforms, ports with scanned hardware, every bus layer with
    *its fragment, runs and `intended`/`routed`/`failed: reason`
    *state, and the loose base layer. Base64 across the bridge.
    */
    fun toNucmB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_to_nucm_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Save the `.nucm` project document to a file. Not available in
    *JS: the WASM build has no filesystem — use `to_nucm_b64`.
    */
    fun saveNucm(path: String): Result<Unit> {
        val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)

        val returnVal = lib.Design_save_nucm(handle, pathSliceMemory.slice);
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

    /** Export the design as a LAYERED `.litematic` (interchange tier):
    *one named region per layer (`inst:{name}`, `bus:{name}`, loose
    *base) plus the design manifest as a root-level
    *`NucleationDesign` tag. Opens in Litematica as a plain
    *multi-region litematic; reimports as a design whose cell
    *references have degraded to embedded copies. Base64 across the
    *bridge.
    */
    fun toLitematicB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Design_to_litematic_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Export the layered `.litematic` to a file. Not available in JS
    *— use `to_litematic_b64`.
    */
    fun exportLitematic(path: String): Result<Unit> {
        val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)

        val returnVal = lib.Design_export_litematic(handle, pathSliceMemory.slice);
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

}
