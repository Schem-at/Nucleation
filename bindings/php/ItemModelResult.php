<?php
namespace Stencil;

final class ItemModelResult {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create( $schematic,  $pack,  $config) {
        $result = Lib::ffi()->ItemModelResult_create($schematic->ptr, $pack->ptr, $config->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new ItemModelResult($result->ok, true);
    }

    public function modelJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->ItemModelResult_model_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function elementCount() {
        $result = Lib::ffi()->ItemModelResult_element_count($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function textureCount() {
        $result = Lib::ffi()->ItemModelResult_texture_count($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function planeCount() {
        $result = Lib::ffi()->ItemModelResult_plane_count($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function dimensions() {
        $result = Lib::ffi()->ItemModelResult_dimensions($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Dimensions::fromFFI($result->ok);
    }

    public function scale() {
        $result = Lib::ffi()->ItemModelResult_scale($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return ItemScale::fromFFI($result->ok);
    }

    public function toResourcePackZipB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->ItemModelResult_to_resource_pack_zip_b64($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function addToPack( $builder) {
        $result = Lib::ffi()->ItemModelResult_add_to_pack($this->ptr, $builder->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->ItemModelResult_destroy($this->ptr);
        }
    }
}
