<?php
namespace Stencil;

final class Shape {
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

    public static function tubeAlong( $curve,  $radius) {
        $result = Lib::ffi()->Shape_tube_along($curve->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public static function sphere( $cx,  $cy,  $cz,  $radius) {
        $ret = Lib::ffi()->Shape_sphere($cx, $cy, $cz, $radius);
        return new Shape($ret, true);
    }

    public static function cuboid( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $ret = Lib::ffi()->Shape_cuboid($min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        return new Shape($ret, true);
    }

    public static function polygonPrism(string $polygon_json,  $y_min,  $y_max) {
        $__n0 = strlen($polygon_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $polygon_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Shape_polygon_prism($__view0, $y_min, $y_max);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public static function ellipsoid( $cx,  $cy,  $cz,  $rx,  $ry,  $rz) {
        $ret = Lib::ffi()->Shape_ellipsoid($cx, $cy, $cz, $rx, $ry, $rz);
        return new Shape($ret, true);
    }

    public static function cylinder( $bx,  $by,  $bz,  $ax,  $ay,  $az,  $radius,  $height) {
        $ret = Lib::ffi()->Shape_cylinder($bx, $by, $bz, $ax, $ay, $az, $radius, $height);
        return new Shape($ret, true);
    }

    public static function cylinderBetween( $x1,  $y1,  $z1,  $x2,  $y2,  $z2,  $radius) {
        $ret = Lib::ffi()->Shape_cylinder_between($x1, $y1, $z1, $x2, $y2, $z2, $radius);
        return new Shape($ret, true);
    }

    public static function cone( $ax,  $ay,  $az,  $dx,  $dy,  $dz,  $radius,  $height) {
        $ret = Lib::ffi()->Shape_cone($ax, $ay, $az, $dx, $dy, $dz, $radius, $height);
        return new Shape($ret, true);
    }

    public static function torus( $cx,  $cy,  $cz,  $major_r,  $minor_r,  $ax,  $ay,  $az) {
        $ret = Lib::ffi()->Shape_torus($cx, $cy, $cz, $major_r, $minor_r, $ax, $ay, $az);
        return new Shape($ret, true);
    }

    public static function pyramid( $bx,  $by,  $bz,  $half_w,  $half_d,  $height,  $ax,  $ay,  $az) {
        $ret = Lib::ffi()->Shape_pyramid($bx, $by, $bz, $half_w, $half_d, $height, $ax, $ay, $az);
        return new Shape($ret, true);
    }

    public static function disk( $cx,  $cy,  $cz,  $radius,  $nx,  $ny,  $nz,  $thickness) {
        $ret = Lib::ffi()->Shape_disk($cx, $cy, $cz, $radius, $nx, $ny, $nz, $thickness);
        return new Shape($ret, true);
    }

    public static function plane( $ox,  $oy,  $oz,  $ux,  $uy,  $uz,  $vx,  $vy,  $vz,  $u_ext,  $v_ext,  $thickness) {
        $ret = Lib::ffi()->Shape_plane($ox, $oy, $oz, $ux, $uy, $uz, $vx, $vy, $vz, $u_ext, $v_ext, $thickness);
        return new Shape($ret, true);
    }

    public static function triangle( $ax,  $ay,  $az,  $bx,  $by,  $bz,  $cx,  $cy,  $cz,  $thickness) {
        $ret = Lib::ffi()->Shape_triangle($ax, $ay, $az, $bx, $by, $bz, $cx, $cy, $cz, $thickness);
        return new Shape($ret, true);
    }

    public static function line( $x1,  $y1,  $z1,  $x2,  $y2,  $z2,  $thickness) {
        $ret = Lib::ffi()->Shape_line($x1, $y1, $z1, $x2, $y2, $z2, $thickness);
        return new Shape($ret, true);
    }

    public static function bezier(array $control_points,  $thickness,  $resolution) {
        $__n0 = count($control_points);
        $__view0 = Lib::ffi()->new('DiplomatF32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("float[" . $__n0 . "]", false);
            foreach ($control_points as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Shape_bezier($__view0, $thickness, $resolution);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public static function sdf(string $sdf_json) {
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
        $result = Lib::ffi()->Shape_sdf($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public static function sdfBounded(string $sdf_json,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
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
        $result = Lib::ffi()->Shape_sdf_bounded($__view0, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public function hollow( $thickness) {
        $ret = Lib::ffi()->Shape_hollow($this->ptr, $thickness);
        return new Shape($ret, true);
    }

    public function unionWith( $other) {
        $ret = Lib::ffi()->Shape_union_with($this->ptr, $other->ptr);
        return new Shape($ret, true);
    }

    public function intersectionWith( $other) {
        $ret = Lib::ffi()->Shape_intersection_with($this->ptr, $other->ptr);
        return new Shape($ret, true);
    }

    public function differenceWith( $other) {
        $ret = Lib::ffi()->Shape_difference_with($this->ptr, $other->ptr);
        return new Shape($ret, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Shape_destroy($this->ptr);
        }
    }
}
