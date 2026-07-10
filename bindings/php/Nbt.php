<?php
namespace Stencil;

final class Nbt {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function textBuild(string $s, string $color,  $bold,  $italic) {
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
        $__n1 = strlen($color);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $color, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Nbt_text_build($__view0, $__view1, $bold, $italic, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function chestBuild(string $items_json, string $name) {
        $__n0 = strlen($items_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $items_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($name);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $name, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Nbt_chest_build($__view0, $__view1, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function signBuild(string $front_json, string $back_json, string $color,  $glowing,  $waxed) {
        $__n0 = strlen($front_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $front_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($back_json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $back_json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $__n2 = strlen($color);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $color, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Nbt_sign_build($__view0, $__view1, $__view2, $glowing, $waxed, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Nbt_destroy($this->ptr);
        }
    }
}
