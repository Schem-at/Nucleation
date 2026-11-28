<?php
namespace Stencil;

final class Field {
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

    public static function constant( $value) {
        $result = Lib::ffi()->Field_constant($value);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function coordX() {
        $ret = Lib::ffi()->Field_coord_x();
        return new Field($ret, true);
    }

    public static function coordY() {
        $ret = Lib::ffi()->Field_coord_y();
        return new Field($ret, true);
    }

    public static function coordZ() {
        $ret = Lib::ffi()->Field_coord_z();
        return new Field($ret, true);
    }

    public static function fromSchematic( $schematic,  $pad) {
        $result = Lib::ffi()->Field_from_schematic($schematic->ptr, $pad);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function fromGlbBytes(array $bytes,  $max_dim,  $pad) {
        $__n0 = count($bytes);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($bytes as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Field_from_glb_bytes($__view0, $max_dim, $pad);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function fromObjStr(string $obj_text,  $max_dim,  $pad) {
        $__n0 = strlen($obj_text);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $obj_text, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Field_from_obj_str($__view0, $max_dim, $pad);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function sphere( $radius) {
        $result = Lib::ffi()->Field_sphere($radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function boxShape( $half_x,  $half_y,  $half_z,  $rounding) {
        $result = Lib::ffi()->Field_box_shape($half_x, $half_y, $half_z, $rounding);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function ellipsoid( $radius_x,  $radius_y,  $radius_z) {
        $result = Lib::ffi()->Field_ellipsoid($radius_x, $radius_y, $radius_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function torus( $major_radius,  $minor_radius) {
        $result = Lib::ffi()->Field_torus($major_radius, $minor_radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function cappedTorus( $major_radius,  $minor_radius,  $cap_angle_degrees) {
        $result = Lib::ffi()->Field_capped_torus($major_radius, $minor_radius, $cap_angle_degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function link( $major_radius,  $minor_radius,  $half_length) {
        $result = Lib::ffi()->Field_link($major_radius, $minor_radius, $half_length);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function capsule( $ax,  $ay,  $az,  $bx,  $by,  $bz,  $radius) {
        $result = Lib::ffi()->Field_capsule($ax, $ay, $az, $bx, $by, $bz, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function roundCone( $ax,  $ay,  $az,  $bx,  $by,  $bz,  $r1,  $r2) {
        $result = Lib::ffi()->Field_round_cone($ax, $ay, $az, $bx, $by, $bz, $r1, $r2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function solidAngle( $radius,  $angle_degrees) {
        $result = Lib::ffi()->Field_solid_angle($radius, $angle_degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function cutSphere( $radius,  $height) {
        $result = Lib::ffi()->Field_cut_sphere($radius, $height);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function cutHollowSphere( $radius,  $height,  $thickness) {
        $result = Lib::ffi()->Field_cut_hollow_sphere($radius, $height, $thickness);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function cappedCylinder( $radius,  $half_height) {
        $result = Lib::ffi()->Field_capped_cylinder($radius, $half_height);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function infiniteCylinder( $radius) {
        $result = Lib::ffi()->Field_infinite_cylinder($radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function cappedCone( $half_height,  $bottom_radius,  $top_radius) {
        $result = Lib::ffi()->Field_capped_cone($half_height, $bottom_radius, $top_radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function plane( $normal_x,  $normal_y,  $normal_z,  $offset) {
        $result = Lib::ffi()->Field_plane($normal_x, $normal_y, $normal_z, $offset);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function octahedron( $size) {
        $result = Lib::ffi()->Field_octahedron($size);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function hexPrism( $radius,  $half_height) {
        $result = Lib::ffi()->Field_hex_prism($radius, $half_height);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function superPrism( $half_x,  $half_y,  $half_z,  $exponent) {
        $result = Lib::ffi()->Field_super_prism($half_x, $half_y, $half_z, $exponent);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function boxFrame( $half_x,  $half_y,  $half_z,  $thickness) {
        $result = Lib::ffi()->Field_box_frame($half_x, $half_y, $half_z, $thickness);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function infiniteCone( $angle_degrees) {
        $result = Lib::ffi()->Field_infinite_cone($angle_degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function squarePyramid( $half_base,  $height) {
        $result = Lib::ffi()->Field_square_pyramid($half_base, $height);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function heightmap( $base_y,  $height) {
        $result = Lib::ffi()->Field_heightmap($base_y, $height->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function valueNoise2( $frequency,  $seed) {
        $result = Lib::ffi()->Field_value_noise2($frequency, $seed);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function valueNoise3( $frequency,  $seed) {
        $result = Lib::ffi()->Field_value_noise3($frequency, $seed);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function perlin2( $frequency,  $seed) {
        $result = Lib::ffi()->Field_perlin2($frequency, $seed);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function perlin3( $frequency,  $seed) {
        $result = Lib::ffi()->Field_perlin3($frequency, $seed);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function fbm2(int $basis,  $frequency,  $seed,  $octaves,  $lacunarity,  $gain,  $ridged) {
        $result = Lib::ffi()->Field_fbm2($basis, $frequency, $seed, $octaves, $lacunarity, $gain, $ridged);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function fbm3(int $basis,  $frequency,  $seed,  $octaves,  $lacunarity,  $gain,  $ridged) {
        $result = Lib::ffi()->Field_fbm3($basis, $frequency, $seed, $octaves, $lacunarity, $gain, $ridged);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function valueFbm3( $frequency,  $seed,  $octaves) {
        $result = Lib::ffi()->Field_value_fbm3($frequency, $seed, $octaves);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function perlinFbm3( $frequency,  $seed,  $octaves) {
        $result = Lib::ffi()->Field_perlin_fbm3($frequency, $seed, $octaves);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function ridgedFbm3( $frequency,  $seed,  $octaves) {
        $result = Lib::ffi()->Field_ridged_fbm3($frequency, $seed, $octaves);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function perlinFbm2( $frequency,  $seed,  $octaves) {
        $result = Lib::ffi()->Field_perlin_fbm2($frequency, $seed, $octaves);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function cells( $frequency,  $seed,  $jitter, int $mode,  $threshold) {
        $result = Lib::ffi()->Field_cells($frequency, $seed, $jitter, $mode, $threshold);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function neg() {
        $ret = Lib::ffi()->Field_neg($this->ptr);
        return new Field($ret, true);
    }

    public function abs() {
        $ret = Lib::ffi()->Field_abs($this->ptr);
        return new Field($ret, true);
    }

    public function sqrt() {
        $ret = Lib::ffi()->Field_sqrt($this->ptr);
        return new Field($ret, true);
    }

    public function sin() {
        $ret = Lib::ffi()->Field_sin($this->ptr);
        return new Field($ret, true);
    }

    public function cos() {
        $ret = Lib::ffi()->Field_cos($this->ptr);
        return new Field($ret, true);
    }

    public function saturate() {
        $ret = Lib::ffi()->Field_saturate($this->ptr);
        return new Field($ret, true);
    }

    public function add( $other) {
        $ret = Lib::ffi()->Field_add($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function sub( $other) {
        $ret = Lib::ffi()->Field_sub($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function mul( $other) {
        $ret = Lib::ffi()->Field_mul($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function div( $other) {
        $ret = Lib::ffi()->Field_div($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function minWith( $other) {
        $ret = Lib::ffi()->Field_min_with($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function maxWith( $other) {
        $ret = Lib::ffi()->Field_max_with($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function addConst( $value) {
        $result = Lib::ffi()->Field_add_const($this->ptr, $value);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function mulConst( $value) {
        $result = Lib::ffi()->Field_mul_const($this->ptr, $value);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function clamp( $lo,  $hi) {
        $result = Lib::ffi()->Field_clamp($this->ptr, $lo, $hi);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function smoothstep( $edge0,  $edge1) {
        $result = Lib::ffi()->Field_smoothstep($this->ptr, $edge0, $edge1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function step( $edge) {
        $result = Lib::ffi()->Field_step($this->ptr, $edge);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function remap( $in_lo,  $in_hi,  $out_lo,  $out_hi,  $clamp) {
        $result = Lib::ffi()->Field_remap($this->ptr, $in_lo, $in_hi, $out_lo, $out_hi, $clamp);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function lerpWith( $other,  $t) {
        $ret = Lib::ffi()->Field_lerp_with($this->ptr, $other->ptr, $t->ptr);
        return new Field($ret, true);
    }

    public static function select( $cond,  $then_field,  $else_field) {
        $ret = Lib::ffi()->Field_select($cond->ptr, $then_field->ptr, $else_field->ptr);
        return new Field($ret, true);
    }

    public function unionWith( $other) {
        $ret = Lib::ffi()->Field_union_with($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function intersectionWith( $other) {
        $ret = Lib::ffi()->Field_intersection_with($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function subtract( $other) {
        $ret = Lib::ffi()->Field_subtract($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function xorWith( $other) {
        $ret = Lib::ffi()->Field_xor_with($this->ptr, $other->ptr);
        return new Field($ret, true);
    }

    public function smoothUnion( $other,  $radius) {
        $result = Lib::ffi()->Field_smooth_union($this->ptr, $other->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function smoothSubtract( $other,  $radius) {
        $result = Lib::ffi()->Field_smooth_subtract($this->ptr, $other->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function smoothIntersection( $other,  $radius) {
        $result = Lib::ffi()->Field_smooth_intersection($this->ptr, $other->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function chamferUnion( $other,  $radius) {
        $result = Lib::ffi()->Field_chamfer_union($this->ptr, $other->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function chamferSubtract( $other,  $radius) {
        $result = Lib::ffi()->Field_chamfer_subtract($this->ptr, $other->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function chamferIntersection( $other,  $radius) {
        $result = Lib::ffi()->Field_chamfer_intersection($this->ptr, $other->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function stepsUnion( $other,  $radius,  $steps) {
        $result = Lib::ffi()->Field_steps_union($this->ptr, $other->ptr, $radius, $steps);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function rounded( $radius) {
        $result = Lib::ffi()->Field_rounded($this->ptr, $radius);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function shell( $thickness) {
        $result = Lib::ffi()->Field_shell($this->ptr, $thickness);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function elongate( $half_x,  $half_y,  $half_z) {
        $result = Lib::ffi()->Field_elongate($this->ptr, $half_x, $half_y, $half_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function displace( $offset,  $amplitude) {
        $result = Lib::ffi()->Field_displace($this->ptr, $offset->ptr, $amplitude);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function displaceNoise( $amplitude,  $frequency,  $seed,  $octaves) {
        $result = Lib::ffi()->Field_displace_noise($this->ptr, $amplitude, $frequency, $seed, $octaves);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function translate( $x,  $y,  $z) {
        $result = Lib::ffi()->Field_translate($this->ptr, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function rotate( $x_degrees,  $y_degrees,  $z_degrees) {
        $result = Lib::ffi()->Field_rotate($this->ptr, $x_degrees, $y_degrees, $z_degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function scale( $factor) {
        $result = Lib::ffi()->Field_scale($this->ptr, $factor);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function scaleXyz( $sx,  $sy,  $sz) {
        $result = Lib::ffi()->Field_scale_xyz($this->ptr, $sx, $sy, $sz);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function mirror(int $axis) {
        $ret = Lib::ffi()->Field_mirror($this->ptr, $axis);
        return new Field($ret, true);
    }

    public function mirrorPlane( $nx,  $ny,  $nz,  $offset) {
        $result = Lib::ffi()->Field_mirror_plane($this->ptr, $nx, $ny, $nz, $offset);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function repeatInfinite( $spacing_x,  $spacing_y,  $spacing_z) {
        $result = Lib::ffi()->Field_repeat_infinite($this->ptr, $spacing_x, $spacing_y, $spacing_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function repeatCounted( $spacing_x,  $spacing_y,  $spacing_z,  $count_x,  $count_y,  $count_z) {
        $result = Lib::ffi()->Field_repeat_counted($this->ptr, $spacing_x, $spacing_y, $spacing_z, $count_x, $count_y, $count_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function repeatPolar( $count) {
        $result = Lib::ffi()->Field_repeat_polar($this->ptr, $count);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function repeatPoints(array $offsets) {
        $__n0 = count($offsets);
        $__view0 = Lib::ffi()->new('DiplomatF32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("float[" . $__n0 . "]", false);
            foreach ($offsets as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Field_repeat_points($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function twist( $amount) {
        $result = Lib::ffi()->Field_twist($this->ptr, $amount);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function bend( $amount) {
        $result = Lib::ffi()->Field_bend($this->ptr, $amount);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function warp( $warp_x,  $warp_y,  $warp_z,  $amplitude) {
        $result = Lib::ffi()->Field_warp($this->ptr, $warp_x->ptr, $warp_y->ptr, $warp_z->ptr, $amplitude);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function warpNoise( $amplitude,  $frequency,  $seed,  $octaves) {
        $result = Lib::ffi()->Field_warp_noise($this->ptr, $amplitude, $frequency, $seed, $octaves);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function tag( $material_id) {
        $result = Lib::ffi()->Field_tag($this->ptr, $material_id);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function bounded( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $result = Lib::ffi()->Field_bounded($this->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public static function expr(string $source) {
        $__n0 = strlen($source);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $source, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Field_expr($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function evalAt( $x,  $y,  $z) {
        $ret = Lib::ffi()->Field_eval_at($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function sampleTagged( $x,  $y,  $z) {
        $ret = Lib::ffi()->Field_sample_tagged($this->ptr, $x, $y, $z);
        return FieldSample::fromFFI($ret);
    }

    public function normal( $x,  $y,  $z,  $epsilon) {
        $result = Lib::ffi()->Field_normal($this->ptr, $x, $y, $z, $epsilon);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return FieldNormal::fromFFI($result->ok);
    }

    public function bounds() {
        $result = Lib::ffi()->Field_bounds($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return FieldBounds::fromFFI($result->ok);
    }

    public function outputRange() {
        $result = Lib::ffi()->Field_output_range($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return FieldRange::fromFFI($result->ok);
    }

    public function distanceKind() {
        $ret = Lib::ffi()->Field_distance_kind($this->ptr);
        return $ret;
    }

    public function toShape() {
        $result = Lib::ffi()->Field_to_shape($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public function toShapeBounded( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $result = Lib::ffi()->Field_to_shape_bounded($this->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Shape($result->ok, true);
    }

    public static function fromJson(string $json) {
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
        $result = Lib::ffi()->Field_from_json($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Field($result->ok, true);
    }

    public function toJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Field_to_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Field_destroy($this->ptr);
        }
    }
}
