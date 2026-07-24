<?php
namespace Stencil;

final class IoLayoutBuilder {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create() {
        $ret = Lib::ffi()->IoLayoutBuilder_create();
        return new IoLayoutBuilder($ret, true);
    }

    public function addInput(string $name,  $io_type,  $layout, array $positions) {
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
        $__n3 = count($positions);
        $__view3 = Lib::ffi()->new('DiplomatI32View');
        if ($__n3 > 0) {
            $__arr3 = Lib::ffi()->new("int32_t[" . $__n3 . "]", false);
            foreach ($positions as $__i3 => $__v3) { $__arr3[$__i3] = $__v3; }
            $__view3->data = \FFI::addr($__arr3[0]);
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->IoLayoutBuilder_add_input($this->ptr, $__view0, $io_type->ptr, $layout->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addOutput(string $name,  $io_type,  $layout, array $positions) {
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
        $__n3 = count($positions);
        $__view3 = Lib::ffi()->new('DiplomatI32View');
        if ($__n3 > 0) {
            $__arr3 = Lib::ffi()->new("int32_t[" . $__n3 . "]", false);
            foreach ($positions as $__i3 => $__v3) { $__arr3[$__i3] = $__v3; }
            $__view3->data = \FFI::addr($__arr3[0]);
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->IoLayoutBuilder_add_output($this->ptr, $__view0, $io_type->ptr, $layout->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addInputAuto(string $name,  $io_type, array $positions) {
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
        $__n2 = count($positions);
        $__view2 = Lib::ffi()->new('DiplomatI32View');
        if ($__n2 > 0) {
            $__arr2 = Lib::ffi()->new("int32_t[" . $__n2 . "]", false);
            foreach ($positions as $__i2 => $__v2) { $__arr2[$__i2] = $__v2; }
            $__view2->data = \FFI::addr($__arr2[0]);
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->IoLayoutBuilder_add_input_auto($this->ptr, $__view0, $io_type->ptr, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addOutputAuto(string $name,  $io_type, array $positions) {
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
        $__n2 = count($positions);
        $__view2 = Lib::ffi()->new('DiplomatI32View');
        if ($__n2 > 0) {
            $__arr2 = Lib::ffi()->new("int32_t[" . $__n2 . "]", false);
            foreach ($positions as $__i2 => $__v2) { $__arr2[$__i2] = $__v2; }
            $__view2->data = \FFI::addr($__arr2[0]);
        } else {
            $__view2->data = null;
        }
        $__view2->len = $__n2;
        $result = Lib::ffi()->IoLayoutBuilder_add_output_auto($this->ptr, $__view0, $io_type->ptr, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addInputFromRegion(string $name,  $io_type,  $layout, array $region_positions) {
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
        $result = Lib::ffi()->IoLayoutBuilder_add_input_from_region($this->ptr, $__view0, $io_type->ptr, $layout->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addInputFromRegionAuto(string $name,  $io_type, array $region_positions) {
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
        $result = Lib::ffi()->IoLayoutBuilder_add_input_from_region_auto($this->ptr, $__view0, $io_type->ptr, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addOutputFromRegion(string $name,  $io_type,  $layout, array $region_positions) {
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
        $result = Lib::ffi()->IoLayoutBuilder_add_output_from_region($this->ptr, $__view0, $io_type->ptr, $layout->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function addOutputFromRegionAuto(string $name,  $io_type, array $region_positions) {
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
        $result = Lib::ffi()->IoLayoutBuilder_add_output_from_region_auto($this->ptr, $__view0, $io_type->ptr, $__view2);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function build() {
        $result = Lib::ffi()->IoLayoutBuilder_build($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new IoLayout($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->IoLayoutBuilder_destroy($this->ptr);
        }
    }
}
