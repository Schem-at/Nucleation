<?php
namespace Stencil;

final class TypedCircuitExecutor {
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

    public static function fromLayout( $world,  $layout) {
        $result = Lib::ffi()->TypedCircuitExecutor_from_layout($world->ptr, $layout->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TypedCircuitExecutor($result->ok, true);
    }

    public static function fromLayoutWithOptions( $world,  $layout,  $optimize,  $io_only) {
        $result = Lib::ffi()->TypedCircuitExecutor_from_layout_with_options($world->ptr, $layout->ptr, $optimize, $io_only);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TypedCircuitExecutor($result->ok, true);
    }

    public static function fromInsign( $schematic) {
        $result = Lib::ffi()->TypedCircuitExecutor_from_insign($schematic->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TypedCircuitExecutor($result->ok, true);
    }

    public static function fromInsignWithOptions( $schematic,  $optimize,  $io_only) {
        $result = Lib::ffi()->TypedCircuitExecutor_from_insign_with_options($schematic->ptr, $optimize, $io_only);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TypedCircuitExecutor($result->ok, true);
    }

    public function setStateMode(string $mode) {
        $__n0 = strlen($mode);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $mode, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $result = Lib::ffi()->TypedCircuitExecutor_set_state_mode($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function reset() {
        $result = Lib::ffi()->TypedCircuitExecutor_reset($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function tick( $ticks) {
        Lib::ffi()->TypedCircuitExecutor_tick($this->ptr, $ticks);
    }

    public function flush() {
        Lib::ffi()->TypedCircuitExecutor_flush($this->ptr);
    }

    public function setInput(string $name,  $value) {
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
        $result = Lib::ffi()->TypedCircuitExecutor_set_input($this->ptr, $__view0, $value->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function readOutput(string $name) {
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
        $result = Lib::ffi()->TypedCircuitExecutor_read_output($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Value($result->ok, true);
    }

    public function execute(string $inputs_json,  $mode) {
        $__n0 = strlen($inputs_json);
        $__view0 = Lib::ffi()->new('DiplomatStringView');
        if ($__n0 > 0) {
            $__buf0 = Lib::ffi()->new("uint8_t[" . $__n0 . "]", false);
            \FFI::memcpy($__buf0, $inputs_json, $__n0);
            $__view0->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf0[0]));
        } else {
            $__view0->data = null;
        }
        $__view0->len = $__n0;
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->TypedCircuitExecutor_execute($this->ptr, $__view0, $mode->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function inputNamesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TypedCircuitExecutor_input_names_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function outputNamesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TypedCircuitExecutor_output_names_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function layoutInfoJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        Lib::ffi()->TypedCircuitExecutor_layout_info_json($this->ptr, $write);
        return Lib::readAndFreeWrite($write);
    }

    public function syncToSchematic() {
        $ret = Lib::ffi()->TypedCircuitExecutor_sync_to_schematic($this->ptr);
        return new Schematic($ret, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->TypedCircuitExecutor_destroy($this->ptr);
        }
    }
}
