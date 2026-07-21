<?php
namespace Stencil;

final class IoType {
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

    public static function unsignedInt( $bits) {
        $ret = Lib::ffi()->IoType_unsigned_int($bits);
        return new IoType($ret, true);
    }

    public static function signedInt( $bits) {
        $ret = Lib::ffi()->IoType_signed_int($bits);
        return new IoType($ret, true);
    }

    public static function float32() {
        $ret = Lib::ffi()->IoType_float32();
        return new IoType($ret, true);
    }

    public static function boolean() {
        $ret = Lib::ffi()->IoType_boolean();
        return new IoType($ret, true);
    }

    public static function ascii( $chars) {
        $ret = Lib::ffi()->IoType_ascii($chars);
        return new IoType($ret, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->IoType_destroy($this->ptr);
        }
    }
}
