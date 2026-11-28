<?php
namespace Stencil;

final class MeshBytes {
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

    public function len() {
        $ret = Lib::ffi()->MeshBytes_len($this->ptr);
        return $ret;
    }

    public function copyInto( $offset, array $out) {
        $__n1 = count($out);
        $__view1 = Lib::ffi()->new('DiplomatU8View');
        if ($__n1 > 0) {
            $__arr1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            foreach ($out as $__i1 => $__v1) { $__arr1[$__i1] = $__v1; }
            $__view1->data = \FFI::addr($__arr1[0]);
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $ret = Lib::ffi()->MeshBytes_copy_into($this->ptr, $offset, $__view1);
        return $ret;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->MeshBytes_destroy($this->ptr);
        }
    }
}
