<?php
namespace Stencil;

final class OutputCondition {
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

    public static function equals( $value) {
        $ret = Lib::ffi()->OutputCondition_equals($value->ptr);
        return new OutputCondition($ret, true);
    }

    public static function notEquals( $value) {
        $ret = Lib::ffi()->OutputCondition_not_equals($value->ptr);
        return new OutputCondition($ret, true);
    }

    public static function greaterThan( $value) {
        $ret = Lib::ffi()->OutputCondition_greater_than($value->ptr);
        return new OutputCondition($ret, true);
    }

    public static function lessThan( $value) {
        $ret = Lib::ffi()->OutputCondition_less_than($value->ptr);
        return new OutputCondition($ret, true);
    }

    public static function bitwiseAnd( $mask) {
        $ret = Lib::ffi()->OutputCondition_bitwise_and($mask);
        return new OutputCondition($ret, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->OutputCondition_destroy($this->ptr);
        }
    }
}
