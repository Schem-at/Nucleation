<?php
namespace Stencil;

final class MeshResult {
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

    public static function create( $schematic,  $pack,  $config) {
        $result = Lib::ffi()->MeshResult_create($schematic->ptr, $pack->ptr, $config->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MeshResult($result->ok, true);
    }

    public static function createUsdz( $schematic,  $pack,  $config) {
        $result = Lib::ffi()->MeshResult_create_usdz($schematic->ptr, $pack->ptr, $config->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MeshResult($result->ok, true);
    }

    public function glbDataB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->MeshResult_glb_data_b64($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function usdzDataB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->MeshResult_usdz_data_b64($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function nucmDataB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->MeshResult_nucm_data_b64($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function vertexCount() {
        $ret = Lib::ffi()->MeshResult_vertex_count($this->ptr);
        return $ret;
    }

    public function triangleCount() {
        $ret = Lib::ffi()->MeshResult_triangle_count($this->ptr);
        return $ret;
    }

    public function hasTransparency() {
        $ret = Lib::ffi()->MeshResult_has_transparency($this->ptr);
        return $ret;
    }

    public function bounds() {
        $ret = Lib::ffi()->MeshResult_bounds($this->ptr);
        return MeshBounds::fromFFI($ret);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->MeshResult_destroy($this->ptr);
        }
    }
}
