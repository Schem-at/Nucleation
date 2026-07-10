<?php
namespace Stencil;

final class RegionBounds {
    public function __construct(public int $min_x, public int $min_y, public int $min_z, public int $max_x, public int $max_y, public int $max_z) {}

    public static function fromFFI($s): self {
        return new self($s->min_x, $s->min_y, $s->min_z, $s->max_x, $s->max_y, $s->max_z);
    }

    public function toFFI() {
        $s = Lib::ffi()->new('RegionBounds');
        $s->min_x = $this->min_x;
        $s->min_y = $this->min_y;
        $s->min_z = $this->min_z;
        $s->max_x = $this->max_x;
        $s->max_y = $this->max_y;
        $s->max_z = $this->max_z;
        return $s;
    }
}
