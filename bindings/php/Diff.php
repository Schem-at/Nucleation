<?php
namespace Stencil;

final class Diff {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function compute( $a,  $b, string $preset) {
        $__n2 = strlen($preset);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $preset, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->Diff_compute($a->ptr, $b->ptr, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Diff($result->ok, true);
    }

    public static function computeWithOpts( $a,  $b, string $preset,  $cost_add,  $cost_delete,  $cost_change,  $cost_swap, string $symmetry) {
        $__n2 = strlen($preset);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $preset, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $__n7 = strlen($symmetry);
        $__view7 = Lib::ffi()->new('DiplomatStringView');
        if ($__n7 > 0) {
            $__buf7 = Lib::ffi()->new("uint8_t[" . $__n7 . "]", false);
            \FFI::memcpy($__buf7, $symmetry, $__n7);
            $__view7->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf7[0]));
        } else {
            $__view7->data = null;
        }
        $__view7->len = $__n7;
        $result = Lib::ffi()->Diff_compute_with_opts($a->ptr, $b->ptr, $__view2, $cost_add, $cost_delete, $cost_change, $cost_swap, $__view7);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Diff($result->ok, true);
    }

    public static function fromJson(string $json) {
        $__n0 = strlen($json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Diff_from_json($__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Diff($result->ok, true);
    }

    public function distance() {
        $ret = Lib::ffi()->Diff_distance($this->ptr);
        return $ret;
    }

    public function support() {
        $ret = Lib::ffi()->Diff_support($this->ptr);
        return $ret;
    }

    public function toJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Diff_to_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function summaryJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->Diff_summary_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function added() {
        $ret = Lib::ffi()->Diff_added($this->ptr);
        return new Schematic($ret, true);
    }

    public function removed() {
        $ret = Lib::ffi()->Diff_removed($this->ptr);
        return new Schematic($ret, true);
    }

    public function changed() {
        $ret = Lib::ffi()->Diff_changed($this->ptr);
        return new Schematic($ret, true);
    }

    public function swapped() {
        $ret = Lib::ffi()->Diff_swapped($this->ptr);
        return new Schematic($ret, true);
    }

    public function markers() {
        $ret = Lib::ffi()->Diff_markers($this->ptr);
        return new Schematic($ret, true);
    }

    public function toOverlayGlbB64(array $after_glb) {
        $__n0 = count($after_glb);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($after_glb as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->Diff_to_overlay_glb_b64($this->ptr, $__view0, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Diff_destroy($this->ptr);
        }
    }
}
