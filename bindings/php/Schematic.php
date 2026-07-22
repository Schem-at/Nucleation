<?php
namespace Stencil;

final class Schematic {
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

    public static function create(string $name) {
        $__n0 = strlen($name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $ret = Lib::ffi()->Schematic_create($__view0);
        return new Schematic($ret, true);
    }

    public function deepClone() {
        $ret = Lib::ffi()->Schematic_deep_clone($this->ptr);
        return new Schematic($ret, true);
    }

    public function dimensions() {
        $ret = Lib::ffi()->Schematic_dimensions($this->ptr);
        return Dimensions::fromFFI($ret);
    }

    public function setBlock( $x,  $y,  $z, string $block_name) {
        $__n3 = strlen($block_name);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $block_name, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->Schematic_set_block($this->ptr, $x, $y, $z, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function getBlockName( $x,  $y,  $z) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_get_block_name($this->ptr, $x, $y, $z, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function saveToFile(string $path) {
        $__n0 = strlen($path);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $path, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_save_to_file($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function loadFromFile(string $path) {
        $__n0 = strlen($path);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $path, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_load_from_file($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function fromData(array $data) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_data($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function fromLitematic(array $data) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_litematic($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public function toLitematicB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_to_litematic_b64($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function fromSchematic(array $data) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_schematic($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public function toSchematicB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_to_schematic_b64($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function fromSnapshot(array $data) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_snapshot($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public function toSnapshotB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_to_snapshot_b64($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function fromMcstructure(array $data) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_mcstructure($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public function toMcstructureB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_to_mcstructure_b64($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function fromMca(array $data) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_mca($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function fromMcaBounded(array $data,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_mca_bounded($__view0, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function fromWorldZip(array $data) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_world_zip($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function fromWorldZipBounded(array $data,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_world_zip_bounded($__view0, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function fromWorldDirectory(string $path) {
        $__n0 = strlen($path);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $path, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_world_directory($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function fromWorldDirectoryBounded(string $path,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $__n0 = strlen($path);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $path, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_from_world_directory_bounded($__view0, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public function toWorldJson(string $options_json) {
        $__n0 = strlen($options_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $options_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_to_world_json($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function saveWorld(string $directory, string $options_json) {
        $__n0 = strlen($directory);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $directory, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($options_json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $options_json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->Schematic_save_world($this->ptr, $__view0, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function toWorldZipB64(string $options_json) {
        $__n0 = strlen($options_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $options_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_to_world_zip_b64($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function setBlockWithProperties( $x,  $y,  $z, string $block_name, string $properties_json) {
        $__n3 = strlen($block_name);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $block_name, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $__n4 = strlen($properties_json);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $properties_json, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->Schematic_set_block_with_properties($this->ptr, $x, $y, $z, $__view3, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setBlockFromString( $x,  $y,  $z, string $block_string) {
        $__n3 = strlen($block_string);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $block_string, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->Schematic_set_block_from_string($this->ptr, $x, $y, $z, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function prepareBlock(string $block_name) {
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
        $result = Lib::ffi()->Schematic_prepare_block($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function place( $x,  $y,  $z,  $palette_index) {
        $result = Lib::ffi()->Schematic_place($this->ptr, $x, $y, $z, $palette_index);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setBlocks(array $positions, string $block_name) {
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
        $result = Lib::ffi()->Schematic_set_blocks($this->ptr, $__view0, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function getBlocksJson(array $positions) {
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
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_get_blocks_json($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function stampBox( $source,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z,  $target_x,  $target_y,  $target_z, string $excluded_blocks_json) {
        $__n10 = strlen($excluded_blocks_json);
        $__view10 = Lib::ffi()->new('DiplomatStringView');
        if ($__n10 > 0) {
            $__buf10 = Lib::ffi()->new("uint8_t[" . $__n10 . "]", false);
            \FFI::memcpy($__buf10, $excluded_blocks_json, $__n10);
            $__view10->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf10[0]));
        } else {
            $__view10->data = null;
        }
        $__view10->len = $__n10;
        $result = Lib::ffi()->Schematic_stamp_box($this->ptr, $source->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z, $target_x, $target_y, $target_z, $__view10);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function stampRegion( $source, string $source_region_name,  $target_x,  $target_y,  $target_z, string $excluded_blocks_json) {
        $__n1 = strlen($source_region_name);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $source_region_name, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $__n5 = strlen($excluded_blocks_json);
        $__view5 = Lib::ffi()->new('DiplomatStringView');
        if ($__n5 > 0) {
            $__buf5 = Lib::ffi()->new("uint8_t[" . $__n5 . "]", false);
            \FFI::memcpy($__buf5, $excluded_blocks_json, $__n5);
            $__view5->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf5[0]));
        } else {
            $__view5->data = null;
        }
        $__view5->len = $__n5;
        $result = Lib::ffi()->Schematic_stamp_region($this->ptr, $source->ptr, $__view1, $target_x, $target_y, $target_z, $__view5);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function copyRegion( $source,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z,  $target_x,  $target_y,  $target_z, string $excluded_blocks_json) {
        $__n10 = strlen($excluded_blocks_json);
        $__view10 = Lib::ffi()->new('DiplomatStringView');
        if ($__n10 > 0) {
            $__buf10 = Lib::ffi()->new("uint8_t[" . $__n10 . "]", false);
            \FFI::memcpy($__buf10, $excluded_blocks_json, $__n10);
            $__view10->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf10[0]));
        } else {
            $__view10->data = null;
        }
        $__view10->len = $__n10;
        $result = Lib::ffi()->Schematic_copy_region($this->ptr, $source->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z, $target_x, $target_y, $target_z, $__view10);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function getBlock( $x,  $y,  $z) {
        $result = Lib::ffi()->Schematic_get_block($this->ptr, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new BlockState($result->ok, true);
    }

    public function getBlockWithProperties( $x,  $y,  $z) {
        $result = Lib::ffi()->Schematic_get_block_with_properties($this->ptr, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new BlockState($result->ok, true);
    }

    public function getBlockInRegion(string $region_name,  $x,  $y,  $z) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_get_block_in_region($this->ptr, $__view0, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new BlockState($result->ok, true);
    }

    public function getBlockStringInRegion(string $region_name,  $x,  $y,  $z) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_get_block_string_in_region($this->ptr, $__view0, $x, $y, $z, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function getBlockString( $x,  $y,  $z) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_get_block_string($this->ptr, $x, $y, $z, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function getBlockEntityJson( $x,  $y,  $z) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_get_block_entity_json($this->ptr, $x, $y, $z, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function getBlockEntityJsonInRegion(string $region_name,  $x,  $y,  $z) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_get_block_entity_json_in_region($this->ptr, $__view0, $x, $y, $z, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function getAllBlockEntitiesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_get_all_block_entities_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function entityCount() {
        $ret = Lib::ffi()->Schematic_entity_count($this->ptr);
        return $ret;
    }

    public function getEntitiesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_get_entities_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function addEntity(string $id,  $x,  $y,  $z, string $nbt_json) {
        $__n0 = strlen($id);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $id, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n4 = strlen($nbt_json);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $nbt_json, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->Schematic_add_entity($this->ptr, $__view0, $x, $y, $z, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addArmorStand( $x,  $y,  $z,  $yaw, string $armor_material) {
        $__n4 = strlen($armor_material);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $armor_material, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->Schematic_add_armor_stand($this->ptr, $x, $y, $z, $yaw, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function removeEntity( $index) {
        $result = Lib::ffi()->Schematic_remove_entity($this->ptr, $index);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function canonicalDataVersion() {
        $ret = Lib::ffi()->Schematic_canonical_data_version();
        return $ret;
    }

    public function convertToDataVersion( $target_data_version,  $source_data_version) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_convert_to_data_version($this->ptr, $target_data_version, $source_data_version, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function convertToVersion( $target_data_version) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_convert_to_version($this->ptr, $target_data_version, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function sourceDataVersion() {
        $ret = Lib::ffi()->Schematic_source_data_version($this->ptr);
        return $ret;
    }

    public function setSourceDataVersion( $version) {
        Lib::ffi()->Schematic_set_source_data_version($this->ptr, $version);
    }

    public function toLitematicForVersionJson( $target_data_version) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_to_litematic_for_version_json($this->ptr, $target_data_version, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function getBlockEntitySnbt( $x,  $y,  $z) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_get_block_entity_snbt($this->ptr, $x, $y, $z, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function setBlockEntity( $x,  $y,  $z, string $id, string $snbt) {
        $__n3 = strlen($id);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $id, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $__n4 = strlen($snbt);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $snbt, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->Schematic_set_block_entity($this->ptr, $x, $y, $z, $__view3, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function removeBlockEntity( $x,  $y,  $z) {
        $result = Lib::ffi()->Schematic_remove_block_entity($this->ptr, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function getAllBlockEntitiesSnbtJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_get_all_block_entities_snbt_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function getEntitiesSnbtJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_get_entities_snbt_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function addEntityFromSnbt(string $snbt) {
        $__n0 = strlen($snbt);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $snbt, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_add_entity_from_snbt($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function getAllBlocksJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_get_all_blocks_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function getChunkBlocksJson( $offset_x,  $offset_y,  $offset_z,  $width,  $height,  $length) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_get_chunk_blocks_json($this->ptr, $offset_x, $offset_y, $offset_z, $width, $height, $length, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function getChunksJson( $chunk_width,  $chunk_height,  $chunk_length) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_get_chunks_json($this->ptr, $chunk_width, $chunk_height, $chunk_length, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function getChunksWithStrategyJson( $chunk_width,  $chunk_height,  $chunk_length, string $strategy,  $camera_x,  $camera_y,  $camera_z) {
        $__n3 = strlen($strategy);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $strategy, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_get_chunks_with_strategy_json($this->ptr, $chunk_width, $chunk_height, $chunk_length, $__view3, $camera_x, $camera_y, $camera_z, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function blockCount() {
        $ret = Lib::ffi()->Schematic_block_count($this->ptr);
        return $ret;
    }

    public function volume() {
        $ret = Lib::ffi()->Schematic_volume($this->ptr);
        return $ret;
    }

    public function regionNamesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_region_names_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function debugInfo() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_debug_info($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function printString() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_print_string($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function printSchematicString() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_print_schematic_string($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function debugString() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_debug_string($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function debugJsonString() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_debug_json_string($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function name() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_name($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function setName(string $name) {
        $__n0 = strlen($name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_set_name($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function author() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_author($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function setAuthor(string $author) {
        $__n0 = strlen($author);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $author, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_set_author($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function description() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_description($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function setDescription(string $description) {
        $__n0 = strlen($description);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $description, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_set_description($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function created() {
        $ret = Lib::ffi()->Schematic_created($this->ptr);
        return $ret;
    }

    public function setCreated( $created) {
        Lib::ffi()->Schematic_set_created($this->ptr, $created);
    }

    public function modified() {
        $ret = Lib::ffi()->Schematic_modified($this->ptr);
        return $ret;
    }

    public function setModified( $modified) {
        Lib::ffi()->Schematic_set_modified($this->ptr, $modified);
    }

    public function lmVersion() {
        $ret = Lib::ffi()->Schematic_lm_version($this->ptr);
        return $ret;
    }

    public function setLmVersion( $version) {
        Lib::ffi()->Schematic_set_lm_version($this->ptr, $version);
    }

    public function mcVersion() {
        $ret = Lib::ffi()->Schematic_mc_version($this->ptr);
        return $ret;
    }

    public function setMcVersion( $version) {
        Lib::ffi()->Schematic_set_mc_version($this->ptr, $version);
    }

    public function weVersion() {
        $ret = Lib::ffi()->Schematic_we_version($this->ptr);
        return $ret;
    }

    public function setWeVersion( $version) {
        Lib::ffi()->Schematic_set_we_version($this->ptr, $version);
    }

    public function flipX() {
        Lib::ffi()->Schematic_flip_x($this->ptr);
    }

    public function flipY() {
        Lib::ffi()->Schematic_flip_y($this->ptr);
    }

    public function flipZ() {
        Lib::ffi()->Schematic_flip_z($this->ptr);
    }

    public function rotateX( $degrees) {
        $result = Lib::ffi()->Schematic_rotate_x($this->ptr, $degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateY( $degrees) {
        $result = Lib::ffi()->Schematic_rotate_y($this->ptr, $degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateZ( $degrees) {
        $result = Lib::ffi()->Schematic_rotate_z($this->ptr, $degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function translate( $dx,  $dy,  $dz) {
        $result = Lib::ffi()->Schematic_translate($this->ptr, $dx, $dy, $dz);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipRegionX(string $region_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_flip_region_x($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipRegionY(string $region_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_flip_region_y($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipRegionZ(string $region_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_flip_region_z($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateRegionX(string $region_name,  $degrees) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_rotate_region_x($this->ptr, $__view0, $degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateRegionY(string $region_name,  $degrees) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_rotate_region_y($this->ptr, $__view0, $degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateRegionZ(string $region_name,  $degrees) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_rotate_region_z($this->ptr, $__view0, $degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function translateRegion(string $region_name,  $dx,  $dy,  $dz) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_translate_region($this->ptr, $__view0, $dx, $dy, $dz);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateSchematicX( $degrees) {
        $result = Lib::ffi()->Schematic_rotate_schematic_x($this->ptr, $degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateSchematicY( $degrees) {
        $result = Lib::ffi()->Schematic_rotate_schematic_y($this->ptr, $degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateSchematicZ( $degrees) {
        $result = Lib::ffi()->Schematic_rotate_schematic_z($this->ptr, $degrees);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipSchematicX() {
        $result = Lib::ffi()->Schematic_flip_schematic_x($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipSchematicY() {
        $result = Lib::ffi()->Schematic_flip_schematic_y($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipSchematicZ() {
        $result = Lib::ffi()->Schematic_flip_schematic_z($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function translateSchematic( $dx,  $dy,  $dz) {
        $result = Lib::ffi()->Schematic_translate_schematic($this->ptr, $dx, $dy, $dz);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function fillCuboid( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z, string $block_name) {
        $__n6 = strlen($block_name);
        $__view6 = Lib::ffi()->new('DiplomatStringView');
        if ($__n6 > 0) {
            $__buf6 = Lib::ffi()->new("uint8_t[" . $__n6 . "]", false);
            \FFI::memcpy($__buf6, $block_name, $__n6);
            $__view6->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf6[0]));
        } else {
            $__view6->data = null;
        }
        $__view6->len = $__n6;
        $result = Lib::ffi()->Schematic_fill_cuboid($this->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z, $__view6);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function fillSphere( $cx,  $cy,  $cz,  $radius, string $block_name) {
        $__n4 = strlen($block_name);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $block_name, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->Schematic_fill_sphere($this->ptr, $cx, $cy, $cz, $radius, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function saveAsB64(string $format, string $version, string $settings) {
        $__n0 = strlen($format);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $format, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($version);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $version, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $__n2 = strlen($settings);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $settings, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_save_as_b64($this->ptr, $__view0, $__view1, $__view2, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function saveToFileWithFormat(string $path, string $format, string $version) {
        $__n0 = strlen($path);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $path, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($format);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $format, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $__n2 = strlen($version);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $version, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->Schematic_save_to_file_with_format($this->ptr, $__view0, $__view1, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function toSchematicVersionB64(string $version) {
        $__n0 = strlen($version);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $version, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_to_schematic_version_b64($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function availableSchematicVersionsJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_available_schematic_versions_json($write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function setBlockWithNbt( $x,  $y,  $z, string $block_name, string $nbt_json) {
        $__n3 = strlen($block_name);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $block_name, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $__n4 = strlen($nbt_json);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $nbt_json, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->Schematic_set_block_with_nbt($this->ptr, $x, $y, $z, $__view3, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setBlockInRegion(string $region_name,  $x,  $y,  $z, string $block_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n4 = strlen($block_name);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $block_name, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->Schematic_set_block_in_region($this->ptr, $__view0, $x, $y, $z, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function hasRegion(string $region_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_has_region($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function createRegion(string $region_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_create_region($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function removeRegion(string $region_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Schematic_remove_region($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function renameRegion(string $old_name, string $new_name) {
        $__n0 = strlen($old_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $old_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($new_name);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $new_name, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->Schematic_rename_region($this->ptr, $__view0, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function boundingBoxJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_bounding_box_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function regionBoundingBoxJson(string $region_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_region_bounding_box_json($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function paletteJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_palette_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function tightDimensions() {
        $ret = Lib::ffi()->Schematic_tight_dimensions($this->ptr);
        return Dimensions::fromFFI($ret);
    }

    public function allocatedDimensions() {
        $ret = Lib::ffi()->Schematic_allocated_dimensions($this->ptr);
        return Dimensions::fromFFI($ret);
    }

    public function extractSignsJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_extract_signs_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function compileInsignJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_compile_insign_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function allPalettesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_all_palettes_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function defaultRegionPaletteJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Schematic_default_region_palette_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function regionPaletteJson(string $region_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Schematic_region_palette_json($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function tightBoundsMin() {
        $result = Lib::ffi()->Schematic_tight_bounds_min($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return BlockPos::fromFFI($result->ok);
    }

    public function tightBoundsMax() {
        $result = Lib::ffi()->Schematic_tight_bounds_max($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return BlockPos::fromFFI($result->ok);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Schematic_destroy($this->ptr);
        }
    }
}
