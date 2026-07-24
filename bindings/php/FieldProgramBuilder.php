<?php
namespace Stencil;

final class FieldProgramBuilder {
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
        $ret = Lib::ffi()->FieldProgramBuilder_create();
        return new FieldProgramBuilder($ret, true);
    }

    public function addSlot(int $value_type) {
        $result = Lib::ffi()->FieldProgramBuilder_add_slot($this->ptr, $value_type);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function pushConstScalar( $value) {
        $result = Lib::ffi()->FieldProgramBuilder_push_const_scalar($this->ptr, $value);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function pushConstVec3( $x,  $y,  $z) {
        $result = Lib::ffi()->FieldProgramBuilder_push_const_vec3($this->ptr, $x, $y, $z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function pushConstBool( $value) {
        $result = Lib::ffi()->FieldProgramBuilder_push_const_bool($this->ptr, $value);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function pushPos() {
        $result = Lib::ffi()->FieldProgramBuilder_push_pos($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function loadLocal( $slot) {
        $result = Lib::ffi()->FieldProgramBuilder_load_local($this->ptr, $slot);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function storeLocal( $slot) {
        $result = Lib::ffi()->FieldProgramBuilder_store_local($this->ptr, $slot);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function pop() {
        $result = Lib::ffi()->FieldProgramBuilder_pop($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function unaryOp(int $op) {
        $result = Lib::ffi()->FieldProgramBuilder_unary_op($this->ptr, $op);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function binaryOp(int $op) {
        $result = Lib::ffi()->FieldProgramBuilder_binary_op($this->ptr, $op);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function clamp() {
        $result = Lib::ffi()->FieldProgramBuilder_clamp($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function select() {
        $result = Lib::ffi()->FieldProgramBuilder_select($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function makeVec3() {
        $result = Lib::ffi()->FieldProgramBuilder_make_vec3($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function breakIf() {
        $result = Lib::ffi()->FieldProgramBuilder_break_if($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function beginRepeat( $count) {
        $result = Lib::ffi()->FieldProgramBuilder_begin_repeat($this->ptr, $count);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function endRepeat() {
        $result = Lib::ffi()->FieldProgramBuilder_end_repeat($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setOutput( $slot) {
        $result = Lib::ffi()->FieldProgramBuilder_set_output($this->ptr, $slot);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setBounds( $min_x,  $min_y,  $min_z,  $max_x,  $max_y,  $max_z) {
        $result = Lib::ffi()->FieldProgramBuilder_set_bounds($this->ptr, $min_x, $min_y, $min_z, $max_x, $max_y, $max_z);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function setDistanceKind(int $kind) {
        $result = Lib::ffi()->FieldProgramBuilder_set_distance_kind($this->ptr, $kind);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function build() {
        $result = Lib::ffi()->FieldProgramBuilder_build($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new FieldProgram($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->FieldProgramBuilder_destroy($this->ptr);
        }
    }
}
