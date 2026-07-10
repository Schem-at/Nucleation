<?php
namespace Stencil;

final class MchprsWorld {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create( $schematic) {
        $result = Lib::ffi()->MchprsWorld_create($schematic->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MchprsWorld($result->ok, true);
    }

    public static function createWithOptions( $schematic,  $optimize,  $io_only) {
        $result = Lib::ffi()->MchprsWorld_create_with_options($schematic->ptr, $optimize, $io_only);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MchprsWorld($result->ok, true);
    }

    public static function createWithCustomIo( $schematic,  $optimize,  $io_only, array $custom_io_positions) {
        $__n3 = count($custom_io_positions);
        $__view3 = Lib::ffi()->new('DiplomatI32View');
        if ($__n3 > 0) {
            $__arr3 = Lib::ffi()->new("int32_t[" . $__n3 . "]", false);
            foreach ($custom_io_positions as $__i3 => $__v3) { $__arr3[$__i3] = $__v3; }
            $__view3->data = \FFI::addr($__arr3[0]);
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->MchprsWorld_create_with_custom_io($schematic->ptr, $optimize, $io_only, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MchprsWorld($result->ok, true);
    }

    public static function simulateUseBlock( $schematic,  $ticks, array $events_xyz) {
        $__n2 = count($events_xyz);
        $__view2 = Lib::ffi()->new('DiplomatI32View');
        if ($__n2 > 0) {
            $__arr2 = Lib::ffi()->new("int32_t[" . $__n2 . "]", false);
            foreach ($events_xyz as $__i2 => $__v2) { $__arr2[$__i2] = $__v2; }
            $__view2->data = \FFI::addr($__arr2[0]);
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->MchprsWorld_simulate_use_block($schematic->ptr, $ticks, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public function tick( $ticks) {
        Lib::ffi()->MchprsWorld_tick($this->ptr, $ticks);
    }

    public function flush() {
        Lib::ffi()->MchprsWorld_flush($this->ptr);
    }

    public function setLeverPower( $x,  $y,  $z,  $powered) {
        Lib::ffi()->MchprsWorld_set_lever_power($this->ptr, $x, $y, $z, $powered);
    }

    public function getLeverPower( $x,  $y,  $z) {
        $ret = Lib::ffi()->MchprsWorld_get_lever_power($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function isLit( $x,  $y,  $z) {
        $ret = Lib::ffi()->MchprsWorld_is_lit($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function setSignalStrength( $x,  $y,  $z,  $strength) {
        Lib::ffi()->MchprsWorld_set_signal_strength($this->ptr, $x, $y, $z, $strength);
    }

    public function getSignalStrength( $x,  $y,  $z) {
        $ret = Lib::ffi()->MchprsWorld_get_signal_strength($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function onUseBlock( $x,  $y,  $z) {
        Lib::ffi()->MchprsWorld_on_use_block($this->ptr, $x, $y, $z);
    }

    public function syncToSchematic() {
        Lib::ffi()->MchprsWorld_sync_to_schematic($this->ptr);
    }

    public function getSchematic() {
        $ret = Lib::ffi()->MchprsWorld_get_schematic($this->ptr);
        return new Schematic($ret, true);
    }

    public function getRedstonePower( $x,  $y,  $z) {
        $ret = Lib::ffi()->MchprsWorld_get_redstone_power($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function checkCustomIoChanges() {
        Lib::ffi()->MchprsWorld_check_custom_io_changes($this->ptr);
    }

    public function pollCustomIoChangesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->MchprsWorld_poll_custom_io_changes_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function peekCustomIoChangesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->MchprsWorld_peek_custom_io_changes_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function clearCustomIoChanges() {
        Lib::ffi()->MchprsWorld_clear_custom_io_changes($this->ptr);
    }

    public function exportGraph() {
        $result = Lib::ffi()->MchprsWorld_export_graph($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new RedstoneGraph($result->ok, true);
    }

    public function exportGraphStructural() {
        $result = Lib::ffi()->MchprsWorld_export_graph_structural($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new RedstoneGraph($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->MchprsWorld_destroy($this->ptr);
        }
    }
}
