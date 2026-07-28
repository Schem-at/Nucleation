package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface TickSimulationLib: Library {
    fun TickSimulation_destroy(handle: Pointer)
    fun TickSimulation_from_snbt(snbt: Slice, settle: Int, originX: Int, originY: Int, originZ: Int, extraStates: Slice): ResultPointerInt
    fun TickSimulation_from_schematic(schematic: Pointer, settle: Int, originX: Int, originY: Int, originZ: Int, extraStates: Slice): ResultPointerInt
    fun TickSimulation_set_rng_seed(handle: Pointer, seed: Long): Unit
    fun TickSimulation_step(handle: Pointer): Unit
    fun TickSimulation_run(handle: Pointer, ticks: FFIUint32): Unit
    fun TickSimulation_run_until_quiescent(handle: Pointer, budget: FFIUint32): Byte
    fun TickSimulation_tick_count(handle: Pointer): FFIUint32
    fun TickSimulation_is_quiescent(handle: Pointer): Byte
    fun TickSimulation_use_block(handle: Pointer, x: Int, y: Int, z: Int): Unit
    fun TickSimulation_place_block(handle: Pointer, x: Int, y: Int, z: Int, state: Slice): ResultUnitInt
    fun TickSimulation_get_block(handle: Pointer, x: Int, y: Int, z: Int, write: Pointer): Unit
    fun TickSimulation_checkpoint(handle: Pointer): FFIUint32
    fun TickSimulation_restore(handle: Pointer, id: FFIUint32): ResultUnitInt
    fun TickSimulation_gametest_snbt(schematic: Pointer, write: Pointer): Unit
    fun TickSimulation_changes_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_item_entities_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_events_summary_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_non_air_count(handle: Pointer): FFIUint32
    fun TickSimulation_non_air_center_x(handle: Pointer): Double
    fun TickSimulation_non_air_min_x(handle: Pointer): Int
    fun TickSimulation_non_air_max_x(handle: Pointer): Int
    fun TickSimulation_changes_count(handle: Pointer): FFIUint32
    fun TickSimulation_world_snapshot_json(handle: Pointer, write: Pointer): Unit
}
/** A headless, vanilla-accurate tick simulation of one structure.
*/
class TickSimulation internal constructor (
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

    private class TickSimulationCleaner(val handle: Pointer, val lib: TickSimulationLib) : Runnable {
        override fun run() {
            lib.TickSimulation_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, TickSimulation.TickSimulationCleaner(handle, TickSimulation.lib));
    }

    companion object {
        internal val libClass: Class<TickSimulationLib> = TickSimulationLib::class.java
        internal val lib: TickSimulationLib = Native.load("nucleation", libClass)
        @JvmStatic

        /** Load from Java structure SNBT text.
        *
        *`extra_states`: semicolon-separated block-state descriptors that
        *later `place_block` calls may write (behaviours bind at
        *construction). `minecraft:redstone_block` is always available.
        *`origin_*`: where the build's (0,0,0) sits in world coordinates —
        *wire update order hashes absolute positions.
        */
        fun fromSnbt(snbt: String, settle: TickSettleMode, originX: Int, originY: Int, originZ: Int, extraStates: String): Result<TickSimulation> {
            val snbtSliceMemory = PrimitiveArrayTools.borrowUtf8(snbt)
            val extraStatesSliceMemory = PrimitiveArrayTools.borrowUtf8(extraStates)

            val returnVal = lib.TickSimulation_from_snbt(snbtSliceMemory.slice, settle.toNative(), originX, originY, originZ, extraStatesSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = TickSimulation(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                snbtSliceMemory.close()
                extraStatesSliceMemory.close()
            }
        }
        @JvmStatic

        /** Load from a schematic (any format nucleation can read), rendered
        *to gametest-flavor structure SNBT for mc-tick's parser.
        */
        fun fromSchematic(schematic: Schematic, settle: TickSettleMode, originX: Int, originY: Int, originZ: Int, extraStates: String): Result<TickSimulation> {
            val extraStatesSliceMemory = PrimitiveArrayTools.borrowUtf8(extraStates)

            val returnVal = lib.TickSimulation_from_schematic(schematic.handle, settle.toNative(), originX, originY, originZ, extraStatesSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = TickSimulation(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                extraStatesSliceMemory.close()
            }
        }
        @JvmStatic

        /** Every recorded block change since settle, as JSON:
        *`[{"tick":N,"pos":[x,y,z],"from":"...","to":"..."}]`.
        *Render a schematic as gametest-flavor structure SNBT — the text
        *`from_snbt` and the corpus/render tooling consume. Lets hosts hand
        *a converted `.litematic`/`.schem` to the video renderer.
        */
        fun gametestSnbt(schematic: Schematic): String {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.TickSimulation_gametest_snbt(schematic.handle, write);

            val returnString = DW.writeToString(write)
            return returnString
        }
    }

    /** Seed the vanilla random source (`java.util.Random`'s LCG,
    *bit-for-bit). Unseeded, jittering behaviours use each
    *distribution's mean — fully deterministic, no noise.
    */
    fun setRngSeed(seed: Long): Unit {

        val returnVal = lib.TickSimulation_set_rng_seed(handle, seed);

    }

    /** Advance one game tick.
    */
    fun step(): Unit {

        val returnVal = lib.TickSimulation_step(handle);

    }

    /** Advance `ticks` game ticks.
    */
    fun run(ticks: UInt): Unit {

        val returnVal = lib.TickSimulation_run(handle, FFIUint32(ticks));

    }

    /** Run until nothing is scheduled or `budget` ticks pass. Returns
    *whether the world went quiet.
    */
    fun runUntilQuiescent(budget: UInt): Boolean {

        val returnVal = lib.TickSimulation_run_until_quiescent(handle, FFIUint32(budget));
        return (returnVal > 0)
    }

    /** Game ticks elapsed since settle.
    */
    fun tickCount(): UInt {

        val returnVal = lib.TickSimulation_tick_count(handle);
        return (returnVal.toUInt())
    }

    /** Whether nothing is scheduled or queued.
    */
    fun isQuiescent(): Boolean {

        val returnVal = lib.TickSimulation_is_quiescent(handle);
        return (returnVal > 0)
    }

    /** Right-click a block with an empty hand (lever, button, note block).
    */
    fun useBlock(x: Int, y: Int, z: Int): Unit {

        val returnVal = lib.TickSimulation_use_block(handle, x, y, z);

    }

    /** Write a block state (`minecraft:air` breaks). The state must be in
    *the structure, in `extra_states`, or `minecraft:redstone_block`.
    */
    fun placeBlock(x: Int, y: Int, z: Int, state: String): Result<Unit> {
        val stateSliceMemory = PrimitiveArrayTools.borrowUtf8(state)

        val returnVal = lib.TickSimulation_place_block(handle, x, y, z, stateSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            stateSliceMemory.close()
        }
    }

    /** The block state descriptor at a position (`minecraft:air` for empty).
    */
    fun getBlock(x: Int, y: Int, z: Int): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_get_block(handle, x, y, z, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Snapshot the entire simulation; returns a checkpoint id.
    */
    fun checkpoint(): UInt {

        val returnVal = lib.TickSimulation_checkpoint(handle);
        return (returnVal.toUInt())
    }

    /** Restore a checkpoint taken earlier on this simulation.
    */
    fun restore(id: UInt): Result<Unit> {

        val returnVal = lib.TickSimulation_restore(handle, FFIUint32(id));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun changesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_changes_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Live item entities and minecarts, as JSON:
    *`{"items":[{"id":N,"item":"...","count":N,"pos":[..],"vel":[..],
    *"on_ground":bool,"contents":[{"id":"...","count":N}]}],
    *"minecarts":[{"id":N,"kind":"...","pos":[..],"vel":[..]}]}`.
    */
    fun itemEntitiesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_item_entities_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Per-tick aggregates over the recorded changes, as JSON:
    *`[{"tick":N,"changes":N,"piston":N,"redstone":N}]` — `piston`
    *counts changes touching piston blocks (base, head, moving), and
    *`redstone` changes touching wire/torch/repeater/comparator/
    *observer/lamp/lever/button/pressure-plate states.
    */
    fun eventsSummaryJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_events_summary_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Every non-air block, as JSON:
    *`[{"pos":[x,y,z],"state":"..."}]`.
    *How many non-air blocks stand in the world right now.
    */
    fun nonAirCount(): UInt {

        val returnVal = lib.TickSimulation_non_air_count(handle);
        return (returnVal.toUInt())
    }

    /** Center of mass (x) of every non-air block — the GA's displacement
    *metric without a JSON round-trip. NaN when the world is empty.
    */
    fun nonAirCenterX(): Double {

        val returnVal = lib.TickSimulation_non_air_center_x(handle);
        return (returnVal)
    }

    /** Smallest x holding a non-air block; `i32::MAX` when empty.
    */
    fun nonAirMinX(): Int {

        val returnVal = lib.TickSimulation_non_air_min_x(handle);
        return (returnVal)
    }

    /** Largest x holding a non-air block; `i32::MIN` when empty.
    */
    fun nonAirMaxX(): Int {

        val returnVal = lib.TickSimulation_non_air_max_x(handle);
        return (returnVal)
    }

    /** How many block changes recording has captured so far.
    */
    fun changesCount(): UInt {

        val returnVal = lib.TickSimulation_changes_count(handle);
        return (returnVal.toUInt())
    }

    fun worldSnapshotJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_world_snapshot_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

}
