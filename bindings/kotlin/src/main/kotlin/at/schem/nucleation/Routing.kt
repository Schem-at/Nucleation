package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface RoutingLib: Library {
    fun Routing_destroy(handle: Pointer)
    fun Routing_route_net(schematic: Pointer, sx: Int, sy: Int, sz: Int, dx: Int, dy: Int, dz: Int, label: Slice, write: Pointer): ResultUnitInt
    fun Routing_route_all(schematic: Pointer, netsJson: Slice, write: Pointer): ResultUnitInt
    fun Routing_lvs(schematic: Pointer, intentJson: Slice, write: Pointer): ResultUnitInt
    fun Routing_drc(schematic: Pointer, checkDecay: Boolean, write: Pointer): ResultUnitInt
    fun Routing_sta(schematic: Pointer, netlistJson: Slice, write: Pointer): ResultUnitInt
}
/** Namespacing opaque for routing entry points (static methods taking
*`&Schematic` explicitly, like `Autostack`).
*/
class Routing internal constructor (
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

    private class RoutingCleaner(val handle: Pointer, val lib: RoutingLib) : Runnable {
        override fun run() {
            lib.Routing_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Routing.RoutingCleaner(handle, Routing.lib));
    }

    companion object {
        internal val libClass: Class<RoutingLib> = RoutingLib::class.java
        internal val lib: RoutingLib = Native.load("nucleation", libClass)
        @JvmStatic

        /** Route one net from `(sx, sy, sz)` to `(dx, dy, dz)` with default
        *rules (torch-ladder vias, stair cap 4, refresh 5) and write the
        *emitted geometry into the schematic. Writes the routed path as a
        *JSON array of `[x, y, z]` cells.
        */
        fun routeNet(schematic: Schematic, sx: Int, sy: Int, sz: Int, dx: Int, dy: Int, dz: Int, label: String): Result<String> {
            val labelSliceMemory = PrimitiveArrayTools.borrowUtf8(label)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Routing_route_net(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, sx, sy, sz, dx, dy, dz, labelSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {

                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                labelSliceMemory.close()
            }
        }
        @JvmStatic

        /** Route every net in `nets_json` with negotiated congestion
        *(pnr-core PathFinder) in one labelled workspace, write the
        *geometry into the schematic, and write the JSON report
        *(`routes` with per-net `path`/`delay_rt`, `notes`,
        *`violations`). Supports per-net-class rule overrides
        *(`classes`: io_contract `NetClassRule`s, with `region`
        *resolving named route zones tagged on the schematic's
        *DefinitionRegions), plus `bounds`, `budget` and `congestion`
        *options — see `crate::routing::route_all_schematic` for the
        *exact request shape.
        */
        fun routeAll(schematic: Schematic, netsJson: String): Result<String> {
            val netsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(netsJson)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Routing_route_all(schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, netsJsonSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {

                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                netsJsonSliceMemory.close()
            }
        }
        @JvmStatic

        /** LVS v1: compare an intended netlist (`{"nets": [{"name",
        *"terminals": [[x,y,z], ...]}]}`) against the conduction
        *netlist extracted statically from the schematic (dust
        *adjacency incl. cut diagonals plus repeater/comparator/torch
        *through-component edges). Writes `{"clean", "matched",
        *"opens", "shorts", "cycles"}`.
        */
        fun lvs(schematic: Schematic, intentJson: String): Result<String> {
            val intentJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(intentJson)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Routing_lvs(schematic.handle, intentJsonSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {

                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                intentJsonSliceMemory.close()
            }
        }
        @JvmStatic

        /** Run design-rule checks (support audit, repeater-cycle detection,
        *optional decay) over the schematic. Writes a JSON array; each
        *element has `kind` plus violation-specific fields. Label-aware
        *short checking needs a labelled workspace and stays native.
        */
        fun drc(schematic: Schematic, checkDecay: Boolean): Result<String> {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Routing_drc(schematic.handle, checkDecay, write);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        /** Static timing over the schematic plus a gate netlist given as
        *JSON: `{"inputs": ["a", ...], "gates": [{"out": "y",
        *"ins": ["a", "b"], "delay_rt": 2}, ...]}`. Writes
        *`{"arrival_rt": {sig: rt}, "critical": [sig, ...]}`.
        */
        fun sta(schematic: Schematic, netlistJson: String): Result<String> {
            val netlistJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(netlistJson)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Routing_sta(schematic.handle, netlistJsonSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {

                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                netlistJsonSliceMemory.close()
            }
        }
    }

}
