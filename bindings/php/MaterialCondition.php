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

    public static function exposedTop() {
        $ret = Lib::ffi()->MaterialCondition_exposed_top();
        return new MaterialCondition($ret, true);
    }

    public static function exposedBottom() {
        $ret = Lib::ffi()->MaterialCondition_exposed_bottom();
        return new MaterialCondition($ret, true);
    }

    public static function exposedSide() {
        $ret = Lib::ffi()->MaterialCondition_exposed_side();
        return new MaterialCondition($ret, true);
    }

    public static function exposedAny() {
        $ret = Lib::ffi()->MaterialCondition_exposed_any();
        return new MaterialCondition($ret, true);
    }

    public static function interior() {
        $ret = Lib::ffi()->MaterialCondition_interior();
        return new MaterialCondition($ret, true);
    }

    public static function yAbove( $min_y) {
        $result = Lib::ffi()->MaterialCondition_y_above($min_y);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function yBelow( $max_y) {
        $result = Lib::ffi()->MaterialCondition_y_below($max_y);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function yBetween( $min_y,  $max_y) {
        $result = Lib::ffi()->MaterialCondition_y_between($min_y, $max_y);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function slopeBelow( $max_slope) {
        $result = Lib::ffi()->MaterialCondition_slope_below($max_slope);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function slopeAbove( $min_slope) {
        $result = Lib::ffi()->MaterialCondition_slope_above($min_slope);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MaterialCondition($result->ok, true);
    }

    public static function depthBetween( $min_depth,  $max_depth) {
        $ret = Lib::ffi()->MaterialCondition_depth_between($min_depth, $max_depth);
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
