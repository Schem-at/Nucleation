<?php
namespace Stencil;

final class Palette {
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

    public static function all() {
        $ret = Lib::ffi()->Palette_all();
        return new Palette($ret, true);
    }

    public static function solid() {
        $ret = Lib::ffi()->Palette_solid();
        return new Palette($ret, true);
    }

    public static function structural() {
        $ret = Lib::ffi()->Palette_structural();
        return new Palette($ret, true);
    }

    public static function decorative() {
        $ret = Lib::ffi()->Palette_decorative();
        return new Palette($ret, true);
    }

    public static function concrete() {
        $ret = Lib::ffi()->Palette_concrete();
        return new Palette($ret, true);
    }

    public static function wool() {
        $ret = Lib::ffi()->Palette_wool();
        return new Palette($ret, true);
    }

    public static function terracotta() {
        $ret = Lib::ffi()->Palette_terracotta();
        return new Palette($ret, true);
    }

    public static function grayscale() {
        $ret = Lib::ffi()->Palette_grayscale();
        return new Palette($ret, true);
    }

    public static function wood() {
        $ret = Lib::ffi()->Palette_wood();
        return new Palette($ret, true);
    }

    public function dithered() {
        $ret = Lib::ffi()->Palette_dithered($this->ptr);
        return new Palette($ret, true);
    }

    public function sortedByLightness() {
        $ret = Lib::ffi()->Palette_sorted_by_lightness($this->ptr);
        return new Palette($ret, true);
    }

    public function rampIdsJson( $r1,  $g1,  $b1,  $r2,  $g2,  $b2,  $steps) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Palette_ramp_ids_json($this->ptr, $r1, $g1, $b1, $r2, $g2, $b2, $steps, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function gradientIdsJson( $r1,  $g1,  $b1,  $r2,  $g2,  $b2,  $steps) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Palette_gradient_ids_json($this->ptr, $r1, $g1, $b1, $r2, $g2, $b2, $steps, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public static function fromBlockIds(string $ids_json) {
        $__n0 = strlen($ids_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $ids_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Palette_from_block_ids($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Palette($result->ok, true);
    }

    public function len() {
        $ret = Lib::ffi()->Palette_len($this->ptr);
        return $ret;
    }

    public function blockIdsJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Palette_block_ids_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function closestBlockDithered( $r,  $g,  $b,  $x,  $y,  $z) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Palette_closest_block_dithered($this->ptr, $r, $g, $b, $x, $y, $z, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function closestBlock( $r,  $g,  $b) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Palette_closest_block($this->ptr, $r, $g, $b, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Palette_destroy($this->ptr);
        }
    }
}
