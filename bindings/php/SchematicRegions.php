<?php
namespace Stencil;

final class SchematicRegions {
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

    public static function add( $schematic, string $name,  $region) {
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
        $result = Lib::ffi()->SchematicRegions_add($schematic->ptr, $__view1, $region->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function update( $schematic, string $name,  $region) {
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
        $result = Lib::ffi()->SchematicRegions_update($schematic->ptr, $__view1, $region->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function get( $schematic, string $name) {
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
        $result = Lib::ffi()->SchematicRegions_get($schematic->ptr, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new DefinitionRegion($result->ok, true);
    }

    public static function remove( $schematic, string $name) {
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
        $result = Lib::ffi()->SchematicRegions_remove($schematic->ptr, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function namesJson( $schematic) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->SchematicRegions_names_json($schematic->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function create( $schematic, string $name) {
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
        $result = Lib::ffi()->SchematicRegions_create($schematic->ptr, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function createFromPoint( $schematic, string $name,  $x,  $y,  $z) {
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
        $result = Lib::ffi()->SchematicRegions_create_from_point($schematic->ptr, $__view1, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function createFromBounds( $schematic, string $name,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
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
        $result = Lib::ffi()->SchematicRegions_create_from_bounds($schematic->ptr, $__view1, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function createRegion( $schematic, string $name,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
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
        $result = Lib::ffi()->SchematicRegions_create_region($schematic->ptr, $__view1, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new DefinitionRegion($result->ok, true);
    }

    public static function addBoundsTo( $schematic, string $name,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
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
        $result = Lib::ffi()->SchematicRegions_add_bounds_to($schematic->ptr, $__view1, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function addPointTo( $schematic, string $name,  $x,  $y,  $z) {
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
        $result = Lib::ffi()->SchematicRegions_add_point_to($schematic->ptr, $__view1, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function setMetadataOn( $schematic, string $name, string $key, string $value) {
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
        $__n2 = strlen($key);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $key, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $__n3 = strlen($value);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $value, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->SchematicRegions_set_metadata_on($schematic->ptr, $__view1, $__view2, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function shiftRegion( $schematic, string $name,  $dx,  $dy,  $dz) {
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
        $result = Lib::ffi()->SchematicRegions_shift_region($schematic->ptr, $__view1, $dx, $dy, $dz);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->SchematicRegions_destroy($this->ptr);
        }
    }
}
