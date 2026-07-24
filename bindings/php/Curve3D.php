<?php
namespace Stencil;

final class Curve3D {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function fromPoints(array $coordinates,  $closed) {
        $__n0 = count($coordinates);
        $__view0 = Lib::ffi()->new('DiplomatF64View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("double[" . $__n0 . "]", false);
            foreach ($coordinates as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->Curve3D_from_points($__view0, $closed);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Curve3D($result->ok, true);
    }

    public function pointCount() {
        $ret = Lib::ffi()->Curve3D_point_count($this->ptr);
        return $ret;
    }

    public function isClosed() {
        $ret = Lib::ffi()->Curve3D_is_closed($this->ptr);
        return $ret;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->Curve3D_destroy($this->ptr);
        }
    }
}
