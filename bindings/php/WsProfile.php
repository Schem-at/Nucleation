<?php
namespace Stencil;

final class WsProfile {
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

    public static function deriveFromDir(string $world_dir,  $min_y,  $max_y,  $sample,  $coverage) {
        $__n0 = strlen($world_dir);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $world_dir, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->WsProfile_derive_from_dir($__view0, $min_y, $max_y, $sample, $coverage);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new WsProfile($result->ok, true);
    }

    public function bandMin() {
        $ret = Lib::ffi()->WsProfile_band_min($this->ptr);
        return $ret;
    }

    public function bandMax() {
        $ret = Lib::ffi()->WsProfile_band_max($this->ptr);
        return $ret;
    }

    public function paletteLen() {
        $ret = Lib::ffi()->WsProfile_palette_len($this->ptr);
        return $ret;
    }

    public function writePaletteJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->WsProfile_write_palette_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->WsProfile_destroy($this->ptr);
        }
    }
}
