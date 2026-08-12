<?php
namespace Stencil;

final class SchematicSplitResult {
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
        $ret = Lib::ffi()->SchematicSplitResult_len($this->ptr);
        return $ret;
    }

    public function piece( $index) {
        $result = Lib::ffi()->SchematicSplitResult_piece($this->ptr, $index);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new Schematic($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->SchematicSplitResult_destroy($this->ptr);
        }
    }
}
