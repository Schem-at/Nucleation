<?php
namespace Stencil;

final class Autostack {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function detectStructures( $schematic) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Autostack_detect_structures($schematic->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public static function detectStructuresGraph( $schematic) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Autostack_detect_structures_graph($schematic->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public static function resize1d( $schematic,  $vx,  $vy,  $vz,  $units) {
        $result = Lib::ffi()->Autostack_resize_1d($schematic->ptr, $vx, $vy, $vz, $units);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function resize2d( $schematic,  $v1x,  $v1y,  $v1z,  $v2x,  $v2y,  $v2z,  $n1,  $n2) {
        $result = Lib::ffi()->Autostack_resize_2d($schematic->ptr, $v1x, $v1y, $v1z, $v2x, $v2y, $v2z, $n1, $n2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Autostack_destroy($this->ptr);
        }
    }
}
