<?php
namespace Stencil;

final class DefinitionRegion {
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

    public static function create() {
        $ret = Lib::ffi()->DefinitionRegion_create();
        return new DefinitionRegion($ret, true);
    }

    public static function fromBounds( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $ret = Lib::ffi()->DefinitionRegion_from_bounds($min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        return new DefinitionRegion($ret, true);
    }

    public static function fromPositions(array $positions) {
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
        $result = Lib::ffi()->DefinitionRegion_from_positions($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new DefinitionRegion($result->ok, true);
    }

    public static function fromBoundingBoxes(array $boxes) {
        $__n0 = count($boxes);
        $__view0 = Lib::ffi()->new('DiplomatI32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("int32_t[" . $__n0 . "]", false);
            foreach ($boxes as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->DefinitionRegion_from_bounding_boxes($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new DefinitionRegion($result->ok, true);
    }

    public function addBounds( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        Lib::ffi()->DefinitionRegion_add_bounds($this->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
    }

    public function addPoint( $x,  $y,  $z) {
        Lib::ffi()->DefinitionRegion_add_point($this->ptr, $x, $y, $z);
    }

    public function setMetadata(string $key, string $value) {
        $__n0 = strlen($key);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $key, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($value);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $value, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->DefinitionRegion_set_metadata($this->ptr, $__view0, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function getMetadata(string $key) {
        $__n0 = strlen($key);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $key, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->DefinitionRegion_get_metadata($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function allMetadataJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->DefinitionRegion_all_metadata_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function metadataKeysJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->DefinitionRegion_metadata_keys_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function addFilter(string $filter) {
        $__n0 = strlen($filter);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $filter, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->DefinitionRegion_add_filter($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function isEmpty() {
        $ret = Lib::ffi()->DefinitionRegion_is_empty($this->ptr);
        return $ret;
    }

    public function volume() {
        $ret = Lib::ffi()->DefinitionRegion_volume($this->ptr);
        return $ret;
    }

    public function contains( $x,  $y,  $z) {
        $ret = Lib::ffi()->DefinitionRegion_contains($this->ptr, $x, $y, $z);
        return $ret;
    }

    public function shift( $dx,  $dy,  $dz) {
        Lib::ffi()->DefinitionRegion_shift($this->ptr, $dx, $dy, $dz);
    }

    public function expand( $x,  $y,  $z) {
        Lib::ffi()->DefinitionRegion_expand($this->ptr, $x, $y, $z);
    }

    public function contract( $amount) {
        Lib::ffi()->DefinitionRegion_contract($this->ptr, $amount);
    }

    public function intersected( $other) {
        $ret = Lib::ffi()->DefinitionRegion_intersected($this->ptr, $other->ptr);
        return new DefinitionRegion($ret, true);
    }

    public function unionWith( $other) {
        $ret = Lib::ffi()->DefinitionRegion_union_with($this->ptr, $other->ptr);
        return new DefinitionRegion($ret, true);
    }

    public function subtracted( $other) {
        $ret = Lib::ffi()->DefinitionRegion_subtracted($this->ptr, $other->ptr);
        return new DefinitionRegion($ret, true);
    }

    public function merge( $other) {
        Lib::ffi()->DefinitionRegion_merge($this->ptr, $other->ptr);
    }

    public function unionInto( $other) {
        Lib::ffi()->DefinitionRegion_union_into($this->ptr, $other->ptr);
    }

    public function bounds() {
        $result = Lib::ffi()->DefinitionRegion_bounds($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return RegionBounds::fromFFI($result->ok);
    }

    public function dimensions() {
        $ret = Lib::ffi()->DefinitionRegion_dimensions($this->ptr);
        return Dimensions::fromFFI($ret);
    }

    public function center() {
        $result = Lib::ffi()->DefinitionRegion_center($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return BlockPos::fromFFI($result->ok);
    }

    public function centerF32Json() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->DefinitionRegion_center_f32_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function positionsJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->DefinitionRegion_positions_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function positionsSortedJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->DefinitionRegion_positions_sorted_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function boxCount() {
        $ret = Lib::ffi()->DefinitionRegion_box_count($this->ptr);
        return $ret;
    }

    public function getBox( $index) {
        $result = Lib::ffi()->DefinitionRegion_get_box($this->ptr, $index);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return RegionBounds::fromFFI($result->ok);
    }

    public function boxesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->DefinitionRegion_boxes_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function isContiguous() {
        $ret = Lib::ffi()->DefinitionRegion_is_contiguous($this->ptr);
        return $ret;
    }

    public function connectedComponents() {
        $ret = Lib::ffi()->DefinitionRegion_connected_components($this->ptr);
        return $ret;
    }

    public function simplify() {
        Lib::ffi()->DefinitionRegion_simplify($this->ptr);
    }

    public function filterByBlock( $schematic, string $block_name) {
        $__n1 = strlen($block_name);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $block_name, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->DefinitionRegion_filter_by_block($this->ptr, $schematic->ptr, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new DefinitionRegion($result->ok, true);
    }

    public function filterByProperties( $schematic, string $properties_json) {
        $__n1 = strlen($properties_json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $properties_json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->DefinitionRegion_filter_by_properties($this->ptr, $schematic->ptr, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new DefinitionRegion($result->ok, true);
    }

    public function excludeBlock( $schematic, string $block_name) {
        $__n1 = strlen($block_name);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $block_name, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->DefinitionRegion_exclude_block($this->ptr, $schematic->ptr, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function intersectsBounds( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $ret = Lib::ffi()->DefinitionRegion_intersects_bounds($this->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        return $ret;
    }

    public function shifted( $dx,  $dy,  $dz) {
        $ret = Lib::ffi()->DefinitionRegion_shifted($this->ptr, $dx, $dy, $dz);
        return new DefinitionRegion($ret, true);
    }

    public function expanded( $x,  $y,  $z) {
        $ret = Lib::ffi()->DefinitionRegion_expanded($this->ptr, $x, $y, $z);
        return new DefinitionRegion($ret, true);
    }

    public function contracted( $amount) {
        $ret = Lib::ffi()->DefinitionRegion_contracted($this->ptr, $amount);
        return new DefinitionRegion($ret, true);
    }

    public function copy() {
        $ret = Lib::ffi()->DefinitionRegion_copy($this->ptr);
        return new DefinitionRegion($ret, true);
    }

    public function setColor( $color) {
        Lib::ffi()->DefinitionRegion_set_color($this->ptr, $color);
    }

    public function blocksJson( $schematic) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->DefinitionRegion_blocks_json($this->ptr, $schematic->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function sync( $schematic, string $name) {
        $__n1 = strlen($name);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $name, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->DefinitionRegion_sync($this->ptr, $schematic->ptr, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->DefinitionRegion_destroy($this->ptr);
        }
    }
}
