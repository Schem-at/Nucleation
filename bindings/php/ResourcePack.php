<?php
namespace Stencil;

final class ResourcePack {
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

    public static function fromBytes(array $data) {
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
        $result = Lib::ffi()->ResourcePack_from_bytes($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new ResourcePack($result->ok, true);
    }

    public static function fromList( $list) {
        $result = Lib::ffi()->ResourcePack_from_list($list->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new ResourcePack($result->ok, true);
    }

    public function blockstateCount() {
        $ret = Lib::ffi()->ResourcePack_blockstate_count($this->ptr);
        return $ret;
    }

    public function modelCount() {
        $ret = Lib::ffi()->ResourcePack_model_count($this->ptr);
        return $ret;
    }

    public function textureCount() {
        $ret = Lib::ffi()->ResourcePack_texture_count($this->ptr);
        return $ret;
    }

    public function namespacesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->ResourcePack_namespaces_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function listBlockstatesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->ResourcePack_list_blockstates_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function listModelsJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->ResourcePack_list_models_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function listTexturesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->ResourcePack_list_textures_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function getBlockstateJson(string $name) {
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
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->ResourcePack_get_blockstate_json($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function getModelJson(string $name) {
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
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->ResourcePack_get_model_json($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function getTextureInfo(string $name) {
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
        $result = Lib::ffi()->ResourcePack_get_texture_info($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return TextureInfo::fromFFI($result->ok);
    }

    public function getTexturePixelsB64(string $name) {
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
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->ResourcePack_get_texture_pixels_b64($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function addBlockstateJson(string $name, string $json) {
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
        $__n1 = strlen($json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->ResourcePack_add_blockstate_json($this->ptr, $__view0, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addModelJson(string $name, string $json) {
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
        $__n1 = strlen($json);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $json, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->ResourcePack_add_model_json($this->ptr, $__view0, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addTexture(string $name,  $width,  $height, array $pixels) {
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
        $__n3 = count($pixels);
        $__view3 = Lib::ffi()->new('DiplomatU8View');
        if ($__n3 > 0) {
            $__arr3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            foreach ($pixels as $__i3 => $__v3) { $__arr3[$__i3] = $__v3; }
            $__view3->data = \FFI::addr($__arr3[0]);
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->ResourcePack_add_texture($this->ptr, $__view0, $width, $height, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function registerMeshExporter() {
        $result = Lib::ffi()->ResourcePack_register_mesh_exporter($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->ResourcePack_destroy($this->ptr);
        }
    }
}
