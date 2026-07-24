<?php
namespace Stencil;

final class ResourcePackList {
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

    public static function create() {
        $ret = Lib::ffi()->ResourcePackList_create();
        return new ResourcePackList($ret, true);
    }

    public function add(array $data) {
        $__n0 = count($data);
        $__view0 = Lib::ffi()->new('DiplomatU8View');
        if ($__n0 > 0) {
            $__arr0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            foreach ($data as $__i0 => $__v0) { $__arr0[$__i0] = $__v0; }
            $__view0->data = \FFI::addr($__arr0[0]);
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        Lib::ffi()->ResourcePackList_add($this->ptr, $__view0);
    }

    public function len() {
        $ret = Lib::ffi()->ResourcePackList_len($this->ptr);
        return $ret;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->ResourcePackList_destroy($this->ptr);
        }
    }
}
