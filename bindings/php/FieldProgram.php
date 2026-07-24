<?php
namespace Stencil;

final class FieldProgram {
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

    public static function fromJsonString(string $json) {
        $__n0 = strlen($json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->FieldProgram_from_json_string($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new FieldProgram($result->ok, true);
    }

    public function toJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->FieldProgram_to_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function evalAt( $x,  $y,  $z) {
        $ret = Lib::ffi()->FieldProgram_eval_at($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function gradient( $x,  $y,  $z,  $epsilon) {
        $result = Lib::ffi()->FieldProgram_gradient($this->ptr, $x, $y, $z, $epsilon);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return SdfNormal::fromFFI($result->ok);
    }

    public function bounds() {
        $ret = Lib::ffi()->FieldProgram_bounds($this->ptr);
        return SdfBounds::fromFFI($ret);
    }

    public function distanceKind() {
        $ret = Lib::ffi()->FieldProgram_distance_kind($this->ptr);
        return $ret;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->FieldProgram_destroy($this->ptr);
        }
    }
}
