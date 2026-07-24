<?php
namespace Stencil;

final class Sdf {
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

    public static function sphere( $radius) {
        $result = Lib::ffi()->Sdf_sphere($radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function boxShape( $half_x,  $half_y,  $half_z,  $rounding) {
        $result = Lib::ffi()->Sdf_box_shape($half_x, $half_y, $half_z, $rounding);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function ellipsoid( $radius_x,  $radius_y,  $radius_z) {
        $result = Lib::ffi()->Sdf_ellipsoid($radius_x, $radius_y, $radius_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function torus( $major_radius,  $minor_radius) {
        $result = Lib::ffi()->Sdf_torus($major_radius, $minor_radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function capsule( $ax,  $ay,  $az,  $bx,  $by,  $bz,  $radius) {
        $result = Lib::ffi()->Sdf_capsule($ax, $ay, $az, $bx, $by, $bz, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function cappedCylinder( $radius,  $half_height) {
        $result = Lib::ffi()->Sdf_capped_cylinder($radius, $half_height);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function cappedCone( $half_height,  $bottom_radius,  $top_radius) {
        $result = Lib::ffi()->Sdf_capped_cone($half_height, $bottom_radius, $top_radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function plane( $normal_x,  $normal_y,  $normal_z,  $offset) {
        $result = Lib::ffi()->Sdf_plane($normal_x, $normal_y, $normal_z, $offset);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function octahedron( $size) {
        $result = Lib::ffi()->Sdf_octahedron($size);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function hexPrism( $radius,  $half_height) {
        $result = Lib::ffi()->Sdf_hex_prism($radius, $half_height);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function superPrism( $half_x,  $half_y,  $half_z,  $exponent) {
        $result = Lib::ffi()->Sdf_super_prism($half_x, $half_y, $half_z, $exponent);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public static function cells( $frequency,  $seed,  $jitter, int $mode,  $threshold) {
        $result = Lib::ffi()->Sdf_cells($frequency, $seed, $jitter, $mode, $threshold);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function unionWith( $other) {
        $ret = Lib::ffi()->Sdf_union_with($this->ptr, $other->ptr);
        return new Sdf($ret, true);
    }

    public function intersectionWith( $other) {
        $ret = Lib::ffi()->Sdf_intersection_with($this->ptr, $other->ptr);
        return new Sdf($ret, true);
    }

    public function subtract( $other) {
        $ret = Lib::ffi()->Sdf_subtract($this->ptr, $other->ptr);
        return new Sdf($ret, true);
    }

    public function smoothUnion( $other,  $radius) {
        $result = Lib::ffi()->Sdf_smooth_union($this->ptr, $other->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function smoothSubtract( $other,  $radius) {
        $result = Lib::ffi()->Sdf_smooth_subtract($this->ptr, $other->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function smoothIntersection( $other,  $radius) {
        $result = Lib::ffi()->Sdf_smooth_intersection($this->ptr, $other->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function rounded( $radius) {
        $result = Lib::ffi()->Sdf_rounded($this->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function shell( $thickness) {
        $result = Lib::ffi()->Sdf_shell($this->ptr, $thickness);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function translate( $x,  $y,  $z) {
        $result = Lib::ffi()->Sdf_translate($this->ptr, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function rotate( $x_degrees,  $y_degrees,  $z_degrees) {
        $result = Lib::ffi()->Sdf_rotate($this->ptr, $x_degrees, $y_degrees, $z_degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function scale( $factor) {
        $result = Lib::ffi()->Sdf_scale($this->ptr, $factor);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function mirror(int $axis) {
        $ret = Lib::ffi()->Sdf_mirror($this->ptr, $axis);
        return new Sdf($ret, true);
    }

    public function repeatInfinite( $spacing_x,  $spacing_y,  $spacing_z) {
        $result = Lib::ffi()->Sdf_repeat_infinite($this->ptr, $spacing_x, $spacing_y, $spacing_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function repeatCounted( $spacing_x,  $spacing_y,  $spacing_z,  $count_x,  $count_y,  $count_z) {
        $result = Lib::ffi()->Sdf_repeat_counted($this->ptr, $spacing_x, $spacing_y, $spacing_z, $count_x, $count_y, $count_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function displace( $amplitude,  $frequency,  $seed,  $octaves) {
        $result = Lib::ffi()->Sdf_displace($this->ptr, $amplitude, $frequency, $seed, $octaves);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function warp( $amplitude,  $frequency,  $seed) {
        $result = Lib::ffi()->Sdf_warp($this->ptr, $amplitude, $frequency, $seed);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function evalAt( $x,  $y,  $z) {
        $ret = Lib::ffi()->Sdf_eval_at($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function normal( $x,  $y,  $z,  $epsilon) {
        $result = Lib::ffi()->Sdf_normal($this->ptr, $x, $y, $z, $epsilon);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return SdfNormal::fromFFI($result->ok);
    }

    public function bounds() {
        $result = Lib::ffi()->Sdf_bounds($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return SdfBounds::fromFFI($result->ok);
    }

    public function toShape() {
        $result = Lib::ffi()->Sdf_to_shape($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public function toShapeBounded( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $result = Lib::ffi()->Sdf_to_shape_bounded($this->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
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
        $result = Lib::ffi()->Sdf_from_json_string($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Sdf($result->ok, true);
    }

    public function toJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Sdf_to_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function schematicFromSdfAuto(string $sdf_json, string $rules_json) {
        $__n0 = strlen($sdf_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $sdf_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($rules_json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $rules_json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->Sdf_schematic_from_sdf_auto($__view0, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function schematicFromSdf(string $sdf_json, string $rules_json,  $has_bounds,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $__n0 = strlen($sdf_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $sdf_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($rules_json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $rules_json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->Sdf_schematic_from_sdf($__view0, $__view1, $has_bounds, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function eval(string $sdf_json,  $x,  $y,  $z) {
        $__n0 = strlen($sdf_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $sdf_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Sdf_eval($__view0, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Sdf_destroy($this->ptr);
        }
    }
}
