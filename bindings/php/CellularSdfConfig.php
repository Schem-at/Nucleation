<?php
namespace Stencil;

final class CellularSdfConfig {
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

    public static function create( $cell_size_x,  $cell_size_z,  $seed,  $max_jitter_x,  $max_jitter_z,  $max_yaw_degrees,  $min_scale,  $max_scale,  $min_y_offset,  $max_y_offset,  $presence_numerator,  $presence_denominator,  $feature_salt) {
        $result = Lib::ffi()->CellularSdfConfig_create($cell_size_x, $cell_size_z, $seed, $max_jitter_x, $max_jitter_z, $max_yaw_degrees, $min_scale, $max_scale, $min_y_offset, $max_y_offset, $presence_numerator, $presence_denominator, $feature_salt);
        if (!$result->is_ok) {
            throw new DiplomatError('NucleationError', $result->err, NucleationError::name($result->err));
        }
        return new CellularSdfConfig($result->ok, true);
    }

    public function __destruct() {
        if ($this->owned) {
            Lib::ffi()->CellularSdfConfig_destroy($this->ptr);
        }
    }
}
