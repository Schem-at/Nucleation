<?php
namespace Stencil;

final class MultiMeshResult {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create( $schematic,  $pack,  $config) {
        $result = Lib::ffi()->MultiMeshResult_create($schematic->ptr, $pack->ptr, $config->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MultiMeshResult($result->ok, true);
    }

    public function regionNamesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->MultiMeshResult_region_names_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function getMesh(string $region_name) {
        $__n0 = strlen($region_name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region_name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->MultiMeshResult_get_mesh($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new MeshResult($result->ok, true);
    }

    public function totalVertexCount() {
        $ret = Lib::ffi()->MultiMeshResult_total_vertex_count($this->ptr);
        return $ret;
    }

    public function totalTriangleCount() {
        $ret = Lib::ffi()->MultiMeshResult_total_triangle_count($this->ptr);
        return $ret;
    }

    public function meshCount() {
        $ret = Lib::ffi()->MultiMeshResult_mesh_count($this->ptr);
        return $ret;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->MultiMeshResult_destroy($this->ptr);
        }
    }
}
