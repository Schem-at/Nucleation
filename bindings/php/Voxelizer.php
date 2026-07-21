<?php
namespace Stencil;

final class Voxelizer {
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

    public static function shapeFromGlb(array $data,  $target_size,  $shell) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Voxelizer_shape_from_glb($__view0, $target_size, $shell);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public static function shapeFromObj(string $text,  $target_size,  $shell) {
        $__n0 = strlen($text);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $text, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Voxelizer_shape_from_obj($__view0, $target_size, $shell);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public static function schematicFromGlbTextured(array $data,  $target_size,  $shell,  $palette, string $name) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n4 = strlen($name);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $name, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->Voxelizer_schematic_from_glb_textured($__view0, $target_size, $shell, $palette->ptr, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Voxelizer_destroy($this->ptr);
        }
    }
}
