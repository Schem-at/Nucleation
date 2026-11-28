<?php
namespace Stencil;

final class ScatterRule {
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

    public static function create(int $face,  $density, string $blocks_json, array $weights,  $seed) {
        $__n2 = strlen($blocks_json);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $blocks_json, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $__n3 = count($weights);
        $__view3 = Lib::ffi()->new('DiplomatF32View');
        if ($__n3 > 0) {
            $__arr3 = Lib::ffi()->new("float[" . $__n3 . "]", false);
            foreach ($weights as $__i3 => $__v3) { $__arr3[$__i3] = $__v3; }
            $__view3->data = \FFI::addr($__arr3[0]);
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->ScatterRule_create($face, $density, $__view2, $__view3, $seed);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new ScatterRule($result->ok, true);
    }

    public function setCondition( $condition) {
        Lib::ffi()->ScatterRule_set_condition($this->ptr, $condition->ptr);
    }

    public function setOnBlock(string $prefix) {
        $__n0 = strlen($prefix);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $prefix, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->ScatterRule_set_on_block($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setHeightRange( $min,  $max) {
        $result = Lib::ffi()->ScatterRule_set_height_range($this->ptr, $min, $max);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->ScatterRule_destroy($this->ptr);
        }
    }
}
