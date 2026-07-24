<?php
namespace Stencil;

final class RenderConfig {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create( $width,  $height) {
        $ret = Lib::ffi()->RenderConfig_create($width, $height);
        return new RenderConfig($ret, true);
    }

    public function setYaw( $yaw) {
        Lib::ffi()->RenderConfig_set_yaw($this->ptr, $yaw);
    }

    public function setPitch( $pitch) {
        Lib::ffi()->RenderConfig_set_pitch($this->ptr, $pitch);
    }

    public function setZoom( $zoom) {
        Lib::ffi()->RenderConfig_set_zoom($this->ptr, $zoom);
    }

    public function setSphereFit( $sphere_fit) {
        Lib::ffi()->RenderConfig_set_sphere_fit($this->ptr, $sphere_fit);
    }

    public function setFov( $fov) {
        Lib::ffi()->RenderConfig_set_fov($this->ptr, $fov);
    }

    public function setDirectionalLight( $x,  $y,  $z,  $intensity) {
        $result = Lib::ffi()->RenderConfig_set_directional_light($this->ptr, $x, $y, $z, $intensity);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setAmbientLight( $ambient) {
        $result = Lib::ffi()->RenderConfig_set_ambient_light($this->ptr, $ambient);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setBackground( $r,  $g,  $b,  $a) {
        Lib::ffi()->RenderConfig_set_background($this->ptr, $r, $g, $b, $a);
    }

    public function clearBackground() {
        Lib::ffi()->RenderConfig_clear_background($this->ptr);
    }

    public function setGrid( $half_extent,  $spacing,  $plane_y,  $show_axes,  $red,  $green,  $blue,  $alpha) {
        Lib::ffi()->RenderConfig_set_grid($this->ptr, $half_extent, $spacing, $plane_y, $show_axes, $red, $green, $blue, $alpha);
    }

    public function setFittedGrid( $margin,  $spacing,  $plane_y,  $show_axes,  $red,  $green,  $blue,  $alpha) {
        Lib::ffi()->RenderConfig_set_fitted_grid($this->ptr, $margin, $spacing, $plane_y, $show_axes, $red, $green, $blue, $alpha);
    }

    public function clearGrid() {
        Lib::ffi()->RenderConfig_clear_grid($this->ptr);
    }

    public function setOrthographic( $orthographic) {
        Lib::ffi()->RenderConfig_set_orthographic($this->ptr, $orthographic);
    }

    public function setIsometric() {
        Lib::ffi()->RenderConfig_set_isometric($this->ptr);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->RenderConfig_destroy($this->ptr);
        }
    }
}
