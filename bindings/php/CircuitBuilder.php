<?php
namespace Stencil;

final class CircuitBuilder {
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

    public static function create( $schematic) {
        $ret = Lib::ffi()->CircuitBuilder_create($schematic->ptr);
        return new CircuitBuilder($ret, true);
    }

    public static function fromInsign( $schematic) {
        $result = Lib::ffi()->CircuitBuilder_from_insign($schematic->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new CircuitBuilder($result->ok, true);
    }

    public function withInput(string $name,  $io_type,  $layout, array $region_positions) {
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
        $__n3 = count($region_positions);
        $__view3 = Lib::ffi()->new('DiplomatI32View');
        if ($__n3 > 0) {
            $__arr3 = Lib::ffi()->new("int32_t[" . $__n3 . "]", false);
            foreach ($region_positions as $__i3 => $__v3) { $__arr3[$__i3] = $__v3; }
            $__view3->data = \FFI::addr($__arr3[0]);
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->CircuitBuilder_with_input($this->ptr, $__view0, $io_type->ptr, $layout->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function withInputSorted(string $name,  $io_type,  $layout, array $region_positions,  $sort) {
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
        $__n3 = count($region_positions);
        $__view3 = Lib::ffi()->new('DiplomatI32View');
        if ($__n3 > 0) {
            $__arr3 = Lib::ffi()->new("int32_t[" . $__n3 . "]", false);
            foreach ($region_positions as $__i3 => $__v3) { $__arr3[$__i3] = $__v3; }
            $__view3->data = \FFI::addr($__arr3[0]);
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->CircuitBuilder_with_input_sorted($this->ptr, $__view0, $io_type->ptr, $layout->ptr, $__view3, $sort->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function withInputAuto(string $name,  $io_type, array $region_positions) {
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
        $__n2 = count($region_positions);
        $__view2 = Lib::ffi()->new('DiplomatI32View');
        if ($__n2 > 0) {
            $__arr2 = Lib::ffi()->new("int32_t[" . $__n2 . "]", false);
            foreach ($region_positions as $__i2 => $__v2) { $__arr2[$__i2] = $__v2; }
            $__view2->data = \FFI::addr($__arr2[0]);
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->CircuitBuilder_with_input_auto($this->ptr, $__view0, $io_type->ptr, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function withInputAutoSorted(string $name,  $io_type, array $region_positions,  $sort) {
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
        $__n2 = count($region_positions);
        $__view2 = Lib::ffi()->new('DiplomatI32View');
        if ($__n2 > 0) {
            $__arr2 = Lib::ffi()->new("int32_t[" . $__n2 . "]", false);
            foreach ($region_positions as $__i2 => $__v2) { $__arr2[$__i2] = $__v2; }
            $__view2->data = \FFI::addr($__arr2[0]);
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->CircuitBuilder_with_input_auto_sorted($this->ptr, $__view0, $io_type->ptr, $__view2, $sort->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function withOutput(string $name,  $io_type,  $layout, array $region_positions) {
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
        $__n3 = count($region_positions);
        $__view3 = Lib::ffi()->new('DiplomatI32View');
        if ($__n3 > 0) {
            $__arr3 = Lib::ffi()->new("int32_t[" . $__n3 . "]", false);
            foreach ($region_positions as $__i3 => $__v3) { $__arr3[$__i3] = $__v3; }
            $__view3->data = \FFI::addr($__arr3[0]);
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->CircuitBuilder_with_output($this->ptr, $__view0, $io_type->ptr, $layout->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function withOutputSorted(string $name,  $io_type,  $layout, array $region_positions,  $sort) {
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
        $__n3 = count($region_positions);
        $__view3 = Lib::ffi()->new('DiplomatI32View');
        if ($__n3 > 0) {
            $__arr3 = Lib::ffi()->new("int32_t[" . $__n3 . "]", false);
            foreach ($region_positions as $__i3 => $__v3) { $__arr3[$__i3] = $__v3; }
            $__view3->data = \FFI::addr($__arr3[0]);
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->CircuitBuilder_with_output_sorted($this->ptr, $__view0, $io_type->ptr, $layout->ptr, $__view3, $sort->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function withOutputAuto(string $name,  $io_type, array $region_positions) {
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
        $__n2 = count($region_positions);
        $__view2 = Lib::ffi()->new('DiplomatI32View');
        if ($__n2 > 0) {
            $__arr2 = Lib::ffi()->new("int32_t[" . $__n2 . "]", false);
            foreach ($region_positions as $__i2 => $__v2) { $__arr2[$__i2] = $__v2; }
            $__view2->data = \FFI::addr($__arr2[0]);
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->CircuitBuilder_with_output_auto($this->ptr, $__view0, $io_type->ptr, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function withOutputAutoSorted(string $name,  $io_type, array $region_positions,  $sort) {
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
        $__n2 = count($region_positions);
        $__view2 = Lib::ffi()->new('DiplomatI32View');
        if ($__n2 > 0) {
            $__arr2 = Lib::ffi()->new("int32_t[" . $__n2 . "]", false);
            foreach ($region_positions as $__i2 => $__v2) { $__arr2[$__i2] = $__v2; }
            $__view2->data = \FFI::addr($__arr2[0]);
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->CircuitBuilder_with_output_auto_sorted($this->ptr, $__view0, $io_type->ptr, $__view2, $sort->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function withOptions( $optimize,  $io_only) {
        $result = Lib::ffi()->CircuitBuilder_with_options($this->ptr, $optimize, $io_only);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function withStateMode(string $mode) {
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
        $result = Lib::ffi()->CircuitBuilder_with_state_mode($this->ptr, $__view0);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function validate() {
        $result = Lib::ffi()->CircuitBuilder_validate($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function build() {
        $result = Lib::ffi()->CircuitBuilder_build($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TypedCircuitExecutor($result->ok, true);
    }

    public function buildValidated() {
        $result = Lib::ffi()->CircuitBuilder_build_validated($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new TypedCircuitExecutor($result->ok, true);
    }

    public function inputCount() {
        $ret = Lib::ffi()->CircuitBuilder_input_count($this->ptr);
        return $ret;
    }

    public function outputCount() {
        $ret = Lib::ffi()->CircuitBuilder_output_count($this->ptr);
        return $ret;
    }

    public function inputNamesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->CircuitBuilder_input_names_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function outputNamesJson() {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->CircuitBuilder_output_names_json($this->ptr, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->CircuitBuilder_destroy($this->ptr);
        }
    }
}
