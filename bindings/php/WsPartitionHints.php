<?php
namespace Stencil;

final class WsPartitionHints {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create() {
        $ret = Lib::ffi()->WsPartitionHints_create();
        return new WsPartitionHints($ret, true);
    }

    public function add(string $id,  $x0,  $x1,  $z0,  $z1) {
        $__n0 = strlen($id);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $id, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->WsPartitionHints_add($this->ptr, $__view0, $x0, $x1, $z0, $z1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function len() {
        $ret = Lib::ffi()->WsPartitionHints_len($this->ptr);
        return $ret;
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->WsPartitionHints_destroy($this->ptr);
        }
    }
}
