<?php
namespace Stencil;

final class DistanceField {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function fromSchematic( $schematic) {
        $ret = Lib::ffi()->DistanceField_from_schematic($schematic->ptr);
        return new DistanceField($ret, true);
    }

    public function depth( $x,  $y,  $z) {
        $ret = Lib::ffi()->DistanceField_depth($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function slope( $x,  $y,  $z) {
        $ret = Lib::ffi()->DistanceField_slope($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function normalJson( $x,  $y,  $z) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->DistanceField_normal_json($this->ptr, $x, $y, $z, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->DistanceField_destroy($this->ptr);
        }
    }
}
