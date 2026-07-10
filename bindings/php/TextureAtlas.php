<?php
namespace Stencil;

final class TextureAtlas {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function buildGlobal( $schematic,  $pack,  $config) {
        $result = Lib::ffi()->TextureAtlas_build_global($schematic->ptr, $pack->ptr, $config->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TextureAtlas($result->ok, true);
    }

    public function width() {
        $ret = Lib::ffi()->TextureAtlas_width($this->ptr);
        return $ret;
    }

    public function height() {
        $ret = Lib::ffi()->TextureAtlas_height($this->ptr);
        return $ret;
    }

    public function rgbaDataB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TextureAtlas_rgba_data_b64($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->TextureAtlas_destroy($this->ptr);
        }
    }
}
