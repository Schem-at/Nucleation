<?php
namespace Stencil;

final class ExecutionMode {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function fixedTicks( $ticks) {
        $ret = Lib::ffi()->ExecutionMode_fixed_ticks($ticks);
        return new ExecutionMode($ret, true);
    }

    public static function untilCondition(string $output_name,  $condition,  $max_ticks,  $check_interval) {
        $__n0 = strlen($output_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $output_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->ExecutionMode_until_condition($__view0, $condition->ptr, $max_ticks, $check_interval);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new ExecutionMode($result->ok, true);
    }

    public static function untilChange( $max_ticks,  $check_interval) {
        $ret = Lib::ffi()->ExecutionMode_until_change($max_ticks, $check_interval);
        return new ExecutionMode($ret, true);
    }

    public static function untilStable( $stable_ticks,  $max_ticks) {
        $ret = Lib::ffi()->ExecutionMode_until_stable($stable_ticks, $max_ticks);
        return new ExecutionMode($ret, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->ExecutionMode_destroy($this->ptr);
        }
    }
}
