<?php
namespace Stencil;

final class GeneratedWorldStream {
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

    public function remaining() {
        $ret = Lib::ffi()->GeneratedWorldStream_remaining($this->ptr);
        return $ret;
    }

    public function next() {
        $result = Lib::ffi()->GeneratedWorldStream_next($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new GeneratedChunk($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->GeneratedWorldStream_destroy($this->ptr);
        }
    }
}
