<?php
namespace Stencil;

final class MeshJob {
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

    public static function start( $schematic,  $pack,  $config,  $chunk_size,  $atlas) {
        $ret = Lib::ffi()->MeshJob_start($schematic->ptr, $pack->ptr, $config->ptr, $chunk_size, $atlas->ptr);
        return new MeshJob($ret, true);
    }

    public function pollProgress() {
        $ret = Lib::ffi()->MeshJob_poll_progress($this->ptr);
        return MeshProgress::fromFFI($ret);
    }

    public function takeResult() {
        $result = Lib::ffi()->MeshJob_take_result($this->ptr);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new ChunkMeshResult($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->MeshJob_destroy($this->ptr);
        }
    }
}
