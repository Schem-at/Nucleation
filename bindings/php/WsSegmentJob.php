<?php
namespace Stencil;

final class WsSegmentJob {
    /** @internal */
    public \FFI\CData $ptr;
    private bool $owned;

    /** @internal */
    public function __construct(\FFI\CData $ptr, bool $owned) {
        $this->ptr = $ptr;
        $this->owned = $owned;
    }

    public static function create( $cell_size,  $closing_radius,  $min_cluster_blocks, string $source_id, string $snapshot_id,  $min_y,  $max_y,  $extracted_at,  $match_iou,  $hard_cut) {
        $__n3 = strlen($source_id);
        $__view3 = Lib::ffi()->new('DiplomatStringView');
        if ($__n3 > 0) {
            $__buf3 = Lib::ffi()->new("uint8_t[" . $__n3 . "]", false);
            \FFI::memcpy($__buf3, $source_id, $__n3);
            $__view3->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf3[0]));
        } else {
            $__view3->data = null;
        }
        $__view3->len = $__n3;
        $__n4 = strlen($snapshot_id);
        $__view4 = Lib::ffi()->new('DiplomatStringView');
        if ($__n4 > 0) {
            $__buf4 = Lib::ffi()->new("uint8_t[" . $__n4 . "]", false);
            \FFI::memcpy($__buf4, $snapshot_id, $__n4);
            $__view4->data = Lib::ffi()->cast('const char*', \FFI::addr($__buf4[0]));
        } else {
            $__view4->data = null;
        }
        $__view4->len = $__n4;
        $result = Lib::ffi()->WsSegmentJob_create($cell_size, $closing_radius, $min_cluster_blocks, $__view3, $__view4, $min_y, $max_y, $extracted_at, $match_iou, $hard_cut);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new WsSegmentJob($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->WsSegmentJob_destroy($this->ptr);
        }
    }
}
