<?php
namespace Stencil;

final class TickSimulation {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;
    private ?object $borrowedFrom;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned, ?object $borrowedFrom = null) {
        $this->ptr = $ptr;
        $this->owned = $owned;
        $this->borrowedFrom = $borrowedFrom;
    }

    public static function fromSnbt(string $snbt, int $settle,  $origin_x,  $origin_y,  $origin_z, string $extra_states) {
        $__n0 = strlen($snbt);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $snbt, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n5 = strlen($extra_states);
        $__view5 = Lib::ffi()->new('DiplomatStringView');
        if ($__n5 > 0) {
            $__buf5 = Lib::ffi()->new("uint8_t[" . $__n5 . "]", false);
            \FFI::memcpy($__buf5, $extra_states, $__n5);
            $__view5->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf5[0]));
        } else {
            $__view5->data = null;
        }
        $__view5->len = $__n5;
        $result = Lib::ffi()->TickSimulation_from_snbt($__view0, $settle, $origin_x, $origin_y, $origin_z, $__view5);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TickSimulation($result->ok, true);
    }

    public static function fromSchematic( $schematic, int $settle,  $origin_x,  $origin_y,  $origin_z, string $extra_states) {
        $__n5 = strlen($extra_states);
        $__view5 = Lib::ffi()->new('DiplomatStringView');
        if ($__n5 > 0) {
            $__buf5 = Lib::ffi()->new("uint8_t[" . $__n5 . "]", false);
            \FFI::memcpy($__buf5, $extra_states, $__n5);
            $__view5->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf5[0]));
        } else {
            $__view5->data = null;
        }
        $__view5->len = $__n5;
        $result = Lib::ffi()->TickSimulation_from_schematic($schematic->ptr, $settle, $origin_x, $origin_y, $origin_z, $__view5);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TickSimulation($result->ok, true);
    }

    public static function fromBlocks( $bx,  $by,  $bz,  $travel,  $x_off, string $palette, array $cells,  $air_index, int $settle,  $origin_x,  $origin_y,  $origin_z) {
        $__n5 = strlen($palette);
        $__view5 = Lib::ffi()->new('DiplomatStringView');
        if ($__n5 > 0) {
            $__buf5 = Lib::ffi()->new("uint8_t[" . $__n5 . "]", false);
            \FFI::memcpy($__buf5, $palette, $__n5);
            $__view5->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf5[0]));
        } else {
            $__view5->data = null;
        }
        $__view5->len = $__n5;
        $__n6 = count($cells);
        $__view6 = Lib::ffi()->new('DiplomatU16View');
        if ($__n6 > 0) {
            $__arr6 = Lib::ffi()->new("uint16_t[" . $__n6 . "]", false);
            foreach ($cells as $__i6 => $__v6) { $__arr6[$__i6] = $__v6; }
            $__view6->data = \FFI::addr($__arr6[0]);
        } else {
            $__view6->data = null;
        }
        $__view6->len = $__n6;
        $result = Lib::ffi()->TickSimulation_from_blocks($bx, $by, $bz, $travel, $x_off, $__view5, $__view6, $air_index, $settle, $origin_x, $origin_y, $origin_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TickSimulation($result->ok, true);
    }

    public static function evalFlightBatch( $bx,  $by,  $bz,  $travel,  $x_off, string $palette, array $cells,  $air_index, array $kicks,  $eval_ticks,  $seed,  $must_move_by_tick,  $need_period,  $early_exit) {
        $__n5 = strlen($palette);
        $__view5 = Lib::ffi()->new('DiplomatStringView');
        if ($__n5 > 0) {
            $__buf5 = Lib::ffi()->new("uint8_t[" . $__n5 . "]", false);
            \FFI::memcpy($__buf5, $palette, $__n5);
            $__view5->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf5[0]));
        } else {
            $__view5->data = null;
        }
        $__view5->len = $__n5;
        $__n6 = count($cells);
        $__view6 = Lib::ffi()->new('DiplomatU16View');
        if ($__n6 > 0) {
            $__arr6 = Lib::ffi()->new("uint16_t[" . $__n6 . "]", false);
            foreach ($cells as $__i6 => $__v6) { $__arr6[$__i6] = $__v6; }
            $__view6->data = \FFI::addr($__arr6[0]);
        } else {
            $__view6->data = null;
        }
        $__view6->len = $__n6;
        $__n8 = count($kicks);
        $__view8 = Lib::ffi()->new('DiplomatI32View');
        if ($__n8 > 0) {
            $__arr8 = Lib::ffi()->new("int32_t[" . $__n8 . "]", false);
            foreach ($kicks as $__i8 => $__v8) { $__arr8[$__i8] = $__v8; }
            $__view8->data = \FFI::addr($__arr8[0]);
        } else {
            $__view8->data = null;
        }
        $__view8->len = $__n8;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->TickSimulation_eval_flight_batch($bx, $by, $bz, $travel, $x_off, $__view5, $__view6, $air_index, $__view8, $eval_ticks, $seed, $must_move_by_tick, $need_period, $early_exit, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function setRngSeed( $seed) {
        Lib::ffi()->TickSimulation_set_rng_seed($this->ptr, $seed);
    }

    public function step() {
        Lib::ffi()->TickSimulation_step($this->ptr);
    }

    public function run( $ticks) {
        Lib::ffi()->TickSimulation_run($this->ptr, $ticks);
    }

    public function runUntilQuiescent( $budget) {
        $ret = Lib::ffi()->TickSimulation_run_until_quiescent($this->ptr, $budget);
        return $ret;
    }

    public function tickCount() {
        $ret = Lib::ffi()->TickSimulation_tick_count($this->ptr);
        return $ret;
    }

    public function isQuiescent() {
        $ret = Lib::ffi()->TickSimulation_is_quiescent($this->ptr);
        return $ret;
    }

    public function useBlock( $x,  $y,  $z) {
        Lib::ffi()->TickSimulation_use_block($this->ptr, $x, $y, $z);
    }

    public function placeBlock( $x,  $y,  $z, string $state) {
        $__n3 = strlen($state);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $state, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->TickSimulation_place_block($this->ptr, $x, $y, $z, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function getBlock( $x,  $y,  $z) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TickSimulation_get_block($this->ptr, $x, $y, $z, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function checkpoint() {
        $ret = Lib::ffi()->TickSimulation_checkpoint($this->ptr);
        return $ret;
    }

    public function restore( $id) {
        $result = Lib::ffi()->TickSimulation_restore($this->ptr, $id);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function gametestSnbt( $schematic) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TickSimulation_gametest_snbt($schematic->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function changesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TickSimulation_changes_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function itemEntitiesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TickSimulation_item_entities_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function eventsSummaryJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TickSimulation_events_summary_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function nonAirCount() {
        $ret = Lib::ffi()->TickSimulation_non_air_count($this->ptr);
        return $ret;
    }

    public function nonAirCenterX() {
        $ret = Lib::ffi()->TickSimulation_non_air_center_x($this->ptr);
        return $ret;
    }

    public function nonAirMinX() {
        $ret = Lib::ffi()->TickSimulation_non_air_min_x($this->ptr);
        return $ret;
    }

    public function nonAirMaxX() {
        $ret = Lib::ffi()->TickSimulation_non_air_max_x($this->ptr);
        return $ret;
    }

    public function changesCount() {
        $ret = Lib::ffi()->TickSimulation_changes_count($this->ptr);
        return $ret;
    }

    public function worldSnapshotJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TickSimulation_world_snapshot_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->TickSimulation_destroy($this->ptr);
        }
    }
}
