<?php
namespace Stencil;

final class RawMeshExport {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create( $schematic,  $pack,  $config) {
        $result = Lib::ffi()->RawMeshExport_create($schematic->ptr, $pack->ptr, $config->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new RawMeshExport($result->ok, true);
    }

    public function vertexCount() {
        $ret = Lib::ffi()->RawMeshExport_vertex_count($this->ptr);
        return $ret;
    }

    public function triangleCount() {
        $ret = Lib::ffi()->RawMeshExport_triangle_count($this->ptr);
        return $ret;
    }

    public function positionsB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->RawMeshExport_positions_b64($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function normalsB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->RawMeshExport_normals_b64($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function uvsB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->RawMeshExport_uvs_b64($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function colorsB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->RawMeshExport_colors_b64($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function indicesB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->RawMeshExport_indices_b64($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function textureRgbaB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->RawMeshExport_texture_rgba_b64($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function textureWidth() {
        $ret = Lib::ffi()->RawMeshExport_texture_width($this->ptr);
        return $ret;
    }

    public function textureHeight() {
        $ret = Lib::ffi()->RawMeshExport_texture_height($this->ptr);
        return $ret;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->RawMeshExport_destroy($this->ptr);
        }
    }
}
