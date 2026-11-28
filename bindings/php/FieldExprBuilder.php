<?php
namespace Stencil;

final class FieldExprBuilder {
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

    public static function create() {
        $ret = Lib::ffi()->FieldExprBuilder_create();
        return new FieldExprBuilder($ret, true);
    }

    public function bind(string $name,  $field) {
        $__n0 = strlen($name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->FieldExprBuilder_bind($this->ptr, $__view0, $field->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setBounds( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $result = Lib::ffi()->FieldExprBuilder_set_bounds($this->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setDistanceKind(int $kind) {
        Lib::ffi()->FieldExprBuilder_set_distance_kind($this->ptr, $kind);
    }

    public function compile(string $source) {
        $__n0 = strlen($source);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $source, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->FieldExprBuilder_compile($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->FieldExprBuilder_destroy($this->ptr);
        }
    }
}
