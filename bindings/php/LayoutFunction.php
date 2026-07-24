<?php
namespace Stencil;

final class LayoutFunction {
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

    public static function oneToOne() {
        $ret = Lib::ffi()->LayoutFunction_one_to_one();
        return new LayoutFunction($ret, true);
    }

    public static function packed4() {
        $ret = Lib::ffi()->LayoutFunction_packed4();
        return new LayoutFunction($ret, true);
    }

    public static function custom(array $mapping) {
        $__n0 = count($mapping);
        $__view0 = Lib::ffi()->new('DiplomatU32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint32_t[" . $__n0 . "]", false);
            foreach ($mapping as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->LayoutFunction_custom($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new LayoutFunction($result->ok, true);
    }

    public static function rowMajor( $rows,  $cols,  $bits_per_element) {
        $ret = Lib::ffi()->LayoutFunction_row_major($rows, $cols, $bits_per_element);
        return new LayoutFunction($ret, true);
    }

    public static function columnMajor( $rows,  $cols,  $bits_per_element) {
        $ret = Lib::ffi()->LayoutFunction_column_major($rows, $cols, $bits_per_element);
        return new LayoutFunction($ret, true);
    }

    public static function scanline( $width,  $height,  $bits_per_pixel) {
        $ret = Lib::ffi()->LayoutFunction_scanline($width, $height, $bits_per_pixel);
        return new LayoutFunction($ret, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->LayoutFunction_destroy($this->ptr);
        }
    }
}
