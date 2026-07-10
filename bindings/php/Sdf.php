<?php
namespace Stencil;

final class Sdf {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function schematicFromSdf(string $sdf_json, string $rules_json,  $has_bounds,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $__n0 = strlen($sdf_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $sdf_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($rules_json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $rules_json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->Sdf_schematic_from_sdf($__view0, $__view1, $has_bounds, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function eval(string $sdf_json,  $x,  $y,  $z) {
        $__n0 = strlen($sdf_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $sdf_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Sdf_eval($__view0, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Sdf_destroy($this->ptr);
        }
    }
}
