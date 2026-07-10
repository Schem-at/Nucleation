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

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->BuildingTool_destroy($this->ptr);
        }
    }
}
