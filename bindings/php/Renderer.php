<?php
namespace Stencil;

final class Renderer {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function renderPixelsB64( $schematic, array $pack_zip,  $config) {
        $__n1 = count($pack_zip);
        $__view1 = Lib::ffi()->new('DiplomatU8View');
        if ($__n1 > 0) {
            $__arr1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            foreach ($pack_zip as $__i1 => $__v1) { $__arr1[$__i1] = $__v1; }
            $__view1->data = \FFI::addr($__arr1[0]);
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Renderer_render_pixels_b64($schematic->ptr, $__view1, $config->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function renderPngB64( $schematic, array $pack_zip,  $config) {
        $__n1 = count($pack_zip);
        $__view1 = Lib::ffi()->new('DiplomatU8View');
        if ($__n1 > 0) {
            $__arr1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            foreach ($pack_zip as $__i1 => $__v1) { $__arr1[$__i1] = $__v1; }
            $__view1->data = \FFI::addr($__arr1[0]);
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Renderer_render_png_b64($schematic->ptr, $__view1, $config->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function renderToFile( $schematic, array $pack_zip,  $config, string $path) {
        $__n1 = count($pack_zip);
        $__view1 = Lib::ffi()->new('DiplomatU8View');
        if ($__n1 > 0) {
            $__arr1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            foreach ($pack_zip as $__i1 => $__v1) { $__arr1[$__i1] = $__v1; }
            $__view1->data = \FFI::addr($__arr1[0]);
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $__n3 = strlen($path);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $path, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->Renderer_render_to_file($schematic->ptr, $__view1, $config->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function renderToFileWithPack( $schematic,  $pack,  $config, string $path) {
        $__n3 = strlen($path);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $path, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->Renderer_render_to_file_with_pack($schematic->ptr, $pack->ptr, $config->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function renderPixelsB64WithPack( $schematic,  $pack,  $config) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Renderer_render_pixels_b64_with_pack($schematic->ptr, $pack->ptr, $config->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function renderPngB64WithPack( $schematic,  $pack,  $config) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Renderer_render_png_b64_with_pack($schematic->ptr, $pack->ptr, $config->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Renderer_destroy($this->ptr);
        }
    }
}
