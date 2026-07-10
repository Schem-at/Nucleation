<?php
namespace Stencil;

final class ChunkMeshResult {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create( $schematic,  $pack,  $config) {
        $result = Lib::ffi()->ChunkMeshResult_create($schematic->ptr, $pack->ptr, $config->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new ChunkMeshResult($result->ok, true);
    }

    public static function createWithSize( $schematic,  $pack,  $config,  $chunk_size) {
        $result = Lib::ffi()->ChunkMeshResult_create_with_size($schematic->ptr, $pack->ptr, $config->ptr, $chunk_size);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new ChunkMeshResult($result->ok, true);
    }

    public static function createWithAtlas( $schematic,  $pack,  $config,  $chunk_size,  $atlas) {
        $result = Lib::ffi()->ChunkMeshResult_create_with_atlas($schematic->ptr, $pack->ptr, $config->ptr, $chunk_size, $atlas->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new ChunkMeshResult($result->ok, true);
    }

    public function chunkCount() {
        $ret = Lib::ffi()->ChunkMeshResult_chunk_count($this->ptr);
        return $ret;
    }

    public function chunkCoordinateAt( $index) {
        $result = Lib::ffi()->ChunkMeshResult_chunk_coordinate_at($this->ptr, $index);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return BlockPos::fromFFI($result->ok);
    }

    public function getMesh( $cx,  $cy,  $cz) {
        $result = Lib::ffi()->ChunkMeshResult_get_mesh($this->ptr, $cx, $cy, $cz);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MeshResult($result->ok, true);
    }

    public function totalVertexCount() {
        $ret = Lib::ffi()->ChunkMeshResult_total_vertex_count($this->ptr);
        return $ret;
    }

    public function totalTriangleCount() {
        $ret = Lib::ffi()->ChunkMeshResult_total_triangle_count($this->ptr);
        return $ret;
    }

    public function nucmDataB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->ChunkMeshResult_nucm_data_b64($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function nucmDataWithAtlasB64( $atlas) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->ChunkMeshResult_nucm_data_with_atlas_b64($this->ptr, $atlas->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->ChunkMeshResult_destroy($this->ptr);
        }
    }
}
