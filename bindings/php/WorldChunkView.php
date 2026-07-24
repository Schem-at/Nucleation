<?php
namespace Stencil;

final class WorldChunkView {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create( $cx,  $cz) {
        $ret = Lib::ffi()->WorldChunkView_create($cx, $cz);
        return new WorldChunkView($ret, true);
    }

    public function cx() {
        $ret = Lib::ffi()->WorldChunkView_cx($this->ptr);
        return $ret;
    }

    public function cz() {
        $ret = Lib::ffi()->WorldChunkView_cz($this->ptr);
        return $ret;
    }

    public function toSchematic() {
        $ret = Lib::ffi()->WorldChunkView_to_schematic($this->ptr);
        return new Schematic($ret, true);
    }

    public static function fromSchematic( $schematic,  $cx,  $cz) {
        $ret = Lib::ffi()->WorldChunkView_from_schematic($schematic->ptr, $cx, $cz);
        return new WorldChunkView($ret, true);
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
        $result = Lib::ffi()->WorldChunkView_set_block($this->ptr, $x, $y, $z, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setBiome(string $biome_name) {
        $__n0 = strlen($biome_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $biome_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->WorldChunkView_set_biome($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function biomePaletteJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->WorldChunkView_biome_palette_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->WorldChunkView_destroy($this->ptr);
        }
    }
}
