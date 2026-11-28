<?php
namespace Stencil;

final class Material {
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

    public static function solid(string $block) {
        $__n0 = strlen($block);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $block, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Material_solid($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Material($result->ok, true);
    }

    public static function skip() {
        $ret = Lib::ffi()->Material_skip();
        return new Material($ret, true);
    }

    public static function color( $r,  $g,  $b,  $palette) {
        $ret = Lib::ffi()->Material_color($r, $g, $b, $palette->ptr);
        return new Material($ret, true);
    }

    public static function fromBrush( $brush) {
        $ret = Lib::ffi()->Material_from_brush($brush->ptr);
        return new Material($ret, true);
    }

    public static function channelGradient(int $channel, array $stops, array $colors,  $lo,  $hi,  $palette, int $space) {
        $__n1 = count($stops);
        $__view1 = Lib::ffi()->new('DiplomatF32View');
        if ($__n1 > 0) {
            $__arr1 = Lib::ffi()->new("float[" . $__n1 . "]", false);
            foreach ($stops as $__i1 => $__v1) { $__arr1[$__i1] = $__v1; }
            $__view1->data = \FFI::addr($__arr1[0]);
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $__n2 = count($colors);
        $__view2 = Lib::ffi()->new('DiplomatU8View');
        if ($__n2 > 0) {
            $__arr2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            foreach ($colors as $__i2 => $__v2) { $__arr2[$__i2] = $__v2; }
            $__view2->data = \FFI::addr($__arr2[0]);
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->Material_channel_gradient($channel, $__view1, $__view2, $lo, $hi, $palette->ptr, $space);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Material($result->ok, true);
    }

    public static function fieldGradient( $field, array $stops, array $colors,  $lo,  $hi,  $palette, int $space) {
        $__n1 = count($stops);
        $__view1 = Lib::ffi()->new('DiplomatF32View');
        if ($__n1 > 0) {
            $__arr1 = Lib::ffi()->new("float[" . $__n1 . "]", false);
            foreach ($stops as $__i1 => $__v1) { $__arr1[$__i1] = $__v1; }
            $__view1->data = \FFI::addr($__arr1[0]);
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $__n2 = count($colors);
        $__view2 = Lib::ffi()->new('DiplomatU8View');
        if ($__n2 > 0) {
            $__arr2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            foreach ($colors as $__i2 => $__v2) { $__arr2[$__i2] = $__v2; }
            $__view2->data = \FFI::addr($__arr2[0]);
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->Material_field_gradient($field->ptr, $__view1, $__view2, $lo, $hi, $palette->ptr, $space);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Material($result->ok, true);
    }

    public static function channelRamp(int $channel,  $lo,  $hi, string $blocks_json,  $dither) {
        $__n3 = strlen($blocks_json);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $blocks_json, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->Material_channel_ramp($channel, $lo, $hi, $__view3, $dither);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Material($result->ok, true);
    }

    public static function fieldRamp( $field,  $lo,  $hi, string $blocks_json,  $dither) {
        $__n3 = strlen($blocks_json);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $blocks_json, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->Material_field_ramp($field->ptr, $lo, $hi, $__view3, $dither);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Material($result->ok, true);
    }

    public static function weighted(string $blocks_json, array $weights,  $seed) {
        $__n0 = strlen($blocks_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $blocks_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = count($weights);
        $__view1 = Lib::ffi()->new('DiplomatF32View');
        if ($__n1 > 0) {
            $__arr1 = Lib::ffi()->new("float[" . $__n1 . "]", false);
            foreach ($weights as $__i1 => $__v1) { $__arr1[$__i1] = $__v1; }
            $__view1->data = \FFI::addr($__arr1[0]);
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->Material_weighted($__view0, $__view1, $seed);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Material($result->ok, true);
    }

    public static function shaded( $r,  $g,  $b,  $lx,  $ly,  $lz,  $ambient,  $palette) {
        $result = Lib::ffi()->Material_shaded($r, $g, $b, $lx, $ly, $lz, $ambient, $palette->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Material($result->ok, true);
    }

    public static function spotlight( $px,  $py,  $pz,  $dx,  $dy,  $dz,  $cone_angle_deg,  $r,  $g,  $b,  $ambient,  $palette) {
        $result = Lib::ffi()->Material_spotlight($px, $py, $pz, $dx, $dy, $dz, $cone_angle_deg, $r, $g, $b, $ambient, $palette->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Material($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Material_destroy($this->ptr);
        }
    }
}
