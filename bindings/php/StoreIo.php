<?php
namespace Stencil;

final class StoreIo {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function open(string $uri) {
        $__n0 = strlen($uri);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $uri, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->StoreIo_open($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public static function save( $schematic, string $uri, string $version) {
        $__n1 = strlen($uri);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $uri, $__n1);
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
        $result = Lib::ffi()->StoreIo_save($schematic->ptr, $__view1, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public static function exportSettingsSchema(string $format) {
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
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->StoreIo_export_settings_schema($__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function importSettingsSchema(string $format) {
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
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->StoreIo_import_settings_schema($__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function supportedImportFormats() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->StoreIo_supported_import_formats($write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function supportedExportFormats() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->StoreIo_supported_export_formats($write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function formatVersions(string $format) {
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
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->StoreIo_format_versions($__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function defaultFormatVersion(string $format) {
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
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->StoreIo_default_format_version($__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->StoreIo_destroy($this->ptr);
        }
    }
}
