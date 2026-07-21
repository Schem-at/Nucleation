<?php
namespace Stencil;

final class BuildAnimation {
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

    public static function create(string $name) {
        $__n0 = strlen($name);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $name, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $ret = Lib::ffi()->BuildAnimation_create($__view0);
        return new BuildAnimation($ret, true);
    }

    public function setDefaultEffect( $effect) {
        Lib::ffi()->BuildAnimation_set_default_effect($this->ptr, $effect->ptr);
    }

    public function withEffect( $effect) {
        $ret = Lib::ffi()->BuildAnimation_with_effect($this->ptr, $effect->ptr);
        return new BuildAnimation($ret, false, $this);
    }

    public function setStepMs( $step_ms) {
        Lib::ffi()->BuildAnimation_set_step_ms($this->ptr, $step_ms);
    }

    public function setStaggerTotalMs( $total_ms) {
        Lib::ffi()->BuildAnimation_set_stagger_total_ms($this->ptr, $total_ms);
    }

    public function clearStagger() {
        Lib::ffi()->BuildAnimation_clear_stagger($this->ptr);
    }

    public function beginGroup() {
        $result = Lib::ffi()->BuildAnimation_begin_group($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function beginKeyedGroup( $key) {
        $result = Lib::ffi()->BuildAnimation_begin_keyed_group($this->ptr, $key);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function endGroup() {
        $result = Lib::ffi()->BuildAnimation_end_group($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function setBlock( $x,  $y,  $z, string $block) {
        $__n3 = strlen($block);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $block, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->BuildAnimation_set_block($this->ptr, $x, $y, $z, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function addArmorStand( $x,  $y,  $z,  $yaw, string $armor_material) {
        $__n4 = strlen($armor_material);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $armor_material, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->BuildAnimation_add_armor_stand($this->ptr, $x, $y, $z, $yaw, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function animateCamera( $effect,  $offset_ms) {
        Lib::ffi()->BuildAnimation_animate_camera($this->ptr, $effect->ptr, $offset_ms);
    }

    public function frameCount( $fps,  $hold_ms) {
        $ret = Lib::ffi()->BuildAnimation_frame_count($this->ptr, $fps, $hold_ms);
        return $ret;
    }

    public function renderGif(array $pack_zip,  $config, string $path,  $fps,  $hold_ms) {
        $__n0 = count($pack_zip);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($pack_zip as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n2 = strlen($path);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $path, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->BuildAnimation_render_gif($this->ptr, $__view0, $config->ptr, $__view2, $fps, $hold_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function renderFrames(array $pack_zip,  $config, string $prefix,  $fps,  $hold_ms) {
        $__n0 = count($pack_zip);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($pack_zip as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n2 = strlen($prefix);
        $__view2 = Lib::ffi()->new('DiplomatStringView');
        if ($__n2 > 0) {
            $__buf2 = Lib::ffi()->new("uint8_t[" . $__n2 . "]", false);
            \FFI::memcpy($__buf2, $prefix, $__n2);
            $__view2->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf2[0]));
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->BuildAnimation_render_frames($this->ptr, $__view0, $config->ptr, $__view2, $fps, $hold_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function saveToFile(string $path) {
        $__n0 = strlen($path);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $path, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->BuildAnimation_save_to_file($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function groupCount() {
        $ret = Lib::ffi()->BuildAnimation_group_count($this->ptr);
        return $ret;
    }

    public function durationMs() {
        $ret = Lib::ffi()->BuildAnimation_duration_ms($this->ptr);
        return $ret;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->BuildAnimation_destroy($this->ptr);
        }
    }
}
