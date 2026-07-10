<?php
namespace Stencil;

final class SortStrategy {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function yxz() {
        $ret = Lib::ffi()->SortStrategy_yxz();
        return new SortStrategy($ret, true);
    }

    public static function xyz() {
        $ret = Lib::ffi()->SortStrategy_xyz();
        return new SortStrategy($ret, true);
    }

    public static function zyx() {
        $ret = Lib::ffi()->SortStrategy_zyx();
        return new SortStrategy($ret, true);
    }

    public static function yDescXz() {
        $ret = Lib::ffi()->SortStrategy_y_desc_xz();
        return new SortStrategy($ret, true);
    }

    public static function xDescYz() {
        $ret = Lib::ffi()->SortStrategy_x_desc_yz();
        return new SortStrategy($ret, true);
    }

    public static function zDescYx() {
        $ret = Lib::ffi()->SortStrategy_z_desc_yx();
        return new SortStrategy($ret, true);
    }

    public static function descending() {
        $ret = Lib::ffi()->SortStrategy_descending();
        return new SortStrategy($ret, true);
    }

    public static function distanceFrom( $x,  $y,  $z) {
        $ret = Lib::ffi()->SortStrategy_distance_from($x, $y, $z);
        return new SortStrategy($ret, true);
    }

    public static function distanceFromDesc( $x,  $y,  $z) {
        $ret = Lib::ffi()->SortStrategy_distance_from_desc($x, $y, $z);
        return new SortStrategy($ret, true);
    }

    public static function preserve() {
        $ret = Lib::ffi()->SortStrategy_preserve();
        return new SortStrategy($ret, true);
    }

    public static function reverse() {
        $ret = Lib::ffi()->SortStrategy_reverse();
        return new SortStrategy($ret, true);
    }

    public static function fromString(string $s) {
        $__n0 = strlen($s);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $s, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->SortStrategy_from_string($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new SortStrategy($result->ok, true);
    }

    public function name() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->SortStrategy_name($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->SortStrategy_destroy($this->ptr);
        }
    }
}
