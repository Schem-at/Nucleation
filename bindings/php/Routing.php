<?php
namespace Stencil;

final class Routing {
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

    public static function routeNet( $schematic,  $sx,  $sy,  $sz,  $dx,  $dy,  $dz, string $label) {
        $__n7 = strlen($label);
        $__view7 = Lib::ffi()->new('DiplomatStringView');
        if ($__n7 > 0) {
            $__buf7 = Lib::ffi()->new("uint8_t[" . $__n7 . "]", false);
            \FFI::memcpy($__buf7, $label, $__n7);
            $__view7->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf7[0]));
        } else {
            $__view7->data = null;
        }
        $__view7->len = $__n7;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Routing_route_net($schematic->ptr, $sx, $sy, $sz, $dx, $dy, $dz, $__view7, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function routeAll( $schematic, string $nets_json) {
        $__n1 = strlen($nets_json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $nets_json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Routing_route_all($schematic->ptr, $__view1, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function lvs( $schematic, string $intent_json) {
        $__n1 = strlen($intent_json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $intent_json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Routing_lvs($schematic->ptr, $__view1, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function drc( $schematic,  $check_decay) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Routing_drc($schematic->ptr, $check_decay, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function sta( $schematic, string $netlist_json) {
        $__n1 = strlen($netlist_json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $netlist_json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Routing_sta($schematic->ptr, $__view1, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Routing_destroy($this->ptr);
        }
    }
}
