<?php
namespace Stencil;

final class MaterialBuilder {
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

    public static function createLayered() {
        $ret = Lib::ffi()->MaterialBuilder_create_layered();
        return new MaterialBuilder($ret, true);
    }

    public static function createTagMatcher( $blend_seams,  $seed) {
        $ret = Lib::ffi()->MaterialBuilder_create_tag_matcher($blend_seams, $seed);
        return new MaterialBuilder($ret, true);
    }

    public function addLayer( $condition,  $material) {
        $result = Lib::ffi()->MaterialBuilder_add_layer($this->ptr, $condition->ptr, $material->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addTagCase( $material_id,  $material) {
        $result = Lib::ffi()->MaterialBuilder_add_tag_case($this->ptr, $material_id, $material->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setDefault( $material) {
        Lib::ffi()->MaterialBuilder_set_default($this->ptr, $material->ptr);
    }

    public function build() {
        $ret = Lib::ffi()->MaterialBuilder_build($this->ptr);
        return new Material($ret, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->MaterialBuilder_destroy($this->ptr);
        }
    }
}
