<?php
namespace Stencil;

final class RawMeshExport {
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

    public function copyPositions(array $out) {
        $__n0 = count($out);
        $__view0 = Lib::ffi()->new('DiplomatF32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("float[" . $__n0 . "]", false);
            foreach ($out as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $ret = Lib::ffi()->RawMeshExport_copy_positions($this->ptr, $__view0);
        return $ret;
    }

    public function copyNormals(array $out) {
        $__n0 = count($out);
        $__view0 = Lib::ffi()->new('DiplomatF32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("float[" . $__n0 . "]", false);
            foreach ($out as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $ret = Lib::ffi()->RawMeshExport_copy_normals($this->ptr, $__view0);
        return $ret;
    }

    public function copyUvs(array $out) {
        $__n0 = count($out);
        $__view0 = Lib::ffi()->new('DiplomatF32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("float[" . $__n0 . "]", false);
            foreach ($out as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $ret = Lib::ffi()->RawMeshExport_copy_uvs($this->ptr, $__view0);
        return $ret;
    }

    public function copyColors(array $out) {
        $__n0 = count($out);
        $__view0 = Lib::ffi()->new('DiplomatF32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("float[" . $__n0 . "]", false);
            foreach ($out as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $ret = Lib::ffi()->RawMeshExport_copy_colors($this->ptr, $__view0);
        return $ret;
    }

    public function copyIndices(array $out) {
        $__n0 = count($out);
        $__view0 = Lib::ffi()->new('DiplomatU32View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint32_t[" . $__n0 . "]", false);
            foreach ($out as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $ret = Lib::ffi()->RawMeshExport_copy_indices($this->ptr, $__view0);
        return $ret;
    }

    public function textureRgbaLen() {
        $ret = Lib::ffi()->RawMeshExport_texture_rgba_len($this->ptr);
        return $ret;
    }

    public function copyTextureRgba(array $out) {
        $__n0 = count($out);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($out as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $ret = Lib::ffi()->RawMeshExport_copy_texture_rgba($this->ptr, $__view0);
        return $ret;
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
