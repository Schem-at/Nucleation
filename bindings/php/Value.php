<?php
namespace Stencil;

final class Value {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function fromU32( $v) {
        $ret = Lib::ffi()->Value_from_u32($v);
        return new Value($ret, true);
    }

    public static function fromI32( $v) {
        $ret = Lib::ffi()->Value_from_i32($v);
        return new Value($ret, true);
    }

    public static function fromF32( $v) {
        $ret = Lib::ffi()->Value_from_f32($v);
        return new Value($ret, true);
    }

    public static function fromBool( $v) {
        $ret = Lib::ffi()->Value_from_bool($v);
        return new Value($ret, true);
    }

    public static function fromString(string $s) {
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
        $result = Lib::ffi()->Value_from_string($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Value($result->ok, true);
    }

    public function asU32() {
        $result = Lib::ffi()->Value_as_u32($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function asI32() {
        $result = Lib::ffi()->Value_as_i32($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function asF32() {
        $result = Lib::ffi()->Value_as_f32($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function asBool() {
        $result = Lib::ffi()->Value_as_bool($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function asString() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Value_as_string($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function typeName() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Value_type_name($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Value_destroy($this->ptr);
        }
    }
}
