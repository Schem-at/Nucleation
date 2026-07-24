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

    public static function fromSchematic( $schematic) {
        $result = Lib::ffi()->BuildAnimation_from_schematic($schematic->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new BuildAnimation($result->ok, true);
    }

    public function animateAll( $effect) {
        Lib::ffi()->BuildAnimation_animate_all($this->ptr, $effect->ptr);
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

    public function setStaggerOffsetMs( $offset_ms) {
        Lib::ffi()->BuildAnimation_set_stagger_offset_ms($this->ptr, $offset_ms);
    }

    public function setLoopPeriodMs( $period_ms) {
        $result = Lib::ffi()->BuildAnimation_set_loop_period_ms($this->ptr, $period_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function clearLoopPeriod() {
        Lib::ffi()->BuildAnimation_clear_loop_period($this->ptr);
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

    public function createRegion(string $name,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
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
        $result = Lib::ffi()->BuildAnimation_create_region($this->ptr, $__view0, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setBlockInRegion(string $region,  $x,  $y,  $z, string $block) {
        $__n0 = strlen($region);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $__n4 = strlen($block);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $block, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->BuildAnimation_set_block_in_region($this->ptr, $__view0, $x, $y, $z, $__view4);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function translate( $x,  $y,  $z,  $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_translate($this->ptr, $x, $y, $z, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function translateRegion(string $region,  $x,  $y,  $z,  $duration_ms) {
        $__n0 = strlen($region);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->BuildAnimation_translate_region($this->ptr, $__view0, $x, $y, $z, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function translateAll( $x,  $y,  $z,  $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_translate_all($this->ptr, $x, $y, $z, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateX( $degrees,  $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_rotate_x($this->ptr, $degrees, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateY( $degrees,  $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_rotate_y($this->ptr, $degrees, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateZ( $degrees,  $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_rotate_z($this->ptr, $degrees, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateRegionX(string $region,  $degrees,  $duration_ms) {
        $__n0 = strlen($region);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->BuildAnimation_rotate_region_x($this->ptr, $__view0, $degrees, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateRegionY(string $region,  $degrees,  $duration_ms) {
        $__n0 = strlen($region);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->BuildAnimation_rotate_region_y($this->ptr, $__view0, $degrees, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateRegionZ(string $region,  $degrees,  $duration_ms) {
        $__n0 = strlen($region);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->BuildAnimation_rotate_region_z($this->ptr, $__view0, $degrees, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateAllX( $degrees,  $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_rotate_all_x($this->ptr, $degrees, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateAllY( $degrees,  $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_rotate_all_y($this->ptr, $degrees, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function rotateAllZ( $degrees,  $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_rotate_all_z($this->ptr, $degrees, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipX( $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_flip_x($this->ptr, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipY( $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_flip_y($this->ptr, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipZ( $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_flip_z($this->ptr, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipRegionX(string $region,  $duration_ms) {
        $__n0 = strlen($region);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->BuildAnimation_flip_region_x($this->ptr, $__view0, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipRegionY(string $region,  $duration_ms) {
        $__n0 = strlen($region);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->BuildAnimation_flip_region_y($this->ptr, $__view0, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipRegionZ(string $region,  $duration_ms) {
        $__n0 = strlen($region);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $region, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->BuildAnimation_flip_region_z($this->ptr, $__view0, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipAllX( $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_flip_all_x($this->ptr, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipAllY( $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_flip_all_y($this->ptr, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function flipAllZ( $duration_ms) {
        $result = Lib::ffi()->BuildAnimation_flip_all_z($this->ptr, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function stampRegion( $source, string $region,  $x,  $y,  $z, string $exclusions,  $duration_ms) {
        $__n1 = strlen($region);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $region, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $__n5 = strlen($exclusions);
        $__view5 = Lib::ffi()->new('DiplomatStringView');
        if ($__n5 > 0) {
            $__buf5 = Lib::ffi()->new("uint8_t[" . $__n5 . "]", false);
            \FFI::memcpy($__buf5, $exclusions, $__n5);
            $__view5->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf5[0]));
        } else {
            $__view5->data = null;
        }
        $__view5->len = $__n5;
        $result = Lib::ffi()->BuildAnimation_stamp_region($this->ptr, $source->ptr, $__view1, $x, $y, $z, $__view5, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function stampBox( $source,  $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z,  $x,  $y,  $z, string $exclusions,  $duration_ms) {
        $__n10 = strlen($exclusions);
        $__view10 = Lib::ffi()->new('DiplomatStringView');
        if ($__n10 > 0) {
            $__buf10 = Lib::ffi()->new("uint8_t[" . $__n10 . "]", false);
            \FFI::memcpy($__buf10, $exclusions, $__n10);
            $__view10->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf10[0]));
        } else {
            $__view10->data = null;
        }
        $__view10->len = $__n10;
        $result = Lib::ffi()->BuildAnimation_stamp_box($this->ptr, $source->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z, $x, $y, $z, $__view10, $duration_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setOperationGizmos( $enabled) {
        Lib::ffi()->BuildAnimation_set_operation_gizmos($this->ptr, $enabled);
    }

    public function operationsJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->BuildAnimation_operations_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function frameJson( $time_ms) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->BuildAnimation_frame_json($this->ptr, $time_ms, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function fillAlongParameter( $shape,  $brush,  $group_count) {
        $result = Lib::ffi()->BuildAnimation_fill_along_parameter($this->ptr, $shape->ptr, $brush->ptr, $group_count);
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

    public function renderVideoWithPack( $pack,  $config,  $video, string $path,  $hold_ms) {
        $__n3 = strlen($path);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $path, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->BuildAnimation_render_video_with_pack($this->ptr, $pack->ptr, $config->ptr, $video->ptr, $__view3, $hold_ms);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function renderVideo(array $pack_zip,  $config,  $video, string $path,  $hold_ms) {
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
        $__n3 = strlen($path);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $path, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->BuildAnimation_render_video($this->ptr, $__view0, $config->ptr, $video->ptr, $__view3, $hold_ms);
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
