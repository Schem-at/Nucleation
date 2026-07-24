<?php
namespace Stencil;

final class MeshConfig {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create() {
        $ret = Lib::ffi()->MeshConfig_create();
        return new MeshConfig($ret, true);
    }

    public function setCullHiddenFaces( $val) {
        Lib::ffi()->MeshConfig_set_cull_hidden_faces($this->ptr, $val);
    }

    public function cullHiddenFaces() {
        $ret = Lib::ffi()->MeshConfig_cull_hidden_faces($this->ptr);
        return $ret;
    }

    public function setAmbientOcclusion( $val) {
        Lib::ffi()->MeshConfig_set_ambient_occlusion($this->ptr, $val);
    }

    public function ambientOcclusion() {
        $ret = Lib::ffi()->MeshConfig_ambient_occlusion($this->ptr);
        return $ret;
    }

    public function setAoIntensity( $val) {
        Lib::ffi()->MeshConfig_set_ao_intensity($this->ptr, $val);
    }

    public function aoIntensity() {
        $ret = Lib::ffi()->MeshConfig_ao_intensity($this->ptr);
        return $ret;
    }

    public function setBiome(string $biome) {
        $__n0 = strlen($biome);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $biome, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->MeshConfig_set_biome($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function clearBiome() {
        Lib::ffi()->MeshConfig_clear_biome($this->ptr);
    }

    public function biome() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->MeshConfig_biome($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function setAtlasMaxSize( $size) {
        Lib::ffi()->MeshConfig_set_atlas_max_size($this->ptr, $size);
    }

    public function atlasMaxSize() {
        $ret = Lib::ffi()->MeshConfig_atlas_max_size($this->ptr);
        return $ret;
    }

    public function setCullOccludedBlocks( $val) {
        Lib::ffi()->MeshConfig_set_cull_occluded_blocks($this->ptr, $val);
    }

    public function cullOccludedBlocks() {
        $ret = Lib::ffi()->MeshConfig_cull_occluded_blocks($this->ptr);
        return $ret;
    }

    public function setGreedyMeshing( $val) {
        Lib::ffi()->MeshConfig_set_greedy_meshing($this->ptr, $val);
    }

    public function greedyMeshing() {
        $ret = Lib::ffi()->MeshConfig_greedy_meshing($this->ptr);
        return $ret;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->MeshConfig_destroy($this->ptr);
        }
    }
}
