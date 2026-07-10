<?php
namespace Stencil;

final class IoLayout {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public function inputNamesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->IoLayout_input_names_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function outputNamesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->IoLayout_output_names_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->IoLayout_destroy($this->ptr);
        }
    }
}
