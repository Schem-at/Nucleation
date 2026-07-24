<?php
namespace Stencil;

final class WsRunResult {
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

    public static function runDir( $job,  $hints,  $profile, string $world_dir) {
        $__n3 = strlen($world_dir);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $world_dir, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $result = Lib::ffi()->WsRunResult_run_dir($job->ptr, $hints->ptr, $profile->ptr, $__view3);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new WsRunResult($result->ok, true);
    }

    public function builds() {
        $ret = Lib::ffi()->WsRunResult_builds($this->ptr);
        return $ret;
    }

    public function tierConfident() {
        $ret = Lib::ffi()->WsRunResult_tier_confident($this->ptr);
        return $ret;
    }

    public function tierProbable() {
        $ret = Lib::ffi()->WsRunResult_tier_probable($this->ptr);
        return $ret;
    }

    public function tierDebris() {
        $ret = Lib::ffi()->WsRunResult_tier_debris($this->ptr);
        return $ret;
    }

    public function crossTile() {
        $ret = Lib::ffi()->WsRunResult_cross_tile($this->ptr);
        return $ret;
    }

    public function largestBlockCount() {
        $ret = Lib::ffi()->WsRunResult_largest_block_count($this->ptr);
        return $ret;
    }

    public function buildCount() {
        $ret = Lib::ffi()->WsRunResult_build_count($this->ptr);
        return $ret;
    }

    public function stableIdHex( $index) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->WsRunResult_stable_id_hex($this->ptr, $index, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function fingerprintHex( $index) {
        $write = Lib::ffi()->diplomat_buffer_write_create(0);
        $result = Lib::ffi()->WsRunResult_fingerprint_hex($this->ptr, $index, $write);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return Lib::readAndFreeWrite($write);
    }

    public function tierOf( $index) {
        $result = Lib::ffi()->WsRunResult_tier_of($this->ptr, $index);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function blockCountOf( $index) {
        $result = Lib::ffi()->WsRunResult_block_count_of($this->ptr, $index);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return $result->ok;
    }

    public function bboxMinOf( $index) {
        $result = Lib::ffi()->WsRunResult_bbox_min_of($this->ptr, $index);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return BlockPos::fromFFI($result->ok);
    }

    public function bboxMaxOf( $index) {
        $result = Lib::ffi()->WsRunResult_bbox_max_of($this->ptr, $index);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return BlockPos::fromFFI($result->ok);
    }

    public function writeSchemTo( $index, string $path) {
        $__n1 = strlen($path);
        $__view1 = Lib::ffi()->new('DiplomatStringView');
        if ($__n1 > 0) {
            $__buf1 = Lib::ffi()->new("uint8_t[" . $__n1 . "]", false);
            \FFI::memcpy($__buf1, $path, $__n1);
            $__view1->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf1[0]));
        } else {
            $__view1->data = null;
        }
        $__view1->len = $__n1;
        $result = Lib::ffi()->WsRunResult_write_schem_to($this->ptr, $index, $__view1);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->WsRunResult_destroy($this->ptr);
        }
    }
}
