<?php
namespace Stencil;

final class AnimationEffect {
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

    public static function create( $duration_ms) {
        $ret = Lib::ffi()->AnimationEffect_create($duration_ms);
        return new AnimationEffect($ret, true);
    }

    public static function instant() {
        $ret = Lib::ffi()->AnimationEffect_instant();
        return new AnimationEffect($ret, true);
    }

    public static function popIn( $duration_ms) {
        $ret = Lib::ffi()->AnimationEffect_pop_in($duration_ms);
        return new AnimationEffect($ret, true);
    }

    public static function dropIn( $duration_ms,  $height) {
        $ret = Lib::ffi()->AnimationEffect_drop_in($duration_ms, $height);
        return new AnimationEffect($ret, true);
    }

    public static function dropAndPop( $duration_ms,  $height) {
        $ret = Lib::ffi()->AnimationEffect_drop_and_pop($duration_ms, $height);
        return new AnimationEffect($ret, true);
    }

    public static function spinIn( $duration_ms,  $turns) {
        $ret = Lib::ffi()->AnimationEffect_spin_in($duration_ms, $turns);
        return new AnimationEffect($ret, true);
    }

    public static function turntable( $duration_ms) {
        $ret = Lib::ffi()->AnimationEffect_turntable($duration_ms);
        return new AnimationEffect($ret, true);
    }

    public function addTween(string $property_name,  $from,  $to, string $easing_name) {
        $__n0 = strlen($property_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $property_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n3 = strlen($easing_name);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $easing_name, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->AnimationEffect_add_tween($this->ptr, $__view0, $from, $to, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addKeyframe(string $property_name,  $at,  $value, string $easing_name) {
        $__n0 = strlen($property_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $property_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n3 = strlen($easing_name);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $easing_name, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->AnimationEffect_add_keyframe($this->ptr, $__view0, $at, $value, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setRepeatForever() {
        Lib::ffi()->AnimationEffect_set_repeat_forever($this->ptr);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->AnimationEffect_destroy($this->ptr);
        }
    }
}
