<?php
namespace Stencil;

final class Brush {
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

    public static function solid(string $block_name) {
        $__n0 = strlen($block_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $block_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Brush_solid($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Brush($result->ok, true);
    }

    public static function color( $r,  $g,  $b) {
        $ret = Lib::ffi()->Brush_color($r, $g, $b);
        return new Brush($ret, true);
    }

    public static function linearGradient( $x1,  $y1,  $z1,  $r1,  $g1,  $b1,  $x2,  $y2,  $z2,  $r2,  $g2,  $b2, int $space) {
        $ret = Lib::ffi()->Brush_linear_gradient($x1, $y1, $z1, $r1, $g1, $b1, $x2, $y2, $z2, $r2, $g2, $b2, $space);
        return new Brush($ret, true);
    }

    public static function shaded( $r,  $g,  $b,  $lx,  $ly,  $lz) {
        $ret = Lib::ffi()->Brush_shaded($r, $g, $b, $lx, $ly, $lz);
        return new Brush($ret, true);
    }

    public static function bilinearGradient( $ox,  $oy,  $oz,  $ux,  $uy,  $uz,  $vx,  $vy,  $vz,  $r00,  $g00,  $b00,  $r10,  $g10,  $b10,  $r01,  $g01,  $b01,  $r11,  $g11,  $b11, int $space) {
        $ret = Lib::ffi()->Brush_bilinear_gradient($ox, $oy, $oz, $ux, $uy, $uz, $vx, $vy, $vz, $r00, $g00, $b00, $r10, $g10, $b10, $r01, $g01, $b01, $r11, $g11, $b11, $space);
        return new Brush($ret, true);
    }

    public static function pointGradient(array $positions, array $colors,  $falloff, int $space) {
        $__n0 = count($positions);
        $__view0 = Lib::ffi()->new('DiplomatI32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("int32_t[" . $__n0 . "]", false);
            foreach ($positions as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = count($colors);
        $__view1 = Lib::ffi()->new('DiplomatU8View');
        if ($__n1 > 0) {
            $__arr1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            foreach ($colors as $__i1 => $__v1) { $__arr1[$__i1] = $__v1; }
            $__view1->data = \FFI::addr($__arr1[0]);
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->Brush_point_gradient($__view0, $__view1, $falloff, $space);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Brush($result->ok, true);
    }

    public static function spotlight( $px,  $py,  $pz,  $dx,  $dy,  $dz,  $cone_angle_deg,  $r,  $g,  $b) {
        $ret = Lib::ffi()->Brush_spotlight($px, $py, $pz, $dx, $dy, $dz, $cone_angle_deg, $r, $g, $b);
        return new Brush($ret, true);
    }

    public function setPalette( $palette) {
        Lib::ffi()->Brush_set_palette($this->ptr, $palette->ptr);
    }

    public static function curveGradient(array $stops, array $colors, int $space) {
        $__n0 = count($stops);
        $__view0 = Lib::ffi()->new('DiplomatF32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("float[" . $__n0 . "]", false);
            foreach ($stops as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = count($colors);
        $__view1 = Lib::ffi()->new('DiplomatU8View');
        if ($__n1 > 0) {
            $__arr1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            foreach ($colors as $__i1 => $__v1) { $__arr1[$__i1] = $__v1; }
            $__view1->data = \FFI::addr($__arr1[0]);
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->Brush_curve_gradient($__view0, $__view1, $space);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Brush($result->ok, true);
    }

    public static function field(string $field_json, array $stops, array $colors,  $lo,  $hi, int $space) {
        $__n0 = strlen($field_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $field_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
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
        $result = Lib::ffi()->Brush_field($__view0, $__view1, $__view2, $lo, $hi, $space);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Brush($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Brush_destroy($this->ptr);
        }
    }
}
