<?php
namespace Stencil;

final class WorldGenerator {
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

    public static function sdf( $volume,  $material,  $min_y,  $max_y, string $source_id, string $version) {
        $__n4 = strlen($source_id);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $source_id, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $__n5 = strlen($version);
        $__view5 = Lib::ffi()->new('DiplomatStringView');
        if ($__n5 > 0) {
            $__buf5 = Lib::ffi()->new("uint8_t[" . $__n5 . "]", false);
            \FFI::memcpy($__buf5, $version, $__n5);
            $__view5->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf5[0]));
        } else {
            $__view5->data = null;
        }
        $__view5->len = $__n5;
        $result = Lib::ffi()->WorldGenerator_sdf($volume->ptr, $material->ptr, $min_y, $max_y, $__view4, $__view5);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new WorldGenerator($result->ok, true);
    }

    public static function cellularSdf( $volume,  $material,  $min_y,  $max_y,  $config, string $source_id, string $version) {
        $__n5 = strlen($source_id);
        $__view5 = Lib::ffi()->new('DiplomatStringView');
        if ($__n5 > 0) {
            $__buf5 = Lib::ffi()->new("uint8_t[" . $__n5 . "]", false);
            \FFI::memcpy($__buf5, $source_id, $__n5);
            $__view5->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf5[0]));
        } else {
            $__view5->data = null;
        }
        $__view5->len = $__n5;
        $__n6 = strlen($version);
        $__view6 = Lib::ffi()->new('DiplomatStringView');
        if ($__n6 > 0) {
            $__buf6 = Lib::ffi()->new("uint8_t[" . $__n6 . "]", false);
            \FFI::memcpy($__buf6, $version, $__n6);
            $__view6->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf6[0]));
        } else {
            $__view6->data = null;
        }
        $__view6->len = $__n6;
        $result = Lib::ffi()->WorldGenerator_cellular_sdf($volume->ptr, $material->ptr, $min_y, $max_y, $config->ptr, $__view5, $__view6);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new WorldGenerator($result->ok, true);
    }

    public static function projectedFootprints(string $buildings_json, string $base_block, string $source_id, string $version) {
        $__n0 = strlen($buildings_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $buildings_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($base_block);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $base_block, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $__n2 = strlen($source_id);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $source_id, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $__n3 = strlen($version);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $version, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->WorldGenerator_projected_footprints($__view0, $__view1, $__view2, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new WorldGenerator($result->ok, true);
    }

    public static function composite(string $source_id, string $version) {
        $__n0 = strlen($source_id);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $source_id, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n1 = strlen($version);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $version, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->WorldGenerator_composite($__view0, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new WorldGenerator($result->ok, true);
    }

    public function addLayer( $source, int $mode) {
        $result = Lib::ffi()->WorldGenerator_add_layer($this->ptr, $source->ptr, $mode);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function generate( $cx,  $cz) {
        $result = Lib::ffi()->WorldGenerator_generate($this->ptr, $cx, $cz);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new GeneratedChunk($result->ok, true);
    }

    public function stream( $min_cx,  $min_cz,  $max_cx,  $max_cz) {
        $result = Lib::ffi()->WorldGenerator_stream($this->ptr, $min_cx, $min_cz, $max_cx, $max_cz);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new GeneratedWorldStream($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->WorldGenerator_destroy($this->ptr);
        }
    }
}
