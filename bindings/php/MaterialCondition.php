<?php
namespace Stencil;

final class MaterialCondition {
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

    public static function always() {
        $ret = Lib::ffi()->MaterialCondition_always();
        return new MaterialCondition($ret, true);
    }

    public function andWith( $other) {
        $ret = Lib::ffi()->MaterialCondition_and_with($this->ptr, $other->ptr);
        return new MaterialCondition($ret, true);
    }

    public function orWith( $other) {
        $ret = Lib::ffi()->MaterialCondition_or_with($this->ptr, $other->ptr);
        return new MaterialCondition($ret, true);
    }

    public function negated() {
        $ret = Lib::ffi()->MaterialCondition_negated($this->ptr);
        return new MaterialCondition($ret, true);
    }

    public static function tag( $material_id) {
        $ret = Lib::ffi()->MaterialCondition_tag($material_id);
        return new MaterialCondition($ret, true);
    }

    public static function channelRange(int $channel,  $has_min,  $min,  $has_max,  $max) {
        $result = Lib::ffi()->MaterialCondition_channel_range($channel, $has_min, $min, $has_max, $max);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function fieldRange( $field,  $has_min,  $min,  $has_max,  $max) {
        $result = Lib::ffi()->MaterialCondition_field_range($field->ptr, $has_min, $min, $has_max, $max);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function yRange( $has_min,  $min,  $has_max,  $max) {
        $result = Lib::ffi()->MaterialCondition_y_range($has_min, $min, $has_max, $max);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function depthBelowSurface( $has_min,  $min,  $has_max,  $max) {
        $ret = Lib::ffi()->MaterialCondition_depth_below_surface($has_min, $min, $has_max, $max);
        return new MaterialCondition($ret, true);
    }

    public static function slopeRange( $has_min,  $min,  $has_max,  $max) {
        $result = Lib::ffi()->MaterialCondition_slope_range($has_min, $min, $has_max, $max);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function curvatureRange( $has_min,  $min,  $has_max,  $max) {
        $result = Lib::ffi()->MaterialCondition_curvature_range($has_min, $min, $has_max, $max);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function normalDot( $dx,  $dy,  $dz,  $has_min,  $min,  $has_max,  $max) {
        $result = Lib::ffi()->MaterialCondition_normal_dot($dx, $dy, $dz, $has_min, $min, $has_max, $max);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function exposedFace( $mask,  $require_all) {
        $ret = Lib::ffi()->MaterialCondition_exposed_face($mask, $require_all);
        return new MaterialCondition($ret, true);
    }

    public static function chance( $probability,  $seed) {
        $result = Lib::ffi()->MaterialCondition_chance($probability, $seed);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->MaterialCondition_destroy($this->ptr);
        }
    }
}
