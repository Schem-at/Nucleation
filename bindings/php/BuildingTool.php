<?php
namespace Stencil;

final class BuildingTool {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function fill( $schematic,  $shape,  $brush) {
        Lib::ffi()->BuildingTool_fill($schematic->ptr, $shape->ptr, $brush->ptr);
    }

    public static function rstack( $schematic,  $shape,  $brush,  $count,  $offset_x,  $offset_y,  $offset_z) {
        Lib::ffi()->BuildingTool_rstack($schematic->ptr, $shape->ptr, $brush->ptr, $count, $offset_x, $offset_y, $offset_z);
    }

    public static function fillOnlyAir( $schematic,  $shape,  $brush) {
        Lib::ffi()->BuildingTool_fill_only_air($schematic->ptr, $shape->ptr, $brush->ptr);
    }

    public static function fillReplacing( $schematic,  $shape,  $brush, string $targets_json) {
        $__n3 = strlen($targets_json);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $targets_json, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->BuildingTool_fill_replacing($schematic->ptr, $shape->ptr, $brush->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->BuildingTool_destroy($this->ptr);
        }
    }
}
