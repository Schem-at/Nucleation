<?php
namespace Stencil;

final class ItemModelPackBuilder {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create() {
        $ret = Lib::ffi()->ItemModelPackBuilder_create();
        return new ItemModelPackBuilder($ret, true);
    }

    public function len() {
        $ret = Lib::ffi()->ItemModelPackBuilder_len($this->ptr);
        return $ret;
    }

    public function buildZipB64() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->ItemModelPackBuilder_build_zip_b64($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->ItemModelPackBuilder_destroy($this->ptr);
        }
    }
}
